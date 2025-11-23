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

use crate::ast::HeaderInfo;
use crate::ast::SectionOptions;
use crate::parser::AstBuilder;
use crate::parser::Rule;
use crate::parser::ast_builder::AssemblyError;
use crate::parser::ast_builder::constants::*;
use crate::parser::ast_builder::utility_functions::*;
use crate::parser::{Directive, Operand};
use anyhow::{Context, Result};

impl<'a> AstBuilder<'a> {
    // build an origin directive
    pub fn build_org_directive(mut self) -> Result<Directive> {
        let addr = self.expect_addr_or_label().context(INVALID_OP_MSG)?;

        Ok(Directive::Org(addr))
    }

    // build a bank directive
    pub fn build_bank_directive(mut self) -> Result<Directive> {
        let id = self.pop_operand().context(INVALID_OP_MSG)?;

        match id {
            Operand::Immediate(val) => {
                if val > 256 || val < 0 {
                    Err(AssemblyError::StructuralError {
                        line: self.line_number,
                        reason: ".bank number must be an unsigned value between 0 and 256"
                            .to_string(),
                    }
                    .into())
                } else {
                    Ok(Directive::Bank(id))
                }
            }
            Operand::Label(_) => Ok(Directive::Bank(id)),
            _ => Err(AssemblyError::StructuralError {
                line: self.line_number,
                reason: ".bank argument must be an immediate value.".to_string(),
            }
            .into()),
        }
    }

    // build a byte data directive
    pub fn build_byte_directive(mut self) -> Result<Directive> {
        let ops = self.expect_op_vector().context("Invalid byte list.")?;
        let mut bytes: Vec<Operand> = Vec::new();

        for op in ops {
            match op {
                Operand::Immediate(val) => {
                    check_unsigned_byte(val, self.line_number)?;
                    bytes.push(op);
                }
                Operand::Label(_) => bytes.push(op),
                _ => {
                    return Err(AssemblyError::StructuralError {
                        line: self.line_number,
                        reason: ".byte data must be a list of immediate values.".to_string(),
                    }
                    .into());
                }
            }
        }

        Ok(Directive::Byte(bytes))
    }

    // build a word data directive
    pub fn build_word_directive(mut self) -> Result<Directive> {
        let ops = self.expect_op_vector().context("Invalid word list.")?;
        let mut words: Vec<Operand> = Vec::new();

        for op in ops {
            match op {
                Operand::Immediate(val) => {
                    check_unsigned_word(val, self.line_number)
                        .with_context(|| format!("Invalid word value: {}", val))?;
                    words.push(op);
                }
                Operand::Label(_) => words.push(op),
                _ => {
                    return Err(AssemblyError::StructuralError {
                        line: self.line_number,
                        reason: ".word data must be a list of immediate values or labels."
                            .to_string(),
                    }
                    .into());
                }
            }
        }

        Ok(Directive::Word(words))
    }

    // build a word data directive
    pub fn build_define_directive(mut self) -> Result<Directive> {
        let label = self.expect_label().context("Invalid define label.")?;
        let value = self.pop_operand().context("Invalid define value.")?;

        match value {
            Operand::Immediate(_) => Ok(Directive::Define(label, value)),
            // TODO: allow more value operand types in the future, like expressions
            _ => {
                return Err(AssemblyError::StructuralError {
                    line: self.line_number,
                    reason: ".define value must be a number.".to_string(),
                }
                .into());
            }
        }
    }

    // build an include directive
    pub fn build_include_directive(mut self) -> Result<Directive> {
        let op = self.pop_operand().context("Invalid include value.")?;

        match op {
            Operand::String(s) => Ok(Directive::Include(s)),
            _ => {
                return Err(AssemblyError::StructuralError {
                    line: self.line_number,
                    reason: ".include value must be a path string.".to_string(),
                }
                .into());
            }
        }
    }

    // build an incbin directive
    pub fn build_incbin_directive(mut self) -> Result<Directive> {
        let op = self.pop_operand().context("Invalid incbin value.")?;

        match op {
            Operand::String(s) => Ok(Directive::Incbin(s)),
            _ => {
                return Err(AssemblyError::StructuralError {
                    line: self.line_number,
                    reason: ".incbin value must be a path string.".to_string(),
                }
                .into());
            }
        }
    }

    // build a header info block directive
    pub fn build_header_directive(self) -> Result<Directive> {
        let mut info = HeaderInfo::default();

        for info_line in self.pairs {
            if info_line.as_rule() != Rule::header_info {
                continue; // Skip comments, newlines, etc.
            }

            let line_number = info_line.as_span().start_pos().line_col().0;
            let Some(sub_directive) = info_line.into_inner().next() else {
                continue;
            };

            let mut field_builder = AstBuilder::new(sub_directive.clone());

            match sub_directive.as_rule() {
                Rule::boot_anim => {
                    let val = field_builder.expect_string_literal()?;
                    if val.len() != 4 {
                        return Err(AssemblyError::StructuralError {
                            line: line_number,
                            reason: ".boot_anim must be exactly 4 characters.".to_string(),
                        }
                        .into());
                    }
                    info.boot_anim = val;
                }
                Rule::title => {
                    let val = field_builder.expect_string_literal()?;
                    if val.len() > 16 {
                        return Err(AssemblyError::StructuralError {
                            line: line_number,
                            reason: ".title must be 16 characters or less.".to_string(),
                        }
                        .into());
                    }
                    info.title = val;
                }
                Rule::developer => {
                    let val = field_builder.expect_string_literal()?;
                    if val.len() > 16 {
                        return Err(AssemblyError::StructuralError {
                            line: line_number,
                            reason: ".developer must be 16 characters or less.".to_string(),
                        }
                        .into());
                    }
                    info.developer = val;
                }
                Rule::version => {
                    info.version = field_builder.expect_unsigned_byte()?;
                }
                Rule::mapper => {
                    info.mapper = field_builder.expect_unsigned_byte()?;
                }
                Rule::rom_size => {
                    info.rom_size = field_builder.expect_unsigned_byte()?;
                }
                Rule::ram_size => {
                    info.ram_size = field_builder.expect_unsigned_byte()?;
                }
                Rule::interrupt_mode => {
                    let val = field_builder.expect_immediate()? as u8;
                    if val > 1 {
                        return Err(AssemblyError::StructuralError {
                            line: line_number,
                            reason: ".interrupt_mode must be 0 or 1.".to_string(),
                        }
                        .into());
                    }
                    info.interrupt_mode = val;
                }
                Rule::hardware_rev => {
                    let val = field_builder.expect_immediate()? as u8;
                    if val > 3 {
                        return Err(AssemblyError::StructuralError {
                            line: line_number,
                            reason:
                                ".interrupt_mode must be an unsigned 2 bit value (max: 3, min: 0)."
                                    .to_string(),
                        }
                        .into());
                    }
                    info.hardware_rev = val;
                }
                Rule::region => {
                    let val = field_builder.expect_immediate()? as u8;
                    if val > 7 {
                        return Err(AssemblyError::StructuralError {
                            line: line_number,
                            reason:
                                ".interrupt_mode must be an unsigned 3 bit value (max: 7, min: 0)."
                                    .to_string(),
                        }
                        .into());
                    }
                    info.region = val;
                }
                _ => {
                    return Err(AssemblyError::StructuralError {
                        line: line_number,
                        reason: "Unknown header field directive.".to_string(),
                    }
                    .into());
                }
            }
        }

        Ok(Directive::Header(info))
    }

    // build an interrupt vector table block directive
    pub fn build_interrupt_directive(self) -> Result<Directive> {
        let mut op_table: Vec<Operand> = Vec::new();

        for table_line in self.pairs {
            let line_number = table_line.as_span().start_pos().line_col().0;

            if let Rule::word_directive = table_line.as_rule() {
                let field_builder = AstBuilder::new(table_line.clone());
                let word = field_builder.build_word_directive()?;

                if let Directive::Word(data) = word {
                    op_table.extend(data);
                } else {
                    return Err(AssemblyError::StructuralError {
                        line: line_number,
                        reason: "Invalid word data.".to_string(),
                    }
                    .into());
                }
            } else {
                return Err(AssemblyError::StructuralError {
                    line: line_number,
                    reason: "Fields of a .interrupt_table directive must be .word directives."
                        .to_string(),
                }
                .into());
            }
        }

        if op_table.len() < 12 || op_table.len() > 16 {
            Err(AssemblyError::StructuralError {
                line: self.line_number,
                reason: "Vector interrupt table must contain at least 12 entries and at most 16 entries."
                    .to_string(),
            }
            .into())
        } else {
            Ok(Directive::Interrupt(op_table))
        }
    }

    pub fn build_section_start_directive(self) -> Result<Directive> {
        let mut section_options: SectionOptions = SectionOptions::default();

        for pair in self.pairs {
            let attr = pair
                .into_inner()
                .next()
                .ok_or_else(|| AssemblyError::StructuralError {
                    line: self.line_number,
                    reason: "Expected an attribute.".to_string(),
                })?;

            let mut field_builder = AstBuilder::new(attr.clone());

            match attr.as_rule() {
                Rule::name_attr => {
                    if section_options.name.is_some() {
                        return Err(AssemblyError::StructuralError {
                            line: self.line_number,
                            reason: ".section name attribute defined multiple times.".to_string(),
                        }
                        .into());
                    }
                    section_options.name = Some(field_builder.expect_string_literal()?);
                }
                Rule::size_attr => {
                    if section_options.size.is_some() {
                        return Err(AssemblyError::StructuralError {
                            line: self.line_number,
                            reason: ".section size attribute defined multiple times.".to_string(),
                        }
                        .into());
                    }
                    section_options.size = Some(field_builder.expect_immediate()? as u32);
                }
                Rule::vaddr_attr => {
                    if section_options.vaddr.is_some() {
                        return Err(AssemblyError::StructuralError {
                            line: self.line_number,
                            reason: ".section vaddr attribute defined multiple times.".to_string(),
                        }
                        .into());
                    }
                    section_options.vaddr = Some(field_builder.expect_addr()? as u32);
                }
                Rule::paddr_attr => {
                    if section_options.paddr.is_some() {
                        return Err(AssemblyError::StructuralError {
                            line: self.line_number,
                            reason: ".section paddr attribute defined multiple times.".to_string(),
                        }
                        .into());
                    }
                    section_options.paddr = Some(field_builder.expect_addr()? as u32);
                }
                Rule::align_attr => {
                    if section_options.align.is_some() {
                        return Err(AssemblyError::StructuralError {
                            line: self.line_number,
                            reason: ".section align attribute defined multiple times.".to_string(),
                        }
                        .into());
                    }
                    let alignment = field_builder.expect_immediate()?;

                    if alignment <= 0 {
                        return Err(AssemblyError::StructuralError {
                            line: self.line_number,
                            reason: ".section align value must be greater than zero.".to_string(),
                        }
                        .into());
                    }

                    section_options.align = Some(alignment as u32);
                }
                _ => {}
            }
        }
        Ok(Directive::SectionStart(section_options))
    }

    pub fn build_align_directive(mut self) -> Result<Directive> {
        let alignment = self.expect_immediate().context(INVALID_OP_MSG)?;

        if alignment <= 0 {
            return Err(AssemblyError::StructuralError {
                line: self.line_number,
                reason: ".align value must be greater than zero.".to_string(),
            }
            .into());
        }

        Ok(Directive::Align(alignment as u32))
    }
}
