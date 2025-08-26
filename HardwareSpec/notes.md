# Project Notes & Future Ideas

**Disclaimer:** This document contains thoughts and potential ideas for the Cicada-16 project. The notes here are **not** part of the official hardware specification unless explicitly integrated into the main `HardwareSpec` documents.

## Missing pieces and next steps

You're in a fantastic position. Here's what I'd focus on next:

Finalize the Core Specs: Address the "Considerations" above. Make firm decisions on unaligned access, PPU timings, and memory access rules. A dedicated "CPU & PPU Timing" document would be invaluable.

Define the System Library: The System_Library.md is currently a placeholder. This is a huge opportunity to shape the development experience. What functions will you provide?

Math Routines: Fast 16x16 multiplication, 32/16 division, or fixed-point math functions.

Memory Routines: Optimized block copy (memcpy) or block set (memset) that might be faster than a programmer's own loop for small transfers.

Decompression: A simple decompression routine (like RLE) would be a fantastic feature, allowing devs to pack more graphics into their ROMs.

Plan the Toolchain: How will developers create games for the Cicada-16?

Assembler: You'll need to define the syntax for an assembler that can translate your mnemonics into the opcodes you've designed.

Asset Formats: How will you import graphics? You'll need tools to convert standard image files (like PNG) into your 4bpp planar tile format. The same goes for sound and music.

Start the Emulator Core: With the specs finalized, you can begin writing the heart of the emulator.

Start with the CPU instruction decoder and execution loop.

Implement the memory map and basic I/O register reads/writes.

Create a simple "stub" PPU that can at least render a basic tilemap to a window to test your CPU and memory code.

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
