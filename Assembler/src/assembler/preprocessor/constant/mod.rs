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

mod validators;

use crate::assembler::AssemblyError;
use crate::assembler::ConstantTable;
use crate::ast::*;
use validators::*;

pub fn process_instruction_constants(
    instruction: &mut Instruction,
    constant_table: &ConstantTable,
    line_number: &usize,
) -> Result<(), AssemblyError> {
    match instruction {
        Instruction::Ldi(_, op) | Instruction::LdAbs(_, op) => {
            replace_constant_with_word(op, constant_table, line_number)?;
        }
        Instruction::LdIndexed(_, _, op) => {
            replace_constant_with_signed_byte(op, constant_table, line_number)?;
        }
        Instruction::StAbs(op, _) => {
            replace_constant_with_word(op, constant_table, line_number)?;
        }
        Instruction::StIndexed(_, op, _) => {
            replace_constant_with_signed_byte(op, constant_table, line_number)?;
        }
        Instruction::LdiB(_, op) => {
            replace_constant_with_unsigned_byte(op, constant_table, line_number)?;
        }
        Instruction::Lea(_, _, op) => {
            replace_constant_with_signed_byte(op, constant_table, line_number)?;
        }
        Instruction::PushI(op) => {
            replace_constant_with_word(op, constant_table, line_number)?;
        }
        Instruction::AddAccI(op)
        | Instruction::SubAccI(op)
        | Instruction::AndAccI(op)
        | Instruction::OrAccI(op)
        | Instruction::XorAccI(op)
        | Instruction::CmpAccI(op)
        | Instruction::AdcAccI(op)
        | Instruction::SbcAccI(op) => {
            replace_constant_with_word(op, constant_table, line_number)?;
        }
        Instruction::AddIReg(_, op)
        | Instruction::SubIReg(_, op)
        | Instruction::AndIReg(_, op)
        | Instruction::OrIReg(_, op)
        | Instruction::XorIReg(_, op)
        | Instruction::CmpIReg(_, op)
        | Instruction::AddSp(op) => {
            replace_constant_with_word(op, constant_table, line_number)?;
        }
        Instruction::BitReg(_, op)
        | Instruction::SetReg(_, op)
        | Instruction::ResReg(_, op)
        | Instruction::BitIndirect(_, op)
        | Instruction::SetIndirect(_, op)
        | Instruction::ResIndirect(_, op) => {
            replace_constant_with_signed_byte(op, constant_table, line_number)?;
        }
        Instruction::BitAbs(abs, bit)
        | Instruction::SetAbs(abs, bit)
        | Instruction::ResAbs(abs, bit) => {
            replace_constant_with_word(abs, constant_table, line_number)?;
            replace_constant_with_signed_byte(bit, constant_table, line_number)?;
        }
        Instruction::JmpI(op) | Instruction::JccI(_, op) => {
            replace_constant_with_word(op, constant_table, line_number)?;
        }
        Instruction::JrI(op) | Instruction::JrccI(_, op) | Instruction::Djnz(op) => {
            replace_constant_with_signed_byte(op, constant_table, line_number)?;
        }
        Instruction::CallI(op) | Instruction::CallccI(_, op) => {
            replace_constant_with_word(op, constant_table, line_number)?;
        }
        Instruction::Syscall(op) => {
            replace_constant_with_word(op, constant_table, line_number)?;
        }
        _ => {}
    }
    Ok(())
}

pub fn process_directive_constants(
    directive: &mut Directive,
    constant_table: &ConstantTable,
    line_number: &usize,
) -> Result<(), AssemblyError> {
    match directive {
        Directive::Org(op) => {
            replace_constant_with_word(op, constant_table, line_number)?;
        }
        Directive::Byte(ops) => {
            for op in ops {
                replace_constant_with_unsigned_byte(op, constant_table, line_number)?;
            }
        }
        Directive::Word(ops) => {
            for op in ops {
                replace_constant_with_word(op, constant_table, line_number)?;
            }
        }
        Directive::Interrupt(ops) => {
            for op in ops {
                replace_constant_with_word(op, constant_table, line_number)?;
            }
        }
        _ => {}
    }
    Ok(())
}
