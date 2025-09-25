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

mod ast_builder;

use crate::ast::*;
use anyhow::Result;
use ast_builder::AstBuilder;
use pest::Parser;
use pest::iterators::Pair;
use pest_derive::Parser;

// Derive the parser from our grammar file.
#[derive(Parser)]
#[grammar = "./grammar.pest"]
pub struct CicadaParser;

// Main parsing function that takes the entire source code string.
pub fn parse_source(source: &str) -> Result<Vec<AssemblyLine>> {
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
            assembly_line.line_number = pair.as_span().start_pos().line_col().0;
            match pair.as_rule() {
                Rule::label => {
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
                Rule::instruction => {
                    assembly_line.instruction = Some(build_instruction(
                        inner.next().unwrap().into_inner().next().unwrap(),
                    )?);
                }
                Rule::directive => {
                    assembly_line.directive = Some(build_directive(
                        inner.next().unwrap().into_inner().next().unwrap(),
                    )?);
                }
                _ => {}
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

fn build_instruction(pair: Pair<Rule>) -> Result<Instruction> {
    let builder = AstBuilder::new(pair.clone());
    builder.build_instruction()
}

fn build_directive(pair: Pair<Rule>) -> Result<Directive> {
    let builder = AstBuilder::new(pair.clone());
    builder.build_directive()
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
    fn test_parse_sub_acc() {
        let source = "sub r1\n";
        let result = parse_source(source);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0].instruction,
            Some(Instruction::SubAcc(Register::R1))
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
            Some(Instruction::AndReg(Register::R2, Register::R3))
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
            Some(Instruction::OrReg(Register::R4, Register::R5))
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
            Some(Instruction::XorReg(Register::R6, Register::R7))
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
            Some(Instruction::CmpReg(Register::R0, Register::R1))
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
            Some(Instruction::AdcReg(Register::R2, Register::R3))
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
            Some(Instruction::SbcReg(Register::R4, Register::R5))
        );
        assert_eq!(lines[0].label, None);
    }

    #[test]
    fn test_parse_and_acc() {
        let source = "and r1\n";
        let result = parse_source(source);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0].instruction,
            Some(Instruction::AndAcc(Register::R1))
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
            Some(Instruction::AddBAcc(Register::R1))
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
            Some(Instruction::SubBAcc(Register::R2))
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
            Some(Instruction::AndBAcc(Register::R3))
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
            Some(Instruction::OrBAcc(Register::R4))
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
            Some(Instruction::XorBAcc(Register::R5))
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
            Some(Instruction::CmpBAcc(Register::R6))
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
            Some(Instruction::LdiB(Register::R1, 0xAB))
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
            Some(Instruction::LdIndexed(Register::R0, Register::R1, 16))
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
            Some(Instruction::StIndexed(Register::R2, -1, Register::R3))
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
            Some(Instruction::Lea(Register::R4, Register::R5, 32))
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
            Some(Instruction::LdPostInc(Register::R6, Register::R7))
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
            Some(Instruction::StPostInc(Register::R0, Register::R1))
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
            Some(Instruction::LdPreDec(Register::R2, Register::R3))
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
            Some(Instruction::StPreDec(Register::R4, Register::R5))
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
            Some(Instruction::LdBPostInc(Register::R6, Register::R7))
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
            Some(Instruction::StBPostInc(Register::R0, Register::R1))
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
            Some(Instruction::LdBPreDec(Register::R2, Register::R3))
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
            Some(Instruction::StBPreDec(Register::R4, Register::R5))
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
            Some(Instruction::BitReg(Register::R1, 7))
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
            Some(Instruction::SetAbs(Operand::AbsAddr(0x1234), 0))
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
            Some(Instruction::ResIndirect(Register::R2, 3))
        );
    }

    #[test]
    fn test_parse_org_directive_hex() {
        let source = ".org 0x3000\n";
        let result = parse_source(source);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0].directive,
            Some(Directive::Org(Operand::Immediate(0x3000)))
        );
    }

    #[test]
    fn test_parse_org_directive_dec() {
        let source = ".org 500\n";
        let result = parse_source(source);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0].directive,
            Some(Directive::Org(Operand::Immediate(500)))
        );
    }

    // #[test]
    // fn test_parse_org_directive_incorrect_operand() {
    //     let source = ".org R0\n";
    //     let result = parse_source(source);
    //     assert!(result.is_err_and(|e| {
    //         e == AssemblyError::StructuralError {
    //             line: 1,
    //             reason: ".org argument must be an immediate value.".to_string(),
    //         }
    //     }));
    // }

    // #[test]
    // fn test_parse_org_directive_out_of_bound() {
    //     let source = ".org 0x12345\n";
    //     let result = parse_source(source);
    //     assert!(result.is_err_and(|e| {
    //         e == AssemblyError::StructuralError {
    //             line: 1,
    //             reason: ".org address must be an unsigned 16 bit value (max: 0xFFFF, min: 0x0000)"
    //                 .to_string(),
    //         }
    //     }));
    // }

    #[test]
    fn test_parse_bank_directive_hex() {
        let source = ".bank 0x3\n";
        let result = parse_source(source);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].directive, Some(Directive::Bank(0x3)));
    }

    #[test]
    fn test_parse_bank_directive_dec() {
        let source = ".bank 5\n";
        let result = parse_source(source);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].directive, Some(Directive::Bank(5)));
    }

    // #[test]
    // fn test_parse_bank_directive_incorrect_operand() {
    //     let source = ".bank R0\n";
    //     let result = parse_source(source);
    //     assert!(result.is_err_and(|e| {
    //         e == AssemblyError::StructuralError {
    //             line: 1,
    //             reason: ".bank argument must be an immediate value.".to_string(),
    //         }
    //     }));
    // }

    // #[test]
    // fn test_parse_bank_directive_out_of_bound() {
    //     let source = ".bank 300\n";
    //     let result = parse_source(source);
    //     assert!(result.is_err_and(|e| {
    //         e == AssemblyError::StructuralError {
    //             line: 1,
    //             reason: ".bank number must be an unsigned value between 0 and 256".to_string(),
    //         }
    //     }));
    // }

    #[test]
    fn test_parse_call_far() {
        let source = "CALL.far LABEL\n";
        let result = parse_source(source);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0].instruction,
            Some(Instruction::CallFar("LABEL".to_string()))
        );
    }

    // #[test]
    // fn test_parse_call_far_invalid_operand() {
    //     let source = "CALL.far 300\n";
    //     let result = parse_source(source);
    //     assert!(result.is_err_and(|e| {
    //         e == AssemblyError::StructuralError {
    //             line: 1,
    //             reason: "Operand to a CALL.far instruction must be a label.".to_string(),
    //         }
    //     }));
    // }

    #[test]
    fn test_parse_call_far_via() {
        let source = "CALL.far LABEL via TRAMP\n";
        let result = parse_source(source);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0].instruction,
            Some(Instruction::CallFarVia(
                "LABEL".to_string(),
                "TRAMP".to_string()
            ))
        );
    }

    // #[test]
    // fn test_parse_call_far_via_invalid_operand() {
    //     let source = "CALL.far 300 via TRAMP\n";
    //     let result = parse_source(source);
    //     assert!(result.is_err_and(|e| {
    //         e == AssemblyError::StructuralError {
    //             line: 1,
    //             reason: "Operand to a CALL.far instruction must be a label.".to_string(),
    //         }
    //     }));
    // }

    // #[test]
    // fn test_parse_call_far_via_invalid_via_operand() {
    //     let source = "CALL.far LABEL via 200\n";
    //     let result = parse_source(source);
    //     assert!(result.is_err_and(|e| {
    //         e == AssemblyError::StructuralError {
    //             line: 1,
    //             reason: "via operand to a CALL.far instruction must be a label.".to_string(),
    //         }
    //     }));
    // }
}
