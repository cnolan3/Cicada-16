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
use crate::parser::{Instruction, Operand};
use anyhow::{Context, Result};

impl<'a> AstBuilder<'a> {
    // build and check operands for a load instruction
    pub fn build_ld(mut self) -> Result<Instruction> {
        let rd = self.expect_register().context(INVALID_DEST_OP_MSG)?;
        let src = self.pop_operand().context(INVALID_SRC_OP_MSG)?;

        match src {
            Operand::Register(rs) => Ok(Instruction::LdReg(rd, rs)),
            Operand::AbsAddr(addr) => Ok(Instruction::LdAbs(rd, Operand::Immediate(addr as i32))),
            Operand::AbsLabel(label) => Ok(Instruction::LdAbs(rd, Operand::Label(label))),
            Operand::Indexed(rs, offset) => Ok(Instruction::LdIndexed(
                rd,
                rs,
                Operand::Immediate(offset as i32),
            )),
            Operand::IndexedLabel(rs, label) => {
                Ok(Instruction::LdIndexed(rd, rs, Operand::Label(label)))
            }
            Operand::Indirect(rs) => Ok(Instruction::LdIndirect(rd, rs)),
            Operand::PreDecrement(rs) => Ok(Instruction::LdPreDec(rd, rs)),
            Operand::PostIncrement(rs) => Ok(Instruction::LdPostInc(rd, rs)),
            _ => Err(AssemblyError::StructuralError {
                line: self.line_number,
                reason: "Invalid source operand for LD instruction.".to_string(),
            }
            .into()),
        }
    }

    // build and check operands for a store instruction
    pub fn build_st(mut self) -> Result<Instruction> {
        let dest = self.pop_operand().context(INVALID_DEST_OP_MSG)?;
        let rs = self.expect_register().context(INVALID_SRC_OP_MSG)?;

        match dest {
            Operand::AbsAddr(addr) => Ok(Instruction::StAbs(Operand::Immediate(addr as i32), rs)),
            Operand::AbsLabel(label) => Ok(Instruction::StAbs(Operand::Label(label), rs)),
            Operand::Indirect(rd) => Ok(Instruction::StIndirect(rd, rs)),
            Operand::PostIncrement(rd) => Ok(Instruction::StPostInc(rd, rs)),
            Operand::PreDecrement(rd) => Ok(Instruction::StPreDec(rd, rs)),
            Operand::Indexed(rd, offset) => Ok(Instruction::StIndexed(
                rd,
                Operand::Immediate(offset as i32),
                rs,
            )),
            Operand::IndexedLabel(rd, label) => {
                Ok(Instruction::StIndexed(rd, Operand::Label(label), rs))
            }
            _ => Err(AssemblyError::StructuralError {
                line: self.line_number,
                reason: "Invalid desitination operand to ST instruction.".to_string(),
            }
            .into()),
        }
    }

    // build and check operands for a load immediate instruction
    pub fn build_ldi(mut self) -> Result<Instruction> {
        let rd = self.expect_register().context(INVALID_DEST_OP_MSG)?;
        let src = self.expect_addr_or_label().context(INVALID_SRC_OP_MSG)?;

        Ok(Instruction::Ldi(rd, src))
    }

    // build and check operands for a load byte instruction
    pub fn build_ld_b(mut self) -> Result<Instruction> {
        let rd = self.expect_register().context(INVALID_DEST_OP_MSG)?;
        let src = self.pop_operand().context(INVALID_SRC_OP_MSG)?;

        match src {
            Operand::AbsAddr(addr) => Ok(Instruction::LdBAbs(rd, Operand::Immediate(addr as i32))),
            Operand::AbsLabel(label) => Ok(Instruction::LdBAbs(rd, Operand::Label(label))),
            Operand::Indirect(rs) => Ok(Instruction::LdBIndirect(rd, rs)),
            Operand::PreDecrement(rs) => Ok(Instruction::LdBPreDec(rd, rs)),
            Operand::PostIncrement(rs) => Ok(Instruction::LdBPostInc(rd, rs)),
            _ => {
                Err(AssemblyError::StructuralError {
                    line: self.line_number,
                    reason: "The source operand of an LD.b instruction must be an indirect address or post-increment/pre-decrement value."
                        .to_string(),
                }.into())
            }
        }
    }

    // build and check operands for a store byte instruction
    pub fn build_st_b(mut self) -> Result<Instruction> {
        let dest = self.pop_operand().context(INVALID_DEST_OP_MSG)?;
        let rs = self.expect_register().context(INVALID_SRC_OP_MSG)?;

        match dest {
            Operand::AbsAddr(addr) => Ok(Instruction::StBAbs(Operand::Immediate(addr as i32), rs)),
            Operand::AbsLabel(label) => Ok(Instruction::StBAbs(Operand::Label(label), rs)),
            Operand::Indirect(rd) => Ok(Instruction::StBIndirect(rd, rs)),
            Operand::PreDecrement(rd) => Ok(Instruction::StBPreDec(rd, rs)),
            Operand::PostIncrement(rd) => Ok(Instruction::StBPostInc(rd, rs)),
            _ => {
                Err(AssemblyError::StructuralError {
                    line: self.line_number,
                    reason: "The destination operand of an ST.b instruction must be an indirect address or post-increment/pre-decrement value."
                        .to_string(),
                }.into())
            }
        }
    }

    // build and check operands for a load byte immediate instruction
    pub fn build_ldi_b(mut self) -> Result<Instruction> {
        let rd = self.expect_register().context(INVALID_DEST_OP_MSG)?;
        let src = self
            .expect_unsigned_byte_or_label()
            .context(INVALID_SRC_OP_MSG)?;

        Ok(Instruction::LdiB(rd, src))
    }

    // build and check operands for a lea instruction
    pub fn build_lea(mut self) -> Result<Instruction> {
        let rd = self.expect_register().context(INVALID_DEST_OP_MSG)?;
        let src = self.pop_operand().context(INVALID_SRC_OP_MSG)?;

        match src {
            Operand::Indexed(rs, offset) => {
                Ok(Instruction::Lea(rd, rs, Operand::Immediate(offset as i32)))
            }
            Operand::IndexedLabel(rs, label) => Ok(Instruction::Lea(rd, rs, Operand::Label(label))),
            _ => Err(AssemblyError::StructuralError {
                line: self.line_number,
                reason: "Invalid operands to LEA instruction.".to_string(),
            }
            .into()),
        }
    }
}
