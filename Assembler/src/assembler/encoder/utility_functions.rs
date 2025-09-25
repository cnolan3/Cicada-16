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

use crate::ast::{ConditionCode, Register};

// helper function to encode a register operand
pub fn encode_register_operand(reg: &Register) -> u8 {
    match reg {
        Register::R0 => 0,
        Register::R1 => 1,
        Register::R2 => 2,
        Register::R3 => 3,
        Register::R4 => 4,
        Register::R5 => 5,
        Register::R6 => 6,
        Register::R7 => 7,
    }
}

// help function to encode condition code opcode
pub fn encode_condition_code_opcode(base_opcode: u8, cc: &ConditionCode) -> u8 {
    let cc_offset = match cc {
        ConditionCode::V => 0,
        ConditionCode::Nv => 1,
        ConditionCode::N => 2,
        ConditionCode::Nn => 3,
        ConditionCode::C => 4,
        ConditionCode::Nc => 5,
        ConditionCode::Z => 6,
        ConditionCode::Nz => 7,
    };

    base_opcode + cc_offset
}

pub fn encode_rd_rs_byte(base_val: u8, rd: &Register, rs: &Register) -> u8 {
    let rd_index = encode_register_operand(rd);
    let rs_index = encode_register_operand(rs);
    base_val | ((rd_index & 0x07) << 3) | (rs_index & 0x07)
}

pub fn encode_reg_opcode(base_opcode: u8, r: &Register) -> u8 {
    base_opcode + encode_register_operand(r)
}
