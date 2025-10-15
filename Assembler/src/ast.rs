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

#[derive(Default, Debug, Clone, PartialEq)]
pub struct HeaderInfo {
    pub boot_anim: String,
    pub title: String,
    pub developer: String,
    pub version: u8,
    pub mapper: u8,
    pub rom_size: u8,
    pub ram_size: u8,
    pub interrupt_mode: u8,
    pub hardware_rev: u8,
    pub region: u8,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Register {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
}

// Represents all possible forms an argument to an instruction can take.
#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Register(Register),
    Immediate(i32),
    Indirect(Register), // e.g., (R1)
    AbsAddr(u16),       // e.g., (0x2020)
    AbsLabel(String),
    Indexed(Register, i8),          // e.g., (R1, 0x10) or (R1, -2)
    IndexedLabel(Register, String), // e.g., (R1, const_val)
    Label(String),                  // e.g., my_label
    PreDecrement(Register),
    PostIncrement(Register),
    String(String),
}

// --- Instructions ---

// Enum representing a single instruction.
// This directly maps to the mnemonics in your ISA documentation.
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    Nop,
    Halt,
    Ei,
    Di,
    Ret,
    Reti,
    Ccf,
    Scf,
    Rcf,
    Enter,
    Leave,

    // 16 bit Load/Store instructions
    LdReg(Register, Register),              // LD r1, r2
    Ldi(Register, Operand),                 // LDI r1, 0x1234 AND LDI r1, label
    LdIndirect(Register, Register),         // LD r1, (r2)
    LdAbs(Register, Operand),               // LD r1, (0x1234) AND LD r1, (label)
    LdIndexed(Register, Register, Operand), // LD r1, (r2, 3)
    LdPreDec(Register, Register),           // LD r1, -(r2)
    LdPostInc(Register, Register),          // LD r1, (r2)+
    StIndirect(Register, Register),         // St (r1), r2
    StAbs(Operand, Register),               // St (0x1234), r1 AND LD (label), r1
    StIndexed(Register, Operand, Register), // St (r1, 4), r2
    StPreDec(Register, Register),           // St -(r1), r2
    StPostInc(Register, Register),          // St (r1)+, r2

    // 8 bit Load/Store instructions
    LdiB(Register, Operand),         // LDI.b r1, 0x12
    LdBIndirect(Register, Register), // LD.b r1, (r2)
    LdBPreDec(Register, Register),   // LD.b r1, -(r2)
    LdBPostInc(Register, Register),  // LD.b r1, (r2)+
    LdBAbs(Register, Operand),       // LD.b r1, (0x1234) AND LD.b r1, (label)
    StBIndirect(Register, Register), // St.b (r1), r2
    StBPreDec(Register, Register),   // St.b -(r1), r2
    StBPostInc(Register, Register),  // St.b (r1)+, r2
    StBAbs(Operand, Register),       // ST.b (0x1234), r1 AND ST.b (label), r1

    // LEA
    Lea(Register, Register, Operand), // LEA r1, (r2, 0x12)

    // Stack Operations
    Push(Register), // PUSH r1
    Pop(Register),  // POP r1
    PushI(Operand), // PUSH 0x1234 AND PUSH label
    PushF,          // PUSH f
    PopF,           // POP f

    // 16 bit accumulator arithmetic
    AddAcc(Register), // ADD r1
    SubAcc(Register), // SUB r1
    AndAcc(Register), // AND r1
    OrAcc(Register),  // OR r1
    XorAcc(Register), // XOR r1
    CmpAcc(Register), // CMP r1
    NegAcc,           // NEG
    NotAcc,           // NOT
    SwapAcc,          // SWAP

    // 16 bit accumulator immediate arithmetic
    AddAccI(Operand), // ADDI 0x1234 AND ADDI label
    SubAccI(Operand), // SUBI 0x1234 AND SUBI label
    AndAccI(Operand), // ANDI 0x1234 AND ANDI label
    OrAccI(Operand),  // ORI 0x1234 AND ORI label
    XorAccI(Operand), // XORI 0x1234 AND XORI label
    CmpAccI(Operand), // CMPI 0x1234 AND CMPI label
    AdcAccI(Operand), // ADCI 0x1234 AND ADCI label
    SbcAccI(Operand), // SBCI 0x1234 AND SBCI label

    // 16 bit register-to-register arithmetic
    AddReg(Register, Register), // ADD r1, r2
    SubReg(Register, Register), // SUB r1, r2
    AndReg(Register, Register), // AND r1, r2
    OrReg(Register, Register),  // OR r1, r2
    XorReg(Register, Register), // XOR r1, r2
    CmpReg(Register, Register), // CMP r1, r1
    AdcReg(Register, Register), // ADC r1, r2
    SbcReg(Register, Register), // SBC r1, r2

    // 16 bit Immediate-to-register arithmetic
    AddIReg(Register, Operand), // ADD r1, 0x1234 AND ADD r1, label
    SubIReg(Register, Operand), // SUB r1, 0x1234 AND SUB r1, label
    AndIReg(Register, Operand), // AND r1, 0x1234 AND AND r1, label
    OrIReg(Register, Operand),  // OR r1, 0x1234 AND OR r1, label
    XorIReg(Register, Operand), // XOR r1, 0x1234 AND XOR r1, label
    CmpIReg(Register, Operand), // CMP r1, 0x1234 AND CMP r1, label
    AddSp(Operand),             // ADD SP, -3
    Inc(Register),              // INC r1,
    Dec(Register),              // DEC r1,

    // 8 bit accumulator arithmetic
    AddBAcc(Register), // ADD.b r1
    SubBAcc(Register), // SUB.b r1
    AndBAcc(Register), // And.b r1
    OrBAcc(Register),  // OR.b r1
    XorBAcc(Register), // XOR.b r1
    CmpBAcc(Register), // CMP.b r1

    // bit manipulation
    Sra(Register),                  // SRA r1
    Shl(Register),                  // SHL r1
    Shr(Register),                  // SHR r1
    Rol(Register),                  // ROL r1
    Ror(Register),                  // ROR r1
    BitReg(Register, Operand),      // BIT r1, 3
    SetReg(Register, Operand),      // SET r1, 3
    ResReg(Register, Operand),      // RES r1, 3
    BitAbs(Operand, Operand),       // BIT (0x1234), 3
    SetAbs(Operand, Operand),       // SET (0x1234), 3
    ResAbs(Operand, Operand),       // RES (0x1234), 3
    BitIndirect(Register, Operand), // BIT (r1), 3
    SetIndirect(Register, Operand), // SET (r1), 3
    ResIndirect(Register, Operand), // RES (r1), 3

    // Control Flow
    JmpI(Operand),                   // JMP 0x1234 AND JMP label
    JmpIndirect(Register),           // JMP (r1)
    JrI(Operand),                    // JR -3 AND JR label
    JccI(ConditionCode, Operand),    // Jcc 0x1234 AND Jcc label
    JrccI(ConditionCode, Operand),   // JRcc -3 AND JRcc label
    Djnz(Operand),                   // DJNZ -3 AND DJNZ label
    CallI(Operand),                  // CALL 0x1234 AND CALL label
    CallIndirect(Register),          // CALL (r1)
    CallccI(ConditionCode, Operand), // CALLcc 0x1234 AND CALL label
    Syscall(Operand),                // SYSCALL 0x20
    CallFar(String),                 // CALL.far label
    CallFarVia(String, String),      // Call.far label via label
    JmpFar(String),                  // JMP.far label
    JmpFarVia(String, String),       // JMP.far label via label
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConditionCode {
    Z,  // Zero
    Nz, // Not Zero
    C,  // Carry
    Nc, // No Carry
    N,  // Negative
    Nn, // Not Negative
    V,  // Overflow
    Nv, // Not Overflow
}

#[derive(Debug, Clone, PartialEq)]
pub enum Directive {
    Org(Operand),                 // .org 0x1234 AND .org label
    Bank(Operand),                // .bank 3
    Byte(Vec<Operand>),           // .byte 0x01, 0x02, 0x03
    Word(Vec<Operand>),           // .word 0x0001, 0x0002, 0x0003 AND .word label, label, label
    Define(String, Operand),      // .define label 0x01
    Include(String),              // .include path
    Header(HeaderInfo),           // .header_start ... .header_end
    Interrupt(Vec<Operand>),      // .interrupt_table ... .table_end
    SectionStart(SectionOptions), // .section
    SectionEnd,                   // .section_end
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct SectionOptions {
    pub name: Option<String>,
    pub size: Option<u32>,
    pub vaddr: Option<u32>,
    pub paddr: Option<u32>,
    pub align: Option<u32>,
}

// --- Assembly Line Structure ---

// Represents a single line of code, which can have a label, an instruction, or both.
#[derive(Debug, Clone, Default)]
pub struct AssemblyLine {
    pub line_number: usize,
    pub label: Option<String>,
    pub instruction: Option<Instruction>,
    pub directive: Option<Directive>, // Add directives later: pub directive: Option<Directive>,
}
