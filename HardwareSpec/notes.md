# Project Notes & Future Ideas

**Disclaimer:** This document contains thoughts and potential ideas for the Cricket-16 project. The notes here are **not** part of the official hardware specification unless explicitly integrated into the main `HardwareSpec` documents.

---

## **Idea: Increase Boot ROM Size**

**Date:** 2025-08-20

**Context:** The current `Boot_Process.md` specifies a 4 KiB internal Boot ROM mapped from `0x0000` to `0x0FFF` at startup.

**Proposal:** Consider increasing the Boot ROM size to 16 KiB, making it occupy the full `0x0000-0x3FFF` address range during the boot sequence. This would completely overlay the cartridge's fixed ROM Bank 0 until the handover is complete.

**Pros:**
*   Provides significantly more space for more complex and elaborate boot animations.
*   Could also allow for built-in system libraries, fonts, or hardware diagnostic tools to be included in the console's firmware.

**Cons/Considerations:**
*   No major hardware changes are needed, as the memory mapping mechanism that swaps the Boot ROM for the cartridge ROM already exists. The change is primarily just the size of the internal ROM chip.

**Conclusion:**
This idea is being noted down. The user is unsure if the extra space will be strictly necessary but wants to keep the possibility in mind for implementing desired boot logo animations.

---

## **Idea: PPU Memory Access for Boot ROM**

**Date:** 2025-08-20

**Context:** The `Boot_Process.md` document details the CPU and memory state during boot, but it's not explicit about the PPU's memory accessibility.

**Proposal:** To implement a boot logo animation, the Boot ROM will require write access to certain PPU and APU memory regions.

**Requirements:**
*   **VRAM Access:** This is essential for the Boot ROM to load tile graphics and tilemaps for the logo.
*   **CRAM Access:** This is essential for the Boot ROM to set the colors for the logo.
*   **APU Register Access:** This is required to play a boot sound or jingle.
*   **OAM Access:** This would only be necessary if the boot animation involves sprites. A decision has not yet been made on whether to use sprites for the boot logo.

**Conclusion:**
This is a note to ensure that when the boot process is finalized, the specification explicitly states that VRAM, CRAM, and the APU registers (and potentially OAM) are mapped and accessible to the CPU from power-on, alongside WRAM and other I/O registers. The Boot ROM would then be responsible for initializing these areas before handing over control to the game cartridge.