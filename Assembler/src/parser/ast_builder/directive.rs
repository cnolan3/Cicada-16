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

use crate::parser::AstBuilder;
use crate::parser::ast_builder::AssemblyError;
use crate::parser::ast_builder::constants::*;
use crate::parser::ast_builder::utility_functions::*;
use crate::parser::{Directive, Operand};
use anyhow::{Context, Result};
use std::u16;

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
                    Ok(Directive::Bank(val as u16))
                }
            }
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
        let mut bytes: Vec<u8> = Vec::new();

        for op in ops {
            match op {
                Operand::Immediate(val) => {
                    check_unsigned_byte(val, self.line_number)?;
                    bytes.push(val as u8);
                }
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
}
