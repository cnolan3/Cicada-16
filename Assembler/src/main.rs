use anyhow::{Context, Result};
use clap::Parser as clap_parser;
use clap::Subcommand;
use std::fs;
use std::path::PathBuf;

// These modules are now pest-specific or unchanged
mod assembler;
mod ast;
mod errors;
mod parser;

// pest needs this to be accessible
extern crate pest;
extern crate pest_derive;

#[derive(clap_parser)]
#[clap(version = "0.1", author = "Your Name")]
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
    // let mut max_rom_size: usize = 4_210_688; // max allowable size of the rom in bytes

    match &opts.command {
        Some(Commands::Boot) => {
            start_addr = 0x0000;
            // max_rom_size = 16_384;
        }
        None => {}
    }

    // 1. Read source file
    let source_code = fs::read_to_string(&opts.input)
        .with_context(|| format!("Failed to read input file: {}", opts.input.display()))?;

    // --- Parse Stage (Now uses pest) ---
    // The new parser processes the whole file at once.
    let parsed_lines = parser::parse_source(&source_code).context("Failed during parsing stage")?;

    // --- Assembly Stage (UNCHANGED) ---
    // Pass 1: Build symbol table
    println!("Running pass 1...");
    let symbol_table = assembler::build_symbol_table(&parsed_lines, &start_addr)
        .context("Failed during assembler phase 1")?;
    println!("Symbol table built: {:?}", symbol_table);

    // Pass 2: Generate bytecode
    println!("Running pass 2...");
    let machine_code = assembler::generate_bytecode(&parsed_lines, &symbol_table, &start_addr)
        .context("Failed during assembler phase 2")?;

    // --- Output Stage (UNCHANGED) ---
    let mut final_rom = Vec::new();
    final_rom.resize(start_addr as usize, 0x00); // Pad header area
    final_rom.extend(machine_code);

    // if final_rom.len() > max_rom_size {
    //     let msg = format!(
    //         "Assembled ROM is over the size threshold.\nROM size: {}\nlimit: {}",
    //         final_rom.len(),
    //         max_rom_size
    //     );
    //     Err(msg.into())
    // } else {
    //     fs::write(&opts.output, final_rom)?;
    //     println!(
    //         "Successfully assembled {} to {}",
    //         opts.input.display(),
    //         opts.output.display()
    //     );
    //
    //     Ok(())
    // }

    fs::write(&opts.output, final_rom)?;
    println!(
        "Successfully assembled {} to {}",
        opts.input.display(),
        opts.output.display()
    );

    Ok(())
}
