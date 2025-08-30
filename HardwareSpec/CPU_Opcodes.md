# **Cicada-16 CPU - Opcode Map**

**This document defines the machine code (byte code) encoding for the Cicada-16 Instruction Set Architecture. It reflects a hybrid accumulator/general-purpose architecture.**

## **Opcode Encoding Philosophy**

- **Registers (rrr, sss, ddd):** A 3-bit value (0-7) represents one of the 8 general-purpose registers (R0-R7).
- **Accumulator:** R0 is the implicit destination for most register-to-register arithmetic.
- **Prefixes:** The `0xFD` (Advanced Addressing 1), `0xFE` (Bit, Byte, and Shift), and `0xFF` (Advanced Addressing 2) prefixes access secondary instruction maps.
- **Instruction Timing:** The cycle counts listed for instructions in the ISA document assume standard WRAM access times. Instructions that access HRAM (`FE00-FFFF`) will be faster.

---

## **Main Opcode Map (1-Byte Instructions)**

| Opcode (Hex)    | Mnemonic      | Bytes | Description                                                       |
| :-------------- | :------------ | :---- | :---------------------------------------------------------------- |
| **<- 00-0F ->** | **-------**   | -     | **---Misc & Immediate Arithmetic---**                             |
| 00              | NOP           | 1     | No operation.                                                     |
| 01-08           | LDI r, n16    | 3     | Load register r with immediate 16-bit value. `0rrr n16_lo n16_hi` |
| 09              | ADDI r, n16   | 4     | `r = r + n16`. `09 rrr n16_lo n16_hi`.                            |
| 0A              | SUBI r, n16   | 4     | `r = r - n16`. `0A rrr n16_lo n16_hi`.                            |
| 0B              | ANDI r, n16   | 4     | `r = r & n16`. `0B rrr n16_lo n16_hi`.                            |
| 0C              | ORI r, n16    | 4     | `r = r or n16`. `0C rrr n16_lo n16_hi`.                           |
| 0D              | XORI r, n16   | 4     | `r = r ^ n16`. `0D rrr n16_lo n16_hi`.                            |
| 0E              | CMPI r, n16   | 4     | Compares r with n16. `0E rrr n16_lo n16_hi`.                      |
| **<- 10-1F ->** | **-------**   | -     | **---Absolute Address Load/Store---**                             |
| 10-17           | LD.w r, (n16) | 3     | Load r from absolute address. `10+r (addr_lo) (addr_hi)`          |
| 18-1F           | ST.w (n16), r | 3     | Store r to absolute address. `18+r (addr_lo) (addr_hi)`           |
| **<- 20-3F ->** | **-------**   | -     | **---Control Flow---**                                            |
| 20              | JMP n16       | 3     | `20 addr_lo addr_hi`.                                             |
| 21-28           | JMP (r)       | 1     | `21+r`. Jump to address in register r.                            |
| 29              | JR n8         | 2     | `29 offset`. Relative jump by signed 8-bit offset.                |
| 30-37           | Jcc n16       | 3     | `30+cc addr_lo addr_hi`. Conditional jump.                        |
| 38-3F           | JRcc n8       | 2     | `38+cc offset`. Conditional relative jump.                        |
| **<- 40-7F ->** | **-------**   | -     | **---Register-to-Register Load---**                               |
| 40-7F           | LD rd, rs     | 1     | `01 ddd sss`. Copies value from rs to rd. (64 opcodes)            |
| **<- 80-AF ->** | **-------**   | -     | **---16-bit Accumulator (R0) Arithmetic---**                      |
| 80-87           | ADD rs        | 1     | `10000 sss`. R0 = R0 + rs.                                        |
| 88-8F           | SUB rs        | 1     | `10001 sss`. R0 = R0 - rs.                                        |
| 90-97           | AND rs        | 1     | `10010 sss`. R0 = R0 & rs.                                        |
| 98-9F           | OR rs         | 1     | `10011 sss`. R0 = R0 or rs.                                       |
| A0-A7           | XOR rs        | 1     | `10100 sss`. R0 = R0 ^ rs.                                        |
| A8-AF           | CMP rs        | 1     | `10101 sss`. Compares R0 with rs.                                 |
| B0-B7           | NEG r         | 1     | `11000 rrr`. r = 0 - r.                                           |
| **<- B8-BF ->** | **-------**   | -     | **---Register-to-Register Arithmetic---**                         |
| B8              | ADD rd, rs    | 2     | `rd = rd + rs`. `B8 ddd sss`.                                     |
| B9              | SUB rd, rs    | 2     | `rd = rd - rs`. `B9 ddd sss`.                                     |
| BA              | AND rd, rs    | 2     | `rd = rd & rs`. `BA ddd sss`.                                     |
| BB              | OR rd, rs     | 2     | `rd = rd or rs`. `BB ddd sss`.                                    |
| BC              | XOR rd, rs    | 2     | `rd = rd ^ rs`. `BC ddd sss`.                                     |
| BD              | CMP rd, rs    | 2     | Compares rd with rs. `BD ddd sss`.                                |
| BE              | ADC rd, rs    | 2     | `BE ddd sss`. rd = rd + rs + C.                                   |
| BF              | SBC rd, rs    | 2     | `BF ddd sss`. rd = rd - rs - C.                                   |
| **<- C0-C7 ->** | **-------**   | -     | **---Accumulator-Immediate Arithmetic (2 bytes)---**              |
| C0              | ADDI n16      | 3     | `C0 imm16`. R0 = R0 + imm16.                                      |
| C1              | SUBI n16      | 3     | `C1 imm16`. R0 = R0 - imm16.                                      |
| C2              | ANDI n16      | 3     | `C2 imm16`. R0 = R0 & imm16.                                      |
| C3              | ORI n16       | 3     | `C3 imm16`. R0 = R0 or imm16.                                     |
| C4              | XORI n16      | 3     | `C4 imm16`. R0 = R0 ^ imm16.                                      |
| C5              | CMPI n16      | 3     | `C5 imm16`. Compare R0 with imm16; flags only.                    |
| C6              | ADDCI n16     | 3     | `C6 imm16`. R0 = R0 + imm16 + C.                                  |
| C7              | SUBCI n16     | 3     | `C7 imm16`. R0 = R0 - imm16 - C.                                  |
| **<- C8-D8 ->** | **-------**   | -     | **---Subroutines---**                                             |
| C8              | CALL n16      | 3     | `CE addr_lo addr_hi`.                                             |
| C9-D0           | CALL (r)      | 1     | `CF+r`. Call subroutine at address in r.                          |
| D1-D8           | CALLcc n16    | 3     | `D1+cc addr_lo addr_hi`. Conditional call.                        |
| **<- D9-F8 ->** | **-------**   | -     | **---Stack Operations & inc/dec---**                              |
| D9-E0           | DEC r         | 1     | `11010 rrr`. Decrement r.                                         |
| E1-E8           | INC r         | 1     | `11011 rrr`. Increment r.                                         |
| E9-F0           | POP r         | 1     | `11100 rrr`. Pop from stack into r.                               |
| F1-F8           | PUSH r        | 1     | `11101 rrr`. Push r onto stack.                                   |
| **<- FA-FF ->** | **-------**   | -     | **---Misc Control & Prefixes---**                                 |
| F9              | RET           | 1     | Return from subroutine.                                           |
| FA              | RETI          | 1     | Return from interrupt.                                            |
| FB              | EI            | 1     | Enable Interrupts.                                                |
| FC              | DI            | 1     | Disable Interrupts.                                               |
| FD              | PREFIX        | 1+    | Advanced Addressing 1. See below.                                 |
| FE              | PREFIX        | 1+    | Bit, Byte, and Shift Operations. See below.                       |
| FF              | PREFIX        | 1+    | Advanced Addressing 2. See below.                                 |

---

## FD Prefix Map (Advanced Addressing 1)

| Opcode (Hex) | Mnemonic          | Bytes | Description                                                   |
| :----------- | :---------------- | :---- | :------------------------------------------------------------ |
| 00-3F        | LD.w rd, (rs, n8) | 3     | `FD 00dddsss n8`. Load word from rs + offset.                 |
| 40-7F        | ST.w (rs, n8), rd | 3     | `FD 01dddsss n8`. Store word to rs + offset.                  |
| 80-BF        | LEA rd, (rs, n8)  | 3     | `FD 10dddsss n8`. Load effective address rs + offset into rd. |

---

## FE Prefix Map (Bit, Byte, and Shift Operations)

| Opcode (Hex) | Mnemonic     | Bytes | Description                                             |
| :----------- | :----------- | :---- | :------------------------------------------------------ |
| 00           | HALT         | 2     | `FE 00`. Halt CPU.                                      |
| 08-0F        | BIT (n16), b | 4     | `FE (08+b) n16`. Test bit b of byte at address n16.     |
| 10-17        | SET (n16), b | 4     | `FE (10+b) n16`. Set bit b of byte at address n16.      |
| 18-1F        | SRA r        | 2     | `FE 18+r`. Shift Right Arithmetic.                      |
| 20-27        | SHL r        | 2     | `FE 20+r`. Shift Left Logical.                          |
| 28-2F        | SHR r        | 2     | `FE 28+r`. Shift Right Logical.                         |
| 30-37        | ROL r        | 2     | `FE 30+r`. Rotate Left through Carry.                   |
| 38-3F        | ROR r        | 2     | `FE 38+r`. Rotate Right through Carry.                  |
| 40-7F        | BIT r, b     | 2     | `FE 01 bbb rrr`. Test bit b of register r.              |
| 80-87        | ADD.b rs     | 2     | `FE 10000 sss`. R0.b = R0.b + rs.b.                     |
| 88-8F        | SUB.b rs     | 2     | `FE 10001 sss`. R0.b = R0.b - rs.b.                     |
| 90-97        | AND.b rs     | 2     | `FE 10010 sss`. R0.b = R0.b & rs.b.                     |
| 98-9F        | OR.b rs      | 2     | `FE 10011 sss`. R0.b = R0.b or rs.b.                    |
| A0-A7        | XOR.b rs     | 2     | `FE 10100 sss`. R0.b = R0.b ^ rs.b.                     |
| A8-AF        | CMP.b rs     | 2     | `FE 10101 sss`. Compares R0.b with rs.b.                |
| B0           | PUSH F       | 2     | `FE B0`. Push Flags register onto stack.                |
| B1           | POP F        | 2     | `FE B1`. Pop Flags register from stack.                 |
| B8-BF        | RES (n16), b | 4     | `FE (B8+b) n16`. Reset bit b of byte at address n16.    |
| C0-DF        | SET r, b     | 2     | `FE 110 bbb rrr`. Set bit b of register r's low byte.   |
| E0-FF        | RES r, b     | 2     | `FE 111 bbb rrr`. Reset bit b of register r's low byte. |

---

## FF Prefix Map (Advanced Addressing 2)

| Opcode (Hex) | Mnemonic      | Bytes | Description                                    |
| :----------- | :------------ | :---- | :--------------------------------------------- |
| 00-3F        | LD.w rd, (rs) | 2     | `FF 00 ddd sss`. Load word from address in rs. |
| 40-7F        | ST.w (rd), rs | 2     | `FF 01 ddd sss`. Store word to address in rd.  |
| 80-BF        | LD.b rd, (rs) | 2     | `FF 10 ddd sss`. Load byte from address in rs. |
| C0-FF        | ST.b (rd), rs | 2     | `FF 11 ddd sss`. Store byte to address in rd.  |

---

© 2025 Connor Nolan. This work is licensed under a
[Creative Commons Attribution-ShareAlike 4.0 International License](http://creativecommons.org/licenses/by-sa/4.0/).
