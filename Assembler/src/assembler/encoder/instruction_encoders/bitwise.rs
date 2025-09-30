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

use crate::assembler::encoder::Encoder;
use crate::assembler::encoder::constants::*;
use crate::assembler::encoder::utility_functions::*;
use crate::ast::{Operand, Register};
use crate::errors::AssemblyError;

impl<'a> Encoder<'a> {
    pub fn encode_shift(
        self,
        base_sub_opcode: u8,
        reg: &Register,
    ) -> Result<Vec<u8>, AssemblyError> {
        Ok(vec![FD_PREFIX, encode_reg_opcode(base_sub_opcode, reg)])
    }

    pub fn encode_bit_reg(self, r: &Register, b: &Operand) -> Result<Vec<u8>, AssemblyError> {
        let reg = encode_register_operand(r);
        let imm = self.expect_immediate(b)?;
        let sub_opcode: u8 = BIT_REG_BASE_SUB_OPCODE + imm as u8;
        Ok(vec![FD_PREFIX, sub_opcode, reg])
    }

    pub fn encode_set_reg(self, r: &Register, b: &Operand) -> Result<Vec<u8>, AssemblyError> {
        let reg = encode_register_operand(r);
        let imm = self.expect_immediate(b)?;
        let sub_opcode: u8 = SET_REG_BASE_SUB_OPCODE + imm as u8;
        Ok(vec![FD_PREFIX, sub_opcode, reg])
    }

    pub fn encode_res_reg(self, r: &Register, b: &Operand) -> Result<Vec<u8>, AssemblyError> {
        let reg = encode_register_operand(r);
        let imm = self.expect_immediate(b)?;
        let sub_opcode: u8 = RES_REG_BASE_SUB_OPCODE + imm as u8;
        Ok(vec![FD_PREFIX, sub_opcode, reg])
    }

    pub fn encode_bit_abs(&self, op: &Operand, b: &Operand) -> Result<Vec<u8>, AssemblyError> {
        let imm = self.expect_immediate(b)?;
        let sub_opcode: u8 = BIT_ABS_BASE_SUB_OPCODE + imm as u8;
        let addr =
            resolve_label_or_immediate(op, self.symbol_table, self.line_num, self.current_bank)?;
        let [low, high] = addr.to_le_bytes();
        Ok(vec![FD_PREFIX, sub_opcode, low, high])
    }

    pub fn encode_set_abs(&self, op: &Operand, b: &Operand) -> Result<Vec<u8>, AssemblyError> {
        let imm = self.expect_immediate(b)?;
        let sub_opcode: u8 = SET_ABS_BASE_SUB_OPCODE + imm as u8;
        let addr =
            resolve_label_or_immediate(op, self.symbol_table, self.line_num, self.current_bank)?;
        let [low, high] = addr.to_le_bytes();
        Ok(vec![FD_PREFIX, sub_opcode, low, high])
    }

    pub fn encode_res_abs(self, op: &Operand, b: &Operand) -> Result<Vec<u8>, AssemblyError> {
        let imm = self.expect_immediate(b)?;
        let sub_opcode: u8 = RES_ABS_BASE_SUB_OPCODE + imm as u8;
        let addr =
            resolve_label_or_immediate(op, self.symbol_table, self.line_num, self.current_bank)?;
        let [low, high] = addr.to_le_bytes();
        Ok(vec![FD_PREFIX, sub_opcode, low, high])
    }

    pub fn encode_bit_indirect(self, r: &Register, b: &Operand) -> Result<Vec<u8>, AssemblyError> {
        let reg = encode_register_operand(r);
        let imm = self.expect_immediate(b)?;
        let sub_opcode: u8 = BIT_INDIR_BASE_SUB_OPCODE + imm as u8;
        Ok(vec![FD_PREFIX, sub_opcode, reg])
    }

    pub fn encode_set_indirect(self, r: &Register, b: &Operand) -> Result<Vec<u8>, AssemblyError> {
        let reg = encode_register_operand(r);
        let imm = self.expect_immediate(b)?;
        let sub_opcode: u8 = SET_INDIR_BASE_SUB_OPCODE + imm as u8;
        Ok(vec![FD_PREFIX, sub_opcode, reg])
    }

    pub fn encode_res_indirect(self, r: &Register, b: &Operand) -> Result<Vec<u8>, AssemblyError> {
        let reg = encode_register_operand(r);
        let imm = self.expect_immediate(b)?;
        let sub_opcode: u8 = RES_INDIR_BASE_SUB_OPCODE + imm as u8;
        Ok(vec![FD_PREFIX, sub_opcode, reg])
    }
}
