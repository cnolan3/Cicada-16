# **Cricket-16 CPU - Opcode Map**

**This document defines the machine code (byte code) encoding for the Cricket-16 Instruction Set Architecture. It reflects a hybrid accumulator/general-purpose architecture.**

## **Opcode Encoding Philosophy**

- **Registers (rrr, sss, ddd):** A 3-bit value (0-7) represents one of the 8 general-purpose registers (R0-R7).
- **Accumulator:** R0 is the implicit destination for most register-to-register arithmetic.
- **Prefixes:** The `CB` and `ED` prefixes access secondary instruction maps for 8-bit/bitwise operations and complex instructions, respectively.

---

## **Main Opcode Map (1-Byte Instructions)**

| Opcode (Hex) | Mnemonic      | Description                                                               |
| :----------- | :------------ | :------------------------------------------------------------------------ |
| **00-0F**    | **Misc & Immediate Arithmetic**                                           |
| 00           | NOP           | No operation.                                                             |
| 01-08        | LD r, n16     | Load register r with immediate 16-bit value. `00+r n16_lo n16_hi`           |
| 09           | ADD r, n16    | `r = r + n16`. `09 rrr n16_lo n16_hi`.                                     |
| 0A           | SUB r, n16    | `r = r - n16`. `0A rrr n16_lo n16_hi`.                                     |
| 0B           | AND r, n16    | `r = r & n16`. `0B rrr n16_lo n16_hi`.                                     |
| 0C           | OR r, n16     | `r = r | n16`. `0C rrr n16_lo n16_hi`.                                     |
| 0D           | XOR r, n16    | `r = r ^ n16`. `0D rrr n16_lo n16_hi`.                                     |
| 0E           | CMP r, n16    | Compares r with n16. `0E rrr n16_lo n16_hi`.                               |
| **10-1F**    | **Absolute Address Load/Store**                                           |
| 10-17        | LD.w r, (n16) | Load r from absolute address. `10+r (addr_lo) (addr_hi)`                    |
| 18-1F        | ST.w (n16), r | Store r to absolute address. `18+r (addr_lo) (addr_hi)`                     |
| **20-3F**    | **Control Flow**                                                          |
| 20           | JMP n16       | `20 addr_lo addr_hi`.                                                     |
| 21-28        | JMP (r)       | `21+r`. Jump to address in register r.                                    |
| 29           | JR n8         | `29 offset`. Relative jump by signed 8-bit offset.                        |
| 30-37        | Jcc n16       | `30+cc addr_lo addr_hi`. Conditional jump.                                |
| 38-3F        | JRcc n8       | `38+cc offset`. Conditional relative jump.                                |
| **40-7F**    | **Register-to-Register Load**                                             |
| 40-7F        | LD rd, rs     | `01 ddd sss`. Copies value from rs to rd. (64 opcodes)                    |
| **80-AF**    | **16-bit Accumulator (R0) Arithmetic**                                    |
| 80-87        | ADD rs        | `10000 sss`. R0 = R0 + rs.                                                |
| 88-8F        | SUB rs        | `10001 sss`. R0 = R0 - rs.                                                |
| 90-97        | AND rs        | `10010 sss`. R0 = R0 & rs.                                                |
| 98-9F        | OR rs         | `10011 sss`. R0 = R0 | rs.                                                |
| A0-A7        | XOR rs        | `10100 sss`. R0 = R0 ^ rs.                                                |
| A8-AF        | CMP rs        | `10101 sss`. Compares R0 with rs.                                         |
| **B0-CF**    | **Subroutines & Stack**                                                   |
| B0           | CALL n16      | `B0 addr_lo addr_hi`.                                                     |
| B1-B8        | CALL (r)      | `B1+r`. Call subroutine at address in r.                                  |
| B8-BF        | CALLcc n16    | `B8+cc addr_lo addr_hi`. Conditional call.                                |
| C0-C7        | PUSH r        | `11000 rrr`. Push register r onto the stack.                              |
| C8-CF        | POP r         | `11001 rrr`. Pop from stack into register r.                              |
| **D0-DF**    | **Register Increment/Decrement**                                          |
| D0-D7        | INC r         | `11010 rrr`. Increment register r.                                        |
| D8-DF        | DEC r         | `11011 rrr`. Decrement register r.                                        |
| **E0-FF**    | **Misc Control**                                                          |
| F1           | RET           | Return from subroutine.                                                   |
| F2           | RETI          | Return from interrupt.                                                    |
| F3           | HALT          | Halt CPU.                                                                 |
| F4           | DI            | Disable Interrupts.                                                       |
| F5           | EI            | Enable Interrupts.                                                        |
| CB           | PREFIX        | Access 8-bit and bitwise instruction map. See below.                      |
| ED           | PREFIX        | Access extended instruction map. See below.                               |

---

## **CB Prefix Map (8-Bit & Bitwise Ops)**

| Opcode (Hex) | Mnemonic     | Description                               |
| :----------- | :----------- | :---------------------------------------- |
| 20-27        | SHL r        | `CB 20+r`. Shift Left Logical.            |
| 28-2F        | SHR r        | `CB 28+r`. Shift Right Logical.           |
| 30-37        | ROL r        | `CB 30+r`. Rotate Left through Carry.     |
| 38-3F        | ROR r        | `CB 38+r`. Rotate Right through Carry.    |
| 40-7F        | BIT r, b     | `CB 01 bbb rrr`. Test bit b of register r.|
| 80-87        | ADD.b rs     | `CB 10000 sss`. R0.b = R0.b + rs.b.       |
| 88-8F        | SUB.b rs     | `CB 10001 sss`. R0.b = R0.b - rs.b.       |
| 90-97        | AND.b rs     | `CB 10010 sss`. R0.b = R0.b & rs.b.       |
| 98-9F        | OR.b rs      | `CB 10011 sss`. R0.b = R0.b | rs.b.       |
| A0-A7        | XOR.b rs     | `CB 10100 sss`. R0.b = R0.b ^ rs.b.       |
| A8-AF        | CMP.b rs     | `CB 10101 sss`. Compares R0.b with rs.b.  |

---

## **ED Prefix Map (Extended Instructions)**

| Opcode (Hex) | Mnemonic      | Description                                      |
| :----------- | :------------ | :----------------------------------------------- |
| 00-3F        | LD.w rd, (rs) | `ED 00 ddd sss`. Load word from address in rs.   |
| 40-7F        | ST.w (rd), rs | `ED 01 ddd sss`. Store word to address in rd.    |
| 80-BF        | LD.b rd, (rs) | `ED 10 ddd sss`. Load byte from address in rs.   |
| C0-FF        | ST.b (rd), rs | `ED 11 ddd sss`. Store byte to address in rd.    |

