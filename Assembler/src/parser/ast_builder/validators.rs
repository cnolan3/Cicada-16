use crate::parser::AstBuilder;
use crate::parser::ast_builder::AssemblyError;
use crate::parser::ast_builder::operand_builders::*;
use crate::parser::ast_builder::utility_functions::*;
use crate::parser::{ConditionCode, Operand, Register};
use anyhow::{Context, Result};

impl<'a> AstBuilder<'a> {
    // Helper to get the next operand
    pub fn pop_operand(&mut self) -> Result<Operand> {
        let pair = self
            .pairs
            .next()
            .ok_or_else(|| AssemblyError::StructuralError {
                line: self.line_number,
                reason: "Expected an operand, but found none.".to_string(),
            })?;
        build_operand(pair)
    }

    pub fn pop_cc(&mut self) -> Result<ConditionCode> {
        let pair = self
            .pairs
            .next()
            .ok_or_else(|| AssemblyError::StructuralError {
                line: self.line_number,
                reason: "Expected a condition code, but found none.".to_string(),
            })?;
        build_condition_code(pair)
    }

    // validation helper
    pub fn expect_register(&mut self) -> Result<Register> {
        let op = self.pop_operand()?;
        if let Operand::Register(r) = op {
            Ok(r)
        } else {
            Err(AssemblyError::StructuralError {
                line: self.line_number,
                reason: "Expected a register operand (R0-R7).".to_string(),
            }
            .into())
        }
    }

    // validate an unsigned byte
    pub fn expect_unsigned_byte(&mut self) -> Result<u8> {
        let op = self.pop_operand()?;

        if let Operand::Immediate(imm) = op {
            check_unsigned_byte(imm, self.line_number)
                .context("Expected an unsigned byte value.")?;
            Ok(imm as u8)
        } else {
            Err(AssemblyError::StructuralError {
                line: self.line_number,
                reason: "Expected an immediate byte value.".to_string(),
            }
            .into())
        }
    }

    // validate a signed byte
    pub fn expect_signed_byte(&mut self) -> Result<i8> {
        let op = self.pop_operand()?;

        if let Operand::Immediate(imm) = op {
            check_signed_byte(imm, self.line_number).context("Expected a signed byte value.")?;
            Ok(imm as i8)
        } else {
            Err(AssemblyError::StructuralError {
                line: self.line_number,
                reason: "Expected an immediate byte value.".to_string(),
            }
            .into())
        }
    }

    // valide bit identifier
    pub fn expect_bit_id(&mut self) -> Result<u8> {
        let op = self.pop_operand()?;

        if let Operand::Immediate(imm) = op {
            check_bit_id(imm, self.line_number).context("Expected a bit ID value.")?;
            Ok(imm as u8)
        } else {
            Err(AssemblyError::StructuralError {
                line: self.line_number,
                reason: "Expected an immediate value.".to_string(),
            }
            .into())
        }
    }

    // validate a label
    pub fn expect_label(&mut self) -> Result<String> {
        let op = self.pop_operand()?;

        match op {
            Operand::Label(label) => Ok(label),
            _ => Err(AssemblyError::StructuralError {
                line: self.line_number,
                reason: "Expected a label.".to_string(),
            }
            .into()),
        }
    }

    // validate an address or label
    pub fn expect_addr_or_label(&mut self) -> Result<Operand> {
        let op = self.pop_operand()?;

        match op {
            Operand::Immediate(addr) => {
                check_unsigned_word(addr, self.line_number)
                    .context("Expected an address value.")?;
                Ok(op)
            }
            Operand::Label(_) => Ok(op),
            _ => Err(AssemblyError::StructuralError {
                line: self.line_number,
                reason: "Expected an address or label.".to_string(),
            }
            .into()),
        }
    }

    // validate a signed byte or label
    pub fn expect_sbyte_or_label(&mut self) -> Result<Operand> {
        let op = self.pop_operand()?;

        match op {
            Operand::Immediate(byte) => {
                check_signed_byte(byte, self.line_number)
                    .context("Expected a signed byte value.")?;
                Ok(op)
            }
            Operand::Label(_) => Ok(op),
            _ => Err(AssemblyError::StructuralError {
                line: self.line_number,
                reason: "Expected an address or label.".to_string(),
            }
            .into()),
        }
    }

    pub fn expect_op_vector(&mut self) -> Result<Vec<Operand>> {
        let line = self.line_number;
        let ops = self.pairs.next().ok_or_else(|| AssemblyError::StructuralError {
            line,
            reason: "Expected a list of operands.".to_string()
        })?.into_inner();

        ops.map(build_operand).collect()
    }
}
