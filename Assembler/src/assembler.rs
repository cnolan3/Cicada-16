use crate::ast::{AssemblyLine, Instruction, Operand, Register};
use crate::errors::AssemblyError;
use std::collections::HashMap;

// The symbol table stores label names and their calculated addresses.
type SymbolTable = HashMap<String, u16>;

/// Pass 1: Build the symbol table.
pub fn build_symbol_table(
    lines: &[AssemblyLine],
    start_addr: &u16,
) -> Result<SymbolTable, AssemblyError> {
    let mut symbol_table = SymbolTable::new();
    let mut current_address: u16 = start_addr.clone(); // Start address after cartridge header

    for line in lines {
        // If a label exists on this line, record its current address.
        if let Some(label) = &line.label {
            if symbol_table.contains_key(label) {
                return Err(AssemblyError::SemanticError {
                    line: line.line_number,
                    reason: format!("Duplicate label definition: {}", label),
                });
            }
            symbol_table.insert(label.clone(), current_address);
        }

        // Increment current_address by the size of the instruction.
        if let Some(instruction) = &line.instruction {
            current_address += calculate_instruction_size(instruction, line.line_number)?;
        }
    }
    Ok(symbol_table)
}

/// Helper function to determine instruction size in bytes during Pass 1.
fn calculate_instruction_size(
    instruction: &Instruction,
    line_num: usize,
) -> Result<u16, AssemblyError> {
    match instruction {
        Instruction::Nop | Instruction::Halt | Instruction::Ret => Ok(1),

        Instruction::Ld(Operand::Register(_), Operand::Register(_)) => {
            // Check if this form maps to the 1-byte LD rd, rs (opcodes 0x80-0xBF)
            Ok(1)
        }
        Instruction::Ld(_, Operand::Immediate(_)) => {
            // LDI r, n16 (3 bytes) or LDI.b r, n8 (3 bytes, prefixed)
            // For simplicity in early stages, assume LDI r, n16.
            Ok(3)
        }
        Instruction::Ld(_, Operand::Label(_)) => {
            // LD r, (n16) where n16 comes from a label. Size is 3 bytes.
            Ok(3)
        }
        Instruction::Jmp(Operand::Label(_)) | Instruction::Jmp(Operand::Immediate(_)) => Ok(3), // JMP n16
        Instruction::Jmp(Operand::Indirect(_)) => Ok(1),
        Instruction::Jr(Operand::Label(_)) => Ok(2), // JR n8s
        Instruction::Add(Operand::Register(_), Some(Operand::Register(_))) => Ok(2),
        Instruction::Add(Operand::Register(_), Some(Operand::Immediate(_))) => Ok(4),
        Instruction::Add(Operand::Register(_), None) => Ok(1),
        Instruction::Add(Operand::Immediate(_), None) => Ok(3),
        Instruction::Sub(Operand::Register(_), Some(Operand::Register(_))) => Ok(2),
        Instruction::Sub(Operand::Register(_), None) => Ok(1),
        Instruction::And(Operand::Register(_), Some(Operand::Register(_))) => Ok(2),
        Instruction::Or(Operand::Register(_), Some(Operand::Register(_))) => Ok(2),
        Instruction::Xor(Operand::Register(_), Some(Operand::Register(_))) => Ok(2),
        Instruction::Cmp(Operand::Register(_), Some(Operand::Register(_))) => Ok(2),
        Instruction::Adc(Operand::Register(_), Some(Operand::Register(_))) => Ok(2),
        Instruction::Sbc(Operand::Register(_), Some(Operand::Register(_))) => Ok(2),
        Instruction::And(Operand::Register(_), None) => Ok(1),
        Instruction::Or(Operand::Register(_), None) => Ok(1),
        Instruction::Xor(Operand::Register(_), None) => Ok(1),
        Instruction::Cmp(Operand::Register(_), None) => Ok(1),
        Instruction::Neg => Ok(1),
        Instruction::Not => Ok(1),
        Instruction::Swap => Ok(1),

        // ... add logic for every instruction variant based on your opcode map ...
        _ => Err(AssemblyError::SemanticError {
            line: line_num,
            reason: "Invalid Instruction.".to_string(),
        }),
    }
}

/// Pass 2: Generate machine code.
pub fn generate_bytecode(
    lines: &[AssemblyLine],
    symbol_table: &SymbolTable,
) -> Result<Vec<u8>, AssemblyError> {
    let mut bytecode = Vec::new();

    for line in lines {
        if let Some(instruction) = &line.instruction {
            let instruction_bytes = encode_instruction(instruction, symbol_table)?;
            bytecode.extend(instruction_bytes);
        }
    }
    Ok(bytecode)
}

// helper function to encode a register operand
fn encode_register_operand(reg: &Register) -> u8 {
    match reg {
        Register::R0 => 0,
        Register::R1 => 1,
        Register::R2 => 2,
        Register::R3 => 3,
        Register::R4 => 4,
        Register::R5 => 5,
        Register::R6 => 6,
        Register::R7 => 7,
    }
}

/// Helper function to translate a single instruction into bytes during Pass 2.
fn encode_instruction(
    instruction: &Instruction,
    symbol_table: &SymbolTable,
) -> Result<Vec<u8>, AssemblyError> {
    match instruction {
        // no op (0x00)
        Instruction::Nop => Ok(vec![0x00]),
        // Example: LDI R1, 0xABCD (Opcode: 0x01 + register index)
        Instruction::Ld(Operand::Register(reg), Operand::Immediate(value)) => {
            let base_opcode = 0x01;
            let reg_index = encode_register_operand(reg);
            let [low, high] = value.to_le_bytes();
            Ok(vec![base_opcode + reg_index, low, high])
        }
        // Example: JMP my_label
        Instruction::Jmp(Operand::Label(label_name)) => {
            let target_address =
                symbol_table
                    .get(label_name)
                    .ok_or_else(|| AssemblyError::SemanticErrorNoLine {
                        reason: format!("Undefined label: {}", label_name),
                    })?;
            let [low, high] = target_address.to_le_bytes();
            Ok(vec![0x51, low, high]) // Opcode for JMP n16
        }
        // jump address
        Instruction::Jmp(Operand::Immediate(addr)) => {
            let [low, high] = addr.to_le_bytes();
            Ok(vec![0x51, low, high]) // Opcode for JMP n16
        }
        // jump indirect
        Instruction::Jmp(Operand::Indirect(reg)) => {
            let reg_index = encode_register_operand(reg);
            Ok(vec![0x52 + reg_index]) // Opcode for JMP n16
        }
        // register-to-register load
        Instruction::Ld(Operand::Register(rd), Operand::Register(rs)) => {
            let rd_index = encode_register_operand(rd);
            let rs_index = encode_register_operand(rs);
            let opcode = 0x80 | ((rd_index & 0x07) << 3) | (rs_index & 0x07);
            Ok(vec![opcode])
        }
        // add reg to reg
        Instruction::Add(Operand::Register(rd), Some(Operand::Register(rs))) => {
            let rd_index = encode_register_operand(rd);
            let rs_index = encode_register_operand(rs);
            let byte0 = (rd_index << 3) | (rs_index & 0x07);
            Ok(vec![0x10, byte0])
        }
        // add immediate
        Instruction::Add(Operand::Register(rd), Some(Operand::Immediate(imm))) => {
            let rd_index = encode_register_operand(rd);
            let [low, high] = imm.to_le_bytes();
            Ok(vec![0x09, rd_index, low, high])
        }
        // add accumulator
        Instruction::Add(Operand::Register(rs), None) => {
            let base_opcode = 0x18;
            let rs_index = encode_register_operand(rs);
            Ok(vec![base_opcode + rs_index])
        }
        // sub reg to reg
        Instruction::Sub(Operand::Register(rd), Some(Operand::Register(rs))) => {
            let rd_index = encode_register_operand(rd);
            let rs_index = encode_register_operand(rs);
            let byte0 = (rd_index << 3) | (rs_index & 0x07);
            Ok(vec![0x11, byte0])
        }
        // sub accumulator
        Instruction::Sub(Operand::Register(rs), None) => {
            let base_opcode = 0x20;
            let rs_index = encode_register_operand(rs);
            Ok(vec![base_opcode + rs_index])
        }
        // and reg to reg
        Instruction::And(Operand::Register(rd), Some(Operand::Register(rs))) => {
            let rd_index = encode_register_operand(rd);
            let rs_index = encode_register_operand(rs);
            let byte0 = (rd_index << 3) | (rs_index & 0x07);
            Ok(vec![0x12, byte0])
        }
        // or reg to reg
        Instruction::Or(Operand::Register(rd), Some(Operand::Register(rs))) => {
            let rd_index = encode_register_operand(rd);
            let rs_index = encode_register_operand(rs);
            let byte0 = (rd_index << 3) | (rs_index & 0x07);
            Ok(vec![0x13, byte0])
        }
        // xor reg to reg
        Instruction::Xor(Operand::Register(rd), Some(Operand::Register(rs))) => {
            let rd_index = encode_register_operand(rd);
            let rs_index = encode_register_operand(rs);
            let byte0 = (rd_index << 3) | (rs_index & 0x07);
            Ok(vec![0x14, byte0])
        }
        // cmp reg to reg
        Instruction::Cmp(Operand::Register(rd), Some(Operand::Register(rs))) => {
            let rd_index = encode_register_operand(rd);
            let rs_index = encode_register_operand(rs);
            let byte0 = (rd_index << 3) | (rs_index & 0x07);
            Ok(vec![0x15, byte0])
        }
        // adc reg to reg
        Instruction::Adc(Operand::Register(rd), Some(Operand::Register(rs))) => {
            let rd_index = encode_register_operand(rd);
            let rs_index = encode_register_operand(rs);
            let byte0 = (rd_index << 3) | (rs_index & 0x07);
            Ok(vec![0x16, byte0])
        }
        // sbc reg to reg
        Instruction::Sbc(Operand::Register(rd), Some(Operand::Register(rs))) => {
            let rd_index = encode_register_operand(rd);
            let rs_index = encode_register_operand(rs);
            let byte0 = (rd_index << 3) | (rs_index & 0x07);
            Ok(vec![0x17, byte0])
        }
        // and accumulator
        Instruction::And(Operand::Register(rs), None) => {
            let base_opcode = 0x28;
            let rs_index = encode_register_operand(rs);
            Ok(vec![base_opcode + rs_index])
        }
        // or accumulator
        Instruction::Or(Operand::Register(rs), None) => {
            let base_opcode = 0x30;
            let rs_index = encode_register_operand(rs);
            Ok(vec![base_opcode + rs_index])
        }
        // xor accumulator
        Instruction::Xor(Operand::Register(rs), None) => {
            let base_opcode = 0x38;
            let rs_index = encode_register_operand(rs);
            Ok(vec![base_opcode + rs_index])
        }
        // cmp accumulator
        Instruction::Cmp(Operand::Register(rs), None) => {
            let base_opcode = 0x40;
            let rs_index = encode_register_operand(rs);
            Ok(vec![base_opcode + rs_index])
        }
        // neg
        Instruction::Neg => Ok(vec![0x48]),
        // not
        Instruction::Not => Ok(vec![0x49]),
        // swap
        Instruction::Swap => Ok(vec![0x4A]),
        // absolute-to-register load
        Instruction::Ld(Operand::Register(rd), Operand::Indirect(rs)) => {
            let rd_index = encode_register_operand(rd);
            let rs_index = encode_register_operand(rs);
            let sub_opcode = 0x00 | ((rd_index & 0x07) << 3) | (rs_index & 0x07);
            Ok(vec![0xFE, sub_opcode])
        }

        // ... add encoding logic for every instruction variant based on your opcode map ...
        _ => Err(AssemblyError::SemanticErrorNoLine {
            reason: "Invalid Instruction".to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_instruction_size_nop() {
        let instruction = Instruction::Nop;
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 1);
    }

    #[test]
    fn test_encode_instruction_nop() {
        let instruction = Instruction::Nop;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table).unwrap(),
            vec![0x00]
        );
    }

    #[test]
    fn test_calculate_instruction_size_sub_reg() {
        let instruction = Instruction::Sub(Operand::Register(Register::R1), None);
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 1);
    }

    #[test]
    fn test_encode_instruction_sub_reg() {
        let instruction = Instruction::Sub(Operand::Register(Register::R1), None);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table).unwrap(),
            vec![0x21]
        );
    }

    #[test]
    fn test_calculate_instruction_size_and_reg_reg() {
        let instruction = Instruction::And(
            Operand::Register(Register::R2),
            Some(Operand::Register(Register::R3)),
        );
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 2);
    }

    #[test]
    fn test_encode_instruction_and_reg_reg() {
        let instruction = Instruction::And(
            Operand::Register(Register::R2),
            Some(Operand::Register(Register::R3)),
        );
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table).unwrap(),
            vec![0x12, (2 << 3) | 3]
        );
    }

    #[test]
    fn test_calculate_instruction_size_or_reg_reg() {
        let instruction = Instruction::Or(
            Operand::Register(Register::R4),
            Some(Operand::Register(Register::R5)),
        );
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 2);
    }

    #[test]
    fn test_encode_instruction_or_reg_reg() {
        let instruction = Instruction::Or(
            Operand::Register(Register::R4),
            Some(Operand::Register(Register::R5)),
        );
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table).unwrap(),
            vec![0x13, (4 << 3) | 5]
        );
    }

    #[test]
    fn test_calculate_instruction_size_xor_reg_reg() {
        let instruction = Instruction::Xor(
            Operand::Register(Register::R6),
            Some(Operand::Register(Register::R7)),
        );
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 2);
    }

    #[test]
    fn test_encode_instruction_xor_reg_reg() {
        let instruction = Instruction::Xor(
            Operand::Register(Register::R6),
            Some(Operand::Register(Register::R7)),
        );
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table).unwrap(),
            vec![0x14, (6 << 3) | 7]
        );
    }

    #[test]
    fn test_calculate_instruction_size_cmp_reg_reg() {
        let instruction = Instruction::Cmp(
            Operand::Register(Register::R0),
            Some(Operand::Register(Register::R1)),
        );
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 2);
    }

    #[test]
    fn test_encode_instruction_cmp_reg_reg() {
        let instruction = Instruction::Cmp(
            Operand::Register(Register::R0),
            Some(Operand::Register(Register::R1)),
        );
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table).unwrap(),
            vec![0x15, (0 << 3) | 1]
        );
    }

    #[test]
    fn test_calculate_instruction_size_adc_reg_reg() {
        let instruction = Instruction::Adc(
            Operand::Register(Register::R2),
            Some(Operand::Register(Register::R3)),
        );
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 2);
    }

    #[test]
    fn test_encode_instruction_adc_reg_reg() {
        let instruction = Instruction::Adc(
            Operand::Register(Register::R2),
            Some(Operand::Register(Register::R3)),
        );
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table).unwrap(),
            vec![0x16, (2 << 3) | 3]
        );
    }

    #[test]
    fn test_calculate_instruction_size_sbc_reg_reg() {
        let instruction = Instruction::Sbc(
            Operand::Register(Register::R4),
            Some(Operand::Register(Register::R5)),
        );
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 2);
    }

    #[test]
    fn test_encode_instruction_sbc_reg_reg() {
        let instruction = Instruction::Sbc(
            Operand::Register(Register::R4),
            Some(Operand::Register(Register::R5)),
        );
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table).unwrap(),
            vec![0x17, (4 << 3) | 5]
        );
    }

    #[test]
    fn test_calculate_instruction_size_and_reg() {
        let instruction = Instruction::And(Operand::Register(Register::R1), None);
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 1);
    }

    #[test]
    fn test_encode_instruction_and_reg() {
        let instruction = Instruction::And(Operand::Register(Register::R1), None);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table).unwrap(),
            vec![0x29]
        );
    }

    #[test]
    fn test_calculate_instruction_size_or_reg() {
        let instruction = Instruction::Or(Operand::Register(Register::R2), None);
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 1);
    }

    #[test]
    fn test_encode_instruction_or_reg() {
        let instruction = Instruction::Or(Operand::Register(Register::R2), None);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table).unwrap(),
            vec![0x32]
        );
    }

    #[test]
    fn test_calculate_instruction_size_xor_reg() {
        let instruction = Instruction::Xor(Operand::Register(Register::R3), None);
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 1);
    }

    #[test]
    fn test_encode_instruction_xor_reg() {
        let instruction = Instruction::Xor(Operand::Register(Register::R3), None);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table).unwrap(),
            vec![0x3B]
        );
    }

    #[test]
    fn test_calculate_instruction_size_cmp_reg() {
        let instruction = Instruction::Cmp(Operand::Register(Register::R4), None);
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 1);
    }

    #[test]
    fn test_encode_instruction_cmp_reg() {
        let instruction = Instruction::Cmp(Operand::Register(Register::R4), None);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table).unwrap(),
            vec![0x44]
        );
    }

    #[test]
    fn test_calculate_instruction_size_neg() {
        let instruction = Instruction::Neg;
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 1);
    }

    #[test]
    fn test_encode_instruction_neg() {
        let instruction = Instruction::Neg;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table).unwrap(),
            vec![0x48]
        );
    }

    #[test]
    fn test_calculate_instruction_size_not() {
        let instruction = Instruction::Not;
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 1);
    }

    #[test]
    fn test_encode_instruction_not() {
        let instruction = Instruction::Not;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table).unwrap(),
            vec![0x49]
        );
    }

    #[test]
    fn test_calculate_instruction_size_swap() {
        let instruction = Instruction::Swap;
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 1);
    }

    #[test]
    fn test_encode_instruction_swap() {
        let instruction = Instruction::Swap;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table).unwrap(),
            vec![0x4A]
        );
    }
}
