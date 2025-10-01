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
use clap::Subcommand;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

#[derive(clap_parser)]
#[clap(version = "0.3.4", author = "Connor Nolan")]
struct Opts {
    #[clap(short, long)]
    input: PathBuf,
    #[clap(short, long)]
    output: PathBuf,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Assemble program as a boot ROM (no header, starts at 0x0000, 16 KiB max, must have
    /// interrupt vector table at 0x3FE0-0x3FFF)
    Boot,
}

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    let mut start_addr: u16 = 0x0100;
    let mut final_logical_addr: u16 = 0x7FFF;

    match &opts.command {
        Some(Commands::Boot) => {
            start_addr = 0x0000;
            final_logical_addr = 0x3FFF;
        }
        None => {}
    }

    // let source_code = fs::read_to_string(&opts.input)
    //     .with_context(|| format!("Failed to read input file: {}", opts.input.display()))?;

    let reader = AsmFileReader;
    let input_path: &Path = Path::new(&opts.input);

    let final_rom = assemble(input_path, start_addr, final_logical_addr, &reader)?;

    fs::write(&opts.output, final_rom)?;
    println!(
        "Successfully assembled {} to {}",
        opts.input.display(),
        opts.output.display()
    );

    Ok(())
}
