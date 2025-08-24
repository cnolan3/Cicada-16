## **Cricket-16 Cartridge ROM Layout**

This document specifies the internal memory layout of a Cricket-16 game cartridge. It details the mandatory header, the structure of the game data, and the implementation of the hybrid interrupt vector table system.

### **Cartridge Memory Overview**

| Address Range   | Size      | Description                             |
| :-------------- | :-------- | :-------------------------------------- |
| 0x0000 - 0x00DF | 240 Bytes | Cartridge Header (metadata, mode flags) |
| 0x00E0 - 0x00FF | 32 Bytes  | Interrupt Vector Table Template         |
| 0x0100          | -         | Game Code Entry Point                   |
| 0x0101 - 0x3FFF | ~16 KiB   | Remainder of Fixed ROM Bank 0           |

### **1. Cartridge Header (0x0000 - 0x00EF)**

Every Cricket-16 cartridge must begin with a header. The console's internal boot ROM reads this header on startup to verify the cartridge's integrity and configure the hardware.

| Address Range | Size | Field Name              | Description                                                                                                                       |
| :------------ | :--- | :---------------------- | :-------------------------------------------------------------------------------------------------------------------------------- |
| 0x0000-0x0003 | 4B   | **Boot Animation ID**   | A 4-byte ASCII identifier (e.g., "WAVE") that the boot ROM can use to apply a custom visual effect to the console's startup logo. |
| 0x0004-0x0023 | 32B  | **Game Title**          | A null-terminated ASCII string for the game's title.                                                                              |
| 0x0024-0x0027 | 4B   | **Manufacturer Code**   | A 4-byte ASCII identifier for the developer or publisher.                                                                         |
| 0x0028        | 1B   | **Game Version**        | A single byte representing the game's version number (e.g., 0x00 for v1.0).                                                       |
| 0x0029        | 1B   | **Mapper Type**         | An enum specifying the memory bank controller (mapper) hardware on the cartridge. 0x00 = ROM only, 0x01 = Standard Mapper, etc.   |
| 0x002A        | 1B   | **ROM Size**            | An enum indicating the total size of the physical ROM chip (e.g., 0x00=32KiB, 0x01=64KiB).                                        |
| 0x002B        | 1B   | **RAM Size**            | An enum indicating the size of battery-backed RAM on the cartridge. 0x00 = No RAM.                                                |
| 0x002C        | 1B   | **Feature Flags**       | A bitfield for hardware features. Bit 0: Has Battery. Bit 7: Interrupt Mode (0=Standard, 1=Enhanced).                             |
| 0x002D        | 1B   | **Header Checksum**     | An 8-bit checksum of bytes 0x0000 to 0x002C. Used by the boot ROM to verify header integrity.                                     |
| 0x002E-0x002F | 2B   | **Global ROM Checksum** | A 16-bit checksum of the entire cartridge ROM. Can be used for a full integrity check.                                            |
| 0x0030-0x00EF | 192B | **Reserved**            | Reserved for future expansion. Must be filled with 0x00.                                                                          |

### **2. Game Code and Data (0x0100 onwards)**

The rest of the cartridge ROM is dedicated to the game's program code, graphics data, sound data, and other assets.

- **Entry Point (0x0100)**: After the boot sequence, the CPU begins executing game code starting at address 0x0100.
- **ROM Bank 0 (0x0100 - 0x3FFF)**: This area contains the rest of the fixed, non-switchable portion of the game's code. It typically holds the main game loop and critical subroutines that need to be accessible at all times.
- **Switchable ROM Banks (Mapped to 0x4000 - 0x7FFF)**: The remainder of the physical ROM chip contains the switchable banks. The game can map these banks into the CPU's address space to access additional code and data, such as level maps, enemy sprites, and music.

### **3. Interrupt Vector Table**

The 32-byte block from `0x00E0` to `0x00FF` is reserved for the Interrupt Vector Table. This table contains the starting addresses for the game's interrupt service routines.

The Cricket-16 supports two different modes for handling interrupts ("Standard" and "Enhanced"), which control whether this table is used directly from ROM or copied to RAM for dynamic modification. The desired mode is selected via a flag in the cartridge header (Bit 7 of byte `0x002C`).

For a complete explanation of the interrupt system, vector table layout, and handling modes, see the **`Interrupts.md`** document.
