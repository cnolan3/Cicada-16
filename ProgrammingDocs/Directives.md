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

## .section / .section_end

Defines a section of code or data with specific attributes like size, virtual address, or physical address.

- **Syntax**:
  ```asm
  .section name="section_name" size=bytes vaddr=address paddr=address
      ; code or data goes here
  .section_end
  ```
- **Attributes**:
  - `name="string"`: Optional name for the section (for documentation purposes)
  - `size=bytes`: Reserve a specific number of bytes for the section (will pad with zeros if content is smaller)
  - `vaddr=address`: Set the logical (virtual) address for this section's contents
  - `paddr=address`: Set the physical ROM address where this section will be placed
- **Description**: The `.section` directive allows you to organize code and data with fine-grained control over memory placement and layout. This is particularly useful for:
  - Creating fixed-size memory regions (using `size=`)
  - Mapping code to specific logical addresses (using `vaddr=`)
  - Placing data at specific ROM locations (using `paddr=`)
  - Organizing code into named logical blocks

When a section ends, the logical address counter is restored to continue from where it would have been without the section's `vaddr` override. Physical address always advances forward.

**Restrictions**:

- Sections cannot be nested
- If a `size=` attribute is specified, the section content cannot exceed that size
- The `.section_end` directive must be present to close each section

**Example 1: Fixed-size font data section**

```asm
; Reserve exactly 2048 bytes for font data
.section name="font_data" size=2048 paddr=0x8000
font_tiles:
    .byte 0x00, 0x3C, 0x42, 0x42, 0x42, 0x3C, 0x00, 0x00  ; Character 'A'
    .byte 0x00, 0x7E, 0x42, 0x7E, 0x42, 0x7E, 0x00, 0x00  ; Character 'B'
    ; ... more font data
.section_end
; Assembler will pad to exactly 2048 bytes, then continue

next_data:
    .word 0x1234
```

**Example 2: Memory-mapped I/O with virtual addressing**

```asm
; Map a buffer to a specific logical address in a switchable bank
.bank 2
.section name="audio_buffer" vaddr=0x6000 size=1024
audio_samples:
    ; This buffer appears at logical address 0x6000-0x63FF
    ; but is physically stored in bank 2
.section_end

; After section ends, logical addressing resumes normally
process_audio:
    LDI R0, audio_samples  ; R0 = 0x6000 (the virtual address)
```

**Example 3: Organizing ROM structure**

```asm
; Place graphics data at a specific ROM location
.section name="sprite_tiles" paddr=0x10000
sprite_data:
    .byte 0xFF, 0xFF, 0xFF, 0xFF  ; Sprite tile data
    ; ...
.section_end

; Jump to next aligned section
.section paddr=0x12000
level_data:
    .word 100, 200, 300  ; Level layout data
.section_end
```

**Example 4: Relative jumps with virtual addressing**

```asm
.bank 1
.section vaddr=0x4100 name="main_loop"
game_loop:
    ; Code at logical address 0x4100
    CALL update_game
    CALL render_frame
    JR game_loop  ; Relative jump works correctly with logical addresses
.section_end
```

## Cartridge Metadata Directives

These directives are used to define the cartridge header and interrupt vector table, which are required for a valid cartridge file.

### .header_start / .header_end

These directives define a block that contains the cartridge header information.

- **Syntax**:
  ```asm
  .header_start
      ; header fields go here
  .header_end
  ```
- **Description**: All cartridge-related metadata must be placed between these two directives. The assembler will use the fields within this block to construct the 96-byte cartridge header at the beginning of the ROM.

The following directives are valid inside a `.header_start` / `.header_end` block:

- **.boot_anim**: A 4-character string for the boot animation ID.
- **.title**: The game's title (up to 16 characters).
- **.developer**: The developer's name (up to 16 characters).
- **.version**: The game's version number (e.g., `1`).
- **.mapper**: The cartridge's memory mapper type.
- **.rom_size**: An enum indicating the ROM size.
- **.ram_size**: An enum indicating the size of save RAM.
- **.interrupt_mode**: `0` for ROM-based vectors, `1` for RAM-based.
- **.hardware_rev**: The hardware revision this cartridge targets.
- **.region**: The intended region for the game.

**Example:**

```asm
.header_start
    .boot_anim "CICA"
    .title "My-Awesome-Game"
    .developer "Awesome-Dev"
    .version 1
    .mapper 0
    .rom_size 2 ; 64KB
    .ram_size 0 ; No RAM
    .interrupt_mode 1
    .hardware_rev 0
    .region 0 ; All regions
.header_end
```

- **Note**: String literals like those used for .boot_anim, .title and .developer cannot contain whitespace, this is a limitation that will be fixed in a later update.

### .interrupt_table / .table_end

These directives define the interrupt vector table.

- **Syntax**:
  ```asm
  .interrupt_table
      .word vblank_handler
      .word hblank_handler
      ; ... and so on
  .table_end
  ```
- **Description**: This block is used to define the addresses of your interrupt service routines (ISRs). The assembler will create a 32-byte table of these addresses. The order of the `.word` directives inside the block matters and must correspond to the interrupt vector order defined in `HardwareSpec/Interrupts.md`. You must provide at least 12 vectors, and at most 16.

**Example:**

```asm
.interrupt_table
    .word reset_handler         ; RESET
    .word bus_error_handler     ; Bus Error
    .word illegal_inst_handler  ; Illegal Instruction
    .word protected_mem_handler ; Protected Memory
    .word stack_overflow_handler; Stack Overflow
    .word vblank_handler        ; V-Blank
    .word hblank_handler        ; H-Blank
    .word lyc_handler           ; LY == LYC
    .word timer_handler         ; Timer
    .word serial_handler        ; Serial
    .word link_status_handler   ; Link Status
    .word joypad_handler        ; Joypad
.table_end
```

---

© 2025 Connor Nolan. This work is licensed under a
[Creative Commons Attribution-ShareAlike 4.0 International License](http://creativecommons.org/licenses/by-sa/4.0/).
