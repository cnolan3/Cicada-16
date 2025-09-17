// --- Operands ---

#[derive(Debug, Clone, PartialEq)]
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
    Indirect(Register),    // e.g., (R1)
    Absolute(u16),         // e.g., (0x2020)
    Indexed(Register, i8), // e.g., (R1, 0x10) or (R1, -2)
    Label(String),         // e.g., my_label
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

    // Load instructions
    Ld(Operand, Operand), // Covers LD rd, rs; LDI r, n16; LD r, (n16); etc.
    St(Operand, Operand),

    // Arithmetic instructions
    Add(Operand, Option<Operand>), // Option for one-operand (ADD rs) vs two-operand (ADD rd, rs)
    Sub(Operand, Option<Operand>),
    And(Operand, Option<Operand>),
    Or(Operand, Option<Operand>),
    Xor(Operand, Option<Operand>),
    Cmp(Operand, Option<Operand>),
    Adc(Operand, Option<Operand>),
    Sbc(Operand, Option<Operand>),
    AddSp(Operand),
    Neg,
    Not,
    Swap,
    Inc(Operand),
    Dec(Operand),

    // Jumps and Calls
    Jmp(Operand),
    Jr(Operand),
    Call(Operand),
    Jcc(ConditionCode, Operand), // Jcc cc, target
    Jrcc(ConditionCode, Operand),
    Djnz(Operand),
    Callcc(ConditionCode, Operand),
    Syscall(Operand),

    // Stack operations
    Push(Operand),
    Pop(Operand),
    PushF,
    PopF,
    // ... add all other instructions from your ISA ...
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

// --- Assembly Line Structure ---

// Represents a single line of code, which can have a label, an instruction, or both.
#[derive(Debug, Clone, Default)]
pub struct AssemblyLine {
    pub line_number: usize,
    pub label: Option<String>,
    pub instruction: Option<Instruction>,
    // Add directives later: pub directive: Option<Directive>,
}
