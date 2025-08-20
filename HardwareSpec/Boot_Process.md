## **Boot Process and Memory Mapping**

This document outlines the sequence of events that occurs when the Cricket-16 console is powered on, from the initial execution of internal code to the final handover to the game cartridge.

### **Boot ROM Memory Map Overview**

| Address Range   | Description                     | Access     | Notes                                                         |
| :-------------- | :------------------------------ | :--------- | :------------------------------------------------------------ |
| 0x0000 - 0x0FFF | Internal Boot ROM               | Read-Only  | Mapped only during boot. Becomes inaccessible after handover. |
| 0x0FF0 - 0x0FFF | Internal Interrupt Vector Table | Hardwired  | Points to ISRs within the Boot ROM for boot animations.       |
| 0x8000          | Cartridge Header Window         | Read-Only  | A temporary, hardwired access port to the cartridge.          |
| 0xA000 - 0xDFFF | Work RAM (WRAM)                 | Read/Write | Available at its final address from power-on.                 |
| 0xF000 - 0xFFFF | I/O, OAM, HRAM                  | Read/Write | Available at final addresses from power-on.                   |

###

### **1. The Internal Boot ROM**

The console contains a small, internal **Boot ROM** that is separate from the main memory map accessible by games. This ROM holds the console's initial startup program, analogous to a PC's BIOS. Its purpose is to initialize the console's hardware and safely verify a game cartridge before running it. This internal ROM is completely inaccessible after the boot sequence is complete.

### **2. Boot Mode Interrupts**

During the boot sequence, the CPU operates in a special "Boot Mode" for interrupts. The CPU is hardwired to use a private **Internal Interrupt Vector Table** located at the end of the Boot ROM (e.g., 0x0FF0). This table points to Interrupt Service Routines (ISRs) also contained within the Boot ROM. This allows the Boot ROM to use V-Blank and H-Blank interrupts to perform animated startup sequences without interfering with the game's own interrupt system.

### **3. The Boot Sequence**

The code on the Boot ROM executes the following steps in order:

1. **Hardware Initialization**: The Boot ROM performs basic hardware setup. This includes clearing WRAM, initializing the Stack Pointer (SP) to the top of WRAM (e.g., 0xDFFF), and setting PPU and APU registers to a known-default, disabled state.
2. **Display Boot Animation**: The Boot ROM displays the console logo, enables interrupts (EI), and uses its internal V-Blank ISR to perform a brief startup animation. It may read the **Boot Animation ID** from the cartridge header to select a specific visual effect.
3. **Cartridge Detection & Verification**: While the animation is playing, the Boot ROM checks for a cartridge and verifies its header via the temporary "Cartridge Window".
4. **Configure Game Interrupt Mode**: It reads the "Interrupt Mode" flag from the cartridge header and sets an internal hardware latch that determines where the CPU will look for interrupt vectors once the game starts (either the cartridge ROM or WRAM).
5. **Initialize RAM Vectors (If Needed)**: If "Enhanced Mode" is selected, the Boot ROM configures and triggers the DMA controller to copy the 16-byte interrupt vector table from the cartridge (at 0x00F0) to WRAM (at 0xC000).
6. **Finalize and Disable Interrupts**: Once the animation is complete and the cartridge is ready, the Boot ROM executes a DI instruction to disable interrupts, ensuring a clean handover.
7. **Memory Map Handover**: The Boot ROM writes to a special I/O register that commands the memory controller to:
   - **Disable and unmap** the internal Boot ROM and its vector table.
   - **Map the game cartridge** to the main memory map, starting at address 0x0000.
8. **Jump to Game Code**: The very last act of the Boot ROM is to execute a JMP 0x0100 instruction. This transfers control to the game's official entry point. The game is now responsible for enabling its own interrupts when it is ready.
