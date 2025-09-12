use crate::ast::*;
use pest::Parser;
use pest::iterators::{Pair, Pairs};
use pest_derive::Parser;

// Derive the parser from our grammar file.
#[derive(Parser)]
#[grammar = "./grammar.pest"]
pub struct CicadaParser;

// Main parsing function that takes the entire source code string.
pub fn parse_source(source: &str) -> Result<Vec<AssemblyLine>, pest::error::Error<Rule>> {
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
                ));
            }
        }

        // Only add non-empty lines to our AST
        if assembly_line.label.is_some() || assembly_line.instruction.is_some() {
            ast.push(assembly_line);
        }
    }

    Ok(ast)
}

// Helper to build an Instruction from a pest Pair
fn build_instruction(pair: Pair<Rule>) -> Instruction {
    match pair.as_rule() {
        Rule::nop => Instruction::Nop,
        Rule::ld => {
            let mut inner = pair.into_inner();
            let dest = build_operand(inner.next().unwrap());
            let src = build_operand(inner.next().unwrap());
            Instruction::Ld(dest, src)
        }
        // ... add cases for all other instructions
        _ => unreachable!("Unknown instruction rule: {:?}", pair.as_rule()),
    }
}

// Helper to build an Operand from a pest Pair
fn build_operand(pair: Pair<Rule>) -> Operand {
    let inner_pair = pair.into_inner().next().unwrap();
    match inner_pair.as_rule() {
        Rule::register => {
            let reg_char = inner_pair.as_str().chars().nth(1).unwrap();
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
        Rule::immediate => {
            let hex_str = &inner_pair.as_str()[2..];
            let value = u16::from_str_radix(hex_str, 16).unwrap();
            Operand::Immediate(value)
        }
        Rule::identifier => Operand::Label(inner_pair.as_str().to_string()),
        _ => unreachable!("Unknown operand rule: {:?}", inner_pair.as_rule()),
    }
}
