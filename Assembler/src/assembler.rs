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

use crate::ast::{AssemblyLine, ConditionCode, Directive, Instruction, Operand, Register};
use crate::errors::AssemblyError;
use std::collections::HashMap;

const BANK_SIZE: u32 = 16384;

#[derive(Debug, PartialEq)]
pub struct Symbol {
    logical_address: u32,
    bank: u32,
}

// The symbol table stores label names and their calculated addresses.
type SymbolTable = HashMap<String, Symbol>;

/// Pass 1: Build the symbol table.
pub fn build_symbol_table(
    lines: &[AssemblyLine],
    start_addr: &u16,
) -> Result<SymbolTable, AssemblyError> {
    let mut symbol_table = SymbolTable::new();
    let mut current_address: u32 = start_addr.clone() as u32; // Start address after cartridge header
    let mut current_bank: u32 = 0;

    for line in lines {
        // If a label exists on this line, record its current address.
        if let Some(label) = &line.label {
            if symbol_table.contains_key(label) {
                return Err(AssemblyError::SemanticError {
                    line: line.line_number,
                    reason: format!("Duplicate label definition: {}", label),
                });
            }

            let logical_address = match current_bank {
                0 => current_address,
                _ => BANK_SIZE + (current_address % BANK_SIZE),
            };

            symbol_table.insert(
                label.clone(),
                Symbol {
                    logical_address,
                    bank: current_bank,
                },
            );
        }

        // Increment current_address by the size of the instruction.
        if let Some(instruction) = &line.instruction {
            current_address += calculate_instruction_size(instruction, line.line_number)?;
        }

        // handle directives
        if let Some(directive) = &line.directive {
            match directive {
                Directive::Org(Operand::Immediate(addr)) => {
                    // It's good practice to ensure .org doesn't move backwards,
                    // as it can overwrite previous label definitions.
                    let new_addr = *addr as u32;
                    if new_addr < current_address {
                        return Err(AssemblyError::SemanticError {
                            line: line.line_number,
                            reason: ".org directive cannot move the address backwards.".to_string(),
                        });
                    }
                    current_address = new_addr;
                    current_bank = current_address / BANK_SIZE;
                }
                Directive::Bank(Operand::Immediate(num)) => {
                    if *num as u32 <= current_bank {
                        return Err(AssemblyError::SemanticError {
                            line: line.line_number,
                            reason: ".bank directive cannot move to a previous bank.".to_string(),
                        });
                    }
                    current_bank = *num as u32;
                    current_address = current_bank * BANK_SIZE;
                }
                Directive::Byte(bytes) => {
                    current_address += bytes.len() as u32;
                }
                Directive::Word(words) => {
                    current_address += (words.len() as u32) * 2;
                }
                _ => {}
            }
        }

        // check for overflow of current bank
        let cur_bank_end = (current_bank as u32 + 1) * BANK_SIZE;
        if current_address > cur_bank_end {
            return Err(AssemblyError::StructuralError {
                line: line.line_number,
                reason: format!("ROM bank {} overflow.", current_bank),
            });
        }
    }
    Ok(symbol_table)
}

/// Helper function to determine instruction size in bytes during Pass 1.
fn calculate_instruction_size(
    instruction: &Instruction,
    line_num: usize,
) -> Result<u32, AssemblyError> {
    match instruction {
        Instruction::Nop
        | Instruction::Halt
        | Instruction::Ret
        | Instruction::Ccf
        | Instruction::Scf
        | Instruction::Rcf
        | Instruction::Enter
        | Instruction::Leave
        | Instruction::Reti
        | Instruction::Ei
        | Instruction::Di
        | Instruction::Inc(_)
        | Instruction::Dec(_) => Ok(1),
        Instruction::St(Operand::Absolute(_), Operand::Register(_)) => Ok(3),
        Instruction::St(Operand::Indirect(_), Operand::Register(_)) => Ok(2),
        Instruction::Ld(Operand::Register(_), Operand::Register(_)) => {
            // Check if this form maps to the 1-byte LD rd, rs (opcodes 0x80-0xBF)
            Ok(1)
        }
        Instruction::Ld(_, Operand::Immediate(_)) => {
            // LDI r, n16 (3 bytes) or LDI.b r, n8 (3 bytes, prefixed)
            // For simplicity in early stages, assume LDI r, n16.
            Ok(3)
        }
        Instruction::Ld(_, Operand::Label(_)) => {
            // LD r, (n16) where n16 comes from a label. Size is 3 bytes.
            Ok(3)
        }
        Instruction::Ld(Operand::Register(_), Operand::Indexed(_, _)) => Ok(3),
        Instruction::Ld(Operand::Register(_), Operand::PostIncrement(_)) => Ok(3),
        Instruction::Ld(Operand::Register(_), Operand::PreDecrement(_)) => Ok(3),
        Instruction::Ldb(Operand::Register(_), Operand::Immediate(_)) => Ok(3),
        Instruction::Ldb(Operand::Register(_), Operand::Indirect(_)) => Ok(2),
        Instruction::Ldb(Operand::Register(_), Operand::PostIncrement(_)) => Ok(3),
        Instruction::Ldb(Operand::Register(_), Operand::PreDecrement(_)) => Ok(3),
        Instruction::St(Operand::Indexed(_, _), Operand::Register(_)) => Ok(3),
        Instruction::St(Operand::PostIncrement(_), Operand::Register(_)) => Ok(3),
        Instruction::St(Operand::PreDecrement(_), Operand::Register(_)) => Ok(3),
        Instruction::Stb(Operand::Indirect(_), Operand::Register(_)) => Ok(2),
        Instruction::Stb(Operand::PostIncrement(_), Operand::Register(_)) => Ok(3),
        Instruction::Stb(Operand::PreDecrement(_), Operand::Register(_)) => Ok(3),
        Instruction::Ld(Operand::Register(_), Operand::Absolute(_)) => Ok(3),
        Instruction::Lea(Operand::Register(_), Operand::Indexed(_, _)) => Ok(3),
        Instruction::Jmp(Operand::Label(_)) | Instruction::Jmp(Operand::Immediate(_)) => Ok(3), // JMP n16
        Instruction::Jmp(Operand::Indirect(_)) => Ok(1),
        Instruction::Jr(Operand::Label(_)) | Instruction::Jr(Operand::Immediate(_)) => Ok(2), // JR n8s
        Instruction::Jcc(_, Operand::Label(_)) | Instruction::Jcc(_, Operand::Immediate(_)) => {
            Ok(3)
        }
        Instruction::Jrcc(_, Operand::Label(_)) | Instruction::Jrcc(_, Operand::Immediate(_)) => {
            Ok(2)
        }
        Instruction::Djnz(Operand::Label(_)) | Instruction::Djnz(Operand::Immediate(_)) => Ok(2),
        Instruction::Call(Operand::Label(_)) | Instruction::Call(Operand::Immediate(_)) => Ok(3), // CALL n16
        Instruction::Call(Operand::Indirect(_)) => Ok(1), // CALL (r)
        Instruction::Callcc(_, _) => Ok(3),               // CALLcc n16
        Instruction::Syscall(_) => Ok(2),                 // SYSCALL n8
        Instruction::Add(Operand::Register(_), Some(Operand::Register(_))) => Ok(2),
        Instruction::Add(Operand::Register(_), Some(Operand::Immediate(_))) => Ok(4),
        Instruction::Sub(Operand::Register(_), Some(Operand::Immediate(_))) => Ok(4),
        Instruction::And(Operand::Register(_), Some(Operand::Immediate(_))) => Ok(4),
        Instruction::Or(Operand::Register(_), Some(Operand::Immediate(_))) => Ok(4),
        Instruction::Xor(Operand::Register(_), Some(Operand::Immediate(_))) => Ok(4),
        Instruction::Cmp(Operand::Register(_), Some(Operand::Immediate(_))) => Ok(4),
        Instruction::Add(Operand::Register(_), None) => Ok(1),
        Instruction::Add(Operand::Immediate(_), None) => Ok(3),
        Instruction::Add(Operand::Label(_), None) => Ok(3),
        Instruction::AddSp(_) => Ok(2),
        Instruction::Sub(Operand::Register(_), Some(Operand::Register(_))) => Ok(2),
        Instruction::Sub(Operand::Register(_), None) => Ok(1),
        Instruction::Sub(Operand::Immediate(_), None) => Ok(3),
        Instruction::Sub(Operand::Label(_), None) => Ok(3),
        Instruction::And(Operand::Register(_), Some(Operand::Register(_))) => Ok(2),
        Instruction::Or(Operand::Register(_), Some(Operand::Register(_))) => Ok(2),
        Instruction::Xor(Operand::Register(_), Some(Operand::Register(_))) => Ok(2),
        Instruction::Cmp(Operand::Register(_), Some(Operand::Register(_))) => Ok(2),
        Instruction::Adc(Operand::Register(_), Some(Operand::Register(_))) => Ok(2),
        Instruction::Sbc(Operand::Register(_), Some(Operand::Register(_))) => Ok(2),
        Instruction::And(Operand::Register(_), None) => Ok(1),
        Instruction::And(Operand::Immediate(_), None) => Ok(3),
        Instruction::And(Operand::Label(_), None) => Ok(3),
        Instruction::Or(Operand::Register(_), None) => Ok(1),
        Instruction::Or(Operand::Immediate(_), None) => Ok(3),
        Instruction::Or(Operand::Label(_), None) => Ok(3),
        Instruction::Xor(Operand::Register(_), None) => Ok(1),
        Instruction::Xor(Operand::Immediate(_), None) => Ok(3),
        Instruction::Xor(Operand::Label(_), None) => Ok(3),
        Instruction::Cmp(Operand::Register(_), None) => Ok(1),
        Instruction::Cmp(Operand::Immediate(_), None) => Ok(3),
        Instruction::Cmp(Operand::Label(_), None) => Ok(3),
        Instruction::Adc(Operand::Immediate(_), None) => Ok(3),
        Instruction::Adc(Operand::Label(_), None) => Ok(3),
        Instruction::Sbc(Operand::Immediate(_), None) => Ok(3),
        Instruction::Sbc(Operand::Label(_), None) => Ok(3),
        Instruction::Push(Operand::Register(_)) => Ok(1),
        Instruction::Push(Operand::Immediate(_)) => Ok(3),
        Instruction::Push(Operand::Label(_)) => Ok(3),
        Instruction::PushF => Ok(1),
        Instruction::Pop(Operand::Register(_)) => Ok(1),
        Instruction::PopF => Ok(1),
        Instruction::Neg => Ok(1),
        Instruction::Not => Ok(1),
        Instruction::Swap => Ok(1),
        Instruction::Sra(_)
        | Instruction::Shl(_)
        | Instruction::Shr(_)
        | Instruction::Rol(_)
        | Instruction::Ror(_)
        | Instruction::Addb(_)
        | Instruction::Subb(_)
        | Instruction::Andb(_)
        | Instruction::Orb(_)
        | Instruction::Xorb(_)
        | Instruction::Cmpb(_) => Ok(2),
        Instruction::Bit(Operand::Register(_), _)
        | Instruction::Set(Operand::Register(_), _)
        | Instruction::Res(Operand::Register(_), _) => Ok(3),
        Instruction::Bit(Operand::Absolute(_), _)
        | Instruction::Set(Operand::Absolute(_), _)
        | Instruction::Res(Operand::Absolute(_), _) => Ok(4),
        Instruction::Bit(Operand::Indirect(_), _)
        | Instruction::Set(Operand::Indirect(_), _)
        | Instruction::Res(Operand::Indirect(_), _) => Ok(3),
        Instruction::CallFar(Operand::Label(_), None) => Ok(8),
        Instruction::CallFar(Operand::Label(_), Some(Operand::Label(_))) => Ok(9),
        Instruction::JmpFar(Operand::Label(_), None) => Ok(8),
        Instruction::JmpFar(Operand::Label(_), Some(Operand::Label(_))) => Ok(9),
        // ... add logic for every instruction variant based on your opcode map ...
        _ => Err(AssemblyError::SemanticError {
            line: line_num,
            reason: "Invalid Instruction.".to_string(),
        }),
    }
}

/// Pass 2: Generate machine code.
pub fn generate_bytecode(
    lines: &[AssemblyLine],
    symbol_table: &SymbolTable,
    start_addr: &u16,
) -> Result<Vec<u8>, AssemblyError> {
    let mut bytecode = Vec::new();
    let mut current_address: u32 = start_addr.clone() as u32; // Start address after cartridge header
    let mut current_bank: u32 = 0;

    for line in lines {
        if let Some(directive) = &line.directive {
            match directive {
                Directive::Org(Operand::Immediate(addr)) => {
                    let new_addr = *addr as u32;
                    if new_addr > current_address {
                        let padding_size = (new_addr - current_address) as usize;
                        bytecode.resize(bytecode.len() + padding_size, 0x00);
                    }
                    current_address = new_addr;
                }
                Directive::Bank(Operand::Immediate(num)) => {
                    let new_addr = *num as u32 * BANK_SIZE;
                    if new_addr > current_address {
                        let padding_size = (new_addr - current_address) as usize;
                        bytecode.resize(bytecode.len() + padding_size, 0x00);
                    }

                    current_bank = *num as u32;
                    current_address = new_addr;
                }
                Directive::Byte(bytes) => {
                    let mut data_block: Vec<u8> = Vec::new();
                    for byte in bytes {
                        if let Operand::Immediate(byte_data) = byte {
                            data_block.push(*byte_data as u8);
                        }
                    }
                    current_address += data_block.len() as u32;
                    bytecode.extend(data_block);
                }
                Directive::Word(words) => {
                    let mut data_block: Vec<u8> = Vec::new();
                    for word in words {
                        match word {
                            Operand::Immediate(word_data) => {
                                let [low, high] = (*word_data as u16).to_le_bytes();
                                data_block.push(low);
                                data_block.push(high);
                            }
                            Operand::Label(label_name) => {
                                let sym = get_symbol(symbol_table, label_name, line.line_number)?;
                                let [low, high] = (sym.logical_address as u16).to_le_bytes();
                                data_block.push(low);
                                data_block.push(high);
                            }
                            _ => {}
                        }
                    }
                    current_address += data_block.len() as u32;
                    bytecode.extend(data_block);
                }
                _ => {}
            }
        }

        if let Some(instruction) = &line.instruction {
            let instruction_bytes = encode_instruction(
                instruction,
                symbol_table,
                &current_address,
                &current_bank,
                line.line_number,
            )?;
            current_address += instruction_bytes.len() as u32;
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

// helper function to encode a register operand
fn encode_register_operand(reg: &Register) -> u8 {
    match reg {
        Register::R0 => 0,
        Register::R1 => 1,
        Register::R2 => 2,
        Register::R3 => 3,
        Register::R4 => 4,
        Register::R5 => 5,
        Register::R6 => 6,
        Register::R7 => 7,
    }
}

// help function to encode condition code opcode
fn encode_condition_code_opcode(base_opcode: u8, cc: &ConditionCode) -> u8 {
    let cc_offset = match cc {
        ConditionCode::V => 0,
        ConditionCode::Nv => 1,
        ConditionCode::N => 2,
        ConditionCode::Nn => 3,
        ConditionCode::C => 4,
        ConditionCode::Nc => 5,
        ConditionCode::Z => 6,
        ConditionCode::Nz => 7,
    };

    base_opcode + cc_offset
}

fn get_symbol<'a>(
    symbol_table: &'a SymbolTable,
    label_name: &String,
    line_num: usize,
) -> Result<&'a Symbol, AssemblyError> {
    let target_symbol =
        symbol_table
            .get(label_name)
            .ok_or_else(|| AssemblyError::SemanticError {
                line: line_num,
                reason: format!("Undefined label: {}", label_name),
            })?;

    Ok(target_symbol)
}

fn get_and_check_symbol<'a>(
    symbol_table: &'a SymbolTable,
    label_name: &String,
    line_num: usize,
    current_bank: &u32,
) -> Result<&'a Symbol, AssemblyError> {
    let target_symbol = get_symbol(symbol_table, label_name, line_num)?;

    if target_symbol.bank != *current_bank {
        return Err(AssemblyError::SemanticError {
            line: line_num,
            reason: format!(
                "Label \"{}\" exists in a different bank than the current instruction.",
                label_name
            ),
        });
    }

    Ok(target_symbol)
}

/// Helper function to translate a single instruction into bytes during Pass 2.
fn encode_instruction(
    instruction: &Instruction,
    symbol_table: &SymbolTable,
    current_address: &u32,
    current_bank: &u32,
    line_num: usize,
) -> Result<Vec<u8>, AssemblyError> {
    match instruction {
        // no op (0x00)
        Instruction::Nop => Ok(vec![0x00]),
        // Halt
        Instruction::Halt => Ok(vec![0x0F]),
        // Ccf
        Instruction::Ccf => Ok(vec![0x4B]),
        // Scf
        Instruction::Scf => Ok(vec![0x4C]),
        // Rcf
        Instruction::Rcf => Ok(vec![0x4D]),
        Instruction::Enter => Ok(vec![0x4F]),
        Instruction::Leave => Ok(vec![0x50]),
        Instruction::Ret => Ok(vec![0xF9]),
        Instruction::Reti => Ok(vec![0xFA]),
        Instruction::Ei => Ok(vec![0xFB]),
        Instruction::Di => Ok(vec![0xFC]),
        // inc
        Instruction::Inc(Operand::Register(reg)) => {
            let opcode = encode_reg_opcode(0xE1, reg);
            Ok(vec![opcode])
        }
        // dec
        Instruction::Dec(Operand::Register(reg)) => {
            let opcode = encode_reg_opcode(0xD9, reg);
            Ok(vec![opcode])
        }
        // Example: LDI R1, 0xABCD (Opcode: 0x01 + register index)
        Instruction::Ld(Operand::Register(reg), Operand::Immediate(value)) => {
            let bytecode = encode_ldi(reg, *value as u16);
            Ok(bytecode)
        }
        // Example: JMP my_label
        Instruction::Jmp(Operand::Label(label_name)) => {
            let target_symbol =
                get_and_check_symbol(symbol_table, label_name, line_num, current_bank)?;
            let [low, high] = (target_symbol.logical_address as u16).to_le_bytes();
            Ok(vec![0x51, low, high]) // Opcode for JMP n16
        }
        // jump address
        Instruction::Jmp(Operand::Immediate(addr)) => {
            let [low, high] = (*addr as u16).to_le_bytes();
            Ok(vec![0x51, low, high]) // Opcode for JMP n16
        }
        // jump indirect
        Instruction::Jmp(Operand::Indirect(reg)) => {
            let opcode = encode_reg_opcode(0x52, reg);
            Ok(vec![opcode]) // Opcode for JMP n16
        }
        // jump relative immediate
        Instruction::Jr(Operand::Immediate(imm)) => {
            let rel = *imm as i8;
            Ok(vec![0x5A, rel as u8]) // Opcode for JMP n16
        }
        // jump relative label
        Instruction::Jr(Operand::Label(label_name)) => {
            let target_symbol =
                get_and_check_symbol(symbol_table, label_name, line_num, current_bank)?;
            let rel: i32 = target_symbol.logical_address as i32 - *current_address as i32;
            if rel > i8::MAX as i32 || rel < i8::MIN as i32 {
                return Err(AssemblyError::SemanticError {
                    line: line_num,
                    reason: format!(
                        "Label \"{}\" too far away for relative jump, must be within {} bytes of JR instruction",
                        label_name,
                        i8::MAX
                    ),
                });
            };
            Ok(vec![0x5A, rel as u8]) // Opcode for JMP n16
        }
        // conditional jump immediate
        Instruction::Jcc(cc, Operand::Immediate(addr)) => {
            let [low, high] = (*addr as u16).to_le_bytes();
            let opcode = encode_condition_code_opcode(0x5B, cc);
            Ok(vec![opcode, low, high]) // Opcode for JMP n16
        }
        // conditional jump label
        Instruction::Jcc(cc, Operand::Label(label_name)) => {
            let target_symbol =
                get_and_check_symbol(symbol_table, label_name, line_num, current_bank)?;
            let [low, high] = (target_symbol.logical_address as u16).to_le_bytes();
            let opcode = encode_condition_code_opcode(0x5B, cc);
            Ok(vec![opcode, low, high]) // Opcode for JMP n16
        }
        // conditional jump relative immediate
        Instruction::Jrcc(cc, Operand::Immediate(imm)) => {
            let rel = *imm as i8;
            let opcode = encode_condition_code_opcode(0x63, cc);
            Ok(vec![opcode, rel as u8]) // Opcode for JMP n16
        }
        // conditional jump relative label
        Instruction::Jrcc(cc, Operand::Label(label_name)) => {
            let target_symbol =
                get_and_check_symbol(symbol_table, label_name, line_num, current_bank)?;
            let rel: i32 = target_symbol.logical_address as i32 - *current_address as i32;
            if rel > i8::MAX as i32 || rel < i8::MIN as i32 {
                return Err(AssemblyError::SemanticError {
                    line: line_num,
                    reason: format!(
                        "Label \"{}\" too far away for relative jump, must be within {} bytes of JRcc instruction",
                        label_name,
                        i8::MAX
                    ),
                });
            };
            let opcode = encode_condition_code_opcode(0x63, cc);
            Ok(vec![opcode, rel as u8]) // Opcode for JMP n16
        }
        // DJNZ immediate
        Instruction::Djnz(Operand::Immediate(imm)) => {
            let rel = *imm as i8;
            Ok(vec![0x6B, rel as u8]) // Opcode for JMP n16
        }
        // DJNZ label
        Instruction::Djnz(Operand::Label(label_name)) => {
            let target_symbol =
                get_and_check_symbol(symbol_table, label_name, line_num, current_bank)?;
            let rel: i32 = target_symbol.logical_address as i32 - *current_address as i32;
            if rel > i8::MAX as i32 || rel < i8::MIN as i32 {
                return Err(AssemblyError::SemanticError {
                    line: line_num,
                    reason: format!(
                        "Label \"{}\" too far away for relative jump, must be within {} bytes of DJNZ instruction",
                        label_name,
                        i8::MAX
                    ),
                });
            };
            Ok(vec![0x6B, rel as u8]) // Opcode for JMP n16
        }
        // CALL immediate
        Instruction::Call(Operand::Immediate(addr)) => {
            let bytecode = encode_call_immediate(*addr as u16);
            Ok(bytecode)
        }
        // CALL label
        Instruction::Call(Operand::Label(label_name)) => {
            let target_symbol =
                get_and_check_symbol(symbol_table, label_name, line_num, current_bank)?;
            let bytecode = encode_call_immediate(target_symbol.logical_address as u16);
            Ok(bytecode)
        }
        // CALL indirect
        Instruction::Call(Operand::Indirect(reg)) => {
            let opcode = encode_reg_opcode(0xC9, reg);
            Ok(vec![opcode]) // Opcode for JMP n16
        }
        // conditional call immediate
        Instruction::Callcc(cc, Operand::Immediate(addr)) => {
            let [low, high] = (*addr as u16).to_le_bytes();
            let opcode = encode_condition_code_opcode(0xD1, cc);
            Ok(vec![opcode, low, high]) // Opcode for JMP n16
        }
        // conditional call label
        Instruction::Callcc(cc, Operand::Label(label_name)) => {
            let target_symbol =
                get_and_check_symbol(symbol_table, label_name, line_num, current_bank)?;
            let [low, high] = (target_symbol.logical_address as u16).to_le_bytes();
            let opcode = encode_condition_code_opcode(0xD1, cc);
            Ok(vec![opcode, low, high]) // Opcode for JMP n16
        }
        // syscall
        Instruction::Syscall(Operand::Immediate(imm)) => {
            let bytecode = encode_syscall(*imm as u8);
            Ok(bytecode)
        }
        // register-to-register load
        Instruction::Ld(Operand::Register(rd), Operand::Register(rs)) => {
            let opcode = encode_rd_rs_byte(0x80, rd, rs);
            Ok(vec![opcode])
        }
        // add reg to reg
        Instruction::Add(Operand::Register(rd), Some(Operand::Register(rs))) => {
            let byte0 = encode_rd_rs_byte(0x00, rd, rs);
            Ok(vec![0x10, byte0])
        }
        // add immediate
        Instruction::Add(Operand::Register(rd), Some(Operand::Immediate(imm))) => {
            let rd_index = encode_register_operand(rd);
            let [low, high] = (*imm as u16).to_le_bytes();
            Ok(vec![0x09, rd_index, low, high])
        }
        Instruction::AddSp(Operand::Immediate(offset)) => Ok(vec![0x6C, *offset as u8]),
        // add accumulator immediate
        Instruction::Add(Operand::Immediate(value), None) => {
            let [low, high] = (*value as u16).to_le_bytes();
            Ok(vec![0xC0, low, high])
        }
        // add accumulator label
        Instruction::Add(Operand::Label(label_name), None) => {
            let target_symbol =
                get_and_check_symbol(symbol_table, label_name, line_num, current_bank)?;
            let [low, high] = (target_symbol.logical_address as u16).to_le_bytes();
            Ok(vec![0xC0, low, high])
        }
        // add accumulator
        Instruction::Add(Operand::Register(rs), None) => {
            let opcode = encode_reg_opcode(0x18, rs);
            Ok(vec![opcode])
        }
        // add.b accumulator
        Instruction::Addb(Operand::Register(rs)) => {
            let opcode = encode_reg_opcode(0x28, rs);
            Ok(vec![0xFD, opcode])
        }
        // sub reg to reg
        Instruction::Sub(Operand::Register(rd), Some(Operand::Register(rs))) => {
            let byte0 = encode_rd_rs_byte(0x00, rd, rs);
            Ok(vec![0x11, byte0])
        }
        // sub accumulator immediate
        Instruction::Sub(Operand::Immediate(value), None) => {
            let [low, high] = (*value as u16).to_le_bytes();
            Ok(vec![0xC1, low, high])
        }
        // sub accumulator label
        Instruction::Sub(Operand::Label(label_name), None) => {
            let target_symbol =
                get_and_check_symbol(symbol_table, label_name, line_num, current_bank)?;
            let [low, high] = (target_symbol.logical_address as u16).to_le_bytes();
            Ok(vec![0xC1, low, high])
        }
        // sub accumulator
        Instruction::Sub(Operand::Register(rs), None) => {
            let opcode = encode_reg_opcode(0x20, rs);
            Ok(vec![opcode])
        }
        // sub immediate
        Instruction::Sub(Operand::Register(rd), Some(Operand::Immediate(imm))) => {
            let rd_index = encode_register_operand(rd);
            let [low, high] = (*imm as u16).to_le_bytes();
            Ok(vec![0x0A, rd_index, low, high])
        }
        // sub.b accumulator
        Instruction::Subb(Operand::Register(rs)) => {
            let opcode = encode_reg_opcode(0x30, rs);
            Ok(vec![0xFD, opcode])
        }
        // and reg to reg
        Instruction::And(Operand::Register(rd), Some(Operand::Register(rs))) => {
            let byte0 = encode_rd_rs_byte(0x00, rd, rs);
            Ok(vec![0x12, byte0])
        }
        // and immediate
        Instruction::And(Operand::Register(rd), Some(Operand::Immediate(imm))) => {
            let rd_index = encode_register_operand(rd);
            let [low, high] = (*imm as u16).to_le_bytes();
            Ok(vec![0x0B, rd_index, low, high])
        }
        // and.b accumulator
        Instruction::Andb(Operand::Register(rs)) => {
            let opcode = encode_reg_opcode(0x38, rs);
            Ok(vec![0xFD, opcode])
        }
        // or reg to reg
        Instruction::Or(Operand::Register(rd), Some(Operand::Register(rs))) => {
            let byte0 = encode_rd_rs_byte(0x00, rd, rs);
            Ok(vec![0x13, byte0])
        }
        // or immediate
        Instruction::Or(Operand::Register(rd), Some(Operand::Immediate(imm))) => {
            let rd_index = encode_register_operand(rd);
            let [low, high] = (*imm as u16).to_le_bytes();
            Ok(vec![0x0C, rd_index, low, high])
        }
        // or.b accumulator
        Instruction::Orb(Operand::Register(rs)) => {
            let opcode = encode_reg_opcode(0x40, rs);
            Ok(vec![0xFD, opcode])
        }
        // xor reg to reg
        Instruction::Xor(Operand::Register(rd), Some(Operand::Register(rs))) => {
            let byte0 = encode_rd_rs_byte(0x00, rd, rs);
            Ok(vec![0x14, byte0])
        }
        // xor immediate
        Instruction::Xor(Operand::Register(rd), Some(Operand::Immediate(imm))) => {
            let rd_index = encode_register_operand(rd);
            let [low, high] = (*imm as u16).to_le_bytes();
            Ok(vec![0x0D, rd_index, low, high])
        }
        // xor.b accumulator
        Instruction::Xorb(Operand::Register(rs)) => {
            let opcode = encode_reg_opcode(0x48, rs);
            Ok(vec![0xFD, opcode])
        }
        // cmp reg to reg
        Instruction::Cmp(Operand::Register(rd), Some(Operand::Register(rs))) => {
            let byte0 = encode_rd_rs_byte(0x00, rd, rs);
            Ok(vec![0x15, byte0])
        }
        // cmp immediate
        Instruction::Cmp(Operand::Register(rd), Some(Operand::Immediate(imm))) => {
            let rd_index = encode_register_operand(rd);
            let [low, high] = (*imm as u16).to_le_bytes();
            Ok(vec![0x0E, rd_index, low, high])
        }
        // cmp.b accumulator
        Instruction::Cmpb(Operand::Register(rs)) => {
            let opcode = encode_reg_opcode(0x50, rs);
            Ok(vec![0xFD, opcode])
        }
        // adc reg to reg
        Instruction::Adc(Operand::Register(rd), Some(Operand::Register(rs))) => {
            let byte0 = encode_rd_rs_byte(0x00, rd, rs);
            Ok(vec![0x16, byte0])
        }
        // adc accumulator immediate
        Instruction::Adc(Operand::Immediate(value), None) => {
            let [low, high] = (*value as u16).to_le_bytes();
            Ok(vec![0xC6, low, high])
        }
        // adc accumulator label
        Instruction::Adc(Operand::Label(label_name), None) => {
            let target_symbol =
                get_and_check_symbol(symbol_table, label_name, line_num, current_bank)?;
            let [low, high] = (target_symbol.logical_address as u16).to_le_bytes();
            Ok(vec![0xC6, low, high])
        }
        // sbc reg to reg
        Instruction::Sbc(Operand::Register(rd), Some(Operand::Register(rs))) => {
            let byte0 = encode_rd_rs_byte(0x00, rd, rs);
            Ok(vec![0x17, byte0])
        }
        // sbc accumulator immediate
        Instruction::Sbc(Operand::Immediate(value), None) => {
            let [low, high] = (*value as u16).to_le_bytes();
            Ok(vec![0xC7, low, high])
        }
        // sbc accumulator label
        Instruction::Sbc(Operand::Label(label_name), None) => {
            let target_symbol =
                get_and_check_symbol(symbol_table, label_name, line_num, current_bank)?;
            let [low, high] = (target_symbol.logical_address as u16).to_le_bytes();
            Ok(vec![0xC7, low, high])
        }
        Instruction::Push(Operand::Register(reg)) => {
            let bytecode = encode_push_r(reg);
            Ok(bytecode)
        }
        Instruction::Push(Operand::Immediate(value)) => {
            let [low, high] = (*value as u16).to_le_bytes();
            Ok(vec![0x7D, low, high])
        }
        Instruction::Push(Operand::Label(label_name)) => {
            let target_symbol =
                get_and_check_symbol(symbol_table, label_name, line_num, current_bank)?;
            let [low, high] = (target_symbol.logical_address as u16).to_le_bytes();
            Ok(vec![0x7D, low, high])
        }
        Instruction::PushF => Ok(vec![0x7E]),
        Instruction::Pop(Operand::Register(reg)) => {
            let bytecode = encode_pop_r(reg);
            Ok(bytecode)
        }
        Instruction::PopF => Ok(vec![0x7F]),
        // and accumulator immediate
        Instruction::And(Operand::Immediate(value), None) => {
            let [low, high] = (*value as u16).to_le_bytes();
            Ok(vec![0xC2, low, high])
        }
        // and accumulator label
        Instruction::And(Operand::Label(label_name), None) => {
            let target_symbol =
                get_and_check_symbol(symbol_table, label_name, line_num, current_bank)?;
            let [low, high] = (target_symbol.logical_address as u16).to_le_bytes();
            Ok(vec![0xC2, low, high])
        }
        // and accumulator
        Instruction::And(Operand::Register(rs), None) => {
            let opcode = encode_reg_opcode(0x28, rs);
            Ok(vec![opcode])
        }
        // or accumulator
        Instruction::Or(Operand::Register(rs), None) => {
            let opcode = encode_reg_opcode(0x30, rs);
            Ok(vec![opcode])
        }
        // or accumulator immediate
        Instruction::Or(Operand::Immediate(value), None) => {
            let [low, high] = (*value as u16).to_le_bytes();
            Ok(vec![0xC3, low, high])
        }
        // or accumulator label
        Instruction::Or(Operand::Label(label_name), None) => {
            let target_symbol =
                get_and_check_symbol(symbol_table, label_name, line_num, current_bank)?;
            let [low, high] = (target_symbol.logical_address as u16).to_le_bytes();
            Ok(vec![0xC3, low, high])
        }
        // xor accumulator
        Instruction::Xor(Operand::Register(rs), None) => {
            let opcode = encode_reg_opcode(0x38, rs);
            Ok(vec![opcode])
        }
        // xor accumulator immediate
        Instruction::Xor(Operand::Immediate(value), None) => {
            let [low, high] = (*value as u16).to_le_bytes();
            Ok(vec![0xC4, low, high])
        }
        // xor accumulator label
        Instruction::Xor(Operand::Label(label_name), None) => {
            let target_symbol =
                get_and_check_symbol(symbol_table, label_name, line_num, current_bank)?;
            let [low, high] = (target_symbol.logical_address as u16).to_le_bytes();
            Ok(vec![0xC4, low, high])
        }
        // cmp accumulator
        Instruction::Cmp(Operand::Register(rs), None) => {
            let opcode = encode_reg_opcode(0x40, rs);
            Ok(vec![opcode])
        }
        // cmp accumulator immediate
        Instruction::Cmp(Operand::Immediate(value), None) => {
            let [low, high] = (*value as u16).to_le_bytes();
            Ok(vec![0xC5, low, high])
        }
        // cmp accumulator label
        Instruction::Cmp(Operand::Label(label_name), None) => {
            let target_symbol =
                get_and_check_symbol(symbol_table, label_name, line_num, current_bank)?;
            let [low, high] = (target_symbol.logical_address as u16).to_le_bytes();
            Ok(vec![0xC5, low, high])
        }
        // neg
        Instruction::Neg => Ok(vec![0x48]),
        // not
        Instruction::Not => Ok(vec![0x49]),
        // swap
        Instruction::Swap => Ok(vec![0x4A]),
        // indirect-to-register load
        Instruction::Ld(Operand::Register(rd), Operand::Indirect(rs)) => {
            let sub_opcode = encode_rd_rs_byte(0x00, rd, rs);
            Ok(vec![0xFE, sub_opcode])
        }
        // register-to-indirect store
        Instruction::St(Operand::Indirect(rd), Operand::Register(rs)) => {
            let sub_opcode = encode_rd_rs_byte(0x40, rd, rs);
            Ok(vec![0xFE, sub_opcode])
        }
        // absolute-to-register load
        Instruction::Ld(Operand::Register(rd), Operand::Absolute(addr)) => {
            let opcode = encode_reg_opcode(0xE9, rd);
            let [low, high] = (*addr as u16).to_le_bytes();
            Ok(vec![opcode, low, high])
        }
        // register-to-absolute store
        Instruction::St(Operand::Absolute(addr), Operand::Register(rs)) => {
            let opcode = encode_reg_opcode(0xF1, rs);
            let [low, high] = (*addr as u16).to_le_bytes();
            Ok(vec![opcode, low, high])
        }

        Instruction::Sra(Operand::Register(reg)) => {
            let opcode = encode_reg_opcode(0x00, reg);
            Ok(vec![0xFD, opcode])
        }
        Instruction::Shl(Operand::Register(reg)) => {
            let opcode = encode_reg_opcode(0x08, reg);
            Ok(vec![0xFD, opcode])
        }
        Instruction::Shr(Operand::Register(reg)) => {
            let opcode = encode_reg_opcode(0x10, reg);
            Ok(vec![0xFD, opcode])
        }
        Instruction::Rol(Operand::Register(reg)) => {
            let opcode = encode_reg_opcode(0x18, reg);
            Ok(vec![0xFD, opcode])
        }
        Instruction::Ror(Operand::Register(reg)) => {
            let opcode = encode_reg_opcode(0x20, reg);
            Ok(vec![0xFD, opcode])
        }
        Instruction::Bit(Operand::Register(r), Operand::Immediate(b)) => {
            let reg = encode_register_operand(r);
            let sub_opcode: u8 = 0x58 + *b as u8;
            Ok(vec![0xFD, sub_opcode, reg])
        }
        Instruction::Set(Operand::Register(r), Operand::Immediate(b)) => {
            let reg = encode_register_operand(r);
            let sub_opcode: u8 = 0x60 + *b as u8;
            Ok(vec![0xFD, sub_opcode, reg])
        }
        Instruction::Res(Operand::Register(r), Operand::Immediate(b)) => {
            let reg = encode_register_operand(r);
            let sub_opcode: u8 = 0x68 + *b as u8;
            Ok(vec![0xFD, sub_opcode, reg])
        }
        Instruction::Bit(Operand::Absolute(addr), Operand::Immediate(b)) => {
            let [low, high] = (*addr as u16).to_le_bytes();
            let sub_opcode: u8 = 0x70 + *b as u8;
            Ok(vec![0xFD, sub_opcode, low, high])
        }
        Instruction::Set(Operand::Absolute(addr), Operand::Immediate(b)) => {
            let [low, high] = (*addr as u16).to_le_bytes();
            let sub_opcode: u8 = 0x78 + *b as u8;
            Ok(vec![0xFD, sub_opcode, low, high])
        }
        Instruction::Res(Operand::Absolute(addr), Operand::Immediate(b)) => {
            let [low, high] = (*addr as u16).to_le_bytes();
            let sub_opcode: u8 = 0x80 + *b as u8;
            Ok(vec![0xFD, sub_opcode, low, high])
        }
        Instruction::Bit(Operand::Indirect(r), Operand::Immediate(b)) => {
            let reg = encode_register_operand(r);
            let sub_opcode: u8 = 0x88 + *b as u8;
            Ok(vec![0xFD, sub_opcode, reg])
        }
        Instruction::Set(Operand::Indirect(r), Operand::Immediate(b)) => {
            let reg = encode_register_operand(r);
            let sub_opcode: u8 = 0x90 + *b as u8;
            Ok(vec![0xFD, sub_opcode, reg])
        }
        Instruction::Res(Operand::Indirect(r), Operand::Immediate(b)) => {
            let reg = encode_register_operand(r);
            let sub_opcode: u8 = 0x98 + *b as u8;
            Ok(vec![0xFD, sub_opcode, reg])
        }
        // accumulator immediate byte load
        Instruction::Ldb(Operand::Register(rd), Operand::Immediate(imm8)) => {
            let sub_opcode = encode_reg_opcode(0xA0, rd);
            Ok(vec![0xFD, sub_opcode, *imm8 as u8])
        }
        Instruction::Ldb(Operand::Register(rd), Operand::Indirect(rs)) => {
            let sub_opcode = encode_rd_rs_byte(0x80, rd, rs);
            Ok(vec![0xFE, sub_opcode])
        }
        Instruction::Stb(Operand::Indirect(rd), Operand::Register(rs)) => {
            let sub_opcode = encode_rd_rs_byte(0x80, rd, rs);
            Ok(vec![0xFE, sub_opcode])
        }
        Instruction::Ld(Operand::Register(rd), Operand::Indexed(rs, offset)) => {
            let sub_opcode = encode_rd_rs_byte(0x00, rd, rs);
            Ok(vec![0xFF, sub_opcode, *offset as u8])
        }
        Instruction::St(Operand::Indexed(rd, offset), Operand::Register(rs)) => {
            let sub_opcode = encode_rd_rs_byte(0x40, rd, rs);
            Ok(vec![0xFF, sub_opcode, *offset as u8])
        }
        Instruction::Lea(Operand::Register(rd), Operand::Indexed(rs, offset)) => {
            let sub_opcode = encode_rd_rs_byte(0x80, rd, rs);
            Ok(vec![0xFF, sub_opcode, *offset as u8])
        }
        Instruction::Ld(Operand::Register(rd), Operand::PostIncrement(rs)) => {
            let sub_opcode = encode_reg_opcode(0xC0, rs);
            let dest = encode_register_operand(rd);
            Ok(vec![0xFF, sub_opcode, dest])
        }
        Instruction::St(Operand::PostIncrement(rd), Operand::Register(rs)) => {
            let sub_opcode = encode_reg_opcode(0xC8, rs);
            let dest = encode_register_operand(rd);
            Ok(vec![0xFF, sub_opcode, dest])
        }
        Instruction::Ld(Operand::Register(rd), Operand::PreDecrement(rs)) => {
            let sub_opcode = encode_reg_opcode(0xD0, rs);
            let dest = encode_register_operand(rd);
            Ok(vec![0xFF, sub_opcode, dest])
        }
        Instruction::St(Operand::PreDecrement(rd), Operand::Register(rs)) => {
            let sub_opcode = encode_reg_opcode(0xD8, rs);
            let dest = encode_register_operand(rd);
            Ok(vec![0xFF, sub_opcode, dest])
        }
        Instruction::Ldb(Operand::Register(rd), Operand::PostIncrement(rs)) => {
            let sub_opcode = encode_reg_opcode(0xE0, rs);
            let dest = encode_register_operand(rd);
            Ok(vec![0xFF, sub_opcode, dest])
        }
        Instruction::Stb(Operand::PostIncrement(rd), Operand::Register(rs)) => {
            let sub_opcode = encode_reg_opcode(0xE8, rs);
            let dest = encode_register_operand(rd);
            Ok(vec![0xFF, sub_opcode, dest])
        }
        Instruction::Ldb(Operand::Register(rd), Operand::PreDecrement(rs)) => {
            let sub_opcode = encode_reg_opcode(0xF0, rs);
            let dest = encode_register_operand(rd);
            Ok(vec![0xFF, sub_opcode, dest])
        }
        Instruction::Stb(Operand::PreDecrement(rd), Operand::Register(rs)) => {
            let sub_opcode = encode_reg_opcode(0xF8, rs);
            let dest = encode_register_operand(rd);
            Ok(vec![0xFF, sub_opcode, dest])
        }
        Instruction::CallFar(Operand::Label(label_name), None) => {
            let target_symbol = get_symbol(symbol_table, label_name, line_num)?;

            if target_symbol.bank == 0 {
                return Err(AssemblyError::SemanticError {
                    line: line_num,
                    reason: format!(
                        "Label \"{}\" exists in bank 0, use a normal CALL instruction instead.",
                        label_name
                    ),
                });
            } else if target_symbol.bank == *current_bank {
                return Err(AssemblyError::SemanticError {
                    line: line_num,
                    reason: format!(
                        "Label \"{}\" exists in the same bank as the CALL.far instruction, use a normal CALL instruction instead.",
                        label_name
                    ),
                });
            }

            let final_bytecode = encode_far(target_symbol, 0x21);
            Ok(final_bytecode)
        }
        Instruction::CallFar(Operand::Label(call_label), Some(Operand::Label(via_label))) => {
            let call_symbol = get_symbol(symbol_table, call_label, line_num)?;
            let via_symbol = get_symbol(symbol_table, via_label, line_num)?;

            if call_symbol.bank == 0 {
                return Err(AssemblyError::SemanticError {
                    line: line_num,
                    reason: format!(
                        "Label \"{}\" exists in bank 0, use a normal CALL instruction instead.",
                        call_label
                    ),
                });
            } else if call_symbol.bank == *current_bank {
                return Err(AssemblyError::SemanticError {
                    line: line_num,
                    reason: format!(
                        "Label \"{}\" exists in the same bank as the CALL.far instruction, use a normal CALL instruction instead.",
                        call_label
                    ),
                });
            }

            if via_symbol.bank != 0 {
                return Err(AssemblyError::SemanticError {
                    line: line_num,
                    reason: format!(
                        "Custom CALL.far via trampoline label must exist in bank 0, \"{}\" found in bank {}",
                        via_label, via_symbol.bank,
                    ),
                });
            }

            let final_bytecode = encode_far_via(call_symbol, via_symbol);
            Ok(final_bytecode)
        }
        Instruction::JmpFar(Operand::Label(label_name), None) => {
            let target_symbol = get_symbol(symbol_table, label_name, line_num)?;

            if target_symbol.bank == 0 {
                return Err(AssemblyError::SemanticError {
                    line: line_num,
                    reason: format!(
                        "Label \"{}\" exists in bank 0, use a normal JMP instruction instead.",
                        label_name
                    ),
                });
            } else if target_symbol.bank == *current_bank {
                return Err(AssemblyError::SemanticError {
                    line: line_num,
                    reason: format!(
                        "Label \"{}\" exists in the same bank as the JMP.far instruction, use a normal JMP instruction instead.",
                        label_name
                    ),
                });
            }

            let final_bytecode = encode_far(target_symbol, 0x22);
            Ok(final_bytecode)
        }
        Instruction::JmpFar(Operand::Label(call_label), Some(Operand::Label(via_label))) => {
            let call_symbol = get_symbol(symbol_table, call_label, line_num)?;
            let via_symbol = get_symbol(symbol_table, via_label, line_num)?;

            if call_symbol.bank == 0 {
                return Err(AssemblyError::SemanticError {
                    line: line_num,
                    reason: format!(
                        "Label \"{}\" exists in bank 0, use a normal JMP instruction instead.",
                        call_label
                    ),
                });
            } else if call_symbol.bank == *current_bank {
                return Err(AssemblyError::SemanticError {
                    line: line_num,
                    reason: format!(
                        "Label \"{}\" exists in the same bank as the JMP.far instruction, use a normal JMP instruction instead.",
                        call_label
                    ),
                });
            }

            if via_symbol.bank != 0 {
                return Err(AssemblyError::SemanticError {
                    line: line_num,
                    reason: format!(
                        "Custom JMP.far via trampoline label must exist in bank 0, \"{}\" found in bank {}",
                        via_label, via_symbol.bank,
                    ),
                });
            }

            let final_bytecode = encode_far_via(call_symbol, via_symbol);
            Ok(final_bytecode)
        }

        // ... add encoding logic for every instruction variant based on your opcode map ...
        _ => Err(AssemblyError::SemanticErrorNoLine {
            reason: "Invalid Instruction".to_string(),
        }),
    }
}

fn encode_far(call_symbol: &Symbol, syslib_index: u8) -> Vec<u8> {
    let mut bytecode = Vec::new();
    bytecode.extend(encode_ldi(&Register::R4, call_symbol.bank as u16));
    bytecode.extend(encode_ldi(
        &Register::R5,
        call_symbol.logical_address as u16,
    ));
    bytecode.extend(encode_syscall(syslib_index));
    bytecode
}

fn encode_far_via(call_symbol: &Symbol, via_symbol: &Symbol) -> Vec<u8> {
    let mut bytecode = Vec::new();
    bytecode.extend(encode_ldi(&Register::R4, call_symbol.bank as u16));
    bytecode.extend(encode_ldi(
        &Register::R5,
        call_symbol.logical_address as u16,
    ));
    bytecode.extend(encode_call_immediate(via_symbol.logical_address as u16));
    bytecode
}

fn encode_push_r(rs: &Register) -> Vec<u8> {
    let index = encode_register_operand(rs);
    vec![0x6D + index]
}

fn encode_pop_r(rd: &Register) -> Vec<u8> {
    let index = encode_register_operand(rd);
    vec![0x75 + index]
}

fn encode_ldi(rd: &Register, val: u16) -> Vec<u8> {
    let opcode = encode_reg_opcode(0x01, rd);
    let [low, high] = val.to_le_bytes();
    vec![opcode, low, high]
}

fn encode_syscall(index: u8) -> Vec<u8> {
    vec![0x4E, index]
}

fn encode_call_immediate(logical_addr: u16) -> Vec<u8> {
    let [low, high] = logical_addr.to_le_bytes();
    vec![0xC8, low, high]
}

fn encode_rd_rs_byte(base_val: u8, rd: &Register, rs: &Register) -> u8 {
    let rd_index = encode_register_operand(rd);
    let rs_index = encode_register_operand(rs);
    base_val | ((rd_index & 0x07) << 3) | (rs_index & 0x07)
}

fn encode_reg_opcode(base_opcode: u8, r: &Register) -> u8 {
    base_opcode + encode_register_operand(r)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_instruction_size_nop() {
        let instruction = Instruction::Nop;
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 1);
    }

    #[test]
    fn test_encode_instruction_nop() {
        let instruction = Instruction::Nop;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x00]
        );
    }

    #[test]
    fn test_calculate_instruction_size_sub_reg() {
        let instruction = Instruction::Sub(Operand::Register(Register::R1), None);
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 1);
    }

    #[test]
    fn test_encode_instruction_sub_reg() {
        let instruction = Instruction::Sub(Operand::Register(Register::R1), None);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x21]
        );
    }

    #[test]
    fn test_calculate_instruction_size_and_reg_reg() {
        let instruction = Instruction::And(
            Operand::Register(Register::R2),
            Some(Operand::Register(Register::R3)),
        );
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 2);
    }

    #[test]
    fn test_encode_instruction_and_reg_reg() {
        let instruction = Instruction::And(
            Operand::Register(Register::R2),
            Some(Operand::Register(Register::R3)),
        );
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x12, (2 << 3) | 3]
        );
    }

    #[test]
    fn test_calculate_instruction_size_or_reg_reg() {
        let instruction = Instruction::Or(
            Operand::Register(Register::R4),
            Some(Operand::Register(Register::R5)),
        );
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 2);
    }

    #[test]
    fn test_encode_instruction_or_reg_reg() {
        let instruction = Instruction::Or(
            Operand::Register(Register::R4),
            Some(Operand::Register(Register::R5)),
        );
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x13, (4 << 3) | 5]
        );
    }

    #[test]
    fn test_calculate_instruction_size_xor_reg_reg() {
        let instruction = Instruction::Xor(
            Operand::Register(Register::R6),
            Some(Operand::Register(Register::R7)),
        );
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 2);
    }

    #[test]
    fn test_encode_instruction_xor_reg_reg() {
        let instruction = Instruction::Xor(
            Operand::Register(Register::R6),
            Some(Operand::Register(Register::R7)),
        );
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x14, (6 << 3) | 7]
        );
    }

    #[test]
    fn test_calculate_instruction_size_cmp_reg_reg() {
        let instruction = Instruction::Cmp(
            Operand::Register(Register::R0),
            Some(Operand::Register(Register::R1)),
        );
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 2);
    }

    #[test]
    fn test_encode_instruction_cmp_reg_reg() {
        let instruction = Instruction::Cmp(
            Operand::Register(Register::R0),
            Some(Operand::Register(Register::R1)),
        );
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x15, (0 << 3) | 1]
        );
    }

    #[test]
    fn test_calculate_instruction_size_adc_reg_reg() {
        let instruction = Instruction::Adc(
            Operand::Register(Register::R2),
            Some(Operand::Register(Register::R3)),
        );
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 2);
    }

    #[test]
    fn test_encode_instruction_adc_reg_reg() {
        let instruction = Instruction::Adc(
            Operand::Register(Register::R2),
            Some(Operand::Register(Register::R3)),
        );
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x16, (2 << 3) | 3]
        );
    }

    #[test]
    fn test_calculate_instruction_size_sbc_reg_reg() {
        let instruction = Instruction::Sbc(
            Operand::Register(Register::R4),
            Some(Operand::Register(Register::R5)),
        );
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 2);
    }

    #[test]
    fn test_encode_instruction_sbc_reg_reg() {
        let instruction = Instruction::Sbc(
            Operand::Register(Register::R4),
            Some(Operand::Register(Register::R5)),
        );
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x17, (4 << 3) | 5]
        );
    }

    #[test]
    fn test_calculate_instruction_size_and_reg() {
        let instruction = Instruction::And(Operand::Register(Register::R1), None);
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 1);
    }

    #[test]
    fn test_encode_instruction_and_reg() {
        let instruction = Instruction::And(Operand::Register(Register::R1), None);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x29]
        );
    }

    #[test]
    fn test_calculate_instruction_size_or_reg() {
        let instruction = Instruction::Or(Operand::Register(Register::R2), None);
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 1);
    }

    #[test]
    fn test_encode_instruction_or_reg() {
        let instruction = Instruction::Or(Operand::Register(Register::R2), None);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x32]
        );
    }

    #[test]
    fn test_calculate_instruction_size_xor_reg() {
        let instruction = Instruction::Xor(Operand::Register(Register::R3), None);
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 1);
    }

    #[test]
    fn test_encode_instruction_xor_reg() {
        let instruction = Instruction::Xor(Operand::Register(Register::R3), None);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x3B]
        );
    }

    #[test]
    fn test_calculate_instruction_size_cmp_reg() {
        let instruction = Instruction::Cmp(Operand::Register(Register::R4), None);
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 1);
    }

    #[test]
    fn test_encode_instruction_cmp_reg() {
        let instruction = Instruction::Cmp(Operand::Register(Register::R4), None);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x44]
        );
    }

    #[test]
    fn test_encode_instruction_add_acc_immediate() {
        let instruction = Instruction::Add(Operand::Immediate(0x1234), None);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xC0, 0x34, 0x12]
        );
    }

    #[test]
    fn test_encode_instruction_add_acc_label() {
        let instruction = Instruction::Add(Operand::Label("TARGET".into()), None);
        let mut symbol_table = SymbolTable::new();
        symbol_table.insert(
            "TARGET".to_string(),
            Symbol {
                logical_address: 0x2468,
                bank: 0,
            },
        );
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xC0, 0x68, 0x24]
        );
    }

    #[test]
    fn test_encode_instruction_sub_acc_immediate() {
        let instruction = Instruction::Sub(Operand::Immediate(0x00FF), None);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xC1, 0xFF, 0x00]
        );
    }

    #[test]
    fn test_encode_instruction_and_acc_immediate() {
        let instruction = Instruction::And(Operand::Immediate(0x0F0F), None);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xC2, 0x0F, 0x0F]
        );
    }

    #[test]
    fn test_encode_instruction_or_acc_immediate() {
        let instruction = Instruction::Or(Operand::Immediate(0x8000), None);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xC3, 0x00, 0x80]
        );
    }

    #[test]
    fn test_encode_instruction_xor_acc_immediate() {
        let instruction = Instruction::Xor(Operand::Immediate(0xAAAA), None);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xC4, 0xAA, 0xAA]
        );
    }

    #[test]
    fn test_encode_instruction_cmp_acc_immediate() {
        let instruction = Instruction::Cmp(Operand::Immediate(0x0A0B), None);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xC5, 0x0B, 0x0A]
        );
    }

    #[test]
    fn test_encode_instruction_adci_acc_immediate() {
        let instruction = Instruction::Adc(Operand::Immediate(0xFFFF), None);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xC6, 0xFF, 0xFF]
        );
    }

    #[test]
    fn test_encode_instruction_sbci_acc_immediate() {
        let instruction = Instruction::Sbc(Operand::Immediate(0x1234), None);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xC7, 0x34, 0x12]
        );
    }

    #[test]
    fn test_calculate_instruction_size_add_sp() {
        let instruction = Instruction::AddSp(Operand::Immediate(-8));
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 2);
    }

    #[test]
    fn test_encode_instruction_add_sp() {
        let instruction = Instruction::AddSp(Operand::Immediate(-5));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x6C, 0xFB]
        );
    }

    #[test]
    fn test_encode_instruction_push_reg() {
        let instruction = Instruction::Push(Operand::Register(Register::R2));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x6F]
        );
    }

    #[test]
    fn test_encode_instruction_push_immediate() {
        let instruction = Instruction::Push(Operand::Immediate(0x1234));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x7D, 0x34, 0x12]
        );
    }

    #[test]
    fn test_encode_instruction_push_label() {
        let instruction = Instruction::Push(Operand::Label("TARGET".into()));
        let mut symbol_table = SymbolTable::new();
        symbol_table.insert(
            "TARGET".to_string(),
            Symbol {
                logical_address: 0x1357,
                bank: 0,
            },
        );
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x7D, 0x57, 0x13]
        );
    }

    #[test]
    fn test_encode_instruction_push_f() {
        let instruction = Instruction::PushF;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x7E]
        );
    }

    #[test]
    fn test_encode_instruction_pop_reg() {
        let instruction = Instruction::Pop(Operand::Register(Register::R3));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x78]
        );
    }

    #[test]
    fn test_encode_instruction_pop_f() {
        let instruction = Instruction::PopF;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x7F]
        );
    }

    #[test]
    fn test_calculate_instruction_size_neg() {
        let instruction = Instruction::Neg;
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 1);
    }

    #[test]
    fn test_encode_instruction_neg() {
        let instruction = Instruction::Neg;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x48]
        );
    }

    #[test]
    fn test_calculate_instruction_size_not() {
        let instruction = Instruction::Not;
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 1);
    }

    #[test]
    fn test_encode_instruction_not() {
        let instruction = Instruction::Not;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x49]
        );
    }

    #[test]
    fn test_calculate_instruction_size_swap() {
        let instruction = Instruction::Swap;
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 1);
    }

    #[test]
    fn test_encode_instruction_swap() {
        let instruction = Instruction::Swap;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x4A]
        );
    }

    #[test]
    fn test_calculate_instruction_size_ccf() {
        let instruction = Instruction::Ccf;
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 1);
    }

    #[test]
    fn test_encode_instruction_ccf() {
        let instruction = Instruction::Ccf;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x4B]
        );
    }

    #[test]
    fn test_calculate_instruction_size_scf() {
        let instruction = Instruction::Scf;
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 1);
    }

    #[test]
    fn test_encode_instruction_scf() {
        let instruction = Instruction::Scf;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x4C]
        );
    }

    #[test]
    fn test_calculate_instruction_size_rcf() {
        let instruction = Instruction::Rcf;
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 1);
    }

    #[test]
    fn test_encode_instruction_rcf() {
        let instruction = Instruction::Rcf;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x4D]
        );
    }

    #[test]
    fn test_calculate_instruction_size_enter() {
        let instruction = Instruction::Enter;
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 1);
    }

    #[test]
    fn test_encode_instruction_enter() {
        let instruction = Instruction::Enter;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x4F]
        );
    }

    #[test]
    fn test_calculate_instruction_size_leave() {
        let instruction = Instruction::Leave;
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 1);
    }

    #[test]
    fn test_encode_instruction_leave() {
        let instruction = Instruction::Leave;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x50]
        );
    }

    #[test]
    fn test_calculate_instruction_size_ret() {
        let instruction = Instruction::Ret;
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 1);
    }

    #[test]
    fn test_encode_instruction_ret() {
        let instruction = Instruction::Ret;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xF9]
        );
    }

    #[test]
    fn test_calculate_instruction_size_reti() {
        let instruction = Instruction::Reti;
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 1);
    }

    #[test]
    fn test_encode_instruction_reti() {
        let instruction = Instruction::Reti;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFA]
        );
    }

    #[test]
    fn test_calculate_instruction_size_ei() {
        let instruction = Instruction::Ei;
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 1);
    }

    #[test]
    fn test_encode_instruction_ei() {
        let instruction = Instruction::Ei;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFB]
        );
    }

    #[test]
    fn test_calculate_instruction_size_di() {
        let instruction = Instruction::Di;
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 1);
    }

    #[test]
    fn test_encode_instruction_di() {
        let instruction = Instruction::Di;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFC]
        );
    }

    #[test]
    fn test_calculate_instruction_size_inc() {
        let instruction = Instruction::Inc(Operand::Register(Register::R1));
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 1);
    }

    #[test]
    fn test_encode_instruction_inc() {
        let instruction = Instruction::Inc(Operand::Register(Register::R1));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xE2]
        );
    }

    #[test]
    fn test_calculate_instruction_size_dec() {
        let instruction = Instruction::Dec(Operand::Register(Register::R2));
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 1);
    }

    #[test]
    fn test_encode_instruction_dec() {
        let instruction = Instruction::Dec(Operand::Register(Register::R2));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xDB]
        );
    }

    #[test]
    fn test_calculate_instruction_size_call_immediate() {
        let instruction = Instruction::Call(Operand::Immediate(0x1234));
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 3);
    }

    #[test]
    fn test_encode_instruction_call_immediate() {
        let instruction = Instruction::Call(Operand::Immediate(0x1234));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xC8, 0x34, 0x12]
        );
    }

    #[test]
    fn test_calculate_instruction_size_call_label() {
        let instruction = Instruction::Call(Operand::Label("test".to_string()));
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 3);
    }

    #[test]
    fn test_encode_instruction_call_label() {
        let instruction = Instruction::Call(Operand::Label("test_label".to_string()));
        let mut symbol_table = SymbolTable::new();
        symbol_table.insert(
            "test_label".to_string(),
            Symbol {
                logical_address: 0x4321,
                bank: 1,
            },
        );
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0x4100, &1, 0).unwrap(),
            vec![0xC8, 0x21, 0x43]
        );
    }

    #[test]
    fn test_calculate_instruction_size_call_indirect() {
        let instruction = Instruction::Call(Operand::Indirect(Register::R4));
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 1);
    }

    #[test]
    fn test_encode_instruction_call_indirect() {
        let instruction = Instruction::Call(Operand::Indirect(Register::R4));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xCD] // 0xC9 + 4
        );
    }

    #[test]
    fn test_calculate_instruction_size_callcc() {
        let instruction =
            Instruction::Callcc(ConditionCode::Nz, Operand::Label("test".to_string()));
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 3);
    }

    #[test]
    fn test_encode_instruction_callcc_immediate() {
        let instruction = Instruction::Callcc(ConditionCode::C, Operand::Immediate(0x1122));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xD5, 0x22, 0x11] // 0xD1 + 4
        );
    }

    #[test]
    fn test_encode_instruction_callcc_label() {
        let instruction =
            Instruction::Callcc(ConditionCode::Nz, Operand::Label("test_label".to_string()));
        let mut symbol_table = SymbolTable::new();
        symbol_table.insert(
            "test_label".to_string(),
            Symbol {
                logical_address: 0x4321,
                bank: 1,
            },
        );
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0x4100, &1, 0).unwrap(),
            vec![0xD8, 0x21, 0x43] // 0xD1 + 7
        );
    }

    #[test]
    fn test_calculate_instruction_size_syscall() {
        let instruction = Instruction::Syscall(Operand::Immediate(0x1A));
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 2);
    }

    #[test]
    fn test_encode_instruction_syscall() {
        let instruction = Instruction::Syscall(Operand::Immediate(0x1A));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x4E, 0x1A]
        );
    }

    #[test]
    fn test_calculate_instruction_size_ld_absolute() {
        let instruction =
            Instruction::Ld(Operand::Register(Register::R1), Operand::Absolute(0x1234));
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 3);
    }

    #[test]
    fn test_encode_instruction_ld_absolute() {
        let instruction =
            Instruction::Ld(Operand::Register(Register::R1), Operand::Absolute(0x1234));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xEA, 0x34, 0x12] // 0xE9 + 1
        );
    }

    #[test]
    fn test_calculate_instruction_size_st_absolute() {
        let instruction =
            Instruction::St(Operand::Absolute(0x4321), Operand::Register(Register::R2));
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 3);
    }

    #[test]
    fn test_encode_instruction_st_absolute() {
        let instruction =
            Instruction::St(Operand::Absolute(0x4321), Operand::Register(Register::R2));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xF3, 0x21, 0x43] // 0xF1 + 2
        );
    }

    #[test]
    fn test_calculate_instruction_size_st_indirect() {
        let instruction = Instruction::St(
            Operand::Indirect(Register::R1),
            Operand::Register(Register::R2),
        );
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 2);
    }

    #[test]
    fn test_encode_instruction_st_indirect() {
        let instruction = Instruction::St(
            Operand::Indirect(Register::R1),
            Operand::Register(Register::R2),
        );
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFE, 0x4A] // 0x40 | (1 << 3) | 2
        );
    }

    #[test]
    fn test_encode_sra() {
        let instruction = Instruction::Sra(Operand::Register(Register::R0));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFD, 0x00]
        );
    }

    #[test]
    fn test_encode_shl() {
        let instruction = Instruction::Shl(Operand::Register(Register::R1));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFD, 0x09]
        );
    }

    #[test]
    fn test_encode_shr() {
        let instruction = Instruction::Shr(Operand::Register(Register::R2));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFD, 0x12]
        );
    }

    #[test]
    fn test_encode_rol() {
        let instruction = Instruction::Rol(Operand::Register(Register::R3));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFD, 0x1B]
        );
    }

    #[test]
    fn test_encode_ror() {
        let instruction = Instruction::Ror(Operand::Register(Register::R4));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFD, 0x24]
        );
    }

    #[test]
    fn test_calculate_instruction_size_add_b() {
        let instruction = Instruction::Addb(Operand::Register(Register::R1));
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 2);
    }

    #[test]
    fn test_encode_instruction_add_b() {
        let instruction = Instruction::Addb(Operand::Register(Register::R1));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFD, 0x29] // 0x28 + 1
        );
    }

    #[test]
    fn test_calculate_instruction_size_sub_b() {
        let instruction = Instruction::Subb(Operand::Register(Register::R2));
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 2);
    }

    #[test]
    fn test_encode_instruction_sub_b() {
        let instruction = Instruction::Subb(Operand::Register(Register::R2));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFD, 0x32] // 0x30 + 2
        );
    }

    #[test]
    fn test_calculate_instruction_size_and_b() {
        let instruction = Instruction::Andb(Operand::Register(Register::R3));
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 2);
    }

    #[test]
    fn test_encode_instruction_and_b() {
        let instruction = Instruction::Andb(Operand::Register(Register::R3));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFD, 0x3B] // 0x38 + 3
        );
    }

    #[test]
    fn test_calculate_instruction_size_or_b() {
        let instruction = Instruction::Orb(Operand::Register(Register::R4));
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 2);
    }

    #[test]
    fn test_encode_instruction_or_b() {
        let instruction = Instruction::Orb(Operand::Register(Register::R4));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFD, 0x44] // 0x40 + 4
        );
    }

    #[test]
    fn test_calculate_instruction_size_xor_b() {
        let instruction = Instruction::Xorb(Operand::Register(Register::R5));
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 2);
    }

    #[test]
    fn test_encode_instruction_xor_b() {
        let instruction = Instruction::Xorb(Operand::Register(Register::R5));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFD, 0x4D] // 0x48 + 5
        );
    }

    #[test]
    fn test_calculate_instruction_size_cmp_b() {
        let instruction = Instruction::Cmpb(Operand::Register(Register::R6));
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 2);
    }

    #[test]
    fn test_encode_instruction_cmp_b() {
        let instruction = Instruction::Cmpb(Operand::Register(Register::R6));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFD, 0x56] // 0x50 + 6
        );
    }

    #[test]
    fn test_calculate_instruction_size_ldi_b() {
        let instruction =
            Instruction::Ldb(Operand::Register(Register::R1), Operand::Immediate(0xAB));
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 3);
    }

    #[test]
    fn test_encode_instruction_ldi_b() {
        let instruction =
            Instruction::Ldb(Operand::Register(Register::R1), Operand::Immediate(0xAB));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFD, 0xA1, 0xAB]
        );
    }

    #[test]
    fn test_calculate_instruction_size_bit_register() {
        let instruction = Instruction::Bit(Operand::Register(Register::R1), Operand::Immediate(7));
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 3);
    }

    #[test]
    fn test_encode_instruction_bit_register() {
        let instruction = Instruction::Bit(Operand::Register(Register::R1), Operand::Immediate(7));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFD, 0x5F, 0x01]
        );
    }

    #[test]
    fn test_calculate_instruction_size_set_absolute() {
        let instruction = Instruction::Set(Operand::Absolute(0x1234), Operand::Immediate(0));
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 4);
    }

    #[test]
    fn test_encode_instruction_set_absolute() {
        let instruction = Instruction::Set(Operand::Absolute(0x1234), Operand::Immediate(0));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFD, 0x78, 0x34, 0x12]
        );
    }

    #[test]
    fn test_calculate_instruction_size_res_indirect() {
        let instruction = Instruction::Res(Operand::Indirect(Register::R2), Operand::Immediate(3));
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 3);
    }

    #[test]
    fn test_encode_instruction_res_indirect() {
        let instruction = Instruction::Res(Operand::Indirect(Register::R2), Operand::Immediate(3));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFD, 0x9B, 0x02]
        );
    }

    #[test]
    fn test_calculate_instruction_size_ld_indexed() {
        let instruction = Instruction::Ld(
            Operand::Register(Register::R0),
            Operand::Indexed(Register::R1, 16),
        );
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 3);
    }

    #[test]
    fn test_encode_instruction_ld_indexed() {
        let instruction = Instruction::Ld(
            Operand::Register(Register::R0),
            Operand::Indexed(Register::R1, 16),
        );
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFF, 0x01, 0x10]
        );
    }

    #[test]
    fn test_calculate_instruction_size_st_indexed() {
        let instruction = Instruction::St(
            Operand::Indexed(Register::R2, -1),
            Operand::Register(Register::R3),
        );
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 3);
    }

    #[test]
    fn test_encode_instruction_st_indexed() {
        let instruction = Instruction::St(
            Operand::Indexed(Register::R2, -1),
            Operand::Register(Register::R3),
        );
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFF, 0x53, 0xFF]
        );
    }

    #[test]
    fn test_calculate_instruction_size_lea_indexed() {
        let instruction = Instruction::Lea(
            Operand::Register(Register::R4),
            Operand::Indexed(Register::R5, 32),
        );
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 3);
    }

    #[test]
    fn test_encode_instruction_lea_indexed() {
        let instruction = Instruction::Lea(
            Operand::Register(Register::R4),
            Operand::Indexed(Register::R5, 32),
        );
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFF, 0xA5, 0x20]
        );
    }

    #[test]
    fn test_calculate_instruction_size_ld_post_increment() {
        let instruction = Instruction::Ld(
            Operand::Register(Register::R6),
            Operand::PostIncrement(Register::R7),
        );
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 3);
    }

    #[test]
    fn test_encode_instruction_ld_post_increment() {
        let instruction = Instruction::Ld(
            Operand::Register(Register::R6),
            Operand::PostIncrement(Register::R7),
        );
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFF, 0xC7, 0x06]
        );
    }

    #[test]
    fn test_calculate_instruction_size_st_post_increment() {
        let instruction = Instruction::St(
            Operand::PostIncrement(Register::R0),
            Operand::Register(Register::R1),
        );
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 3);
    }

    #[test]
    fn test_encode_instruction_st_post_increment() {
        let instruction = Instruction::St(
            Operand::PostIncrement(Register::R0),
            Operand::Register(Register::R1),
        );
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFF, 0xC9, 0x00]
        );
    }

    #[test]
    fn test_calculate_instruction_size_ld_pre_decrement() {
        let instruction = Instruction::Ld(
            Operand::Register(Register::R2),
            Operand::PreDecrement(Register::R3),
        );
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 3);
    }

    #[test]
    fn test_encode_instruction_ld_pre_decrement() {
        let instruction = Instruction::Ld(
            Operand::Register(Register::R2),
            Operand::PreDecrement(Register::R3),
        );
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFF, 0xD3, 0x02]
        );
    }

    #[test]
    fn test_calculate_instruction_size_st_pre_decrement() {
        let instruction = Instruction::St(
            Operand::PreDecrement(Register::R4),
            Operand::Register(Register::R5),
        );
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 3);
    }

    #[test]
    fn test_encode_instruction_st_pre_decrement() {
        let instruction = Instruction::St(
            Operand::PreDecrement(Register::R4),
            Operand::Register(Register::R5),
        );
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFF, 0xDD, 0x04]
        );
    }

    #[test]
    fn test_calculate_instruction_size_ld_b_post_increment() {
        let instruction = Instruction::Ldb(
            Operand::Register(Register::R6),
            Operand::PostIncrement(Register::R7),
        );
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 3);
    }

    #[test]
    fn test_encode_instruction_ld_b_post_increment() {
        let instruction = Instruction::Ldb(
            Operand::Register(Register::R6),
            Operand::PostIncrement(Register::R7),
        );
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFF, 0xE7, 0x06]
        );
    }

    #[test]
    fn test_calculate_instruction_size_st_b_post_increment() {
        let instruction = Instruction::Stb(
            Operand::PostIncrement(Register::R0),
            Operand::Register(Register::R1),
        );
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 3);
    }

    #[test]
    fn test_encode_instruction_st_b_post_increment() {
        let instruction = Instruction::Stb(
            Operand::PostIncrement(Register::R0),
            Operand::Register(Register::R1),
        );
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFF, 0xE9, 0x00]
        );
    }

    #[test]
    fn test_calculate_instruction_size_ld_b_pre_decrement() {
        let instruction = Instruction::Ldb(
            Operand::Register(Register::R2),
            Operand::PreDecrement(Register::R3),
        );
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 3);
    }

    #[test]
    fn test_encode_instruction_ld_b_pre_decrement() {
        let instruction = Instruction::Ldb(
            Operand::Register(Register::R2),
            Operand::PreDecrement(Register::R3),
        );
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFF, 0xF3, 0x02]
        );
    }

    #[test]
    fn test_calculate_instruction_size_st_b_pre_decrement() {
        let instruction = Instruction::Stb(
            Operand::PreDecrement(Register::R4),
            Operand::Register(Register::R5),
        );
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 3);
    }

    #[test]
    fn test_encode_instruction_st_b_pre_decrement() {
        let instruction = Instruction::Stb(
            Operand::PreDecrement(Register::R4),
            Operand::Register(Register::R5),
        );
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFF, 0xFD, 0x04]
        );
    }

    #[test]
    fn test_encode_instruction_call_far() {
        let instruction = Instruction::CallFar(Operand::Label("test_label".to_string()), None);
        let mut symbol_table = SymbolTable::new();
        symbol_table.insert(
            "test_label".to_string(),
            Symbol {
                logical_address: 0x4321,
                bank: 1,
            },
        );
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0x1100, &0, 0).unwrap(),
            vec![0x05, 0x01, 0x00, 0x06, 0x21, 0x43, 0x4E, 0x21]
        );
    }

    #[test]
    fn test_encode_instruction_call_far_bank_0_fail() {
        let instruction = Instruction::CallFar(Operand::Label("test_label".to_string()), None);
        let mut symbol_table = SymbolTable::new();
        symbol_table.insert(
            "test_label".to_string(),
            Symbol {
                logical_address: 0x2100,
                bank: 0,
            },
        );
        let result = encode_instruction(&instruction, &symbol_table, &0x5432, &1, 1);
        assert!(result.is_err_and(|e| {
            e == AssemblyError::SemanticError {
                line: 1,
                reason:
                    "Label \"test_label\" exists in bank 0, use a normal CALL instruction instead."
                        .to_string(),
            }
        }));
    }

    #[test]
    fn test_encode_instruction_call_far_same_bank_fail() {
        let instruction = Instruction::CallFar(Operand::Label("test_label".to_string()), None);
        let mut symbol_table = SymbolTable::new();
        symbol_table.insert(
            "test_label".to_string(),
            Symbol {
                logical_address: 0x4321,
                bank: 1,
            },
        );
        let result = encode_instruction(&instruction, &symbol_table, &0x5432, &1, 1);
        assert!(result.is_err_and(|e| {
            e == AssemblyError::SemanticError {
                line: 1,
                reason:
                    "Label \"test_label\" exists in the same bank as the CALL.far instruction, use a normal CALL instruction instead."
                        .to_string(),
            }
        }));
    }

    #[test]
    fn test_encode_instruction_call_far_via() {
        let instruction = Instruction::CallFar(
            Operand::Label("test_label".to_string()),
            Some(Operand::Label("tramp".to_string())),
        );
        let mut symbol_table = SymbolTable::new();
        symbol_table.insert(
            "test_label".to_string(),
            Symbol {
                logical_address: 0x4321,
                bank: 1,
            },
        );
        symbol_table.insert(
            "tramp".to_string(),
            Symbol {
                logical_address: 0x0200,
                bank: 0,
            },
        );
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0x1100, &0, 0).unwrap(),
            vec![0x05, 0x01, 0x00, 0x06, 0x21, 0x43, 0xC8, 0x00, 0x02]
        );
    }

    #[test]
    fn test_encode_instruction_call_far_via_bank_0_fail() {
        let instruction = Instruction::CallFar(
            Operand::Label("test_label".to_string()),
            Some(Operand::Label("tramp".to_string())),
        );
        let mut symbol_table = SymbolTable::new();
        symbol_table.insert(
            "test_label".to_string(),
            Symbol {
                logical_address: 0x2100,
                bank: 0,
            },
        );
        symbol_table.insert(
            "tramp".to_string(),
            Symbol {
                logical_address: 0x0200,
                bank: 0,
            },
        );
        let result = encode_instruction(&instruction, &symbol_table, &0x5432, &1, 1);
        assert!(result.is_err_and(|e| {
            e == AssemblyError::SemanticError {
                line: 1,
                reason:
                    "Label \"test_label\" exists in bank 0, use a normal CALL instruction instead."
                        .to_string(),
            }
        }));
    }

    #[test]
    fn test_encode_instruction_call_far_via_same_bank_fail() {
        let instruction = Instruction::CallFar(
            Operand::Label("test_label".to_string()),
            Some(Operand::Label("tramp".to_string())),
        );
        let mut symbol_table = SymbolTable::new();
        symbol_table.insert(
            "test_label".to_string(),
            Symbol {
                logical_address: 0x4321,
                bank: 1,
            },
        );
        symbol_table.insert(
            "tramp".to_string(),
            Symbol {
                logical_address: 0x0200,
                bank: 0,
            },
        );
        let result = encode_instruction(&instruction, &symbol_table, &0x5432, &1, 1);
        assert!(result.is_err_and(|e| {
            e == AssemblyError::SemanticError {
                line: 1,
                reason:
                    "Label \"test_label\" exists in the same bank as the CALL.far instruction, use a normal CALL instruction instead."
                        .to_string(),
            }
        }));
    }

    #[test]
    fn test_encode_instruction_call_far_via_invalid_bank_fail() {
        let instruction = Instruction::CallFar(
            Operand::Label("test_label".to_string()),
            Some(Operand::Label("tramp".to_string())),
        );
        let mut symbol_table = SymbolTable::new();
        symbol_table.insert(
            "test_label".to_string(),
            Symbol {
                logical_address: 0x4400,
                bank: 2,
            },
        );
        symbol_table.insert(
            "tramp".to_string(),
            Symbol {
                logical_address: 0x4848,
                bank: 1,
            },
        );
        let result = encode_instruction(&instruction, &symbol_table, &0x5432, &1, 1);
        assert!(result.is_err_and(|e| {
            e == AssemblyError::SemanticError {
                line: 1,
                reason:
                    "Custom CALL.far via trampoline label must exist in bank 0, \"tramp\" found in bank 1"
                        .to_string(),
            }
        }));
    }
}
