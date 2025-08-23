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
- **Prefixes:** `0xFD` accesses the "Advanced Addressing 1" instruction map. `0xFE` accesses the "Bit, Byte, and Shift Operations" instruction map. `0xFF` accesses the "Advanced Addressing 2" instruction map.

## **3. Instruction Set**

### **Load/Store Instructions**

**These instructions move data between registers and memory. Memory is byte-addressable. 16-bit word accesses must be aligned to an even address. Attempting an unaligned 16-bit access will trigger a Bus Error fault.**

| Mnemonic          | Operands       | Bytes | Cycles | Description                                                                                               |
| :---------------- | :------------- | :---- | :----- | :-------------------------------------------------------------------------------------------------------- |
| LD rd, rs         | R1, R2         | 1     | 4      | Copies the 16-bit value of R2 into R1.                                                                    |
| LDI r, n16        | R0, 0x8000     | 3     | 8      | Loads the immediate 16-bit value 0x8000 into R0.                                                          |
| LD.w rd, (rs)     | R2, (R3)       | 2     | 12     | Loads the 16-bit word from the address in R3 into R2. (Prefixed)                                          |
| LD.b rd, (rs)     | R2, (R3)       | 2     | 12     | Loads the 8-bit byte from the address in R3 into the low byte of R2, and zero-extends it. (Prefixed)      |
| ST.w (rd), rs     | (R4), R5       | 2     | 12     | Stores the 16-bit word from R5 into the memory address pointed to by R4. (Prefixed)                       |
| ST.b (rd), rs     | (R4), R5       | 2     | 12     | Stores the low 8-bit byte from R5 into the memory address pointed to by R4. (Prefixed)                    |
| LD.w rd, (rs, n8) | R2, (R3, 0x10) | 3     | 16     | Loads the 16-bit word from the address in R3 + signed 8-bit offset into R2. (Prefixed)                    |
| ST.w (rs, n8), rd | (R4, 0x10), R5 | 3     | 16     | Stores the 16-bit word from R5 into the memory address pointed to by R4 + signed 8-bit offset. (Prefixed) |
| LD.w r, (n16)     | R0, (0xC000)   | 3     | 12     | Loads a 16-bit word from the absolute address 0xC000 into R0.                                             |
| ST.w (n16), r     | (0xC000), R0   | 3     | 12     | Stores the 16-bit word in R0 to the absolute address 0xC000.                                              |
| PUSH r            | R0             | 1     | 12     | Pushes the value of a register onto the stack. Decrements SP by 2.                                        |
| POP r             | R0             | 1     | 12     | Pops a value from the stack into a register. Increments SP by 2.                                          |
| PUSH F            |                | 2     | 12     | Pushes the Flags register (F) onto the stack. Decrements SP by 2. (Prefixed)                              |
| POP F             |                | 2     | 12     | Pops a value from the stack into the Flags register (F). Increments SP by 2. (Prefixed)                   |

### **16-Bit Arithmetic/Logic Instructions**

#### **Accumulator (R0) Arithmetic**
**These instructions use R0 as an implicit accumulator.**

| Mnemonic    | Operands   | Bytes | Cycles | Description                                                                      |
| :---------- | :--------- | :---- | :----- | :------------------------------------------------------------------------------- |
| ADD rs      | R1         | 1     | 4      | R0 = R0 + R1. Affects Z, N, C, V flags.                                          |
| SUB rs      | R1         | 1     | 4      | R0 = R0 - R1. Affects Z, N, C, V flags.                                          |
| AND rs      | R1         | 1     | 4      | R0 = R0 & R1. Affects Z, N flags.                                                |
| OR rs       | R1         | 1     | 4      | R0 = R0 or R1. Affects Z, N flags.                                               |
| XOR rs      | R1         | 1     | 4      | R0 = R0 ^ R1. Affects Z, N flags.                                                |
| CMP rs      | R1         | 1     | 4      | Compares R0 and R1 (calculates R0 - R1) and sets flags, but discards the result. |
| NEG r       | R1         | 1     | 4      | r = -r. Affects Z, N, C, V flags.                                                |

#### **Register-to-Register Arithmetic**

| Mnemonic    | Operands   | Bytes | Cycles | Description                                                                      |
| :---------- | :--------- | :---- | :----- | :------------------------------------------------------------------------------- |
| ADD rd, rs  | R1, R2     | 2     | 8      | rd = rd + rs. Affects Z, N, C, V flags.                                          |
| SUB rd, rs  | R1, R2     | 2     | 8      | rd = rd - rs. Affects Z, N, C, V flags.                                          |
| AND rd, rs  | R1, R2     | 2     | 8      | rd = rd & rs. Affects Z, N flags.                                                |
| OR rd, rs   | R1, R2     | 2     | 8      | rd = rd or rs. Affects Z, N flags.                                              |
| XOR rd, rs  | R1, R2     | 2     | 8      | rd = rd ^ rs. Affects Z, N flags.                                                |
| CMP rd, rs  | R1, R2     | 2     | 8      | Compares rd and rs and sets flags, but discards the result.                      |
| ADC rd, rs  | R1, R2     | 2     | 8      | rd = rd + rs + C. Affects Z, N, C, V flags.                                      |
| SBC rd, rs  | R1, R2     | 2     | 8      | rd = rd - rs - C. Affects Z, N, C, V flags.                                      |

#### **Immediate Arithmetic**

| Mnemonic    | Operands   | Bytes | Cycles | Description                                                                      |
| :---------- | :--------- | :---- | :----- | :------------------------------------------------------------------------------- |
| ADDI r, n16 | R1, 0x100  | 4     | 12     | r = r + n16. Affects Z, N, C, V flags.                                           |
| SUBI r, n16 | R1, 0x100  | 4     | 12     | r = r - n16. Affects Z, N, C, V flags.                                           |
| ANDI r, n16 | R1, 0xFF   | 4     | 12     | r = r & n16. Affects Z, N flags.                                                 |
| ORI r, n16  | R1, 0xF0F0 | 4     | 12     | r = r or n16. Affects Z, N flags.                                                |
| XORI r, n16 | R1, 0xFFFF | 4     | 12     | r = r ^ n16. Affects Z, N flags.                                                 |
| CMPI r, n16 | R1, 0x4000 | 4     | 12     | Compares r with n16 and sets flags, discarding the result.                       |
| INC r       | R2         | 1     | 4      | R2 = R2 + 1. Affects Z, N, V flags.                                              |
| DEC r       | R2         | 1     | 4      | R2 = R2 - 1. Affects Z, N, V flags.                                              |

#### **Accumulator-Immediate Arithmetic**

| Mnemonic    | Operands   | Bytes | Cycles | Description                                                                      |
| :---------- | :--------- | :---- | :----- | :------------------------------------------------------------------------------- |
| ADDI n16    | 0x100      | 3     | 8      | R0 = R0 + n16. Affects Z, N, C, V flags.                                         |
| SUBI n16    | 0x100      | 3     | 8      | R0 = R0 - n16. Affects Z, N, C, V flags.                                         |
| ANDI n16    | 0xFF       | 3     | 8      | R0 = R0 & n16. Affects Z, N flags.                                               |
| ORI n16     | 0xF0F0     | 3     | 8      | R0 = R0 or n16. Affects Z, N flags.                                              |
| XORI n16    | 0xFFFF     | 3     | 8      | R0 = R0 ^ n16. Affects Z, N flags.                                               |
| CMPI n16    | 0x4000     | 3     | 8      | Compares R0 with n16 and sets flags, discarding the result.                      |
| ADDCI n16   | 0x100      | 3     | 8      | R0 = R0 + n16 + C. Affects Z, N, C, V flags.                                     |
| SUBCI n16   | 0x100      | 3     | 8      | R0 = R0 - n16 - C. Affects Z, N, C, V flags.                                     |

### **8-Bit Arithmetic/Logic Instructions**

**These operate on the lower 8 bits of the specified registers. The upper 8 bits of the destination register are unaffected. R0.b is the implicit accumulator. All are prefixed instructions.**

| Mnemonic    | Operands | Bytes | Cycles | Description                                          |
| :---------- | :------- | :---- | :----- | :--------------------------------------------------- |
| ADD.b rs    | R1       | 2     | 8      | R0.b = R0.b + R1.b.                                  |
| SUB.b rs    | R1       | 2     | 8      | R0.b = R0.b - R1.b.                                  |
| AND.b rs    | R1       | 2     | 8      | R0.b = R0.b & R1.b.                                  |
| OR.b rs     | R1       | 2     | 8      | R0.b = R0.b or R1.b.                                 |
| XOR.b rs    | R1       | 2     | 8      | R0.b = R0.b ^ R1.b.                                  |
| CMP.b rs    | R1       | 2     | 8      | Compares the low bytes of R0 and R1.                 |

### **Rotate, Shift, and Bit Instructions**

| Mnemonic  | Operands | Bytes | Cycles | Description                                                         |
| :-------- | :------- | :---- | :----- | :------------------------------------------------------------------ |
| SHL r     | R0       | 2     | 8      | Shift Left Logical. C <- MSB <- ... <- LSB <- 0. (Prefixed)         |
| SHR r     | R0       | 2     | 8      | Shift Right Logical. 0 -> MSB -> ... -> LSB -> C. (Prefixed)        |
| SRA r     | R0       | 2     | 8      | Shift Right Arithmetic. MSB -> MSB -> ... -> LSB -> C. (Prefixed)   |
| ROL r     | R0       | 2     | 8      | Rotate Left through Carry. C <- MSB <- ... <- LSB <- C. (Prefixed)  |
| ROR r     | R0       | 2     | 8      | Rotate Right through Carry. C -> MSB -> ... -> LSB -> C. (Prefixed) |
| BIT r, b  | R0, 7    | 2     | 8      | Test bit b (0-7) of register r's low byte. Sets Z flag if bit is 0. (Prefixed) |
| SET r, b  | R0, 7    | 2     | 8      | Set bit b (0-7) of register r's low byte to 1. (Prefixed)           |
| RES r, b  | R0, 7    | 2     | 8      | Reset bit b (0-7) of register r's low byte to 0. (Prefixed)         |
| BIT (n16), b | (0xC000), 7 | 4     | 16     | Test bit b (0-7) of the byte at memory address n16. Sets Z flag if bit is 0. (Prefixed) |
| SET (n16), b | (0xC000), 7 | 4     | 20     | Set bit b (0-7) of the byte at memory address n16 to 1. (Prefixed)           |
| RES (n16), b | (0xC000), 7 | 4     | 20     | Reset bit b (0-7) of the byte at memory address n16 to 0. (Prefixed)         |

### **Control Flow Instructions (Jumps, Calls, Returns)**

| Mnemonic   | Operands  | Bytes | Cycles | Description                                                                                        |
| :--------- | :-------- | :---- | :----- | :------------------------------------------------------------------------------------------------- |
| JMP n16    | 0x1234    | 3     | 12     | Unconditional jump to absolute address n16.                                                        |
| JMP (r)    | (R0)      | 1     | 8      | Unconditional jump to address stored in register r.                                                |
| JR n8      | $10       | 2     | 8      | Unconditional relative jump by signed offset n8.                                                   |
| Jcc n16    | Z, 0x1234 | 3     | 12/8   | Conditional jump to n16 if condition cc is met. (12 if jump taken, 8 if not).                      |
| JRcc n8    | NZ, -$4   | 2     | 8/4    | Conditional relative jump by n8 if condition cc is met. (8 if jump taken, 4 if not).               |
| CALL n16   | 0x2000    | 3     | 20     | Call subroutine at address n16. Pushes PC+3 onto stack.                                            |
| CALL (r)   | (R0)      | 1     | 16     | Call subroutine at address in register r. Pushes PC+1 onto stack.                                  |
| CALLcc n16 | C, 0x2000 | 3     | 20/8   | Conditional call if condition cc is met.                                                           |
| RET        |           | 1     | 16     | Return from subroutine. Pops PC from stack.                                                        |
| RETI       |           | 1     | 16     | Return from interrupt. Pops PC from stack and enables interrupts. See `Interrupts.md` for details. |

#### **Note on Condition Codes (cc)**

**The cc in instructions like Jcc is a placeholder for a specific condition. To form a valid instruction, replace cc with a code corresponding to a CPU flag. For example, Jcc becomes JZ (Jump if Zero), JNC (Jump if No Carry), or JV (Jump if Overflow). The available codes are: Z, NZ, C, NC, N, NN, V, NV.**

### **CPU Control Instructions**

| Mnemonic | Operands | Bytes | Cycles | Description                                          |
| :------- | :------- | :---- | :----- | :--------------------------------------------------- |
| NOP      |          | 1     | 4      | No operation. Wastes 4 cycles.                       |
| HALT     |          | 2     | 8      | Halts CPU until an interrupt occurs. Low power mode. (Prefixed) |
| ID       |          | 1     | 4      | Disable interrupts.                                  |
| IE       |          | 1     | 4      | Enable interrupts.                                   |
