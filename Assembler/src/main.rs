use clap::Parser as clap_parser;
use std::fs;
use std::path::PathBuf;

// These modules are now pest-specific or unchanged
mod assembler;
mod ast;
mod parser;

// pest needs this to be accessible
extern crate pest;
#[macro_use]
extern crate pest_derive;

#[derive(clap_parser)]
#[clap(version = "0.1", author = "Your Name")]
struct Opts {
    #[clap(short, long)]
    input: PathBuf,
    #[clap(short, long)]
    output: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: Opts = Opts::parse();

    // 1. Read source file
    let source_code = fs::read_to_string(&opts.input)?;

    // --- Parse Stage (Now uses pest) ---
    // The new parser processes the whole file at once.
    let parsed_lines =
        parser::parse_source(&source_code).map_err(|e| format!("Parsing error:\n{}", e))?;

    // --- Assembly Stage (UNCHANGED) ---
    // Pass 1: Build symbol table
    println!("Running pass 1...");
    let start_addr: u16 = 0x0100;
    let symbol_table = assembler::build_symbol_table(&parsed_lines, &start_addr)
        .map_err(|e| format!("Pass 1 error: {}", e))?;
    println!("Symbol table built: {:?}", symbol_table);

    // Pass 2: Generate bytecode
    println!("Running pass 2...");
    let machine_code = assembler::generate_bytecode(&parsed_lines, &symbol_table)
        .map_err(|e| format!("Pass 2 error: {}", e))?;

    // --- Output Stage (UNCHANGED) ---
    let mut final_rom = Vec::new();
    final_rom.resize(start_addr as usize, 0x00); // Pad header area
    final_rom.extend(machine_code);

    fs::write(&opts.output, final_rom)?;
    println!(
        "Successfully assembled {} to {}",
        opts.input.display(),
        opts.output.display()
    );

    Ok(())
}
