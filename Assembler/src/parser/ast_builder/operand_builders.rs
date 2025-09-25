use crate::errors::AssemblyError;
use crate::parser::ast_builder::utility_functions::*;
use crate::parser::{ConditionCode, Operand};
use crate::parser::{Pair, Rule};
use anyhow::{Context, Result};

// Helper to build an Operand from a pest Pair
pub fn build_operand(pair: Pair<Rule>) -> Result<Operand> {
    let inner_pair = pair.into_inner().next().unwrap();
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
        _ => unreachable!("Unknown operand rule: {:?}", inner_pair.as_rule()),
    }
}

// build a register object from a pair
pub fn build_register(pair: Pair<Rule>) -> Result<Operand> {
    let inner = pair.into_inner().next().unwrap();
    let reg = pair_to_reg(inner)?;
    Ok(Operand::Register(reg))
}

// build an immediate object
pub fn build_immediate_hex(pair: Pair<Rule>) -> Result<Operand> {
    let inner = pair.into_inner().next().unwrap();
    let value = i32::from_str_radix(inner.as_str(), 16).unwrap();
    Ok(Operand::Immediate(value))
}

// build an immediate object
pub fn build_immediate_dec(pair: Pair<Rule>) -> Result<Operand> {
    let dec_str = pair.as_str();
    let value = dec_str.parse::<i32>().unwrap();
    Ok(Operand::Immediate(value))
}

// build an identifier object
pub fn build_identifier(pair: Pair<Rule>) -> Result<Operand> {
    Ok(Operand::Label(pair.as_str().to_string()))
}

// build an indirect object
pub fn build_indirect(pair: Pair<Rule>) -> Result<Operand> {
    let reg_pair = pair
        .into_inner()
        .next()
        .unwrap()
        .into_inner()
        .next()
        .unwrap();
    let reg = pair_to_reg(reg_pair)?;
    Ok(Operand::Indirect(reg))
}

pub fn build_absolute(pair: Pair<Rule>) -> Result<Operand> {
    let line = pair.as_span().start_pos().line_col().0;
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::immediate_hex => {
            let hex = inner.into_inner().next().unwrap();
            let value = u16::from_str_radix(hex.as_str(), 16).unwrap();
            Ok(Operand::AbsAddr(value))
        }
        Rule::identifier => Ok(Operand::AbsLabel(inner.as_str().to_string())),
        _ => Err(AssemblyError::StructuralError {
            line,
            reason: "Invalid absolute operand, must be a 16 bit immediate or a label.".to_string(),
        }
        .into()),
    }
}

pub fn build_pre_decrement(pair: Pair<Rule>) -> Result<Operand> {
    let reg_pair = pair
        .into_inner()
        .next()
        .unwrap()
        .into_inner()
        .next()
        .unwrap();
    let reg = pair_to_reg(reg_pair)?;
    Ok(Operand::PreDecrement(reg))
}

pub fn build_post_increment(pair: Pair<Rule>) -> Result<Operand> {
    let reg_pair = pair
        .into_inner()
        .next()
        .unwrap()
        .into_inner()
        .next()
        .unwrap();
    let reg = pair_to_reg(reg_pair)?;
    Ok(Operand::PostIncrement(reg))
}

pub fn build_indexed(pair: Pair<Rule>) -> Result<Operand> {
    let mut inner = pair.into_inner();
    let reg_pair = inner.next().unwrap().into_inner().next().unwrap();
    let imm_pair = inner.next().unwrap();

    let reg = pair_to_reg(reg_pair)?;

    let imm = pair_to_signed_byte(imm_pair).context("Invalid offset for indexed operand.")?;

    Ok(Operand::Indexed(reg, imm))
}

pub fn build_condition_code(pair: Pair<Rule>) -> ConditionCode {
    let cc = pair
        .into_inner()
        .next()
        .unwrap()
        .into_inner()
        .next()
        .unwrap();

    match cc.as_rule() {
        Rule::zero => ConditionCode::Z,
        Rule::not_zero => ConditionCode::Nz,
        Rule::carry => ConditionCode::C,
        Rule::not_carry => ConditionCode::Nc,
        Rule::negative => ConditionCode::N,
        Rule::not_negative => ConditionCode::Nn,
        Rule::overflow => ConditionCode::V,
        Rule::not_overflow => ConditionCode::Nv,
        _ => unreachable!("Invalid condition code"),
    }
}
