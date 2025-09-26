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

use crate::parser::Register;
use crate::parser::Rule;
use crate::parser::ast_builder::AssemblyError;
use anyhow::Result;
use pest::iterators::Pair;

// translate a pair to a register
pub fn pair_to_reg(pair: Pair<Rule>) -> Result<Register> {
    let line = pair.as_span().start_pos().line_col().0;
    match pair.as_str() {
        "0" => Ok(Register::R0),
        "1" => Ok(Register::R1),
        "2" => Ok(Register::R2),
        "3" => Ok(Register::R3),
        "4" => Ok(Register::R4),
        "5" => Ok(Register::R5),
        "6" => Ok(Register::R6),
        "7" => Ok(Register::R7),
        _ => Err(AssemblyError::StructuralError {
            line,
            reason: "Invalid register identifier, must be R0-R7.".to_string(),
        }
        .into()),
    }
}

// translate a pair to an unsigned byte and check value
pub fn pair_to_unsigned_byte(pair: Pair<Rule>) -> Result<u8> {
    let line = pair.as_span().start_pos().line_col().0;
    let val_str = pair.as_str();

    let val = val_str
        .parse::<i32>()
        .map_err(|_| AssemblyError::StructuralError {
            line,
            reason: format!("Invalid immediate value: {}", val_str),
        })?;

    check_unsigned_byte(val, line)?;

    Ok(val as u8)
}

// translate a pair to a signed byte and check value
pub fn pair_to_signed_byte(pair: Pair<Rule>) -> Result<i8> {
    let line = pair.as_span().start_pos().line_col().0;
    let val_str = pair.as_str();

    let val = val_str
        .parse::<i32>()
        .map_err(|_| AssemblyError::StructuralError {
            line,
            reason: format!("Invalid immediate value: {}", val_str),
        })?;

    check_signed_byte(val, line)?;

    Ok(val as i8)
}

// translate a pair to an unsigned word and check value
pub fn pair_to_unsigned_word(pair: Pair<Rule>) -> Result<u16> {
    let line = pair.as_span().start_pos().line_col().0;
    let val_str = pair.as_str();

    let val = val_str
        .parse::<i32>()
        .map_err(|_| AssemblyError::StructuralError {
            line,
            reason: format!("Invalid immediate value: {}", val_str),
        })?;

    check_unsigned_word(val, line)?;

    Ok(val as u16)
}

// translate a pair to a signed word and check value
pub fn pair_to_signed_word(pair: Pair<Rule>) -> Result<i16> {
    let line = pair.as_span().start_pos().line_col().0;
    let val_str = pair.as_str();

    let val = val_str
        .parse::<i32>()
        .map_err(|_| AssemblyError::StructuralError {
            line,
            reason: format!("Invalid immediate value: {}", val_str),
        })?;

    check_signed_word(val, line)?;

    Ok(val as i16)
}

pub fn check_unsigned_byte(val: i32, line_num: usize) -> Result<()> {
    if val > u8::MAX as i32 || val < 0 {
        Err(AssemblyError::StructuralError {
            line: line_num,
            reason: format!(
                "Value must be an unsigned 8 bit value, (max: {}, min: 0)",
                u8::MAX
            ),
        }
        .into())
    } else {
        Ok(())
    }
}

pub fn check_signed_byte(val: i32, line_num: usize) -> Result<()> {
    if val > i8::MAX as i32 || val < i8::MIN as i32 {
        Err(AssemblyError::StructuralError {
            line: line_num,
            reason: format!(
                "Value must be a signed 8 bit value, (max: {}, min: {})",
                i8::MAX,
                i8::MIN
            ),
        }
        .into())
    } else {
        Ok(())
    }
}

pub fn check_unsigned_word(val: i32, line_num: usize) -> Result<()> {
    if val > u16::MAX as i32 || val < 0 {
        Err(AssemblyError::StructuralError {
            line: line_num,
            reason: format!(
                "Value must be an unsigned 16 bit value, (max: {}, min: 0)",
                u16::MAX
            ),
        }
        .into())
    } else {
        Ok(())
    }
}

pub fn check_signed_word(val: i32, line_num: usize) -> Result<(), AssemblyError> {
    if val > i16::MAX as i32 || val < i16::MIN as i32 {
        Err(AssemblyError::StructuralError {
            line: line_num,
            reason: format!(
                "Value must be a signed 16 bit value, (max: {}, min: {})",
                i16::MAX,
                i16::MIN
            ),
        }
        .into())
    } else {
        Ok(())
    }
}

pub fn check_bit_id(val: i32, line_num: usize) -> Result<(), AssemblyError> {
    if val > 7 || val < 0 {
        Err(AssemblyError::StructuralError {
            line: line_num,
            reason: "Value must be a bit ID value between 0 and 7 inclusive".to_string(),
        }
        .into())
    } else {
        Ok(())
    }
}
