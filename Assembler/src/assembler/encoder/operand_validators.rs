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
use crate::ast::Operand;
use crate::errors::AssemblyError;

impl<'a> Encoder<'a> {
    pub fn expect_immediate(self, op: &Operand) -> Result<i32, AssemblyError> {
        if let Operand::Immediate(val) = op {
            Ok(*val)
        } else {
            Err(AssemblyError::StructuralError {
                line: *self.line_num,
                reason: "Expected an immediate.".to_string(),
            })
        }
    }
}
