## **Boot Process and Memory Mapping**

This document outlines the sequence of events that occurs when the Cricket-16 console is powered on, from the initial execution of internal code to the final handover to the game cartridge.

### **Boot ROM Memory Map Overview**

This table describes the memory map from the CPU's perspective while the internal Boot ROM is active.

| Address Range   | Description                     | Access     | Notes                                                                                |
| :-------------- | :------------------------------ | :--------- | :----------------------------------------------------------------------------------- |
| 0x0000 - 0x3FFF | Internal Boot ROM               | Read-Only  | Mapped only during boot. Overlays Cartridge ROM Bank 0. Inaccessible after handover. |
| 0x3FF0 - 0x3FFF | Internal Interrupt Vector Table | Hardwired  | Points to ISRs within the Boot ROM. See `Interrupts.md` for the table layout.      |
| 0x6000 - 0x7FFF | VRAM Window                     | Read/Write | Locked to VRAM Bank 0 during boot. The `VRAM_BANK` register is ignored.              |
| 0xA000 - 0xBFFF | Work RAM (WRAM0)                | Read/Write | The WRAM1 window (`D000-DFFF`) is unmapped during boot.                              |
| 0xE000 - 0xEFFF | Wave RAM                        | Read/Write | Available for boot sound data.                                                       |
| 0xF000 - 0xF0FF | I/O Registers                   | Read/Write |                                                                                      |
| 0xF100 - 0xF1FF | PPU Registers                   | Read/Write |                                                                                      |
| 0xF200 - 0xF201 | IE & IF Registers               | Read/Write |                                                                                      |
| 0xF300 - 0xF4FF | CRAM                            | Read/Write | CPU writes should be timed to V-Blank/H-Blank.                                       |
| 0xF500 - 0xF5FF | APU Registers                   | Read/Write |                                                                                      |
| 0xF600 - 0xF7FF | OAM (Sprite Attribute Memory)   | Read/Write |                                                                                      |
| 0xF800 - 0xFBFF | DSP Delay Buffer                | Read/Write |                                                                                      |
| 0xFC00 - 0xFFFF | (Unmapped)                      | -          | HRAM is not available during the boot sequence.                                      |

###

### **1. The Internal Boot ROM**

The console contains a small, internal **Boot ROM** that is separate from the main memory map accessible by games. This ROM holds the console's initial startup program, analogous to a PC's BIOS. Its purpose is to initialize the console's hardware and safely verify a game cartridge before running it. This internal ROM is completely inaccessible after the boot sequence is complete.

### **2. Boot Mode Interrupts**

During the boot sequence, the CPU operates in a special "Boot Mode" for interrupts. The CPU is hardwired to use a private **Internal Interrupt Vector Table** located at the end of the Boot ROM (`0x3FF0-0x3FFF`). This table points to Interrupt Service Routines (ISRs) also contained within the Boot ROM. This allows the Boot ROM to use V-Blank and H-Blank interrupts to perform animated startup sequences without interfering with the game's own interrupt system. See the **`Interrupts.md`** document for full details on the vector table layout and interrupt process.

### **3. The Boot Sequence**

The code on the Boot ROM executes the following steps in order:

1. **Hardware Initialization**: From power-on, the Boot ROM has full access to WRAM, VRAM, CRAM, OAM, and all I/O registers. It performs basic hardware setup, which includes clearing WRAM and initializing the Stack Pointer (SP) to the top of WRAM0 (e.g., 0xCFFF).
2. **System Library Copy**: The Boot ROM uses the DMA controller to copy the first 2 KiB of its own address space (which contains the System Library functions) to the start of WRAM0 at `0xB000 - 0xB7FF`. After this, it sets the PPU and APU registers to a known-default, disabled state.
3. **Display Boot Animation**: The Boot ROM displays the console logo, enables interrupts (EI), and uses its internal V-Blank ISR to perform a brief startup animation. It may read the **Boot Animation ID** from the cartridge header to select a specific visual effect.
4. **Cartridge Detection & Verification**: While the animation is playing, the Boot ROM checks for a cartridge and verifies its header.
5. **Configure Game Interrupt Mode**: It reads the "Interrupt Mode" flag from the cartridge header and sets an internal hardware latch that determines where the CPU will look for interrupt vectors once the game starts (either the cartridge ROM or WRAM).
6. **Initialize RAM Vectors (If Needed)**: If "Enhanced Mode" is selected, the Boot ROM configures and triggers the DMA controller to copy the 16-byte interrupt vector table from the cartridge (at 0x00F0) to WRAM (at 0xC000).
7. **Finalize and Disable Interrupts**: Once the animation is complete and the cartridge is ready, the Boot ROM executes a DI instruction to disable interrupts, ensuring a clean handover.
8. **Memory Map Handover**: The Boot ROM writes to a special I/O register that commands the memory controller to:
   - **Disable and unmap** the internal Boot ROM and its vector table.
   - **Map the game cartridge** to the main memory map, starting at address 0x0000.
9. **Jump to Game Code**: The very last act of the Boot ROM is to execute a JMP 0x0100 instruction. This transfers control to the game's official entry point. The game is now responsible for enabling its own interrupts when it is ready.
