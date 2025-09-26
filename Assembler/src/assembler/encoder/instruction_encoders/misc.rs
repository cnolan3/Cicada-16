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
use crate::errors::AssemblyError;

impl<'a> Encoder<'a> {
    pub fn encode_nop(self) -> Result<Vec<u8>, AssemblyError> {
        Ok(vec![NOP_OPCODE])
    }

    pub fn encode_halt(self) -> Result<Vec<u8>, AssemblyError> {
        Ok(vec![HALT_OPCODE])
    }

    pub fn encode_ccf(self) -> Result<Vec<u8>, AssemblyError> {
        Ok(vec![CCF_OPCODE])
    }

    pub fn encode_scf(self) -> Result<Vec<u8>, AssemblyError> {
        Ok(vec![SCF_OPCODE])
    }

    pub fn encode_rcf(self) -> Result<Vec<u8>, AssemblyError> {
        Ok(vec![RCF_OPCODE])
    }

    pub fn encode_enter(self) -> Result<Vec<u8>, AssemblyError> {
        Ok(vec![ENTER_OPCODE])
    }

    pub fn encode_leave(self) -> Result<Vec<u8>, AssemblyError> {
        Ok(vec![LEAVE_OPCODE])
    }

    pub fn encode_ret(self) -> Result<Vec<u8>, AssemblyError> {
        Ok(vec![RET_OPCODE])
    }

    pub fn encode_reti(self) -> Result<Vec<u8>, AssemblyError> {
        Ok(vec![RETI_OPCODE])
    }

    pub fn encode_ei(self) -> Result<Vec<u8>, AssemblyError> {
        Ok(vec![EI_OPCODE])
    }

    pub fn encode_di(self) -> Result<Vec<u8>, AssemblyError> {
        Ok(vec![DI_OPCODE])
    }
}
