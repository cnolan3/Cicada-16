use crate::ast::{AssemblyLine, ConditionCode, Instruction, Operand, Register};
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
        Instruction::Nop
        | Instruction::Halt
        | Instruction::Ret
        | Instruction::Ccf
        | Instruction::Scf
        | Instruction::Rcf
        | Instruction::Enter
        | Instruction::Leave
        | Instruction::Reti
        | Instruction::Ei
        | Instruction::Di
        | Instruction::Inc(_)
        | Instruction::Dec(_) => Ok(1),
        Instruction::St(Operand::Absolute(_), Operand::Register(_)) => Ok(3),
        Instruction::St(Operand::Indirect(_), Operand::Register(_)) => Ok(2),
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
        Instruction::Ld(Operand::Register(_), Operand::Absolute(_)) => Ok(3),
        Instruction::Jmp(Operand::Label(_)) | Instruction::Jmp(Operand::Immediate(_)) => Ok(3), // JMP n16
        Instruction::Jmp(Operand::Indirect(_)) => Ok(1),
        Instruction::Jr(Operand::Label(_)) | Instruction::Jr(Operand::Immediate(_)) => Ok(2), // JR n8s
        Instruction::Jcc(_, Operand::Label(_)) | Instruction::Jcc(_, Operand::Immediate(_)) => {
            Ok(3)
        }
        Instruction::Jrcc(_, Operand::Label(_)) | Instruction::Jrcc(_, Operand::Immediate(_)) => {
            Ok(2)
        }
        Instruction::Djnz(Operand::Label(_)) | Instruction::Djnz(Operand::Immediate(_)) => Ok(2),
        Instruction::Call(Operand::Label(_)) | Instruction::Call(Operand::Immediate(_)) => Ok(3), // CALL n16
        Instruction::Call(Operand::Indirect(_)) => Ok(1), // CALL (r)
        Instruction::Callcc(_, _) => Ok(3),               // CALLcc n16
        Instruction::Syscall(_) => Ok(2),                 // SYSCALL n8
        Instruction::Add(Operand::Register(_), Some(Operand::Register(_))) => Ok(2),
        Instruction::Add(Operand::Register(_), Some(Operand::Immediate(_))) => Ok(4),
        Instruction::Sub(Operand::Register(_), Some(Operand::Immediate(_))) => Ok(4),
        Instruction::And(Operand::Register(_), Some(Operand::Immediate(_))) => Ok(4),
        Instruction::Or(Operand::Register(_), Some(Operand::Immediate(_))) => Ok(4),
        Instruction::Xor(Operand::Register(_), Some(Operand::Immediate(_))) => Ok(4),
        Instruction::Cmp(Operand::Register(_), Some(Operand::Immediate(_))) => Ok(4),
        Instruction::Add(Operand::Register(_), None) => Ok(1),
        Instruction::Add(Operand::Immediate(_), None) => Ok(3),
        Instruction::Add(Operand::Label(_), None) => Ok(3),
        Instruction::AddSp(_) => Ok(2),
        Instruction::Sub(Operand::Register(_), Some(Operand::Register(_))) => Ok(2),
        Instruction::Sub(Operand::Register(_), None) => Ok(1),
        Instruction::Sub(Operand::Immediate(_), None) => Ok(3),
        Instruction::Sub(Operand::Label(_), None) => Ok(3),
        Instruction::And(Operand::Register(_), Some(Operand::Register(_))) => Ok(2),
        Instruction::Or(Operand::Register(_), Some(Operand::Register(_))) => Ok(2),
        Instruction::Xor(Operand::Register(_), Some(Operand::Register(_))) => Ok(2),
        Instruction::Cmp(Operand::Register(_), Some(Operand::Register(_))) => Ok(2),
        Instruction::Adc(Operand::Register(_), Some(Operand::Register(_))) => Ok(2),
        Instruction::Sbc(Operand::Register(_), Some(Operand::Register(_))) => Ok(2),
        Instruction::And(Operand::Register(_), None) => Ok(1),
        Instruction::And(Operand::Immediate(_), None) => Ok(3),
        Instruction::And(Operand::Label(_), None) => Ok(3),
        Instruction::Or(Operand::Register(_), None) => Ok(1),
        Instruction::Or(Operand::Immediate(_), None) => Ok(3),
        Instruction::Or(Operand::Label(_), None) => Ok(3),
        Instruction::Xor(Operand::Register(_), None) => Ok(1),
        Instruction::Xor(Operand::Immediate(_), None) => Ok(3),
        Instruction::Xor(Operand::Label(_), None) => Ok(3),
        Instruction::Cmp(Operand::Register(_), None) => Ok(1),
        Instruction::Cmp(Operand::Immediate(_), None) => Ok(3),
        Instruction::Cmp(Operand::Label(_), None) => Ok(3),
        Instruction::Adc(Operand::Immediate(_), None) => Ok(3),
        Instruction::Adc(Operand::Label(_), None) => Ok(3),
        Instruction::Sbc(Operand::Immediate(_), None) => Ok(3),
        Instruction::Sbc(Operand::Label(_), None) => Ok(3),
        Instruction::Push(Operand::Register(_)) => Ok(1),
        Instruction::Push(Operand::Immediate(_)) => Ok(3),
        Instruction::Push(Operand::Label(_)) => Ok(3),
        Instruction::PushF => Ok(1),
        Instruction::Pop(Operand::Register(_)) => Ok(1),
        Instruction::PopF => Ok(1),
        Instruction::Neg => Ok(1),
        Instruction::Not => Ok(1),
        Instruction::Swap => Ok(1),
        Instruction::Sra(_)
        | Instruction::Shl(_)
        | Instruction::Shr(_)
        | Instruction::Rol(_)
        | Instruction::Ror(_)
        | Instruction::Addb(_)
        | Instruction::Subb(_)
        | Instruction::Andb(_)
        | Instruction::Orb(_)
        | Instruction::Xorb(_)
        | Instruction::Cmpb(_) => Ok(2),

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
    start_addr: &u16,
) -> Result<Vec<u8>, AssemblyError> {
    let mut bytecode = Vec::new();
    let mut current_address: u16 = start_addr.clone(); // Start address after cartridge header

    for line in lines {
        if let Some(instruction) = &line.instruction {
            let instruction_bytes = encode_instruction(
                instruction,
                symbol_table,
                &current_address,
                line.line_number,
            )?;
            current_address += calculate_instruction_size(instruction, line.line_number)?;
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

// help function to encode condition code opcode
fn encode_condition_code_opcode(base_opcode: u8, cc: &ConditionCode) -> u8 {
    let cc_offset = match cc {
        ConditionCode::V => 0,
        ConditionCode::Nv => 1,
        ConditionCode::N => 2,
        ConditionCode::Nn => 3,
        ConditionCode::C => 4,
        ConditionCode::Nc => 5,
        ConditionCode::Z => 6,
        ConditionCode::Nz => 7,
    };

    base_opcode + cc_offset
}

/// Helper function to translate a single instruction into bytes during Pass 2.
fn encode_instruction(
    instruction: &Instruction,
    symbol_table: &SymbolTable,
    current_address: &u16,
    line_num: usize,
) -> Result<Vec<u8>, AssemblyError> {
    match instruction {
        // no op (0x00)
        Instruction::Nop => Ok(vec![0x00]),
        // Halt
        Instruction::Halt => Ok(vec![0x0F]),
        // Ccf
        Instruction::Ccf => Ok(vec![0x4B]),
        // Scf
        Instruction::Scf => Ok(vec![0x4C]),
        // Rcf
        Instruction::Rcf => Ok(vec![0x4D]),
        Instruction::Enter => Ok(vec![0x4F]),
        Instruction::Leave => Ok(vec![0x50]),
        Instruction::Ret => Ok(vec![0xF9]),
        Instruction::Reti => Ok(vec![0xFA]),
        Instruction::Ei => Ok(vec![0xFB]),
        Instruction::Di => Ok(vec![0xFC]),
        // inc
        Instruction::Inc(Operand::Register(reg)) => {
            let opcode = encode_reg_opcode(0xE1, reg);
            Ok(vec![opcode])
        }
        // dec
        Instruction::Dec(Operand::Register(reg)) => {
            let opcode = encode_reg_opcode(0xD9, reg);
            Ok(vec![opcode])
        }
        // Example: LDI R1, 0xABCD (Opcode: 0x01 + register index)
        Instruction::Ld(Operand::Register(reg), Operand::Immediate(value)) => {
            let opcode = encode_reg_opcode(0x01, reg);
            let [low, high] = (*value as u16).to_le_bytes();
            Ok(vec![opcode, low, high])
        }
        // Example: JMP my_label
        Instruction::Jmp(Operand::Label(label_name)) => {
            let target_address =
                symbol_table
                    .get(label_name)
                    .ok_or_else(|| AssemblyError::SemanticError {
                        line: line_num,
                        reason: format!("Undefined label: {}", label_name),
                    })?;
            let [low, high] = (*target_address as u16).to_le_bytes();
            Ok(vec![0x51, low, high]) // Opcode for JMP n16
        }
        // jump address
        Instruction::Jmp(Operand::Immediate(addr)) => {
            let [low, high] = (*addr as u16).to_le_bytes();
            Ok(vec![0x51, low, high]) // Opcode for JMP n16
        }
        // jump indirect
        Instruction::Jmp(Operand::Indirect(reg)) => {
            let opcode = encode_reg_opcode(0x52, reg);
            Ok(vec![opcode]) // Opcode for JMP n16
        }
        // jump relative immediate
        Instruction::Jr(Operand::Immediate(imm)) => {
            let rel = *imm as i8;
            Ok(vec![0x5A, rel as u8]) // Opcode for JMP n16
        }
        // jump relative label
        Instruction::Jr(Operand::Label(label_name)) => {
            let target_address =
                symbol_table
                    .get(label_name)
                    .ok_or_else(|| AssemblyError::SemanticError {
                        line: line_num,
                        reason: format!("Undefined label: {}", label_name),
                    })?;
            let rel: i32 = *target_address as i32 - *current_address as i32;
            if rel > i8::MAX as i32 || rel < i8::MIN as i32 {
                return Err(AssemblyError::SemanticError {
                    line: line_num,
                    reason: format!(
                        "Label \"{}\" too far away for relative jump, must be within {} bytes of JR instruction",
                        label_name,
                        i8::MAX
                    ),
                });
            };
            Ok(vec![0x5A, rel as u8]) // Opcode for JMP n16
        }
        // conditional jump immediate
        Instruction::Jcc(cc, Operand::Immediate(addr)) => {
            let [low, high] = (*addr as u16).to_le_bytes();
            let opcode = encode_condition_code_opcode(0x5B, cc);
            Ok(vec![opcode, low, high]) // Opcode for JMP n16
        }
        // conditional jump label
        Instruction::Jcc(cc, Operand::Label(label_name)) => {
            let target_address =
                symbol_table
                    .get(label_name)
                    .ok_or_else(|| AssemblyError::SemanticError {
                        line: line_num,
                        reason: format!("Undefined label: {}", label_name),
                    })?;
            let [low, high] = (*target_address as u16).to_le_bytes();
            let opcode = encode_condition_code_opcode(0x5B, cc);
            Ok(vec![opcode, low, high]) // Opcode for JMP n16
        }
        // conditional jump relative immediate
        Instruction::Jrcc(cc, Operand::Immediate(imm)) => {
            let rel = *imm as i8;
            let opcode = encode_condition_code_opcode(0x63, cc);
            Ok(vec![opcode, rel as u8]) // Opcode for JMP n16
        }
        // conditional jump relative label
        Instruction::Jrcc(cc, Operand::Label(label_name)) => {
            let target_address =
                symbol_table
                    .get(label_name)
                    .ok_or_else(|| AssemblyError::SemanticError {
                        line: line_num,
                        reason: format!("Undefined label: {}", label_name),
                    })?;
            let rel: i32 = *target_address as i32 - *current_address as i32;
            if rel > i8::MAX as i32 || rel < i8::MIN as i32 {
                return Err(AssemblyError::SemanticError {
                    line: line_num,
                    reason: format!(
                        "Label \"{}\" too far away for relative jump, must be within {} bytes of JRcc instruction",
                        label_name,
                        i8::MAX
                    ),
                });
            };
            let opcode = encode_condition_code_opcode(0x63, cc);
            Ok(vec![opcode, rel as u8]) // Opcode for JMP n16
        }
        // DJNZ immediate
        Instruction::Djnz(Operand::Immediate(imm)) => {
            let rel = *imm as i8;
            Ok(vec![0x6B, rel as u8]) // Opcode for JMP n16
        }
        // DJNZ label
        Instruction::Djnz(Operand::Label(label_name)) => {
            let target_address =
                symbol_table
                    .get(label_name)
                    .ok_or_else(|| AssemblyError::SemanticError {
                        line: line_num,
                        reason: format!("Undefined label: {}", label_name),
                    })?;
            let rel: i32 = *target_address as i32 - *current_address as i32;
            if rel > i8::MAX as i32 || rel < i8::MIN as i32 {
                return Err(AssemblyError::SemanticError {
                    line: line_num,
                    reason: format!(
                        "Label \"{}\" too far away for relative jump, must be within {} bytes of DJNZ instruction",
                        label_name,
                        i8::MAX
                    ),
                });
            };
            Ok(vec![0x6B, rel as u8]) // Opcode for JMP n16
        }
        // CALL immediate
        Instruction::Call(Operand::Immediate(addr)) => {
            let [low, high] = (*addr as u16).to_le_bytes();
            Ok(vec![0xC8, low, high])
        }
        // CALL label
        Instruction::Call(Operand::Label(label_name)) => {
            let target_address =
                symbol_table
                    .get(label_name)
                    .ok_or_else(|| AssemblyError::SemanticError {
                        line: line_num,
                        reason: format!("Undefined label: {}", label_name),
                    })?;
            let [low, high] = (*target_address as u16).to_le_bytes();
            Ok(vec![0xC8, low, high]) // Opcode for JMP n16
        }
        // CALL indirect
        Instruction::Call(Operand::Indirect(reg)) => {
            let opcode = encode_reg_opcode(0xC9, reg);
            Ok(vec![opcode]) // Opcode for JMP n16
        }
        // conditional call immediate
        Instruction::Callcc(cc, Operand::Immediate(addr)) => {
            let [low, high] = (*addr as u16).to_le_bytes();
            let opcode = encode_condition_code_opcode(0xD1, cc);
            Ok(vec![opcode, low, high]) // Opcode for JMP n16
        }
        // conditional call label
        Instruction::Callcc(cc, Operand::Label(label_name)) => {
            let target_address =
                symbol_table
                    .get(label_name)
                    .ok_or_else(|| AssemblyError::SemanticError {
                        line: line_num,
                        reason: format!("Undefined label: {}", label_name),
                    })?;
            let [low, high] = (*target_address as u16).to_le_bytes();
            let opcode = encode_condition_code_opcode(0xD1, cc);
            Ok(vec![opcode, low, high]) // Opcode for JMP n16
        }
        // syscall
        Instruction::Syscall(Operand::Immediate(imm)) => Ok(vec![0x4E, *imm as u8]),
        // register-to-register load
        Instruction::Ld(Operand::Register(rd), Operand::Register(rs)) => {
            let opcode = encode_rd_rs_byte(0x80, rd, rs);
            Ok(vec![opcode])
        }
        // add reg to reg
        Instruction::Add(Operand::Register(rd), Some(Operand::Register(rs))) => {
            let byte0 = encode_rd_rs_byte(0x00, rd, rs);
            Ok(vec![0x10, byte0])
        }
        // add immediate
        Instruction::Add(Operand::Register(rd), Some(Operand::Immediate(imm))) => {
            let rd_index = encode_register_operand(rd);
            let [low, high] = (*imm as u16).to_le_bytes();
            Ok(vec![0x09, rd_index, low, high])
        }
        Instruction::AddSp(Operand::Immediate(offset)) => Ok(vec![0x6C, *offset as u8]),
        // add accumulator immediate
        Instruction::Add(Operand::Immediate(value), None) => {
            let [low, high] = (*value as u16).to_le_bytes();
            Ok(vec![0xC0, low, high])
        }
        // add accumulator label
        Instruction::Add(Operand::Label(label_name), None) => {
            let target_address =
                symbol_table
                    .get(label_name)
                    .ok_or_else(|| AssemblyError::SemanticError {
                        line: line_num,
                        reason: format!("Undefined label: {}", label_name),
                    })?;
            let [low, high] = (*target_address as u16).to_le_bytes();
            Ok(vec![0xC0, low, high])
        }
        // add accumulator
        Instruction::Add(Operand::Register(rs), None) => {
            let opcode = encode_reg_opcode(0x18, rs);
            Ok(vec![opcode])
        }
        // add.b accumulator
        Instruction::Addb(Operand::Register(rs)) => {
            let opcode = encode_reg_opcode(0x28, rs);
            Ok(vec![0xFD, opcode])
        }
        // sub reg to reg
        Instruction::Sub(Operand::Register(rd), Some(Operand::Register(rs))) => {
            let byte0 = encode_rd_rs_byte(0x00, rd, rs);
            Ok(vec![0x11, byte0])
        }
        // sub accumulator immediate
        Instruction::Sub(Operand::Immediate(value), None) => {
            let [low, high] = (*value as u16).to_le_bytes();
            Ok(vec![0xC1, low, high])
        }
        // sub accumulator label
        Instruction::Sub(Operand::Label(label_name), None) => {
            let target_address =
                symbol_table
                    .get(label_name)
                    .ok_or_else(|| AssemblyError::SemanticError {
                        line: line_num,
                        reason: format!("Undefined label: {}", label_name),
                    })?;
            let [low, high] = (*target_address as u16).to_le_bytes();
            Ok(vec![0xC1, low, high])
        }
        // sub accumulator
        Instruction::Sub(Operand::Register(rs), None) => {
            let opcode = encode_reg_opcode(0x20, rs);
            Ok(vec![opcode])
        }
        // sub immediate
        Instruction::Sub(Operand::Register(rd), Some(Operand::Immediate(imm))) => {
            let rd_index = encode_register_operand(rd);
            let [low, high] = (*imm as u16).to_le_bytes();
            Ok(vec![0x0A, rd_index, low, high])
        }
        // sub.b accumulator
        Instruction::Subb(Operand::Register(rs)) => {
            let opcode = encode_reg_opcode(0x30, rs);
            Ok(vec![0xFD, opcode])
        }
        // and reg to reg
        Instruction::And(Operand::Register(rd), Some(Operand::Register(rs))) => {
            let byte0 = encode_rd_rs_byte(0x00, rd, rs);
            Ok(vec![0x12, byte0])
        }
        // and immediate
        Instruction::And(Operand::Register(rd), Some(Operand::Immediate(imm))) => {
            let rd_index = encode_register_operand(rd);
            let [low, high] = (*imm as u16).to_le_bytes();
            Ok(vec![0x0B, rd_index, low, high])
        }
        // and.b accumulator
        Instruction::Andb(Operand::Register(rs)) => {
            let opcode = encode_reg_opcode(0x38, rs);
            Ok(vec![0xFD, opcode])
        }
        // or reg to reg
        Instruction::Or(Operand::Register(rd), Some(Operand::Register(rs))) => {
            let byte0 = encode_rd_rs_byte(0x00, rd, rs);
            Ok(vec![0x13, byte0])
        }
        // or immediate
        Instruction::Or(Operand::Register(rd), Some(Operand::Immediate(imm))) => {
            let rd_index = encode_register_operand(rd);
            let [low, high] = (*imm as u16).to_le_bytes();
            Ok(vec![0x0C, rd_index, low, high])
        }
        // or.b accumulator
        Instruction::Orb(Operand::Register(rs)) => {
            let opcode = encode_reg_opcode(0x40, rs);
            Ok(vec![0xFD, opcode])
        }
        // xor reg to reg
        Instruction::Xor(Operand::Register(rd), Some(Operand::Register(rs))) => {
            let byte0 = encode_rd_rs_byte(0x00, rd, rs);
            Ok(vec![0x14, byte0])
        }
        // xor immediate
        Instruction::Xor(Operand::Register(rd), Some(Operand::Immediate(imm))) => {
            let rd_index = encode_register_operand(rd);
            let [low, high] = (*imm as u16).to_le_bytes();
            Ok(vec![0x0D, rd_index, low, high])
        }
        // xor.b accumulator
        Instruction::Xorb(Operand::Register(rs)) => {
            let opcode = encode_reg_opcode(0x48, rs);
            Ok(vec![0xFD, opcode])
        }
        // cmp reg to reg
        Instruction::Cmp(Operand::Register(rd), Some(Operand::Register(rs))) => {
            let byte0 = encode_rd_rs_byte(0x00, rd, rs);
            Ok(vec![0x15, byte0])
        }
        // cmp immediate
        Instruction::Cmp(Operand::Register(rd), Some(Operand::Immediate(imm))) => {
            let rd_index = encode_register_operand(rd);
            let [low, high] = (*imm as u16).to_le_bytes();
            Ok(vec![0x0E, rd_index, low, high])
        }
        // cmp.b accumulator
        Instruction::Cmpb(Operand::Register(rs)) => {
            let opcode = encode_reg_opcode(0x50, rs);
            Ok(vec![0xFD, opcode])
        }
        // adc reg to reg
        Instruction::Adc(Operand::Register(rd), Some(Operand::Register(rs))) => {
            let byte0 = encode_rd_rs_byte(0x00, rd, rs);
            Ok(vec![0x16, byte0])
        }
        // adc accumulator immediate
        Instruction::Adc(Operand::Immediate(value), None) => {
            let [low, high] = (*value as u16).to_le_bytes();
            Ok(vec![0xC6, low, high])
        }
        // adc accumulator label
        Instruction::Adc(Operand::Label(label_name), None) => {
            let target_address =
                symbol_table
                    .get(label_name)
                    .ok_or_else(|| AssemblyError::SemanticError {
                        line: line_num,
                        reason: format!("Undefined label: {}", label_name),
                    })?;
            let [low, high] = (*target_address as u16).to_le_bytes();
            Ok(vec![0xC6, low, high])
        }
        // sbc reg to reg
        Instruction::Sbc(Operand::Register(rd), Some(Operand::Register(rs))) => {
            let byte0 = encode_rd_rs_byte(0x00, rd, rs);
            Ok(vec![0x17, byte0])
        }
        // sbc accumulator immediate
        Instruction::Sbc(Operand::Immediate(value), None) => {
            let [low, high] = (*value as u16).to_le_bytes();
            Ok(vec![0xC7, low, high])
        }
        // sbc accumulator label
        Instruction::Sbc(Operand::Label(label_name), None) => {
            let target_address =
                symbol_table
                    .get(label_name)
                    .ok_or_else(|| AssemblyError::SemanticError {
                        line: line_num,
                        reason: format!("Undefined label: {}", label_name),
                    })?;
            let [low, high] = (*target_address as u16).to_le_bytes();
            Ok(vec![0xC7, low, high])
        }
        Instruction::Push(Operand::Register(reg)) => {
            let index = encode_register_operand(reg);
            Ok(vec![0x6D + index])
        }
        Instruction::Push(Operand::Immediate(value)) => {
            let [low, high] = (*value as u16).to_le_bytes();
            Ok(vec![0x7D, low, high])
        }
        Instruction::Push(Operand::Label(label_name)) => {
            let target_address =
                symbol_table
                    .get(label_name)
                    .ok_or_else(|| AssemblyError::SemanticError {
                        line: line_num,
                        reason: format!("Undefined label: {}", label_name),
                    })?;
            let [low, high] = (*target_address as u16).to_le_bytes();
            Ok(vec![0x7D, low, high])
        }
        Instruction::PushF => Ok(vec![0x7E]),
        Instruction::Pop(Operand::Register(reg)) => {
            let index = encode_register_operand(reg);
            Ok(vec![0x75 + index])
        }
        Instruction::PopF => Ok(vec![0x7F]),
        // and accumulator immediate
        Instruction::And(Operand::Immediate(value), None) => {
            let [low, high] = (*value as u16).to_le_bytes();
            Ok(vec![0xC2, low, high])
        }
        // and accumulator label
        Instruction::And(Operand::Label(label_name), None) => {
            let target_address =
                symbol_table
                    .get(label_name)
                    .ok_or_else(|| AssemblyError::SemanticError {
                        line: line_num,
                        reason: format!("Undefined label: {}", label_name),
                    })?;
            let [low, high] = (*target_address as u16).to_le_bytes();
            Ok(vec![0xC2, low, high])
        }
        // and accumulator
        Instruction::And(Operand::Register(rs), None) => {
            let opcode = encode_reg_opcode(0x28, rs);
            Ok(vec![opcode])
        }
        // or accumulator
        Instruction::Or(Operand::Register(rs), None) => {
            let opcode = encode_reg_opcode(0x30, rs);
            Ok(vec![opcode])
        }
        // or accumulator immediate
        Instruction::Or(Operand::Immediate(value), None) => {
            let [low, high] = (*value as u16).to_le_bytes();
            Ok(vec![0xC3, low, high])
        }
        // or accumulator label
        Instruction::Or(Operand::Label(label_name), None) => {
            let target_address =
                symbol_table
                    .get(label_name)
                    .ok_or_else(|| AssemblyError::SemanticError {
                        line: line_num,
                        reason: format!("Undefined label: {}", label_name),
                    })?;
            let [low, high] = (*target_address as u16).to_le_bytes();
            Ok(vec![0xC3, low, high])
        }
        // xor accumulator
        Instruction::Xor(Operand::Register(rs), None) => {
            let opcode = encode_reg_opcode(0x38, rs);
            Ok(vec![opcode])
        }
        // xor accumulator immediate
        Instruction::Xor(Operand::Immediate(value), None) => {
            let [low, high] = (*value as u16).to_le_bytes();
            Ok(vec![0xC4, low, high])
        }
        // xor accumulator label
        Instruction::Xor(Operand::Label(label_name), None) => {
            let target_address =
                symbol_table
                    .get(label_name)
                    .ok_or_else(|| AssemblyError::SemanticError {
                        line: line_num,
                        reason: format!("Undefined label: {}", label_name),
                    })?;
            let [low, high] = (*target_address as u16).to_le_bytes();
            Ok(vec![0xC4, low, high])
        }
        // cmp accumulator
        Instruction::Cmp(Operand::Register(rs), None) => {
            let opcode = encode_reg_opcode(0x40, rs);
            Ok(vec![opcode])
        }
        // cmp accumulator immediate
        Instruction::Cmp(Operand::Immediate(value), None) => {
            let [low, high] = (*value as u16).to_le_bytes();
            Ok(vec![0xC5, low, high])
        }
        // cmp accumulator label
        Instruction::Cmp(Operand::Label(label_name), None) => {
            let target_address =
                symbol_table
                    .get(label_name)
                    .ok_or_else(|| AssemblyError::SemanticError {
                        line: line_num,
                        reason: format!("Undefined label: {}", label_name),
                    })?;
            let [low, high] = (*target_address as u16).to_le_bytes();
            Ok(vec![0xC5, low, high])
        }
        // neg
        Instruction::Neg => Ok(vec![0x48]),
        // not
        Instruction::Not => Ok(vec![0x49]),
        // swap
        Instruction::Swap => Ok(vec![0x4A]),
        // indirect-to-register load
        Instruction::Ld(Operand::Register(rd), Operand::Indirect(rs)) => {
            let sub_opcode = encode_rd_rs_byte(0x00, rd, rs);
            Ok(vec![0xFE, sub_opcode])
        }
        // register-to-indirect store
        Instruction::St(Operand::Indirect(rd), Operand::Register(rs)) => {
            let sub_opcode = encode_rd_rs_byte(0x40, rd, rs);
            Ok(vec![0xFE, sub_opcode])
        }
        // absolute-to-register load
        Instruction::Ld(Operand::Register(rd), Operand::Absolute(addr)) => {
            let opcode = encode_reg_opcode(0xE9, rd);
            let [low, high] = (*addr as u16).to_le_bytes();
            Ok(vec![opcode, low, high])
        }
        // register-to-absolute store
        Instruction::St(Operand::Absolute(addr), Operand::Register(rs)) => {
            let opcode = encode_reg_opcode(0xF1, rs);
            let [low, high] = (*addr as u16).to_le_bytes();
            Ok(vec![opcode, low, high])
        }

        Instruction::Sra(Operand::Register(reg)) => {
            let opcode = encode_reg_opcode(0x00, reg);
            Ok(vec![0xFD, opcode])
        }
        Instruction::Shl(Operand::Register(reg)) => {
            let opcode = encode_reg_opcode(0x08, reg);
            Ok(vec![0xFD, opcode])
        }
        Instruction::Shr(Operand::Register(reg)) => {
            let opcode = encode_reg_opcode(0x10, reg);
            Ok(vec![0xFD, opcode])
        }
        Instruction::Rol(Operand::Register(reg)) => {
            let opcode = encode_reg_opcode(0x18, reg);
            Ok(vec![0xFD, opcode])
        }
        Instruction::Ror(Operand::Register(reg)) => {
            let opcode = encode_reg_opcode(0x20, reg);
            Ok(vec![0xFD, opcode])
        }

        // ... add encoding logic for every instruction variant based on your opcode map ...
        _ => Err(AssemblyError::SemanticErrorNoLine {
            reason: "Invalid Instruction".to_string(),
        }),
    }
}

fn encode_rd_rs_byte(base_val: u8, rd: &Register, rs: &Register) -> u8 {
    let rd_index = encode_register_operand(rd);
    let rs_index = encode_register_operand(rs);
    base_val | ((rd_index & 0x07) << 3) | (rs_index & 0x07)
}

fn encode_reg_opcode(base_opcode: u8, r: &Register) -> u8 {
    base_opcode + encode_register_operand(r)
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
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
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
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
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
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
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
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
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
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
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
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
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
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
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
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
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
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
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
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
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
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
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
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0x44]
        );
    }

    #[test]
    fn test_encode_instruction_add_acc_immediate() {
        let instruction = Instruction::Add(Operand::Immediate(0x1234), None);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0xC0, 0x34, 0x12]
        );
    }

    #[test]
    fn test_encode_instruction_add_acc_label() {
        let instruction = Instruction::Add(Operand::Label("TARGET".into()), None);
        let mut symbol_table = SymbolTable::new();
        symbol_table.insert("TARGET".to_string(), 0x2468);
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0xC0, 0x68, 0x24]
        );
    }

    #[test]
    fn test_encode_instruction_sub_acc_immediate() {
        let instruction = Instruction::Sub(Operand::Immediate(0x00FF), None);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0xC1, 0xFF, 0x00]
        );
    }

    #[test]
    fn test_encode_instruction_and_acc_immediate() {
        let instruction = Instruction::And(Operand::Immediate(0x0F0F), None);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0xC2, 0x0F, 0x0F]
        );
    }

    #[test]
    fn test_encode_instruction_or_acc_immediate() {
        let instruction = Instruction::Or(Operand::Immediate(0x8000), None);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0xC3, 0x00, 0x80]
        );
    }

    #[test]
    fn test_encode_instruction_xor_acc_immediate() {
        let instruction = Instruction::Xor(Operand::Immediate(0xAAAA), None);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0xC4, 0xAA, 0xAA]
        );
    }

    #[test]
    fn test_encode_instruction_cmp_acc_immediate() {
        let instruction = Instruction::Cmp(Operand::Immediate(0x0A0B), None);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0xC5, 0x0B, 0x0A]
        );
    }

    #[test]
    fn test_encode_instruction_adci_acc_immediate() {
        let instruction = Instruction::Adc(Operand::Immediate(0xFFFF), None);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0xC6, 0xFF, 0xFF]
        );
    }

    #[test]
    fn test_encode_instruction_sbci_acc_immediate() {
        let instruction = Instruction::Sbc(Operand::Immediate(0x1234), None);
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0xC7, 0x34, 0x12]
        );
    }

    #[test]
    fn test_calculate_instruction_size_add_sp() {
        let instruction = Instruction::AddSp(Operand::Immediate(-8));
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 2);
    }

    #[test]
    fn test_encode_instruction_add_sp() {
        let instruction = Instruction::AddSp(Operand::Immediate(-5));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0x6C, 0xFB]
        );
    }

    #[test]
    fn test_encode_instruction_push_reg() {
        let instruction = Instruction::Push(Operand::Register(Register::R2));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0x6F]
        );
    }

    #[test]
    fn test_encode_instruction_push_immediate() {
        let instruction = Instruction::Push(Operand::Immediate(0x1234));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0x7D, 0x34, 0x12]
        );
    }

    #[test]
    fn test_encode_instruction_push_label() {
        let instruction = Instruction::Push(Operand::Label("TARGET".into()));
        let mut symbol_table = SymbolTable::new();
        symbol_table.insert("TARGET".to_string(), 0x1357);
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0x7D, 0x57, 0x13]
        );
    }

    #[test]
    fn test_encode_instruction_push_f() {
        let instruction = Instruction::PushF;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0x7E]
        );
    }

    #[test]
    fn test_encode_instruction_pop_reg() {
        let instruction = Instruction::Pop(Operand::Register(Register::R3));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0x78]
        );
    }

    #[test]
    fn test_encode_instruction_pop_f() {
        let instruction = Instruction::PopF;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0x7F]
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
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
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
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
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
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0x4A]
        );
    }

    #[test]
    fn test_calculate_instruction_size_ccf() {
        let instruction = Instruction::Ccf;
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 1);
    }

    #[test]
    fn test_encode_instruction_ccf() {
        let instruction = Instruction::Ccf;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0x4B]
        );
    }

    #[test]
    fn test_calculate_instruction_size_scf() {
        let instruction = Instruction::Scf;
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 1);
    }

    #[test]
    fn test_encode_instruction_scf() {
        let instruction = Instruction::Scf;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0x4C]
        );
    }

    #[test]
    fn test_calculate_instruction_size_rcf() {
        let instruction = Instruction::Rcf;
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 1);
    }

    #[test]
    fn test_encode_instruction_rcf() {
        let instruction = Instruction::Rcf;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0x4D]
        );
    }

    #[test]
    fn test_calculate_instruction_size_enter() {
        let instruction = Instruction::Enter;
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 1);
    }

    #[test]
    fn test_encode_instruction_enter() {
        let instruction = Instruction::Enter;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0x4F]
        );
    }

    #[test]
    fn test_calculate_instruction_size_leave() {
        let instruction = Instruction::Leave;
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 1);
    }

    #[test]
    fn test_encode_instruction_leave() {
        let instruction = Instruction::Leave;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0x50]
        );
    }

    #[test]
    fn test_calculate_instruction_size_ret() {
        let instruction = Instruction::Ret;
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 1);
    }

    #[test]
    fn test_encode_instruction_ret() {
        let instruction = Instruction::Ret;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0xF9]
        );
    }

    #[test]
    fn test_calculate_instruction_size_reti() {
        let instruction = Instruction::Reti;
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 1);
    }

    #[test]
    fn test_encode_instruction_reti() {
        let instruction = Instruction::Reti;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0xFA]
        );
    }

    #[test]
    fn test_calculate_instruction_size_ei() {
        let instruction = Instruction::Ei;
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 1);
    }

    #[test]
    fn test_encode_instruction_ei() {
        let instruction = Instruction::Ei;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0xFB]
        );
    }

    #[test]
    fn test_calculate_instruction_size_di() {
        let instruction = Instruction::Di;
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 1);
    }

    #[test]
    fn test_encode_instruction_di() {
        let instruction = Instruction::Di;
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0xFC]
        );
    }

    #[test]
    fn test_calculate_instruction_size_inc() {
        let instruction = Instruction::Inc(Operand::Register(Register::R1));
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 1);
    }

    #[test]
    fn test_encode_instruction_inc() {
        let instruction = Instruction::Inc(Operand::Register(Register::R1));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0xE2]
        );
    }

    #[test]
    fn test_calculate_instruction_size_dec() {
        let instruction = Instruction::Dec(Operand::Register(Register::R2));
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 1);
    }

    #[test]
    fn test_encode_instruction_dec() {
        let instruction = Instruction::Dec(Operand::Register(Register::R2));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0xDB]
        );
    }

    #[test]
    fn test_calculate_instruction_size_call_immediate() {
        let instruction = Instruction::Call(Operand::Immediate(0x1234));
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 3);
    }

    #[test]
    fn test_encode_instruction_call_immediate() {
        let instruction = Instruction::Call(Operand::Immediate(0x1234));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0xC8, 0x34, 0x12]
        );
    }

    #[test]
    fn test_calculate_instruction_size_call_label() {
        let instruction = Instruction::Call(Operand::Label("test".to_string()));
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 3);
    }

    #[test]
    fn test_encode_instruction_call_label() {
        let instruction = Instruction::Call(Operand::Label("test_label".to_string()));
        let mut symbol_table = SymbolTable::new();
        symbol_table.insert("test_label".to_string(), 0x4321);
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0xC8, 0x21, 0x43]
        );
    }

    #[test]
    fn test_calculate_instruction_size_call_indirect() {
        let instruction = Instruction::Call(Operand::Indirect(Register::R4));
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 1);
    }

    #[test]
    fn test_encode_instruction_call_indirect() {
        let instruction = Instruction::Call(Operand::Indirect(Register::R4));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0xCD] // 0xC9 + 4
        );
    }

    #[test]
    fn test_calculate_instruction_size_callcc() {
        let instruction =
            Instruction::Callcc(ConditionCode::Nz, Operand::Label("test".to_string()));
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 3);
    }

    #[test]
    fn test_encode_instruction_callcc_immediate() {
        let instruction = Instruction::Callcc(ConditionCode::C, Operand::Immediate(0x1122));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0xD5, 0x22, 0x11] // 0xD1 + 4
        );
    }

    #[test]
    fn test_encode_instruction_callcc_label() {
        let instruction =
            Instruction::Callcc(ConditionCode::Nz, Operand::Label("test_label".to_string()));
        let mut symbol_table = SymbolTable::new();
        symbol_table.insert("test_label".to_string(), 0x4321);
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0xD8, 0x21, 0x43] // 0xD1 + 7
        );
    }

    #[test]
    fn test_calculate_instruction_size_syscall() {
        let instruction = Instruction::Syscall(Operand::Immediate(0x1A));
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 2);
    }

    #[test]
    fn test_encode_instruction_syscall() {
        let instruction = Instruction::Syscall(Operand::Immediate(0x1A));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0x4E, 0x1A]
        );
    }

    #[test]
    fn test_calculate_instruction_size_ld_absolute() {
        let instruction =
            Instruction::Ld(Operand::Register(Register::R1), Operand::Absolute(0x1234));
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 3);
    }

    #[test]
    fn test_encode_instruction_ld_absolute() {
        let instruction =
            Instruction::Ld(Operand::Register(Register::R1), Operand::Absolute(0x1234));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0xEA, 0x34, 0x12] // 0xE9 + 1
        );
    }

    #[test]
    fn test_calculate_instruction_size_st_absolute() {
        let instruction =
            Instruction::St(Operand::Absolute(0x4321), Operand::Register(Register::R2));
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 3);
    }

    #[test]
    fn test_encode_instruction_st_absolute() {
        let instruction =
            Instruction::St(Operand::Absolute(0x4321), Operand::Register(Register::R2));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0xF3, 0x21, 0x43] // 0xF1 + 2
        );
    }

    #[test]
    fn test_calculate_instruction_size_st_indirect() {
        let instruction = Instruction::St(
            Operand::Indirect(Register::R1),
            Operand::Register(Register::R2),
        );
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 2);
    }

    #[test]
    fn test_encode_instruction_st_indirect() {
        let instruction = Instruction::St(
            Operand::Indirect(Register::R1),
            Operand::Register(Register::R2),
        );
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0xFE, 0x4A] // 0x40 | (1 << 3) | 2
        );
    }

    #[test]
    fn test_encode_sra() {
        let instruction = Instruction::Sra(Operand::Register(Register::R0));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0xFD, 0x00]
        );
    }

    #[test]
    fn test_encode_shl() {
        let instruction = Instruction::Shl(Operand::Register(Register::R1));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0xFD, 0x09]
        );
    }

    #[test]
    fn test_encode_shr() {
        let instruction = Instruction::Shr(Operand::Register(Register::R2));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0xFD, 0x12]
        );
    }

    #[test]
    fn test_encode_rol() {
        let instruction = Instruction::Rol(Operand::Register(Register::R3));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0xFD, 0x1B]
        );
    }

    #[test]
    fn test_encode_ror() {
        let instruction = Instruction::Ror(Operand::Register(Register::R4));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0xFD, 0x24]
        );
    }

    #[test]
    fn test_calculate_instruction_size_add_b() {
        let instruction = Instruction::Addb(Operand::Register(Register::R1));
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 2);
    }

    #[test]
    fn test_encode_instruction_add_b() {
        let instruction = Instruction::Addb(Operand::Register(Register::R1));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0xFD, 0x29] // 0x28 + 1
        );
    }

    #[test]
    fn test_calculate_instruction_size_sub_b() {
        let instruction = Instruction::Subb(Operand::Register(Register::R2));
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 2);
    }

    #[test]
    fn test_encode_instruction_sub_b() {
        let instruction = Instruction::Subb(Operand::Register(Register::R2));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0xFD, 0x32] // 0x30 + 2
        );
    }

    #[test]
    fn test_calculate_instruction_size_and_b() {
        let instruction = Instruction::Andb(Operand::Register(Register::R3));
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 2);
    }

    #[test]
    fn test_encode_instruction_and_b() {
        let instruction = Instruction::Andb(Operand::Register(Register::R3));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0xFD, 0x3B] // 0x38 + 3
        );
    }

    #[test]
    fn test_calculate_instruction_size_or_b() {
        let instruction = Instruction::Orb(Operand::Register(Register::R4));
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 2);
    }

    #[test]
    fn test_encode_instruction_or_b() {
        let instruction = Instruction::Orb(Operand::Register(Register::R4));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0xFD, 0x44] // 0x40 + 4
        );
    }

    #[test]
    fn test_calculate_instruction_size_xor_b() {
        let instruction = Instruction::Xorb(Operand::Register(Register::R5));
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 2);
    }

    #[test]
    fn test_encode_instruction_xor_b() {
        let instruction = Instruction::Xorb(Operand::Register(Register::R5));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0xFD, 0x4D] // 0x48 + 5
        );
    }

    #[test]
    fn test_calculate_instruction_size_cmp_b() {
        let instruction = Instruction::Cmpb(Operand::Register(Register::R6));
        assert_eq!(calculate_instruction_size(&instruction, 0).unwrap(), 2);
    }

    #[test]
    fn test_encode_instruction_cmp_b() {
        let instruction = Instruction::Cmpb(Operand::Register(Register::R6));
        let symbol_table = SymbolTable::new();
        assert_eq!(
            encode_instruction(&instruction, &symbol_table, &0, 0).unwrap(),
            vec![0xFD, 0x56] // 0x50 + 6
        );
    }
}
