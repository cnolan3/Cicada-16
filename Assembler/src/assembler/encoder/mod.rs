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

mod constants;
mod instruction_encoders;
mod utility_functions;

use crate::assembler::symbol_table::*;
use crate::ast::Instruction;
use crate::errors::AssemblyError;
use constants::*;

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

// encode instruction wrapper
pub fn encode_instruction(
    instruction: &Instruction,
    symbol_table: &SymbolTable,
    current_address: &u32,
    current_bank: &u32,
    line_num: &usize,
) -> Result<Vec<u8>, AssemblyError> {
    let encoder = Encoder::new(
        instruction,
        symbol_table,
        current_address,
        current_bank,
        line_num,
    );
    encoder.encode_instruction()
}

pub struct Encoder<'a> {
    pub instruction: &'a Instruction,
    pub symbol_table: &'a SymbolTable,
    pub current_address: &'a u32,
    pub current_bank: &'a u32,
    pub line_num: &'a usize,
}

impl<'a> Encoder<'a> {
    pub fn new(
        instruction: &'a Instruction,
        symbol_table: &'a SymbolTable,
        current_address: &'a u32,
        current_bank: &'a u32,
        line_num: &'a usize,
    ) -> Self {
        Self {
            instruction,
            symbol_table,
            current_address,
            current_bank,
            line_num,
        }
    }

    /// Helper function to translate a single instruction into bytes during Pass 2.
    pub fn encode_instruction(self) -> Result<Vec<u8>, AssemblyError> {
        match self.instruction {
            Instruction::Nop => self.encode_nop(),
            Instruction::Halt => self.encode_halt(),
            Instruction::Ei => self.encode_ei(),
            Instruction::Di => self.encode_di(),
            Instruction::Ret => self.encode_ret(),
            Instruction::Reti => self.encode_reti(),
            Instruction::Ccf => self.encode_ccf(),
            Instruction::Scf => self.encode_scf(),
            Instruction::Rcf => self.encode_rcf(),
            Instruction::Enter => self.encode_enter(),
            Instruction::Leave => self.encode_leave(),

            // Load/Store
            Instruction::LdReg(rd, rs) => self.encode_ld_reg(rd, rs),
            Instruction::Ldi(rd, op) => self.encode_ldi(rd, op),
            Instruction::LdIndirect(rd, rs) => self.encode_ld_indirect(rd, rs),
            Instruction::LdAbs(rd, op) => self.encode_ld_abs(rd, op),
            Instruction::LdIndexed(rd, rs, offset) => self.encode_ld_indexed(rd, rs, offset),
            Instruction::LdPreDec(rd, rs) => self.encode_ld_pre_dec(rd, rs),
            Instruction::LdPostInc(rd, rs) => self.encode_ld_post_inc(rd, rs),
            Instruction::StIndirect(rd, rs) => self.encode_st_indirect(rd, rs),
            Instruction::StAbs(op, rs) => self.encode_st_abs(op, rs),
            Instruction::StIndexed(rd, offset, rs) => self.encode_st_indexed(rd, offset, rs),
            Instruction::StPreDec(rd, rs) => self.encode_st_pre_dec(rd, rs),
            Instruction::StPostInc(rd, rs) => self.encode_st_post_inc(rd, rs),
            Instruction::LdiB(rd, imm8) => self.encode_ldib(rd, imm8),
            Instruction::LdBIndirect(rd, rs) => self.encode_ldb_indirect(rd, rs),
            Instruction::LdBPreDec(rd, rs) => self.encode_ldb_pre_dec(rd, rs),
            Instruction::LdBPostInc(rd, rs) => self.encode_ldb_post_inc(rd, rs),
            Instruction::StBIndirect(rd, rs) => self.encode_stb_indirect(rd, rs),
            Instruction::StBPreDec(rd, rs) => self.encode_stb_pre_dec(rd, rs),
            Instruction::StBPostInc(rd, rs) => self.encode_stb_post_inc(rd, rs),
            Instruction::Lea(rd, rs, offset) => self.encode_lea(rd, rs, offset),

            // Stack
            Instruction::Push(op) => self.encode_push_reg(op),
            Instruction::Pop(reg) => self.encode_pop(reg),
            Instruction::PushI(op) => self.encode_push_imm(op),
            Instruction::PushF => self.encode_pushf(),
            Instruction::PopF => self.encode_popf(),

            // Arithmetic
            Instruction::AddAcc(rs) => self.encode_acc_math(ADD_ACC_BASE_OPCODE, rs),
            Instruction::SubAcc(rs) => self.encode_acc_math(SUB_ACC_BASE_OPCODE, rs),
            Instruction::AndAcc(rs) => self.encode_acc_math(AND_ACC_BASE_OPCODE, rs),
            Instruction::OrAcc(rs) => self.encode_acc_math(OR_ACC_BASE_OPCODE, rs),
            Instruction::XorAcc(rs) => self.encode_acc_math(XOR_ACC_BASE_OPCODE, rs),
            Instruction::CmpAcc(rs) => self.encode_acc_math(CMP_ACC_BASE_OPCODE, rs),
            Instruction::NegAcc => Ok(vec![NEG_ACC_OPCODE]),
            Instruction::NotAcc => Ok(vec![NOR_ACC_OPCODE]),
            Instruction::SwapAcc => Ok(vec![SWAP_ACC_OPCODE]),
            Instruction::AddAccI(op) => self.encode_acc_imm_math(ADDI_ACC_OPCODE, op),
            Instruction::SubAccI(op) => self.encode_acc_imm_math(SUBI_ACC_OPCODE, op),
            Instruction::AndAccI(op) => self.encode_acc_imm_math(ANDI_ACC_OPCODE, op),
            Instruction::OrAccI(op) => self.encode_acc_imm_math(ORI_ACC_OPCODE, op),
            Instruction::XorAccI(op) => self.encode_acc_imm_math(XORI_ACC_OPCODE, op),
            Instruction::CmpAccI(op) => self.encode_acc_imm_math(CMPI_ACC_OPCODE, op),
            Instruction::AdcAccI(op) => self.encode_acc_imm_math(ADCI_ACC_OPCODE, op),
            Instruction::SbcAccI(op) => self.encode_acc_imm_math(SBCI_ACC_OPCODE, op),
            Instruction::AddReg(rd, rs) => self.encode_reg_math(ADD_OPCODE, rd, rs),
            Instruction::SubReg(rd, rs) => self.encode_reg_math(SUB_OPCODE, rd, rs),
            Instruction::AndReg(rd, rs) => self.encode_reg_math(AND_OPCODE, rd, rs),
            Instruction::OrReg(rd, rs) => self.encode_reg_math(OR_OPCODE, rd, rs),
            Instruction::XorReg(rd, rs) => self.encode_reg_math(XOR_OPCODE, rd, rs),
            Instruction::CmpReg(rd, rs) => self.encode_reg_math(CMP_OPCODE, rd, rs),
            Instruction::AdcReg(rd, rs) => self.encode_reg_math(ADC_OPCODE, rd, rs),
            Instruction::SbcReg(rd, rs) => self.encode_reg_math(SBC_OPCODE, rd, rs),
            Instruction::AddIReg(rd, op) => self.encode_reg_imm_math(ADDI_OPCODE, rd, op),
            Instruction::SubIReg(rd, op) => self.encode_reg_imm_math(SUBI_OPCODE, rd, op),
            Instruction::AndIReg(rd, op) => self.encode_reg_imm_math(ANDI_OPCODE, rd, op),
            Instruction::OrIReg(rd, op) => self.encode_reg_imm_math(ORI_OPCODE, rd, op),
            Instruction::XorIReg(rd, op) => self.encode_reg_imm_math(XORI_OPCODE, rd, op),
            Instruction::CmpIReg(rd, op) => self.encode_reg_imm_math(CMPI_OPCODE, rd, op),
            Instruction::AddSp(offset) => self.encode_add_sp(offset),
            Instruction::Inc(reg) => self.encode_inc(reg),
            Instruction::Dec(reg) => self.encode_dec(reg),
            Instruction::AddBAcc(rs) => self.encode_acc_byte_math(ADDB_BASE_SUB_OPCODE, rs),
            Instruction::SubBAcc(rs) => self.encode_acc_byte_math(SUBB_BASE_SUB_OPCODE, rs),
            Instruction::AndBAcc(rs) => self.encode_acc_byte_math(ANDB_BASE_SUB_OPCODE, rs),
            Instruction::OrBAcc(rs) => self.encode_acc_byte_math(ORB_BASE_SUB_OPCODE, rs),
            Instruction::XorBAcc(rs) => self.encode_acc_byte_math(XORB_BASE_SUB_OPCODE, rs),
            Instruction::CmpBAcc(rs) => self.encode_acc_byte_math(CMPB_BASE_SUB_OPCODE, rs),

            // Bitwise
            Instruction::Sra(reg) => self.encode_shift(SRA_BASE_SUB_OPCODE, reg),
            Instruction::Shl(reg) => self.encode_shift(SHL_BASE_SUB_OPCODE, reg),
            Instruction::Shr(reg) => self.encode_shift(SHR_BASE_SUB_OPCODE, reg),
            Instruction::Rol(reg) => self.encode_shift(ROL_BASE_SUB_OPCODE, reg),
            Instruction::Ror(reg) => self.encode_shift(ROR_BASE_SUB_OPCODE, reg),
            Instruction::BitReg(r, b) => self.encode_bit_reg(r, b),
            Instruction::SetReg(r, b) => self.encode_set_reg(r, b),
            Instruction::ResReg(r, b) => self.encode_res_reg(r, b),
            Instruction::BitAbs(op, b) => self.encode_bit_abs(op, b),
            Instruction::SetAbs(op, b) => self.encode_set_abs(op, b),
            Instruction::ResAbs(op, b) => self.encode_res_abs(op, b),
            Instruction::BitIndirect(r, b) => self.encode_bit_indirect(r, b),
            Instruction::SetIndirect(r, b) => self.encode_set_indirect(r, b),
            Instruction::ResIndirect(r, b) => self.encode_res_indirect(r, b),

            // Control Flow
            Instruction::JmpI(op) => self.encode_jmp_imm(op),
            Instruction::JmpIndirect(reg) => self.encode_jmp_indirect(reg),
            Instruction::JrI(op) => self.encode_jr(op),
            Instruction::JccI(cc, op) => self.encode_jcc(cc, op),
            Instruction::JrccI(cc, op) => self.encode_jrcc(cc, op),
            Instruction::Djnz(op) => self.encode_djnz(op),
            Instruction::CallI(op) => self.encode_call_imm(op),
            Instruction::CallIndirect(reg) => self.encode_call_indirect(reg),
            Instruction::CallccI(cc, op) => self.encode_callcc(cc, op),
            Instruction::Syscall(imm) => self.encode_syscall_imm(imm),
            Instruction::CallFar(call_label) => self.encode_call_far(call_label),
            Instruction::CallFarVia(call_label, via_label) => {
                self.encode_call_far_via(call_label, via_label)
            }
            Instruction::JmpFar(label_name) => self.encode_jmp_far(label_name),
            Instruction::JmpFarVia(call_label, via_label) => {
                self.encode_jmp_far_via(call_label, via_label)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{ConditionCode, Operand, Register};

    #[test]
    fn test_encode_instruction_nop() {
        let instruction = Instruction::Nop;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0x00]
        );
    }

    #[test]
    fn test_encode_instruction_sub_acc() {
        let instruction = Instruction::SubAcc(Register::R1);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0x21]
        );
    }

    #[test]
    fn test_encode_instruction_and_reg_reg() {
        let instruction = Instruction::AndReg(Register::R2, Register::R3);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0x12, (2 << 3) | 3]
        );
    }

    #[test]
    fn test_encode_instruction_or_reg_reg() {
        let instruction = Instruction::OrReg(Register::R4, Register::R5);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0x13, (4 << 3) | 5]
        );
    }

    #[test]
    fn test_encode_instruction_xor_reg_reg() {
        let instruction = Instruction::XorReg(Register::R6, Register::R7);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0x14, (6 << 3) | 7]
        );
    }

    #[test]
    fn test_encode_instruction_cmp_reg_reg() {
        let instruction = Instruction::CmpReg(Register::R0, Register::R1);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0x15, (0 << 3) | 1]
        );
    }

    #[test]
    fn test_encode_instruction_adc_reg_reg() {
        let instruction = Instruction::AdcReg(Register::R2, Register::R3);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0x16, (2 << 3) | 3]
        );
    }

    #[test]
    fn test_encode_instruction_sbc_reg_reg() {
        let instruction = Instruction::SbcReg(Register::R4, Register::R5);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0x17, (4 << 3) | 5]
        );
    }

    #[test]
    fn test_encode_instruction_and_acc() {
        let instruction = Instruction::AndAcc(Register::R1);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0x29]
        );
    }

    #[test]
    fn test_encode_instruction_or_acc() {
        let instruction = Instruction::OrAcc(Register::R2);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0x32]
        );
    }

    #[test]
    fn test_encode_instruction_xor_acc() {
        let instruction = Instruction::XorAcc(Register::R3);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0x3B]
        );
    }

    #[test]
    fn test_encode_instruction_cmp_acc() {
        let instruction = Instruction::CmpAcc(Register::R4);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0x44]
        );
    }

    #[test]
    fn test_encode_instruction_add_acc_immediate() {
        let instruction = Instruction::AddAccI(Operand::Immediate(0x1234));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
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
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0xC0, 0x68, 0x24]
        );
    }

    #[test]
    fn test_encode_instruction_sub_acc_immediate() {
        let instruction = Instruction::SubAccI(Operand::Immediate(0x00FF));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0xC1, 0xFF, 0x00]
        );
    }

    #[test]
    fn test_encode_instruction_and_acc_immediate() {
        let instruction = Instruction::AndAccI(Operand::Immediate(0x0F0F));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0xC2, 0x0F, 0x0F]
        );
    }

    #[test]
    fn test_encode_instruction_or_acc_immediate() {
        let instruction = Instruction::OrAccI(Operand::Immediate(0x8000));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0xC3, 0x00, 0x80]
        );
    }

    #[test]
    fn test_encode_instruction_xor_acc_immediate() {
        let instruction = Instruction::XorAccI(Operand::Immediate(0xAAAA));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0xC4, 0xAA, 0xAA]
        );
    }

    #[test]
    fn test_encode_instruction_cmp_acc_immediate() {
        let instruction = Instruction::CmpAccI(Operand::Immediate(0x0A0B));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0xC5, 0x0B, 0x0A]
        );
    }

    #[test]
    fn test_encode_instruction_adci_acc_immediate() {
        let instruction = Instruction::AdcAccI(Operand::Immediate(0xFFFF));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0xC6, 0xFF, 0xFF]
        );
    }

    #[test]
    fn test_encode_instruction_sbci_acc_immediate() {
        let instruction = Instruction::SbcAccI(Operand::Immediate(0x1234));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0xC7, 0x34, 0x12]
        );
    }

    #[test]
    fn test_encode_instruction_add_sp() {
        let instruction = Instruction::AddSp(-5);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0x6C, 0xFB]
        );
    }

    #[test]
    fn test_encode_instruction_push_reg() {
        let instruction = Instruction::Push(Register::R2);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0x6F]
        );
    }

    #[test]
    fn test_encode_instruction_push_immediate() {
        let instruction = Instruction::PushI(Operand::Immediate(0x1234));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
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
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0x7D, 0x57, 0x13]
        );
    }

    #[test]
    fn test_encode_instruction_push_f() {
        let instruction = Instruction::PushF;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0x7E]
        );
    }

    #[test]
    fn test_encode_instruction_pop_reg() {
        let instruction = Instruction::Pop(Register::R3);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0x78]
        );
    }

    #[test]
    fn test_encode_instruction_pop_f() {
        let instruction = Instruction::PopF;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0x7F]
        );
    }

    #[test]
    fn test_encode_instruction_neg() {
        let instruction = Instruction::NegAcc;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0x48]
        );
    }

    #[test]
    fn test_encode_instruction_not() {
        let instruction = Instruction::NotAcc;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0x49]
        );
    }

    #[test]
    fn test_encode_instruction_swap() {
        let instruction = Instruction::SwapAcc;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0x4A]
        );
    }

    #[test]
    fn test_encode_instruction_ccf() {
        let instruction = Instruction::Ccf;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0x4B]
        );
    }

    #[test]
    fn test_encode_instruction_scf() {
        let instruction = Instruction::Scf;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0x4C]
        );
    }

    #[test]
    fn test_encode_instruction_rcf() {
        let instruction = Instruction::Rcf;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0x4D]
        );
    }

    #[test]
    fn test_encode_instruction_enter() {
        let instruction = Instruction::Enter;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0x4F]
        );
    }

    #[test]
    fn test_encode_instruction_leave() {
        let instruction = Instruction::Leave;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0x50]
        );
    }

    #[test]
    fn test_encode_instruction_ret() {
        let instruction = Instruction::Ret;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0xF9]
        );
    }

    #[test]
    fn test_encode_instruction_reti() {
        let instruction = Instruction::Reti;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0xFA]
        );
    }

    #[test]
    fn test_encode_instruction_ei() {
        let instruction = Instruction::Ei;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0xFB]
        );
    }

    #[test]
    fn test_encode_instruction_di() {
        let instruction = Instruction::Di;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0xFC]
        );
    }

    #[test]
    fn test_encode_instruction_inc() {
        let instruction = Instruction::Inc(Register::R1);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0xE2]
        );
    }

    #[test]
    fn test_encode_instruction_dec() {
        let instruction = Instruction::Dec(Register::R2);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0xDB]
        );
    }

    #[test]
    fn test_encode_instruction_call_immediate() {
        let instruction = Instruction::CallI(Operand::Immediate(0x1234));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
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
            encode_instruction(&instruction, &symbol_table, &0x4100, &1, &0).unwrap(),
            vec![0xC8, 0x21, 0x43]
        );
    }

    #[test]
    fn test_encode_instruction_call_indirect() {
        let instruction = Instruction::CallIndirect(Register::R4);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0xCD] // 0xC9 + 4
        );
    }

    #[test]
    fn test_encode_instruction_callcc_immediate() {
        let instruction = Instruction::CallccI(ConditionCode::C, Operand::Immediate(0x1122));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
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
            encode_instruction(&instruction, &symbol_table, &0x4100, &1, &0).unwrap(),
            vec![0xD8, 0x21, 0x43] // 0xD1 + 7
        );
    }

    #[test]
    fn test_encode_instruction_syscall() {
        let instruction = Instruction::Syscall(0x1A);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0x4E, 0x1A]
        );
    }

    #[test]
    fn test_encode_instruction_ld_absolute() {
        let instruction = Instruction::LdAbs(Register::R1, Operand::AbsAddr(0x1234));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0xEA, 0x34, 0x12] // 0xE9 + 1
        );
    }

    #[test]
    fn test_encode_instruction_st_absolute() {
        let instruction = Instruction::StAbs(Operand::AbsAddr(0x4321), Register::R2);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0xF3, 0x21, 0x43] // 0xF1 + 2
        );
    }

    #[test]
    fn test_encode_instruction_st_indirect() {
        let instruction = Instruction::StIndirect(Register::R1, Register::R2);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0xFE, 0x4A] // 0x40 | (1 << 3) | 2
        );
    }

    #[test]
    fn test_encode_sra() {
        let instruction = Instruction::Sra(Register::R0);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0xFD, 0x00]
        );
    }

    #[test]
    fn test_encode_shl() {
        let instruction = Instruction::Shl(Register::R1);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0xFD, 0x09]
        );
    }

    #[test]
    fn test_encode_shr() {
        let instruction = Instruction::Shr(Register::R2);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0xFD, 0x12]
        );
    }

    #[test]
    fn test_encode_rol() {
        let instruction = Instruction::Rol(Register::R3);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0xFD, 0x1B]
        );
    }

    #[test]
    fn test_encode_ror() {
        let instruction = Instruction::Ror(Register::R4);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0xFD, 0x24]
        );
    }

    #[test]
    fn test_encode_instruction_add_b() {
        let instruction = Instruction::AddBAcc(Register::R1);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0xFD, 0x29] // 0x28 + 1
        );
    }

    #[test]
    fn test_encode_instruction_sub_b() {
        let instruction = Instruction::SubBAcc(Register::R2);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0xFD, 0x32] // 0x30 + 2
        );
    }

    #[test]
    fn test_encode_instruction_and_b() {
        let instruction = Instruction::AndBAcc(Register::R3);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0xFD, 0x3B] // 0x38 + 3
        );
    }

    #[test]
    fn test_encode_instruction_or_b() {
        let instruction = Instruction::OrBAcc(Register::R4);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0xFD, 0x44] // 0x40 + 4
        );
    }

    #[test]
    fn test_encode_instruction_xor_b() {
        let instruction = Instruction::XorBAcc(Register::R5);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0xFD, 0x4D] // 0x48 + 5
        );
    }

    #[test]
    fn test_encode_instruction_cmp_b() {
        let instruction = Instruction::CmpBAcc(Register::R6);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0xFD, 0x56] // 0x50 + 6
        );
    }

    #[test]
    fn test_encode_instruction_ldi_b() {
        let instruction = Instruction::LdiB(Register::R1, 0xAB);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0xFD, 0xA1, 0xAB] // 0xA0 + 1
        );
    }

    #[test]
    fn test_encode_instruction_bit_register() {
        let instruction = Instruction::BitReg(Register::R1, 7);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0xFD, 0x5F, 1]
        );
    }

    #[test]
    fn test_encode_instruction_set_absolute() {
        let instruction = Instruction::SetAbs(Operand::AbsAddr(0x1234), 0);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0xFD, 0x78, 0x34, 0x12]
        );
    }

    #[test]
    fn test_encode_instruction_res_indirect() {
        let instruction = Instruction::ResIndirect(Register::R2, 3);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0xFD, 0x9B, 2]
        );
    }

    #[test]
    fn test_encode_instruction_ld_indexed() {
        let instruction = Instruction::LdIndexed(Register::R0, Register::R1, 16);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0xFF, 0x01, 16]
        );
    }

    #[test]
    fn test_encode_instruction_st_indexed() {
        let instruction = Instruction::StIndexed(Register::R2, -1, Register::R3);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0xFF, 0x53, 255]
        );
    }

    #[test]
    fn test_encode_instruction_lea_indexed() {
        let instruction = Instruction::Lea(Register::R4, Register::R5, 32);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0xFF, 0xA5, 32]
        );
    }

    #[test]
    fn test_encode_instruction_ld_post_increment() {
        let instruction = Instruction::LdPostInc(Register::R6, Register::R7);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0xFF, 0xC7, 6]
        );
    }

    #[test]
    fn test_encode_instruction_st_post_increment() {
        let instruction = Instruction::StPostInc(Register::R0, Register::R1);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0xFF, 0xC9, 0]
        );
    }

    #[test]
    fn test_encode_instruction_ld_pre_decrement() {
        let instruction = Instruction::LdPreDec(Register::R2, Register::R3);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0xFF, 0xD3, 2]
        );
    }

    #[test]
    fn test_encode_instruction_st_pre_decrement() {
        let instruction = Instruction::StPreDec(Register::R4, Register::R5);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0xFF, 0xDD, 4]
        );
    }

    #[test]
    fn test_encode_instruction_ld_b_post_increment() {
        let instruction = Instruction::LdBPostInc(Register::R6, Register::R7);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0xFF, 0xE7, 6]
        );
    }

    #[test]
    fn test_encode_instruction_st_b_post_increment() {
        let instruction = Instruction::StBPostInc(Register::R0, Register::R1);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0xFF, 0xE9, 0]
        );
    }

    #[test]
    fn test_encode_instruction_ld_b_pre_decrement() {
        let instruction = Instruction::LdBPreDec(Register::R2, Register::R3);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
            vec![0xFF, 0xF3, 2]
        );
    }

    #[test]
    fn test_encode_instruction_st_b_pre_decrement() {
        let instruction = Instruction::StBPreDec(Register::R4, Register::R5);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, &0, &0).unwrap(),
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
            encode_instruction(&instruction, &symbol_table, &0x1100, &0, &0).unwrap(),
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
        let result = encode_instruction(&instruction, &symbol_table, &0x5432, &1, &1);
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
        let result = encode_instruction(&instruction, &symbol_table, &0x5432, &1, &1);
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
            encode_instruction(&instruction, &symbol_table, &0x1100, &0, &0).unwrap(),
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
        let result = encode_instruction(&instruction, &symbol_table, &0x5432, &1, &1);
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
        let result = encode_instruction(&instruction, &symbol_table, &0x5432, &1, &1);
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
        let result = encode_instruction(&instruction, &symbol_table, &0x5432, &1, &1);
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
