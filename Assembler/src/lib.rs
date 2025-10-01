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

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use file_reader::FileReader;
use std::collections::HashSet;

pub mod assembler;
pub mod ast;
pub mod errors;
pub mod file_reader;
pub mod parser;

extern crate pest;
extern crate pest_derive;

pub fn assemble<F: FileReader>(
    source_path: &Path,
    start_addr: u16,
    final_logical_addr: u16,
    reader: &F,
) -> Result<Vec<u8>> {
    let mut include_stack: HashSet<PathBuf> = HashSet::new();
    let mut parsed_lines = parser::parse_source_recursive(source_path, &mut include_stack, reader)
        .context("Failed during parsing stage")?;

    let constant_table = assembler::build_constant_table(&parsed_lines)
        .context("Failed during assembler phase 0")?;

    assembler::process_constants(&mut parsed_lines, &constant_table)
        .context("Failed during assembler phase 0.5")?;

    let symbol_table = assembler::build_symbol_table(
        &parsed_lines,
        &start_addr,
        &final_logical_addr,
        &constant_table,
    )
    .context("Failed during assembler phase 1")?;

    let machine_code = assembler::generate_bytecode(&parsed_lines, &symbol_table, &start_addr)
        .context("Failed during assembler phase 2")?;

    let mut final_rom = Vec::new();
    if start_addr > 0 {
        final_rom.resize(start_addr as usize, 0x00);
    }
    final_rom.extend(machine_code);

    Ok(final_rom)
}
