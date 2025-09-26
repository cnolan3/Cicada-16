# Cicada-16 Instruction Set Reference

This document provides a detailed reference for every instruction in the Cicada-16 assembly language, grouped by category.

## Data Transfer Instructions

These instructions are used to move data between registers and memory.

| Mnemonic | Syntax                 | Description                                                                              | Size (bytes) |
| -------- | ---------------------- | ---------------------------------------------------------------------------------------- | ------------ |
| `LD`     | `LD Rd, Rs`            | Copies the value from register `Rs` to `Rd`.                                             | 1            |
| `LD`     | `LD Rd, (Rs)`          | Loads a 16-bit word from the address in `Rs` into `Rd`. (Indirect)                       | 2            |
| `LD`     | `LD Rd, (addr)`        | Loads a 16-bit word from the absolute `addr` into `Rd`.                                  | 3            |
| `LD`     | `LD Rd, (Rs, offset)`  | Loads a 16-bit word from `Rs + offset` into `Rd`. (Indexed)                              | 3            |
| `LD`     | `LD Rd, (Rs)+`         | Loads a 16-bit word from the address in `Rs` into `Rd`, then increments `Rs` by 2.       | 3            |
| `LD`     | `LD Rd, -(Rs)`         | Decrements `Rs` by 2, then loads a 16-bit word from the new address in `Rs` into `Rd`.   | 3            |
| `LDI`    | `LDI Rd, imm16`        | Loads an immediate 16-bit value into `Rd`.                                               | 3            |
| `LD.b`   | `LD.b Rd, (Rs)`        | Loads an 8-bit byte from the address in `Rs` into the lower byte of `Rd`.                | 2            |
| `LD.b`   | `LD.b Rd, (Rs)+`       | Loads a byte from the address in `Rs` into `Rd`, then increments `Rs` by 1.              | 3            |
| `LD.b`   | `LD.b Rd, -(Rs)`       | Decrements `Rs` by 1, then loads a byte from the new address in `Rs` into `Rd`.          | 3            |
| `LDI.b`  | `LDI.b Rd, imm8`       | Loads an immediate 8-bit value into the lower byte of `Rd`.                              | 3            |
| `ST`     | `ST (Rd), Rs`          | Stores the 16-bit value from `Rs` at the address in `Rd`. (Indirect)                     | 2            |
| `ST`     | `ST (addr), Rs`        | Stores the 16-bit value from `Rs` at the absolute `addr`.                                | 3            |
| `ST`     | `ST (Rd, offset), Rs`  | Stores the 16-bit value from `Rs` at the address `Rd + offset`. (Indexed)                | 3            |
| `ST`     | `ST (Rd)+, Rs`         | Stores the 16-bit value from `Rs` at the address in `Rd`, then increments `Rd` by 2.     | 3            |
| `ST`     | `ST -(Rd), Rs`         | Decrements `Rd` by 2, then stores the 16-bit value from `Rs` at the new address in `Rd`. | 3            |
| `ST.b`   | `ST.b (Rd), Rs`        | Stores the lower 8-bits from `Rs` at the address in `Rd`.                                | 2            |
| `ST.b`   | `ST.b (Rd)+, Rs`       | Stores the lower byte from `Rs` at the address in `Rd`, then increments `Rd` by 1.       | 3            |
| `ST.b`   | `ST.b -(Rd), Rs`       | Decrements `Rd` by 1, then stores the lower byte from `Rs` at the new address in `Rd`.   | 3            |
| `LEA`    | `LEA Rd, (Rs, offset)` | Loads the effective address `Rs + offset` into `Rd`. Does not access memory.             | 3            |

---

## Arithmetic Instructions

These instructions perform arithmetic operations. Most operations affect the Z, N, C, and V flags.

| Mnemonic       | Syntax           | Description                                                                             | Size (bytes) |
| -------------- | ---------------- | --------------------------------------------------------------------------------------- | ------------ |
| `ADD`          | `ADD Rs`         | Adds `Rs` to `R0` (accumulator). `R0 = R0 + Rs`.                                        | 1            |
| `ADD`          | `ADD Rd, Rs`     | Adds `Rs` to `Rd`. `Rd = Rd + Rs`.                                                      | 2            |
| `ADDI`         | `ADDI imm16`     | Adds an immediate 16-bit value to `R0`. `R0 = R0 + imm16`.                              | 3            |
| `ADDI`         | `ADDI Rd, imm16` | Adds an immediate 16-bit value to `Rd`. `Rd = Rd + imm16`.                              | 4            |
| `ADC`          | `ADC Rs`         | Adds `Rs` and the Carry flag to `R0`. `R0 = R0 + Rs + C`.                               | 1            |
| `ADC`          | `ADC Rd, Rs`     | Adds `Rs` and the Carry flag to `Rd`. `Rd = Rd + Rs + C`.                               | 2            |
| `ADCI`         | `ADCI imm16`     | Adds an immediate value and the Carry flag to `R0`. `R0 = R0 + imm16 + C`.              | 3            |
| `SUB`          | `SUB Rs`         | Subtracts `Rs` from `R0`. `R0 = R0 - Rs`.                                               | 1            |
| `SUB`          | `SUB Rd, Rs`     | Subtracts `Rs` from `Rd`. `Rd = Rd - Rs`.                                               | 2            |
| `SUBI`         | `SUBI imm16`     | Subtracts an immediate value from `R0`. `R0 = R0 - imm16`.                              | 3            |
| `SUBI`         | `SUBI Rd, imm16` | Subtracts an immediate value from `Rd`. `Rd = Rd - imm16`.                              | 4            |
| `SBC`          | `SBC Rs`         | Subtracts `Rs` and the Carry flag from `R0`. `R0 = R0 - Rs - C`.                        | 1            |
| `SBC`          | `SBC Rd, Rs`     | Subtracts `Rs` and the Carry flag from `Rd`. `Rd = Rd - Rs - C`.                        | 2            |
| `SBCI`         | `SBCI imm16`     | Subtracts an immediate value and the Carry flag from `R0`. `R0 = R0 - imm16 - C`.       | 3            |
| `CMP`          | `CMP Rs`         | Compares `R0` with `Rs` by performing `R0 - Rs` and setting flags. Result is discarded. | 1            |
| `CMP`          | `CMP Rd, Rs`     | Compares `Rd` with `Rs` and sets flags.                                                 | 2            |
| `CMPI`         | `CMPI imm16`     | Compares `R0` with an immediate value and sets flags.                                   | 3            |
| `CMPI`         | `CMPI Rd, imm16` | Compares `Rd` with an immediate value and sets flags.                                   | 4            |
| `ADD.b`        | `ADD.b Rs`       | Adds the lower byte of `Rs` to the lower byte of `R0`.                                  | 2            |
| `SUB.b`        | `SUB.b Rs`       | Subtracts the lower byte of `Rs` from the lower byte of `R0`.                           | 2            |
| `CMP.b`        | `CMP.b Rs`       | Compares the lower byte of `R0` with the lower byte of `Rs`.                            | 2            |
| `INC`          | `INC Rd`         | Increments `Rd` by 1.                                                                   | 1            |
| `DEC`          | `DEC Rd`         | Decrements `Rd` by 1.                                                                   | 1            |
| `NEG`          | `NEG`            | Negates the value in `R0` (two's complement).                                           | 1            |
| `ADD SP, imm8` | `ADD SP, imm8`   | Adds a signed 8-bit immediate to the Stack Pointer.                                     | 2            |

---

## Logic and Bitwise Instructions

These instructions perform bitwise logical operations. All operations affect the Z and N flags.

| Mnemonic | Syntax            | Description                                                      | Size (bytes) |
| -------- | ----------------- | ---------------------------------------------------------------- | ------------ |
| `AND`    | `AND Rs`          | Performs a bitwise AND between `R0` and `Rs`. `R0 = R0 & Rs`.    | 1            |
| `AND`    | `AND Rd, Rs`      | `Rd = Rd & Rs`.                                                  | 2            |
| `ANDI`   | `ANDI imm16`      | `R0 = R0 & imm16`.                                               | 3            |
| `ANDI`   | `ANDI Rd, imm16`  | `Rd = Rd & imm16`.                                               | 4            |
| `OR`     | `OR Rs`           | `R0 = R0 \| Rs`.                                                 | 1            |
| `OR`     | `OR Rd, Rs`       | `Rd = Rd \| Rs`.                                                 | 2            |
| `ORI`    | `ORI imm16`       | `R0 = R0 \| imm16`.                                              | 3            |
| `ORI`    | `ORI Rd, imm16`   | `Rd = Rd \| imm16`.                                              | 4            |
| `XOR`    | `XOR Rs`          | `R0 = R0 ^ Rs`.                                                  | 1            |
| `XOR`    | `XOR Rd, Rs`      | `Rd = Rd ^ Rs`.                                                  | 2            |
| `XORI`   | `XORI imm16`      | `R0 = R0 ^ imm16`.                                               | 3            |
| `XORI`   | `XORI Rd, imm16`  | `Rd = Rd ^ imm16`.                                               | 4            |
| `NOT`    | `NOT`             | Performs a bitwise NOT on `R0`. `R0 = ~R0`.                      | 1            |
| `AND.b`  | `AND.b Rs`        | Bitwise AND on the lower bytes of `R0` and `Rs`.                 | 2            |
| `OR.b`   | `OR.b Rs`         | Bitwise OR on the lower bytes of `R0` and `Rs`.                  | 2            |
| `XOR.b`  | `XOR.b Rs`        | Bitwise XOR on the lower bytes of `R0` and `Rs`.                 | 2            |
| `SRA`    | `SRA Rd`          | Arithmetic shift right on `Rd`.                                  | 2            |
| `SHL`    | `SHL Rd`          | Logical shift left on `Rd`.                                      | 2            |
| `SHR`    | `SHR Rd`          | Logical shift right on `Rd`.                                     | 2            |
| `ROL`    | `ROL Rd`          | Rotate left on `Rd`.                                             | 2            |
| `ROR`    | `ROR Rd`          | Rotate right on `Rd`.                                            | 2            |
| `BIT`    | `BIT Rd, bit`     | Tests bit `bit` (0-7) of register `Rd`. Sets Z flag if bit is 0. | 3            |
| `BIT`    | `BIT (addr), bit` | Tests bit `bit` of the byte at the absolute `addr`.              | 4            |
| `BIT`    | `BIT (Rd), bit`   | Tests bit `bit` of the byte at the address in `Rd`.              | 3            |
| `SET`    | `SET Rd, bit`     | Sets bit `bit` of register `Rd`.                                 | 3            |
| `SET`    | `SET (addr), bit` | Sets bit `bit` of the byte at the absolute `addr`.               | 4            |
| `SET`    | `SET (Rd), bit`   | Sets bit `bit` of the byte at the address in `Rd`.               | 3            |
| `RES`    | `RES Rd, bit`     | Resets (clears) bit `bit` of register `Rd`.                      | 3            |
| `RES`    | `RES (addr), bit` | Resets bit `bit` of the byte at the absolute `addr`.             | 4            |
| `RES`    | `RES (Rd), bit`   | Resets bit `bit` of the byte at the address in `Rd`.             | 3            |
| `SWAP`   | `SWAP`            | Swaps the upper and lower bytes of `R0`.                         | 1            |

---

## Control Flow Instructions

These instructions alter the flow of program execution.

| Mnemonic   | Syntax           | Description                                                      | Size (bytes) |
| ---------- | ---------------- | ---------------------------------------------------------------- | ------------ |
| `JMP`      | `JMP addr`       | Unconditionally jumps to the specified address or label.         | 3            |
| `JMP`      | `JMP (Rd)`       | Jumps to the address contained in `Rd`.                          | 1            |
| `JR`       | `JR offset`      | Jumps relative to the current address by a signed 8-bit offset.  | 2            |
| `Jcc`      | `Jcc addr`       | Jumps to `addr` if condition `cc` is met.                        | 3            |
| `JRcc`     | `JRcc offset`    | Jumps relative by `offset` if condition `cc` is met.             | 2            |
| `DJNZ`     | `DJNZ offset`    | Decrements `R0`, and jumps by `offset` if `R0` is not zero.      | 2            |
| `CALL`     | `CALL addr`      | Pushes the return address to the stack and jumps to `addr`.      | 3            |
| `CALL`     | `CALL (Rd)`      | Calls the subroutine at the address in `Rd`.                     | 1            |
| `CALLcc`   | `CALLcc addr`    | Calls subroutine at `addr` if condition `cc` is met.             | 3            |
| `RET`      | `RET`            | Pops the return address from the stack and jumps to it.          | 1            |
| `RETI`     | `RETI`           | Returns from an interrupt service routine.                       | 1            |
| `CALL.far` | `CALL.far label` | Performs a long-distance call to a label in another memory bank. | 8            |
| `JMP.far`  | `JMP.far label`  | Performs a long-distance jump to a label in another memory bank. | 8            |

**Condition Codes (`cc`)**

- `Z`: Zero
- `NZ`: Not Zero
- `C`: Carry
- `NC`: No Carry
- `N`: Negative
- `NN`: Not Negative
- `V`: Overflow
- `NV`: No Overflow

---

## Stack Instructions

| Mnemonic | Syntax       | Description                                                     | Size (bytes) |
| -------- | ------------ | --------------------------------------------------------------- | ------------ |
| `PUSH`   | `PUSH Rs`    | Pushes the value of register `Rs` onto the stack.               | 1            |
| `PUSH`   | `PUSH imm16` | Pushes an immediate 16-bit value onto the stack.                | 3            |
| `POP`    | `POP Rd`     | Pops a 16-bit value from the stack into register `Rd`.          | 1            |
| `PUSHF`  | `PUSHF`      | Pushes the 16-bit flags register `F` onto the stack.            | 1            |
| `POPF`   | `POPF`       | Pops a 16-bit value from the stack into the flags register `F`. | 1            |
| `ENTER`  | `ENTER`      | Standard subroutine entry. Equivalent to `PUSH R7`.             | 1            |
| `LEAVE`  | `LEAVE`      | Standard subroutine exit. Equivalent to `POP R7`.               | 1            |

---

## Miscellaneous Instructions

| Mnemonic  | Syntax         | Description                                        | Size (bytes) |
| --------- | -------------- | -------------------------------------------------- | ------------ |
| `NOP`     | `NOP`          | No operation. Does nothing for one clock cycle.    | 1            |
| `HALT`    | `HALT`         | Halts the CPU until an interrupt occurs.           | 1            |
| `SYSCALL` | `SYSCALL imm8` | Triggers a system call with the given 8-bit index. | 2            |
| `EI`      | `EI`           | Enables maskable interrupts.                       | 1            |
| `DI`      | `DI`           | Disables maskable interrupts.                      | 1            |
| `CCF`     | `CCF`          | Complements (flips) the Carry flag.                | 1            |
| `SCF`     | `SCF`          | Sets the Carry flag to 1.                          | 1            |
| `RCF`     | `RCF`          | Resets (clears) the Carry flag to 0.               | 1            |

---

© 2025 Connor Nolan. This work is licensed under a
[Creative Commons Attribution-ShareAlike 4.0 International License](http://creativecommons.org/licenses/by-sa/4.0/).
