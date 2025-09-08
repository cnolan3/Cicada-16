# Project Notes & Future Ideas

**Disclaimer:** This document contains thoughts and potential ideas for the Cicada-16 project. The notes here are **not** part of the official hardware specification unless explicitly integrated into the main `HardwareSpec` documents.

---

## Idea: Instruction Set Modifications

1. Change DJNZ Target Register:
   Instead of decrementing R0 (the accumulator), the DJNZ instruction should decrement R5.

2. Add Indirect Bit Manipulation:
   Add three new Indirect Bit Manipulation instructions, `BIT (rs), b`, `SET (rs), b` and `RES (rs), b` where `rs` is the register containing the memory address to manipulate and `b` is the bit number within that word
   to manipulate (0-15). Add these instructions to the `FD` prefix block. Each of these instructions are three bytes long and occupy eight sub-opcodes (24 sub-opcodes in total for all three instructions), each
   sub-opcode corresponds to one source register.

3. Add 8-bit Immediate Load:
   Add a `LDI.b rd, n8` instruction that loads an immediate-8 value into the lower byte of a register. Add this instruction to the `FD` prefix block. A three byte instruction occupying eight sub-opcodes, each sub-opcode
   corresponds to one destination register.

4. Add a SYSCALL instruction:
   Add a dedicated instruction for calling system library functions: `SYSCALL n8`. This instruction takes an index into the system library vector table as an argument and automatically gets the system library function
   address, saves current flags and jumps to that address (saves current PC, sets PC to function address). This instruction will take one of the currently reserved opcodes in the main opcode map (0x4E or 0x4F) and will
   be two bytes long.

5. Add Stack Pointer Immediate Arithmetic:
   Add `ADD SP, n8s` which takes a signed 8-bit value (`n8s`) and increments/decrements the stack pointer `R7` by that value. This instruction will take one of the currently reserved opcodes in the main opcode map (0x4E
   or 0x4F) and will be two bytes long.

6. Add Post-Increment/Pre-Decrement Addressing Instructions:
   Add four new LD/ST instructions: `LD rd, (rs)+`, `ST (rs)+, rd`, `LD rd, -(rs)` and `ST -(rs), rd`. The "Post-Increment" instructions (`(rs)+`) perform their LD/ST operations and then increment the address stored in
   the source address by 2. The "Pre-Decrement" instructions (`-(rs)`) decrement the address stored in the source register by 2 and then perform their LD/ST operation. Add these instructions to the `FF` prefix block.
   Each of these instructions are three bytes long and occupy eight sub-opcodes (32 sub-opcodes in total for all four instructions), each sub-opcode corresponds to one destination register.

---

## Instruction Cycle Timing Calculation

The total cycle count for any instruction is the sum of its **Fetch Cost** and its **Execution Cost**. All timings are measured in **T-cycles**.

### Rule 1: Fetch Cost

This is the baseline cost for the CPU to read the instruction itself from memory. The formula is straightforward:

- **Fetch Cost = (Instruction Size in Bytes) × 4 T-cycles**

For example, a 3-byte instruction like `LDI r, n16` has a fetch cost of $3 \times 4 = 12$ T-cycles.

### Rule 2: Execution Cost

This is the additional time an instruction takes to perform its specific job, such as accessing memory or doing complex internal work.

- **Internal Operations:** Simple register-to-register operations (e.g., `LD rd, rs`, `ADD rs`, `NEG r`) are considered fast and have an execution cost of **0 T-cycles**. Their work is completed within the final fetch cycle.

- **Memory Access:** Instructions that read from or write to memory during their execution phase incur the following costs:

| Memory Access Type    | WRAM Cost   | HRAM Cost   |
| :-------------------- | :---------- | :---------- |
| **16-bit Read/Write** | +8 T-cycles | +4 T-cycles |
| **8-bit Read/Write**  | +4 T-cycles | +2 T-cycles |

Stack operations like `PUSH` and `POP` are treated as 16-bit memory accesses.

### Rule 3: Control Flow Instructions

The documented cycle counts for control flow instructions represent their intrinsic work, not the systemic cost of a pipeline stall.

- **Jumps (`JMP`, `JR`, `Jcc`)**: These instructions only have a **Fetch Cost**. The work of changing the Program Counter is a fast internal operation with no extra cycle cost. Any delay from the pipeline stall is handled by the emulator, not included in the instruction's base time.

- **Calls & Returns (`CALL`, `RET`)**: These instructions are more expensive because their execution phase involves a **memory access** (pushing or popping the return address on the stack).
  - For example, `CALL n16` costs 12 cycles to fetch its 3 bytes, plus 8 cycles to execute the push of the Program Counter, for a total of 20 cycles.

### The Final Formula

The complete calculation for any instruction's documented cycle time is:

**Total Cycles = (Fetch Cost) + (Execution Cost)**

---

## Possible Sequel Console

The following ideas are things that I think may be appropriate to add to a possible sequel console ("Cicada-16 pro"?).

- Double buffered OAM/CRAM/VRAM
  - Double buffering all of the RAM areas shared between the CPU and the PPU, removes the need to "race the beam", no need to try to fit all visual update logic into the V-blank timing of each frame.
- Additional background layer
- Additional APU channels
- Enhanced WAV channel
  - increase wave sample size from 4 bits to 8, increase wave ram
- Advanced DSP effects
- Hardware multiplication/division

---

© 2025 Connor Nolan. This work is licensed under a
[Creative Commons Attribution-ShareAlike 4.0 International License](http://creativecommons.org/licenses/by-sa/4.0/).
