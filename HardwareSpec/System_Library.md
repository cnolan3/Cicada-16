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
; Assume the vector number for '''fastMultiply16''' is 0x2A

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

**note**: the addresses of functions listed below have a placeholder of 0xXXXX for now

## **Graphics Routines**

### `initDefaultFont` : Index 0x05 (Addr 0xXXXX)

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

### **Default Font Data** : Index 0x00 (Addr 0xE100)

#### **Font Storage Format (1bpp vs 4bpp)**

To save a significant amount of space in the System Library, the default font is stored in a compact, 1-bit-per-pixel (1bpp) monochrome format.

- **Hardware Format (4bpp):** The PPU requires every 8x8 tile in VRAM to be in its native 4bpp planar format, which takes 32 bytes. Storing the 96-character font this way would consume `96 * 32 = 3072` bytes, which is too large.
- **Storage Format (1bpp):** In the 1bpp library format, each 8x8 character only requires 8 bytes (1 bit per pixel). The total storage size is therefore `96 * 8 = 768` bytes.

The `initDefaultFont` function handles the conversion, reading the compact 8-byte characters and expanding them into the 32-byte, 4bpp format that the PPU requires before writing them to VRAM.

## **Music and Sound Routines**

### **Note Frequency Table** : Index 0x02 (Addr 0xE500)

To simplify music creation, the System Library provides a pre-calculated lookup table containing the 16-bit `FREQ_REG` values for a range of standard musical notes. This allows developers to play specific pitches without needing to perform the frequency calculation manually.

- **Location:** The table resides at a fixed address within the System Library space.
- **Range:** The table covers 5 octaves, from C2 to B6.
- **Format:** The table is a simple array of 16-bit unsigned integers. Each entry corresponds to a note in the chromatic scale.

**Note:** The exact addresses and constant names will be finalized in the official toolchain documentation.

### **Sine Wave Table** : Index 0x01 (Addr 0xE400)

A 256-byte table containing a single cycle of a sine wave. This can be used by the APU's wave channel to produce a pure tone, or as a building block for more complex sounds.

### **Default APU Waves** : Index 0x03 (Addr 0xE578)

This is a 128-byte block containing four simple, ready-to-use 32-byte waveforms for the APU's wave channel. These include a sawtooth wave, a triangle wave, and others, providing a quick way to get varied sounds without having to define custom waveforms.

### **APU Percussion Presets** : Index 0x04 (Addr 0xE5F8)

A small 24-byte data block containing a set of pre-configured ADSR and timing parameters for creating common percussion sounds (like a kick drum or hi-hat) using the APU's noise channel.

## **Serial Communication Functions**

To simplify serial communication, the System Library provides functions to handle the low-level boilerplate of exchanging single bytes.

### `serialExchangeByte` : Index 0x06 (Addr 0xXXXX)

This function, intended to be called by the **Master** console, simultaneously sends one byte and receives one byte.

- **Input:** A byte to send, typically passed in a general-purpose register (e.g., the low byte of `R0`).
- **Action:**
  1.  Waits until the serial port is not busy (i.e., the `START` bit in the `SC` register is 0).
  2.  Writes the input byte to the **`SB`** (Serial Buffer) register at `F002`.
  3.  Sets the `START` bit in the **`SC`** (Serial Control) register at `F003` to begin the transfer.
  4.  Waits for the transfer to complete (i.e., for the `START` bit to be cleared by hardware).
- **Output:** The byte received from the other console during the exchange. This is read from the `SB` register after the transfer and returned, typically in a general-purpose register.

**Note:** This is a blocking function. It will halt CPU execution until the transfer is complete. It should only be called by the Master console (`CLK_SRC = 0`). The Slave console must have its outgoing byte pre-loaded into its `SB` register before this function is called by the Master.

### `serialByteWrite` : Index 0x07 (Addr 0xXXXX)

This non-blocking function simply writes a byte to the serial buffer. It is intended to be used by the **Slave** console to prepare the next byte for transmission before the Master initiates an exchange.

- **Input:** A byte to write, typically passed in a general-purpose register (e.g., the low byte of `R0`).
- **Action:**
  1.  Writes the input byte to the **`SB`** (Serial Buffer) register at `F002`.
- **Output:** None.

### `serialByteRecv` : Index 0x08 (Addr 0xXXXX)

This function reads a single byte from the serial buffer. It is designed to be lightweight and is typically called from within the Serial Transfer Complete interrupt service routine, primarily on the **Slave** console.

- **Input:** None.
- **Action:**
  1.  Reads the byte from the **`SB`** (Serial Buffer) register at `F002`.
- **Output:** The received byte, typically returned in a general-purpose register (e.g., the low byte of `R0`).

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

## **Advanced Arithmetic Functions**

Since the Cicada-16 CPU does not have hardware support for multiplication or division, the System Library provides highly optimized routines for these common operations.

### `fastMultiply16` : Index 0x09 (Addr 0xXXXX)

Multiplies two 16-bit unsigned integers and returns a 32-bit result.

- **Inputs:**
  - `R0`: Multiplicand (16-bit)
  - `R1`: Multiplier (16-bit)
- **Output:**
  - `R0`: High word of the 32-bit result.
  - `R1`: Low word of the 32-bit result.
- **Clobbered Registers:** `R2`, `R3` (These registers are used internally by the function and their previous values will be lost).

### `fastDivide32` : Index 0x0A (Addr 0xXXXX)

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

### `fastMultiply8` : Index 0x0B (Addr 0xXXXX)

Multiplies two 8-bit unsigned integers and returns a 16-bit result. This is the fastest multiplication routine.

- **Inputs:**
  - `R0.b`: Multiplicand (low byte of R0).
  - `R1.b`: Multiplier (low byte of R1).
- **Output:**
  - `R0`: 16-bit result.
- **Clobbered Registers:** `R1`.

### `fastDivide16` : Index 0x0C (Addr 0xXXXX)

Divides a 16-bit unsigned integer by an 8-bit unsigned integer.

- **Inputs:**
  - `R0`: 16-bit dividend.
  - `R1.b`: 8-bit divisor (low byte of R1).
- **Output:**
  - `R0.h`: 8-bit remainder (high byte of R0).
  - `R0.l`: 8-bit quotient (low byte of R0).
- **Error Handling:** If the divisor in `R1.b` is zero, the function will immediately return, setting the **Carry Flag (F.C)** to 1. The contents of `R0` will be undefined in this case.
- **Clobbered Registers:** `R1`, `R2`.

## **Data Decompression Functions**

### `decompressRLE` : Index 0x0D (Addr 0xXXXX)

Decompresses data that was compressed using a Run-Length Encoding (RLE) scheme. The function processes control bytes to expand runs of repeated data and copy raw data blocks.

- **RLE Format Definition:**

  - **Run Packet (Bit 7 = 1):** A control byte with the high bit set indicates a run of repeated data.
    - `1NNNNNNN`: The lower 7 bits (`N`) specify the number of times (`N+1`) to repeat the data byte that immediately follows.
    - Example: `0x83 0xAA` would decompress to `0xAA 0xAA 0xAA 0xAA` (4 bytes).
  - **Raw Packet (Bit 7 = 0):** A control byte with the high bit clear indicates a block of raw, uncompressed data.
    - `0NNNNNNN`: The lower 7 bits (`N`) specify the number of raw bytes (`N+1`) to copy directly from the source to the destination.
    - Example: `0x02 0x11 0x22 0x33` would decompress to `0x11 0x22 0x33` (3 bytes).
  - **End of Stream:** A control byte of `0xFF` (or -1) marks the end of the compressed data stream.

- **Inputs:**
  - `R0`: Source address (pointer to the compressed RLE data).
  - `R1`: Destination address (pointer to the RAM where data will be decompressed).
- **Output:**
  - `R0`: Address of the byte following the end-of-stream marker.
  - `R1`: Address of the byte following the last written destination byte.
- **Clobbered Registers:** `R2`, `R3`.
- **Important Note:** The developer is responsible for ensuring the destination buffer in RAM is large enough to hold the fully decompressed data. This function does not perform any bounds checking.

---

© 2025 Connor Nolan. This work is licensed under a
[Creative Commons Attribution-ShareAlike 4.0 International License](http://creativecommons.org/licenses/by-sa/4.0/).

