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
    pub fn encode_ld_reg(self, rd: &Register, rs: &Register) -> Result<Vec<u8>, AssemblyError> {
        Ok(vec![encode_rd_rs_byte(LD_REG_REG_BASE_OPCODE, rd, rs)])
    }

    pub fn encode_ldi(self, rd: &Register, op: &Operand) -> Result<Vec<u8>, AssemblyError> {
        let addr = resolve_label_or_immediate(op, self.symbol_table, self.line_num)?;
        Ok(encode_ldi_data(rd, addr))
    }

    pub fn encode_ld_indirect(
        self,
        rd: &Register,
        rs: &Register,
    ) -> Result<Vec<u8>, AssemblyError> {
        Ok(vec![
            FE_PREFIX,
            encode_rd_rs_byte(LD_INDIR_BASE_SUB_OPCODE, rd, rs),
        ])
    }

    pub fn encode_ld_abs(self, rd: &Register, op: &Operand) -> Result<Vec<u8>, AssemblyError> {
        let opcode = encode_reg_opcode(LD_ABS_BASE_OPCODE, rd);
        let addr = resolve_label_or_immediate(op, self.symbol_table, self.line_num)?;
        let [low, high] = addr.to_le_bytes();
        Ok(vec![opcode, low, high])
    }

    pub fn encode_ld_indexed(
        self,
        rd: &Register,
        rs: &Register,
        offset: &Operand,
    ) -> Result<Vec<u8>, AssemblyError> {
        let sub_opcode = encode_rd_rs_byte(LD_INDEX_BASE_SUB_OPCODE, rd, rs);
        let imm = self.expect_immediate(offset)?;
        Ok(vec![FF_PREFIX, sub_opcode, imm as u8])
    }

    pub fn encode_ld_pre_dec(self, rd: &Register, rs: &Register) -> Result<Vec<u8>, AssemblyError> {
        let sub_opcode = encode_reg_opcode(LD_PRE_DEC_BASE_SUB_OPCODE, rs);
        let dest = encode_register_operand(rd);
        Ok(vec![FF_PREFIX, sub_opcode, dest])
    }

    pub fn encode_ld_post_inc(
        self,
        rd: &Register,
        rs: &Register,
    ) -> Result<Vec<u8>, AssemblyError> {
        let sub_opcode = encode_reg_opcode(LD_POST_INC_BASE_SUB_OPCODE, rs);
        let dest = encode_register_operand(rd);
        Ok(vec![FF_PREFIX, sub_opcode, dest])
    }

    pub fn encode_st_indirect(
        self,
        rd: &Register,
        rs: &Register,
    ) -> Result<Vec<u8>, AssemblyError> {
        let sub_opcode = encode_rd_rs_byte(ST_INDIR_BASE_SUB_OPCODE, rd, rs);
        Ok(vec![FE_PREFIX, sub_opcode])
    }

    pub fn encode_st_abs(self, op: &Operand, rs: &Register) -> Result<Vec<u8>, AssemblyError> {
        let opcode = encode_reg_opcode(ST_ABS_BASE_OPCODE, rs);
        let addr = resolve_label_or_immediate(op, self.symbol_table, self.line_num)?;
        let [low, high] = addr.to_le_bytes();
        Ok(vec![opcode, low, high])
    }

    pub fn encode_st_indexed(
        self,
        rd: &Register,
        offset: &Operand,
        rs: &Register,
    ) -> Result<Vec<u8>, AssemblyError> {
        let sub_opcode = encode_rd_rs_byte(ST_INDEX_BASE_SUB_OPCODE, rd, rs);
        let imm = self.expect_immediate(offset)?;
        Ok(vec![FF_PREFIX, sub_opcode, imm as u8])
    }

    pub fn encode_st_pre_dec(self, rd: &Register, rs: &Register) -> Result<Vec<u8>, AssemblyError> {
        let sub_opcode = encode_reg_opcode(ST_PRE_DEC_BASE_SUB_OPCODE, rs);
        let dest = encode_register_operand(rd);
        Ok(vec![FF_PREFIX, sub_opcode, dest])
    }

    pub fn encode_st_post_inc(
        self,
        rd: &Register,
        rs: &Register,
    ) -> Result<Vec<u8>, AssemblyError> {
        let sub_opcode = encode_reg_opcode(ST_POST_INC_BASE_SUB_OPCODE, rs);
        let dest = encode_register_operand(rd);
        Ok(vec![FF_PREFIX, sub_opcode, dest])
    }

    pub fn encode_ldib(self, rd: &Register, imm8: &Operand) -> Result<Vec<u8>, AssemblyError> {
        let sub_opcode = encode_reg_opcode(LDIB_BASE_SUB_OPCODE, rd);
        let imm = self.expect_immediate(imm8)?;
        Ok(vec![FD_PREFIX, sub_opcode, imm as u8])
    }

    pub fn encode_ldb_indirect(
        self,
        rd: &Register,
        rs: &Register,
    ) -> Result<Vec<u8>, AssemblyError> {
        let sub_opcode = encode_rd_rs_byte(LDB_INDIR_BASE_SUB_OPCODE, rd, rs);
        Ok(vec![FE_PREFIX, sub_opcode])
    }

    pub fn encode_ldb_pre_dec(
        self,
        rd: &Register,
        rs: &Register,
    ) -> Result<Vec<u8>, AssemblyError> {
        let sub_opcode = encode_reg_opcode(LDB_PRE_DEC_BASE_SUB_OPCODE, rs);
        let dest = encode_register_operand(rd);
        Ok(vec![FF_PREFIX, sub_opcode, dest])
    }

    pub fn encode_ldb_post_inc(
        self,
        rd: &Register,
        rs: &Register,
    ) -> Result<Vec<u8>, AssemblyError> {
        let sub_opcode = encode_reg_opcode(LDB_POST_INC_BASE_SUB_OPCODE, rs);
        let dest = encode_register_operand(rd);
        Ok(vec![FF_PREFIX, sub_opcode, dest])
    }

    pub fn encode_ldb_abs(self, rd: &Register, op: &Operand) -> Result<Vec<u8>, AssemblyError> {
        let sub_opcode = encode_reg_opcode(LDB_ABS_BASE_SUB_OPCODE, rd);
        let addr = resolve_label_or_immediate(op, self.symbol_table, self.line_num)?;
        let [low, high] = addr.to_le_bytes();
        Ok(vec![FD_PREFIX, sub_opcode, low, high])
    }

    pub fn encode_stb_indirect(
        self,
        rd: &Register,
        rs: &Register,
    ) -> Result<Vec<u8>, AssemblyError> {
        let sub_opcode = encode_rd_rs_byte(STB_INDIR_BASE_SUB_OPCODE, rd, rs);
        Ok(vec![FE_PREFIX, sub_opcode])
    }

    pub fn encode_stb_pre_dec(
        self,
        rd: &Register,
        rs: &Register,
    ) -> Result<Vec<u8>, AssemblyError> {
        let sub_opcode = encode_reg_opcode(STB_PRE_DEC_BASE_SUB_OPCODE, rs);
        let dest = encode_register_operand(rd);
        Ok(vec![FF_PREFIX, sub_opcode, dest])
    }

    pub fn encode_stb_post_inc(
        self,
        rd: &Register,
        rs: &Register,
    ) -> Result<Vec<u8>, AssemblyError> {
        let sub_opcode = encode_reg_opcode(STB_POST_INC_BASE_SUB_OPCODE, rs);
        let dest = encode_register_operand(rd);
        Ok(vec![FF_PREFIX, sub_opcode, dest])
    }

    pub fn encode_stb_abs(self, op: &Operand, rs: &Register) -> Result<Vec<u8>, AssemblyError> {
        let sub_opcode = encode_reg_opcode(STB_ABS_BASE_SUB_OPCODE, rs);
        let addr = resolve_label_or_immediate(op, self.symbol_table, self.line_num)?;
        let [low, high] = addr.to_le_bytes();
        Ok(vec![FD_PREFIX, sub_opcode, low, high])
    }

    pub fn encode_lea(
        self,
        rd: &Register,
        rs: &Register,
        offset: &Operand,
    ) -> Result<Vec<u8>, AssemblyError> {
        let sub_opcode = encode_rd_rs_byte(LEA_BASE_SUB_OPCODE, rd, rs);
        let imm = self.expect_immediate(offset)?;
        Ok(vec![FF_PREFIX, sub_opcode, imm as u8])
    }
}

// ------- reusable load/store encoding functions -------

pub fn encode_ldi_data(rd: &Register, val: u16) -> Vec<u8> {
    let opcode = encode_reg_opcode(LDI_BASE_OPCODE, rd);
    let [low, high] = val.to_le_bytes();
    vec![opcode, low, high]
}
