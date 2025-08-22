# **Cricket-16 CPU - Instruction Set Architecture (ISA)**

**This document outlines the instruction set for the "Cricket-16" fantasy console's main processor. The architecture is a 16-bit design, intended to be reminiscent of late 80s / early 90s home consoles. It features a clean register set and instructions that operate on 16-bit words by default, with support for 8-bit byte manipulation.**

## **1. CPU Registers**

**The CPU has a set of general-purpose and special-purpose 16-bit registers. Unlike 8-bit CPUs that pair registers, these are all native 16-bit.**

### **16-bit General Purpose Registers**

- **R0, R1, R2, R3, R4, R5:** General Purpose Registers - Can be used for data manipulation and as address pointers. For register-to-register arithmetic instructions (ADD, SUB, etc.), R0 serves as the accumulator.
- **R6:** General Purpose / Frame Pointer (FP) - Can be used as a general register, but is often used by compilers to manage stack frames.
- **R7:** Stack Pointer (SP) - Points to the top of the stack in WRAM. PUSH and POP operations implicitly use this register.

### **Special Purpose 16-bit Registers**

- **PC:** Program Counter - Points to the address of the next instruction to be executed.
- **F:** Flags Register - A 16-bit register holding status flags.

### **Flags Register (F)**

**The F register holds status flags that are set or cleared based on the results of 16-bit or 8-bit operations.**

- **Z (Bit 15):** Zero Flag - Set if the result of an operation is 0.
- **N (Bit 14):** Negative Flag - Set if the most significant bit (MSB) of the result is 1.
- **C (Bit 13):** Carry Flag - Set if there was a carry from the most significant bit (bit 15 for words, bit 7 for bytes).
- **V (Bit 12):** Overflow Flag - Set if a signed arithmetic operation resulted in an overflow.
- **Bits 11-0:** Unused/Reserved

## **2. Instruction Notation**

- **r, rs, rd:** Any 16-bit general purpose register (R0-R7)
- **n16:** An immediate 16-bit value (e.g., 0x1234)
- **n8:** An immediate 8-bit value (e.g., 0x2A)
- **(addr):** The value stored at the memory address addr.
- **.w:** Suffix for a word (16-bit) operation. This is the default for most instructions.
- **.b:** Suffix for a byte (8-bit) operation.
- **cc:** Condition code for jumps/calls (Z, NZ, C, NC, N, NN, V, NV)

## **3. Instruction Set**

### **Load/Store Instructions**

**These instructions move data between registers and memory. Memory is byte-addressable. 16-bit word accesses must be aligned to an even address. Attempting an unaligned 16-bit access will trigger a Bus Error fault.**

| Mnemonic      | Operands     | Bytes | Cycles | Description                                                                               |
| :------------ | :----------- | :---- | :----- | :---------------------------------------------------------------------------------------- |
| LD r, r       | R1, R2       | 1     | 4      | Copies the 16-bit value of R2 into R1.                                                    |
| LD r, n16     | R0, 0x8000   | 3     | 8      | Loads the immediate 16-bit value 0x8000 into R0.                                          |
| LD.w rd, (rs) | R2, (R3)     | 2     | 12     | Loads the 16-bit word from the address in R3 into R2. (Prefixed)                          |
| LD.b rd, (rs) | R2, (R3)     | 2     | 12     | Loads the 8-bit byte from the address in R3 into the low byte of R2, and zero-extends it. (Prefixed) |
| ST.w (rd), rs | (R4), R5     | 2     | 12     | Stores the 16-bit word from R5 into the memory address pointed to by R4. (Prefixed)       |
| ST.b (rd), rs | (R4), R5     | 2     | 12     | Stores the low 8-bit byte from R5 into the memory address pointed to by R4. (Prefixed)    |
| LD.w r, (n16) | R0, (0xC000) | 3     | 12     | Loads a 16-bit word from the absolute address 0xC000 into R0.                             |
| ST.w (n16), r | (0xC000), R0 | 3     | 12     | Stores the 16-bit word in R0 to the absolute address 0xC000.                              |
| PUSH r        | R0           | 1     | 12     | Pushes the value of a register onto the stack. Decrements SP by 2.                        |
| POP r         | R0           | 1     | 12     | Pops a value from the stack into a register. Increments SP by 2.                          |

### **16-Bit Arithmetic/Logic Instructions**

**These instructions perform operations on 16-bit data. For register-to-register operations, R0 is the implicit destination (accumulator).**

| Mnemonic    | Operands   | Bytes | Cycles | Description                                                                      |
| :---------- | :--------- | :---- | :----- | :------------------------------------------------------------------------------- |
| ADD rs      | R1         | 1     | 4      | R0 = R0 + R1. Affects Z, N, C, V flags.                                          |
| ADD r, n16 | R0, 0x0100 | 3     | 8      | R0 = R0 + 0x0100. Affects Z, N, C, V flags.                                      |
| SUB rs      | R1         | 1     | 4      | R0 = R0 - R1. Affects Z, N, C, V flags.                                          |
| SUB r, n16 | R0, 0x0100 | 3     | 8      | R0 = R0 - 0x0100. Affects Z, N, C, V flags.                                      |
| AND rs      | R1         | 1     | 4      | R0 = R0 & R1. Affects Z, N flags.                                                |
| AND r, n16 | R0, 0x00FF | 3     | 8      | R0 = R0 & 0x00FF. Affects Z, N flags.                                            |
| OR rs       | R1         | 1     | 4      | R0 = R0 | R1. Affects Z, N flags.                                                |
| OR r, n16  | R0, 0xF0F0 | 3     | 8      | R0 = R0 | 0xF0F0. Affects Z, N flags.                                            |
| XOR rs      | R1         | 1     | 4      | R0 = R0 ^ R1. Affects Z, N flags.                                                |
| XOR r, n16 | R0, 0xFFFF | 3     | 8      | R0 = R0 ^ 0xFFFF. Affects Z, N flags.                                            |
| INC r       | R2         | 1     | 4      | R2 = R2 + 1. Affects Z, N, V flags.                                              |
| DEC r       | R2         | 1     | 4      | R2 = R2 - 1. Affects Z, N, V flags.                                              |
| CMP rs      | R1         | 1     | 4      | Compares R0 and R1 (calculates R0 - R1) and sets flags, but discards the result. |
| CMP r, n16  | R0, 0x4000 | 3     | 8      | Compares R0 with an immediate value and sets flags, discarding the result.       |

### **8-Bit Arithmetic/Logic Instructions**

**These operate on the lower 8 bits of the specified registers. The upper 8 bits of the destination register are unaffected. R0.b is the implicit accumulator.**

| Mnemonic     | Operands | Bytes | Cycles | Description                                          |
| :----------- | :------- | :---- | :----- | :--------------------------------------------------- |
| ADD.b rs     | R1       | 1     | 4      | R0.b = R0.b + R1.b.                                  |
| ADD.b r, n8 | R0, 0x10 | 2     | 8      | R0.b = R0.b + 0x10.                                  |
| SUB.b rs     | R1       | 1     | 4      | R0.b = R0.b - R1.b.                                  |
| AND.b rs     | R1       | 1     | 4      | R0.b = R0.b & R1.b.                                  |
| OR.b rs      | R1       | 1     | 4      | R0.b = R0.b | R1.b.                                  |
| XOR.b rs     | R1       | 1     | 4      | R0.b = R0.b ^ R1.b.                                  |
| INC.b r      | R2       | 1     | 4      | R2.b = R2.b + 1.                                     |
| DEC.b r      | R2       | 1     | 4      | R2.b = R2.b - 1.                                     |
| CMP.b rs     | R1       | 1     | 4      | Compares the low bytes of R0 and R1.                 |
| CMP.b r, n8  | R0, 0x40 | 2     | 8      | Compares the low byte of R0 with an immediate value. |

### **Rotate, Shift, and Bit Instructions**

| Mnemonic  | Operands | Bytes | Cycles | Description                                              |
| :-------- | :------- | :---- | :----- | :------------------------------------------------------- |
| SHL r     | R0       | 1     | 4      | Shift Left Logical. C <- MSB <- ... <- LSB <- 0.         |
| SHR r     | R0       | 1     | 4      | Shift Right Logical. 0 -> MSB -> ... -> LSB -> C.        |
| ROL r     | R0       | 1     | 4      | Rotate Left through Carry. C <- MSB <- ... <- LSB <- C.  |
| ROR r     | R0       | 1     | 4      | Rotate Right through Carry. C -> MSB -> ... -> LSB -> C. |
| BIT r, n8 | R0, 7    | 2     | 8      | Test bit n8 of register R0. Sets Z flag if bit is 0.     |

### **Control Flow Instructions (Jumps, Calls, Returns)**

| Mnemonic   | Operands  | Bytes | Cycles | Description                                                                          |
| :--------- | :-------- | :---- | :----- | :----------------------------------------------------------------------------------- |
| JMP n16    | 0x1234    | 3     | 12     | Unconditional jump to absolute address n16.                                          |
| JMP (r)    | (R0)      | 1     | 8      | Unconditional jump to address stored in register r.                                  |
| JR n8      | $10       | 2     | 8      | Unconditional relative jump by signed offset n8.                                     |
| Jcc n16    | Z, 0x1234 | 3     | 12/8   | Conditional jump to n16 if condition cc is met. (12 if jump taken, 8 if not).        |
| JRcc n8    | NZ, -$4   | 2     | 8/4    | Conditional relative jump by n8 if condition cc is met. (8 if jump taken, 4 if not). |
| CALL n16   | 0x2000    | 3     | 20     | Call subroutine at address n16. Pushes PC+3 onto stack.                              |
| CALLcc n16 | C, 0x2000 | 3     | 20/8   | Conditional call if condition cc is met.                                             |
| RET        |           | 1     | 16     | Return from subroutine. Pops PC from stack.                                          |
| RETI       |           | 1     | 16     | Return from interrupt. Pops PC from stack and enables interrupts. See `Interrupts.md` for details. |

#### **Note on Condition Codes (cc)**

**The cc in instructions like Jcc is a placeholder for a specific condition. To form a valid instruction, replace cc with a code corresponding to a CPU flag. For example, Jcc becomes JZ (Jump if Zero), JNC (Jump if No Carry), or JV (Jump if Overflow). The available codes are: Z, NZ, C, NC, N, NN, V, NV.**

### **CPU Control Instructions**

| Mnemonic | Operands | Bytes | Cycles | Description                                          |
| :------- | :------- | :---- | :----- | :--------------------------------------------------- |
| NOP      |          | 1     | 4      | No operation. Wastes 4 cycles.                       |
| HALT     |          | 1     | 4      | Halts CPU until an interrupt occurs. Low power mode. |
| DI       |          | 1     | 4      | Disable interrupts.                                  |
| EI       |          | 1     | 4      | Enable interrupts.                                   |
