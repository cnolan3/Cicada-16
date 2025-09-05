# **Cicada-16 CPU - Opcode Map**

**This document defines the machine code (byte code) encoding for the Cicada-16 Instruction Set Architecture. It reflects a hybrid accumulator/general-purpose architecture.**

## **Opcode Encoding Philosophy**

- **Registers (rrr, sss, ddd):** A 3-bit value (0-7) represents one of the 8 general-purpose registers (R0-R7).
- **Accumulator:** R0 is the implicit destination for most register-to-register arithmetic.
- **Prefixes:** The `0xFD` (Advanced Addressing 1), `0xFE` (Bit, Byte, and Shift), and `0xFF` (Advanced Addressing 2) prefixes access secondary instruction maps.
- **Instruction Timing:** The cycle counts listed for instructions in the ISA document assume standard WRAM access times. Instructions that access HRAM (`FE00-FFFF`) will be faster.

---

## **Main Opcode Map (1-Byte Instructions)**

| Opcode (Hex)    | Mnemonic      | Bytes | Description                                                                                              |
| :-------------- | :------------ | :---- | :------------------------------------------------------------------------------------------------------- |
| **<- 00-0F ->** | **-------**   | -     | **---Misc & Immediate Arithmetic---**                                                                    |
| 00              | NOP           | 1     | No operation.                                                                                            |
| 01-08           | LDI r, n16    | 3     | Load register r with immediate 16-bit value, the opcode is 0x01 + r. `(3 bytes) => 0x01+r n16_lo n16_hi` |
| 09              | ADDI r, n16   | 4     | r = r + n16. `(4 bytes) => 0x09 b'00000rrr n16_lo n16_hi`.                                               |
| 0A              | SUBI r, n16   | 4     | r = r - n16. `(4 bytes) => 0x0A b'00000rrr n16_lo n16_hi`.                                               |
| 0B              | ANDI r, n16   | 4     | r = r & n16. `(4 bytes) => 0x0B b'00000rrr n16_lo n16_hi`.                                               |
| 0C              | ORI r, n16    | 4     | r = r or n16. `(4 bytes) => 0x0C b'00000rrr n16_lo n16_hi`.                                              |
| 0D              | XORI r, n16   | 4     | r = r ^ n16. `(4 bytes) => 0x0D b'00000rrr n16_lo n16_hi`.                                               |
| 0E              | CMPI r, n16   | 4     | Compares r with n16. `(4 bytes) => 0x0E b'00000rrr n16_lo n16_hi`.                                       |
| 0F              | HALT          | 1     | Halt CPU.                                                                                                |
| **<- 10-17 ->** | **-------**   | -     | **---Register-to-Register Arithmetic---**                                                                |
| 10              | ADD rd, rs    | 2     | rd = rd + rs. `(2 bytes) => 0x10 b'00dddsss`.                                                            |
| 11              | SUB rd, rs    | 2     | rd = rd - rs. `(2 bytes) => 0x11 b'00dddsss`.                                                            |
| 12              | AND rd, rs    | 2     | rd = rd & rs. `(2 bytes) => 0x12 b'00dddsss`.                                                            |
| 13              | OR rd, rs     | 2     | rd = rd or rs. `(2 bytes) => 0x13 b'00dddsss`.                                                           |
| 14              | XOR rd, rs    | 2     | d = rd ^ rs. `(2 bytes) => 0x14 b'00dddsss`.                                                             |
| 15              | CMP rd, rs    | 2     | Compares rd with rs. `(2 bytes) => 0x15 b'00dddsss`.                                                     |
| 16              | ADC rd, rs    | 2     | `(2 bytes) => 0x16 b'00dddsss`. rd = rd + rs + C.                                                        |
| 17              | SBC rd, rs    | 2     | `(2 bytes) => 0x17 b'00dddsss`. rd = rd - rs - C.                                                        |
| **<- 18-50 ->** | **-------**   | -     | **---16-bit Accumulator (R0) Arithmetic---**                                                             |
| 18-1F           | ADD rs        | 1     | `(1 byte) => 0x18+rs`. R0 = R0 + rs.                                                                     |
| 20-27           | SUB rs        | 1     | `(1 byte) => 0x20+rs`. R0 = R0 - rs.                                                                     |
| 28-2F           | AND rs        | 1     | `(1 byte) => 0x28+rs`. R0 = R0 & rs.                                                                     |
| 30-37           | OR rs         | 1     | `(1 byte) => 0x30+rs`. R0 = R0 or rs.                                                                    |
| 38-3F           | XOR rs        | 1     | `(1 byte) => 0x38+rs`. R0 = R0 ^ rs.                                                                     |
| 40-47           | CMP rs        | 1     | `(1 byte) => 0x40+rs`. Compares R0 with rs.                                                              |
| 48-4F           | NEG r         | 1     | `(1 byte) => 0x48+r`. r = 0 - r.                                                                         |
| 50              | NOT           | 1     | `(1 byte) => 0x50`. R0 = !R0.                                                                            |
| **<- 51-52 ->** | **-------**   | -     | **---Flag Manipulation---**                                                                              |
| 51              | CCF           | 1     | Complement carry flag. N flag is reset.                                                                  |
| 52              | SCF           | 2     | `(2 bytes) => 0x52 b'0000000c`. Set carry flag to c. N flag is reset.                                    |
| **<- 53-6D ->** | **-------**   | -     | **---Control Flow---**                                                                                   |
| 53              | JMP n16       | 3     | `(3 bytes) => 0x53 addr_lo addr_hi`.                                                                     |
| 54-5B           | JMP (r)       | 1     | `(1 byte) => 0x54+r`. Jump to address in register r.                                                     |
| 5C              | JR n8         | 2     | `(2 bytes) => 0x5C offset`. Relative jump by signed 8-bit offset.                                        |
| 5D-64           | Jcc n16       | 3     | `(3 bytes) => 0x5D+cc addr_lo addr_hi`. Conditional jump.                                                |
| 65-6C           | JRcc n8       | 2     | `(2 bytes) => 0x65+cc offset`. Conditional relative jump.                                                |
| 6D              | DJNZ n8       | 2     | `(2 bytes) => 0x6D offset`. Decrement the accumulator and jump to relative address if not zero.          |
| **<- 6E-7F ->** | **-------**   | -     | **---Stack Operations---**                                                                               |
| 6E-75           | POP r         | 1     | `(1 byte) => 0x6E+r`. Pop from stack into r.                                                             |
| 76-7D           | PUSH r        | 1     | `(1 byte) => 0x76+r`. Push r onto stack.                                                                 |
| 7E              | PUSH F        | 1     | `(1 byte) => 0x7E`. Push Flags register onto stack.                                                      |
| 7F              | POP F         | 1     | `(1 byte) => 0x7F`. Pop Flags register from stack.                                                       |
| **<- 80-BF ->** | **-------**   | -     | **---Register-to-Register Load---**                                                                      |
| 80-BF           | LD rd, rs     | 1     | `(1 byte) => b'10dddsss`. Copies value from rs to rd. (64 opcodes)                                       |
| **<- C0-C7 ->** | **-------**   | -     | **---Accumulator-Immediate Arithmetic (2 bytes)---**                                                     |
| C0              | ADDI n16      | 3     | `(3 bytes) => 0xC0 n16_lo n16_hi`. R0 = R0 + imm16.                                                      |
| C1              | SUBI n16      | 3     | `(3 bytes) => 0xC1 n16_lo n16_hi`. R0 = R0 - imm16.                                                      |
| C2              | ANDI n16      | 3     | `(3 bytes) => 0xC2 n16_lo n16_hi`. R0 = R0 & imm16.                                                      |
| C3              | ORI n16       | 3     | `(3 bytes) => 0xC3 n16_lo n16_hi`. R0 = R0 or imm16.                                                     |
| C4              | XORI n16      | 3     | `(3 bytes) => 0xC4 n16_lo n16_hi`. R0 = R0 ^ imm16.                                                      |
| C5              | CMPI n16      | 3     | `(3 bytes) => 0xC5 n16_lo n16_hi`. Compare R0 with imm16; flags only.                                    |
| C6              | ADDCI n16     | 3     | `(3 bytes) => 0xC6 n16_lo n16_hi`. R0 = R0 + imm16 + C.                                                  |
| C7              | SUBCI n16     | 3     | `(3 bytes) => 0xC7 n16_lo n16_hi`. R0 = R0 - imm16 - C.                                                  |
| **<- C8-D8 ->** | **-------**   | -     | **---Subroutines---**                                                                                    |
| C8              | CALL n16      | 3     | `(3 bytes) => 0xC8 addr_lo addr_hi`.                                                                     |
| C9-D0           | CALL (r)      | 1     | `(1 byte) => 0xC9+r`. Call subroutine at address in r.                                                   |
| D1-D8           | CALLcc n16    | 3     | `(3 bytes) => 0xD1+cc addr_lo addr_hi`. Conditional call.                                                |
| **<- D9-E8 ->** | **-------**   | -     | **---Stack Operations & inc/dec---**                                                                     |
| D9-E0           | DEC r         | 1     | `(1 byte) => 0xD9+r`. Decrement r.                                                                       |
| E1-E8           | INC r         | 1     | `(1 byte) => 0xE1+r`. Increment r.                                                                       |
| **<- E9-F8 ->** | **-------**   | -     | **---Absolute Address Load/Store---**                                                                    |
| E9-F0           | LD.w r, (n16) | 3     | Load r from absolute address. `(3 bytes) => 0xE9+r addr_lo addr_hi`                                      |
| F1-F8           | ST.w (n16), r | 3     | Store r to absolute address. `(3 bytes) => 0xF1+r addr_lo addr_hi`                                       |
| **<- F9-FF ->** | **-------**   | -     | **---Misc Control & Prefixes---**                                                                        |
| F9              | RET           | 1     | Return from subroutine.                                                                                  |
| FA              | RETI          | 1     | Return from interrupt.                                                                                   |
| FB              | EI            | 1     | Enable Interrupts.                                                                                       |
| FC              | DI            | 1     | Disable Interrupts.                                                                                      |
| FD              | PREFIX        | 1+    | Bit, Byte, and Shift Operations. See below.                                                              |
| FE              | PREFIX        | 1+    | Advanced Addressing 1. See below.                                                                        |
| FF              | PREFIX        | 1+    | Advanced Addressing 2. See below.                                                                        |

---

## FD Prefix Map (Bit, Byte, and Shift Operations)

| Opcode (Hex) | Mnemonic     | Bytes | Description                                                                   |
| :----------- | :----------- | :---- | :---------------------------------------------------------------------------- |
| 00-07        | SRA r        | 2     | `(2 bytes) => 0xFD 0x00+r`. Shift Right Arithmetic.                           |
| 08-0F        | SHL r        | 2     | `(2 bytes) => 0xFD 0x08+r`. Shift Left Logical.                               |
| 10-17        | SHR r        | 2     | `(2 bytes) => 0xFD 0x10+r`. Shift Right Logical.                              |
| 18-1F        | ROL r        | 2     | `(2 bytes) => 0xFD 0x18+r`. Rotate Left through Carry.                        |
| 20-27        | ROR r        | 2     | `(2 bytes) => 0xFD 0x20+r`. Rotate Right through Carry.                       |
| 28-2F        | ADD.b rs     | 2     | `(2 bytes) => 0xFD 0x28+rs`. R0.b = R0.b + rs.b.                              |
| 30-37        | SUB.b rs     | 2     | `(2 bytes) => 0xFD 0x30+rs`. R0.b = R0.b - rs.b.                              |
| 38-3F        | AND.b rs     | 2     | `(2 bytes) => 0xFD 0x38+rs`. R0.b = R0.b & rs.b.                              |
| 40-47        | OR.b rs      | 2     | `(2 bytes) => 0xFD 0x40+rs`. R0.b = R0.b or rs.b.                             |
| 48-4F        | XOR.b rs     | 2     | `(2 bytes) => 0xFD 0x48+rs`. R0.b = R0.b ^ rs.b.                              |
| 50-57        | CMP.b rs     | 2     | `(2 bytes) => 0xFD 0x50+rs`. Compares R0.b with rs.b.                         |
| 58-5F        | BIT r, b     | 3     | `(3 bytes) => 0xFD 0x58+b b'00000rrr`. Test bit b of register r.              |
| 60-67        | SET r, b     | 3     | `(3 bytes) => 0xFD 0x60+b b'00000rrr`. Set bit b of register r's low byte.    |
| 68-6F        | RES r, b     | 3     | `(3 bytes) => 0xFD 0x68+b b'00000rrr`. Reset bit b of register r's low byte.  |
| 70-77        | BIT (n16), b | 4     | `(4 bytes) => 0xFD 0x70+b n16_lo n16_hi`. Test bit b of byte at address n16.  |
| 78-7F        | SET (n16), b | 4     | `(4 bytes) => 0xFD 0x78+b n16_lo n16_hi`. Set bit b of byte at address n16.   |
| 80-87        | RES (n16), b | 4     | `(4 bytes) => 0xFD 0x80+b n16_lo n16_hi`. Reset bit b of byte at address n16. |
| 88-8F        | SWAP r       | 2     | `(2 bytes) => 0xFD 0x88+r`. Swap the upper and lower bytes of r.              |

---

## FE Prefix Map (Advanced Addressing 1)

| Opcode (Hex) | Mnemonic      | Bytes | Description                                                   |
| :----------- | :------------ | :---- | :------------------------------------------------------------ |
| 00-3F        | LD.w rd, (rs) | 2     | `(2 bytes) => 0xFE b'00dddsss`. Load word from address in rs. |
| 40-7F        | ST.w (rd), rs | 2     | `(2 bytes) => 0xFE b'01dddsss`. Store word to address in rd.  |
| 80-BF        | LD.b rd, (rs) | 2     | `(2 bytes) => 0xFE b'10dddsss`. Load byte from address in rs. |
| C0-FF        | ST.b (rd), rs | 2     | `(2 bytes) => 0xFE b'11dddsss`. Store byte to address in rd.  |

---

## FF Prefix Map (Advanced Addressing 2)

| Opcode (Hex) | Mnemonic          | Bytes | Description                                                                    |
| :----------- | :---------------- | :---- | :----------------------------------------------------------------------------- |
| 00-3F        | LD.w rd, (rs, n8) | 3     | `(3 bytes) => 0xFF b'00dddsss n8`. Load word from rs + offset.                 |
| 40-7F        | ST.w (rs, n8), rd | 3     | `(3 bytes) => 0xFF b'01dddsss n8`. Store word to rs + offset.                  |
| 80-BF        | LEA rd, (rs, n8)  | 3     | `(3 bytes) => 0xFF b'10dddsss n8`. Load effective address rs + offset into rd. |

---

© 2025 Connor Nolan. This work is licensed under a
[Creative Commons Attribution-ShareAlike 4.0 International License](http://creativecommons.org/licenses/by-sa/4.0/).
