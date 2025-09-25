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

use crate::parser::AstBuilder;
use crate::parser::Instruction;
use crate::parser::ast_builder::constants::*;
use anyhow::{Context, Result};

impl<'a> AstBuilder<'a> {
    // build and check operands for a 2 operand add instruction
    pub fn build_add_2_op(mut self) -> Result<Instruction> {
        let rd = self.expect_register().context(INVALID_DEST_OP_MSG)?;
        let rs = self.expect_register().context(INVALID_SRC_OP_MSG)?;

        Ok(Instruction::AddReg(rd, rs))
    }

    // build and check operands for a add immediate instruction
    pub fn build_addi_2_op(mut self) -> Result<Instruction> {
        let rd = self.expect_register().context(INVALID_DEST_OP_MSG)?;
        let src = self.expect_addr_or_label().context(INVALID_SRC_OP_MSG)?;

        Ok(Instruction::AddIReg(rd, src))
    }

    // build and check operands for a 1 operand add instruction
    pub fn build_add_1_op(mut self) -> Result<Instruction> {
        let r = self.expect_register().context(INVALID_OP_MSG)?;

        Ok(Instruction::AddAcc(r))
    }

    // build and check operands for a ADD SP instruction
    pub fn build_add_sp(mut self) -> Result<Instruction> {
        let val = self.expect_signed_byte().context(INVALID_OP_MSG)?;

        Ok(Instruction::AddSp(val))
    }

    // build and check operands for an add.b instruction
    pub fn build_add_b(mut self) -> Result<Instruction> {
        let r = self.expect_register().context(INVALID_OP_MSG)?;

        Ok(Instruction::AddBAcc(r))
    }

    // build and check operands for an add accumulator immediate instruction
    pub fn build_addi_1_op(mut self) -> Result<Instruction> {
        let src = self.expect_addr_or_label().context(INVALID_OP_MSG)?;

        Ok(Instruction::AddAccI(src))
    }

    // build and check operands for a 2 operand sub instruction
    pub fn build_sub_2_op(mut self) -> Result<Instruction> {
        let rd = self.expect_register().context(INVALID_DEST_OP_MSG)?;
        let rs = self.expect_register().context(INVALID_SRC_OP_MSG)?;

        Ok(Instruction::SubReg(rd, rs))
    }

    // build and check operands for a sub immediate instruction
    pub fn build_subi_2_op(mut self) -> Result<Instruction> {
        let rd = self.expect_register().context(INVALID_DEST_OP_MSG)?;
        let src = self.expect_addr_or_label().context(INVALID_SRC_OP_MSG)?;

        Ok(Instruction::SubIReg(rd, src))
    }

    // build and check operands for a 1 operand sub instruction
    pub fn build_sub_1_op(mut self) -> Result<Instruction> {
        let r = self.expect_register().context(INVALID_OP_MSG)?;

        Ok(Instruction::SubAcc(r))
    }

    // build and check operands for an sub.b instruction
    pub fn build_sub_b(mut self) -> Result<Instruction> {
        let r = self.expect_register().context(INVALID_OP_MSG)?;

        Ok(Instruction::SubBAcc(r))
    }

    // build and check operands for a sub accumulator immediate instruction
    pub fn build_subi_1_op(mut self) -> Result<Instruction> {
        let src = self.expect_addr_or_label().context(INVALID_OP_MSG)?;

        Ok(Instruction::SubAccI(src))
    }

    // build and check operands for a 2 operand and instruction
    pub fn build_and_2_op(mut self) -> Result<Instruction> {
        let rd = self.expect_register().context(INVALID_DEST_OP_MSG)?;
        let rs = self.expect_register().context(INVALID_SRC_OP_MSG)?;

        Ok(Instruction::AndReg(rd, rs))
    }

    // build and check operands for a and immediate instruction
    pub fn build_andi_2_op(mut self) -> Result<Instruction> {
        let rd = self.expect_register().context(INVALID_DEST_OP_MSG)?;
        let src = self.expect_addr_or_label().context(INVALID_SRC_OP_MSG)?;

        Ok(Instruction::AndIReg(rd, src))
    }

    // build and check operands for a 1 operand and instruction
    pub fn build_and_1_op(mut self) -> Result<Instruction> {
        let r = self.expect_register().context(INVALID_OP_MSG)?;

        Ok(Instruction::AndAcc(r))
    }

    // build and check operands for an and.b instruction
    pub fn build_and_b(mut self) -> Result<Instruction> {
        let r = self.expect_register().context(INVALID_OP_MSG)?;

        Ok(Instruction::AndBAcc(r))
    }

    // build and check operands for a and accumulator immediate instruction
    pub fn build_andi_1_op(mut self) -> Result<Instruction> {
        let src = self.expect_addr_or_label().context(INVALID_OP_MSG)?;

        Ok(Instruction::AndAccI(src))
    }

    // build and check operands for a 2 operand or instruction
    pub fn build_or_2_op(mut self) -> Result<Instruction> {
        let rd = self.expect_register().context(INVALID_DEST_OP_MSG)?;
        let rs = self.expect_register().context(INVALID_SRC_OP_MSG)?;

        Ok(Instruction::OrReg(rd, rs))
    }

    // build and check operands for a or immediate instruction
    pub fn build_ori_2_op(mut self) -> Result<Instruction> {
        let rd = self.expect_register().context(INVALID_DEST_OP_MSG)?;
        let src = self.expect_addr_or_label().context(INVALID_SRC_OP_MSG)?;

        Ok(Instruction::OrIReg(rd, src))
    }

    // build and check operands for a 1 operand or instruction
    pub fn build_or_1_op(mut self) -> Result<Instruction> {
        let r = self.expect_register().context(INVALID_OP_MSG)?;

        Ok(Instruction::OrAcc(r))
    }

    // build and check operands for an or.b instruction
    pub fn build_or_b(mut self) -> Result<Instruction> {
        let r = self.expect_register().context(INVALID_OP_MSG)?;

        Ok(Instruction::OrBAcc(r))
    }

    // build and check operands for an or accumulator immediate instruction
    pub fn build_ori_1_op(mut self) -> Result<Instruction> {
        let src = self.expect_addr_or_label().context(INVALID_OP_MSG)?;

        Ok(Instruction::OrAccI(src))
    }

    // build and check operands for a 2 operand xor instruction
    pub fn build_xor_2_op(mut self) -> Result<Instruction> {
        let rd = self.expect_register().context(INVALID_DEST_OP_MSG)?;
        let rs = self.expect_register().context(INVALID_SRC_OP_MSG)?;

        Ok(Instruction::XorReg(rd, rs))
    }

    // build and check operands for a xor immediate instruction
    pub fn build_xori_2_op(mut self) -> Result<Instruction> {
        let rd = self.expect_register().context(INVALID_DEST_OP_MSG)?;
        let src = self.expect_addr_or_label().context(INVALID_SRC_OP_MSG)?;

        Ok(Instruction::XorIReg(rd, src))
    }

    // build and check operands for a 1 operand xor instruction
    pub fn build_xor_1_op(mut self) -> Result<Instruction> {
        let r = self.expect_register().context(INVALID_OP_MSG)?;

        Ok(Instruction::XorAcc(r))
    }

    // build and check operands for an xor.b instruction
    pub fn build_xor_b(mut self) -> Result<Instruction> {
        let r = self.expect_register().context(INVALID_OP_MSG)?;

        Ok(Instruction::XorBAcc(r))
    }

    // build and check operands for a xor accumulator immediate instruction
    pub fn build_xori_1_op(mut self) -> Result<Instruction> {
        let src = self.expect_addr_or_label().context(INVALID_OP_MSG)?;

        Ok(Instruction::XorAccI(src))
    }

    // build and check operands for a 2 operand cmp instruction
    pub fn build_cmp_2_op(mut self) -> Result<Instruction> {
        let rd = self.expect_register().context(INVALID_DEST_OP_MSG)?;
        let rs = self.expect_register().context(INVALID_SRC_OP_MSG)?;

        Ok(Instruction::CmpReg(rd, rs))
    }

    // build and check operands for a cmp immediate instruction
    pub fn build_cmpi_2_op(mut self) -> Result<Instruction> {
        let rd = self.expect_register().context(INVALID_DEST_OP_MSG)?;
        let src = self.expect_addr_or_label().context(INVALID_SRC_OP_MSG)?;

        Ok(Instruction::CmpIReg(rd, src))
    }

    // build and check operands for a 1 operand cmp instruction
    pub fn build_cmp_1_op(mut self) -> Result<Instruction> {
        let r = self.expect_register().context(INVALID_OP_MSG)?;

        Ok(Instruction::CmpAcc(r))
    }

    // build and check operands for an cmp.b instruction
    pub fn build_cmp_b(mut self) -> Result<Instruction> {
        let r = self.expect_register().context(INVALID_OP_MSG)?;

        Ok(Instruction::CmpBAcc(r))
    }

    // build and check operands for a cmp accumulator immediate instruction
    pub fn build_cmpi_1_op(mut self) -> Result<Instruction> {
        let src = self.expect_addr_or_label().context(INVALID_OP_MSG)?;

        Ok(Instruction::CmpAccI(src))
    }

    // build and check operands for a 2 operand adc instruction
    pub fn build_adc_2_op(mut self) -> Result<Instruction> {
        let rd = self.expect_register().context(INVALID_DEST_OP_MSG)?;
        let rs = self.expect_register().context(INVALID_SRC_OP_MSG)?;

        Ok(Instruction::AdcReg(rd, rs))
    }

    // build and check operands for an ADC accumulator immediate instruction
    pub fn build_adci_1_op(mut self) -> Result<Instruction> {
        let src = self.expect_addr_or_label().context(INVALID_OP_MSG)?;

        Ok(Instruction::AdcAccI(src))
    }

    // build and check operands for a 2 operand sbc instruction
    pub fn build_sbc_2_op(mut self) -> Result<Instruction> {
        let rd = self.expect_register().context(INVALID_DEST_OP_MSG)?;
        let rs = self.expect_register().context(INVALID_SRC_OP_MSG)?;

        Ok(Instruction::SbcReg(rd, rs))
    }

    // build and check operands for an SBC accumulator immediate instruction
    pub fn build_sbci_1_op(mut self) -> Result<Instruction> {
        let src = self.expect_addr_or_label().context(INVALID_OP_MSG)?;

        Ok(Instruction::SbcAccI(src))
    }

    // build and check operands for a 1 operand inc instruction
    pub fn build_inc(mut self) -> Result<Instruction> {
        let r = self.expect_register().context(INVALID_OP_MSG)?;

        Ok(Instruction::Inc(r))
    }

    // build and check operands for a 1 operand dec instruction
    pub fn build_dec(mut self) -> Result<Instruction> {
        let r = self.expect_register().context(INVALID_OP_MSG)?;

        Ok(Instruction::Dec(r))
    }
}
