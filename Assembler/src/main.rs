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

use anyhow::Result;
use cicasm::assemble;
use cicasm::file_reader::AsmFileReader;
use clap::Parser as clap_parser;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

#[derive(clap_parser)]
#[clap(version = "0.3.7", author = "Connor Nolan")]
struct Opts {
    /// Input file to assemble
    input: PathBuf,

    /// Assembled output file path (optional, default: ./assembled.bin)
    #[clap(short, long)]
    output: Option<PathBuf>,

    /// Assemble program as a boot ROM (no header, starts at 0x0000, 16 KiB max, must have
    /// interrupt vector table at 0x3FE0-0x3FFF)
    #[clap(short, long)]
    boot: bool,
}

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    let mut final_logical_addr: u16 = 0x7FFF;
    let mut expected_interrupt_table_addr: Option<u16> = Some(0x0060);
    let mut expected_header_addr: Option<u16> = Some(0x0000);
    let mut output_path: PathBuf = env::current_dir()?;
    output_path.push("assembled.bin");

    if let Some(out) = opts.output {
        output_path = out;
    }

    if opts.boot {
        final_logical_addr = 0x3FFF;
        expected_interrupt_table_addr = Some(0x3FE0);
        expected_header_addr = None;
    }

    let reader = AsmFileReader;
    let input_path: &Path = Path::new(&opts.input);

    let final_rom = assemble(
        input_path,
        final_logical_addr,
        expected_interrupt_table_addr,
        expected_header_addr,
        &reader,
    )?;

    fs::write(&output_path, final_rom)?;

    println!(
        "Successfully assembled {} to {}",
        opts.input.display(),
        output_path.display()
    );

    Ok(())
}
