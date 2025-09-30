# Introduction to Cicada-16 Assembly

Welcome to the official programmer's reference for the Cicada-16 assembly language. This document provides a high-level overview of the Cicada-16 architecture and the syntax used by the official assembler (`casm`).

## Architecture Overview

The Cicada-16 is a 16-bit CISC-style microprocessor. It features a straightforward memory model and a flexible instruction set designed to be powerful yet easy to learn.

### Key Features

- **16-bit Data Path**: The primary data size is 16 bits (a "word").
- **8 General-Purpose Registers**: A set of 8 versatile 16-bit registers.
- **16-bit Address Bus**: Can address up to 64 KB of memory per bank.
- **Memory Banking**: The architecture supports memory banking to extend beyond the 64 KB address space, with each bank being 16 KB in size.
- **Implicit Accumulator**: Many arithmetic and logic operations implicitly use the `R0` register as an accumulator for more compact instruction encoding.

## Assembly Language Syntax

### Comments

Comments begin with a semicolon (`;`) and extend to the end of the line. They are ignored by the assembler.

```asm
; This is a comment
LDI R1, 0x100 ; Load the value 0x100 into register R1
```

### Labels

Labels are used to mark specific memory addresses, such as the start of a subroutine or a data location. A label is an identifier followed by a colon (`:`).

```asm
my_label:
    NOP
    JMP my_label ; An infinite loop
```

### Numbers

Numbers can be specified in decimal or hexadecimal format.

- **Decimal**: Standard base-10 numbers (e.g., `10`, `255`, `-5`).
- **Hexadecimal**: Prefixed with `0x` or `$` (e.g., `0xFF`, `$1A00`).

### Registers

The 8 general-purpose registers are referred to as `R0` through `R7`. The stack pointer is referred to as `SP`.

### Directives

Directives are special commands for the assembler that are not translated into machine code but control how the code is assembled. They begin with a period (`.`).

```asm
.define MY_CONST 10 ; Define a constant
.org 0x100   ; Assemble the following code starting at address 0x100
.byte 1, 2, 3 ; Define a sequence of bytes
```

---

© 2025 Connor Nolan. This work is licensed under a
[Creative Commons Attribution-ShareAlike 4.0 International License](http://creativecommons.org/licenses/by-sa/4.0/).
