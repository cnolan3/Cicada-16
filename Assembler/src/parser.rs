use std::u16;

use crate::ast::*;
use crate::errors::AssemblyError;
use pest::Parser;
use pest::iterators::Pair;
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

        // Check for a directive
        if let Some(pair) = inner.peek() {
            if pair.as_rule() == Rule::directive {
                assembly_line.directive = Some(build_directive(
                    inner.next().unwrap().into_inner().next().unwrap(),
                )?);
            }
        }

        // Only add non-empty lines to our AST
        if assembly_line.label.is_some()
            || assembly_line.instruction.is_some()
            || assembly_line.directive.is_some()
        {
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
        Rule::absolute => build_absolute(inner_pair),
        Rule::predec => build_pre_decrement(inner_pair),
        Rule::postinc => build_post_increment(inner_pair),
        Rule::indexed => build_indexed(inner_pair),
        _ => unreachable!("Unknown operand rule: {:?}", inner_pair.as_rule()),
    }
}

// build a register object from a pair
fn build_register(pair: Pair<Rule>) -> Operand {
    let inner = pair.into_inner().next().unwrap();
    let reg = pair_to_reg(inner);
    Operand::Register(reg)
}

// build an immediate object
fn build_immediate_hex(pair: Pair<Rule>) -> Operand {
    let inner = pair.into_inner().next().unwrap();
    let value = i32::from_str_radix(inner.as_str(), 16).unwrap();
    Operand::Immediate(value)
}

// build an immediate object
fn build_immediate_dec(pair: Pair<Rule>) -> Operand {
    let dec_str = pair.as_str();
    let value = dec_str.parse::<i32>().unwrap();
    Operand::Immediate(value)
}

// build an identifier object
fn build_identifier(pair: Pair<Rule>) -> Operand {
    Operand::Label(pair.as_str().to_string())
}

// build an indirect object
fn build_indirect(pair: Pair<Rule>) -> Operand {
    let reg_pair = pair
        .into_inner()
        .next()
        .unwrap()
        .into_inner()
        .next()
        .unwrap();
    let reg = pair_to_reg(reg_pair);
    Operand::Indirect(reg)
}

fn build_absolute(pair: Pair<Rule>) -> Operand {
    let hex = pair
        .into_inner()
        .next()
        .unwrap()
        .into_inner()
        .next()
        .unwrap();
    let value = u16::from_str_radix(hex.as_str(), 16).unwrap();
    Operand::Absolute(value)
}

fn build_pre_decrement(pair: Pair<Rule>) -> Operand {
    let reg_pair = pair
        .into_inner()
        .next()
        .unwrap()
        .into_inner()
        .next()
        .unwrap();
    let reg = pair_to_reg(reg_pair);
    Operand::PreDecrement(reg)
}

fn build_post_increment(pair: Pair<Rule>) -> Operand {
    let reg_pair = pair
        .into_inner()
        .next()
        .unwrap()
        .into_inner()
        .next()
        .unwrap();
    let reg = pair_to_reg(reg_pair);
    Operand::PostIncrement(reg)
}

fn build_indexed(pair: Pair<Rule>) -> Operand {
    let mut inner = pair.into_inner();
    let reg_pair = inner.next().unwrap().into_inner().next().unwrap();
    let imm_str = inner.next().unwrap().as_str();

    let reg = pair_to_reg(reg_pair);

    let imm = imm_str.parse::<i32>().unwrap();

    Operand::Indexed(reg, imm as i8)
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

fn pair_to_reg(reg_pair: Pair<Rule>) -> Register {
    let reg_char = reg_pair.as_str().chars().nth(0).unwrap();
    match reg_char {
        '0' => Register::R0,
        '1' => Register::R1,
        '2' => Register::R2,
        '3' => Register::R3,
        '4' => Register::R4,
        '5' => Register::R5,
        '6' => Register::R6,
        '7' => Register::R7,
        _ => unreachable!("Invalid register"),
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
        Operand::Indexed(_, offset) => {
            if offset > i8::MAX || offset < i8::MIN {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: format!(
                        "The offset value given to a LD offset instruction must be a signed 8 bit value (max: {}, min: {})",
                        i8::MAX,
                        i8::MIN
                    ),
                });
            }
        }
        _ => {}
    }

    Ok(Instruction::Ld(dest, src))
}

// build and check operands for a store instruction
fn build_st_2_op(st_pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = st_pair.as_span().start_pos().line_col().0;

    let mut inner = st_pair.into_inner();
    let dest = build_operand(inner.next().unwrap());
    let src = build_operand(inner.next().unwrap());

    if let Operand::Register(_) = src {
    } else {
        return Err(AssemblyError::StructuralError {
            line,
            reason: "The source of an ST instruction must be a register value (R0-7).".to_string(),
        });
    }

    match dest {
        Operand::Absolute(_)
        | Operand::Indirect(_)
        | Operand::PostIncrement(_)
        | Operand::PreDecrement(_) => {}
        Operand::Indexed(_, offset) => {
            if offset > i8::MAX || offset < i8::MIN {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: format!(
                        "The offset value given to a ST offset instruction must be a signed 8 bit value (max: {}, min: {})",
                        i8::MAX,
                        i8::MIN
                    ),
                });
            }
        }
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "Invalid desitination operand to ST instruction.".to_string(),
            });
        }
    }

    Ok(Instruction::St(dest, src))
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
            reason: "The destination of an LDI instruction must be a register value (R0-7)."
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
                    reason: format!("LDI immediate value must be 16 bits (max: {})", u16::MAX),
                });
            }
        }
        Operand::Label(_)
        | Operand::Indirect(_)
        | Operand::PostIncrement(_)
        | Operand::PreDecrement(_) => {}
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "Invalid source operand to LD instruction.".to_string(),
            });
        }
    }

    Ok(Instruction::Ld(dest, src))
}

// build and check operands for a load byte instruction
fn build_ld_b(ld_pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = ld_pair.as_span().start_pos().line_col().0;

    let mut inner = ld_pair.into_inner();
    let dest = build_operand(inner.next().unwrap());
    let src = build_operand(inner.next().unwrap());

    if let Operand::Register(_) = dest {
    } else {
        return Err(AssemblyError::StructuralError {
            line,
            reason: "The destination of an LD.b instruction must be a register value (R0-7)."
                .to_string(),
        });
    }

    match src {
        Operand::Indirect(_) | Operand::PreDecrement(_) | Operand::PostIncrement(_) => {}
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "The source operand of an LD.b instruction must be an indirect address or post-increment/pre-decrement value."
                    .to_string(),
            });
        }
    }

    Ok(Instruction::Ldb(dest, src))
}

// build and check operands for a store byte instruction
fn build_st_b(ld_pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = ld_pair.as_span().start_pos().line_col().0;

    let mut inner = ld_pair.into_inner();
    let dest = build_operand(inner.next().unwrap());
    let src = build_operand(inner.next().unwrap());

    if let Operand::Register(_) = src {
    } else {
        return Err(AssemblyError::StructuralError {
            line,
            reason: "The source of an ST.b instruction must be a register value (R0-7)."
                .to_string(),
        });
    }

    match dest {
        Operand::Indirect(_) | Operand::PreDecrement(_) | Operand::PostIncrement(_) => {}
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "The destination operand of an ST.b instruction must be an indirect address or post-increment/pre-decrement value."
                    .to_string(),
            });
        }
    }

    Ok(Instruction::Stb(dest, src))
}

// build and check operands for a load byte immediate instruction
fn build_ldi_b(ld_pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = ld_pair.as_span().start_pos().line_col().0;

    let mut inner = ld_pair.into_inner();
    let dest = build_operand(inner.next().unwrap());
    let src = build_operand(inner.next().unwrap());

    if let Operand::Register(_) = dest {
    } else {
        return Err(AssemblyError::StructuralError {
            line,
            reason: "The destination of an LDI.b instruction must be a register value (R0-7)."
                .to_string(),
        });
    }

    match src {
        Operand::Immediate(imm) => {
            if imm < 0 {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: "LDI.b immediate value must be unsigned.".to_string(),
                });
            } else if imm > (u8::MAX as i32) {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: format!(
                        "LDI.b immediate value must be an unsigned 8 bit value (max: {})",
                        u8::MAX
                    ),
                });
            }
        }
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "The source operand of an LDI.b instruction must be an immediate value."
                    .to_string(),
            });
        }
    }

    Ok(Instruction::Ldb(dest, src))
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
                    reason: format!("ADDI immediate value must be 16 bits (max: {})", u16::MAX),
                });
            }
        }
        Operand::Label(_) => {}
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason:
                    "The source operand of an ADDI instruction must be an immediate value or a label." .to_string(),
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

fn build_add_sp(add_pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = add_pair.as_span().start_pos().line_col().0;

    let mut inner = add_pair.into_inner();
    let operand = build_operand(inner.next().unwrap());

    match operand {
        Operand::Immediate(value) => {
            if value > i8::MAX as i32 || value < i8::MIN as i32 {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: format!(
                        "ADD SP immediate value must fit in a signed 8 bit value (max: {}, min: {})",
                        i8::MAX,
                        i8::MIN
                    ),
                });
            }
        }
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "ADD SP operand must be an immediate value.".to_string(),
            });
        }
    }

    Ok(Instruction::AddSp(operand))
}

// build and check operands for an add.b instruction
fn build_add_b(add_pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = add_pair.as_span().start_pos().line_col().0;

    let mut inner = add_pair.into_inner();
    let src = build_operand(inner.next().unwrap());

    if let Operand::Register(_) = src {
    } else {
        return Err(AssemblyError::StructuralError {
            line,
            reason: "Operand to an ADD.b Accumulator instruction must be a register (R0-R7)."
                .to_string(),
        });
    }

    Ok(Instruction::Addb(src))
}

// build and check operands for an add accumulator immediate instruction
fn build_addi_1_op(add_pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = add_pair.as_span().start_pos().line_col().0;

    let mut inner = add_pair.into_inner();
    let src = build_operand(inner.next().unwrap());

    match &src {
        Operand::Immediate(imm) => {
            if *imm < 0 {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: "ADDI immediate value must be unsigned.".to_string(),
                });
            } else if *imm > (u16::MAX as i32) {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: format!("ADDI immediate value must be 16 bits (max: {})", u16::MAX),
                });
            }
        }
        Operand::Label(_) => {}
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason:
                    "The source operand of an ADDI instruction must be an immediate value or a label." .to_string(),
            });
        }
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
                    reason: format!("SUBI immediate value must be 16 bits (max: {})", u16::MAX),
                });
            }
        }
        Operand::Label(_) => {}
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason:
                    "The source operand of an SUBI instruction must be an immediate value or a label." .to_string(),
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

// build and check operands for an sub.b instruction
fn build_sub_b(add_pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = add_pair.as_span().start_pos().line_col().0;

    let mut inner = add_pair.into_inner();
    let src = build_operand(inner.next().unwrap());

    if let Operand::Register(_) = src {
    } else {
        return Err(AssemblyError::StructuralError {
            line,
            reason: "Operand to an SUB.b Accumulator instruction must be a register (R0-R7)."
                .to_string(),
        });
    }

    Ok(Instruction::Subb(src))
}

// build and check operands for a sub accumulator immediate instruction
fn build_subi_1_op(sub_pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = sub_pair.as_span().start_pos().line_col().0;

    let mut inner = sub_pair.into_inner();
    let src = build_operand(inner.next().unwrap());

    match &src {
        Operand::Immediate(imm) => {
            if *imm < 0 {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: "SUBI immediate value must be unsigned.".to_string(),
                });
            } else if *imm > (u16::MAX as i32) {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: format!("SUBI immediate value must be 16 bits (max: {})", u16::MAX),
                });
            }
        }
        Operand::Label(_) => {}
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason:
                    "The source operand of an SUBI instruction must be an immediate value or a label." .to_string(),
            });
        }
    }

    Ok(Instruction::Sub(src, None))
}

// build and check operands for a lea instruction
fn build_lea(ld_pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = ld_pair.as_span().start_pos().line_col().0;

    let mut inner = ld_pair.into_inner();
    let dest = build_operand(inner.next().unwrap());
    let src = build_operand(inner.next().unwrap());

    if let Operand::Register(_) = dest {
    } else {
        return Err(AssemblyError::StructuralError {
            line,
            reason: "The destination of an LEA instruction must be a register value (R0-7)."
                .to_string(),
        });
    }

    match src {
        Operand::Indexed(_, offset) => {
            if offset > i8::MAX || offset < i8::MIN {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: format!(
                        "The offset value given to a LEA instruction must be a signed 8 bit value (max: {}, min: {})",
                        i8::MAX,
                        i8::MIN
                    ),
                });
            }
        }
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "Invalid operands to LEA instruction.".to_string(),
            });
        }
    }

    Ok(Instruction::Lea(dest, src))
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
                    reason: format!("ANDI immediate value must be 16 bits (max: {})", u16::MAX),
                });
            }
        }
        Operand::Label(_) => {}
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason:
                    "The source operand of an ANDI instruction must be an immediate value or a label." .to_string(),
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

// build and check operands for an and.b instruction
fn build_and_b(add_pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = add_pair.as_span().start_pos().line_col().0;

    let mut inner = add_pair.into_inner();
    let src = build_operand(inner.next().unwrap());

    if let Operand::Register(_) = src {
    } else {
        return Err(AssemblyError::StructuralError {
            line,
            reason: "Operand to an AND.b Accumulator instruction must be a register (R0-R7)."
                .to_string(),
        });
    }

    Ok(Instruction::Andb(src))
}

// build and check operands for a and accumulator immediate instruction
fn build_andi_1_op(and_pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = and_pair.as_span().start_pos().line_col().0;

    let mut inner = and_pair.into_inner();
    let src = build_operand(inner.next().unwrap());

    match &src {
        Operand::Immediate(imm) => {
            if *imm < 0 {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: "ANDI immediate value must be unsigned.".to_string(),
                });
            } else if *imm > (u16::MAX as i32) {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: format!("ANDI immediate value must be 16 bits (max: {})", u16::MAX),
                });
            }
        }
        Operand::Label(_) => {}
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason:
                    "The source operand of an ANDI instruction must be an immediate value or a label." .to_string(),
            });
        }
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
                    reason: format!("ORI immediate value must be 16 bits (max: {})", u16::MAX),
                });
            }
        }
        Operand::Label(_) => {}
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason:
                    "The source operand of an ORI instruction must be an immediate value or a label." .to_string(),
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

// build and check operands for an or.b instruction
fn build_or_b(add_pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = add_pair.as_span().start_pos().line_col().0;

    let mut inner = add_pair.into_inner();
    let src = build_operand(inner.next().unwrap());

    if let Operand::Register(_) = src {
    } else {
        return Err(AssemblyError::StructuralError {
            line,
            reason: "Operand to an OR.b Accumulator instruction must be a register (R0-R7)."
                .to_string(),
        });
    }

    Ok(Instruction::Orb(src))
}

// build and check operands for an or accumulator immediate instruction
fn build_ori_1_op(or_pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = or_pair.as_span().start_pos().line_col().0;

    let mut inner = or_pair.into_inner();
    let src = build_operand(inner.next().unwrap());

    match &src {
        Operand::Immediate(imm) => {
            if *imm < 0 {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: "ORI immediate value must be unsigned.".to_string(),
                });
            } else if *imm > (u16::MAX as i32) {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: format!("ORI immediate value must be 16 bits (max: {})", u16::MAX),
                });
            }
        }
        Operand::Label(_) => {}
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason:
                    "The source operand of an ORI instruction must be an immediate value or a label." .to_string(),
            });
        }
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
                    reason: format!("XORI immediate value must be 16 bits (max: {})", u16::MAX),
                });
            }
        }
        Operand::Label(_) => {}
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason:
                    "The source operand of an XORI instruction must be an immediate value or a label." .to_string(),
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

// build and check operands for an xor.b instruction
fn build_xor_b(add_pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = add_pair.as_span().start_pos().line_col().0;

    let mut inner = add_pair.into_inner();
    let src = build_operand(inner.next().unwrap());

    if let Operand::Register(_) = src {
    } else {
        return Err(AssemblyError::StructuralError {
            line,
            reason: "Operand to an XOR.b Accumulator instruction must be a register (R0-R7)."
                .to_string(),
        });
    }

    Ok(Instruction::Xorb(src))
}

// build and check operands for a xor accumulator immediate instruction
fn build_xori_1_op(xor_pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = xor_pair.as_span().start_pos().line_col().0;

    let mut inner = xor_pair.into_inner();
    let src = build_operand(inner.next().unwrap());

    match &src {
        Operand::Immediate(imm) => {
            if *imm < 0 {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: "XORI immediate value must be unsigned.".to_string(),
                });
            } else if *imm > (u16::MAX as i32) {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: format!("XORI immediate value must be 16 bits (max: {})", u16::MAX),
                });
            }
        }
        Operand::Label(_) => {}
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason:
                    "The source operand of an XORI instruction must be an immediate value or a label." .to_string(),
            });
        }
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
                    reason: format!("CMPI immediate value must be 16 bits (max: {})", u16::MAX),
                });
            }
        }
        Operand::Label(_) => {}
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason:
                    "The source operand of an CMPI instruction must be an immediate value or a label." .to_string(),
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

// build and check operands for an cmp.b instruction
fn build_cmp_b(add_pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = add_pair.as_span().start_pos().line_col().0;

    let mut inner = add_pair.into_inner();
    let src = build_operand(inner.next().unwrap());

    if let Operand::Register(_) = src {
    } else {
        return Err(AssemblyError::StructuralError {
            line,
            reason: "Operand to an CMP.b Accumulator instruction must be a register (R0-R7)."
                .to_string(),
        });
    }

    Ok(Instruction::Cmpb(src))
}

// build and check operands for a cmp accumulator immediate instruction
fn build_cmpi_1_op(pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = pair.as_span().start_pos().line_col().0;

    let mut inner = pair.into_inner();
    let src = build_operand(inner.next().unwrap());

    match &src {
        Operand::Immediate(imm) => {
            if *imm < 0 {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: "CMPI immediate value must be unsigned.".to_string(),
                });
            } else if *imm > (u16::MAX as i32) {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: format!("CMPI immediate value must be 16 bits (max: {})", u16::MAX),
                });
            }
        }
        Operand::Label(_) => {}
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "The operand of a CMPI instruction must be an immediate value or a label."
                    .to_string(),
            });
        }
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

// build and check operands for an ADC accumulator immediate instruction
fn build_adci_1_op(pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = pair.as_span().start_pos().line_col().0;

    let mut inner = pair.into_inner();
    let src = build_operand(inner.next().unwrap());

    match &src {
        Operand::Immediate(imm) => {
            if *imm < 0 {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: "ADCI immediate value must be unsigned.".to_string(),
                });
            } else if *imm > (u16::MAX as i32) {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: format!("ADCI immediate value must be 16 bits (max: {})", u16::MAX),
                });
            }
        }
        Operand::Label(_) => {}
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "The operand of an ADCI instruction must be an immediate value or a label."
                    .to_string(),
            });
        }
    }

    Ok(Instruction::Adc(src, None))
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

// build and check operands for an SBC accumulator immediate instruction
fn build_sbci_1_op(pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = pair.as_span().start_pos().line_col().0;

    let mut inner = pair.into_inner();
    let src = build_operand(inner.next().unwrap());

    match &src {
        Operand::Immediate(imm) => {
            if *imm < 0 {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: "SBCI immediate value must be unsigned.".to_string(),
                });
            } else if *imm > (u16::MAX as i32) {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: format!("SBCI immediate value must be 16 bits (max: {})", u16::MAX),
                });
            }
        }
        Operand::Label(_) => {}
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "The operand of an SBCI instruction must be an immediate value or a label."
                    .to_string(),
            });
        }
    }

    Ok(Instruction::Sbc(src, None))
}

fn build_push(pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = pair.as_span().start_pos().line_col().0;

    let mut inner = pair.into_inner();
    let operand = build_operand(inner.next().unwrap());

    match operand {
        Operand::Register(_) => {}
        Operand::Immediate(value) => {
            if value < 0 {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: "PUSH immediate value must be unsigned.".to_string(),
                });
            } else if value > (u16::MAX as i32) {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: format!("PUSH immediate value must be 16 bits (max: {})", u16::MAX),
                });
            }
        }
        Operand::Label(_) => {}
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "Invalid operand to PUSH instruction.".to_string(),
            });
        }
    }

    Ok(Instruction::Push(operand))
}

fn build_pop(pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = pair.as_span().start_pos().line_col().0;

    let mut inner = pair.into_inner();
    let operand = build_operand(inner.next().unwrap());

    match operand {
        Operand::Register(_) => {}
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "Operand to a POP instruction must be a register (R0-R7).".to_string(),
            });
        }
    }

    Ok(Instruction::Pop(operand))
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
                reason:
                    "Operand to a JMP instruction must be an indirect address ((R0-R7)), label or immediate address." .to_string(),
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
                        "JR immediate relative value must be 8 bits (max: {}, min: {})",
                        i8::MAX,
                        i8::MIN
                    ),
                });
            }
        }
        Operand::Label(_) => {} // Corrected: Removed unnecessary escape for underscore
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
                        "JRcc immediate relative value must be 8 bits (max: {}, min: {})",
                        i8::MAX,
                        i8::MIN
                    ),
                });
            }
        }
        Operand::Label(_) => {} // Corrected: Removed unnecessary escape for underscore
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
                        "DJNZ immediate relative value must be 8 bits (max: {}, min: {})",
                        i8::MAX,
                        i8::MIN
                    ),
                });
            }
        }
        Operand::Label(_) => {} // Corrected: Removed unnecessary escape for underscore
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason:
                    "Operand to a DJNZ instruction must be an immediate relative value or a label."
                        .to_string(),
            });
        }
    }

    Ok(Instruction::Djnz(op))
}

// build and check operands for a 1 operand inc instruction
fn build_inc(pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = pair.as_span().start_pos().line_col().0;

    let mut inner = pair.into_inner();
    let src = build_operand(inner.next().unwrap());

    if let Operand::Register(_) = src {
    } else {
        return Err(AssemblyError::StructuralError {
            line,
            reason: "Operand to an INC instruction must be a register (R0-R7).".to_string(),
        });
    }

    Ok(Instruction::Inc(src))
}

// build and check operands for a 1 operand dec instruction
fn build_dec(pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = pair.as_span().start_pos().line_col().0;

    let mut inner = pair.into_inner();
    let src = build_operand(inner.next().unwrap());

    if let Operand::Register(_) = src {
    } else {
        return Err(AssemblyError::StructuralError {
            line,
            reason: "Operand to a DEC instruction must be a register (R0-R7).".to_string(),
        });
    }

    Ok(Instruction::Dec(src))
}

// build and check operands for a sra instruction
fn build_sra(pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = pair.as_span().start_pos().line_col().0;
    let mut inner = pair.into_inner();
    let op = build_operand(inner.next().unwrap());

    if let Operand::Register(_) = op {
    } else {
        return Err(AssemblyError::StructuralError {
            line,
            reason: "Operand for SRA instruction must be a register.".to_string(),
        });
    }

    Ok(Instruction::Sra(op))
}

// build and check operands for a shl instruction
fn build_shl(pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = pair.as_span().start_pos().line_col().0;
    let mut inner = pair.into_inner();
    let op = build_operand(inner.next().unwrap());

    if let Operand::Register(_) = op {
    } else {
        return Err(AssemblyError::StructuralError {
            line,
            reason: "Operand for Shl instruction must be a register.".to_string(),
        });
    }

    Ok(Instruction::Shl(op))
}

// build and check operands for a shr instruction
fn build_shr(pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = pair.as_span().start_pos().line_col().0;
    let mut inner = pair.into_inner();
    let op = build_operand(inner.next().unwrap());

    if let Operand::Register(_) = op {
    } else {
        return Err(AssemblyError::StructuralError {
            line,
            reason: "Operand for shr instruction must be a register.".to_string(),
        });
    }

    Ok(Instruction::Shr(op))
}

// build and check operands for a rol instruction
fn build_rol(pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = pair.as_span().start_pos().line_col().0;
    let mut inner = pair.into_inner();
    let op = build_operand(inner.next().unwrap());

    if let Operand::Register(_) = op {
    } else {
        return Err(AssemblyError::StructuralError {
            line,
            reason: "Operand for rol instruction must be a register.".to_string(),
        });
    }

    Ok(Instruction::Rol(op))
}

// build and check operands for a ror instruction
fn build_ror(pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = pair.as_span().start_pos().line_col().0;
    let mut inner = pair.into_inner();
    let op = build_operand(inner.next().unwrap());

    if let Operand::Register(_) = op {
    } else {
        return Err(AssemblyError::StructuralError {
            line,
            reason: "Operand for ror instruction must be a register.".to_string(),
        });
    }

    Ok(Instruction::Ror(op))
}

// build and check operands for a call instruction
fn build_call(pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = pair.as_span().start_pos().line_col().0;

    let mut inner = pair.into_inner();
    let op = build_operand(inner.next().unwrap());

    match op {
        Operand::Indirect(_) | Operand::Label(_) => {} // Corrected: Removed unnecessary escape for underscore
        Operand::Immediate(imm) => {
            if imm > (u16::MAX as i32) || imm < 0 {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: format!(
                        "CALL immediate address must be an unsigned 16 bit value (max: {}, min: 0).",
                        u16::MAX,
                    ),
                });
            }
        }
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason:
                    "Operand to a CALL instruction must be an indirect address ((R0-R7)), label or immediate address." .to_string(),
            });
        }
    }

    Ok(Instruction::Call(op))
}

// build and check operands for a conditional call instruction
fn build_callcc(pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = pair.as_span().start_pos().line_col().0;

    let mut inner = pair.into_inner();
    let cc = build_condition_code(inner.next().unwrap());
    let op = build_operand(inner.next().unwrap());

    match op {
        Operand::Label(_) => {} // Corrected: Removed unnecessary escape for underscore
        Operand::Immediate(imm) => {
            if imm > (u16::MAX as i32) || imm < 0 {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: format!(
                        "CALLcc immediate address must be an unsigned 16 bit value (max: {}, min: 0).",
                        u16::MAX,
                    ),
                });
            }
        }
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "Operand to a CALLcc instruction must be a label or immediate address."
                    .to_string(),
            });
        }
    }

    Ok(Instruction::Callcc(cc, op))
}

// build and check operands for a SYSCALL instruction
fn build_syscall(pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = pair.as_span().start_pos().line_col().0;

    let mut inner = pair.into_inner();
    let op = build_operand(inner.next().unwrap());

    match op {
        Operand::Immediate(imm) => {
            if imm > (u8::MAX as i32) || imm < 0 {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: format!(
                        "SYSCALL immediate value must be 8 bits (max: {}, min: 0).",
                        u8::MAX,
                    ),
                });
            }
        }
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "Operand to a SYSCALL instruction must be an immediate 8-bit value."
                    .to_string(),
            });
        }
    }

    Ok(Instruction::Syscall(op))
}

// build and check operands for a bit check instruction
fn build_bit(add_pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = add_pair.as_span().start_pos().line_col().0;

    let mut inner = add_pair.into_inner();
    let dest = build_operand(inner.next().unwrap());
    let b = build_operand(inner.next().unwrap());

    match &dest {
        Operand::Register(_) | Operand::Indirect(_) | Operand::Absolute(_) => {}
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "BIT destination operand must be a register, indirect address or absolute address".to_string(),
            });
        }
    }

    match &b {
        Operand::Immediate(imm) => {
            if *imm > 7 || *imm < 0 {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: "BIT bit ID operand must be between 0 and 7.".to_string(),
                });
            }
        }
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "BIT bit ID operand must be an unsigned immediate value".to_string(),
            });
        }
    }

    Ok(Instruction::Bit(dest, b))
}

// build and check operands for a set instruction
fn build_set(add_pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = add_pair.as_span().start_pos().line_col().0;

    let mut inner = add_pair.into_inner();
    let dest = build_operand(inner.next().unwrap());
    let b = build_operand(inner.next().unwrap());

    match &dest {
        Operand::Register(_) | Operand::Indirect(_) | Operand::Absolute(_) => {}
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "SET destination operand must be a register, indirect address or absolute address.".to_string(),
            });
        }
    }

    match &b {
        Operand::Immediate(imm) => {
            if *imm > 7 || *imm < 0 {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: "SET bit ID operand must be between 0 and 7.".to_string(),
                });
            }
        }
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "SET bit ID operand must be an unsigned immediate value.".to_string(),
            });
        }
    }

    Ok(Instruction::Set(dest, b))
}

// build and check operands for a res instruction
fn build_res(add_pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = add_pair.as_span().start_pos().line_col().0;

    let mut inner = add_pair.into_inner();
    let dest = build_operand(inner.next().unwrap());
    let b = build_operand(inner.next().unwrap());

    match &dest {
        Operand::Register(_) | Operand::Indirect(_) | Operand::Absolute(_) => {}
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "RES destination operand must be a register, indirect address or absolute address.".to_string(),
            });
        }
    }

    match &b {
        Operand::Immediate(imm) => {
            if *imm > 7 || *imm < 0 {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason: "RES bit ID operand must be between 0 and 7.".to_string(),
                });
            }
        }
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "RES bit ID operand must be an unsigned immediate value.".to_string(),
            });
        }
    }

    Ok(Instruction::Res(dest, b))
}

// ------------- build instruction –------------

// Helper to build an Instruction from a pest Pair
fn build_instruction(pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    match pair.as_rule() {
        Rule::nop => Ok(Instruction::Nop),
        Rule::halt => Ok(Instruction::Halt),
        Rule::st_2_op => build_st_2_op(pair),
        Rule::ld_2_op => build_ld_2_op(pair),
        Rule::ldi_2_op => build_ldi_2_op(pair),
        Rule::add_sp => build_add_sp(pair),
        Rule::add_2_op => build_add_2_op(pair),
        Rule::addi_2_op => build_addi_2_op(pair),
        Rule::add_1_op => build_add_1_op(pair),
        Rule::addi_1_op => build_addi_1_op(pair),
        Rule::sub_2_op => build_sub_2_op(pair),
        Rule::subi_2_op => build_subi_2_op(pair),
        Rule::sub_1_op => build_sub_1_op(pair),
        Rule::subi_1_op => build_subi_1_op(pair),
        Rule::and_2_op => build_and_2_op(pair),
        Rule::andi_2_op => build_andi_2_op(pair),
        Rule::and_1_op => build_and_1_op(pair),
        Rule::andi_1_op => build_andi_1_op(pair),
        Rule::or_2_op => build_or_2_op(pair),
        Rule::ori_2_op => build_ori_2_op(pair),
        Rule::or_1_op => build_or_1_op(pair),
        Rule::ori_1_op => build_ori_1_op(pair),
        Rule::xor_2_op => build_xor_2_op(pair),
        Rule::xori_2_op => build_xori_2_op(pair),
        Rule::xor_1_op => build_xor_1_op(pair),
        Rule::xori_1_op => build_xori_1_op(pair),
        Rule::cmp_2_op => build_cmp_2_op(pair),
        Rule::cmpi_2_op => build_cmpi_2_op(pair),
        Rule::cmp_1_op => build_cmp_1_op(pair),
        Rule::cmpi_1_op => build_cmpi_1_op(pair),
        Rule::adc_2_op => build_adc_2_op(pair),
        Rule::adci_1_op => build_adci_1_op(pair),
        Rule::sbc_2_op => build_sbc_2_op(pair),
        Rule::sbci_1_op => build_sbci_1_op(pair),
        Rule::ldi_b => build_ldi_b(pair),
        Rule::add_b => build_add_b(pair),
        Rule::sub_b => build_sub_b(pair),
        Rule::and_b => build_and_b(pair),
        Rule::or_b => build_or_b(pair),
        Rule::xor_b => build_xor_b(pair),
        Rule::cmp_b => build_cmp_b(pair),
        Rule::ld_b => build_ld_b(pair),
        Rule::st_b => build_st_b(pair),
        Rule::push_f => Ok(Instruction::PushF),
        Rule::pop_f => Ok(Instruction::PopF),
        Rule::push_op => build_push(pair),
        Rule::pop_op => build_pop(pair),
        Rule::neg => Ok(Instruction::Neg),
        Rule::not => Ok(Instruction::Not),
        Rule::swap => Ok(Instruction::Swap),
        Rule::jmp => build_jmp(pair),
        Rule::jr => build_jr(pair),
        Rule::jmp_con => build_jcc(pair),
        Rule::jr_con => build_jrcc(pair),
        Rule::djnz => build_djnz(pair),
        Rule::call => build_call(pair),
        Rule::call_con => build_callcc(pair),
        Rule::syscall => build_syscall(pair),
        Rule::ccf => Ok(Instruction::Ccf),
        Rule::scf => Ok(Instruction::Scf),
        Rule::rcf => Ok(Instruction::Rcf),
        Rule::enter => Ok(Instruction::Enter),
        Rule::leave => Ok(Instruction::Leave),
        Rule::ret => Ok(Instruction::Ret),
        Rule::reti => Ok(Instruction::Reti),
        Rule::ei => Ok(Instruction::Ei),
        Rule::di => Ok(Instruction::Di),
        Rule::inc => build_inc(pair),
        Rule::dec => build_dec(pair),
        Rule::sra => build_sra(pair),
        Rule::shl => build_shl(pair),
        Rule::shr => build_shr(pair),
        Rule::rol => build_rol(pair),
        Rule::ror => build_ror(pair),
        Rule::bit => build_bit(pair),
        Rule::set => build_set(pair),
        Rule::res => build_res(pair),
        Rule::lea => build_lea(pair),
        // ... add cases for all other instructions
        _ => unreachable!("Unknown instruction rule: {:?}", pair.as_rule()),
    }
}

// ------------- directives –------------

fn build_directive(pair: Pair<Rule>) -> Result<Directive, AssemblyError> {
    match pair.as_rule() {
        Rule::org_directive => build_org_dir(pair),
        _ => unreachable!("Unknown directive rule: {:?}", pair.as_rule()),
    }
}

fn build_org_dir(pair: Pair<Rule>) -> Result<Directive, AssemblyError> {
    let line = pair.as_span().start_pos().line_col().0;
    let op = build_operand(pair.into_inner().next().unwrap());

    match op {
        Operand::Immediate(addr) => {
            if addr > u16::MAX as i32 || addr < 0 {
                return Err(AssemblyError::StructuralError {
                    line,
                    reason:
                        ".org address must be an unsigned 16 bit value (max: 0xFFFF, min: 0x0000)"
                            .to_string(),
                });
            }
        }
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: ".org argument must be an immediate value.".to_string(),
            });
        }
    }

    Ok(Directive::Org(op))
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
    fn test_parse_add_b() {
        let source = "add.b r1\n";
        let result = parse_source(source);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0].instruction,
            Some(Instruction::Addb(Operand::Register(Register::R1)))
        );
    }

    #[test]
    fn test_parse_sub_b() {
        let source = "sub.b r2\n";
        let result = parse_source(source);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0].instruction,
            Some(Instruction::Subb(Operand::Register(Register::R2)))
        );
    }

    #[test]
    fn test_parse_and_b() {
        let source = "and.b r3\n";
        let result = parse_source(source);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0].instruction,
            Some(Instruction::Andb(Operand::Register(Register::R3)))
        );
    }

    #[test]
    fn test_parse_or_b() {
        let source = "or.b r4\n";
        let result = parse_source(source);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0].instruction,
            Some(Instruction::Orb(Operand::Register(Register::R4)))
        );
    }

    #[test]
    fn test_parse_xor_b() {
        let source = "xor.b r5\n";
        let result = parse_source(source);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0].instruction,
            Some(Instruction::Xorb(Operand::Register(Register::R5)))
        );
    }

    #[test]
    fn test_parse_cmp_b() {
        let source = "cmp.b r6\n";
        let result = parse_source(source);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0].instruction,
            Some(Instruction::Cmpb(Operand::Register(Register::R6)))
        );
    }

    #[test]
    fn test_parse_ldi_b() {
        let source = "ldi.b r1, 0xAB\n";
        let result = parse_source(source);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0].instruction,
            Some(Instruction::Ldb(
                Operand::Register(Register::R1),
                Operand::Immediate(0xAB)
            ))
        );
    }

    #[test]
    fn test_parse_ld_indexed() {
        let source = "ld r0, (r1, 16)\n";
        let result = parse_source(source);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0].instruction,
            Some(Instruction::Ld(
                Operand::Register(Register::R0),
                Operand::Indexed(Register::R1, 16)
            ))
        );
    }

    #[test]
    fn test_parse_st_indexed() {
        let source = "st (r2, -1), r3\n";
        let result = parse_source(source);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0].instruction,
            Some(Instruction::St(
                Operand::Indexed(Register::R2, -1),
                Operand::Register(Register::R3)
            ))
        );
    }

    #[test]
    fn test_parse_lea_indexed() {
        let source = "lea r4, (r5, 32)\n";
        let result = parse_source(source);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0].instruction,
            Some(Instruction::Lea(
                Operand::Register(Register::R4),
                Operand::Indexed(Register::R5, 32)
            ))
        );
    }

    #[test]
    fn test_parse_ld_post_increment() {
        let source = "ld r6, (r7)+\n";
        let result = parse_source(source);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0].instruction,
            Some(Instruction::Ld(
                Operand::Register(Register::R6),
                Operand::PostIncrement(Register::R7)
            ))
        );
    }

    #[test]
    fn test_parse_st_post_increment() {
        let source = "st (r0)+, r1\n";
        let result = parse_source(source);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0].instruction,
            Some(Instruction::St(
                Operand::PostIncrement(Register::R0),
                Operand::Register(Register::R1)
            ))
        );
    }

    #[test]
    fn test_parse_ld_pre_decrement() {
        let source = "ld r2, -(r3)\n";
        let result = parse_source(source);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0].instruction,
            Some(Instruction::Ld(
                Operand::Register(Register::R2),
                Operand::PreDecrement(Register::R3)
            ))
        );
    }

    #[test]
    fn test_parse_st_pre_decrement() {
        let source = "st -(r4), r5\n";
        let result = parse_source(source);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0].instruction,
            Some(Instruction::St(
                Operand::PreDecrement(Register::R4),
                Operand::Register(Register::R5)
            ))
        );
    }

    #[test]
    fn test_parse_ld_b_post_increment() {
        let source = "ld.b r6, (r7)+\n";
        let result = parse_source(source);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0].instruction,
            Some(Instruction::Ldb(
                Operand::Register(Register::R6),
                Operand::PostIncrement(Register::R7)
            ))
        );
    }

    #[test]
    fn test_parse_st_b_post_increment() {
        let source = "st.b (r0)+, r1\n";
        let result = parse_source(source);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0].instruction,
            Some(Instruction::Stb(
                Operand::PostIncrement(Register::R0),
                Operand::Register(Register::R1)
            ))
        );
    }

    #[test]
    fn test_parse_ld_b_pre_decrement() {
        let source = "ld.b r2, -(r3)\n";
        let result = parse_source(source);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0].instruction,
            Some(Instruction::Ldb(
                Operand::Register(Register::R2),
                Operand::PreDecrement(Register::R3)
            ))
        );
    }

    #[test]
    fn test_parse_st_b_pre_decrement() {
        let source = "st.b -(r4), r5\n";
        let result = parse_source(source);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0].instruction,
            Some(Instruction::Stb(
                Operand::PreDecrement(Register::R4),
                Operand::Register(Register::R5)
            ))
        );
    }

    #[test]
    fn test_parse_bit_register() {
        let source = "bit r1, 7\n";
        let result = parse_source(source);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0].instruction,
            Some(Instruction::Bit(
                Operand::Register(Register::R1),
                Operand::Immediate(7)
            ))
        );
    }

    #[test]
    fn test_parse_set_absolute() {
        let source = "set (0x1234), 0\n";
        let result = parse_source(source);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0].instruction,
            Some(Instruction::Set(
                Operand::Absolute(0x1234),
                Operand::Immediate(0)
            ))
        );
    }

    #[test]
    fn test_parse_res_indirect() {
        let source = "res (r2), 3\n";
        let result = parse_source(source);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0].instruction,
            Some(Instruction::Res(
                Operand::Indirect(Register::R2),
                Operand::Immediate(3)
            ))
        );
    }
}
