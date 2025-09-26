# Introduction to the Cicada-16 Hardware Specification

Welcome to the technical heart of the Cicada-16! This collection of documents serves as the official hardware specification for the console. It provides a detailed, low-level blueprint of every component, from the CPU's instruction set to the memory layout and peripheral hardware.

These documents are intended for emulator developers, tool creators, and any programmer who wants a deep understanding of how the Cicada-16 works under the hood.

---

## Document Overview

Below is a guide to the contents of each document within this directory.

### **Core Architecture**

- **`CPU_ISA.md`**: The **Instruction Set Architecture**. This is the primary reference for the Cicada-16's custom processor. It details the programming model, CPU registers, flags, and provides a full description of every assembly instruction.

- **`CPU_Opcodes.md`**: This document is the companion to the ISA. It provides the complete opcode map, showing exactly how each instruction and its operands are encoded into machine-readable bytes.

- **`Memory_Map.md`**: Details the 16-bit address space of the console. It outlines the location of ROM, VRAM, WRAM, I/O registers, and other memory-mapped hardware components.

### **Sub-Processors & Controllers**

- **`PPU_Architecture.md`**: Describes the **Picture Processing Unit**. This document explains how graphics are rendered, covering the color system, background layers, sprites, tilemaps, and the palette and object attribute memories (CRAM & OAM).

- **`APU_Architecture.md`**: Describes the **Audio Processing Unit**. This document details the sound generation hardware, including the four sound channels (Pulse, Wave, Noise), ADSR envelopes, and the simple DSP for echo effects.

- **`DMA_Controller.md`**: Explains the **Direct Memory Access** controller. This hardware is used for performing high-speed memory copy operations, which is critical for graphics performance.

### **System Processes**

- **`Interrupts.md`**: Covers the console's interrupt system. It explains the different interrupt sources (V-Blank, H-Blank, timers), the interrupt handling process, and the layout of the vector table.

- **`Boot_Process.md`**: Outlines the sequence of events that occurs when the console is powered on, from the initial execution of the internal Boot ROM to the final handover of control to a game cartridge.

- **`Serial_Communication.md`**: Details the hardware and protocol for the serial communication port, which is used to link two consoles together for multiplayer gameplay.

### **Software & Game Format**

- **`Cartridge_ROM.md`**: Defines the required memory layout for a physical game cartridge, including the mandatory header information that the Boot ROM uses for verification.

- **`System_Library.md`**: Describes the collection of built-in functions and data tables available to all games. These routines, stored in a dedicated RAM area, provide optimized code for common tasks like decompression, multiplication, and interacting with hardware.

### **Miscellaneous**

- **`notes.md`**: A collection of informal thoughts, design notes, and ideas for potential future versions of the console. **Note:** The content in this file is not part of the official specification.

- **`LICENSE.txt`**: The full text of the Creative Commons Attribution-ShareAlike 4.0 International License under which these hardware specification documents are released.

---

© 2025 Connor Nolan. This work is licensed under a
[Creative Commons Attribution-ShareAlike 4.0 International License](http://creativecommons.org/licenses/by-sa/4.0/).
