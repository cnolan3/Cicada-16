use std::u16;

use crate::ast::*;
use crate::errors::AssemblyError;
use pest::Parser;
use pest::iterators::{Pair, Pairs};
use pest_derive::Parser;

// Derive the parser from our grammar file.
#[derive(Parser)]
#[grammar = "./grammar.pest"]
pub struct CicadaParser;

// Main parsing function that takes the entire source code string.
pub fn parse_source(source: &str) -> Result<Vec<AssemblyLine>, AssemblyError> {
    let pairs = CicadaParser::parse(Rule::program, source)?;
    let mut ast = Vec::new();

    for line_pair in pairs
        .flatten()
        .filter(|p| p.as_rule() == Rule::line_content)
    {
        let mut inner = line_pair.into_inner();
        let mut assembly_line = AssemblyLine::default();

        // Check for a label first
        if let Some(pair) = inner.peek() {
            if pair.as_rule() == Rule::label {
                assembly_line.label = Some(
                    inner
                        .next()
                        .unwrap()
                        .into_inner()
                        .next()
                        .unwrap()
                        .as_str()
                        .to_string(),
                );
            }
        }

        // Check for an instruction
        if let Some(pair) = inner.peek() {
            if pair.as_rule() == Rule::instruction {
                assembly_line.instruction = Some(build_instruction(
                    inner.next().unwrap().into_inner().next().unwrap(),
                )?);
            }
        }

        // Only add non-empty lines to our AST
        if assembly_line.label.is_some() || assembly_line.instruction.is_some() {
            ast.push(assembly_line);
        }
    }

    Ok(ast)
}

// ------------- operand builder helpers –------------

// Helper to build an Operand from a pest Pair
fn build_operand(pair: Pair<Rule>) -> Operand {
    let inner_pair = pair.into_inner().next().unwrap();
    match inner_pair.as_rule() {
        Rule::register => build_register(inner_pair),
        Rule::immediate_hex => build_immediate_hex(inner_pair),
        Rule::immediate_dec => build_immediate_dec(inner_pair),
        Rule::identifier => build_identifier(inner_pair),
        Rule::indirect => build_indirect(inner_pair),
        _ => unreachable!("Unknown operand rule: {:?}", inner_pair.as_rule()),
    }
}

// build a register object from a pair
fn build_register(pair: Pair<Rule>) -> Operand {
    let reg_char = pair.as_str().chars().nth(1).unwrap();
    let reg = match reg_char {
        '0' => Register::R0,
        '1' => Register::R1,
        '2' => Register::R2,
        '3' => Register::R3,
        '4' => Register::R4,
        '5' => Register::R5,
        '6' => Register::R6,
        '7' => Register::R7,
        _ => unreachable!("Invalid register"),
    };
    Operand::Register(reg)
}

// build an immediate object
fn build_immediate_hex(pair: Pair<Rule>) -> Operand {
    let mut hex_str: &str = "";
    match &pair.as_str().chars().nth(0) {
        Some('$') => {
            hex_str = &pair.as_str()[1..];
        }
        Some('0') => {
            hex_str = &pair.as_str()[2..];
        }
        _ => {}
    }

    let value = i32::from_str_radix(hex_str, 16).unwrap();
    Operand::Immediate(value)
}

// build an immediate object
fn build_immediate_dec(pair: Pair<Rule>) -> Operand {
    let dec_str = &pair.as_str();
    let value = dec_str.parse::<i32>().unwrap();
    Operand::Immediate(value)
}

// build an identifier object
fn build_identifier(pair: Pair<Rule>) -> Operand {
    Operand::Label(pair.as_str().to_string())
}

// build an indirect object
fn build_indirect(pair: Pair<Rule>) -> Operand {
    let reg_char = pair.as_str().chars().nth(2).unwrap();
    let reg = match reg_char {
        '0' => Register::R0,
        '1' => Register::R1,
        '2' => Register::R2,
        '3' => Register::R3,
        '4' => Register::R4,
        '5' => Register::R5,
        '6' => Register::R6,
        '7' => Register::R7,
        _ => unreachable!("Invalid register"),
    };
    Operand::Indirect(reg)
}

fn build_condition_code(pair: Pair<Rule>) -> ConditionCode {
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

// ------------- instruction builder helpers –------------

// build and check operands for a load instruction
fn build_ld_2_op(ld_pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = ld_pair.as_span().start_pos().line_col().0;

    let mut inner = ld_pair.into_inner();
    let dest = build_operand(inner.next().unwrap());
    let src = build_operand(inner.next().unwrap());

    if let Operand::Register(_) = dest {
    } else {
        return Err(AssemblyError::StructuralError {
            line,
            reason: "The destination of an LD instruction must be a register value (R0-7)."
                .to_string(),
        });
    }

    match src {
        Operand::Immediate(_) => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "The source operand of a LD instruction cannot be an immediate value."
                    .to_string(),
            });
        }
        _ => {}
    }

    Ok(Instruction::Ld(dest, src))
}

// build and check operands for a load immediate instruction
fn build_ldi_2_op(ld_pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = ld_pair.as_span().start_pos().line_col().0;

    let mut inner = ld_pair.into_inner();
    let dest = build_operand(inner.next().unwrap());
    let src = build_operand(inner.next().unwrap());

    if let Operand::Register(_) = dest {
    } else {
        return Err(AssemblyError::StructuralError {
            line,
            reason: "The destination of an LD instruction must be a register value (R0-7)."
                .to_string(),
        });
    }

    match src {
        Operand::Immediate(imm) => {
            if imm < 0 {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: "LDI immediate value must be unsigned.".to_string(),
                });
            } else if imm > (u16::MAX as i32) {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: format!("LDI immediate value must be 16 bits (max: {}).", u16::MAX),
                });
            }
        }
        Operand::Label(_) => {}
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason:
                    "The source operand of an LD instruction must be an immediate value or a label."
                        .to_string(),
            });
        }
    }

    Ok(Instruction::Ld(dest, src))
}

// build and check operands for a 2 operand add instruction
fn build_add_2_op(add_pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = add_pair.as_span().start_pos().line_col().0;

    let mut inner = add_pair.into_inner();
    let dest = build_operand(inner.next().unwrap());
    let src = build_operand(inner.next().unwrap());

    match (&dest, &src) {
        (_, Operand::Immediate(_)) => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "The source operand of an ADD instruction cannot be an immediate value."
                    .to_string(),
            });
        }
        (Operand::Label(_), _) | (_, Operand::Label(_)) => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "A label is not a valid operand to an ADD instruction.".to_string(),
            });
        }
        (Operand::Register(_), _) => { /* success */ }
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "Invalid operands to ADD instruction.".to_string(),
            });
        }
    }

    Ok(Instruction::Add(dest, Some(src)))
}

// build and check operands for a add immediate instruction
fn build_addi_2_op(add_pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = add_pair.as_span().start_pos().line_col().0;

    let mut inner = add_pair.into_inner();
    let dest = build_operand(inner.next().unwrap());
    let src = build_operand(inner.next().unwrap());

    if let Operand::Register(_) = dest {
    } else {
        return Err(AssemblyError::StructuralError {
            line,
            reason: "The destination of an ADDI instruction must be a register value (R0-7)."
                .to_string(),
        });
    }

    match src {
        Operand::Immediate(imm) => {
            if imm < 0 {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: "ADDI immediate value must be unsigned.".to_string(),
                });
            } else if imm > (u16::MAX as i32) {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: format!("ADDI immediate value must be 16 bits (max: {}).", u16::MAX),
                });
            }
        }
        Operand::Label(_) => {}
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason:
                    "The source operand of an ADDI instruction must be an immediate value or a label."
                        .to_string(),
            });
        }
    }

    Ok(Instruction::Add(dest, Some(src)))
}

// build and check operands for a 1 operand add instruction
fn build_add_1_op(add_pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = add_pair.as_span().start_pos().line_col().0;

    let mut inner = add_pair.into_inner();
    let src = build_operand(inner.next().unwrap());

    if let Operand::Register(_) = src {
    } else {
        return Err(AssemblyError::StructuralError {
            line,
            reason: "Operand to an ADD Accumulator instruction must be a register (R0-R7)."
                .to_string(),
        });
    }

    Ok(Instruction::Add(src, None))
}

// build and check operands for a 2 operand sub instruction
fn build_sub_2_op(sub_pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = sub_pair.as_span().start_pos().line_col().0;

    let mut inner = sub_pair.into_inner();
    let dest = build_operand(inner.next().unwrap());
    let src = build_operand(inner.next().unwrap());

    match (&dest, &src) {
        (_, Operand::Immediate(_)) => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "The source operand of a SUB instruction cannot be an immediate value."
                    .to_string(),
            });
        }
        (Operand::Label(_), _) | (_, Operand::Label(_)) => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "A label is not a valid operand to a SUB instruction.".to_string(),
            });
        }
        (Operand::Register(_), _) => {}
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "Invalid operands to SUB instruction.".to_string(),
            });
        }
    }

    Ok(Instruction::Sub(dest, Some(src)))
}

// build and check operands for a sub immediate instruction
fn build_subi_2_op(sub_pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = sub_pair.as_span().start_pos().line_col().0;

    let mut inner = sub_pair.into_inner();
    let dest = build_operand(inner.next().unwrap());
    let src = build_operand(inner.next().unwrap());

    if let Operand::Register(_) = dest {
    } else {
        return Err(AssemblyError::StructuralError {
            line,
            reason: "The destination of an SUBI instruction must be a register value (R0-7)."
                .to_string(),
        });
    }

    match src {
        Operand::Immediate(imm) => {
            if imm < 0 {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: "SUBI immediate value must be unsigned.".to_string(),
                });
            } else if imm > (u16::MAX as i32) {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: format!("SUBI immediate value must be 16 bits (max: {}).", u16::MAX),
                });
            }
        }
        Operand::Label(_) => {}
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason:
                    "The source operand of an SUBI instruction must be an immediate value or a label."
                        .to_string(),
            });
        }
    }

    Ok(Instruction::Sub(dest, Some(src)))
}

// build and check operands for a 1 operand sub instruction
fn build_sub_1_op(sub_pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = sub_pair.as_span().start_pos().line_col().0;

    let mut inner = sub_pair.into_inner();
    let src = build_operand(inner.next().unwrap());

    if let Operand::Register(_) = src {
    } else {
        return Err(AssemblyError::StructuralError {
            line,
            reason: "Operand to a SUB Accumulator instruction must be a register (R0-R7)."
                .to_string(),
        });
    }

    Ok(Instruction::Sub(src, None))
}

// build and check operands for a 2 operand and instruction
fn build_and_2_op(pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = pair.as_span().start_pos().line_col().0;

    let mut inner = pair.into_inner();
    let dest = build_operand(inner.next().unwrap());
    let src = build_operand(inner.next().unwrap());

    match (&dest, &src) {
        (_, Operand::Immediate(_)) => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "The source operand of an AND instruction cannot be an immediate value."
                    .to_string(),
            });
        }
        (Operand::Label(_), _) | (_, Operand::Label(_)) => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "A label is not a valid operand to an AND instruction.".to_string(),
            });
        }
        (Operand::Register(_), _) => {}
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "Invalid operands to AND instruction.".to_string(),
            });
        }
    }

    Ok(Instruction::And(dest, Some(src)))
}

// build and check operands for a and immediate instruction
fn build_andi_2_op(and_pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = and_pair.as_span().start_pos().line_col().0;

    let mut inner = and_pair.into_inner();
    let dest = build_operand(inner.next().unwrap());
    let src = build_operand(inner.next().unwrap());

    if let Operand::Register(_) = dest {
    } else {
        return Err(AssemblyError::StructuralError {
            line,
            reason: "The destination of an ANDI instruction must be a register value (R0-7)."
                .to_string(),
        });
    }

    match src {
        Operand::Immediate(imm) => {
            if imm < 0 {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: "ANDI immediate value must be unsigned.".to_string(),
                });
            } else if imm > (u16::MAX as i32) {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: format!("ANDI immediate value must be 16 bits (max: {}).", u16::MAX),
                });
            }
        }
        Operand::Label(_) => {}
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason:
                    "The source operand of an ANDI instruction must be an immediate value or a label."
                        .to_string(),
            });
        }
    }

    Ok(Instruction::And(dest, Some(src)))
}

// build and check operands for a 1 operand and instruction
fn build_and_1_op(pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = pair.as_span().start_pos().line_col().0;

    let mut inner = pair.into_inner();
    let src = build_operand(inner.next().unwrap());

    if let Operand::Register(_) = src {
    } else {
        return Err(AssemblyError::StructuralError {
            line,
            reason: "Operand to an AND Accumulator instruction must be a register (R0-R7)."
                .to_string(),
        });
    }

    Ok(Instruction::And(src, None))
}

// build and check operands for a 2 operand or instruction
fn build_or_2_op(pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = pair.as_span().start_pos().line_col().0;

    let mut inner = pair.into_inner();
    let dest = build_operand(inner.next().unwrap());
    let src = build_operand(inner.next().unwrap());

    match (&dest, &src) {
        (_, Operand::Immediate(_)) => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "The source operand of an OR instruction cannot be an immediate value."
                    .to_string(),
            });
        }
        (Operand::Label(_), _) | (_, Operand::Label(_)) => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "A label is not a valid operand to an OR instruction.".to_string(),
            });
        }
        (Operand::Register(_), _) => {}
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "Invalid operands to OR instruction.".to_string(),
            });
        }
    }

    Ok(Instruction::Or(dest, Some(src)))
}

// build and check operands for a or immediate instruction
fn build_ori_2_op(or_pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = or_pair.as_span().start_pos().line_col().0;

    let mut inner = or_pair.into_inner();
    let dest = build_operand(inner.next().unwrap());
    let src = build_operand(inner.next().unwrap());

    if let Operand::Register(_) = dest {
    } else {
        return Err(AssemblyError::StructuralError {
            line,
            reason: "The destination of an ORI instruction must be a register value (R0-7)."
                .to_string(),
        });
    }

    match src {
        Operand::Immediate(imm) => {
            if imm < 0 {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: "ORI immediate value must be unsigned.".to_string(),
                });
            } else if imm > (u16::MAX as i32) {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: format!("ORI immediate value must be 16 bits (max: {}).", u16::MAX),
                });
            }
        }
        Operand::Label(_) => {}
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason:
                    "The source operand of an ORI instruction must be an immediate value or a label."
                        .to_string(),
            });
        }
    }

    Ok(Instruction::Or(dest, Some(src)))
}

// build and check operands for a 1 operand or instruction
fn build_or_1_op(pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = pair.as_span().start_pos().line_col().0;

    let mut inner = pair.into_inner();
    let src = build_operand(inner.next().unwrap());

    if let Operand::Register(_) = src {
    } else {
        return Err(AssemblyError::StructuralError {
            line,
            reason: "Operand to an OR Accumulator instruction must be a register (R0-R7)."
                .to_string(),
        });
    }

    Ok(Instruction::Or(src, None))
}

// build and check operands for a 2 operand xor instruction
fn build_xor_2_op(pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = pair.as_span().start_pos().line_col().0;

    let mut inner = pair.into_inner();
    let dest = build_operand(inner.next().unwrap());
    let src = build_operand(inner.next().unwrap());

    match (&dest, &src) {
        (_, Operand::Immediate(_)) => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "The source operand of an XOR instruction cannot be an immediate value."
                    .to_string(),
            });
        }
        (Operand::Label(_), _) | (_, Operand::Label(_)) => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "A label is not a valid operand to a XOR instruction.".to_string(),
            });
        }
        (Operand::Register(_), _) => {}
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "Invalid operands to XOR instruction.".to_string(),
            });
        }
    }

    Ok(Instruction::Xor(dest, Some(src)))
}

// build and check operands for a xor immediate instruction
fn build_xori_2_op(xor_pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = xor_pair.as_span().start_pos().line_col().0;

    let mut inner = xor_pair.into_inner();
    let dest = build_operand(inner.next().unwrap());
    let src = build_operand(inner.next().unwrap());

    if let Operand::Register(_) = dest {
    } else {
        return Err(AssemblyError::StructuralError {
            line,
            reason: "The destination of an XORI instruction must be a register value (R0-7)."
                .to_string(),
        });
    }

    match src {
        Operand::Immediate(imm) => {
            if imm < 0 {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: "XORI immediate value must be unsigned.".to_string(),
                });
            } else if imm > (u16::MAX as i32) {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: format!("XORI immediate value must be 16 bits (max: {}).", u16::MAX),
                });
            }
        }
        Operand::Label(_) => {}
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason:
                    "The source operand of an XORI instruction must be an immediate value or a label."
                        .to_string(),
            });
        }
    }

    Ok(Instruction::Xor(dest, Some(src)))
}

// build and check operands for a 1 operand xor instruction
fn build_xor_1_op(pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = pair.as_span().start_pos().line_col().0;

    let mut inner = pair.into_inner();
    let src = build_operand(inner.next().unwrap());

    if let Operand::Register(_) = src {
    } else {
        return Err(AssemblyError::StructuralError {
            line,
            reason: "Operand to a XOR Accumulator instruction must be a register (R0-R7)."
                .to_string(),
        });
    }

    Ok(Instruction::Xor(src, None))
}

// build and check operands for a 2 operand cmp instruction
fn build_cmp_2_op(pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = pair.as_span().start_pos().line_col().0;

    let mut inner = pair.into_inner();
    let dest = build_operand(inner.next().unwrap());
    let src = build_operand(inner.next().unwrap());

    match (&dest, &src) {
        (_, Operand::Immediate(_)) => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "The source operand of a CMP instruction cannot be an immediate value."
                    .to_string(),
            });
        }
        (Operand::Label(_), _) | (_, Operand::Label(_)) => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "A label is not a valid operand to a CMP instruction.".to_string(),
            });
        }
        (Operand::Register(_), _) => {}
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "Invalid operands to CMP instruction.".to_string(),
            });
        }
    }

    Ok(Instruction::Cmp(dest, Some(src)))
}

// build and check operands for a cmp immediate instruction
fn build_cmpi_2_op(cmp_pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = cmp_pair.as_span().start_pos().line_col().0;

    let mut inner = cmp_pair.into_inner();
    let dest = build_operand(inner.next().unwrap());
    let src = build_operand(inner.next().unwrap());

    if let Operand::Register(_) = dest {
    } else {
        return Err(AssemblyError::StructuralError {
            line,
            reason: "The destination of an CMPI instruction must be a register value (R0-7)."
                .to_string(),
        });
    }

    match src {
        Operand::Immediate(imm) => {
            if imm < 0 {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: "CMPI immediate value must be unsigned.".to_string(),
                });
            } else if imm > (u16::MAX as i32) {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: format!("CMPI immediate value must be 16 bits (max: {}).", u16::MAX),
                });
            }
        }
        Operand::Label(_) => {}
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason:
                    "The source operand of an CMPI instruction must be an immediate value or a label."
                        .to_string(),
            });
        }
    }

    Ok(Instruction::Cmp(dest, Some(src)))
}

// build and check operands for a 1 operand cmp instruction
fn build_cmp_1_op(pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = pair.as_span().start_pos().line_col().0;

    let mut inner = pair.into_inner();
    let src = build_operand(inner.next().unwrap());

    if let Operand::Register(_) = src {
    } else {
        return Err(AssemblyError::StructuralError {
            line,
            reason: "Operand to a CMP Accumulator instruction must be a register (R0-R7)."
                .to_string(),
        });
    }

    Ok(Instruction::Cmp(src, None))
}

// build and check operands for a 2 operand adc instruction
fn build_adc_2_op(pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = pair.as_span().start_pos().line_col().0;

    let mut inner = pair.into_inner();
    let dest = build_operand(inner.next().unwrap());
    let src = build_operand(inner.next().unwrap());

    match (&dest, &src) {
        (_, Operand::Immediate(_)) => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "The source operand of an ADC instruction cannot be an immediate value."
                    .to_string(),
            });
        }
        (Operand::Label(_), _) | (_, Operand::Label(_)) => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "A label is not a valid operand to an ADC instruction.".to_string(),
            });
        }
        (Operand::Register(_), _) => {}
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "Invalid operands to ADC instruction.".to_string(),
            });
        }
    }

    Ok(Instruction::Adc(dest, Some(src)))
}

// build and check operands for a 2 operand sbc instruction
fn build_sbc_2_op(pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = pair.as_span().start_pos().line_col().0;

    let mut inner = pair.into_inner();
    let dest = build_operand(inner.next().unwrap());
    let src = build_operand(inner.next().unwrap());

    match (&dest, &src) {
        (_, Operand::Immediate(_)) => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "The source operand of an SBC instruction cannot be an immediate value."
                    .to_string(),
            });
        }
        (Operand::Label(_), _) | (_, Operand::Label(_)) => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "A label is not a valid operand to an SBC instruction.".to_string(),
            });
        }
        (Operand::Register(_), _) => {}
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "Invalid operands to SBC instruction.".to_string(),
            });
        }
    }

    Ok(Instruction::Sbc(dest, Some(src)))
}

// build and check operands for a jump instruction
fn build_jmp(jmp_pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = jmp_pair.as_span().start_pos().line_col().0;

    let mut inner = jmp_pair.into_inner();
    let op = build_operand(inner.next().unwrap());

    match op {
        Operand::Indirect(_) | Operand::Label(_) => {} // Corrected: Removed unnecessary escape for underscore
        Operand::Immediate(imm) => {
            if imm > (u16::MAX as i32) || imm < 0 {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: format!(
                        "JMP immediate address must be an unsigned 16 bit value (max: {}, min: 0).",
                        u16::MAX,
                    ),
                });
            }
        }
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "Operand to a JMP instruction must be an indirect address ((R0-R7)), label or immediate address."
                    .to_string(),
            });
        }
    }

    Ok(Instruction::Jmp(op))
}

// build and check operands for a jump relative instruction
fn build_jr(jmp_pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = jmp_pair.as_span().start_pos().line_col().0;

    let mut inner = jmp_pair.into_inner();
    let op = build_operand(inner.next().unwrap());

    match op {
        Operand::Immediate(imm) => {
            if imm > (i8::MAX as i32) || imm < (i8::MIN as i32) {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: format!(
                        "JR immediate relative value must be 8 bits (max: {}, min: {}).",
                        i8::MAX,
                        i8::MIN
                    ),
                });
            }
        }
        Operand::Label(_) => {}
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason:
                    "Operand to a JR instruction must be an immediate relative value or a label."
                        .to_string(),
            });
        }
    }

    Ok(Instruction::Jr(op))
}

// build and check operands for a conditional jump instruction
fn build_jcc(jmp_pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = jmp_pair.as_span().start_pos().line_col().0;

    let mut inner = jmp_pair.into_inner();
    let cc = build_condition_code(inner.next().unwrap());
    let op = build_operand(inner.next().unwrap());

    match op {
        Operand::Label(_) => {} // Corrected: Removed unnecessary escape for underscore
        Operand::Immediate(imm) => {
            if imm > (u16::MAX as i32) || imm < 0 {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: format!(
                        "Jcc immediate address must be an unsigned 16 bit value (max: {}, min: 0).",
                        u16::MAX,
                    ),
                });
            }
        }
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "Operand to a Jcc instruction must be a label or immediate address."
                    .to_string(),
            });
        }
    }

    Ok(Instruction::Jcc(cc, op))
}

// build and check operands for a conditional jump relative instruction
fn build_jrcc(jmp_pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = jmp_pair.as_span().start_pos().line_col().0;

    let mut inner = jmp_pair.into_inner();
    let cc = build_condition_code(inner.next().unwrap());
    let op = build_operand(inner.next().unwrap());

    match op {
        Operand::Immediate(imm) => {
            if imm > (i8::MAX as i32) || imm < (i8::MIN as i32) {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: format!(
                        "JRcc immediate relative value must be 8 bits (max: {}, min: {}).",
                        i8::MAX,
                        i8::MIN
                    ),
                });
            }
        }
        Operand::Label(_) => {}
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason:
                    "Operand to a JRcc instruction must be an immediate relative value or a label."
                        .to_string(),
            });
        }
    }

    Ok(Instruction::Jrcc(cc, op))
}

// build and check operands for a DJNZ instruction
fn build_djnz(jmp_pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = jmp_pair.as_span().start_pos().line_col().0;

    let mut inner = jmp_pair.into_inner();
    let op = build_operand(inner.next().unwrap());

    match op {
        Operand::Immediate(imm) => {
            if imm > (i8::MAX as i32) || imm < (i8::MIN as i32) {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: format!(
                        "DJNZ immediate relative value must be 8 bits (max: {}, min: {}).",
                        i8::MAX,
                        i8::MIN
                    ),
                });
            }
        }
        Operand::Label(_) => {}
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason:
                    "Operand to a DJNZ instruction must be an immediate relative value or a label."
                        .to_string(),
            });
        }
    }

    Ok(Instruction::djnz(op))
}

// ------------- build instruction –------------

// Helper to build an Instruction from a pest Pair
fn build_instruction(pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    match pair.as_rule() {
        Rule::nop => Ok(Instruction::Nop),
        Rule::halt => Ok(Instruction::Halt),
        Rule::ld_2_op => build_ld_2_op(pair),
        Rule::ldi_2_op => build_ldi_2_op(pair),
        Rule::add_2_op => build_add_2_op(pair),
        Rule::addi_2_op => build_addi_2_op(pair),
        Rule::add_1_op => build_add_1_op(pair),
        Rule::sub_2_op => build_sub_2_op(pair),
        Rule::subi_2_op => build_subi_2_op(pair),
        Rule::sub_1_op => build_sub_1_op(pair),
        Rule::and_2_op => build_and_2_op(pair),
        Rule::andi_2_op => build_andi_2_op(pair),
        Rule::and_1_op => build_and_1_op(pair),
        Rule::or_2_op => build_or_2_op(pair),
        Rule::ori_2_op => build_ori_2_op(pair),
        Rule::or_1_op => build_or_1_op(pair),
        Rule::xor_2_op => build_xor_2_op(pair),
        Rule::xori_2_op => build_xori_2_op(pair),
        Rule::xor_1_op => build_xor_1_op(pair),
        Rule::cmp_2_op => build_cmp_2_op(pair),
        Rule::cmpi_2_op => build_cmpi_2_op(pair),
        Rule::cmp_1_op => build_cmp_1_op(pair),
        Rule::adc_2_op => build_adc_2_op(pair),
        Rule::sbc_2_op => build_sbc_2_op(pair),
        Rule::neg => Ok(Instruction::Neg),
        Rule::not => Ok(Instruction::Not),
        Rule::swap => Ok(Instruction::Swap),
        Rule::jmp => build_jmp(pair),
        Rule::jr => build_jr(pair),
        Rule::jmp_con => build_jcc(pair),
        Rule::jr_con => build_jrcc(pair),
        Rule::djnz => build_djnz(pair),
        Rule::ccf => Ok(Instruction::Ccf),
        Rule::scf => Ok(Instruction::Scf),
        Rule::rcf => Ok(Instruction::Rcf),
        // ... add cases for all other instructions
        _ => unreachable!("Unknown instruction rule: {:?}", pair.as_rule()),
    }
}

// ------------- unit tests –------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_nop() {
        let source = "nop\n";
        let result = parse_source(source);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].instruction, Some(Instruction::Nop));
        assert_eq!(lines[0].label, None);
    }

    #[test]
    fn test_parse_sub_reg() {
        let source = "sub r1\n";
        let result = parse_source(source);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0].instruction,
            Some(Instruction::Sub(Operand::Register(Register::R1), None))
        );
        assert_eq!(lines[0].label, None);
    }

    #[test]
    fn test_parse_and_reg_reg() {
        let source = "and r2, r3\n";
        let result = parse_source(source);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0].instruction,
            Some(Instruction::And(
                Operand::Register(Register::R2),
                Some(Operand::Register(Register::R3))
            ))
        );
        assert_eq!(lines[0].label, None);
    }

    #[test]
    fn test_parse_or_reg_reg() {
        let source = "or r4, r5\n";
        let result = parse_source(source);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0].instruction,
            Some(Instruction::Or(
                Operand::Register(Register::R4),
                Some(Operand::Register(Register::R5))
            ))
        );
        assert_eq!(lines[0].label, None);
    }

    #[test]
    fn test_parse_xor_reg_reg() {
        let source = "xor r6, r7\n";
        let result = parse_source(source);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0].instruction,
            Some(Instruction::Xor(
                Operand::Register(Register::R6),
                Some(Operand::Register(Register::R7))
            ))
        );
        assert_eq!(lines[0].label, None);
    }

    #[test]
    fn test_parse_cmp_reg_reg() {
        let source = "cmp r0, r1\n";
        let result = parse_source(source);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0].instruction,
            Some(Instruction::Cmp(
                Operand::Register(Register::R0),
                Some(Operand::Register(Register::R1))
            ))
        );
        assert_eq!(lines[0].label, None);
    }

    #[test]
    fn test_parse_adc_reg_reg() {
        let source = "adc r2, r3\n";
        let result = parse_source(source);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0].instruction,
            Some(Instruction::Adc(
                Operand::Register(Register::R2),
                Some(Operand::Register(Register::R3))
            ))
        );
        assert_eq!(lines[0].label, None);
    }

    #[test]
    fn test_parse_sbc_reg_reg() {
        let source = "sbc r4, r5\n";
        let result = parse_source(source);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0].instruction,
            Some(Instruction::Sbc(
                Operand::Register(Register::R4),
                Some(Operand::Register(Register::R5))
            ))
        );
        assert_eq!(lines[0].label, None);
    }

    #[test]
    fn test_parse_and_reg() {
        let source = "and r1\n";
        let result = parse_source(source);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0].instruction,
            Some(Instruction::And(Operand::Register(Register::R1), None))
        );
        assert_eq!(lines[0].label, None);
    }

    #[test]
    fn test_parse_or_reg() {
        let source = "or r2\n";
        let result = parse_source(source);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0].instruction,
            Some(Instruction::Or(Operand::Register(Register::R2), None))
        );
        assert_eq!(lines[0].label, None);
    }

    #[test]
    fn test_parse_xor_reg() {
        let source = "xor r3\n";
        let result = parse_source(source);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0].instruction,
            Some(Instruction::Xor(Operand::Register(Register::R3), None))
        );
        assert_eq!(lines[0].label, None);
    }

    #[test]
    fn test_parse_cmp_reg() {
        let source = "cmp r4\n";
        let result = parse_source(source);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0].instruction,
            Some(Instruction::Cmp(Operand::Register(Register::R4), None))
        );
        assert_eq!(lines[0].label, None);
    }

    #[test]
    fn test_parse_neg() {
        let source = "neg\n";
        let result = parse_source(source);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].instruction, Some(Instruction::Neg));
        assert_eq!(lines[0].label, None);
    }

    #[test]
    fn test_parse_not() {
        let source = "not\n";
        let result = parse_source(source);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].instruction, Some(Instruction::Not));
        assert_eq!(lines[0].label, None);
    }

    #[test]
    fn test_parse_swap() {
        let source = "swap\n";
        let result = parse_source(source);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].instruction, Some(Instruction::Swap));
        assert_eq!(lines[0].label, None);
    }

    #[test]
    fn test_parse_ccf() {
        let source = "ccf\n";
        let result = parse_source(source);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].instruction, Some(Instruction::Ccf));
        assert_eq!(lines[0].label, None);
    }

    #[test]
    fn test_parse_scf() {
        let source = "scf\n";
        let result = parse_source(source);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].instruction, Some(Instruction::Scf));
        assert_eq!(lines[0].label, None);
    }

    #[test]
    fn test_parse_rcf() {
        let source = "rcf\n";
        let result = parse_source(source);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].instruction, Some(Instruction::Rcf));
        assert_eq!(lines[0].label, None);
    }
}
