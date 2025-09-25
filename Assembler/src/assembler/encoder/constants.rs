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

// instruction opcodes
pub const NOP_OPCODE: u8 = 0x0;
pub const LDI_BASE_OPCODE: u8 = 0x01;
pub const ADDI_OPCODE: u8 = 0x09;
pub const SUBI_OPCODE: u8 = 0x0A;
pub const ANDI_OPCODE: u8 = 0x0B;
pub const ORI_OPCODE: u8 = 0x0C;
pub const XORI_OPCODE: u8 = 0x0D;
pub const CMPI_OPCODE: u8 = 0x0E;
pub const HALT_OPCODE: u8 = 0x0F;
pub const ADD_OPCODE: u8 = 0x10;
pub const SUB_OPCODE: u8 = 0x11;
pub const AND_OPCODE: u8 = 0x12;
pub const OR_OPCODE: u8 = 0x13;
pub const XOR_OPCODE: u8 = 0x14;
pub const CMP_OPCODE: u8 = 0x15;
pub const ADC_OPCODE: u8 = 0x16;
pub const SBC_OPCODE: u8 = 0x17;
pub const ADD_ACC_BASE_OPCODE: u8 = 0x18;
pub const SUB_ACC_BASE_OPCODE: u8 = 0x20;
pub const AND_ACC_BASE_OPCODE: u8 = 0x28;
pub const OR_ACC_BASE_OPCODE: u8 = 0x30;
pub const XOR_ACC_BASE_OPCODE: u8 = 0x38;
pub const CMP_ACC_BASE_OPCODE: u8 = 0x40;
pub const NEG_ACC_OPCODE: u8 = 0x48;
pub const NOR_ACC_OPCODE: u8 = 0x49;
pub const SWAP_ACC_OPCODE: u8 = 0x4A;
pub const CCF_OPCODE: u8 = 0x4B;
pub const SCF_OPCODE: u8 = 0x4C;
pub const RCF_OPCODE: u8 = 0x4D;
pub const SYSCALL_OPCODE: u8 = 0x4E;
pub const ENTER_OPCODE: u8 = 0x4F;
pub const LEAVE_OPCODE: u8 = 0x50;
pub const JMP_IMM_OPCODE: u8 = 0x51;
pub const JMP_INDIR_BASE_OPCODE: u8 = 0x52;
pub const JR_OPCODE: u8 = 0x5A;
pub const JCC_BASE_OPCODE: u8 = 0x5B;
pub const JRCC_BASE_OPCODE: u8 = 0x63;
pub const DJNZ_OPCODE: u8 = 0x6B;
pub const ADD_SP_OPCODE: u8 = 0x6C;
pub const PUSH_REG_BASE_OPCODE: u8 = 0x6D;
pub const POP_REG_BASE_OPCODE: u8 = 0x75;
pub const PUSH_IMM_OPCODE: u8 = 0x7D;
pub const PUSH_F_OPCODE: u8 = 0x7E;
pub const POP_F_OPCODE: u8 = 0x7F;
pub const LD_REG_REG_BASE_OPCODE: u8 = 0x80;
pub const ADDI_ACC_OPCODE: u8 = 0xC0;
pub const SUBI_ACC_OPCODE: u8 = 0xC1;
pub const ANDI_ACC_OPCODE: u8 = 0xC2;
pub const ORI_ACC_OPCODE: u8 = 0xC3;
pub const XORI_ACC_OPCODE: u8 = 0xC4;
pub const CMPI_ACC_OPCODE: u8 = 0xC5;
pub const ADCI_ACC_OPCODE: u8 = 0xC6;
pub const SBCI_ACC_OPCODE: u8 = 0xC7;
pub const CALL_IMM_OPCODE: u8 = 0xC8;
pub const CALL_INDIR_BASE_OPCODE: u8 = 0xC9;
pub const CALLCC_BASE_OPCODE: u8 = 0xD1;
pub const DEC_BASE_OPCODE: u8 = 0xD9;
pub const INC_BASE_OPCODE: u8 = 0xE1;
pub const LD_ABS_BASE_OPCODE: u8 = 0xE9;
pub const ST_ABS_BASE_OPCODE: u8 = 0xF1;
pub const RET_OPCODE: u8 = 0xF9;
pub const RETI_OPCODE: u8 = 0xFA;
pub const EI_OPCODE: u8 = 0xFB;
pub const DI_OPCODE: u8 = 0xFC;
pub const FD_PREFIX: u8 = 0xFD;
pub const FE_PREFIX: u8 = 0xFE;
pub const FF_PREFIX: u8 = 0xFF;

pub const SRA_BASE_SUB_OPCODE: u8 = 0x00;
pub const SHL_BASE_SUB_OPCODE: u8 = 0x08;
pub const SHR_BASE_SUB_OPCODE: u8 = 0x10;
pub const ROL_BASE_SUB_OPCODE: u8 = 0x18;
pub const ROR_BASE_SUB_OPCODE: u8 = 0x20;
pub const ADDB_BASE_SUB_OPCODE: u8 = 0x28;
pub const SUBB_BASE_SUB_OPCODE: u8 = 0x30;
pub const ANDB_BASE_SUB_OPCODE: u8 = 0x38;
pub const ORB_BASE_SUB_OPCODE: u8 = 0x40;
pub const XORB_BASE_SUB_OPCODE: u8 = 0x48;
pub const CMPB_BASE_SUB_OPCODE: u8 = 0x50;
pub const BIT_REG_BASE_SUB_OPCODE: u8 = 0x58;
pub const SET_REG_BASE_SUB_OPCODE: u8 = 0x60;
pub const RES_REG_BASE_SUB_OPCODE: u8 = 0x68;
pub const BIT_ABS_BASE_SUB_OPCODE: u8 = 0x70;
pub const SET_ABS_BASE_SUB_OPCODE: u8 = 0x78;
pub const RES_ABS_BASE_SUB_OPCODE: u8 = 0x80;
pub const BIT_INDIR_BASE_SUB_OPCODE: u8 = 0x88;
pub const SET_INDIR_BASE_SUB_OPCODE: u8 = 0x90;
pub const RES_INDIR_BASE_SUB_OPCODE: u8 = 0x98;
pub const LDIB_BASE_SUB_OPCODE: u8 = 0xA0;

pub const LD_INDIR_BASE_SUB_OPCODE: u8 = 0x00;
pub const ST_INDIR_BASE_SUB_OPCODE: u8 = 0x40;
pub const LDB_INDIR_BASE_SUB_OPCODE: u8 = 0x80;
pub const STB_INDIR_BASE_SUB_OPCODE: u8 = 0xC0;

pub const LD_INDEX_BASE_SUB_OPCODE: u8 = 0x00;
pub const ST_INDEX_BASE_SUB_OPCODE: u8 = 0x40;
pub const LEA_BASE_SUB_OPCODE: u8 = 0x80;
pub const LD_POST_INC_BASE_SUB_OPCODE: u8 = 0xC0;
pub const ST_POST_INC_BASE_SUB_OPCODE: u8 = 0xC8;
pub const LD_PRE_DEC_BASE_SUB_OPCODE: u8 = 0xD0;
pub const ST_PRE_DEC_BASE_SUB_OPCODE: u8 = 0xD8;
pub const LDB_POST_INC_BASE_SUB_OPCODE: u8 = 0xE0;
pub const STB_POST_INC_BASE_SUB_OPCODE: u8 = 0xE8;
pub const LDB_PRE_DEC_BASE_SUB_OPCODE: u8 = 0xF0;
pub const STB_PRE_DEC_BASE_SUB_OPCODE: u8 = 0xF8;
