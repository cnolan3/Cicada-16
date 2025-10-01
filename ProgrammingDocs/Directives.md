# Assembler Directives

Assembler directives are commands that are interpreted by the assembler (`casm`) at assembly time. They are not translated into machine code but are used to control the assembly process, define data, and organize the final binary.

All directives begin with a period (`.`).

## .org

Sets the location counter, telling the assembler where to place the subsequent code or data in memory.

- **Syntax**: `.org address`
- **Operand**: A 16-bit immediate address or a label.
- **Description**: The `.org` directive sets the starting address for the code that follows it. The assembler will pad the output with zeros if the new address is greater than the current location counter.

```asm
.org 0x100 ; Start assembling at address 0x0100

start:
    LDI R1, 0xFF

.org 0x200 ; Jump to address 0x0200
my_data:
    .byte 10
```

## .define

Creates a named constant that can be used in place of an immediate value.

- **Syntax**: `.define NAME value`
- **Operands**:
    - `NAME`: An identifier for the constant.
    - `value`: A 16-bit immediate value.
- **Description**: The `.define` directive is used to create a constant. The assembler will substitute every occurrence of `NAME` with its corresponding `value` during a pre-processing pass. This is useful for defining configuration values and other "magic numbers" in a readable way. Unlike labels, defined constants do not have an address.

```asm
.define SCREEN_WIDTH 320
.define SCREEN_HEIGHT 240

LDI R1, SCREEN_WIDTH ; This is assembled as LDI R1, 320
```

## .bank

Selects the memory bank to which subsequent code and data will be assembled.

- **Syntax**: `.bank number`
- **Operand**: An immediate value representing the bank number.
- **Description**: The Cicada-16 architecture uses memory banking to access more than 64KB of memory. Each bank is 16KB. This directive tells the assembler to switch to a new bank. The location counter is reset to the start of the specified bank (e.g., `.bank 1` sets the address to `0x4000`).

```asm
.bank 0 ; Code in bank 0
    JMP.far bank1_start

.bank 1 ; Code in bank 1
bank1_start:
    NOP
```

## .include

Includes another source file into the current file at the location of the directive.

- **Syntax**: `.include "path/to/file.asm"`
- **Operand**: A string literal containing the path to the source file to be included.
- **Description**: The `.include` directive instructs the assembler to pause parsing the current file and begin parsing the specified file. Once the included file is fully parsed, the assembler resumes parsing the original file. This allows you to split your code into multiple files for better organization. Paths are resolved relative to the file containing the `.include` directive.

```asm
; main.asm
.org 0x100
.include "constants.asm"
.include "subroutines.asm"

start:
    LDI R1, MY_CONSTANT ; MY_CONSTANT is defined in constants.asm
    CALL my_subroutine  ; my_subroutine is defined in subroutines.asm
```

## .byte

Defines one or more 8-bit constant values.

- **Syntax**: `.byte value1, value2, ...`
- **Operands**: A comma-separated list of 8-bit immediate values.
- **Description**: This directive reserves and initializes one or more bytes of memory with the specified values.

```asm
message: .byte 0x48, 0x65, 0x6C, 0x6C, 0x6F ; "Hello"
```

## .word

Defines one or more 16-bit constant values.

- **Syntax**: `.word value1, value2, ...`
- **Operands**: A comma-separated list of 16-bit immediate values or labels.
- **Description**: This directive reserves and initializes one or more 16-bit words of memory. If a label is provided, the assembler will substitute it with the label's 16-bit address.

```asm
; A table of values
data_table: .word 0x1000, 0x2000, 0x3000

; A table of pointers (addresses)
pointer_table: .word start, message, data_table
```

---

© 2025 Connor Nolan. This work is licensed under a
[Creative Commons Attribution-ShareAlike 4.0 International License](http://creativecommons.org/licenses/by-sa/4.0/).
