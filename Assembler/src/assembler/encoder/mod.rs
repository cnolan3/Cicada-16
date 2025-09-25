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

mod components;
mod constants;
mod utility_functions;

use crate::assembler::encoder::constants::*;
use crate::assembler::encoder::utility_functions::*;
use crate::assembler::symbol_table::*;
use crate::ast::{Instruction, Operand};
use crate::errors::AssemblyError;
use components::*;

/// Helper function to determine instruction size in bytes during Pass 1.
pub fn calculate_instruction_size(instruction: &Instruction) -> u32 {
    match instruction {
        Instruction::Nop
        | Instruction::Halt
        | Instruction::Ret
        | Instruction::Ccf
        | Instruction::Scf
        | Instruction::Rcf
        | Instruction::Enter
        | Instruction::Leave
        | Instruction::Reti
        | Instruction::Ei
        | Instruction::Di => 1,
        // 16 bit Load/Store instructions
        Instruction::LdReg(_, _) => 1,
        Instruction::Ldi(_, _) => 3,
        Instruction::LdIndirect(_, _) => 2,
        Instruction::LdAbs(_, _) => 3,
        Instruction::LdIndexed(_, _, _) => 3,
        Instruction::LdPreDec(_, _) => 3,
        Instruction::LdPostInc(_, _) => 3,
        Instruction::StIndirect(_, _) => 2,
        Instruction::StAbs(_, _) => 3,
        Instruction::StIndexed(_, _, _) => 3,
        Instruction::StPreDec(_, _) => 3,
        Instruction::StPostInc(_, _) => 3,
        // 8 bit Load/Store instructions
        Instruction::LdiB(_, _) => 3,
        Instruction::LdBIndirect(_, _) => 2,
        Instruction::LdBPreDec(_, _) => 3,
        Instruction::LdBPostInc(_, _) => 3,
        Instruction::StBIndirect(_, _) => 2,
        Instruction::StBPreDec(_, _) => 3,
        Instruction::StBPostInc(_, _) => 3,
        // LEA
        Instruction::Lea(_, _, _) => 3,
        // Stack Operations
        Instruction::Push(_) => 1,
        Instruction::PushI(_) => 3,
        Instruction::Pop(_) => 1,
        Instruction::PushF => 1,
        Instruction::PopF => 1,
        // 16 bit accumulator arithmetic
        Instruction::AddAcc(_) => 1,
        Instruction::SubAcc(_) => 1,
        Instruction::AndAcc(_) => 1,
        Instruction::XorAcc(_) => 1,
        Instruction::OrAcc(_) => 1,
        Instruction::CmpAcc(_) => 1,
        Instruction::NegAcc => 1,
        Instruction::NotAcc => 1,
        Instruction::SwapAcc => 1,
        // 16 bit accumulator immediate arithmetic
        Instruction::AddAccI(_) => 3,
        Instruction::SubAccI(_) => 3,
        Instruction::AndAccI(_) => 3,
        Instruction::OrAccI(_) => 3,
        Instruction::XorAccI(_) => 3,
        Instruction::CmpAccI(_) => 3,
        Instruction::AdcAccI(_) => 3,
        Instruction::SbcAccI(_) => 3,
        // 16 bit register-to-register arithmetic
        Instruction::AddReg(_, _) => 2,
        Instruction::SubReg(_, _) => 2,
        Instruction::AndReg(_, _) => 2,
        Instruction::OrReg(_, _) => 2,
        Instruction::XorReg(_, _) => 2,
        Instruction::CmpReg(_, _) => 2,
        Instruction::AdcReg(_, _) => 2,
        Instruction::SbcReg(_, _) => 2,
        // 16 bit Immediate-to-register arithmetic
        Instruction::AddIReg(_, _) => 4,
        Instruction::SubIReg(_, _) => 4,
        Instruction::AndIReg(_, _) => 4,
        Instruction::OrIReg(_, _) => 4,
        Instruction::XorIReg(_, _) => 4,
        Instruction::CmpIReg(_, _) => 4,
        Instruction::AddSp(_) => 2,
        Instruction::Inc(_) => 1,
        Instruction::Dec(_) => 1,
        // 8 bit accumulator arithmetic
        Instruction::AddBAcc(_) => 2,
        Instruction::SubBAcc(_) => 2,
        Instruction::AndBAcc(_) => 2,
        Instruction::OrBAcc(_) => 2,
        Instruction::XorBAcc(_) => 2,
        Instruction::CmpBAcc(_) => 2,
        // bit manipulation
        Instruction::Sra(_) => 2,
        Instruction::Shl(_) => 2,
        Instruction::Shr(_) => 2,
        Instruction::Rol(_) => 2,
        Instruction::Ror(_) => 2,
        Instruction::BitReg(_, _) => 3,
        Instruction::SetReg(_, _) => 3,
        Instruction::ResReg(_, _) => 3,
        Instruction::BitAbs(_, _) => 4,
        Instruction::SetAbs(_, _) => 4,
        Instruction::ResAbs(_, _) => 4,
        Instruction::BitIndirect(_, _) => 3,
        Instruction::SetIndirect(_, _) => 3,
        Instruction::ResIndirect(_, _) => 3,
        // Control Flow
        Instruction::JmpI(_) => 3,
        Instruction::JmpIndirect(_) => 1,
        Instruction::JrI(_) => 2,
        Instruction::JccI(_, _) => 3,
        Instruction::JrccI(_, _) => 2,
        Instruction::Djnz(_) => 2,
        Instruction::CallI(_) => 3,
        Instruction::CallIndirect(_) => 1,
        Instruction::CallccI(_, _) => 3,
        Instruction::Syscall(_) => 2,
        Instruction::CallFar(_) => 8,
        Instruction::CallFarVia(_, _) => 9,
        Instruction::JmpFar(_) => 8,
        Instruction::JmpFarVia(_, _) => 9,
    }
}

/// Resolves an operand that can be a label or an immediate value into a 16-bit address.
fn resolve_label_or_immediate(
    op: &Operand,
    symbol_table: &SymbolTable,
    line_num: usize,
    current_bank: &u32,
) -> Result<u16, AssemblyError> {
    match op {
        Operand::Immediate(value) => Ok(*value as u16),
        Operand::Label(label_name) => {
            let target_symbol =
                get_and_check_symbol(symbol_table, label_name, line_num, current_bank)?;
            Ok(target_symbol.logical_address as u16)
        }
        _ => Err(AssemblyError::SemanticError {
            line: line_num,
            reason: "Expected an immediate value or a label.".to_string(),
        }),
    }
}

/// Helper function to translate a single instruction into bytes during Pass 2.
pub fn encode_instruction(
    instruction: &Instruction,
    symbol_table: &SymbolTable,
    current_address: &u32,
    current_bank: &u32,
    line_num: usize,
) -> Result<Vec<u8>, AssemblyError> {
    match instruction {
        // no op (0x00)
        Instruction::Nop => Ok(vec![NOP_OPCODE]),
        // Halt
        Instruction::Halt => Ok(vec![HALT_OPCODE]),
        // Ccf
        Instruction::Ccf => Ok(vec![CCF_OPCODE]),
        // Scf
        Instruction::Scf => Ok(vec![SCF_OPCODE]),
        // Rcf
        Instruction::Rcf => Ok(vec![RCF_OPCODE]),
        Instruction::Enter => Ok(vec![ENTER_OPCODE]),
        Instruction::Leave => Ok(vec![LEAVE_OPCODE]),
        Instruction::Ret => Ok(vec![RET_OPCODE]),
        Instruction::Reti => Ok(vec![RETI_OPCODE]),
        Instruction::Ei => Ok(vec![EI_OPCODE]),
        Instruction::Di => Ok(vec![DI_OPCODE]),
        // --------- 16 bit Load/Store instructions ---------
        // register-to-register load
        Instruction::LdReg(rd, rs) => {
            let opcode = encode_rd_rs_byte(LD_REG_REG_BASE_OPCODE, rd, rs);
            Ok(vec![opcode])
        }
        Instruction::Ldi(rd, op) => {
            let value = resolve_label_or_immediate(op, symbol_table, line_num, current_bank)?;
            Ok(encode_ldi(rd, value))
        }
        // indirect-to-register load
        Instruction::LdIndirect(rd, rs) => {
            let sub_opcode = encode_rd_rs_byte(LD_INDIR_BASE_SUB_OPCODE, rd, rs);
            Ok(vec![FE_PREFIX, sub_opcode])
        }
        // absolute-to-register load
        Instruction::LdAbs(rd, Operand::AbsAddr(addr)) => {
            let opcode = encode_reg_opcode(LD_ABS_BASE_OPCODE, rd);
            let [low, high] = (*addr as u16).to_le_bytes();
            Ok(vec![opcode, low, high])
        }
        // absolute-to-register load
        Instruction::LdAbs(rd, Operand::AbsLabel(label_name)) => {
            let opcode = encode_reg_opcode(LD_ABS_BASE_OPCODE, rd);
            let target_symbol =
                get_and_check_symbol(symbol_table, label_name, line_num, current_bank)?;
            let [low, high] = (target_symbol.logical_address as u16).to_le_bytes();
            Ok(vec![opcode, low, high])
        }
        // Indexed address register load
        Instruction::LdIndexed(rd, rs, offset) => {
            let sub_opcode = encode_rd_rs_byte(LD_INDEX_BASE_SUB_OPCODE, rd, rs);
            Ok(vec![FF_PREFIX, sub_opcode, *offset as u8])
        }
        // Load Pre-decrement
        Instruction::LdPreDec(rd, rs) => {
            let sub_opcode = encode_reg_opcode(LD_PRE_DEC_BASE_SUB_OPCODE, rs);
            let dest = encode_register_operand(rd);
            Ok(vec![FF_PREFIX, sub_opcode, dest])
        }
        // Load Post-Increment
        Instruction::LdPostInc(rd, rs) => {
            let sub_opcode = encode_reg_opcode(LD_POST_INC_BASE_SUB_OPCODE, rs);
            let dest = encode_register_operand(rd);
            Ok(vec![FF_PREFIX, sub_opcode, dest])
        }
        Instruction::StIndirect(rd, rs) => {
            let sub_opcode = encode_rd_rs_byte(ST_INDIR_BASE_SUB_OPCODE, rd, rs);
            Ok(vec![FE_PREFIX, sub_opcode])
        }
        Instruction::StAbs(Operand::AbsAddr(addr), rs) => {
            let opcode = encode_reg_opcode(ST_ABS_BASE_OPCODE, rs);
            let [low, high] = (*addr as u16).to_le_bytes();
            Ok(vec![opcode, low, high])
        }
        Instruction::StAbs(Operand::AbsLabel(label_name), rs) => {
            let opcode = encode_reg_opcode(ST_ABS_BASE_OPCODE, rs);
            let target_symbol =
                get_and_check_symbol(symbol_table, label_name, line_num, current_bank)?;
            let [low, high] = (target_symbol.logical_address as u16).to_le_bytes();
            Ok(vec![opcode, low, high])
        }
        Instruction::StIndexed(rd, offset, rs) => {
            let sub_opcode = encode_rd_rs_byte(ST_INDEX_BASE_SUB_OPCODE, rd, rs);
            Ok(vec![FF_PREFIX, sub_opcode, *offset as u8])
        }
        Instruction::StPreDec(rd, rs) => {
            let sub_opcode = encode_reg_opcode(ST_PRE_DEC_BASE_SUB_OPCODE, rs);
            let dest = encode_register_operand(rd);
            Ok(vec![FF_PREFIX, sub_opcode, dest])
        }
        Instruction::StPostInc(rd, rs) => {
            let sub_opcode = encode_reg_opcode(ST_POST_INC_BASE_SUB_OPCODE, rs);
            let dest = encode_register_operand(rd);
            Ok(vec![FF_PREFIX, sub_opcode, dest])
        }

        // --------- 8 bit Load/Store instructions ---------
        Instruction::LdiB(rd, imm8) => {
            let sub_opcode = encode_reg_opcode(LDIB_BASE_SUB_OPCODE, rd);
            Ok(vec![FD_PREFIX, sub_opcode, *imm8 as u8])
        }
        Instruction::LdBIndirect(rd, rs) => {
            let sub_opcode = encode_rd_rs_byte(LDB_INDIR_BASE_SUB_OPCODE, rd, rs);
            Ok(vec![FE_PREFIX, sub_opcode])
        }
        Instruction::LdBPreDec(rd, rs) => {
            let sub_opcode = encode_reg_opcode(LDB_PRE_DEC_BASE_SUB_OPCODE, rs);
            let dest = encode_register_operand(rd);
            Ok(vec![FF_PREFIX, sub_opcode, dest])
        }
        Instruction::LdBPostInc(rd, rs) => {
            let sub_opcode = encode_reg_opcode(LDB_POST_INC_BASE_SUB_OPCODE, rs);
            let dest = encode_register_operand(rd);
            Ok(vec![FF_PREFIX, sub_opcode, dest])
        }
        Instruction::StBIndirect(rd, rs) => {
            let sub_opcode = encode_rd_rs_byte(STB_INDIR_BASE_SUB_OPCODE, rd, rs);
            Ok(vec![FE_PREFIX, sub_opcode])
        }
        Instruction::StBPreDec(rd, rs) => {
            let sub_opcode = encode_reg_opcode(STB_PRE_DEC_BASE_SUB_OPCODE, rs);
            let dest = encode_register_operand(rd);
            Ok(vec![FF_PREFIX, sub_opcode, dest])
        }
        Instruction::StBPostInc(rd, rs) => {
            let sub_opcode = encode_reg_opcode(STB_POST_INC_BASE_SUB_OPCODE, rs);
            let dest = encode_register_operand(rd);
            Ok(vec![FF_PREFIX, sub_opcode, dest])
        }

        // --------- LEA instruction ---------
        Instruction::Lea(rd, rs, offset) => {
            let sub_opcode = encode_rd_rs_byte(LEA_BASE_SUB_OPCODE, rd, rs);
            Ok(vec![FF_PREFIX, sub_opcode, *offset as u8])
        }

        // --------- Stack Operations ---------
        Instruction::Push(reg) => Ok(encode_push_r(reg)),
        Instruction::PushI(op) => {
            let value = resolve_label_or_immediate(op, symbol_table, line_num, current_bank)?;
            let [low, high] = value.to_le_bytes();
            Ok(vec![PUSH_IMM_OPCODE, low, high])
        }
        Instruction::Pop(reg) => Ok(encode_pop_r(reg)),
        Instruction::PushF => Ok(vec![PUSH_F_OPCODE]),
        Instruction::PopF => Ok(vec![POP_F_OPCODE]),

        // --------- 16 bit accumulator arithmetic ---------
        Instruction::AddAcc(rs) => Ok(vec![encode_reg_opcode(ADD_ACC_BASE_OPCODE, rs)]),
        Instruction::SubAcc(rs) => Ok(vec![encode_reg_opcode(SUB_ACC_BASE_OPCODE, rs)]),
        Instruction::AndAcc(rs) => Ok(vec![encode_reg_opcode(AND_ACC_BASE_OPCODE, rs)]),
        Instruction::OrAcc(rs) => Ok(vec![encode_reg_opcode(OR_ACC_BASE_OPCODE, rs)]),
        Instruction::XorAcc(rs) => Ok(vec![encode_reg_opcode(XOR_ACC_BASE_OPCODE, rs)]),
        Instruction::CmpAcc(rs) => Ok(vec![encode_reg_opcode(CMP_ACC_BASE_OPCODE, rs)]),
        Instruction::NegAcc => Ok(vec![NEG_ACC_OPCODE]),
        Instruction::NotAcc => Ok(vec![NOR_ACC_OPCODE]),
        Instruction::SwapAcc => Ok(vec![SWAP_ACC_OPCODE]),

        // --------- 16 bit accumulator immediate arithmetic ---------
        Instruction::AddAccI(op) => {
            let value = resolve_label_or_immediate(op, symbol_table, line_num, current_bank)?;
            let [low, high] = value.to_le_bytes();
            Ok(vec![ADDI_ACC_OPCODE, low, high])
        }
        Instruction::SubAccI(op) => {
            let value = resolve_label_or_immediate(op, symbol_table, line_num, current_bank)?;
            let [low, high] = value.to_le_bytes();
            Ok(vec![SUBI_ACC_OPCODE, low, high])
        }
        Instruction::AndAccI(op) => {
            let value = resolve_label_or_immediate(op, symbol_table, line_num, current_bank)?;
            let [low, high] = value.to_le_bytes();
            Ok(vec![ANDI_ACC_OPCODE, low, high])
        }
        Instruction::OrAccI(op) => {
            let value = resolve_label_or_immediate(op, symbol_table, line_num, current_bank)?;
            let [low, high] = value.to_le_bytes();
            Ok(vec![ORI_ACC_OPCODE, low, high])
        }
        Instruction::XorAccI(op) => {
            let value = resolve_label_or_immediate(op, symbol_table, line_num, current_bank)?;
            let [low, high] = value.to_le_bytes();
            Ok(vec![XORI_ACC_OPCODE, low, high])
        }
        Instruction::CmpAccI(op) => {
            let value = resolve_label_or_immediate(op, symbol_table, line_num, current_bank)?;
            let [low, high] = value.to_le_bytes();
            Ok(vec![CMPI_ACC_OPCODE, low, high])
        }
        Instruction::AdcAccI(op) => {
            let value = resolve_label_or_immediate(op, symbol_table, line_num, current_bank)?;
            let [low, high] = value.to_le_bytes();
            Ok(vec![ADCI_ACC_OPCODE, low, high])
        }
        Instruction::SbcAccI(op) => {
            let value = resolve_label_or_immediate(op, symbol_table, line_num, current_bank)?;
            let [low, high] = value.to_le_bytes();
            Ok(vec![SBCI_ACC_OPCODE, low, high])
        }

        // --------- 16 bit register-to-register arithmetic ---------
        Instruction::AddReg(rd, rs) => Ok(vec![ADD_OPCODE, encode_rd_rs_byte(0x00, rd, rs)]),
        Instruction::SubReg(rd, rs) => Ok(vec![SUB_OPCODE, encode_rd_rs_byte(0x00, rd, rs)]),
        Instruction::AndReg(rd, rs) => Ok(vec![AND_OPCODE, encode_rd_rs_byte(0x00, rd, rs)]),
        Instruction::OrReg(rd, rs) => Ok(vec![OR_OPCODE, encode_rd_rs_byte(0x00, rd, rs)]),
        Instruction::XorReg(rd, rs) => Ok(vec![XOR_OPCODE, encode_rd_rs_byte(0x00, rd, rs)]),
        Instruction::CmpReg(rd, rs) => Ok(vec![CMP_OPCODE, encode_rd_rs_byte(0x00, rd, rs)]),
        Instruction::AdcReg(rd, rs) => Ok(vec![ADC_OPCODE, encode_rd_rs_byte(0x00, rd, rs)]),
        Instruction::SbcReg(rd, rs) => Ok(vec![SBC_OPCODE, encode_rd_rs_byte(0x00, rd, rs)]),

        // --------- 16 bit Immediate-to-register arithmetic ---------
        Instruction::AddIReg(rd, op) => {
            let imm = resolve_label_or_immediate(op, symbol_table, line_num, current_bank)?;
            let rd_index = encode_register_operand(rd);
            let [low, high] = imm.to_le_bytes();
            Ok(vec![ADDI_OPCODE, rd_index, low, high])
        }
        Instruction::SubIReg(rd, op) => {
            let imm = resolve_label_or_immediate(op, symbol_table, line_num, current_bank)?;
            let rd_index = encode_register_operand(rd);
            let [low, high] = imm.to_le_bytes();
            Ok(vec![SUBI_OPCODE, rd_index, low, high])
        }
        Instruction::AndIReg(rd, op) => {
            let imm = resolve_label_or_immediate(op, symbol_table, line_num, current_bank)?;
            let rd_index = encode_register_operand(rd);
            let [low, high] = imm.to_le_bytes();
            Ok(vec![ANDI_OPCODE, rd_index, low, high])
        }
        Instruction::OrIReg(rd, op) => {
            let imm = resolve_label_or_immediate(op, symbol_table, line_num, current_bank)?;
            let rd_index = encode_register_operand(rd);
            let [low, high] = imm.to_le_bytes();
            Ok(vec![ORI_OPCODE, rd_index, low, high])
        }
        Instruction::XorIReg(rd, op) => {
            let imm = resolve_label_or_immediate(op, symbol_table, line_num, current_bank)?;
            let rd_index = encode_register_operand(rd);
            let [low, high] = imm.to_le_bytes();
            Ok(vec![XORI_OPCODE, rd_index, low, high])
        }
        Instruction::CmpIReg(rd, op) => {
            let imm = resolve_label_or_immediate(op, symbol_table, line_num, current_bank)?;
            let rd_index = encode_register_operand(rd);
            let [low, high] = imm.to_le_bytes();
            Ok(vec![CMPI_OPCODE, rd_index, low, high])
        }
        Instruction::AddSp(offset) => Ok(vec![ADD_SP_OPCODE, *offset as u8]),
        Instruction::Inc(reg) => Ok(vec![encode_reg_opcode(INC_BASE_OPCODE, reg)]),
        Instruction::Dec(reg) => Ok(vec![encode_reg_opcode(DEC_BASE_OPCODE, reg)]),

        // --------- 8 bit accumulator arithmetic ---------
        Instruction::AddBAcc(rs) => Ok(vec![FD_PREFIX, encode_reg_opcode(ADDB_BASE_SUB_OPCODE, rs)]),
        Instruction::SubBAcc(rs) => Ok(vec![FD_PREFIX, encode_reg_opcode(SUBB_BASE_SUB_OPCODE, rs)]),
        Instruction::AndBAcc(rs) => Ok(vec![FD_PREFIX, encode_reg_opcode(ANDB_BASE_SUB_OPCODE, rs)]),
        Instruction::OrBAcc(rs) => Ok(vec![FD_PREFIX, encode_reg_opcode(ORB_BASE_SUB_OPCODE, rs)]),
        Instruction::XorBAcc(rs) => Ok(vec![FD_PREFIX, encode_reg_opcode(XORB_BASE_SUB_OPCODE, rs)]),
        Instruction::CmpBAcc(rs) => Ok(vec![FD_PREFIX, encode_reg_opcode(CMPB_BASE_SUB_OPCODE, rs)]),

        // --------- bit manipulation ---------
        Instruction::Sra(reg) => Ok(vec![FD_PREFIX, encode_reg_opcode(SRA_BASE_SUB_OPCODE, reg)]),
        Instruction::Shl(reg) => Ok(vec![FD_PREFIX, encode_reg_opcode(SHL_BASE_SUB_OPCODE, reg)]),
        Instruction::Shr(reg) => Ok(vec![FD_PREFIX, encode_reg_opcode(SHR_BASE_SUB_OPCODE, reg)]),
        Instruction::Rol(reg) => Ok(vec![FD_PREFIX, encode_reg_opcode(ROL_BASE_SUB_OPCODE, reg)]),
        Instruction::Ror(reg) => Ok(vec![FD_PREFIX, encode_reg_opcode(ROR_BASE_SUB_OPCODE, reg)]),
        Instruction::BitReg(r, b) => {
            let reg = encode_register_operand(r);
            let sub_opcode: u8 = BIT_REG_BASE_SUB_OPCODE + b;
            Ok(vec![FD_PREFIX, sub_opcode, reg])
        }
        Instruction::SetReg(r, b) => {
            let reg = encode_register_operand(r);
            let sub_opcode: u8 = SET_REG_BASE_SUB_OPCODE + b;
            Ok(vec![FD_PREFIX, sub_opcode, reg])
        }
        Instruction::ResReg(r, b) => {
            let reg = encode_register_operand(r);
            let sub_opcode: u8 = RES_REG_BASE_SUB_OPCODE + b;
            Ok(vec![FD_PREFIX, sub_opcode, reg])
        }
        Instruction::BitAbs(Operand::AbsAddr(addr), b) => {
            let [low, high] = (*addr as u16).to_le_bytes();
            let sub_opcode: u8 = BIT_ABS_BASE_SUB_OPCODE + b;
            Ok(vec![FD_PREFIX, sub_opcode, low, high])
        }
        Instruction::SetAbs(Operand::AbsAddr(addr), b) => {
            let [low, high] = (*addr as u16).to_le_bytes();
            let sub_opcode: u8 = SET_ABS_BASE_SUB_OPCODE + b;
            Ok(vec![FD_PREFIX, sub_opcode, low, high])
        }
        Instruction::ResAbs(Operand::AbsAddr(addr), b) => {
            let [low, high] = (*addr as u16).to_le_bytes();
            let sub_opcode: u8 = RES_ABS_BASE_SUB_OPCODE + b;
            Ok(vec![FD_PREFIX, sub_opcode, low, high])
        }
        Instruction::BitIndirect(r, b) => {
            let reg = encode_register_operand(r);
            let sub_opcode: u8 = BIT_INDIR_BASE_SUB_OPCODE + b;
            Ok(vec![FD_PREFIX, sub_opcode, reg])
        }
        Instruction::SetIndirect(r, b) => {
            let reg = encode_register_operand(r);
            let sub_opcode: u8 = SET_INDIR_BASE_SUB_OPCODE + b;
            Ok(vec![FD_PREFIX, sub_opcode, reg])
        }
        Instruction::ResIndirect(r, b) => {
            let reg = encode_register_operand(r);
            let sub_opcode: u8 = RES_INDIR_BASE_SUB_OPCODE + b;
            Ok(vec![FD_PREFIX, sub_opcode, reg])
        }

        // --------- Control Flow ---------
        Instruction::JmpI(op) => {
            let addr = resolve_label_or_immediate(op, symbol_table, line_num, current_bank)?;
            let [low, high] = addr.to_le_bytes();
            Ok(vec![JMP_IMM_OPCODE, low, high])
        }
        Instruction::JmpIndirect(reg) => {
            let opcode = encode_reg_opcode(JMP_INDIR_BASE_OPCODE, reg);
            Ok(vec![opcode])
        }
        Instruction::JrI(Operand::Immediate(imm)) => {
            let rel = *imm as i8;
            Ok(vec![JR_OPCODE, rel as u8])
        }
        Instruction::JrI(Operand::Label(label_name)) => {
            let target_symbol =
                get_and_check_symbol(symbol_table, label_name, line_num, current_bank)?;
            let rel: i32 = target_symbol.logical_address as i32 - *current_address as i32;
            if rel > i8::MAX as i32 || rel < i8::MIN as i32 {
                return Err(AssemblyError::SemanticError {
                    line: line_num,
                    reason: format!(
                        "Label \"{}\" too far away for relative jump, must be within {} bytes of JR instruction",
                        label_name,
                        i8::MAX
                    ),
                });
            };
            Ok(vec![JR_OPCODE, rel as u8])
        }
        Instruction::JccI(cc, op) => {
            let addr = resolve_label_or_immediate(op, symbol_table, line_num, current_bank)?;
            let opcode = encode_condition_code_opcode(JCC_BASE_OPCODE, cc);
            let [low, high] = addr.to_le_bytes();
            Ok(vec![opcode, low, high])
        }
        Instruction::JrccI(cc, Operand::Immediate(imm)) => {
            let rel = *imm as i8;
            let opcode = encode_condition_code_opcode(JRCC_BASE_OPCODE, cc);
            Ok(vec![opcode, rel as u8])
        }
        Instruction::JrccI(cc, Operand::Label(label_name)) => {
            let target_symbol =
                get_and_check_symbol(symbol_table, label_name, line_num, current_bank)?;
            let rel: i32 = target_symbol.logical_address as i32 - *current_address as i32;
            if rel > i8::MAX as i32 || rel < i8::MIN as i32 {
                return Err(AssemblyError::SemanticError {
                    line: line_num,
                    reason: format!(
                        "Label \"{}\" too far away for relative jump, must be within {} bytes of JRcc instruction",
                        label_name,
                        i8::MAX
                    ),
                });
            };
            let opcode = encode_condition_code_opcode(JRCC_BASE_OPCODE, cc);
            Ok(vec![opcode, rel as u8])
        }
        Instruction::Djnz(Operand::Immediate(imm)) => {
            let rel = *imm as i8;
            Ok(vec![DJNZ_OPCODE, rel as u8])
        }
        Instruction::Djnz(Operand::Label(label_name)) => {
            let target_symbol =
                get_and_check_symbol(symbol_table, label_name, line_num, current_bank)?;
            let rel: i32 = target_symbol.logical_address as i32 - *current_address as i32;
            if rel > i8::MAX as i32 || rel < i8::MIN as i32 {
                return Err(AssemblyError::SemanticError {
                    line: line_num,
                    reason: format!(
                        "Label \"{}\" too far away for relative jump, must be within {} bytes of DJNZ instruction",
                        label_name,
                        i8::MAX
                    ),
                });
            };
            Ok(vec![DJNZ_OPCODE, rel as u8])
        }
        Instruction::CallI(op) => {
            let addr = resolve_label_or_immediate(op, symbol_table, line_num, current_bank)?;
            Ok(encode_call_immediate(addr))
        }
        Instruction::CallIndirect(reg) => {
            let opcode = encode_reg_opcode(CALL_INDIR_BASE_OPCODE, reg);
            Ok(vec![opcode])
        }
        Instruction::CallccI(cc, op) => {
            let addr = resolve_label_or_immediate(op, symbol_table, line_num, current_bank)?;
            let opcode = encode_condition_code_opcode(CALLCC_BASE_OPCODE, cc);
            let [low, high] = addr.to_le_bytes();
            Ok(vec![opcode, low, high])
        }
        Instruction::Syscall(imm) => Ok(encode_syscall(*imm as u8)),
        Instruction::CallFar(call_label) => {
            let target_symbol = get_symbol(symbol_table, call_label, line_num)?;

            if target_symbol.bank == 0 {
                return Err(AssemblyError::SemanticError {
                    line: line_num,
                    reason: format!(
                        "Label \"{}\" exists in bank 0, use a normal CALL instruction instead.",
                        call_label
                    ),
                });
            } else if target_symbol.bank == *current_bank {
                return Err(AssemblyError::SemanticError {
                    line: line_num,
                    reason: format!(
                        "Label \"{}\" exists in the same bank as the CALL.far instruction, use a normal CALL instruction instead.",
                        call_label
                    ),
                });
            }

            let final_bytecode = encode_far(target_symbol, 0x21);
            Ok(final_bytecode)
        }
        Instruction::CallFarVia(call_label, via_label) => {
            let call_symbol = get_symbol(symbol_table, call_label, line_num)?;
            let via_symbol = get_symbol(symbol_table, via_label, line_num)?;

            if call_symbol.bank == 0 {
                return Err(AssemblyError::SemanticError {
                    line: line_num,
                    reason: format!(
                        "Label \"{}\" exists in bank 0, use a normal CALL instruction instead.",
                        call_label
                    ),
                });
            } else if call_symbol.bank == *current_bank {
                return Err(AssemblyError::SemanticError {
                    line: line_num,
                    reason: format!(
                        "Label \"{}\" exists in the same bank as the CALL.far instruction, use a normal CALL instruction instead.",
                        call_label
                    ),
                });
            }

            if via_symbol.bank != 0 {
                return Err(AssemblyError::SemanticError {
                    line: line_num,
                    reason: format!(
                        "Custom CALL.far via trampoline label must exist in bank 0, \"{}\" found in bank {}",
                        via_label, via_symbol.bank,
                    ),
                });
            }

            let final_bytecode = encode_far_via(call_symbol, via_symbol);
            Ok(final_bytecode)
        }
        Instruction::JmpFar(label_name) => {
            let target_symbol = get_symbol(symbol_table, label_name, line_num)?;

            if target_symbol.bank == 0 {
                return Err(AssemblyError::SemanticError {
                    line: line_num,
                    reason: format!(
                        "Label \"{}\" exists in bank 0, use a normal JMP instruction instead.",
                        label_name
                    ),
                });
            } else if target_symbol.bank == *current_bank {
                return Err(AssemblyError::SemanticError {
                    line: line_num,
                    reason: format!(
                        "Label \"{}\" exists in the same bank as the JMP.far instruction, use a normal JMP instruction instead.",
                        label_name
                    ),
                });
            }

            let final_bytecode = encode_far(target_symbol, 0x22);
            Ok(final_bytecode)
        }
        Instruction::JmpFarVia(call_label, via_label) => {
            let call_symbol = get_symbol(symbol_table, call_label, line_num)?;
            let via_symbol = get_symbol(symbol_table, via_label, line_num)?;

            if call_symbol.bank == 0 {
                return Err(AssemblyError::SemanticError {
                    line: line_num,
                    reason: format!(
                        "Label \"{}\" exists in bank 0, use a normal JMP instruction instead.",
                        call_label
                    ),
                });
            } else if call_symbol.bank == *current_bank {
                return Err(AssemblyError::SemanticError {
                    line: line_num,
                    reason: format!(
                        "Label \"{}\" exists in the same bank as the JMP.far instruction, use a normal JMP instruction instead.",
                        call_label
                    ),
                });
            }

            if via_symbol.bank != 0 {
                return Err(AssemblyError::SemanticError {
                    line: line_num,
                    reason: format!(
                        "Custom JMP.far via trampoline label must exist in bank 0, \"{}\" found in bank {}",
                        via_label, via_symbol.bank,
                    ),
                });
            }

            let final_bytecode = encode_far_via(call_symbol, via_symbol);
            Ok(final_bytecode)
        }

        // ... add encoding logic for every instruction variant based on your opcode map ...
        _ => Err(AssemblyError::SemanticErrorNoLine {
            reason: "Invalid Instruction".to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{ConditionCode, Register};

    #[test]
    fn test_encode_instruction_nop() {
        let instruction = Instruction::Nop;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x00]
        );
    }

    #[test]
    fn test_encode_instruction_sub_acc() {
        let instruction = Instruction::SubAcc(Register::R1);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x21]
        );
    }

    #[test]
    fn test_encode_instruction_and_reg_reg() {
        let instruction = Instruction::AndReg(Register::R2, Register::R3);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x12, (2 << 3) | 3]
        );
    }

    #[test]
    fn test_encode_instruction_or_reg_reg() {
        let instruction = Instruction::OrReg(Register::R4, Register::R5);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x13, (4 << 3) | 5]
        );
    }

    #[test]
    fn test_encode_instruction_xor_reg_reg() {
        let instruction = Instruction::XorReg(Register::R6, Register::R7);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x14, (6 << 3) | 7]
        );
    }

    #[test]
    fn test_encode_instruction_cmp_reg_reg() {
        let instruction = Instruction::CmpReg(Register::R0, Register::R1);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x15, (0 << 3) | 1]
        );
    }

    #[test]
    fn test_encode_instruction_adc_reg_reg() {
        let instruction = Instruction::AdcReg(Register::R2, Register::R3);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x16, (2 << 3) | 3]
        );
    }

    #[test]
    fn test_encode_instruction_sbc_reg_reg() {
        let instruction = Instruction::SbcReg(Register::R4, Register::R5);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x17, (4 << 3) | 5]
        );
    }

    #[test]
    fn test_encode_instruction_and_acc() {
        let instruction = Instruction::AndAcc(Register::R1);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x29]
        );
    }

    #[test]
    fn test_encode_instruction_or_acc() {
        let instruction = Instruction::OrAcc(Register::R2);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x32]
        );
    }

    #[test]
    fn test_encode_instruction_xor_acc() {
        let instruction = Instruction::XorAcc(Register::R3);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x3B]
        );
    }

    #[test]
    fn test_encode_instruction_cmp_acc() {
        let instruction = Instruction::CmpAcc(Register::R4);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x44]
        );
    }

    #[test]
    fn test_encode_instruction_add_acc_immediate() {
        let instruction = Instruction::AddAccI(Operand::Immediate(0x1234));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xC0, 0x34, 0x12]
        );
    }

    #[test]
    fn test_encode_instruction_add_acc_label() {
        let instruction = Instruction::AddAccI(Operand::Label("TARGET".into()));
        let mut symbol_table = SymbolTable::new();
        symbol_table.insert(
            "TARGET".to_string(),
            Symbol {
                logical_address: 0x2468,
                bank: 0,
            },
        );
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xC0, 0x68, 0x24]
        );
    }

    #[test]
    fn test_encode_instruction_sub_acc_immediate() {
        let instruction = Instruction::SubAccI(Operand::Immediate(0x00FF));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xC1, 0xFF, 0x00]
        );
    }

    #[test]
    fn test_encode_instruction_and_acc_immediate() {
        let instruction = Instruction::AndAccI(Operand::Immediate(0x0F0F));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xC2, 0x0F, 0x0F]
        );
    }

    #[test]
    fn test_encode_instruction_or_acc_immediate() {
        let instruction = Instruction::OrAccI(Operand::Immediate(0x8000));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xC3, 0x00, 0x80]
        );
    }

    #[test]
    fn test_encode_instruction_xor_acc_immediate() {
        let instruction = Instruction::XorAccI(Operand::Immediate(0xAAAA));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xC4, 0xAA, 0xAA]
        );
    }

    #[test]
    fn test_encode_instruction_cmp_acc_immediate() {
        let instruction = Instruction::CmpAccI(Operand::Immediate(0x0A0B));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xC5, 0x0B, 0x0A]
        );
    }

    #[test]
    fn test_encode_instruction_adci_acc_immediate() {
        let instruction = Instruction::AdcAccI(Operand::Immediate(0xFFFF));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xC6, 0xFF, 0xFF]
        );
    }

    #[test]
    fn test_encode_instruction_sbci_acc_immediate() {
        let instruction = Instruction::SbcAccI(Operand::Immediate(0x1234));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xC7, 0x34, 0x12]
        );
    }

    #[test]
    fn test_encode_instruction_add_sp() {
        let instruction = Instruction::AddSp(-5);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x6C, 0xFB]
        );
    }

    #[test]
    fn test_encode_instruction_push_reg() {
        let instruction = Instruction::Push(Register::R2);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x6F]
        );
    }

    #[test]
    fn test_encode_instruction_push_immediate() {
        let instruction = Instruction::PushI(Operand::Immediate(0x1234));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x7D, 0x34, 0x12]
        );
    }

    #[test]
    fn test_encode_instruction_push_label() {
        let instruction = Instruction::PushI(Operand::Label("TARGET".into()));
        let mut symbol_table = SymbolTable::new();
        symbol_table.insert(
            "TARGET".to_string(),
            Symbol {
                logical_address: 0x1357,
                bank: 0,
            },
        );
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x7D, 0x57, 0x13]
        );
    }

    #[test]
    fn test_encode_instruction_push_f() {
        let instruction = Instruction::PushF;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x7E]
        );
    }

    #[test]
    fn test_encode_instruction_pop_reg() {
        let instruction = Instruction::Pop(Register::R3);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x78]
        );
    }

    #[test]
    fn test_encode_instruction_pop_f() {
        let instruction = Instruction::PopF;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x7F]
        );
    }

    #[test]
    fn test_encode_instruction_neg() {
        let instruction = Instruction::NegAcc;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x48]
        );
    }

    #[test]
    fn test_encode_instruction_not() {
        let instruction = Instruction::NotAcc;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x49]
        );
    }

    #[test]
    fn test_encode_instruction_swap() {
        let instruction = Instruction::SwapAcc;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x4A]
        );
    }

    #[test]
    fn test_encode_instruction_ccf() {
        let instruction = Instruction::Ccf;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x4B]
        );
    }

    #[test]
    fn test_encode_instruction_scf() {
        let instruction = Instruction::Scf;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x4C]
        );
    }

    #[test]
    fn test_encode_instruction_rcf() {
        let instruction = Instruction::Rcf;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x4D]
        );
    }

    #[test]
    fn test_encode_instruction_enter() {
        let instruction = Instruction::Enter;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x4F]
        );
    }

    #[test]
    fn test_encode_instruction_leave() {
        let instruction = Instruction::Leave;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x50]
        );
    }

    #[test]
    fn test_encode_instruction_ret() {
        let instruction = Instruction::Ret;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xF9]
        );
    }

    #[test]
    fn test_encode_instruction_reti() {
        let instruction = Instruction::Reti;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFA]
        );
    }

    #[test]
    fn test_encode_instruction_ei() {
        let instruction = Instruction::Ei;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFB]
        );
    }

    #[test]
    fn test_encode_instruction_di() {
        let instruction = Instruction::Di;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFC]
        );
    }

    #[test]
    fn test_encode_instruction_inc() {
        let instruction = Instruction::Inc(Register::R1);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xE2]
        );
    }

    #[test]
    fn test_encode_instruction_dec() {
        let instruction = Instruction::Dec(Register::R2);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xDB]
        );
    }

    #[test]
    fn test_encode_instruction_call_immediate() {
        let instruction = Instruction::CallI(Operand::Immediate(0x1234));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xC8, 0x34, 0x12]
        );
    }

    #[test]
    fn test_encode_instruction_call_label() {
        let instruction = Instruction::CallI(Operand::Label("test_label".to_string()));
        let mut symbol_table = SymbolTable::new();
        symbol_table.insert(
            "test_label".to_string(),
            Symbol {
                logical_address: 0x4321,
                bank: 1,
            },
        );
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0x4100, &1, 0).unwrap(),
            vec![0xC8, 0x21, 0x43]
        );
    }

    #[test]
    fn test_encode_instruction_call_indirect() {
        let instruction = Instruction::CallIndirect(Register::R4);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xCD] // 0xC9 + 4
        );
    }

    #[test]
    fn test_encode_instruction_callcc_immediate() {
        let instruction = Instruction::CallccI(ConditionCode::C, Operand::Immediate(0x1122));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xD5, 0x22, 0x11] // 0xD1 + 4
        );
    }

    #[test]
    fn test_encode_instruction_callcc_label() {
        let instruction =
            Instruction::CallccI(ConditionCode::Nz, Operand::Label("test_label".to_string()));
        let mut symbol_table = SymbolTable::new();
        symbol_table.insert(
            "test_label".to_string(),
            Symbol {
                logical_address: 0x4321,
                bank: 1,
            },
        );
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0x4100, &1, 0).unwrap(),
            vec![0xD8, 0x21, 0x43] // 0xD1 + 7
        );
    }

    #[test]
    fn test_encode_instruction_syscall() {
        let instruction = Instruction::Syscall(0x1A);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0x4E, 0x1A]
        );
    }

    #[test]
    fn test_encode_instruction_ld_absolute() {
        let instruction = Instruction::LdAbs(Register::R1, Operand::AbsAddr(0x1234));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xEA, 0x34, 0x12] // 0xE9 + 1
        );
    }

    #[test]
    fn test_encode_instruction_st_absolute() {
        let instruction = Instruction::StAbs(Operand::AbsAddr(0x4321), Register::R2);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xF3, 0x21, 0x43] // 0xF1 + 2
        );
    }

    #[test]
    fn test_encode_instruction_st_indirect() {
        let instruction = Instruction::StIndirect(Register::R1, Register::R2);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFE, 0x4A] // 0x40 | (1 << 3) | 2
        );
    }

    #[test]
    fn test_encode_sra() {
        let instruction = Instruction::Sra(Register::R0);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFD, 0x00]
        );
    }

    #[test]
    fn test_encode_shl() {
        let instruction = Instruction::Shl(Register::R1);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFD, 0x09]
        );
    }

    #[test]
    fn test_encode_shr() {
        let instruction = Instruction::Shr(Register::R2);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFD, 0x12]
        );
    }

    #[test]
    fn test_encode_rol() {
        let instruction = Instruction::Rol(Register::R3);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFD, 0x1B]
        );
    }

    #[test]
    fn test_encode_ror() {
        let instruction = Instruction::Ror(Register::R4);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFD, 0x24]
        );
    }

    #[test]
    fn test_encode_instruction_add_b() {
        let instruction = Instruction::AddBAcc(Register::R1);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFD, 0x29] // 0x28 + 1
        );
    }

    #[test]
    fn test_encode_instruction_sub_b() {
        let instruction = Instruction::SubBAcc(Register::R2);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFD, 0x32] // 0x30 + 2
        );
    }

    #[test]
    fn test_encode_instruction_and_b() {
        let instruction = Instruction::AndBAcc(Register::R3);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFD, 0x3B] // 0x38 + 3
        );
    }

    #[test]
    fn test_encode_instruction_or_b() {
        let instruction = Instruction::OrBAcc(Register::R4);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFD, 0x44] // 0x40 + 4
        );
    }

    #[test]
    fn test_encode_instruction_xor_b() {
        let instruction = Instruction::XorBAcc(Register::R5);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFD, 0x4D] // 0x48 + 5
        );
    }

    #[test]
    fn test_encode_instruction_cmp_b() {
        let instruction = Instruction::CmpBAcc(Register::R6);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFD, 0x56] // 0x50 + 6
        );
    }

    #[test]
    fn test_encode_instruction_ldi_b() {
        let instruction = Instruction::LdiB(Register::R1, 0xAB);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFD, 0xA1, 0xAB] // 0xA0 + 1
        );
    }

    #[test]
    fn test_encode_instruction_bit_register() {
        let instruction = Instruction::BitReg(Register::R1, 7);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFD, 0x5F, 1]
        );
    }

    #[test]
    fn test_encode_instruction_set_absolute() {
        let instruction = Instruction::SetAbs(Operand::AbsAddr(0x1234), 0);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFD, 0x78, 0x34, 0x12]
        );
    }

    #[test]
    fn test_encode_instruction_res_indirect() {
        let instruction = Instruction::ResIndirect(Register::R2, 3);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFD, 0x9B, 2]
        );
    }

    #[test]
    fn test_encode_instruction_ld_indexed() {
        let instruction = Instruction::LdIndexed(Register::R0, Register::R1, 16);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFF, 0x01, 16]
        );
    }

    #[test]
    fn test_encode_instruction_st_indexed() {
        let instruction = Instruction::StIndexed(Register::R2, -1, Register::R3);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFF, 0x53, 255]
        );
    }

    #[test]
    fn test_encode_instruction_lea_indexed() {
        let instruction = Instruction::Lea(Register::R4, Register::R5, 32);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFF, 0xA5, 32]
        );
    }

    #[test]
    fn test_encode_instruction_ld_post_increment() {
        let instruction = Instruction::LdPostInc(Register::R6, Register::R7);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFF, 0xC7, 6]
        );
    }

    #[test]
    fn test_encode_instruction_st_post_increment() {
        let instruction = Instruction::StPostInc(Register::R0, Register::R1);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFF, 0xC9, 0]
        );
    }

    #[test]
    fn test_encode_instruction_ld_pre_decrement() {
        let instruction = Instruction::LdPreDec(Register::R2, Register::R3);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFF, 0xD3, 2]
        );
    }

    #[test]
    fn test_encode_instruction_st_pre_decrement() {
        let instruction = Instruction::StPreDec(Register::R4, Register::R5);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFF, 0xDD, 4]
        );
    }

    #[test]
    fn test_encode_instruction_ld_b_post_increment() {
        let instruction = Instruction::LdBPostInc(Register::R6, Register::R7);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFF, 0xE7, 6]
        );
    }

    #[test]
    fn test_encode_instruction_st_b_post_increment() {
        let instruction = Instruction::StBPostInc(Register::R0, Register::R1);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFF, 0xE9, 0]
        );
    }

    #[test]
    fn test_encode_instruction_ld_b_pre_decrement() {
        let instruction = Instruction::LdBPreDec(Register::R2, Register::R3);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFF, 0xF3, 2]
        );
    }

    #[test]
    fn test_encode_instruction_st_b_pre_decrement() {
        let instruction = Instruction::StBPreDec(Register::R4, Register::R5);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, 0).unwrap(),
            vec![0xFF, 0xFD, 4]
        );
    }

    #[test]
    fn test_encode_instruction_call_far() {
        let instruction = Instruction::CallFar("test_label".to_string());
        let mut symbol_table = SymbolTable::new();
        symbol_table.insert(
            "test_label".to_string(),
            Symbol {
                logical_address: 0x4321,
                bank: 1,
            },
        );
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0x1100, &0, 0).unwrap(),
            vec![0x05, 0x01, 0x00, 0x06, 0x21, 0x43, 0x4E, 0x21]
        );
    }

    #[test]
    fn test_encode_instruction_call_far_bank_0_fail() {
        let instruction = Instruction::CallFar("test_label".to_string());
        let mut symbol_table = SymbolTable::new();
        symbol_table.insert(
            "test_label".to_string(),
            Symbol {
                logical_address: 0x2100,
                bank: 0,
            },
        );
        let result = encode_instruction(&instruction, &symbol_table, &0x5432, &1, 1);
        assert!(result.is_err_and(|e| {
            e == AssemblyError::SemanticError {
                line: 1,
                reason:
                    "Label \"test_label\" exists in bank 0, use a normal CALL instruction instead."
                        .to_string(),
            }
        }));
    }

    #[test]
    fn test_encode_instruction_call_far_same_bank_fail() {
        let instruction = Instruction::CallFar("test_label".to_string());
        let mut symbol_table = SymbolTable::new();
        symbol_table.insert(
            "test_label".to_string(),
            Symbol {
                logical_address: 0x4321,
                bank: 1,
            },
        );
        let result = encode_instruction(&instruction, &symbol_table, &0x5432, &1, 1);
        assert!(result.is_err_and(|e| {
            e == AssemblyError::SemanticError {
                line: 1,
                reason:
                    "Label \"test_label\" exists in the same bank as the CALL.far instruction, use a normal CALL instruction instead."
                        .to_string(),
            }
        }));
    }

    #[test]
    fn test_encode_instruction_call_far_via() {
        let instruction = Instruction::CallFarVia("test_label".to_string(), "tramp".to_string());
        let mut symbol_table = SymbolTable::new();
        symbol_table.insert(
            "test_label".to_string(),
            Symbol {
                logical_address: 0x4321,
                bank: 1,
            },
        );
        symbol_table.insert(
            "tramp".to_string(),
            Symbol {
                logical_address: 0x0200,
                bank: 0,
            },
        );
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0x1100, &0, 0).unwrap(),
            vec![0x05, 0x01, 0x00, 0x06, 0x21, 0x43, 0xC8, 0x00, 0x02]
        );
    }

    #[test]
    fn test_encode_instruction_call_far_via_bank_0_fail() {
        let instruction = Instruction::CallFarVia("test_label".to_string(), "tramp".to_string());
        let mut symbol_table = SymbolTable::new();
        symbol_table.insert(
            "test_label".to_string(),
            Symbol {
                logical_address: 0x2100,
                bank: 0,
            },
        );
        symbol_table.insert(
            "tramp".to_string(),
            Symbol {
                logical_address: 0x0200,
                bank: 0,
            },
        );
        let result = encode_instruction(&instruction, &symbol_table, &0x5432, &1, 1);
        assert!(result.is_err_and(|e| {
            e == AssemblyError::SemanticError {
                line: 1,
                reason:
                    "Label \"test_label\" exists in bank 0, use a normal CALL instruction instead."
                        .to_string(),
            }
        }));
    }

    #[test]
    fn test_encode_instruction_call_far_via_same_bank_fail() {
        let instruction = Instruction::CallFarVia("test_label".to_string(), "tramp".to_string());
        let mut symbol_table = SymbolTable::new();
        symbol_table.insert(
            "test_label".to_string(),
            Symbol {
                logical_address: 0x4321,
                bank: 1,
            },
        );
        symbol_table.insert(
            "tramp".to_string(),
            Symbol {
                logical_address: 0x0200,
                bank: 0,
            },
        );
        let result = encode_instruction(&instruction, &symbol_table, &0x5432, &1, 1);
        assert!(result.is_err_and(|e| {
            e == AssemblyError::SemanticError {
                line: 1,
                reason:
                    "Label \"test_label\" exists in the same bank as the CALL.far instruction, use a normal CALL instruction instead."
                        .to_string(),
            }
        }));
    }

    #[test]
    fn test_encode_instruction_call_far_via_invalid_bank_fail() {
        let instruction = Instruction::CallFarVia("test_label".to_string(), "tramp".to_string());
        let mut symbol_table = SymbolTable::new();
        symbol_table.insert(
            "test_label".to_string(),
            Symbol {
                logical_address: 0x4400,
                bank: 2,
            },
        );
        symbol_table.insert(
            "tramp".to_string(),
            Symbol {
                logical_address: 0x4848,
                bank: 1,
            },
        );
        let result = encode_instruction(&instruction, &symbol_table, &0x5432, &1, 1);
        assert!(result.is_err_and(|e| {
            e == AssemblyError::SemanticError {
                line: 1,
                reason:
                    "Custom CALL.far via trampoline label must exist in bank 0, \"tramp\" found in bank 1"
                        .to_string(),
            }
        }));
    }
}
