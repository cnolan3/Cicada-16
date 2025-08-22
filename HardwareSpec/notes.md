# Project Notes & Future Ideas

**Disclaimer:** This document contains thoughts and potential ideas for the Cricket-16 project. The notes here are **not** part of the official hardware specification unless explicitly integrated into the main `HardwareSpec` documents.

---

## idea: increase the size of the ROM1 switchable window

I could increase the size of ROM1 from 8KiB to 16KiB by: 1. reduce the size of the WRAM1 bankable window to 4KiB (increasing the number of banks from 3 banks of 8KiB to 6 banks of 4KiB) 2. removing the dedicated 2KiB of ram space for the system library and instead storing the system library within WRAM0 3. use this newly freed 6KiB of space (4KiB from reducing WRAM1, 2KiB from eliminating dedicated system library ram) along with the existing 2KiB of reserved space at 0x9800 (total of 8KiB free) to increase the size of the ROM1 window from 8KiB to 16KiB (shifting the locations of VRAM, cartridge RAM, WRAM0 and WRAM1) 4. eleminate the half-select system for the ROM1 window

