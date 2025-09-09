use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

mod assembler;
mod ast;
mod parser; // Modules we defined above

#[derive(Parser)]
#[clap(version = "0.1", author = "Your Name")]
struct Opts {
    /// Input assembly file (.asm)
    #[clap(short, long)]
    input: PathBuf,

    /// Output ROM file (.rom)
    #[clap(short, long)]
    output: PathBuf,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Boot,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: Opts = Opts::parse();

    // 1. Read source file
    let source_code = fs::read_to_string(&opts.input)?;

    // --- Parse Stage ---
    // Simple line-by-line parsing. A real implementation would handle multi-line statements.
    let mut parsed_lines = Vec::new();
    for line in source_code.lines() {
        if line.trim().is_empty() || line.trim().starts_with(';') {
            continue; // Skip empty lines and comments
        }
        // TODO: Proper error handling for parse failures
        let (_, assembly_line) = parser::parse_line(line).expect("Parsing failed");
        parsed_lines.push(assembly_line);
    }

    // --- Assembly Stage ---
    // Pass 1: Build symbol table
    println!("Running pass 1...");
    let symbol_table =
        assembler::build_symbol_table(&parsed_lines).map_err(|e| format!("Pass 1 error: {}", e))?;
    println!("Symbol table built: {:?}", symbol_table);

    // Pass 2: Generate bytecode
    println!("Running pass 2...");
    let machine_code = assembler::generate_bytecode(&parsed_lines, &symbol_table)
        .map_err(|e| format!("Pass 2 error: {}", e))?;

    // --- Output Stage ---
    // TODO: Generate cartridge header here first.
    let mut final_rom = Vec::new();
    // let header = create_cartridge_header(&machine_code);
    // final_rom.extend(header);

    // For now, just write code starting at 0x0100. Pad to get there.
    final_rom.resize(0x100, 0x00); // Pad header area with zeros
    final_rom.extend(machine_code);

    // Write to output file
    fs::write(&opts.output, final_rom)?;
    println!(
        "Successfully assembled {} to {}",
        opts.input.display(),
        opts.output.display()
    );

    Ok(())
}
