# **Cicada-16 CPU - Opcode Map**

**This document defines the machine code (byte code) encoding for the Cicada-16 Instruction Set Architecture. It reflects a hybrid accumulator/general-purpose architecture.**

## **Opcode Encoding Philosophy**

- **Registers (rrr, sss, ddd):** A 3-bit value (0-7) represents one of the 8 general-purpose registers (R0-R7).
- **Accumulator:** R0 is the implicit destination for most register-to-register arithmetic.
- **Prefixes:** The `0xFD` (Advanced Addressing 1), `0xFE` (Bit, Byte, and Shift), and `0xFF` (Advanced Addressing 2) prefixes access secondary instruction maps.
- **Instruction Timing:** The cycle counts listed for instructions in the ISA document assume standard WRAM access times. Instructions that access HRAM (`FE00-FFFF`) will be faster.

---

### Note on Conditional Instruction Encoding

All conditional instructions (Jcc, JRcc, CALLcc) use the same encoding for the condition code (cc):

| cc value | condition code    |
| :------- | :---------------- |
| 0        | V (overflow)      |
| 1        | NV (Not overflow) |
| 2        | N (negative)      |
| 3        | NN (not negative) |
| 4        | C (carry)         |
| 5        | NC (not carry)    |
| 6        | Z (zero)          |
| 7        | NZ (not zero)     |

ex) opcode 0x63 => Jump Zero (0x63 = 0x5D + **6**, cc = 6)

---

### **Note on Instruction Cycle Timing**

The "Cycles" column in the tables below denotes the number of T-cycles required for each instruction, presented in the format **`T (F : E)`**:

- **T**: Total cycles for the instruction.
- **F**: Fetch cycles, the time taken to read the instruction from memory.
- **E**: Execution cycles, the additional time for memory access or other internal operations.

These values are calculated based on the following rules:

1.  **Fetch Cost (F):** The cost to read an instruction from memory is 4 T-cycles per byte.
    - `Fetch Cost = (Instruction Size in Bytes) * 4`

2.  **Execution Cost (E):** This is the cost of any work done after fetching, primarily memory access. Simple internal register operations have an execution cost of 0.
    - **16-bit Memory Access (Read/Write):** 8 T-cycles for WRAM, or 4 T-cycles for HRAM. Stack operations (`PUSH`, `POP`) are considered 16-bit accesses.
    - **8-bit Memory Access (Read/Write):** 4 T-cycles for WRAM, or 2 T-cycles for HRAM.

The total cycle count is the sum of fetch and execution costs: **`Total Cycles = Fetch Cost + Execution Cost`**.

For instructions with memory access, cycle counts may be presented as `HRAM/WRAM` (e.g., `16/20`). Unless specified, timings assume WRAM access. Control flow instructions like jumps do not include pipeline stall penalties in their base cost.

---

## **Main Opcode Map (1-Byte Instructions)**

| Opcode (Hex)    | Mnemonic    | Bytes | Cycles           | Description                                                                                              |
| :-------------- | :---------- | :---- | :--------------- | :------------------------------------------------------------------------------------------------------- |
| **<- 00-0F ->** | **-------** | -     | **-------**      | **---Misc & Immediate Arithmetic---**                                                                    |
| 00              | NOP         | 1     | 4 (4 : 0)        | No operation.                                                                                            |
| 01-08           | LDI r, n16  | 3     | 12 (12 : 0)      | `(3 bytes) => 0x01+r n16_lo n16_hi` Load register r with immediate 16-bit value, the opcode is 0x01 + r. |
| 09              | ADDI r, n16 | 4     | 16 (16 : 0)      | `(4 bytes) => 0x09 b'00000rrr n16_lo n16_hi`. r = r + n16.                                               |
| 0A              | SUBI r, n16 | 4     | 16 (16 : 0)      | `(4 bytes) => 0x0A b'00000rrr n16_lo n16_hi`. r = r - n16.                                               |
| 0B              | ANDI r, n16 | 4     | 16 (16 : 0)      | `(4 bytes) => 0x0B b'00000rrr n16_lo n16_hi`. r = r & n16.                                               |
| 0C              | ORI r, n16  | 4     | 16 (16 : 0)      | `(4 bytes) => 0x0C b'00000rrr n16_lo n16_hi`. r = r or n16.                                              |
| 0D              | XORI r, n16 | 4     | 16 (16 : 0)      | `(4 bytes) => 0x0D b'00000rrr n16_lo n16_hi`. r = r ^ n16.                                               |
| 0E              | CMPI r, n16 | 4     | 16 (16 : 0)      | `(4 bytes) => 0x0E b'00000rrr n16_lo n16_hi`. Compares r with n16.                                       |
| 0F              | HALT        | 1     | 4 (4 : 0)        | Halt CPU.                                                                                                |
| **<- 10-17 ->** | **-------** | -     | **-------**      | **---Register-to-Register Arithmetic---**                                                                |
| 10              | ADD rd, rs  | 2     | 8 (8 : 0)        | `(2 bytes) => 0x10 b'00dddsss`. rd = rd + rs.                                                            |
| 11              | SUB rd, rs  | 2     | 8 (8 : 0)        | `(2 bytes) => 0x11 b'00dddsss`. rd = rd - rs.                                                            |
| 12              | AND rd, rs  | 2     | 8 (8 : 0)        | `(2 bytes) => 0x12 b'00dddsss`. rd = rd & rs.                                                            |
| 13              | OR rd, rs   | 2     | 8 (8 : 0)        | `(2 bytes) => 0x13 b'00dddsss`. rd = rd or rs.                                                           |
| 14              | XOR rd, rs  | 2     | 8 (8 : 0)        | `(2 bytes) => 0x14 b'00dddsss`. d = rd ^ rs.                                                             |
| 15              | CMP rd, rs  | 2     | 8 (8 : 0)        | `(2 bytes) => 0x15 b'00dddsss`. Compares rd with rs.                                                     |
| 16              | ADC rd, rs  | 2     | 8 (8 : 0)        | `(2 bytes) => 0x16 b'00dddsss`. rd = rd + rs + C.                                                        |
| 17              | SBC rd, rs  | 2     | 8 (8 : 0)        | `(2 bytes) => 0x17 b'00dddsss`. rd = rd - rs - C.                                                        |
| **<- 18-50 ->** | **-------** | -     | **-------**      | **---16-bit Accumulator (R0) Arithmetic---**                                                             |
| 18-1F           | ADD rs      | 1     | 4 (4 : 0)        | `(1 byte) => 0x18+rs`. R0 = R0 + rs.                                                                     |
| 20-27           | SUB rs      | 1     | 4 (4 : 0)        | `(1 byte) => 0x20+rs`. R0 = R0 - rs.                                                                     |
| 28-2F           | AND rs      | 1     | 4 (4 : 0)        | `(1 byte) => 0x28+rs`. R0 = R0 & rs.                                                                     |
| 30-37           | OR rs       | 1     | 4 (4 : 0)        | `(1 byte) => 0x30+rs`. R0 = R0 or rs.                                                                    |
| 38-3F           | XOR rs      | 1     | 4 (4 : 0)        | `(1 byte) => 0x38+rs`. R0 = R0 ^ rs.                                                                     |
| 40-47           | CMP rs      | 1     | 4 (4 : 0)        | `(1 byte) => 0x40+rs`. Compares R0 with rs.                                                              |
| 48              | NEG         | 1     | 4 (4 : 0)        | R0 = 0 - R0.                                                                                             |
| 49              | NOT         | 1     | 4 (4 : 0)        | R0 = !R0.                                                                                                |
| 4A              | SWAP        | 1     | 4 (4 : 0)        | Swap the upper and lower bytes of R0.                                                                    |
| **<- 4B-4D ->** | **-------** | -     | **-------**      | **---Flag Manipulation---**                                                                              |
| 4B              | CCF         | 1     | 4 (4 : 0)        | Complement carry flag. N flag is reset.                                                                  |
| 4C              | SCF         | 1     | 4 (4 : 0)        | Set carry flag to 1. N flag is reset.                                                                    |
| 4D              | RCF         | 1     | 4 (4 : 0)        | Reset carry flag to 0. N flag is reset.                                                                  |
| **<- 4E ->**    | **-------** | -     | **-------**      | **---System Library Call---**                                                                            |
| 4E              | SYSCALL n8  | 2     | 24 (8 : 16)      | `(2 bytes) => 0x4E n8`. Call system routine at index n8.                                                 |
| **<- 50-51 ->** | **-------** | -     | **-------**      | **---Enter/Leave---**                                                                                    |
| 4F              | ENTER       | 1     | 12 (4 : 8)       | Enter a function, See [CPU_ISA.md](./CPU_ISA.md) for details.                                            |
| 50              | LEAVE       | 1     | 12 (4 : 8)       | Leave a function, See [CPU_ISA.md](./CPU_ISA.md) for details.                                            |
| **<- 52-6C ->** | **-------** | -     | **-------**      | **---Control Flow---**                                                                                   |
| 51              | JMP n16     | 3     | 12 (12 : 0)      | `(3 bytes) => 0x51 addr_lo addr_hi`.                                                                     |
| 52-59           | JMP (r)     | 1     | 4 (4 : 0)        | `(1 byte) => 0x52+r`. Jump to address in register r.                                                     |
| 5A              | JR n8s      | 2     | 8 (8 : 0)        | `(2 bytes) => 0x5A offset`. Relative jump by signed 8-bit signed offset.                                 |
| 5B-62           | Jcc n16     | 3     | 12 (12 : 0)      | `(3 bytes) => 0x5B+cc addr_lo addr_hi`. Conditional jump.                                                |
| 63-6A           | JRcc n8s    | 2     | 8 (8 : 0)        | `(2 bytes) => 0x63+cc offset`. Conditional relative jump.                                                |
| 6B              | DJNZ n8s    | 2     | 8 (8 : 0)        | `(2 bytes) => 0x6B offset`. Decrement R5 and jump to relative address if not zero.                       |
| **<- 6D-7F ->** | **-------** | -     | **-------**      | **---Stack Operations---**                                                                               |
| 6C              | ADD SP, n8s | 2     | 8 (8 : 0)        | `(2 bytes) => 0x6C n8s`. SP = SP + n8s. Stack pointer immediate arithmetic.                              |
| 6D-74           | PUSH r      | 1     | 12 (4 : 8)       | `(1 byte) => 0x6D+r`. Push r onto stack.                                                                 |
| 75-7C           | POP r       | 1     | 12 (4 : 8)       | `(1 byte) => 0x75+r`. Pop from stack into r.                                                             |
| 7D              | PUSH n16    | 3     | 20 (12 : 8)      | `(1 byte) => 0x7D n16_lo n16_hi`. Push immediate value onto stack.                                       |
| 7E              | PUSH F      | 1     | 12 (4 : 8)       | Push Flags register onto stack.                                                                          |
| 7F              | POP F       | 1     | 12 (4 : 8)       | Pop Flags register from stack.                                                                           |
| **<- 80-BF ->** | **-------** | -     | **-------**      | **---Register-to-Register Load---**                                                                      |
| 80-BF           | LD rd, rs   | 1     | 4 (4 : 0)        | `(1 byte) => b'10dddsss`. Copies value from rs to rd. (64 opcodes)                                       |
| **<- C0-C7 ->** | **-------** | -     | **-------**      | **---Accumulator-Immediate Arithmetic---**                                                               |
| C0              | ADDI n16    | 3     | 12 (12 : 0)      | `(3 bytes) => 0xC0 n16_lo n16_hi`. R0 = R0 + imm16.                                                      |
| C1              | SUBI n16    | 3     | 12 (12 : 0)      | `(3 bytes) => 0xC1 n16_lo n16_hi`. R0 = R0 - imm16.                                                      |
| C2              | ANDI n16    | 3     | 12 (12 : 0)      | `(3 bytes) => 0xC2 n16_lo n16_hi`. R0 = R0 & imm16.                                                      |
| C3              | ORI n16     | 3     | 12 (12 : 0)      | `(3 bytes) => 0xC3 n16_lo n16_hi`. R0 = R0 or imm16.                                                     |
| C4              | XORI n16    | 3     | 12 (12 : 0)      | `(3 bytes) => 0xC4 n16_lo n16_hi`. R0 = R0 ^ imm16.                                                      |
| C5              | CMPI n16    | 3     | 12 (12 : 0)      | `(3 bytes) => 0xC5 n16_lo n16_hi`. Compare R0 with imm16; flags only.                                    |
| C6              | ADCI n16    | 3     | 12 (12 : 0)      | `(3 bytes) => 0xC6 n16_lo n16_hi`. R0 = R0 + imm16 + C.                                                  |
| C7              | SBCI n16    | 3     | 12 (12 : 0)      | `(3 bytes) => 0xC7 n16_lo n16_hi`. R0 = R0 - imm16 - C.                                                  |
| **<- C8-D8 ->** | **-------** | -     | **-------**      | **---Subroutines---**                                                                                    |
| C8              | CALL n16    | 3     | 20 (12 : 8)      | `(3 bytes) => 0xC8 addr_lo addr_hi`.                                                                     |
| C9-D0           | CALL (r)    | 1     | 12 (4 : 8)       | `(1 byte) => 0xC9+r`. Call subroutine at address in r.                                                   |
| D1-D8           | CALLcc n16  | 3     | 12/20 (12 : 0/8) | `(3 bytes) => 0xD1+cc addr_lo addr_hi`. Conditional call.                                                |
| **<- D9-E8 ->** | **-------** | -     | **-------**      | **---Inc/Dec---**                                                                                        |
| D9-E0           | DEC r       | 1     | 4 (4 : 0)        | `(1 byte) => 0xD9+r`. Decrement r.                                                                       |
| E1-E8           | INC r       | 1     | 4 (4 : 0)        | `(1 byte) => 0xE1+r`. Increment r.                                                                       |
| **<- E9-F8 ->** | **-------** | -     | **-------**      | **---Absolute Address Load/Store---**                                                                    |
| E9-F0           | LD r, (n16) | 3     | 16/20 (12 : 4/8) | `(3 bytes) => 0xE9+r addr_lo addr_hi`. Load r from absolute address.                                     |
| F1-F8           | ST (n16), r | 3     | 16/20 (12 : 4/8) | `(3 bytes) => 0xF1+r addr_lo addr_hi`. Store r to absolute address.                                      |
| **<- F9-FF ->** | **-------** | -     | **-------**      | **---Misc Control & Prefixes---**                                                                        |
| F9              | RET         | 1     | 12 (4 : 8)       | Return from subroutine.                                                                                  |
| FA              | RETI        | 1     | 12 (4 : 8)       | Return from interrupt.                                                                                   |
| FB              | EI          | 1     | 4 (4 : 0)        | Enable Interrupts.                                                                                       |
| FC              | DI          | 1     | 4 (4 : 0)        | Disable Interrupts.                                                                                      |
| FD              | PREFIX      | 1+    | 4 (4 : 0)        | Bit, Byte, and Shift Operations. See below.                                                              |
| FE              | PREFIX      | 1+    | 4 (4 : 0)        | Advanced Addressing 1. See below.                                                                        |
| FF              | PREFIX      | 1+    | 4 (4 : 0)        | Advanced Addressing 2. See below.                                                                        |

---

## FD Prefix Map (Bit, Byte, and Shift Operations)

| Opcode (Hex) | Mnemonic     | Bytes | Cycles            | Description                                                                     |
| :----------- | :----------- | :---- | :---------------- | :------------------------------------------------------------------------------ |
| 00-07        | SRA r        | 2     | 8 (8 : 0)         | `(2 bytes) => 0xFD 0x00+r`. Shift Right Arithmetic.                             |
| 08-0F        | SHL r        | 2     | 8 (8 : 0)         | `(2 bytes) => 0xFD 0x08+r`. Shift Left Logical.                                 |
| 10-17        | SHR r        | 2     | 8 (8 : 0)         | `(2 bytes) => 0xFD 0x10+r`. Shift Right Logical.                                |
| 18-1F        | ROL r        | 2     | 8 (8 : 0)         | `(2 bytes) => 0xFD 0x18+r`. Rotate Left through Carry.                          |
| 20-27        | ROR r        | 2     | 8 (8 : 0)         | `(2 bytes) => 0xFD 0x20+r`. Rotate Right through Carry.                         |
| 28-2F        | ADD.b rs     | 2     | 8 (8 : 0)         | `(2 bytes) => 0xFD 0x28+rs`. R0.b = R0.b + rs.b.                                |
| 30-37        | SUB.b rs     | 2     | 8 (8 : 0)         | `(2 bytes) => 0xFD 0x30+rs`. R0.b = R0.b - rs.b.                                |
| 38-3F        | AND.b rs     | 2     | 8 (8 : 0)         | `(2 bytes) => 0xFD 0x38+rs`. R0.b = R0.b & rs.b.                                |
| 40-47        | OR.b rs      | 2     | 8 (8 : 0)         | `(2 bytes) => 0xFD 0x40+rs`. R0.b = R0.b or rs.b.                               |
| 48-4F        | XOR.b rs     | 2     | 8 (8 : 0)         | `(2 bytes) => 0xFD 0x48+rs`. R0.b = R0.b ^ rs.b.                                |
| 50-57        | CMP.b rs     | 2     | 8 (8 : 0)         | `(2 bytes) => 0xFD 0x50+rs`. Compares R0.b with rs.b.                           |
| 58-5F        | BIT r, b     | 3     | 12 (12 : 0)       | `(3 bytes) => 0xFD 0x58+b b'00000rrr`. Test bit b of register r.                |
| 60-67        | SET r, b     | 3     | 12 (12 : 0)       | `(3 bytes) => 0xFD 0x60+b b'00000rrr`. Set bit b of register r's low byte.      |
| 68-6F        | RES r, b     | 3     | 12 (12 : 0)       | `(3 bytes) => 0xFD 0x68+b b'00000rrr`. Reset bit b of register r's low byte.    |
| 70-77        | BIT (n16), b | 4     | 18/20 (16 : 2/4)  | `(4 bytes) => 0xFD 0x70+b n16_lo n16_hi`. Test bit b of byte at address n16.    |
| 78-7F        | SET (n16), b | 4     | 20/24 (16 : 4/8)  | `(4 bytes) => 0xFD 0x78+b n16_lo n16_hi`. Set bit b of byte at address n16.     |
| 80-87        | RES (n16), b | 4     | 20/24 (16 : 4/8)  | `(4 bytes) => 0xFD 0x80+b n16_lo n16_hi`. Reset bit b of byte at address n16.   |
| 88-8F        | BIT (rs), b  | 3     | 16/20 (12 : 4/8)  | `(3 bytes) => 0xFD 0x88+b rs`. Test bit b of byte at address in rs.             |
| 90-97        | SET (rs), b  | 3     | 20/28 (12 : 8/16) | `(3 bytes) => 0xFD 0x90+b rs`. Set bit b of byte at address in rs.              |
| 98-9F        | RES (rs), b     | 3     | 20/28 (12 : 8/16) | `(3 bytes) => 0xFD 0x98+b rs`. Reset bit b of byte at address in rs.                    |
| A0-A7        | LDI.b rd, n8    | 3     | 12 (12 : 0)       | `(3 bytes) => 0xFD 0xA0+rd n8`. Load low byte of rd with immediate 8-bit value.         |
| A8-AF        | LD.b rd, (n16)  | 4     | 14/16 (16 : 2/4)  | `(4 bytes) => 0xFD 0xA8+rd n16_lo n16_hi`. Load byte from absolute address into rd.     |
| B0-B7        | ST.b (n16), rs  | 4     | 14/16 (16 : 2/4)  | `(4 bytes) => 0xFD 0xB0+rs n16_lo n16_hi`. Store byte from rs to absolute address.      |

---

## FE Prefix Map (Advanced Addressing 1)

| Opcode (Hex) | Mnemonic      | Bytes | Cycles          | Description                                                   |
| :----------- | :------------ | :---- | :-------------- | :------------------------------------------------------------ |
| 00-3F        | LD rd, (rs)   | 2     | 12/16 (8 : 4/8) | `(2 bytes) => 0xFE b'00dddsss`. Load word from address in rs. |
| 40-7F        | ST (rd), rs   | 2     | 12/16 (8 : 4/8) | `(2 bytes) => 0xFE b'01dddsss`. Store word to address in rd.  |
| 80-BF        | LD.b rd, (rs) | 2     | 10/12 (8 : 2/4) | `(2 bytes) => 0xFE b'10dddsss`. Load byte from address in rs. |
| C0-FF        | ST.b (rd), rs | 2     | 10/12 (8 : 2/4) | `(2 bytes) => 0xFE b'11dddsss`. Store byte to address in rd.  |

---

## FF Prefix Map (Advanced Addressing 2)

| Opcode (Hex) | Mnemonic          | Bytes | Cycles           | Description                                                                                      |
| :----------- | :---------------- | :---- | :--------------- | :----------------------------------------------------------------------------------------------- |
| 00-3F        | LD rd, (rs, n8s)  | 3     | 16/20 (12 : 4/8) | `(3 bytes) => 0xFF b'00dddsss n8`. Load word from rs + offset.                                   |
| 40-7F        | ST (rd, n8s), rs  | 3     | 16/20 (12 : 4/8) | `(3 bytes) => 0xFF b'01dddsss n8`. Store word to rd + offset.                                    |
| 80-BF        | LEA rd, (rs, n8s) | 3     | 12 (12 : 0)      | `(3 bytes) => 0xFF b'10dddsss n8`. Load effective address rs + offset into rd.                   |
| C0-C7        | LD rd, (rs)+      | 3     | 16/20 (12 : 4/8) | `(3 bytes) => 0xFF 0xC0+rs b'00000ddd'`. Load word from (rs) into rd, then increments rs by 2.   |
| C8-CF        | ST (rd)+, rs      | 3     | 16/20 (12 : 4/8) | `(3 bytes) => 0xFF 0xC8+rs b'00000ddd'`. Store word from rs into (rd), then increments rd by 2.  |
| D0-D7        | LD rd, -(rs)      | 3     | 16/20 (12 : 4/8) | `(3 bytes) => 0xFF 0xD0+rs b'00000ddd'`. Decrements rs by 2, then loads word from (rs) into rd.  |
| D8-DF        | ST -(rd), rs      | 3     | 16/20 (12 : 4/8) | `(3 bytes) => 0xFF 0xD8+rs b'00000ddd'`. Decrements rd by 2, then stores word from rs into (rd). |
| E0-E7        | LD.b rd, (rs)+    | 3     | 14/16 (12 : 2/4) | `(3 bytes) => 0xFF 0xE0+rs b'00000ddd'`. Load byte from (rs) into rd, then increments rs by 1.   |
| E8-EF        | ST.b (rd)+, rs    | 3     | 14/16 (12 : 2/4) | `(3 bytes) => 0xFF 0xE8+rs b'00000ddd'`. Store byte from rs into (rd), then increments rd by 1.  |
| F0-F7        | LD.b rd, -(rs)    | 3     | 14/16 (12 : 2/4) | `(3 bytes) => 0xFF 0xF0+rs b'00000ddd'`. Decrements rs by 1, then loads byte from (rs) into rd.  |
| F8-FF        | ST.b -(rd), rs    | 3     | 14/16 (12 : 2/4) | `(3 bytes) => 0xFF 0xF8+rs b'00000ddd'`. Decrements rd by 1, then stores byte from rs into (rd). |

---

© 2025 Connor Nolan. This work is licensed under a
[Creative Commons Attribution-ShareAlike 4.0 International License](http://creativecommons.org/licenses/by-sa/4.0/).
org/licenses/by-sa/4.0/).
