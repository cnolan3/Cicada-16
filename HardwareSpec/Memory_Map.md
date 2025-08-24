# **Cricket-16 Memory Map**

## **Memory Map**

| Start Addr | End Addr | Size       | Description                                                |
| :--------- | :------- | :--------- | :--------------------------------------------------------- |
| 0000       | 3FFF     | **16 KiB** | **ROM Bank 0 (fixed)**                                     |
| 4000       | 7FFF     | **16 KiB** | **ROM Bank N (switchable)**                                |
| 8000       | 9FFF     | **8 KiB**  | **VRAM Window (banked, 1 of 4 banks, 32 KiB total)**       |
| A000       | AFFF     | **4 KiB**  | **Cartridge RAM Window (banked)**                          |
| B000       | CFFF     | **8 KiB**  | **Work RAM Bank 0 (WRAM0, fixed, 32 KiB total)**           |
| D000       | DFFF     | **4 KiB**  | **Work RAM Window (WRAM1, banked, 1 of 6 switchable banks)** |
| E000       | E7FF     | **2 KiB**  | **Wave RAM (user wave tables)**                            |
| E800       | EFFF     | **2 KiB**  | **System Library RAM (Read-Only after boot)**              |
| F000       | F0FF     | **256 B**  | **IO Registers (joypad, timers/div, RTC, DMA, mapper)**    |
| F100       | F1FF     | **256 B**  | **PPU Registers (LCDC, STAT, SCX, SCY, LY/LYC, palettes)** |
| F200       | F200     | **1 B**    | **IE (Interrupt Enable)**                                  |
| F201       | F201     | **1 B**    | **IF (Interrupt Flag) (write-1-to-clear bits)**            |
| F202       | F2FF     | **254 B**  | **Reserved**                                               |
| F300       | F4FF     | **512 B**  | **CRAM (color pallete entries)**                           |
| F500       | F5FF     | **256 B**  | **APU Registers (Core, Mixer, DSP)**                       |
| F600       | F7FF     | **512 B**  | **OAM (sprite attribute table)**                           |
| F800       | FBFF     | **1 KiB**  | **DSP Delay Buffer**                                       |
| FC00       | FFFF     | **1 KiB**  | **HRAM (high speed ram)**                                  |

## **MMU Behavior and Rules**

The Memory Management Unit (MMU) is the hardware component responsible for interpreting the CPU's memory accesses and mapping them to the appropriate physical memory device.

### **Bank Switching**

To expand the amount of available RAM beyond the limits of the 16-bit address space, the system uses bank switching for several memory regions. This is controlled by writing to specific I/O registers.

-   **VRAM (`8000-9FFF`):** This 8 KiB window can be mapped to one of four 8 KiB banks within the PPU's 32 KiB of VRAM. The active bank is selected by the **`VRAM_BANK`** register at `F013`.
-   **WRAM (`D000-DFFF`):** This 4 KiB window can be mapped to one of six 4 KiB banks of switchable Work RAM (WRAM1-6). The active bank is selected by the **`WRAM_BANK`** register at `F014`.
-   **Cartridge ROM (`4000-7FFF`):** This 16 KiB window can be mapped to any 16 KiB bank in the cartridge's ROM. The active bank is selected by the **`MPR_BANK`** register at `F010`.
-   **Cartridge RAM (`A000-AFFF`):** This 4 KiB window can be mapped to different banks of RAM on the cartridge, if present. The active bank is selected by the **`RAM_BANK`** register at `F011`.

### **Boot-time Mapping**

At power-on, the MMU starts in a special state to execute the internal Boot ROM.
-   The Boot ROM is mapped to addresses `0x0000-0x3FFF`, temporarily overlaying the game cartridge.
-   After the boot sequence finishes its hardware setup and verification, it commands the MMU to unmap the Boot ROM.
-   The MMU then maps the game cartridge into the `0x0000-7FFF` range, and the CPU jumps to the game's entry point.

### **Memory Protection**

The MMU enforces access rules on certain memory regions.
-   **Read-Only Memory:** The Cartridge ROM (`0000-7FFF`) and the System Library RAM (`E800-EFFF`, after boot) are read-only. Any attempt by the CPU to write to these regions will be blocked by the MMU and will trigger a **Protected Memory Fault**.
-   **PPU Access Restrictions:** The PPU's internal RAM (VRAM, OAM, CRAM) is shared between the CPU and the PPU. While the CPU can access it at any time, writing to it while the PPU is actively drawing (`STAT` mode 3) can lead to visual glitches. Safe access is guaranteed during H-Blank (Mode 0) and V-Blank (Mode 1).

### **Memory Access Alignment**

The CPU requires 16-bit data to be aligned to an even memory address.
-   Any `LD.w` or `ST.w` instruction that attempts to read or write a 16-bit word from an odd address will be blocked by the MMU and will trigger a **Bus Error Fault**.
-   8-bit operations (`LD.b`, `ST.b`) can access any address without issue.

### **HRAM vs. WRAM Access Speed**

Not all RAM is equal in speed.
-   **HRAM (High RAM, `FC00-FFFF`):** This small 1 KiB region is internal to the main processor chip. It can be accessed without any extra wait states, making it the fastest RAM in the system. It is ideal for storing frequently accessed variables, temporary "scratchpad" data, or time-critical interrupt handler code.
-   **WRAM (Work RAM, `B000-DFFF`):** This is a larger pool of general-purpose external RAM. Accessing it incurs a small number of wait states, making it slightly slower than HRAM. The cycle counts listed in the CPU ISA documentation assume WRAM access times. Accesses to HRAM using the same instructions will be faster.

## **IO Registers (F000–F0FF)**

| Address | Name          | Description                                                       |
| :------ | :------------ | :---------------------------------------------------------------- |
| F000    | **JOYP**      | **Joypad: read buttons, write column select**                     |
| F004    | **DIVL**      | **16-bit free-running divider (low)**                             |
| F005    | **DIVH**      | **16-bit free-running divider (high)**                            |
| F006    | **TIMA**      | **8-bit timer counter (IRQ on overflow → IF.TMR)**                |
| F007    | **TMA**       | **8-bit timer modulo (reload value on overflow)**                 |
| F008    | **TAC**       | **Timer control: bit2=EN, bits1..0=clock sel**                    |
| F00A    | **DMA_SRC_L** | **DMA source address low**                                        |
| F00B    | **DMA_SRC_H** | **DMA source address high**                                       |
| F00C    | **DMA_DST_L** | **DMA destination low**                                           |
| F00D    | **DMA_DST_H** | **DMA destination high**                                          |
| F00E    | **DMA_LEN**   | **DMA length in bytes (0 => special 256/512 default)**            |
| F00F    | **DMA_CTL**   | **DMA control: bit0=START, bit1=DIR, bit2=VRAM_ONLY, etc.**       |
| F010    | **MPR_BANK**  | **ROM bank select for 4000-7FFF window**                          |
| F011    | **RAM_BANK**  | **Bank select for banked Cart RAM (if enabled)**                  |
| F012    | **WE_LATCH**  | **Write-enable latch for battery RAM (write key)**                |
| F013    | **VRAM_BANK** | **VRAM Bank Select (0-3 for 8000-9FFF window)**                   |
| F014    | **WRAM_BANK** | **WRAM Bank Select (0-5 for D000-DFFF window -> maps banks 1-6)** |
| F018    | **RTC_SEC**   | **0..59 (latched)**                                               |
| F019    | **RTC_MIN**   | **0..59 (latched)**                                               |
| F01A    | **RTC_HOUR**  | **0..23 (latched)**                                               |
| F01B    | **RTC_DAY_L** | **day counter low (latched)**                                     |
| F01C    | **RTC_DAY_H** | **day counter high (latched)**                                    |
| F01D    | **RTC_CTL**   | **bit0=HALT, bit1=LATCH (1=latch snapshot)**                      |
| F01E    | **RTC_STS**   | **bit0=LATCHED, bit1=BAT_OK (optional)**                          |

## **Joypad Register (JOYP)**

The JOYP register at F000 uses a matrix layout to read all 12 buttons (D-Pad, Action, Shoulder, and Utility buttons) using a small number of I/O bits. The game must first write to the register to select which group of four buttons it wants to read, and then read the register to get their state. Buttons that are currently pressed are represented by a 0 bit (active low).

### **JOYP (F000) Bit Assignments**

| Bit     | Name        | Type  | Description                                                        |
| :------ | :---------- | :---- | :----------------------------------------------------------------- |
| 7-6     | -           | R     | Unused (read as 1)                                                 |
| **5-4** | **GRP_SEL** | **W** | **Selects button group to read (01=D-Pad, 10=Action, 11=Utility)** |
| **3**   | **BTN_3**   | **R** | **Input for Button 3 of the selected group**                       |
| **2**   | **BTN_2**   | **R** | **Input for Button 2 of the selected group**                       |
| **1**   | **BTN_1**   | **R** | **Input for Button 1 of the selected group**                       |
| **0**   | **BTN_0**   | **R** | **Input for Button 0 of the selected group**                       |

### **Button Group Mapping**

The 2-bit value written to GRP_SEL determines which set of physical buttons is mapped to the four readable input bits (BTN_0 to BTN_3).

| GRP_SEL Value | Group Name  | BTN_0 (Bit 0) | BTN_1 (Bit 1) | BTN_2 (Bit 2)      | BTN_3 (Bit 3)     |
| :------------ | :---------- | :------------ | :------------ | :----------------- | :---------------- |
| 01            | **D-Pad**   | Right         | Left          | Up                 | Down              |
| 10            | **Action**  | A             | B             | X                  | Y                 |
| 11            | **Utility** | Start         | Select        | R (Right Shoulder) | L (Left Shoulder) |

### **Reading the Joypad**

1. **Select a Button Group:** Write a value to F000 to set bits 5-4, selecting a group.
   - To read the D-Pad, write `0x10`.
   - To read the Action Buttons (A, B, X, Y), write `0x20`.
   - To read the Utility Buttons (Start, Select, L, R), write `0x30`.
2. **Read the Button State:** Read from F000. The lower 4 bits will reflect the state of the selected buttons. For example, if the Action group was selected and the player is pressing **A** and **X**, reading the register will return a value where bits 0 and 2 are 0.

## **Timer Control Register (TAC)**

The TAC register at F008 controls the operation of the 8-bit timer (TIMA). It allows the game to enable or disable the timer and select its clock frequency. The frequency is derived from the main system clock by tapping into specific bits of the free-running 16-bit DIV register.

### **TAC (F008) Bit Assignments**

| Bit     | Name        | Type    | Description                                  |
| :------ | :---------- | :------ | :------------------------------------------- |
| 7-3     | -           | R/W     | Unused                                       |
| **2**   | **TMR_EN**  | **R/W** | **Timer Enable (0 = Stop, 1 = Start)**       |
| **1-0** | **CLK_SEL** | **R/W** | **Clock Select (determines TIMA frequency)** |

### **Clock Selection (CLK_SEL)**

The lower two bits of TAC select how often the TIMA register increments.

| CLK_SEL Value | Frequency (Taps DIV bit)            |
| :------------ | :---------------------------------- |
| 00            | **System Clock / 1024 (DIV bit 9)** |
| 01            | **System Clock / 16 (DIV bit 3)**   |
| 10            | **System Clock / 64 (DIV bit 5)**   |
| 11            | **System Clock / 256 (DIV bit 7)**  |

## **Interrupt Registers (IE & IF)**

The interrupt system is controlled by two main registers: **IE (Interrupt Enable)** at F200 and **IF (Interrupt Flag)** at F201. For an interrupt to be triggered, the corresponding bit must be set in both the IE and IF registers, and the master interrupt switch must be enabled by the CPU's EI instruction.

### **IE: Interrupt Enable (F200)**

This register determines which types of interrupts are allowed to trigger an interrupt service routine. Writing a 1 to a bit enables that interrupt source.

| Bit   | Name        | Description                                       |
| :---- | :---------- | :------------------------------------------------ |
| **0** | **V-Blank** | **Enable Vertical Blank interrupt**               |
| **1** | **H-Blank** | **Enable Horizontal Blank interrupt (from STAT)** |
| **2** | **Timer**   | **Enable Timer Overflow interrupt**               |
| **3** | **Serial**  | **Enable Serial Transfer Complete interrupt**     |
| **4** | **Joypad**  | **Enable Joypad button press interrupt**          |
| 5-7   | -           | Unused                                            |

### **IF: Interrupt Flag (F201)**

This register indicates that an interrupt-triggering event has occurred. The hardware sets the appropriate bit to 1 when an event happens. The program can then request service by also setting the corresponding bit in the IE register.

- **Clearing Flags:** To clear a flag, the program must write a 1 to that bit's position. This is a common "write-1-to-clear" mechanism.

| Bit   | Name        | Description                                     |
| :---- | :---------- | :---------------------------------------------- |
| **0** | **V-Blank** | **Set when the PPU enters the V-Blank period**  |
| **1** | **H-Blank** | **Set when the PPU enters the H-Blank period**  |
| **2** | **Timer**   | **Set when the TIMA timer overflows**           |
| **3** | **Serial**  | **Set when a serial data transfer is complete** |
| **4** | **Joypad**  | **Set when a joypad button is pressed**         |
| 5-7   | -           | Unused                                          |

## **Cartridge Mapper and Bank Switching**

To support games larger than the CPU's addressable ROM space, the console uses a cartridge-based mapper for bank switching. The mapper hardware resides on the game cartridge and is controlled by writing to I/O registers. The primary register for this is **MPR_BANK** at address F010. The value written to this register directly selects which 16 KiB ROM bank is mapped into the `4000-7FFF` address window.

## **APU Registers (F500-F5FF)**

| Address Range | Name          | Description                                                 |
| :------------ | :------------ | :---------------------------------------------------------- |
| F500-F57F     | **APU Core**  | **Registers for the 4 sound channels (Pulse, Wave, Noise)** |
| F580-F59F     | **APU Mixer** | **Master volume, panning, and mixing controls**             |
| F5A0-F5FF     | **APU DSP**   | **Control registers for the DSP (echo/delay)**              |