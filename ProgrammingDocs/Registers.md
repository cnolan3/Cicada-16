# Cicada-16 CPU Registers

This document describes the user-accessible registers in the Cicada-16 CPU.

## General-Purpose Registers

The CPU has eight 16-bit general-purpose registers, designated `R0` through `R7`.

- **`R0` - `R7`**: These registers can be used for storing 16-bit data or addresses. Most instructions can operate on any of these registers.

### R0 as Accumulator

For many arithmetic and logical instructions, the `R0` register acts as an implicit accumulator. This means it serves as both a source and the destination for the operation, resulting in more compact and efficient code.

For example, the `ADD R1` instruction is equivalent to `R0 = R0 + R1`.

## Special-Purpose Registers

### SP (Stack Pointer)

`SP` is the 16-bit Stack Pointer. It holds the address of the top of the stack. The stack grows downwards in memory.

- **`PUSH`**: When a value is pushed onto the stack, `SP` is decremented before the value is written.
- **`POP`**: When a value is popped from the stack, the value is read from the address in `SP`, and then `SP` is incremented.

The `SP` is also implicitly used by `CALL` and `RET` instructions.

### F (Flags Register)

`F` is a 16-bit register that holds various status flags reflecting the result of the most recent arithmetic or logical operation. You cannot access the `F` register directly, but its contents can be saved and restored using `PUSHF` and `POPF`.

The primary flags are:

- **Z (Zero Flag)**: Set if the result of an operation is zero; cleared otherwise.
- **N (Negative Flag)**: Set if the most significant bit (MSB) of the result is 1; cleared otherwise.
- **C (Carry Flag)**: Set if an operation resulted in a carry out of the most significant bit. This is used for unsigned arithmetic.
- **V (Overflow Flag)**: Set if an operation resulted in a signed overflow (e.g., adding two large positive numbers results in a negative number). This is used for signed arithmetic.

Conditional jump and call instructions (`Jcc`, `JRcc`, `CALLcc`) use the state of these flags to determine whether to execute.

---

© 2025 Connor Nolan. This work is licensed under a
[Creative Commons Attribution-ShareAlike 4.0 International License](http://creativecommons.org/licenses/by-sa/4.0/).
