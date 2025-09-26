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

use crate::errors::AssemblyError;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct Symbol {
    pub logical_address: u32,
    pub bank: u32,
}

// The symbol table stores label names and their calculated addresses.
pub type SymbolTable = HashMap<String, Symbol>;

pub fn get_symbol<'a>(
    symbol_table: &'a SymbolTable,
    label_name: &String,
    line_num: &usize,
) -> Result<&'a Symbol, AssemblyError> {
    let target_symbol =
        symbol_table
            .get(label_name)
            .ok_or_else(|| AssemblyError::SemanticError {
                line: *line_num,
                reason: format!("Undefined label: {}", label_name),
            })?;

    Ok(target_symbol)
}

pub fn get_and_check_symbol<'a>(
    symbol_table: &'a SymbolTable,
    label_name: &String,
    line_num: &usize,
    current_bank: &u32,
) -> Result<&'a Symbol, AssemblyError> {
    let target_symbol = get_symbol(symbol_table, label_name, line_num)?;

    if target_symbol.bank != *current_bank {
        return Err(AssemblyError::SemanticError {
            line: *line_num,
            reason: format!(
                "Label \"{}\" exists in a different bank than the current instruction.",
                label_name
            ),
        });
    }

    Ok(target_symbol)
}
