# **Cicada-16 DMA - Architecture**

This document describes the design and functionality of the Direct Memory Access (DMA) controller for the Cicada-16 console. The DMA unit is responsible for performing fast memory copy operations, which significantly offloads the CPU and is essential for high-performance graphics.

## **1. DMA Registers (F00A–F010)**

The DMA system is controlled by a set of memory-mapped I/O registers.

| Address   | Name      | Description                                                          |
| :-------- | :-------- | :------------------------------------------------------------------- |
| F00A-F00B | `DMA_SRC` | The 16-bit source address for the transfer.                          |
| F00C-F00D | `DMA_DST` | The 16-bit destination address for the transfer.                     |
| F00E-F00F | `DMA_LEN` | The 16-bit length/parameter register. Usage depends on the DMA mode. |
| F010      | `DMA_CTL` | The DMA Control Register.                                            |

## **2. The DMA Transfer Process**

A DMA transfer is a multi-step process initiated by the CPU.

1.  **Set Source:** Write the 16-bit source address to `DMA_SRC` (low byte to `F00A`, high byte to `F00B`).
2.  **Set Destination:** Write the 16-bit destination address to `DMA_DST` (low byte to `F00C`, high byte to `F00D`). Depending on the DMA mode, this may be ignored or interpreted differently.
3.  **Set Length:** Write the 16-bit length or parameter value to `DMA_LEN` (low byte to `F00E`, high byte to `F00F`). The interpretation depends on the selected DMA mode.
4.  **Configure Control:** Set the desired transfer properties (e.g., DMA mode, address mode, VRAM-safe transfer) in the `DMA_CTL` register (`F010`).
5.  **Start Transfer:** Write a `1` to the `START` bit (bit 0) of the `DMA_CTL` register. The transfer will begin immediately (unless `VRAM_SAFE` is enabled).

## **3. CPU State and Timing**

To prevent conflicts over the system bus, the DMA controller uses a **CPU Halting** model.

- **CPU Halt:** The moment the `START` bit is written, the CPU is completely frozen and relinquishes control of the memory bus.
- **Timing:**
  - **Normal DMA (Mode 0):** The transfer takes **4 system clock cycles per byte** copied. For example, copying 256 bytes takes `256 * 4 = 1024` clock cycles.
  - **Special DMA Modes (Modes 1-6):** All special modes use high-speed transfer at **2 system clock cycles per byte**. The same 256-byte transfer would take only `256 * 2 = 512` clock cycles.
- **Completion:** Once the specified number of bytes has been transferred, the DMA controller clears the `START` bit in `DMA_CTL` to `0`, releases the memory bus, and the CPU resumes execution on the next clock cycle.

## **4. Control Register (`DMA_CTL`)**

The `DMA_CTL` register at `F010` configures the behavior of the transfer.

| Bit | Name        | Function                                                                                                                                                                                                                                                                                                           |
| :-- | :---------- | :----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 7-6 | -           | Reserved.                                                                                                                                                                                                                                                                                                          |
| 5-3 | `DMA_MODE`  | **DMA Mode Select.** A 3-bit field (0-7) that selects the DMA transfer mode. See section 5 for details on each mode.                                                                                                                                                                                               |
| 2   | `VRAM_SAFE` | **VRAM Safe Transfer.** If this bit is `1`, the DMA transfer will not begin until the PPU enters a non-rendering period (H-Blank or V-Blank). This prevents visual artifacts when writing to VRAM or CRAM, but adds a small, variable delay to the start of the transfer. If `0`, the transfer begins immediately. |
| 1   | `ADDR_MODE` | **Address Mode.** If `0`, the source and destination addresses increment after each byte is copied. If `1`, the addresses decrement. **Note:** This bit is ignored in special DMA modes that have fixed destination behavior.                                                                                      |
| 0   | `START`     | **Start/Status Bit.** Writing a `1` to this bit initiates the transfer. This bit will read as `1` while the transfer is in progress and is automatically cleared to `0` by the hardware upon completion.                                                                                                           |

## **5. DMA Modes**

The `DMA_MODE` field (bits 5-3) in the `DMA_CTL` register selects one of eight transfer modes. Each mode has different behaviors and uses the `DMA_SRC`, `DMA_DST`, and `DMA_LEN` registers in specific ways.

### **Mode 0: Normal DMA**

Standard memory-to-memory transfer with full flexibility.

- **DMA_SRC**: Source address in memory.
- **DMA_DST**: Destination address in memory.
- **DMA_LEN**: Number of bytes to transfer (1-65535). A value of 0 is treated as 65536 bytes.
- **Speed**: 4 clock cycles per byte.
- **Behavior**: Copies data from source to destination. The `ADDR_MODE` bit controls whether addresses increment or decrement after each byte.
- **Use case**: General-purpose memory transfers, copying tile data, loading level data.

### **Mode 1: System Library / OAM Scanline DMA**

This mode has two different behaviors depending on whether the console is in boot mode or not.

#### **During Boot (System Library DMA)**

- **DMA_SRC**: Source address in the Boot ROM.
- **DMA_DST**: Ignored. Destination is fixed to `E000` (System Library RAM).
- **DMA_LEN**: Ignored. Transfer size is fixed at 4096 bytes (4 KiB).
- **Speed**: 2 clock cycles per byte.
- **Behavior**: Copies the System Library from the internal Boot ROM to the dedicated System Library RAM region.
- **Availability**: This behavior is only available while the console is in its initial boot state. After boot completes, this mode is inaccessible and the mode switches to OAM Scanline DMA behavior.

#### **After Boot (OAM Scanline DMA)**

- **DMA_SRC**: Source address in ROM/RAM containing sprite data.
- **DMA_DST**: The low byte specifies the starting OAM sprite slot (0-63).
- **DMA_LEN**: Number of sprite entries to copy. Each sprite entry is 8 bytes, so the actual bytes transferred = `DMA_LEN × 8`.
- **Speed**: 2 clock cycles per byte.
- **Actual bytes transferred**: `DMA_LEN * 8`
- **Destination Calculation**: `F400 + (DMA_DST & 0x00FF)`
- **Behavior**: Copies sprite data to a specific portion of OAM. This allows partial sprite updates without transferring all 512 bytes.
- **Use case**: Update only active sprites, animate specific sprite groups, or perform incremental OAM updates. To copy all 64 sprites, set `DMA_DST = 0x0000` and `DMA_LEN = 64`.

### **Mode 2: VRAM Slot DMA**

Transfers data directly to a specific 2 KiB slot within VRAM's 32 KiB address space, bypassing the need for manual bank switching.

- **DMA_SRC**: Source address in ROM/RAM.
- **DMA_DST**: Low byte specifies the starting destination VRAM slot (0-15).
- **DMA_LEN**: Number of VRAM slots to transfer.
- **Speed**: 2 clock cycles per byte.
- **Actual bytes transferred**: `DMA_LEN * 2048`
- **Destination Calculation**: VRAM address = `(slot × 2048) + DMA_DST`
- **Behavior**: The hardware automatically routes writes to the correct internal VRAM bank. The CPU's VRAM bank register is not affected.
- **Use case**: Rapidly upload tilemap or tile graphics data to a specific VRAM region without managing bank switching. Ideal for level loading or dynamic tile updates.

### **Mode 3: CRAM (Palette) DMA**

Fast palette transfer for color updates.

- **DMA_SRC**: Source address in ROM/RAM containing palette data.
- **DMA_DST**: The low byte specifies the starting palette index (0-255).
- **DMA_LEN**: Number of palette entries (colors) to copy. Each entry is 2 bytes (RGB555 format).
- **Speed**: 2 clock cycles per byte.
- **Actual bytes transferred**: `DMA_LEN × 2`
- **Destination Calculation**: `F200 + (DMA_DST & 0x00FF) × 2`
- **Behavior**: Copies color data to CRAM starting at the specified palette index.
- **Use case**: Palette swaps, fade effects, day/night cycles, cutscene color changes. To update an entire 16-color sub-palette, set `DMA_LEN = 16`. To update all 256 colors, set `DMA_DST = 0` and `DMA_LEN = 256`.

### **Mode 4: Wave RAM DMA**

Transfers waveform data to the APU's Wave RAM for dynamic audio synthesis.

- **DMA_SRC**: Source address in ROM/RAM containing waveform data.
- **DMA_DST**: The low 5 bits specify the starting waveform slot (0-31).
- **DMA_LEN**: Number of waveforms to copy. Each waveform is 32 bytes (64 4-bit samples).
- **Speed**: 2 clock cycles per byte.
- **Actual bytes transferred**: `DMA_LEN × 32`
- **Destination Calculation**: `FA00 + ((DMA_DST & 0x001F) × 32)`
- **Behavior**: Copies waveform data to Wave RAM starting at the specified slot.
- **Use case**: Load custom instrument waveforms, switch sound effect samples, implement dynamic audio synthesis. To load a single waveform to slot 10, set `DMA_DST = 10` and `DMA_LEN = 1`. To load 4 consecutive waveforms starting at slot 0, set `DMA_DST = 0` and `DMA_LEN = 4`.

### **Mode 5: DSP Delay Buffer DMA**

Transfers data to or from the APU's DSP echo/delay buffer.

- **DMA_SRC**: Source address in ROM/RAM.
- **DMA_DST**: Ignored. Destination is fixed to `F600` (DSP Delay Buffer start).
- **DMA_LEN**: Number of bytes to copy (1-1024).
- **Speed**: 2 clock cycles per byte.
- **Behavior**: Copies data to the DSP's 1 KiB delay buffer. Can be used to pre-fill the buffer with specific patterns or clear it.
- **Use case**: Initialize custom reverb/echo effects, create special audio textures, clear the delay buffer for clean transitions.

### **Mode 6: Fill/Pattern Mode**

Hardware-accelerated memory fill operation that writes a repeating 16-bit pattern to memory.

- **DMA_SRC**: Contains the 16-bit pattern value to repeat (loaded as a word from `DMA_SRC` address).
- **DMA_DST**: Destination address in memory.
- **DMA_LEN**: Number of 16-bit words to write.
- **Speed**: 2 clock cycles per byte (4 cycles per 16-bit word).
- **Actual bytes written**: `DMA_LEN × 2`
- **Behavior**: Reads a single 16-bit value from the source address once, then repeatedly writes this value to consecutive 16-bit locations starting at the destination. This is significantly faster than a CPU loop for large fills.
- **Use case**: Clear tilemaps with a specific tile, initialize background layers, fill VRAM regions with solid patterns, bulk initialization of data structures.

### **Mode 7: Reserved**

This mode is reserved for future expansion.

---

© 2025 Connor Nolan. This work is licensed under a
[Creative Commons Attribution-ShareAlike 4.0 International License](http://creativecommons.org/licenses/by-sa/4.0/).
