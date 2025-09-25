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
use crate::parser::{Instruction, Operand};
use anyhow::{Context, Result};

impl<'a> AstBuilder<'a> {
    // build and check operands for a jump instruction
    pub fn build_jmp(mut self) -> Result<Instruction> {
        let op = self.pop_operand().context(INVALID_OP_MSG)?;

        match op {
            Operand::Indirect(r) => Ok(Instruction::JmpIndirect(r)),
            Operand::Label(_) => Ok(Instruction::JmpI(op)),
            Operand::Immediate(imm) => {
                check_unsigned_word(imm, self.line_number)?;
                Ok(Instruction::JmpI(op))
            }
            _ => {
                Err(AssemblyError::StructuralError {
                line: self.line_number,
                reason:
                    "Operand to a JMP instruction must be an indirect address ((R0-R7)), label or immediate address." .to_string(),
                }.into())
            }
        }
    }

    // build and check operands for a jump relative instruction
    pub fn build_jr(mut self) -> Result<Instruction> {
        let op = self.expect_sbyte_or_label().context(INVALID_OP_MSG)?;

        Ok(Instruction::JrI(op))
    }

    // build and check operands for a conditional jump instruction
    pub fn build_jcc(mut self) -> Result<Instruction> {
        let cc = self.pop_cc().context("Invalid condidtion code.")?;
        let op = self.expect_addr_or_label().context(INVALID_OP_MSG)?;

        Ok(Instruction::JccI(cc, op))
    }

    // build and check operands for a conditional jump relative instruction
    pub fn build_jrcc(mut self) -> Result<Instruction> {
        let cc = self.pop_cc().context("Invalid condidtion code.")?;
        let op = self.expect_sbyte_or_label().context(INVALID_OP_MSG)?;

        Ok(Instruction::JrccI(cc, op))
    }

    // build and check operands for a DJNZ instruction
    pub fn build_djnz(mut self) -> Result<Instruction> {
        let op = self.expect_sbyte_or_label().context(INVALID_OP_MSG)?;

        Ok(Instruction::Djnz(op))
    }

    // build and check operands for a call instruction
    pub fn build_call(mut self) -> Result<Instruction> {
        let op = self.pop_operand().context(INVALID_OP_MSG)?;

        match op {
            Operand::Indirect(r) => Ok(Instruction::CallIndirect(r)),
            Operand::Label(_) => Ok(Instruction::CallI(op)),
            Operand::Immediate(imm) => {
                check_unsigned_word(imm, self.line_number)?;
                Ok(Instruction::CallI(op))
            }
            _ => {
                Err(AssemblyError::StructuralError {
                line: self.line_number,
                reason:
                    "Operand to a CALL instruction must be an indirect address ((R0-R7)), label or immediate address." .to_string(),
                }.into())
            }
        }
    }

    // build and check operands for a conditional call instruction
    pub fn build_callcc(mut self) -> Result<Instruction> {
        let cc = self.pop_cc().context("Invalid condition code.")?;
        let op = self.expect_addr_or_label().context(INVALID_OP_MSG)?;

        Ok(Instruction::CallccI(cc, op))
    }

    // build and check operands for a SYSCALL instruction
    pub fn build_syscall(mut self) -> Result<Instruction> {
        let index = self.expect_unsigned_byte().context(INVALID_OP_MSG)?;

        Ok(Instruction::Syscall(index))
    }

    // ----------- Synthetic Control Flow Instructions -----------

    // build a call.far instruction
    pub fn build_call_far(mut self) -> Result<Instruction> {
        let label = self.expect_label().context(INVALID_OP_MSG)?;

        Ok(Instruction::CallFar(label))
    }

    // build a call.far via instruction
    pub fn build_call_far_via(mut self) -> Result<Instruction> {
        let call_label = self.expect_label().context(INVALID_OP_MSG)?;
        let via_label = self.expect_label().context(INVALID_OP_MSG)?;

        Ok(Instruction::CallFarVia(call_label, via_label))
    }

    // build a jump.far instruction
    pub fn build_jmp_far(mut self) -> Result<Instruction> {
        let label = self.expect_label().context(INVALID_OP_MSG)?;

        Ok(Instruction::JmpFar(label))
    }

    // build a jump.far via instruction
    pub fn build_jmp_far_via(mut self) -> Result<Instruction> {
        let call_label = self.expect_label().context(INVALID_OP_MSG)?;
        let via_label = self.expect_label().context(INVALID_OP_MSG)?;

        Ok(Instruction::JmpFarVia(call_label, via_label))
    }
}
