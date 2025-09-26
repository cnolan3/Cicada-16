# Cicada-16 Addressing Modes

Cicada-16 provides several addressing modes to access data in registers and memory. The addressing mode determines how the CPU interprets the operand(s) of an instruction to find the data it needs to operate on.

## Register Direct

In this mode, the operand is a CPU register. The instruction operates directly on the data held within that register.

- **Syntax**: `R_n_`
- **Example**: `LD R1, R2` - Copies the 16-bit value from `R2` directly into `R1`.

## Immediate

In immediate mode, the operand is a constant value that is encoded as part of the instruction itself. This is used for loading constant values into registers.

- **Syntax**: `value` (decimal or hex)
- **Example**: `LDI R1, 0x1234` - Loads the 16-bit constant `0x1234` into register `R1`.

## Absolute

In absolute mode, the instruction contains a fixed 16-bit address that points to the location of the operand in memory.

- **Syntax**: `(address)`
- **Example**: `LD R1, (0x2020)` - Loads the 16-bit value from memory address `0x2020` into `R1`.

## Register Indirect

This mode uses a register to hold the address of the operand. The CPU reads the address from the specified register and then accesses the data at that address.

- **Syntax**: `(R_n_)`
- **Example**: `LD R1, (R2)` - Loads the 16-bit value from the memory location whose address is stored in `R2` into `R1`.

## Indexed

Indexed addressing is useful for accessing elements in arrays or data structures. The final memory address is calculated by adding a signed 8-bit offset to the value of a base register.

- **Syntax**: `(R_base_, offset)`
- **Example**: `LD R1, (R2, -4)` - Loads data from the address calculated by `R2 - 4` into `R1`.

## Register Indirect with Post-Increment

In this mode, the operand's address is specified by a register. After the data is accessed, the value of the register is automatically incremented (by 1 for byte operations, by 2 for word operations).

- **Syntax**: `(R_n_)+`
- **Example**: `LD R1, (R2)+` - Loads the 16-bit value from the address in `R2` into `R1`, and then increments `R2` by 2.

## Register Indirect with Pre-Decrement

In this mode, the value of the register is automatically decremented (by 1 for byte operations, by 2 for word operations) _before_ the data is accessed. The resulting address is then used to access the operand.

- **Syntax**: `-(R_n_)`
- **Example**: `LD R1, -(R2)` - Decrements `R2` by 2, then loads the 16-bit value from the new address in `R2` into `R1`.

## Label

Labels can be used in place of immediate addresses for jump/call targets or data access. The assembler resolves the label to its 16-bit address during the assembly process.

- **Syntax**: `label`
- **Example**: `JMP my_subroutine` - Jumps to the address associated with `my_subroutine`.
- **Example**: `LDI R1, my_data` - Loads the _address_ of `my_data` into `R1`.

---

© 2025 Connor Nolan. This work is licensed under a
[Creative Commons Attribution-ShareAlike 4.0 International License](http://creativecommons.org/licenses/by-sa/4.0/).
