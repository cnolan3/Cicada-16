## **Boot Process and Memory Mapping**

This document outlines the sequence of events that occurs when the Cicada-16 console is powered on, from the initial execution of internal code to the final handover to the game cartridge.

### **Boot ROM Memory Map Overview**

This table describes the memory map from the CPU's perspective while the internal Boot ROM is active.

| Address Range   | Description                     | Access     | Notes                                                                                                      |
| :-------------- | :------------------------------ | :--------- | :--------------------------------------------------------------------------------------------------------- |
| 0x0000 - 0x3FDF | Internal Boot ROM               | Read-Only  | Mapped only during boot. Overlays Cartridge ROM Bank 0. Inaccessible after handover.                       |
| 0x3FE0 - 0x3FFF | Internal Interrupt Vector Table | Hardwired  | Points to ISRs within the Boot ROM. See `Interrupts.md` for the table layout.                              |
| 0x4000 - 0x7FFF | Cartridge ROM Bank 0            | Read-Only  | Temporarily mapped here during boot to allow the Boot ROM to access the cartridge header for verification. |
| 0x9000 - 0xAFFF | VRAM Window                     | Read/Write | Locked to VRAM Bank 0 during boot. The `VRAM_BANK` register is ignored.                                    |
| 0xB000 - 0xCFFF | Work RAM Bank 0 (WRAM0)         | Read/Write | The WRAM1 window (`D000-DFFF`) is unmapped during boot.                                                    |
| 0xE000 - 0xEFFF | System Library RAM              | Read/Write | Copied here during boot. Made Read-Only after handover.                                                    |
| 0xF000 - 0xF03F | I/O Registers                   | Read/Write |                                                                                                            |
| 0xF040 - 0xF07F | PPU Registers                   | Read/Write |                                                                                                            |
| 0xF080 - 0xF0BF | APU Registers                   | Read/Write |                                                                                                            |
| 0xF200 - 0xF3FF | CRAM                            | Read/Write | CPU writes should be timed to V-Blank/H-Blank.                                                             |
| 0xF400 - 0xF5FF | OAM (Sprite Attribute Memory)   | Read/Write |                                                                                                            |
| 0xF600 - 0xF9FF | DSP Delay Buffer                | Read/Write |                                                                                                            |
| 0xFA00 - 0xFDFF | Wave RAM                        | Read/Write | Available for boot sound data.                                                                             |
| 0xFE00 - 0xFFFF | HRAM                            | Read/Write |                                                                                                            |

###

### **1. The Internal Boot ROM**

The console contains a small, internal **Boot ROM** that is separate from the main memory map accessible by games. This ROM holds the console's initial startup program, analogous to a PC's BIOS. Its purpose is to initialize the console's hardware and safely verify a game cartridge before running it. This internal ROM is completely inaccessible after the boot sequence is complete.

### **2. Boot Mode Interrupts**

During the boot sequence, the CPU operates in a special "Boot Mode" for interrupts. The CPU is hardwired to use a private **Internal Interrupt Vector Table** located at the end of the Boot ROM (`0x3FE0-0x3FFF`). This table points to Interrupt Service Routines (ISRs) also contained within the Boot ROM. This allows the Boot ROM to use V-Blank and H-Blank interrupts to perform animated startup sequences without interfering with the game's own interrupt system. See the **`Interrupts.md`** document for full details on the vector table layout and interrupt process.

### **3. The Boot Sequence**

The code on the Boot ROM executes the following steps in order:

1. **Hardware Initialization**: From power-on, the Boot ROM has full access to WRAM, VRAM, CRAM, OAM, and all I/O registers. It performs basic hardware setup, which includes clearing WRAM and initializing the Stack Pointer (SP) to the top of WRAM0 (e.g., `0xCFFF`).
2. **System Library Copy**: The Boot ROM initiates a DMA transfer to copy the 4 KiB System Library from its internal ROM to the dedicated System Library RAM at `E000-EFFF`. It does this by:
   - Writing the source address in the Boot ROM to the `DMA_SRC` register.
   - Setting the DMA mode to `1` (System Library DMA) in the `DMA_CTL` register (bits 5-3).
   - Setting the `START` bit in `DMA_CTL`.
     After the DMA transfer is complete, it sets the PPU and APU registers to a known-default, disabled state.
3. **Display Boot Animation**: The Boot ROM displays the console logo, enables interrupts (EI), and uses its internal V-Blank ISR to perform a brief startup animation. It may read the **Boot Animation ID** from the cartridge header to select a specific visual effect.
4. **Cartridge Detection & Verification**: While the animation is playing, the Boot ROM checks for a cartridge and verifies its header.
5. **Configure Game Interrupt Mode**: It reads the "Interrupt Mode" flag from the cartridge header and sets an internal hardware latch that determines where the CPU will look for interrupt vectors once the game starts (either the cartridge ROM or WRAM).
6. **Initialize RAM Vectors (If Needed)**: If "Enhanced Mode" is selected, the Boot ROM configures and triggers the DMA controller to copy the 32-byte interrupt vector table from the cartridge (at `0x4060` during boot) to WRAM (at `0xBFE0`).
7. **Finalize and Disable Interrupts**: Once the animation is complete and the cartridge is ready, the Boot ROM executes a DI instruction to disable interrupts, ensuring a clean handover.
8. **Memory Map Handover**: The Boot ROM writes a value of `0x01` to the **`BOOT_CTRL`** register at `F022`. This write-only action commands the memory controller to:
   - **Disable and unmap** the internal Boot ROM and its vector table.
   - **Enable read-only protection** on the System Library RAM (`E000-EFFF`).
   - **Map the game cartridge** to the main memory map, starting at address `0x0000`.
9. **Jump to Game Code**: The very last act of the Boot ROM is to execute a `JMP 0x0080` instruction. This transfers control to the game's official entry point. The game is now responsible for enabling its own interrupts when it is ready.

---

© 2025 Connor Nolan. This work is licensed under a
[Creative Commons Attribution-ShareAlike 4.0 International License](http://creativecommons.org/licenses/by-sa/4.0/).
org/licenses/by-sa/4.0/).
