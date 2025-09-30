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
    pub fn encode_push_reg(self, reg: &Register) -> Result<Vec<u8>, AssemblyError> {
        Ok(encode_push_r_data(reg))
    }

    pub fn encode_push_imm(self, op: &Operand) -> Result<Vec<u8>, AssemblyError> {
        let value = resolve_label_or_immediate(op, self.symbol_table, self.line_num)?;
        let [low, high] = value.to_le_bytes();
        Ok(vec![PUSH_IMM_OPCODE, low, high])
    }

    pub fn encode_pop(self, reg: &Register) -> Result<Vec<u8>, AssemblyError> {
        Ok(encode_pop_r_data(reg))
    }

    pub fn encode_pushf(self) -> Result<Vec<u8>, AssemblyError> {
        Ok(vec![PUSH_F_OPCODE])
    }

    pub fn encode_popf(self) -> Result<Vec<u8>, AssemblyError> {
        Ok(vec![POP_F_OPCODE])
    }
}

// ------- reusable stack encoding functions -------

pub fn encode_push_r_data(rs: &Register) -> Vec<u8> {
    let index = encode_register_operand(rs);
    vec![PUSH_REG_BASE_OPCODE + index]
}

pub fn encode_pop_r_data(rd: &Register) -> Vec<u8> {
    let index = encode_register_operand(rd);
    vec![POP_REG_BASE_OPCODE + index]
}
