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
    // build and check operands for a sra instruction
    pub fn build_sra(mut self) -> Result<Instruction> {
        let r = self.expect_register().context(INVALID_OP_MSG)?;

        Ok(Instruction::Sra(r))
    }

    // build and check operands for a shl instruction
    pub fn build_shl(mut self) -> Result<Instruction> {
        let r = self.expect_register().context(INVALID_OP_MSG)?;

        Ok(Instruction::Shl(r))
    }

    // build and check operands for a shr instruction
    pub fn build_shr(mut self) -> Result<Instruction> {
        let r = self.expect_register().context(INVALID_OP_MSG)?;

        Ok(Instruction::Shr(r))
    }

    // build and check operands for a rol instruction
    pub fn build_rol(mut self) -> Result<Instruction> {
        let r = self.expect_register().context(INVALID_OP_MSG)?;

        Ok(Instruction::Rol(r))
    }

    // build and check operands for a ror instruction
    pub fn build_ror(mut self) -> Result<Instruction> {
        let r = self.expect_register().context(INVALID_OP_MSG)?;

        Ok(Instruction::Ror(r))
    }

    // build and check operands for a bit check instruction
    pub fn build_bit(mut self) -> Result<Instruction> {
        let op = self.pop_operand().context(INVALID_OP_MSG)?;
        let bit = self
            .expect_bit_id_or_label()
            .context("Invalid bit ID operand.")?;

        match op {
            Operand::Register(r) => Ok(Instruction::BitReg(r, bit)),
            Operand::Indirect(r) => Ok(Instruction::BitIndirect(r, bit)),
            Operand::AbsAddr(addr) => Ok(Instruction::BitAbs(Operand::Immediate(addr as i32), bit)),
            Operand::AbsLabel(label) => Ok(Instruction::BitAbs(Operand::Label(label), bit)),
            _ => {
                Err(AssemblyError::StructuralError {
                    line: self.line_number,
                    reason: "BIT destination operand must be a register, indirect address or absolute address".to_string(),
                }.into())
            }
        }
    }

    // build and check operands for a set instruction
    pub fn build_set(mut self) -> Result<Instruction> {
        let op = self.pop_operand().context(INVALID_OP_MSG)?;
        let bit = self
            .expect_bit_id_or_label()
            .context("Invalid bit ID operand.")?;

        match op {
            Operand::Register(r) => Ok(Instruction::SetReg(r, bit)),
            Operand::Indirect(r) => Ok(Instruction::SetIndirect(r, bit)),
            Operand::AbsAddr(addr) => Ok(Instruction::SetAbs(Operand::Immediate(addr as i32), bit)),
            Operand::AbsLabel(label) => Ok(Instruction::SetAbs(Operand::Label(label), bit)),
            _ => {
                Err(AssemblyError::StructuralError {
                    line: self.line_number,
                    reason: "SET destination operand must be a register, indirect address or absolute address".to_string(),
                }.into())
            }
        }
    }

    // build and check operands for a res instruction
    pub fn build_res(mut self) -> Result<Instruction> {
        let op = self.pop_operand().context(INVALID_OP_MSG)?;
        let bit = self
            .expect_bit_id_or_label()
            .context("Invalid bit ID operand.")?;

        match op {
            Operand::Register(r) => Ok(Instruction::ResReg(r, bit)),
            Operand::Indirect(r) => Ok(Instruction::ResIndirect(r, bit)),
            Operand::AbsAddr(addr) => Ok(Instruction::ResAbs(Operand::Immediate(addr as i32), bit)),
            Operand::AbsLabel(label) => Ok(Instruction::ResAbs(Operand::Label(label), bit)),
            _ => {
                Err(AssemblyError::StructuralError {
                    line: self.line_number,
                    reason: "RES destination operand must be a register, indirect address or absolute address".to_string(),
                }.into())
            }
        }
    }
}
