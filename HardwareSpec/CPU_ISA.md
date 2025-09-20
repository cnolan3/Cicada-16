# **Cicada-16 CPU - Instruction Set Architecture (ISA)**

**This document outlines the instruction set for the "Cicada-16" fantasy console's main processor. The architecture is a 16-bit design, intended to be reminiscent of late 80s / early 90s home consoles. It features a clean register set and instructions that operate on 16-bit words by default, with support for 8-bit byte manipulation.**

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

### **1.1. Execution and Timing**

The master system clock frequency is **8.388608 MHz** (2^23 Hz).

The Cicada-16 CPU features a simple **2-stage instruction pipeline** to improve performance. The two stages are:

1.  **Fetch:** The CPU fetches the next instruction from memory.
2.  **Execute:** The CPU decodes and executes the current instruction.

This allows the CPU to fetch one instruction while executing the previous one, overlapping operations and increasing instruction throughput.

The core timing of the system is measured in two units:

- **T-cycles (Clock Cycles):** This is the fundamental clock rate of the system, driven by the main crystal oscillator.
- **M-cycles (Machine Cycles):** One M-cycle consists of **4 T-cycles**. M-cycles are the basic unit of time used for instruction execution. A simple operation, like a register-to-register move, might take one M-cycle (4 T-cycles), while a more complex instruction that requires multiple memory accesses will take several M-cycles.

All instruction cycle counts in this document are given in **T-cycles**.

## **2. Instruction Notation**

- **r, rs, rd:** Any 16-bit general purpose register (R0-R7)
- **n16:** An immediate 16-bit value (e.g., 0x1234)
- **n8:** An immediate 8-bit value (e.g., 0x2A)
- **(addr):** The value stored at the memory address addr.
- **Default Bit Width:** Most instructions are word (16-bit) operations. This is the default for most instructions.
- **.b:** Suffix for a byte (8-bit) operation.
- **cc:** Condition code for jumps/calls (Z, NZ, C, NC, N, NN, V, NV)
- **Prefixes:** `0xFD` accesses the "Bit, Byte, and Shift Operations" instruction map. `0xFE` accesses the "Advanced Addressing 1" instruction map. `0xFF` accesses the "Advanced Addressing 2" instruction map.

## **3. Instruction Set**

### **Load/Store Instructions**

**These instructions move data between registers and memory. Memory is byte-addressable. 16-bit word accesses must be aligned to an even address. Attempting an unaligned 16-bit access will trigger a Bus Error fault. Cycle counts are listed as `HRAM/Other` for instructions that can target memory (HRAM = `FE00-FFFF`; Other = all non-HRAM regions). If only one value is shown, the instruction does not access HRAM-addressable memory.**

| Mnemonic         | Operands       | Bytes | Cycles | Description                                                                                               |
| :--------------- | :------------- | :---- | :----- | :-------------------------------------------------------------------------------------------------------- |
| LD rd, rs        | R1, R2         | 1     | 4      | Copies the 16-bit value of R2 into R1.                                                                    |
| LDI r, n16       | R0, 0x8000     | 3     | 12     | Loads the immediate 16-bit value 0x8000 into R0.                                                          |
| LD rd, (rs)      | R2, (R3)       | 2     | 12/16  | Loads the 16-bit word from the address in R3 into R2. (Prefixed)                                          |
| LD.b rd, (rs)    | R2, (R3)       | 2     | 10/12  | Loads the 8-bit byte from the address in R3 into the low byte of R2, and zero-extends it. (Prefixed)      |
| ST (rd), rs      | (R4), R5       | 2     | 12/16  | Stores the 16-bit word from R5 into the memory address pointed to by R4. (Prefixed)                       |
| ST.b (rd), rs    | (R4), R5       | 2     | 10/12  | Stores the low 8-bit byte from R5 into the memory address pointed to by R4. (Prefixed)                    |
| LD rd, (rs, n8)  | R2, (R3, 0x10) | 3     | 16/20  | Loads the 16-bit word from the address in R3 + signed 8-bit offset into R2. (Prefixed)                    |
| ST (rd, n8), rs  | (R4, 0x10), R5 | 3     | 16/20  | Stores the 16-bit word from R5 into the memory address pointed to by R4 + signed 8-bit offset. (Prefixed) |
| LEA rd, (rs, n8) | R1, (R2, 0x10) | 3     | 12     | Calculates the address R2 + signed 8-bit offset and stores it in R1. (Prefixed)                           |
| LD r, (n16)      | R0, (0xC000)   | 3     | 16/20  | Loads a 16-bit word from the absolute address 0xC000 into R0.                                             |
| ST (n16), r      | (0xC000), R0   | 3     | 16/20  | Stores the 16-bit word in R0 to the absolute address 0xC000.                                              |
| LDI.b rd, n8     | R0, 0xAB       | 3     | 12     | Loads the immediate 8-bit value into the low byte of rd, and zero-extends it. (Prefixed)                  |
| LD rd, (rs)+     | R0, (R1)+      | 3     | 16/20  | Loads a word from the address in rs into rd, then increments rs by 2. (Prefixed)                          |
| ST (rd)+, rs     | (R1)+, R0      | 3     | 16/20  | Stores the word from rs to the address in rd, then increments rd by 2. (Prefixed)                         |
| LD.b rd, (rs)+   | R0, (R1)+      | 3     | 14/16  | Loads a byte from the address in rs into rd, zero-extends it, then increments rs by 1. (Prefixed)         |
| ST.b (rd)+, rs   | (R1)+, R0      | 3     | 14/16  | Stores the low byte from rs to the address in rd, then increments rs by 1. (Prefixed)                     |
| LD rd, -(rs)     | R0, -(R1)      | 3     | 16/20  | Decrements rs by 2, then loads a word from the new address in rs into rd. (Prefixed)                      |
| ST -(rd), rs     | -(R1), R0      | 3     | 16/20  | Decrements rd by 2, then stores the word from rs to the new address in rd. (Prefixed)                     |
| LD.b rd, -(rs)   | R0, -(R1)      | 3     | 14/16  | Decrements rs by 1, then loads a byte from the new address in rs into rd and zero-extends it. (Prefixed)  |
| ST.b -(rd), rs   | -(R1), R0      | 3     | 14/16  | Decrements rd by 1, then stores the low byte from rs to the new address in rd. (Prefixed)                 |
| PUSH r           | R0             | 1     | 12     | Pushes the value of a register onto the stack. Decrements SP by 2.                                        |
| POP r            | R0             | 1     | 12     | Pops a value from the stack into a register. Increments SP by 2.                                          |
| PUSH n16         | 0x1234         | 3     | 20     | Pushes an immediate 16-bit value onto the stack. Decrements SP by 2.                                      |
| PUSH F           |                | 1     | 12     | Pushes the Flags register (F) onto the stack. Decrements SP by 2.                                         |
| POP F            |                | 1     | 12     | Pops a value from the stack into the Flags register (F). Increments SP by 2.                              |

#### **Post-Increment and Pre-Decrement Addressing**

The post-increment and pre-decrement addressing modes are powerful features for iterating through data structures in memory.

- **Post-increment `(rs)+`**: The instruction first uses the address currently in the register `rs` to perform the load or store operation. After the operation is complete, the value of `rs` is automatically incremented. For word operations, it is incremented by 2, and for byte operations (`.b`), it is incremented by 1.
- **Pre-decrement `-(rs)`**: The instruction first decrements the value in the register `rs`. For word operations, it is decremented by 2, and for byte operations (`.b`), it is decremented by 1. The instruction then uses the _new_ address in `rs` to perform the load or store operation.

### **16-Bit Arithmetic/Logic Instructions**

#### **Accumulator (R0) Arithmetic**

**These instructions use R0 as an implicit accumulator.**

| Mnemonic | Operands | Bytes | Cycles | Description                                                                      |
| :------- | :------- | :---- | :----- | :------------------------------------------------------------------------------- |
| ADD rs   | R1       | 1     | 4      | R0 = R0 + R1. Affects Z, N, C, V flags.                                          |
| SUB rs   | R1       | 1     | 4      | R0 = R0 - R1. Affects Z, N, C, V flags.                                          |
| AND rs   | R1       | 1     | 4      | R0 = R0 & R1. Affects Z, N flags.                                                |
| OR rs    | R1       | 1     | 4      | R0 = R0 or R1. Affects Z, N flags.                                               |
| XOR rs   | R1       | 1     | 4      | R0 = R0 ^ R1. Affects Z, N flags.                                                |
| CMP rs   | R1       | 1     | 4      | Compares R0 and R1 (calculates R0 - R1) and sets flags, but discards the result. |
| NEG      |          | 1     | 4      | R0 = -R0 (two's complement). Affects Z, N, C, V flags.                           |
| NOT      |          | 1     | 4      | R0 = !R0 (bitwise NOT). Affects Z, N flags.                                      |
| SWAP     |          | 1     | 4      | Swaps the upper and lower bytes of R0. Affects Z, N flags.                       |

#### **Register-to-Register Arithmetic**

| Mnemonic   | Operands | Bytes | Cycles | Description                                                 |
| :--------- | :------- | :---- | :----- | :---------------------------------------------------------- |
| ADD rd, rs | R1, R2   | 2     | 8      | rd = rd + rs. Affects Z, N, C, V flags.                     |
| SUB rd, rs | R1, R2   | 2     | 8      | rd = rd - rs. Affects Z, N, C, V flags.                     |
| AND rd, rs | R1, R2   | 2     | 8      | rd = rd & rs. Affects Z, N flags.                           |
| OR rd, rs  | R1, R2   | 2     | 8      | rd = rd or rs. Affects Z, N flags.                          |
| XOR rd, rs | R1, R2   | 2     | 8      | rd = rd ^ rs. Affects Z, N flags.                           |
| CMP rd, rs | R1, R2   | 2     | 8      | Compares rd and rs and sets flags, but discards the result. |
| ADC rd, rs | R1, R2   | 2     | 8      | rd = rd + rs + C. Affects Z, N, C, V flags.                 |
| SBC rd, rs | R1, R2   | 2     | 8      | rd = rd - rs - C. Affects Z, N, C, V flags.                 |

#### **Immediate Arithmetic**

| Mnemonic    | Operands   | Bytes | Cycles | Description                                                |
| :---------- | :--------- | :---- | :----- | :--------------------------------------------------------- |
| ADDI r, n16 | R1, 0x100  | 4     | 16     | r = r + n16. Affects Z, N, C, V flags.                     |
| SUBI r, n16 | R1, 0x100  | 4     | 16     | r = r - n16. Affects Z, N, C, V flags.                     |
| ANDI r, n16 | R1, 0xFF   | 4     | 16     | r = r & n16. Affects Z, N flags.                           |
| ORI r, n16  | R1, 0xF0F0 | 4     | 16     | r = r or n16. Affects Z, N flags.                          |
| XORI r, n16 | R1, 0xFFFF | 4     | 16     | r = r ^ n16. Affects Z, N flags.                           |
| CMPI r, n16 | R1, 0x4000 | 4     | 16     | Compares r with n16 and sets flags, discarding the result. |
| ADD SP, n8s | SP, -16    | 2     | 8      | Adds a signed 8-bit immediate to the stack pointer (R7).   |
| INC r       | R2         | 1     | 4      | R2 = R2 + 1. Affects Z, N, V flags.                        |
| DEC r       | R2         | 1     | 4      | R2 = R2 - 1. Affects Z, N, V flags.                        |

#### **Accumulator-Immediate Arithmetic**

| Mnemonic  | Operands | Bytes | Cycles | Description                                                 |
| :-------- | :------- | :---- | :----- | :---------------------------------------------------------- |
| ADDI n16  | 0x100    | 3     | 12     | R0 = R0 + n16. Affects Z, N, C, V flags.                    |
| SUBI n16  | 0x100    | 3     | 12     | R0 = R0 - n16. Affects Z, N, C, V flags.                    |
| ANDI n16  | 0xFF     | 3     | 12     | R0 = R0 & n16. Affects Z, N flags.                          |
| ORI n16   | 0xF0F0   | 3     | 12     | R0 = R0 or n16. Affects Z, N flags.                         |
| XORI n16  | 0xFFFF   | 3     | 12     | R0 = R0 ^ n16. Affects Z, N flags.                          |
| CMPI n16  | 0x4000   | 3     | 12     | Compares R0 with n16 and sets flags, discarding the result. |
| ADDCI n16 | 0x100    | 3     | 12     | R0 = R0 + n16 + C. Affects Z, N, C, V flags.                |
| SUBCI n16 | 0x100    | 3     | 12     | R0 = R0 - n16 - C. Affects Z, N, C, V flags.                |

### **8-Bit Arithmetic/Logic Instructions**

**These operate on the lower 8 bits of the specified registers. The upper 8 bits of the destination register are unaffected. R0.b is the implicit accumulator. All are prefixed instructions.**

| Mnemonic | Operands | Bytes | Cycles | Description                          |
| :------- | :------- | :---- | :----- | :----------------------------------- |
| ADD.b rs | R1       | 2     | 8      | R0.b = R0.b + R1.b.                  |
| SUB.b rs | R1       | 2     | 8      | R0.b = R0.b - R1.b.                  |
| AND.b rs | R1       | 2     | 8      | R0.b = R0.b & R1.b.                  |
| OR.b rs  | R1       | 2     | 8      | R0.b = R0.b or R1.b.                 |
| XOR.b rs | R1       | 2     | 8      | R0.b = R0.b ^ R1.b.                  |
| CMP.b rs | R1       | 2     | 8      | Compares the low bytes of R0 and R1. |

### **Rotate, Shift, and Bit Instructions**

| Mnemonic     | Operands    | Bytes | Cycles | Description                                                                               |
| :----------- | :---------- | :---- | :----- | :---------------------------------------------------------------------------------------- |
| SHL r        | R0          | 2     | 8      | Shift Left Logical. C <- MSB <- ... <- LSB <- 0. (Prefixed)                               |
| SHR r        | R0          | 2     | 8      | Shift Right Logical. 0 -> MSB -> ... -> LSB -> C. (Prefixed)                              |
| SRA r        | R0          | 2     | 8      | Shift Right Arithmetic. MSB -> MSB -> ... -> LSB -> C. (Prefixed)                         |
| ROL r        | R0          | 2     | 8      | Rotate Left through Carry. C <- MSB <- ... <- LSB <- C. (Prefixed)                        |
| ROR r        | R0          | 2     | 8      | Rotate Right through Carry. C -> MSB -> ... -> LSB -> C. (Prefixed)                       |
| BIT r, b     | R0, 7       | 3     | 12     | Test bit b (0-7) of register r's low byte. Sets Z flag if bit is 0. (Prefixed)            |
| SET r, b     | R0, 7       | 3     | 12     | Set bit b (0-7) of register r's low byte to 1. (Prefixed)                                 |
| RES r, b     | R0, 7       | 3     | 12     | Reset bit b (0-7) of register r's low byte to 0. (Prefixed)                               |
| BIT (n16), b | (0xC000), 7 | 4     | 18/20  | Test bit b (0-7) of the byte at memory address n16. Sets Z flag if bit is 0. (Prefixed)   |
| SET (n16), b | (0xC000), 7 | 4     | 22/24  | Set bit b (0-7) of the byte at memory address n16 to 1. (Prefixed)                        |
| RES (n16), b | (0xC000), 7 | 4     | 22/24  | Reset bit b (0-7) of the byte at memory address n16 to 0. (Prefixed)                      |
| BIT (rs), b  | (R0), 15    | 3     | 16/20  | Test bit b (0-7) of the byte at memory address in rs. Sets Z flag if bit is 0. (Prefixed) |
| SET (rs), b  | (R0), 15    | 3     | 20/28  | Set bit b (0-7) of the byte at memory address in rs to 1. (Prefixed)                      |
| RES (rs), b  | (R0), 15    | 3     | 20/28  | Reset bit b (0-7) of the byte at memory address in rs to 0. (Prefixed)                    |

### **Control Flow Instructions (Jumps, Calls, Returns)**

| Mnemonic   | Operands  | Bytes | Cycles | Description                                                                                                                        |
| :--------- | :-------- | :---- | :----- | :--------------------------------------------------------------------------------------------------------------------------------- |
| JMP n16    | 0x1234    | 3     | 12     | Unconditional jump to absolute address n16.                                                                                        |
| JMP (r)    | (R0)      | 1     | 4      | Unconditional jump to address stored in register r.                                                                                |
| JR n8      | $10       | 2     | 8      | Unconditional relative jump by signed offset n8.                                                                                   |
| Jcc n16    | Z, 0x1234 | 3     | 12     | Conditional jump to n16 if condition cc is met.                                                                                    |
| JRcc n8    | NZ, -$4   | 2     | 8      | Conditional relative jump by n8 if condition cc is met.                                                                            |
| DJNZ n8    | -$4       | 2     | 8      | Decrement R5 and jump relative if not zero.                                                                                        |
| CALL n16   | 0x2000    | 3     | 20     | Call subroutine at address n16. Pushes PC+3 onto stack.                                                                            |
| CALL (r)   | (R0)      | 1     | 12     | Call subroutine at address in register r. Pushes PC+1 onto stack.                                                                  |
| CALLcc n16 | C, 0x2000 | 3     | 12/20  | Conditional call if condition cc is met.                                                                                           |
| SYSCALL n8 | 0x1A      | 2     | 24     | Calls a system library routine. Pushes PC, then pushes F, then jumps to the address from the system vector table at index n8.      |
| RET        |           | 1     | 12     | Return from subroutine. Pops PC from stack.                                                                                        |
| RETI       |           | 1     | 12     | Return from interrupt. Pops flags from the stack, then pops PC from stack and enables interrupts. See `Interrupts.md` for details. |
| ENTER      |           | 1     | 12     | Creates a new stack frame. See explanation below.                                                                                  |
| LEAVE      |           | 1     | 12     | Destroys the current stack frame and prepares for return.                                                                          |

#### **Stack Frame Management (ENTER/LEAVE)**

The `ENTER` and `LEAVE` instructions simplify the creation and destruction of stack frames for subroutines, a common practice in compiled languages like C. They automate the process of managing the frame pointer (R6).

- **`ENTER`**: This instruction creates a new stack frame. It performs two actions:

  1.  It pushes the current value of the frame pointer (R6) onto the stack.
  2.  It copies the current value of the stack pointer (R7) into the frame pointer (R6).
      This is equivalent to `PUSH R6` followed by `LD R6, R7`. This establishes a new frame, and local variables for the subroutine can then be accessed via negative offsets from R6.

- **`LEAVE`**: This instruction destroys the current stack frame to return to the caller's frame. It performs two actions:
  1.  It copies the value of the frame pointer (R6) into the stack pointer (R7). This deallocates any space used by local variables.
  2.  It pops the previous frame pointer from the stack back into R6.
      This is equivalent to `LD R7, R6` followed by `POP R6`. After a `LEAVE`, a `RET` instruction is typically used to return to the caller.

#### **System Calls (SYSCALL)**

The `SYSCALL` instruction provides a standardized way for user programs to request services from the system library or operating system kernel. This is the primary mechanism for interacting with hardware peripherals, managing files, or performing other privileged operations.

When `SYSCALL n8` is executed, the CPU performs the following sequence:

1.  It pushes the address of the _next_ instruction (PC) onto the stack. This allows the system routine to return control to the user program.
2.  It pushes the 16-bit Flags register (F) onto the stack.
3.  It looks up the address of the system routine in a vector table located in low memory. The `n8` immediate value is an index into this table.
4.  It sets the PC to the address fetched from the vector table, effectively transferring control to the system routine.

System routines are expected to finish with a `RETI` (Return from Interrupt) instruction, which restores the flags and the program counter, resuming the user program.

#### **Note on Condition Codes (cc)**

**The cc in instructions like Jcc is a placeholder for a specific condition. To form a valid instruction, replace cc with a code corresponding to a CPU flag. For example, Jcc becomes JZ (Jump if Zero), JNC (Jump if No Carry), or JV (Jump if Overflow). The available codes are: Z, NZ, C, NC, N, NN, V, NV.**

### **CPU Control Instructions**

| Mnemonic | Operands | Bytes | Cycles | Description                                          |
| :------- | :------- | :---- | :----- | :--------------------------------------------------- |
| NOP      |          | 1     | 4      | No operation. Wastes 4 cycles.                       |
| HALT     |          | 1     | 4      | Halts CPU until an interrupt occurs. Low power mode. |
| DI       |          | 1     | 4      | Disable interrupts.                                  |
| EI       |          | 1     | 4      | Enable interrupts.                                   |
| CCF      |          | 1     | 4      | Complement carry flag. N flag is reset.              |
| SCF      |          | 1     | 4      | Set carry flag to 1. N flag is reset.                |
| RCF      |          | 1     | 4      | Reset carry flag to 0. N flag is reset.              |

---

© 2025 Connor Nolan. This work is licensed under a
[Creative Commons Attribution-ShareAlike 4.0 International License](http://creativecommons.org/licenses/by-sa/4.0/).
