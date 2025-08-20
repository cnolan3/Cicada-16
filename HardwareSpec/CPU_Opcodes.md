# **Cricket-16 CPU - Opcode Map**

**This document defines the machine code (byte code) encoding for the Cricket-16 Instruction Set Architecture.**

## **Opcode Encoding Philosophy**

**The opcodes are designed to be as regular as possible. A single instruction byte often encodes the operation, source register, and destination register.**

- **Registers (rrr or ddd):** A 3-bit value represents one of the 8 general-purpose registers.
  - 000: R0
  - 001: R1
  - 010: R2
  - 011: R3
  - 100: R4
  - 101: R5
  - 110: R6 / FP
  - 111: R7 / SP
- **Condition Codes (cc):** A 3-bit value represents a condition for jumps/calls.
  - 000: Z (Zero)
  - 001: NZ (Not Zero)
  - 010: C (Carry)
  - 011: NC (No Carry)
  - 100: N (Negative)
  - 101: NN (Not Negative)
  - 110: V (Overflow)
  - 111: NV (No Overflow)

## **Load/Store Instructions (LD, ST, PUSH, POP)**

| Opcode (Hex) | Mnemonic      | Description                                                               |
| :----------- | :------------ | :------------------------------------------------------------------------ |
| 01-08        | LD r, n16     | Load register r with immediate 16-bit value. 00+r n16_lo n16_hi           |
| 10-17        | LD.w r, (n16) | Load register r from absolute 16-bit address. 10+r (addr_lo) (addr_hi)    |
| 18-1F        | ST.w (n16), r | Store register r to absolute 16-bit address. 18+r (addr_lo) (addr_hi)     |
| 40-7F        | LD rd, rs     | 01 ddd sss. Copies value from source sss to destination ddd. (64 opcodes) |
| C0-C7        | PUSH r        | Push register r onto the stack. C0+r                                      |
| C8-CF        | POP r         | Pop from stack into register r. C8+r                                      |
| D0-D7        | LD.w rd, (rs) | 11010 ddd sss. Load word from address in rs.                              |
| D8-DF        | LD.b rd, (rs) | 11011 ddd sss. Load byte from address in rs.                              |
| E0-E7        | ST.w (rd), rs | 11100 ddd sss. Store word to address in rd.                               |
| E8-EF        | ST.b (rd), rs | 11101 ddd sss. Store byte to address in rd.                               |

## **16-Bit Arithmetic/Logic Instructions**

| Opcode (Hex) | Mnemonic   | Description                                                      |
| :----------- | :--------- | :--------------------------------------------------------------- |
| 80           | ADD rd, rs | 2-byte instruction. 80 followed by ddd sss. rd = rd + rs.        |
| 09           | ADD r, n16 | 09 rrr n16_lo n16_hi. r = r + n16.                               |
| 82           | SUB rd, rs | 2-byte instruction. 82 followed by ddd sss. rd = rd - rs.        |
| 0A           | SUB r, n16 | 0A rrr n16_lo n16_hi. r = r - n16.                               |
| 84           | AND rd, rs | 2-byte instruction. 84 followed by ddd sss. rd = rd & rs.        |
| 0B           | AND r, n16 | 0B rrr n16_lo n16_hi. r = r & n16.                               |
| 86           | OR rd, rs  | 2-byte instruction. 86 followed by ddd sss. rd = rd              |
| 0C           | OR r, n16  | 0C rrr n16_lo n16_hi. r = r                                      |
| 88           | XOR rd, rs | 2-byte instruction. 88 followed by ddd sss. rd = rd ^ rs.        |
| 0D           | XOR r, n16 | 0D rrr n16_lo n16_hi. r = r ^ n16.                               |
| 8A           | CMP rd, rs | 2-byte instruction. 8A followed by ddd sss. Compares rd with rs. |
| 0E           | CMP r, n16 | 0E rrr n16_lo n16_hi. Compares r with n16.                       |
| 90-97        | INC r      | 90+r. Increment register r.                                      |
| 98-9F        | DEC r      | 98+r. Decrement register r.                                      |

## **8-Bit Arithmetic/Logic Instructions**

**These use a prefix byte CB to indicate a byte-level operation, similar to the Z80.**

| Opcode (Hex) | Mnemonic     | Description    |
| :----------- | :----------- | :------------- |
| CB 80        | ADD.b rd, rs | CB 80 ddd sss. |
| CB 09        | ADD.b r, n8  | CB 09 rrr n8.  |
| CB 82        | SUB.b rd, rs | CB 82 ddd sss. |
| CB 0A        | SUB.b r, n8  | CB 0A rrr n8.  |
| CB 8A        | CMP.b rd, rs | CB 8A ddd sss. |
| CB 0E        | CMP.b r, n8  | CB 0E rrr n8.  |

## **Rotate, Shift, and Bit Instructions**

**These also use the CB prefix.**

| Opcode (Hex) | Mnemonic | Description                              |
| :----------- | :------- | :--------------------------------------- |
| CB 20-27     | SHL r    | CB 20+r.                                 |
| CB 28-2F     | SHR r    | CB 28+r.                                 |
| CB 30-37     | ROL r    | CB 30+r.                                 |
| CB 38-3F     | ROR r    | CB 38+r.                                 |
| CB 40-7F     | BIT r, b | CB 01 bbb rrr. Test bit b of register r. |

## **Control Flow Instructions**

| Opcode (Hex) | Mnemonic   | Description                                      |
| :----------- | :--------- | :----------------------------------------------- |
| 20           | JMP n16    | 20 addr_lo addr_hi.                              |
| 21-28        | JMP (r)    | 21+r. Jump to address in register r.             |
| 29           | JR n8      | 29 offset. Relative jump by signed 8-bit offset. |
| 30-37        | Jcc n16    | 30+cc addr_lo addr_hi. Conditional jump.         |
| 38-3F        | JRcc n8    | 38+cc offset. Conditional relative jump.         |
| B0           | CALL n16   | B0 addr_lo addr_hi.                              |
| B1-B8        | CALL (r)   | B1+r. Call subroutine at address in r.           |
| B8-BF        | CALLcc n16 | B8+cc addr_lo addr_hi. Conditional call.         |
| F1           | RET        | Return from subroutine.                          |
| F2           | RETI       | Return from interrupt.                           |

## **CPU Control Instructions**

| Opcode (Hex) | Mnemonic | Description         |
| :----------- | :------- | :------------------ |
| 00           | NOP      | No operation.       |
| F3           | HALT     | Halt CPU.           |
| F4           | DI       | Disable Interrupts. |
| F5           | EI       | Enable Interrupts.  |
