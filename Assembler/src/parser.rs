use nom::{
    IResult,
    branch::alt,
    bytes::complete::{tag, tag_no_case},
    character::complete::{alpha1, alphanumeric1, char, hex_digit1, multispace0, multispace1},
    combinator::{map, map_res, opt, recognize},
    sequence::{delimited, pair, preceded, terminated, tuple},
};
// Import your AST structures
use crate::ast::*;

// --- Basic Building Blocks ---

// Parse a register like "R0", "r1", "R7"
fn parse_register(input: &str) -> IResult<&str, Register> {
    map_res(preceded(tag_no_case("r"), one_of("01234567")), |s: char| {
        match s {
            '0' => Ok(Register::R0),
            '1' => Ok(Register::R1),
            '2' => Ok(Register::R2),
            '3' => Ok(Register::R3),
            '4' => Ok(Register::R4),
            '5' => Ok(Register::R5),
            '6' => Ok(Register::R6),
            '7' => Ok(Register::R7),
            _ => Err("Invalid register number"), // Should be unreachable due to one_of
        }
    })(input)
}

// Parse a hexadecimal number like "0x1A" or "$FF"
fn parse_hex_number(input: &str) -> IResult<&str, u16> {
    map_res(
        preceded(alt((tag("0x"), tag("$"))), hex_digit1),
        |s: &str| u16::from_str_radix(s, 16),
    )(input)
}

// Parse a label name (e.g., "loop_start")
fn parse_label_name(input: &str) -> IResult<&str, String> {
    map(
        recognize(pair(alpha1, many0(alt((alphanumeric1, tag("_")))))),
        |s: &str| s.to_string(),
    )(input)
}

// --- Operand Parser ---

fn parse_operand(input: &str) -> IResult<&str, Operand> {
    alt((
        // 1. Register direct: R1
        map(parse_register, Operand::Register),
        // 2. Immediate value: 0x1234
        map(parse_hex_number, Operand::Immediate),
        // 3. Label reference: my_label
        map(parse_label_name, Operand::Label),
        // 4. Indirect: (R1) - Add this later
        // 5. Indexed: (R1, 0x10) - Add this later
    ))(input)
}

// --- Instruction Parsers ---

// Example: Parse "NOP"
fn parse_nop(input: &str) -> IResult<&str, Instruction> {
    map(tag_no_case("nop"), |_| Instruction::Nop)(input)
}

// Example: Parse "LD R0, R1" or "LDI R0, 0x1234"
fn parse_ld(input: &str) -> IResult<&str, Instruction> {
    map(
        tuple((
            tag_no_case("ld"), // Mnemonic (LDI is handled by operand type)
            multispace1,
            parse_operand,                                  // Destination operand
            delimited(multispace0, char(','), multispace0), // Comma separator
            parse_operand,                                  // Source operand
        )),
        |(_, _, dest, _, src)| Instruction::Ld(dest, src),
    )(input)
}

// --- Line Parser ---

// Parse an entire line: [label:] [instruction] [comment]
pub fn parse_line(input: &str) -> IResult<&str, AssemblyLine> {
    // 1. Parse optional label "my_label:"
    let (input, label) = opt(terminated(parse_label_name, char(':')))(input)?;

    // 2. Eat whitespace
    let (input, _) = multispace0(input)?;

    // 3. Parse optional instruction
    let (input, instruction) = opt(alt((parse_nop, parse_ld)))(input)?;
    // TODO: Add parsers for comments and directives

    Ok((input, AssemblyLine { label, instruction }))
}
