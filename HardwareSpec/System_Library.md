# **Cricket-16 System Library**

This document describes the functions that are available in the permanent System Library, which is located in **Work RAM Bank 0 (WRAM0)** (B000-CFFF). The System Library occupies the first 2 KiB of WRAM0 (B000-B7FF).

These functions are copied from the internal Boot ROM to the System Library RAM by the boot process and are available for any game to use. They provide standardized, optimized routines for common tasks.

_This document will be updated with a full list of available functions, their addresses, and their usage instructions as they are defined._

## **Music and Sound Routines**

### **Note Frequency Table**

To simplify music creation, the System Library provides a pre-calculated lookup table containing the 16-bit `FREQ_REG` values for a range of standard musical notes. This allows developers to play specific pitches without needing to perform the frequency calculation manually.

- **Location:** The table resides at a fixed address within the System Library space.
- **Range:** The table covers 5 octaves, from C2 to B6.
- **Format:** The table is a simple array of 16-bit unsigned integers. Each entry corresponds to a note in the chromatic scale.

**Example Usage (Conceptual):**

A developer could access the table using constants defined in an official include file.

```assembly
; Load the value for A-4 into R1
LD.w R1, (NOTE_A4_ADDR)

; Write the value to the APU frequency register for Channel 0
ST.w (CH0_FREQ), R1
```

**Note:** The exact addresses and constant names will be finalized in the official toolchain documentation.

