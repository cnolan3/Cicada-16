# Project Notes & Future Ideas

**Disclaimer:** This document contains thoughts and potential ideas for the Cricket-16 project. The notes here are **not** part of the official hardware specification unless explicitly integrated into the main `HardwareSpec` documents.

# Potential Issues & considerations

These are minor points, mostly requests for clarification that will be critical when you start writing the emulator code.

1. Opcode Map Inconsistencies:
   - 2-Byte Instructions: In CPU_Opcodes.md, instructions like ADD rd, rs are listed as opcode 80 followed by a byte ddd sss. This makes it a 2-byte instruction. However, LD rd, rs is a 1-byte instruction 01 ddd sss. Was it an intentional design choice to make register-to-register arithmetic take more space and cycles than register-to-register loads? A single-byte encoding like 1000 0ddd 0sss might be more efficient. This isn't wrong, but it's a design decision worth confirming.

   - Typo in OR Description: The descriptions for OR rd, rs and OR r, n16 are incomplete copy-pastes ("rd = rd"). This is just a minor documentation error.

2. Undefined CPU Behavior:
   - Unaligned Memory Access: The ISA document states, "16-bit word accesses must be aligned to an even address." This is a great, realistic limitation. However, you need to define what happens if a programmer tries to perform an unaligned 16-bit read or write (e.g., LD.w R0, (0xB001)).
     - Does it trigger a hardware fault/exception?

     - Does it ignore the LSB of the address (reading from 0xB000)?

     - Does it read one byte from 0xB001 and the next from 0xB002? This behavior must be strictly defined for the emulator to be accurate.

3. PPU Timing Specifics:
   - The STAT register defines the PPU modes (H-Blank, V-Blank, OAM Scan, Drawing). However, the precise timing is not specified. You will need to define:
     - How many CPU clock cycles does one full scanline take?

     - How many cycles are spent in "Drawing Pixels" vs. "H-Blank" vs. "OAM Scan"?

     - How many visible scanlines are there (160)? And how many V-Blank scanlines?

   - This timing is absolutely critical for the emulator's main loop and for developers who want to perform "racing the beam" effects (e.g., changing scroll registers mid-frame).

4. Sprite Rendering Priority:
   - The PPU supports up to 64 sprites and has a 16-sprite-per-scanline limit. This is great. But you need to define the priority for overlapping sprites. If two sprites overlap on the same scanline, which one is drawn on top? In many classic systems, it's the sprite with the lower index in OAM (i.e., sprite 0 is drawn over sprite 1). This should be explicitly stated.

5. Memory Access Conflicts:
   - The VRAM_SAFE DMA transfer is well-defined. You should also define the rules for accessing OAM. What happens if the CPU tries to write to OAM (F600-F7FF) while the PPU is in its "OAM Scan" mode? Is access blocked? Does it cause graphical glitches? This is a common source of bugs and "gotchas" in retro development that adds to the charm.
