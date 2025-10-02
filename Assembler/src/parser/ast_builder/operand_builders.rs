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

use crate::errors::AssemblyError;
use crate::parser::ast_builder::utility_functions::*;
use crate::parser::{ConditionCode, Operand};
use crate::parser::{Pair, Rule};
use anyhow::{Context, Result};

// Helper to build an Operand from a pest Pair
pub fn build_operand(pair: Pair<Rule>) -> Result<Operand> {
    let line = pair.as_span().start_pos().line_col().0;
    let inner_pair = pair
        .into_inner()
        .next()
        .ok_or_else(|| AssemblyError::StructuralError {
            line,
            reason: "Expected an inner operand rule.".to_string(),
        })?;
    match inner_pair.as_rule() {
        Rule::register => build_register(inner_pair),
        Rule::immediate_hex => build_immediate_hex(inner_pair),
        Rule::immediate_dec => build_immediate_dec(inner_pair),
        Rule::identifier => build_identifier(inner_pair),
        Rule::indirect => build_indirect(inner_pair),
        Rule::absolute => build_absolute(inner_pair),
        Rule::predec => build_pre_decrement(inner_pair),
        Rule::postinc => build_post_increment(inner_pair),
        Rule::indexed => build_indexed(inner_pair),
        Rule::str_literal => build_string_literal(inner_pair),
        _ => unreachable!("Unknown operand rule: {:?}", inner_pair.as_rule()),
    }
}

// build a register object from a pair
pub fn build_register(pair: Pair<Rule>) -> Result<Operand> {
    let line = pair.as_span().start_pos().line_col().0;
    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| AssemblyError::StructuralError {
            line,
            reason: "Expected a register number.".to_string(),
        })?;
    let reg = pair_to_reg(inner)?;
    Ok(Operand::Register(reg))
}

// build an immediate object
pub fn build_immediate_hex(pair: Pair<Rule>) -> Result<Operand> {
    let line = pair.as_span().start_pos().line_col().0;
    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| AssemblyError::StructuralError {
            line,
            reason: "Expected a hex value body.".to_string(),
        })?;
    let value =
        i32::from_str_radix(inner.as_str(), 16).map_err(|_| AssemblyError::StructuralError {
            line,
            reason: format!("Invalid hex value: {}", inner.as_str()),
        })?;
    Ok(Operand::Immediate(value))
}

// build an immediate object
pub fn build_immediate_dec(pair: Pair<Rule>) -> Result<Operand> {
    let line = pair.as_span().start_pos().line_col().0;
    let dec_str = pair.as_str();
    let value = dec_str
        .parse::<i32>()
        .map_err(|_| AssemblyError::StructuralError {
            line,
            reason: format!("Invalid decimal value: {}", dec_str),
        })?;
    Ok(Operand::Immediate(value))
}

// build an identifier object
pub fn build_identifier(pair: Pair<Rule>) -> Result<Operand> {
    Ok(Operand::Label(pair.as_str().to_string()))
}

// build an indirect object
pub fn build_indirect(pair: Pair<Rule>) -> Result<Operand> {
    let line = pair.as_span().start_pos().line_col().0;
    let reg_pair = pair
        .into_inner()
        .next()
        .ok_or_else(|| AssemblyError::StructuralError {
            line,
            reason: "Expected a register for indirect addressing.".to_string(),
        })?
        .into_inner()
        .next()
        .ok_or_else(|| AssemblyError::StructuralError {
            line,
            reason: "Expected a register for indirect addressing.".to_string(),
        })?;
    let reg = pair_to_reg(reg_pair)?;
    Ok(Operand::Indirect(reg))
}

pub fn build_absolute(pair: Pair<Rule>) -> Result<Operand> {
    let line = pair.as_span().start_pos().line_col().0;
    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| AssemblyError::StructuralError {
            line,
            reason: "Expected an address or label for absolute addressing.".to_string(),
        })?;

    match inner.as_rule() {
        Rule::immediate_hex => {
            let hex = inner
                .into_inner()
                .next()
                .ok_or_else(|| AssemblyError::StructuralError {
                    line,
                    reason: "Expected a hex value body.".to_string(),
                })?;
            let value = u16::from_str_radix(hex.as_str(), 16).map_err(|_| {
                AssemblyError::StructuralError {
                    line,
                    reason: format!("Invalid hex value: {}", hex.as_str()),
                }
            })?;
            Ok(Operand::AbsAddr(value))
        }
        Rule::identifier => Ok(Operand::AbsLabel(inner.as_str().to_string())),
        _ => Err(AssemblyError::StructuralError {
            line,
            reason: "Expected a hex value or label for absolute addressing.".to_string(),
        }
        .into()),
    }
}

pub fn build_pre_decrement(pair: Pair<Rule>) -> Result<Operand> {
    let line = pair.as_span().start_pos().line_col().0;
    let reg_pair = pair
        .into_inner()
        .next()
        .ok_or_else(|| AssemblyError::StructuralError {
            line,
            reason: "Expected a register for pre-decrement addressing.".to_string(),
        })?
        .into_inner()
        .next()
        .ok_or_else(|| AssemblyError::StructuralError {
            line,
            reason: "Expected a register for pre-decrement addressing.".to_string(),
        })?;
    let reg = pair_to_reg(reg_pair)?;
    Ok(Operand::PreDecrement(reg))
}

pub fn build_post_increment(pair: Pair<Rule>) -> Result<Operand> {
    let line = pair.as_span().start_pos().line_col().0;
    let reg_pair = pair
        .into_inner()
        .next()
        .ok_or_else(|| AssemblyError::StructuralError {
            line,
            reason: "Expected a register for post-increment addressing.".to_string(),
        })?
        .into_inner()
        .next()
        .ok_or_else(|| AssemblyError::StructuralError {
            line,
            reason: "Expected a register for post-increment addressing.".to_string(),
        })?;
    let reg = pair_to_reg(reg_pair)?;
    Ok(Operand::PostIncrement(reg))
}

pub fn build_indexed(pair: Pair<Rule>) -> Result<Operand> {
    let line = pair.as_span().start_pos().line_col().0;
    let mut inner = pair.into_inner();
    let reg_pair = inner
        .next()
        .ok_or_else(|| AssemblyError::StructuralError {
            line,
            reason: "Expected a register for indexed addressing.".to_string(),
        })?
        .into_inner()
        .next()
        .ok_or_else(|| AssemblyError::StructuralError {
            line,
            reason: "Expected a register for indexed addressing.".to_string(),
        })?;
    let op_pair = inner.next().ok_or_else(|| AssemblyError::StructuralError {
        line,
        reason: "Expected an offset for indexed addressing.".to_string(),
    })?;

    let reg = pair_to_reg(reg_pair)?;

    match op_pair.as_rule() {
        Rule::immediate_hex | Rule::immediate_dec => {
            let imm =
                pair_to_signed_byte(op_pair).context("Invalid offset for indexed operand.")?;
            Ok(Operand::Indexed(reg, imm))
        }
        Rule::label => Ok(Operand::IndexedLabel(reg, op_pair.as_str().to_string())),
        _ => Err(AssemblyError::StructuralError {
            line,
            reason: "Expected an immediate value or label for offset.".to_string(),
        }
        .into()),
    }
}

pub fn build_string_literal(pair: Pair<Rule>) -> Result<Operand> {
    let line = pair.as_span().start_pos().line_col().0;
    let mut inner = pair.into_inner();
    let op_pair = inner.next().ok_or_else(|| AssemblyError::StructuralError {
        line,
        reason: "Expected a string literal.".to_string(),
    })?;

    match op_pair.as_rule() {
        Rule::str_val => Ok(Operand::String(op_pair.as_str().to_string())),
        _ => Err(AssemblyError::StructuralError {
            line,
            reason: "Expected a string.".to_string(),
        }
        .into()),
    }
}

pub fn build_condition_code(pair: Pair<Rule>) -> Result<ConditionCode> {
    let line = pair.as_span().start_pos().line_col().0;
    let cc = pair
        .into_inner()
        .next()
        .ok_or_else(|| AssemblyError::StructuralError {
            line,
            reason: "Expected inner condition code rule.".to_string(),
        })?
        .into_inner()
        .next()
        .ok_or_else(|| AssemblyError::StructuralError {
            line,
            reason: "Expected specific condition code rule.".to_string(),
        })?;

    match cc.as_rule() {
        Rule::zero => Ok(ConditionCode::Z),
        Rule::not_zero => Ok(ConditionCode::Nz),
        Rule::carry => Ok(ConditionCode::C),
        Rule::not_carry => Ok(ConditionCode::Nc),
        Rule::negative => Ok(ConditionCode::N),
        Rule::not_negative => Ok(ConditionCode::Nn),
        Rule::overflow => Ok(ConditionCode::V),
        Rule::not_overflow => Ok(ConditionCode::Nv),
        _ => unreachable!("Invalid condition code"),
    }
}
