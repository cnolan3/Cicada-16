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
    pub fn encode_acc_math(self, base_opcode: u8, rs: &Register) -> Result<Vec<u8>, AssemblyError> {
        Ok(vec![encode_reg_opcode(base_opcode, rs)])
    }

    pub fn encode_acc_imm_math(self, opcode: u8, op: &Operand) -> Result<Vec<u8>, AssemblyError> {
        let value = resolve_label_or_immediate(op, self.symbol_table, self.line_num)?;
        let [low, high] = value.to_le_bytes();
        Ok(vec![opcode, low, high])
    }

    pub fn encode_reg_math(
        self,
        opcode: u8,
        rd: &Register,
        rs: &Register,
    ) -> Result<Vec<u8>, AssemblyError> {
        Ok(vec![opcode, encode_rd_rs_byte(0x00, rd, rs)])
    }

    pub fn encode_reg_imm_math(
        self,
        opcode: u8,
        rd: &Register,
        op: &Operand,
    ) -> Result<Vec<u8>, AssemblyError> {
        let imm = resolve_label_or_immediate(op, self.symbol_table, self.line_num)?;
        let rd_index = encode_register_operand(rd);
        let [low, high] = imm.to_le_bytes();
        Ok(vec![opcode, rd_index, low, high])
    }

    pub fn encode_add_sp(self, offset: &Operand) -> Result<Vec<u8>, AssemblyError> {
        let imm = self.expect_immediate(offset)?;
        Ok(vec![ADD_SP_OPCODE, imm as u8])
    }

    pub fn encode_inc(self, reg: &Register) -> Result<Vec<u8>, AssemblyError> {
        Ok(vec![encode_reg_opcode(INC_BASE_OPCODE, reg)])
    }

    pub fn encode_dec(self, reg: &Register) -> Result<Vec<u8>, AssemblyError> {
        Ok(vec![encode_reg_opcode(DEC_BASE_OPCODE, reg)])
    }

    pub fn encode_acc_byte_math(
        self,
        base_sub_opcode: u8,
        rs: &Register,
    ) -> Result<Vec<u8>, AssemblyError> {
        Ok(vec![FD_PREFIX, encode_reg_opcode(base_sub_opcode, rs)])
    }
}
