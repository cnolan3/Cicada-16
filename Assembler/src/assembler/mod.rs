/*
Copyright 2025 Connor Nolan

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

mod constant_table;
mod encoder;
mod preprocessor;
mod section_stack;
mod symbol_table;

use crate::ast::{AssemblyLine, Directive, Operand};
use crate::errors::AssemblyError;
use constant_table::*;
use encoder::utility_functions::resolve_label_or_immediate;
use section_stack::*;
use symbol_table::*;

const BANK_SIZE: u32 = 16384;

/// Pass 0: build the constant table
pub fn build_constant_table(lines: &[AssemblyLine]) -> Result<ConstantTable, AssemblyError> {
    let mut constant_table = ConstantTable::new();

    for line in lines {
        // handle directives
        if let Some(directive) = &line.directive {
            match directive {
                Directive::Define(label, op) => {
                    if constant_table.contains_key(label) {
                        return Err(AssemblyError::SemanticError {
                            line: line.line_number,
                            reason: format!("Duplicate constant definition: {}", label),
                        });
                    }

                    match op {
                        Operand::Immediate(value) => {
                            constant_table.insert(label.clone(), value.clone());
                        }
                        _ => {
                            return Err(AssemblyError::SemanticError {
                                line: line.line_number,
                                reason: "Invalid value for .define statement.".to_string(),
                            });
                        }
                    }
                }
                _ => {}
            }
        }
    }

    Ok(constant_table)
}

/// Pass 0.5: Replace constant values
pub fn process_constants(
    lines: &mut [AssemblyLine],
    constant_table: &ConstantTable,
) -> Result<(), AssemblyError> {
    for line in lines {
        if let Some(instruction) = &mut line.instruction {
            preprocessor::constant::process_instruction_constants(
                instruction,
                constant_table,
                &line.line_number,
            )?;
        }

        if let Some(directive) = &mut line.directive {
            preprocessor::constant::process_directive_constants(
                directive,
                constant_table,
                &line.line_number,
            )?;
        }
    }
    Ok(())
}

/// Pass 1: Build the symbol table.
pub fn build_symbol_table(
    lines: &[AssemblyLine],
    final_logical_addr: &u16,
    expected_interrupt_table_addr: Option<u16>,
    expected_header_addr: Option<u16>,
    constant_table: &ConstantTable,
) -> Result<SymbolTable, AssemblyError> {
    let mut symbol_table = SymbolTable::new();
    let mut addr_counter: AddrCounter = AddrCounter::new();
    let mut found_interrupt_table_addr: Option<u32> = None;
    let mut found_header_addr: Option<u32> = None;
    let mut context_stack: ContextStack = vec![];

    for line in lines {
        // If a label exists on this line, record its current address.
        if let Some(label) = &line.label {
            if symbol_table.contains_key(label) {
                return Err(AssemblyError::SemanticError {
                    line: line.line_number,
                    reason: format!("Duplicate label definition: {}", label),
                });
            }

            // check to see if there is already a constant defined with the same name
            if constant_table.contains_key(label) {
                return Err(AssemblyError::SemanticError {
                    line: line.line_number,
                    reason: format!("Label already defined as a constant: {}", label),
                });
            }

            // let logical_address = match current_bank {
            //     0 => physical_address,
            //     _ => BANK_SIZE + (physical_address % BANK_SIZE),
            // };

            symbol_table.insert(
                label.clone(),
                Symbol {
                    logical_address: addr_counter.logical_addr,
                    bank: addr_counter.bank,
                },
            );
        }

        // Increment physical_address by the size of the instruction.
        if let Some(instruction) = &line.instruction {
            let instruction_size = encoder::calculate_instruction_size(instruction);
            addr_counter.increment_by(instruction_size);
        }

        // handle directives
        if let Some(directive) = &line.directive {
            match directive {
                Directive::Org(Operand::Immediate(addr)) => {
                    addr_counter.logical_addr = *addr as u32;

                    if addr_counter.logical_addr > *final_logical_addr as u32 {
                        return Err(AssemblyError::SemanticError {
                            line: line.line_number,
                            reason: format!(
                                ".org directive cannot move beyond the final logical address for a rom (final addr: 0x{:04x}).",
                                final_logical_addr
                            ),
                        });
                    }

                    if addr_counter.bank == 0 && addr_counter.logical_addr > 0x3FFF {
                        return Err(AssemblyError::SemanticError {
                            line: line.line_number,
                            reason: format!(
                                "Currently selected bank is bank 0, the given .org address (0x{:04x}) is outside of the bank 0 fixed address space 0x0000-0x3FFF.",
                                addr_counter.logical_addr
                            ),
                        });
                    }

                    if addr_counter.bank != 0 && addr_counter.logical_addr < 0x4000 {
                        return Err(AssemblyError::SemanticError {
                            line: line.line_number,
                            reason: format!(
                                "Currently selected bank is bank {} (a switchable bank), the given .org address (0x{:04x}) is outside of the switchable bank address space 0x4000-0x7FFF.",
                                addr_counter.bank, addr_counter.logical_addr
                            ),
                        });
                    }

                    // calculate the new physical address
                    let new_physical_addr = calculate_physical_addr(
                        &(addr_counter.logical_addr as u16),
                        &addr_counter.bank,
                    );

                    // It's good practice to ensure .org doesn't move backwards,
                    // as it can overwrite previous label definitions.
                    if new_physical_addr < addr_counter.physical_addr {
                        return Err(AssemblyError::SemanticError {
                            line: line.line_number,
                            reason: ".org directive cannot move the address backwards.".to_string(),
                        });
                    }

                    addr_counter.num_bytes += new_physical_addr - addr_counter.physical_addr;
                    addr_counter.physical_addr = new_physical_addr;
                }
                Directive::Bank(Operand::Immediate(num)) => {
                    if (*num as u32) < addr_counter.bank {
                        return Err(AssemblyError::SemanticError {
                            line: line.line_number,
                            reason: ".bank directive cannot move to a previous bank.".to_string(),
                        });
                    }

                    // if trying to select the already currently selected bank, do nothing
                    if *num as u32 != addr_counter.bank {
                        addr_counter.bank = *num as u32;
                        let new_physical_addr = addr_counter.bank * BANK_SIZE;
                        addr_counter.num_bytes += new_physical_addr - addr_counter.physical_addr;
                        addr_counter.physical_addr = new_physical_addr;
                        addr_counter.logical_addr = 0;
                    }
                }
                Directive::Byte(bytes) => {
                    let num_bytes = bytes.len() as u32;
                    addr_counter.increment_by(num_bytes);
                }
                Directive::Word(words) => {
                    let num_bytes = (words.len() as u32) * 2;
                    addr_counter.increment_by(num_bytes);
                }
                Directive::Header(_) => {
                    if let None = expected_header_addr {
                        return Err(AssemblyError::StructuralError {
                            line: line.line_number,
                            reason: "Cartridge rom header not allowed in boot roms.".to_string(),
                        });
                    }
                    found_header_addr = Some(addr_counter.physical_addr);
                    addr_counter.increment_by(96);
                }
                Directive::Interrupt(_) => {
                    if let None = expected_interrupt_table_addr {
                        return Err(AssemblyError::StructuralError {
                            line: line.line_number,
                            reason: "Interrupt vector table not expected.".to_string(),
                        });
                    }
                    found_interrupt_table_addr = Some(addr_counter.physical_addr);
                    addr_counter.increment_by(32);
                }
                Directive::SectionStart(section_options) => {
                    // disallow nested sections for now
                    if !context_stack.is_empty() {
                        let mut name: String = "UNNAMED".to_string();

                        let old_context: Context = context_stack.pop().unwrap();

                        if let Some(n) = old_context.name {
                            name = n;
                        }

                        return Err(AssemblyError::StructuralError {
                            line: line.line_number,
                            reason: format!(
                                "Cannot nest a section within a section, already within the \"{}\" section.",
                                name
                            ),
                        });
                    }

                    let new_context: Context = Context {
                        name: section_options.name.clone(),
                        size: section_options.size,
                        vaddr: section_options.vaddr,
                        paddr: section_options.paddr,
                        // align: section_options.align,
                        address: addr_counter.clone(),
                    };

                    // reset section size counter
                    addr_counter.num_bytes = 0;

                    // set logical address
                    if let Some(vaddr) = new_context.vaddr {
                        addr_counter.logical_addr = vaddr;
                    }

                    // set physical address
                    if let Some(paddr) = new_context.paddr {
                        addr_counter.physical_addr = paddr;
                    }

                    // TODO: set alignment
                    // if let Some(align) = new_context.align {
                    //  ...
                    // }

                    context_stack.push(new_context);
                }
                Directive::SectionEnd => {
                    if context_stack.is_empty() {
                        return Err(AssemblyError::StructuralError {
                            line: line.line_number,
                            reason: ".section_end found without a preceding .section statement."
                                .to_string(),
                        });
                    }

                    let old_context: Context = context_stack.pop().unwrap();
                    let mut name: String = "UNNAMED".to_string();

                    if let Some(n) = old_context.name {
                        name = n;
                    }

                    if old_context.vaddr.is_some() {
                        addr_counter.logical_addr =
                            calculate_logical_addr(&addr_counter.physical_addr);
                    }

                    if let Some(size) = old_context.size {
                        if addr_counter.num_bytes > size {
                            return Err(AssemblyError::StructuralError {
                                line: line.line_number,
                                reason: format!(
                                    "Section \"{}\" larger than the allotted section size of {} bytes, ({} bytes)",
                                    name, size, addr_counter.num_bytes
                                ),
                            });
                        }

                        let pad_size = size - addr_counter.num_bytes;

                        addr_counter.increment_by(pad_size);
                    }

                    // restore the old num_bytes value
                    addr_counter.num_bytes += old_context.address.num_bytes;
                }
                _ => {}
            }
        }

        // check for overflow of current bank
        let cur_bank_end = (addr_counter.bank as u32 + 1) * BANK_SIZE;
        if addr_counter.physical_addr > cur_bank_end {
            return Err(AssemblyError::StructuralError {
                line: line.line_number,
                reason: format!("ROM bank {} overflow.", addr_counter.bank),
            });
        }
    }

    // check for unclosed sections
    if !context_stack.is_empty() {
        let left_over = context_stack.pop().unwrap();

        let mut name: String = "UNNAMED".to_string();

        if let Some(n) = left_over.name {
            name = n;
        }

        return Err(AssemblyError::StructuralErrorNoLine {
            reason: format!(
                "section \"{}\" has no matching .section_end statement.",
                name
            ),
        });
    }

    // check for correct header placement
    match (expected_header_addr, found_header_addr) {
        (Some(ex_addr), Some(found_addr)) => {
            if ex_addr as u32 != found_addr {
                return Err(AssemblyError::StructuralErrorNoLine {
                    reason: format!(
                        "Expected cartridge rom header at 0x{:04x}, found at 0x{:04x}",
                        ex_addr, found_addr
                    ),
                });
            }
        }
        (Some(ex_addr), None) => {
            return Err(AssemblyError::StructuralErrorNoLine {
                reason: format!("Expected cartridge rom header at 0x{:04x}", ex_addr),
            });
        }
        _ => {}
    }

    // check for correct interrupt vector table placement
    match (expected_interrupt_table_addr, found_interrupt_table_addr) {
        (Some(ex_addr), Some(found_addr)) => {
            if ex_addr as u32 != found_addr {
                return Err(AssemblyError::StructuralErrorNoLine {
                    reason: format!(
                        "Expected interrupt vector table at 0x{:04x}, found at 0x{:04x}",
                        ex_addr, found_addr
                    ),
                });
            }
        }
        (Some(ex_addr), None) => {
            return Err(AssemblyError::StructuralErrorNoLine {
                reason: format!("Expected interrupt vector table at 0x{:04x}", ex_addr),
            });
        }
        _ => {}
    }

    Ok(symbol_table)
}

/// Pass 2: Generate machine code.
pub fn generate_bytecode(
    lines: &[AssemblyLine],
    symbol_table: &SymbolTable,
) -> Result<Vec<u8>, AssemblyError> {
    let mut bytecode = Vec::new();
    let mut addr_counter = AddrCounter::new();
    let mut context_stack: ContextStack = vec![];

    for line in lines {
        if let Some(directive) = &line.directive {
            match directive {
                Directive::Org(Operand::Immediate(addr)) => {
                    let new_addr = *addr as u16;
                    let new_physical_addr = calculate_physical_addr(&new_addr, &addr_counter.bank);
                    if new_physical_addr > addr_counter.physical_addr {
                        let padding_size =
                            (new_physical_addr - addr_counter.physical_addr) as usize;
                        bytecode.resize(bytecode.len() + padding_size, 0x00);
                        addr_counter.logical_addr += padding_size as u32;
                        addr_counter.num_bytes += padding_size as u32;
                        addr_counter.physical_addr = new_physical_addr;
                    }
                }
                Directive::Bank(Operand::Immediate(num)) => {
                    let new_addr = *num as u32 * BANK_SIZE;
                    if new_addr > addr_counter.physical_addr {
                        let padding_size = (new_addr - addr_counter.physical_addr) as usize;
                        bytecode.resize(bytecode.len() + padding_size, 0x00);
                        addr_counter.num_bytes += padding_size as u32;
                    }

                    addr_counter.bank = *num as u32;
                    addr_counter.physical_addr = new_addr;
                    addr_counter.logical_addr = 0;
                }
                Directive::Byte(bytes) => {
                    let byte_vec: Vec<u8> = bytes
                        .iter()
                        .flat_map(|byte| -> Vec<u8> {
                            match byte {
                                Operand::Immediate(byte_data) => {
                                    vec![*byte_data as u8]
                                }
                                _ => vec![], // Should be unreachable
                            }
                        })
                        .collect();
                    addr_counter.increment_by(byte_vec.len() as u32);
                    bytecode.extend(byte_vec);
                }
                Directive::Word(words) => {
                    let word_bytes: Vec<u8> = words
                        .iter()
                        .flat_map(|word| -> Vec<u8> {
                            match word {
                                Operand::Immediate(word_data) => {
                                    (*word_data as u16).to_le_bytes().to_vec()
                                }
                                Operand::Label(label_name) => {
                                    // This unwrap is safe because symbols are validated in pass 1
                                    let sym =
                                        get_symbol(symbol_table, label_name, &line.line_number)
                                            .unwrap();
                                    (sym.logical_address as u16).to_le_bytes().to_vec()
                                }
                                _ => vec![], // Should be unreachable
                            }
                        })
                        .collect();
                    addr_counter.increment_by(word_bytes.len() as u32);
                    bytecode.extend(word_bytes);
                }
                Directive::Header(info) => {
                    let mut header: Vec<u8> = Vec::new();

                    header.extend(info.boot_anim.as_bytes());

                    header.extend(info.title.as_bytes());
                    if header.len() < 0x14 {
                        header.resize(0x14, 0x00);
                    }

                    header.extend(info.developer.as_bytes());
                    if header.len() < 0x24 {
                        header.resize(0x24, 0x00);
                    }

                    header.push(info.version);

                    header.push(info.rom_size);

                    header.push(info.ram_size);

                    let mut cart_info: u8 = info.hardware_rev & 0x3;
                    cart_info = (cart_info << 3) | (info.region & 0x7);
                    cart_info = cart_info << 3;
                    header.push(cart_info);

                    let mut features: u8 = info.interrupt_mode & 0x1;
                    features = (features << 2) | (info.mapper & 0x3);
                    features = features << 5;
                    header.push(features);

                    header.resize(0x60, 0x00);
                    // checksums will be caclculated and added later

                    addr_counter.increment_by(header.len() as u32);
                    bytecode.extend(header);
                }
                Directive::Interrupt(words) => {
                    let mut word_bytes: Vec<u8> = words
                        .iter()
                        .flat_map(|word| -> Vec<u8> {
                            let addr =
                                resolve_label_or_immediate(word, symbol_table, &line.line_number)
                                    .unwrap();
                            addr.to_le_bytes().to_vec()
                        })
                        .collect();

                    if word_bytes.len() < 32 {
                        word_bytes.resize(32, 0x00);
                    }
                    addr_counter.increment_by(word_bytes.len() as u32);
                    bytecode.extend(word_bytes);
                }
                Directive::SectionStart(section_options) => {
                    let new_context: Context = Context {
                        name: section_options.name.clone(),
                        size: section_options.size,
                        vaddr: section_options.vaddr,
                        paddr: section_options.paddr,
                        // align: section_options.align,
                        address: addr_counter.clone(),
                    };

                    // reset section size counter
                    addr_counter.num_bytes = 0;

                    // set logical address
                    if let Some(vaddr) = new_context.vaddr {
                        addr_counter.logical_addr = vaddr;
                    }

                    // set physical address
                    if let Some(paddr) = new_context.paddr {
                        if paddr > addr_counter.physical_addr {
                            let padding_size = (paddr - addr_counter.physical_addr) as usize;

                            bytecode.resize(bytecode.len() + padding_size, 0x00);
                        }
                        addr_counter.physical_addr = paddr;
                    }

                    // TODO: set alignment
                    // if let Some(align) = new_context.align {
                    //  ...
                    // }

                    context_stack.push(new_context);
                }
                Directive::SectionEnd => {
                    if context_stack.is_empty() {
                        return Err(AssemblyError::StructuralError {
                            line: line.line_number,
                            reason: ".section_end found without a preceding .section statement."
                                .to_string(),
                        });
                    }

                    let old_context: Context = context_stack.pop().unwrap();

                    if old_context.vaddr.is_some() {
                        addr_counter.logical_addr =
                            calculate_logical_addr(&addr_counter.physical_addr);
                    }

                    if let Some(size) = old_context.size {
                        if size > addr_counter.num_bytes {
                            let padding_size = size - addr_counter.num_bytes;
                            bytecode.resize(bytecode.len() + padding_size as usize, 0x00);

                            addr_counter.increment_by(padding_size);
                        }
                    }
                }
                _ => {}
            }
        }

        if let Some(instruction) = &line.instruction {
            let instruction_bytes = encoder::encode_instruction(
                instruction,
                symbol_table,
                &addr_counter.logical_addr,
                &addr_counter.bank,
                &line.line_number,
            )?;
            addr_counter.increment_by(instruction_bytes.len() as u32);
            bytecode.extend(instruction_bytes);
        }
    }

    // pad the resulting bytecode to the next bank size
    let mut num_banks = bytecode.len() as u32 / BANK_SIZE;

    num_banks = if bytecode.len() as u32 % BANK_SIZE > 0 {
        num_banks + 1
    } else {
        num_banks
    };

    num_banks = std::cmp::max(num_banks, 2);

    bytecode.resize((num_banks * BANK_SIZE) as usize, 0xFF);

    // final bytecode
    Ok(bytecode)
}

fn calculate_physical_addr(logical_addr: &u16, bank: &u32) -> u32 {
    if *bank <= 1 {
        *logical_addr as u32
    } else {
        (*bank as u32 * BANK_SIZE) + (*logical_addr as u32 - 0x4000)
    }
}

fn calculate_logical_addr(physical_addr: &u32) -> u32 {
    if *physical_addr < BANK_SIZE {
        *physical_addr
    } else {
        (*physical_addr % BANK_SIZE) + BANK_SIZE
    }
}
