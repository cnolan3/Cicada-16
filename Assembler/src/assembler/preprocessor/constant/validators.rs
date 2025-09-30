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

use crate::assembler::AssemblyError;
use crate::assembler::ConstantTable;
use crate::ast::*;

pub fn replace_constant_with_word(
    op: &mut Operand,
    constant_table: &ConstantTable,
    line_number: &usize,
) -> Result<(), AssemblyError> {
    if let Operand::Label(label) = op {
        if let Some(val) = constant_table.get(label) {
            if *val > u16::MAX as i32 || *val < 0 {
                return Err(AssemblyError::SemanticError {
                    line: *line_number,
                    reason: format!(
                        "Expected constant to be an unsigned 16 bit value (max: {}, min: 0)",
                        u16::MAX
                    ),
                });
            }
            *op = Operand::Immediate(*val);
        }
    }
    Ok(())
}

pub fn replace_constant_with_signed_byte(
    op: &mut Operand,
    constant_table: &ConstantTable,
    line_number: &usize,
) -> Result<(), AssemblyError> {
    if let Operand::Label(label) = op {
        if let Some(val) = constant_table.get(label) {
            if *val > i8::MAX as i32 || *val < i8::MIN as i32 {
                return Err(AssemblyError::SemanticError {
                    line: *line_number,
                    reason: format!(
                        "Expected constant to be a signed 8 bit value (max: {}, min: {})",
                        i8::MAX,
                        i8::MIN
                    ),
                });
            }
            *op = Operand::Immediate(*val);
        }
    }
    Ok(())
}

pub fn replace_constant_with_unsigned_byte(
    op: &mut Operand,
    constant_table: &ConstantTable,
    line_number: &usize,
) -> Result<(), AssemblyError> {
    if let Operand::Label(label) = op {
        if let Some(val) = constant_table.get(label) {
            if *val > u8::MAX as i32 || *val < u8::MIN as i32 {
                return Err(AssemblyError::SemanticError {
                    line: *line_number,
                    reason: format!(
                        "Expected constant to be an unsigned 8 bit value (max: {}, min: 0)",
                        u8::MAX
                    ),
                });
            }
            *op = Operand::Immediate(*val);
        }
    }
    Ok(())
}
