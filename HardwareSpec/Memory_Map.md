# **Cicada-16 Memory Map**

## **Memory Map**

| Start Addr | End Addr | Size       | Description                                                  |
| :--------- | :------- | :--------- | :----------------------------------------------------------- |
| 0000       | 3FFF     | **16 KiB** | **ROM Bank 0 (fixed)**                                       |
| 4000       | 7FFF     | **16 KiB** | **ROM Bank N (switchable)**                                  |
| 8000       | 8FFF     | **4 KiB**  | **Cartridge RAM Window (banked)**                            |
| 9000       | AFFF     | **8 KiB**  | **VRAM Window (banked, 1 of 4 banks, 32 KiB total)**         |
| B000       | CFFF     | **8 KiB**  | **Work RAM Bank 0 (WRAM0, fixed, 32 KiB total)**             |
| D000       | DFFF     | **4 KiB**  | **Work RAM Window (WRAM1, banked, 1 of 6 switchable banks)** |
| E000       | EFFF     | **4 KiB**  | **System Library RAM (Read-Only after boot)**                |
| F000       | F03F     | **64 B**   | **IO Registers (joypad, timers/div, RTC, DMA, mapper)**      |
| F040       | F07F     | **64 B**   | **PPU Registers (LCDC, STAT, SCX, SCY, LY/LYC, palettes)**   |
| F080       | F0BF     | **64 B**   | **APU Registers (Core, Mixer, DSP)**                         |
| F0C0       | F1FF     | **320 B**  | **Reserved**                                                 |
| F200       | F3FF     | **512 B**  | **CRAM (color pallete entries)**                             |
| F400       | F5FF     | **512 B**  | **OAM (sprite attribute table)**                             |
| F600       | F9FF     | **1 KiB**  | **DSP Delay Buffer**                                         |
| FA00       | FDFF     | **1 KiB**  | **Wave RAM (user wave tables)**                              |
| FE00       | FFFF     | **512 B**  | **HRAM (high speed ram)**                                    |

## **MMU Behavior and Rules**

The Memory Management Unit (MMU) is the hardware component responsible for interpreting the CPU's memory accesses and mapping them to the appropriate physical memory device.

### **Bank Switching**

To expand the amount of available RAM beyond the limits of the 16-bit address space, the system uses bank switching for several memory regions. This is controlled by writing to specific I/O registers.

- **VRAM (`9000-AFFF`):** This 8 KiB window can be mapped to one of four 8 KiB banks within the PPU's 32 KiB of VRAM. The active bank is selected by the **`VRAM_BANK`** register at `F014`.
- **WRAM (`D000-DFFF`):** This 4 KiB window can be mapped to one of six 4 KiB banks of switchable Work RAM (WRAM1-6). The active bank is selected by the **`WRAM_BANK`** register at `F015`.
- **Cartridge ROM (`4000-7FFF`):** This 16 KiB window can be mapped to any 16 KiB bank in the cartridge's ROM. The active bank is selected by the **`MPR_BANK`** register at `F011`.
- **Cartridge RAM (`8000-8FFF`):** This 4 KiB window can be mapped to different banks of RAM on the cartridge, if present. The active bank is selected by the **`RAM_BANK`** register at `F012`.

### **Boot-time Mapping**

At power-on, the MMU starts in a special state to execute the internal Boot ROM.

- The Boot ROM is mapped to addresses `0x0000-0x3FFF`, temporarily overlaying where the game cartridge's ROM Bank 0 will eventually be.
- To allow the Boot ROM to read the cartridge header for verification, the first 16 KiB of the cartridge ROM (Bank 0) is temporarily mapped to `0x4000-0x7FFF`.
- After the boot sequence finishes, it writes to the `BOOT_CTRL` register, which commands the MMU to perform the final memory map handover:
  - The Boot ROM at `0x0000-0x3FFF` is unmapped.
  - The temporary mapping of Cartridge ROM Bank 0 at `0x4000-0x7FFF` is removed.
  - The game cartridge is mapped into its final configuration: ROM Bank 0 is mapped to `0x0000-0x3FFF` and the switchable ROM bank window is enabled at `0x4000-0x7FFF`.
- The CPU then jumps to the game's entry point at `0x0080`.

### **Memory Protection**

The MMU enforces access rules on certain memory regions.

- **Read-Only Memory:** The Cartridge ROM (`0000-7FFF`) and the System Library RAM (`E000-EFFF`, after boot) are read-only. Any attempt by the CPU to write to these regions will be blocked by the MMU and will trigger a **Protected Memory Fault**.
- **PPU Access Restrictions:** The PPU's internal RAM (VRAM, OAM, CRAM) is shared between the CPU and the PPU. While the CPU can access it at any time, writing to it while the PPU is actively drawing (`STAT` mode 3) can lead to visual glitches. Safe access is guaranteed during H-Blank (Mode 0) and V-Blank (Mode 1).

### **Memory Access Alignment**

The CPU requires 16-bit data to be aligned to an even memory address.

- Any `LD.w` or `ST.w` instruction that attempts to read or write a 16-bit word from an odd address will be blocked by the MMU and will trigger a **Bus Error Fault**.
- 8-bit operations (`LD.b`, `ST.b`) can access any address without issue.

### **HRAM vs. WRAM Access Speed**

Not all RAM is equal in speed.

- **HRAM (High RAM, `FE00-FFFF`):** This small 512 B region is internal to the main processor chip. It can be accessed without any extra wait states, making it the fastest RAM in the system. It is ideal for storing frequently accessed variables, temporary "scratchpad" data, or time-critical interrupt handler code.
- **WRAM (Work RAM, `B000-DFFF`):** This is a larger pool of general-purpose external RAM. Accessing it incurs a small number of wait states, making it slightly slower than HRAM. The cycle counts listed in the CPU ISA documentation assume WRAM access times. Accesses to HRAM using the same instructions will be faster.

## **I/O Peripherals**

The Cicada-16 includes a comprehensive set of memory-mapped I/O registers at addresses F000-F03F for controlling hardware peripherals. For detailed information about all I/O registers and peripheral systems (including the programmable timers, joypad, RTC, divider registers, DMA controller, and cartridge mapper), see the **`IO_Peripherals.md`** document.

---

© 2025 Connor Nolan. This work is licensed under a
[Creative Commons Attribution-ShareAlike 4.0 International License](http://creativecommons.org/licenses/by-sa/4.0/).
