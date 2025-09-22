# **Cicada-16 System Library**

This document describes the functions that are available in the permanent System Library, which is located in its own dedicated, read-only RAM region at **`E000-EFFF`**.

These functions are copied from the internal Boot ROM to the System Library RAM by the boot process and are available for any game to use. They provide standardized, optimized routines for common tasks. After the boot sequence completes, this memory region is made read-only to protect the library's integrity.

## **System Call Vector Table**

To provide a stable and future-proof API, all System Library functions and data are accessed indirectly through a **System Call Vector Table**. This table is located at the very beginning of the System Library RAM, starting at `E000`.

- **Size**: The table has 128 entries, each being a 16-bit pointer. This provides a total of 128 unique system calls.
- **Memory Footprint**: 128 entries \* 2 bytes/entry = 256 bytes (`E000-E0FF`).
- **Function**: Each entry in the table holds the absolute 16-bit address of a specific system function or data block. To call a function, a developer uses its official vector number (e.g., `SYSCALL_DECOMPRESS_RLE`) to look up the address in the table and then performs an indirect call.
- **Data Blocks**: For library data blocks (such as the Default Font, Note Frequency Table, Default Waveforms, and Percussion Presets), the vector table contains a single entry that points to the beginning address of the respective data block.

This vector table approach ensures that even if the internal layout of the System Library changes in future hardware revisions, the vector numbers for existing functions will remain the same, maintaining backward compatibility for all software.

**Example Usage (Conceptual):**

```assembly
; Assume R0 points to the start of the vector table (E000)
; Assume the vector number for 'fastMultiply16' is 0x2A

; Calculate the address of the vector entry
LDI R1, 0x2A * 2 ; Vector number * 2 bytes per entry
ADD R0, R1       ; R0 now points to the correct entry in the table

; Load the function's actual address from the vector table
LD.w R0, (R0)

; Call the function
CALL (R0)
```

## **Memory Layout and Budget**

The 4 KiB (4096 bytes) of System Library RAM is allocated as follows:

| Component                  | Size (Bytes) | Address Range (Approx.) | Notes                                 |
| :------------------------- | :----------- | :---------------------- | :------------------------------------ |
| **System Call Vectors**    | 256          | `E000-E0FF`             | 128 entries \* 2 bytes each           |
| **Default Font Data**      | 768          | `E100-E3FF`             | 96 characters \* 8 bytes each (1bpp)  |
| **Sine Wave Table**        | 256          | `E400-E4FF`             | 256 samples for wave synthesis        |
| **Note Frequency Table**   | 120          | `E500-E577`             | 5 octaves _ 12 notes _ 2 bytes each   |
| **Default APU Waves**      | 128          | `E578-E5F7`             | 4 waveforms \* 32 bytes each          |
| **APU Percussion Presets** | 24           | `E5F8-E610`             | ADSR/timing for drum sounds           |
| **(Function Code)**        | 2544         | `E611-EFFF`             | Remaining space for library functions |
| **Total**                  | **4096**     | `E000-EFFF`             |                                       |

## Vector Table Index Map

| Index     | Type       | Name                     |
| :-------- | :--------- | :----------------------- |
| 0x00      | Data Block | `Default Font Data`      |
| 0x01      | Data Block | `Sine Wave Table`        |
| 0x02      | Data Block | `Note Frequency Table`   |
| 0x03      | Data Block | `Default APU Waves`      |
| 0x04      | Data Block | `APU Percussion Presets` |
| 0x05      | Function   | `initDefaultFont`        |
| 0x06      | Function   | `serialExchangeByte`     |
| 0x07      | Function   | `serialByteWrite`        |
| 0x08      | Function   | `serialByteRecv`         |
| 0x09      | Function   | `fastMultiply16`         |
| 0x0A      | Function   | `fastDivide32`           |
| 0x0B      | Function   | `fastMultiply8`          |
| 0x0C      | Function   | `fastDivide16`           |
| 0x0D      | Function   | `decompressRLE`          |
| 0x0E      | Function   | `clearTilemap`           |
| 0x0F      | Function   | `waitForVBlank`          |
| 0x10      | Function   | `drawChar`               |
| 0x11      | Function   | `drawString`             |
| 0x12      | Function   | `setPalette`             |
| 0x13      | Function   | `memcpy`                 |
| 0x14      | Function   | `memset`                 |
| 0x15      | Function   | `setBankROM`             |
| 0x16      | Function   | `setBankWRAM`            |
| 0x17      | Function   | `setBankVRAM`            |
| 0x18      | Function   | `playSoundEffect`        |
| 0x19      | Function   | `initMusicDriver`        |
| 0x1A      | Function   | `updateMusicDriver`      |
| 0x1B      | Function   | `readJoypad`             |
| 0x1C      | Function   | `readJoypadTrigger`      |
| 0x1D      | Function   | `rand`                   |
| 0x1E      | Function   | `setInterruptHandler`    |
| 0x1F      | Function   | `dmaCopy`                |
| 0x20      | Function   | `dmaCopyVBlank`          |
| 0x21      | Function   | `callFar`                |
| 0x22      | Function   | `jmpFar`                 |
| 0x23-0x7F | ---        | **Unused**               |

## **System Library Functions and Data**

### **Default Font Data** : Index 0x00

#### **Font Storage Format (1bpp vs 4bpp)**

To save a significant amount of space in the System Library, the default font is stored in a compact, 1-bit-per-pixel (1bpp) monochrome format.

- **Hardware Format (4bpp):** The PPU requires every 8x8 tile in VRAM to be in its native 4bpp planar format, which takes 32 bytes. Storing the 96-character font this way would consume `96 * 32 = 3072` bytes, which is too large.
- **Storage Format (1bpp):** In the 1bpp library format, each 8x8 character only requires 8 bytes (1 bit per pixel). The total storage size is therefore `96 * 8 = 768` bytes.

The `initDefaultFont` function handles the conversion, reading the compact 8-byte characters and expanding them into the 32-byte, 4bpp format that the PPU requires before writing them to VRAM.

### **Sine Wave Table** : Index 0x01

A 256-byte table containing a single cycle of a sine wave. This can be used by the APU's wave channel to produce a pure tone, or as a building block for more complex sounds.

### **Note Frequency Table** : Index 0x02

To simplify music creation, the System Library provides a pre-calculated lookup table containing the 16-bit `FREQ_REG` values for a range of standard musical notes. This allows developers to play specific pitches without needing to perform the frequency calculation manually.

- **Location:** The table resides at a fixed address within the System Library space.
- **Range:** The table covers 5 octaves, from C2 to B6.
- **Format:** The table is a simple array of 16-bit unsigned integers. Each entry corresponds to a note in the chromatic scale.

**Note:** The exact addresses and constant names will be finalized in the official toolchain documentation.

### **Default APU Waves** : Index 0x03

This is a 128-byte block containing four simple, ready-to-use 32-byte waveforms for the APU's wave channel. These include a sawtooth wave, a triangle wave, and others, providing a quick way to get varied sounds without having to define custom waveforms.

### **APU Percussion Presets** : Index 0x04

A small 24-byte data block containing a set of pre-configured ADSR and timing parameters for creating common percussion sounds (like a kick drum or hi-hat) using the APU's noise channel.

### `initDefaultFont` : Index 0x05

This function initializes the default system font by copying it from the System Library's compact storage format into VRAM in the PPU-ready 4bpp format.

- **Inputs:**
  - `R0`: The starting address in VRAM where the expanded font data should be written.
- **Action:**
  1.  Reads the 1bpp font data from its internal location within the System Library.
  2.  Iterates through each of the 96 characters.
  3.  For each character, it expands the 8 bytes of 1bpp data into the 32 bytes required for the 4bpp planar format.
  4.  Writes the resulting 32-byte tile to the destination address in VRAM specified by `R0`.
  5.  `R0` is incremented by 32 after each tile is written.
- **Output:** None.
- **Clobbered Registers:** `R1`, `R2`, `R3`.

### `serialExchangeByte` : Index 0x06

This function, intended to be called by the **Master** console, simultaneously sends one byte and receives one byte.

- **Inputs:**
  - `R0.b`: A byte to send.
- **Action:**
  1.  Waits until the serial port is not busy (i.e., the `START` bit in the `SC` register is 0).
  2.  Writes the input byte to the **`SB`** (Serial Buffer) register at `F002`.
  3.  Sets the `START` bit in the **`SC`** (Serial Control) register at `F003` to begin the transfer.
  4.  Waits for the transfer to complete (i.e., for the `START` bit to be cleared by hardware).
- **Output:**
  - `R0.b`: The byte received from the other console.

**Note:** This is a blocking function. It will halt CPU execution until the transfer is complete. It should only be called by the Master console (`CLK_SRC = 0`). The Slave console must have its outgoing byte pre-loaded into its `SB` register before this function is called by the Master.

### `serialByteWrite` : Index 0x07

This non-blocking function simply writes a byte to the serial buffer. It is intended to be used by the **Slave** console to prepare the next byte for transmission before the Master initiates an exchange.

- **Inputs:**
  - `R0.b`: A byte to write.
- **Action:**
  1.  Writes the input byte to the **`SB`** (Serial Buffer) register at `F002`.
- **Output:** None.

### `serialByteRecv` : Index 0x08

This function reads a single byte from the serial buffer. It is designed to be lightweight and is typically called from within the Serial Transfer Complete interrupt service routine, primarily on the **Slave** console.

- **Inputs:** None.
- **Action:**
  1.  Reads the byte from the **`SB`** (Serial Buffer) register at `F002`.
- **Output:**
  - `R0.b`: The received byte.

**Example ISR Usage on Slave Console (Conceptual):**

```assembly
; Serial Transfer Complete ISR (on Slave)
Serial_ISR:
    ; The master has just finished a transfer.
    ; Our outgoing byte was sent, and we have received a byte from the master.

    ; Call the library function to get the byte from the buffer.
    CALL serialByteRecv

    ; R0 now holds the received byte.
    ; Process the byte (e.g., store it in a RAM buffer).
    ...

    ; Pre-load the SB register for the *next* transfer using the new function.
    LD.b R1, (next_byte_to_send) ; Load the next byte into a register
    CALL serialByteWrite        ; Call the library function to write it to SB

    RETI ; Return from interrupt
```

### `fastMultiply16` : Index 0x09

Multiplies two 16-bit unsigned integers and returns a 32-bit result.

- **Inputs:**
  - `R0`: Multiplicand (16-bit)
  - `R1`: Multiplier (16-bit)
- **Output:**
  - `R0`: High word of the 32-bit result.
  - `R1`: Low word of the 32-bit result.
- **Clobbered Registers:** `R2`, `R3` (These registers are used internally by the function and their previous values will be lost).

### `fastDivide32` : Index 0x0A

Divides a 32-bit unsigned integer by a 16-bit unsigned integer.

- **Inputs:**
  - `R0`: High word of the 32-bit dividend.
  - `R1`: Low word of the 32-bit dividend.
  - `R2`: 16-bit divisor.
- **Output:**
  - `R0`: 16-bit quotient.
  - `R1`: 16-bit remainder.
- **Error Handling:** If the divisor in `R2` is zero, the function will immediately return, setting the **Carry Flag (F.C)** to 1. The contents of `R0` and `R1` will be undefined in this case.
- **Clobbered Registers:** `R3`.

### `fastMultiply8` : Index 0x0B

Multiplies two 8-bit unsigned integers and returns a 16-bit result. This is the fastest multiplication routine.

- **Inputs:**
  - `R0.b`: Multiplicand (low byte of R0).
  - `R1.b`: Multiplier (low byte of R1).
- **Output:**
  - `R0`: 16-bit result.
- **Clobbered Registers:** `R1`.

### `fastDivide16` : Index 0x0C

Divides a 16-bit unsigned integer by an 8-bit unsigned integer.

- **Inputs:**
  - `R0`: 16-bit dividend.
  - `R1.b`: 8-bit divisor (low byte of R1).
- **Output:**
  - `R0.h`: 8-bit remainder (high byte of R0).
  - `R0.l`: 8-bit quotient (low byte of R0).
- **Error Handling:** If the divisor in `R1.b` is zero, the function will immediately return, setting the **Carry Flag (F.C)** to 1. The contents of `R0` will be undefined in this case.
- **Clobbered Registers:** `R1`, `R2`.

### `decompressRLE` : Index 0x0D

Decompresses data that was compressed using a Run-Length Encoding (RLE) scheme. The function processes control bytes to expand runs of repeated data and copy raw data blocks.

- **Inputs:**
  - `R0`: Source address (pointer to the compressed RLE data).
  - `R1`: Destination address (pointer to the RAM where data will be decompressed).
- **Action:**
  - Decompresses RLE data from source to destination.
- **Output:**
  - `R0`: Address of the byte following the end-of-stream marker.
  - `R1`: Address of the byte following the last written destination byte.
- **Clobbered Registers:** `R2`, `R3`.
- **Important Note:** The developer is responsible for ensuring the destination buffer in RAM is large enough to hold the fully decompressed data. This function does not perform any bounds checking.

### `clearTilemap` : Index 0x0E

A highly optimized routine to fill a rectangular area of a background tilemap in VRAM with a specific tile entry. This is far faster than a developer's own loop in game code. It's essential for clearing the screen or dialogue boxes.

- **Inputs:**
  - R0: VRAM address of the tilemap
  - R1: Tile entry value to write (16-bit)
  - R2.b: Width of area in tiles
  - R3.b: Height of area in tiles
- **Action:**
  - Fills the specified rectangular area of the tilemap with the given tile entry.
- **Output:** None.
- **Clobbered Registers:** R0, R1, R2, R3.

### `waitForVBlank` : Index 0x0F

This is arguably the most critical function for any game. It puts the CPU into a low-power HALT state until the V-Blank interrupt occurs. This synchronizes the game loop to the screen's refresh rate (preventing tearing) and is the correct way to idle, as it saves power and frees the memory bus. Every game's main loop would call this once per frame.

- **Inputs:** None.
- **Action:**
  - Halts the CPU until a V-Blank interrupt occurs.
- **Output:** None.
- **Clobbered Registers:** None.

### `drawChar` : Index 0x10

You already have initDefaultFont to load the font tiles; these functions would use them. drawChar writes a single character's tile index to a specified tilemap location.

- **Inputs:**
  - R0: Character to draw (ASCII)
  - R1: Pointer to destination in VRAM tilemap
- **Action:**
  - Writes the tile index for the given character to the specified tilemap location.
- **Output:** None.
- **Clobbered Registers:** R2, R3.

### `drawString` : Index 0x11

drawString iterates over a null-terminated string in RAM and calls drawChar for each character, handling advancing the "cursor" position on the tilemap.

- **Inputs:**
  - R0: Pointer to null-terminated string
  - R1: Pointer to destination in VRAM tilemap
  - R2.b: Tilemap width (for line wrapping)
- **Action:**
  - Iterates over the string and draws each character to the tilemap, handling line wrapping.
- **Output:**
  - R0 and R1 are updated to point past the end of the source/destination.
- **Clobbered Registers:** R2, R3.

### `setPalette` : Index 0x12

A simple helper to copy a block of color data into the PPU's CRAM. It would essentially be an optimized memcpy but serves to standardize the process. Developers should still be taught to only call this during V-Blank.

- **Inputs:**
  - R0: Source address of palette data
  - R1: Destination CRAM index (0-255)
  - R2.b: Number of colors to copy
- **Action:**
  - Copies a block of color data to CRAM.
- **Output:** None.
- **Clobbered Registers:** R0, R1, R2, R3.

### `memcpy` : Index 0x13

Standard C library equivalents. memcpy copies a block of memory from a source to a destination.

- **Inputs:**
  - R0: Source
  - R1: Destination
  - R2: Length in bytes
- **Action:**
  - Copies a block of memory.
- **Output:** None.
- **Clobbered Registers:** R0, R1, R2, R3.

### `memset` : Index 0x14

memset fills a block of memory with a specific byte value. These would be written in highly optimized assembly, making them faster than any loop a developer might write themselves. They're used for everything from clearing RAM to initializing data structures.

- **Inputs:**
  - R0: Destination
  - R1.b: Value to write
  - R2: Length in bytes
- **Action:**
  - Fills a block of memory with a specific byte value.
- **Output:** None.
- **Clobbered Registers:** R0, R1, R2, R3.

### `setBankROM` : Index 0x15

Since bank switching is a core mechanic of the console, providing standardized functions to do it is a must. This function would simply take a bank number and write it to the correct I/O register (MPR_BANK).

- **Inputs:**
  - R0.b: Bank number
- **Action:**
  - Writes the given bank number to the MPR_BANK register.
- **Output:** None.
- **Clobbered Registers:** None.

### `setBankWRAM` : Index 0x16

This function would simply take a bank number and write it to the correct I/O register (WRAM_BANK).

- **Inputs:**
  - R0.b: Bank number
- **Action:**
  - Writes the given bank number to the WRAM_BANK register.
- **Output:** None.
- **Clobbered Registers:** None.

### `setBankVRAM` : Index 0x17

This function would simply take a bank number and write it to the correct I/O register (VRAM_BANK).

- **Inputs:**
  - R0.b: Bank number
- **Action:**
  - Writes the given bank number to the VRAM_BANK register.
- **Output:** None.
- **Clobbered Registers:** None.

### `playSoundEffect` : Index 0x18

This would be a huge quality-of-life improvement. Instead of manually setting 5-6 APU registers, a developer could call this single function. It would take a pointer to a small data structure in RAM/ROM that defines a sound effect (e.g., which channel to use, initial frequency, ADSR settings, noise parameters, etc.). The function reads this structure and configures the appropriate APU channel.

- **Inputs:**
  - R0: Pointer to sound effect data structure
  - R1.b: Channel to play on (0-3)
- **Action:**
  - Configures and plays a sound effect on the specified APU channel.
- **Output:** None.
- **Clobbered Registers:** R2.

### `initMusicDriver` : Index 0x19

A simple, tick-based music driver. initMusicDriver would take a pointer to the song data and set up internal state variables.

- **Inputs:**
  - R0: Pointer to music data
- **Action:**
  - Initializes the music driver with the given song data.
- **Output:** None.
- **Clobbered Registers:** Varies (would need several internal state registers).

### `updateMusicDriver` : Index 0x1A

updateMusicDriver would be called once per frame (typically from the V-Blank interrupt) to process the next "tick" of the song data, read note/effect commands, and update the APU registers accordingly. This abstracts away the entire complexity of a music tracker engine.

- **Inputs:** None.
- **Action:**
  - Processes the next tick of the current song data and updates the APU.
- **Output:** None.
- **Clobbered Registers:** Varies (would need several internal state registers).

### `readJoypad` : Index 0x1B

This function performs the necessary sequence of writes and reads to the JOYP register to poll all three button groups (D-Pad, Action, Utility). It then combines the results into a single, clean 16-bit bitmask. This is much more convenient than doing it manually.

- **Inputs:** None.
- **Action:**
  - Polls the joypad and returns the current state of all buttons.
- **Output:**
  - R0: A 16-bit bitmask where each bit represents a button (e.g., bit 0 = Right, bit 1 = Left, bit 8 = A, etc.).
- **Clobbered Registers:** R1.

### `readJoypadTrigger` : Index 0x1C

An even more useful input function. It would call readJoypad and compare the result with the state from the previous frame (which it stores internally in System Library RAM). It returns a bitmask of only the buttons that were just pressed on this frame (a 0-to-1 transition). This is what most game logic actually needs (e.g., "jump when A is pressed," not "jump while A is held down").

- **Inputs:** None.
- **Action:**
  - Polls the joypad and returns a bitmask of newly pressed buttons.
- **Output:**
  - R0: A 16-bit bitmask of newly pressed buttons.
- **Clobbered Registers:** R1.

### `rand` : Index 0x1D

Provides a standardized Pseudo-Random Number Generator (PRNG). The first time it's called, it could seed itself using the free-running DIV register. Subsequent calls would use a fast algorithm (like a Linear Congruential Generator or a Xorshift) to produce the next number in the sequence. Absolutely essential for any game with random elements.

- **Inputs:** None.
- **Action:**
  - Generates a pseudo-random number.
- **Output:**
  - R0: A 16-bit pseudo-random number.
- **Clobbered Registers:** R1.

### `setInterruptHandler` : Index 0x1E

For games running in "Enhanced Mode" (RAM-based vectors), this provides a safe, standardized way to change an interrupt service routine. It takes an interrupt vector number and a function pointer and writes the address into the correct slot in the WRAM vector table (0xC000). This is safer than having the developer hardcode memory addresses.

- **Inputs:**
  - R0.b: Interrupt vector number
  - R1: Address of the new ISR
- **Action:**
  - Sets the interrupt handler for the given vector to the given address.
- **Output:** None.
- **Clobbered Registers:** None.

### `dmaCopy` : Index 0x1F

A simple wrapper that configures and starts a DMA transfer. The developer provides source, destination, and length, and this function writes them to the DMA_SRC, DMA_DST, and DMA_LEN registers before setting the START bit. It abstracts away the I/O addresses.

- **Inputs:**
  - R0: Source
  - R1: Destination
  - R2: Length
- **Action:**
  - Starts a DMA transfer.
- **Output:** None.
- **Clobbered Registers:** None (The CPU is halted during the transfer).

### `dmaCopyVBlank` : Index 0x20

The most common use for DMA is updating graphics in VRAM. This is a specialized version of dmaCopy that automatically sets the VRAM_SAFE bit in the DMA_CTL register. This guarantees the transfer will only begin during a non-rendering period (H-Blank or V-Blank), preventing screen tearing and other artifacts automatically.

- **Inputs:**
  - R0: Source
  - R1: Destination
  - R2: Length
- **Action:**
  - Starts a VRAM-safe DMA transfer.
- **Output:** None.
- **Clobbered Registers:** None.

### `callFar` : Index 0x21

This function provides a "trampoline" to call a function located in a different ROM bank and have it return seamlessly. It handles switching to the target bank, calling the function, and switching back to the original bank automatically. This is the standard mechanism for cross-bank function calls.

This function is designed to be as transparent as possible, allowing registers `R0-R3` to be used for passing arguments to the far function.

- **Inputs:**
  - `R4.b`: The number of the ROM bank to switch to.
  - `R5`: The 16-bit address of the function to call within the target bank.
- **Action:**
  1.  Reads and saves the current ROM bank number.
  2.  Switches to the target ROM bank specified in `R4`.
  3.  Calls the function at the address specified in `R5`.
  4.  Waits for the called function to return.
  5.  Switches back to the original ROM bank.
  6.  Returns to the caller.
- **Register Usage:**
  - **Argument Passing:** Use registers `R0`, `R1`, `R2`, and `R3` to pass arguments to the target (far) function.
  - **Return Values:** Return values from the far function in `R0-R3` are preserved and passed back to the original caller.
  - **Clobbered Registers:** `R4` and `R5` are used as inputs by this function and their contents may be clobbered. All other registers (`R0-R3`, `R6`) are preserved.

### `jmpFar` : Index 0x22

This function provides a "trampoline" to jump to a label located in a different ROM bank. It handles switching to the target bank and then jumping, effectively transferring execution control without returning. This is the standard mechanism for cross-bank jumps.

- **Inputs:**
  - `R4.b`: The number of the ROM bank to switch to.
  - `R5`: The 16-bit address of the label to jump to within the target bank.
- **Action:**
  1.  Switches to the target ROM bank specified in `R4`.
  2.  Jumps to the address specified in `R5`. Execution does not return to the caller.
- **Register Usage:**
  - The state of all registers is preserved during the bank switch and jump. They are not used or modified by the `jmpFar` function itself.

---

© 2025 Connor Nolan. This work is licensed under a
[Creative Commons Attribution-ShareAlike 4.0 International License](http://creativecommons.org/licenses/by-sa/4.0/).
