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

use crate::assembler::encoder::constants::*;
use crate::assembler::encoder::utility_functions::*;
use crate::assembler::symbol_table::*;
use crate::ast::Register;

pub fn encode_far(call_symbol: &Symbol, syslib_index: u8) -> Vec<u8> {
    let mut bytecode = Vec::new();
    bytecode.extend(encode_ldi(&Register::R4, call_symbol.bank as u16));
    bytecode.extend(encode_ldi(
        &Register::R5,
        call_symbol.logical_address as u16,
    ));
    bytecode.extend(encode_syscall(syslib_index));
    bytecode
}

pub fn encode_far_via(call_symbol: &Symbol, via_symbol: &Symbol) -> Vec<u8> {
    let mut bytecode = Vec::new();
    bytecode.extend(encode_ldi(&Register::R4, call_symbol.bank as u16));
    bytecode.extend(encode_ldi(
        &Register::R5,
        call_symbol.logical_address as u16,
    ));
    bytecode.extend(encode_call_immediate(via_symbol.logical_address as u16));
    bytecode
}

pub fn encode_push_r(rs: &Register) -> Vec<u8> {
    let index = encode_register_operand(rs);
    vec![PUSH_REG_BASE_OPCODE + index]
}

pub fn encode_pop_r(rd: &Register) -> Vec<u8> {
    let index = encode_register_operand(rd);
    vec![POP_REG_BASE_OPCODE + index]
}

pub fn encode_ldi(rd: &Register, val: u16) -> Vec<u8> {
    let opcode = encode_reg_opcode(LDI_BASE_OPCODE, rd);
    let [low, high] = val.to_le_bytes();
    vec![opcode, low, high]
}

pub fn encode_syscall(index: u8) -> Vec<u8> {
    vec![SYSCALL_OPCODE, index]
}

pub fn encode_call_immediate(logical_addr: u16) -> Vec<u8> {
    let [low, high] = logical_addr.to_le_bytes();
    vec![CALL_IMM_OPCODE, low, high]
}
