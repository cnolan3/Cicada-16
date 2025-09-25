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
    // build and check operands for a push instruction
    pub fn build_push(mut self) -> Result<Instruction> {
        let op = self.pop_operand().context(INVALID_OP_MSG)?;

        match op {
            Operand::Register(r) => Ok(Instruction::Push(r)),
            Operand::Immediate(value) => {
                check_unsigned_word(value, self.line_number)?;
                Ok(Instruction::PushI(op))
            }
            Operand::Label(_) => Ok(Instruction::PushI(op)),
            _ => Err(AssemblyError::StructuralError {
                line: self.line_number,
                reason: "Invalid operand to PUSH instruction.".to_string(),
            }
            .into()),
        }
    }

    // build and check operands for a pop instruction
    pub fn build_pop(mut self) -> Result<Instruction> {
        let r = self.expect_register().context(INVALID_OP_MSG)?;

        Ok(Instruction::Pop(r))
    }
}
