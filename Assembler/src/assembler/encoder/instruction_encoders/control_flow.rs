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
use crate::assembler::encoder::instruction_encoders::load_store::encode_ldi_data;
use crate::assembler::encoder::utility_functions::*;
use crate::assembler::symbol_table::{Symbol, get_and_check_symbol, get_symbol};
use crate::ast::{ConditionCode, Operand, Register};
use crate::errors::AssemblyError;

impl<'a> Encoder<'a> {
    pub fn encode_jmp_imm(self, op: &Operand) -> Result<Vec<u8>, AssemblyError> {
        let addr = resolve_label_or_immediate(op, self.symbol_table, self.line_num)?;
        let [low, high] = addr.to_le_bytes();
        Ok(vec![JMP_IMM_OPCODE, low, high])
    }

    pub fn encode_jmp_indirect(self, reg: &Register) -> Result<Vec<u8>, AssemblyError> {
        Ok(vec![encode_reg_opcode(JMP_INDIR_BASE_OPCODE, reg)])
    }

    pub fn encode_jr(self, op: &Operand) -> Result<Vec<u8>, AssemblyError> {
        let rel = match op {
            Operand::Immediate(imm) => *imm as i8,
            Operand::Label(label_name) => {
                let target_symbol = get_and_check_symbol(
                    self.symbol_table,
                    label_name,
                    self.line_num,
                    self.current_bank,
                )?;
                let rel: i32 = target_symbol.logical_address as i32 - *self.logical_address as i32;
                if rel > i8::MAX as i32 || rel < i8::MIN as i32 {
                    return Err(AssemblyError::SemanticError {
                        line: *self.line_num,
                        reason: format!(
                            "Label \"{}\" too far away for relative jump, must be within {} bytes of JR instruction",
                            label_name,
                            i8::MAX
                        ),
                    });
                }
                rel as i8
            }
            _ => unreachable!(),
        };
        Ok(vec![JR_OPCODE, rel as u8])
    }

    pub fn encode_jcc(self, cc: &ConditionCode, op: &Operand) -> Result<Vec<u8>, AssemblyError> {
        let addr = resolve_label_or_immediate(op, self.symbol_table, self.line_num)?;
        let opcode = encode_condition_code_opcode(JCC_BASE_OPCODE, cc);
        let [low, high] = addr.to_le_bytes();
        Ok(vec![opcode, low, high])
    }

    pub fn encode_jrcc(self, cc: &ConditionCode, op: &Operand) -> Result<Vec<u8>, AssemblyError> {
        let rel = match op {
            Operand::Immediate(imm) => *imm as i8,
            Operand::Label(label_name) => {
                let target_symbol = get_and_check_symbol(
                    self.symbol_table,
                    label_name,
                    self.line_num,
                    self.current_bank,
                )?;
                let rel: i32 = target_symbol.logical_address as i32 - *self.logical_address as i32;
                if rel > i8::MAX as i32 || rel < i8::MIN as i32 {
                    return Err(AssemblyError::SemanticError {
                        line: *self.line_num,
                        reason: format!(
                            "Label \"{}\" too far away for relative jump, must be within {} bytes of JRcc instruction",
                            label_name,
                            i8::MAX
                        ),
                    });
                }
                rel as i8
            }
            _ => unreachable!(),
        };
        let opcode = encode_condition_code_opcode(JRCC_BASE_OPCODE, cc);
        Ok(vec![opcode, rel as u8])
    }

    pub fn encode_djnz(self, op: &Operand) -> Result<Vec<u8>, AssemblyError> {
        let rel = match op {
            Operand::Immediate(imm) => *imm as i8,
            Operand::Label(label_name) => {
                let target_symbol = get_and_check_symbol(
                    self.symbol_table,
                    label_name,
                    self.line_num,
                    self.current_bank,
                )?;
                let rel: i32 = target_symbol.logical_address as i32 - *self.logical_address as i32;
                if rel > i8::MAX as i32 || rel < i8::MIN as i32 {
                    return Err(AssemblyError::SemanticError {
                        line: *self.line_num,
                        reason: format!(
                            "Label \"{}\" too far away for relative jump, must be within {} bytes of DJNZ instruction",
                            label_name,
                            i8::MAX
                        ),
                    });
                }
                rel as i8
            }
            _ => unreachable!(),
        };
        Ok(vec![DJNZ_OPCODE, rel as u8])
    }

    pub fn encode_call_imm(self, op: &Operand) -> Result<Vec<u8>, AssemblyError> {
        let addr = resolve_label_or_immediate(op, self.symbol_table, self.line_num)?;
        Ok(encode_call_immediate_data(addr))
    }

    pub fn encode_call_indirect(self, reg: &Register) -> Result<Vec<u8>, AssemblyError> {
        Ok(vec![encode_reg_opcode(CALL_INDIR_BASE_OPCODE, reg)])
    }

    pub fn encode_callcc(self, cc: &ConditionCode, op: &Operand) -> Result<Vec<u8>, AssemblyError> {
        let addr = resolve_label_or_immediate(op, self.symbol_table, self.line_num)?;
        let opcode = encode_condition_code_opcode(CALLCC_BASE_OPCODE, cc);
        let [low, high] = addr.to_le_bytes();
        Ok(vec![opcode, low, high])
    }

    pub fn encode_syscall_imm(self, imm: &Operand) -> Result<Vec<u8>, AssemblyError> {
        let val = self.expect_immediate(imm)?;
        Ok(encode_syscall_data(val as u8))
    }

    pub fn encode_call_far(self, call_label: &String) -> Result<Vec<u8>, AssemblyError> {
        let target_symbol = get_symbol(self.symbol_table, call_label, self.line_num)?;

        if target_symbol.bank == 0 {
            return Err(AssemblyError::SemanticError {
                line: *self.line_num,
                reason: format!(
                    "Label \"{}\" exists in bank 0, use a normal CALL instruction instead.",
                    call_label
                ),
            });
        } else if target_symbol.bank == *self.current_bank {
            return Err(AssemblyError::SemanticError {
                line: *self.line_num,
                reason: format!(
                    "Label \"{}\" exists in the same bank as the CALL.far instruction, use a normal CALL instruction instead.",
                    call_label
                ),
            });
        }

        Ok(encode_far_data(target_symbol, 0x21))
    }

    pub fn encode_call_far_via(
        self,
        call_label: &String,
        via_label: &String,
    ) -> Result<Vec<u8>, AssemblyError> {
        let call_symbol = get_symbol(self.symbol_table, call_label, self.line_num)?;
        let via_symbol = get_symbol(self.symbol_table, via_label, self.line_num)?;

        if call_symbol.bank == 0 {
            return Err(AssemblyError::SemanticError {
                line: *self.line_num,
                reason: format!(
                    "Label \"{}\" exists in bank 0, use a normal CALL instruction instead.",
                    call_label
                ),
            });
        } else if call_symbol.bank == *self.current_bank {
            return Err(AssemblyError::SemanticError {
                line: *self.line_num,
                reason: format!(
                    "Label \"{}\" exists in the same bank as the CALL.far instruction, use a normal CALL instruction instead.",
                    call_label
                ),
            });
        }

        if via_symbol.bank != 0 {
            return Err(AssemblyError::SemanticError {
                line: *self.line_num,
                reason: format!(
                    "Custom CALL.far via trampoline label must exist in bank 0, \"{}\" found in bank {}",
                    via_label, via_symbol.bank
                ),
            });
        }

        Ok(encode_far_via_data(call_symbol, via_symbol))
    }

    pub fn encode_jmp_far(self, label_name: &String) -> Result<Vec<u8>, AssemblyError> {
        let target_symbol = get_symbol(self.symbol_table, label_name, self.line_num)?;

        if target_symbol.bank == 0 {
            return Err(AssemblyError::SemanticError {
                line: *self.line_num,
                reason: format!(
                    "Label \"{}\" exists in bank 0, use a normal JMP instruction instead.",
                    label_name
                ),
            });
        } else if target_symbol.bank == *self.current_bank {
            return Err(AssemblyError::SemanticError {
                line: *self.line_num,
                reason: format!(
                    "Label \"{}\" exists in the same bank as the JMP.far instruction, use a normal JMP instruction instead.",
                    label_name
                ),
            });
        }

        Ok(encode_far_data(target_symbol, 0x22))
    }

    pub fn encode_jmp_far_via(
        self,
        call_label: &String,
        via_label: &String,
    ) -> Result<Vec<u8>, AssemblyError> {
        let call_symbol = get_symbol(self.symbol_table, call_label, self.line_num)?;
        let via_symbol = get_symbol(self.symbol_table, via_label, self.line_num)?;

        if call_symbol.bank == 0 {
            return Err(AssemblyError::SemanticError {
                line: *self.line_num,
                reason: format!(
                    "Label \"{}\" exists in bank 0, use a normal JMP instruction instead.",
                    call_label
                ),
            });
        } else if call_symbol.bank == *self.current_bank {
            return Err(AssemblyError::SemanticError {
                line: *self.line_num,
                reason: format!(
                    "Label \"{}\" exists in the same bank as the JMP.far instruction, use a normal JMP instruction instead.",
                    call_label
                ),
            });
        }

        if via_symbol.bank != 0 {
            return Err(AssemblyError::SemanticError {
                line: *self.line_num,
                reason: format!(
                    "Custom JMP.far via trampoline label must exist in bank 0, \"{}\" found in bank {}",
                    via_label, via_symbol.bank
                ),
            });
        }

        Ok(encode_far_via_data(call_symbol, via_symbol))
    }
}

// ------- reusable control flow encoding functions -------

fn encode_syscall_data(index: u8) -> Vec<u8> {
    vec![SYSCALL_OPCODE, index]
}

fn encode_call_immediate_data(logical_addr: u16) -> Vec<u8> {
    let [low, high] = logical_addr.to_le_bytes();
    vec![CALL_IMM_OPCODE, low, high]
}

fn encode_far_data(call_symbol: &Symbol, syslib_index: u8) -> Vec<u8> {
    let mut bytecode = Vec::new();
    bytecode.extend(encode_ldi_data(&Register::R4, call_symbol.bank as u16));
    bytecode.extend(encode_ldi_data(
        &Register::R5,
        call_symbol.logical_address as u16,
    ));
    bytecode.extend(encode_syscall_data(syslib_index));
    bytecode
}

fn encode_far_via_data(call_symbol: &Symbol, via_symbol: &Symbol) -> Vec<u8> {
    let mut bytecode = Vec::new();
    bytecode.extend(encode_ldi_data(&Register::R4, call_symbol.bank as u16));
    bytecode.extend(encode_ldi_data(
        &Register::R5,
        call_symbol.logical_address as u16,
    ));
    bytecode.extend(encode_call_immediate_data(
        via_symbol.logical_address as u16,
    ));
    bytecode
}
