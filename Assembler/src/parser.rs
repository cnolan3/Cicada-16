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
            // let t = inner.next().unwrap().into_inner().next().unwrap();
            // println!("{}", inner.next().unwrap().into_inner().next().unwrap());
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

// ------------- operand builder helpers -------------

// Helper to build an Operand from a pest Pair
fn build_operand(pair: Pair<Rule>) -> Operand {
    let inner_pair = pair.into_inner().next().unwrap();
    match inner_pair.as_rule() {
        Rule::register => build_register(inner_pair),
        Rule::immediate => build_immediate(inner_pair),
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
fn build_immediate(pair: Pair<Rule>) -> Operand {
    let hex_str = &pair.as_str()[2..];
    let value = u16::from_str_radix(hex_str, 16).unwrap();
    Operand::Immediate(value)
}

// build an identifier object
fn build_identifier(pair: Pair<Rule>) -> Operand {
    Operand::Label(pair.as_str().to_string())
}

// build an indirect object
fn build_indirect(pair: Pair<Rule>) -> Operand {
    let reg_char = pair.as_str().chars().nth(2).unwrap();
    println!("{}", reg_char);
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

// ------------- instruction builder helpers -------------

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

    Ok(Instruction::Ld(dest, src))
}

// build and check operands for a 2 operand add instruction
fn build_add_2_op(add_pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = add_pair.as_span().start_pos().line_col().0;

    let mut inner = add_pair.into_inner();
    let dest = build_operand(inner.next().unwrap());
    let src = build_operand(inner.next().unwrap());

    match (&dest, &src) {
        (Operand::Label(_), _) | (_, Operand::Label(_)) => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "A label is not a valid operand to an ADD instruction.".to_string(),
            });
        }
        (Operand::Register(_), Operand::Register(_)) => {}
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "Invalid operands to ADD instruction.".to_string(),
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
        (Operand::Label(_), _) | (_, Operand::Label(_)) => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "A label is not a valid operand to a SUB instruction.".to_string(),
            });
        }
        (Operand::Register(_), Operand::Register(_)) => {}
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "Invalid operands to SUB instruction.".to_string(),
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
        (Operand::Label(_), _) | (_, Operand::Label(_)) => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "A label is not a valid operand to an AND instruction.".to_string(),
            });
        }
        (Operand::Register(_), Operand::Register(_)) => {}
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "Invalid operands to AND instruction.".to_string(),
            });
        }
    }

    Ok(Instruction::And(dest, Some(src)))
}

// build and check operands for a 2 operand or instruction
fn build_or_2_op(pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = pair.as_span().start_pos().line_col().0;

    let mut inner = pair.into_inner();
    let dest = build_operand(inner.next().unwrap());
    let src = build_operand(inner.next().unwrap());

    match (&dest, &src) {
        (Operand::Label(_), _) | (_, Operand::Label(_)) => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "A label is not a valid operand to an OR instruction.".to_string(),
            });
        }
        (Operand::Register(_), Operand::Register(_)) => {}
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "Invalid operands to OR instruction.".to_string(),
            });
        }
    }

    Ok(Instruction::Or(dest, Some(src)))
}

// build and check operands for a 2 operand xor instruction
fn build_xor_2_op(pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = pair.as_span().start_pos().line_col().0;

    let mut inner = pair.into_inner();
    let dest = build_operand(inner.next().unwrap());
    let src = build_operand(inner.next().unwrap());

    match (&dest, &src) {
        (Operand::Label(_), _) | (_, Operand::Label(_)) => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "A label is not a valid operand to a XOR instruction.".to_string(),
            });
        }
        (Operand::Register(_), Operand::Register(_)) => {}
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "Invalid operands to XOR instruction.".to_string(),
            });
        }
    }

    Ok(Instruction::Xor(dest, Some(src)))
}

// build and check operands for a 2 operand cmp instruction
fn build_cmp_2_op(pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = pair.as_span().start_pos().line_col().0;

    let mut inner = pair.into_inner();
    let dest = build_operand(inner.next().unwrap());
    let src = build_operand(inner.next().unwrap());

    match (&dest, &src) {
        (Operand::Label(_), _) | (_, Operand::Label(_)) => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "A label is not a valid operand to a CMP instruction.".to_string(),
            });
        }
        (Operand::Register(_), Operand::Register(_)) => {}
        _ => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "Invalid operands to CMP instruction.".to_string(),
            });
        }
    }

    Ok(Instruction::Cmp(dest, Some(src)))
}

// build and check operands for a 2 operand adc instruction
fn build_adc_2_op(pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    let line = pair.as_span().start_pos().line_col().0;

    let mut inner = pair.into_inner();
    let dest = build_operand(inner.next().unwrap());
    let src = build_operand(inner.next().unwrap());

    match (&dest, &src) {
        (Operand::Label(_), _) | (_, Operand::Label(_)) => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "A label is not a valid operand to an ADC instruction.".to_string(),
            });
        }
        (Operand::Register(_), Operand::Register(_)) => {}
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
        (Operand::Label(_), _) | (_, Operand::Label(_)) => {
            return Err(AssemblyError::StructuralError {
                line,
                reason: "A label is not a valid operand to an SBC instruction.".to_string(),
            });
        }
        (Operand::Register(_), Operand::Register(_)) => {}
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
        Operand::Indirect(_) | Operand::Label(_) | Operand::Immediate(_) => {}
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

// ------------- build instruction -------------

// Helper to build an Instruction from a pest Pair
fn build_instruction(pair: Pair<Rule>) -> Result<Instruction, AssemblyError> {
    match pair.as_rule() {
        Rule::nop => Ok(Instruction::Nop),
        Rule::ld_2_op => build_ld_2_op(pair),
        Rule::add_2_op => build_add_2_op(pair),
        Rule::add_1_op => build_add_1_op(pair),
        Rule::sub_2_op => build_sub_2_op(pair),
        Rule::sub_1_op => build_sub_1_op(pair),
        Rule::and_2_op => build_and_2_op(pair),
        Rule::or_2_op => build_or_2_op(pair),
        Rule::xor_2_op => build_xor_2_op(pair),
        Rule::cmp_2_op => build_cmp_2_op(pair),
        Rule::adc_2_op => build_adc_2_op(pair),
        Rule::sbc_2_op => build_sbc_2_op(pair),
        Rule::jmp => build_jmp(pair),
        // ... add cases for all other instructions
        _ => unreachable!("Unknown instruction rule: {:?}", pair.as_rule()),
    }
}

