# **Cricket-16 Memory Map**

## **Memory Map**

| Start Addr | End Addr | Size       | Description                                                |
| :--------- | :------- | :--------- | :--------------------------------------------------------- |
| 0000       | 3FFF     | **16 KiB** | **ROM Bank 0 (fixed)**                                     |
| 4000       | 5FFF     | **8 KiB**  | **ROM Bank N (switchable, half-select)**                   |
| 6000       | 7FFF     | **8 KiB**  | **VRAM (PPU-visible)**                                     |
| 8000       | 9FFF     | **8 KiB**  | **Cartridge RAM (banked, battery-backed)**                 |
| A000       | BFFF     | **8 KiB**  | **Work RAM (WRAM0)**                                       |
| C000       | DFFF     | **8 KiB**  | **Work RAM (WRAM1)**                                       |
| E000       | EFFF     | **4 KiB**  | **Wave RAM (user wave tables)**                            |
| F000       | F0FF     | **256 B**  | **IO Registers (joypad, timers/div, RTC, DMA, mapper)**    |
| F100       | F1FF     | **256 B**  | **PPU Registers (LCDC, STAT, SCX, SCY, LY/LYC, palettes)** |
| F200       | F200     | **1 B**    | **IE (Interrupt Enable)**                                  |
| F201       | F201     | **1 B**    | **IF (Interrupt Flag) (write-1-to-clear bits)**            |
| F202       | F4FF     | **765 B**  | **Reserved**                                               |
| F500       | F5FF     | **256 B**  | **APU Registers (Core, Mixer, DSP)**                       |
| F600       | F7FF     | **512 B**  | **OAM (sprite attribute table)**                           |
| F800       | FBFF     | **1 KiB**  | **DSP Delay Buffer**                                       |
| FC00       | FFFF     | **1 KiB**  | **HRAM (high speed ram)**                                  |

## **IO Registers (F000–F0FF)**

| Address | Name          | Description                                                 |
| :------ | :------------ | :---------------------------------------------------------- |
| F000    | **JOYP**      | **Joypad: read buttons, write column select**               |
| F004    | **DIVL**      | **16-bit free-running divider (low)**                       |
| F005    | **DIVH**      | **16-bit free-running divider (high)**                      |
| F006    | **TIMA**      | **8-bit timer counter (IRQ on overflow → IF.TMR)**          |
| F007    | **TMA**       | **8-bit timer modulo (reload value on overflow)**           |
| F008    | **TAC**       | **Timer control: bit2=EN, bits1..0=clock sel**              |
| F00A    | **DMA_SRC_L** | **DMA source address low**                                  |
| F00B    | **DMA_SRC_H** | **DMA source address high**                                 |
| F00C    | **DMA_DST_L** | **DMA destination low**                                     |
| F00D    | **DMA_DST_H** | **DMA destination high**                                    |
| F00E    | **DMA_LEN**   | **DMA length in bytes (0 => special 256/512 default)**      |
| F00F    | **DMA_CTL**   | **DMA control: bit0=START, bit1=DIR, bit2=VRAM_ONLY, etc.** |
| F010    | **MPR_BANK**  | **ROM bank select (half-select within 16 KiB banks)**       |
| F011    | **RAM_BANK**  | **Bank select for banked Cart RAM (if enabled)**            |
| F012    | **WE_LATCH**  | **Write-enable latch for battery RAM (write key)**          |
| F018    | **RTC_SEC**   | **0..59 (latched)**                                         |
| F019    | **RTC_MIN**   | **0..59 (latched)**                                         |
| F01A    | **RTC_HOUR**  | **0..23 (latched)**                                         |
| F01B    | **RTC_DAY_L** | **day counter low (latched)**                               |
| F01C    | **RTC_DAY_H** | **day counter high (latched)**                              |
| F01D    | **RTC_CTL**   | **bit0=HALT, bit1=LATCH (1=latch snapshot)**                |
| F01E    | **RTC_STS**   | **bit0=LATCHED, bit1=BAT_OK (optional)**                    |

## **Joypad Register (JOYP)**

The JOYP register at F000 uses a matrix layout to read all 8 buttons (D-Pad + Action buttons) using a small number of bits. The game must first write to the register to select which group of buttons it wants to read, and then read the register to get their state. Buttons that are currently pressed are represented by a 0 bit (active low).

### **JOYP (F000) Bit Assignments**

| Bit   | Name         | Type  | Description                                                       |
| :---- | :----------- | :---- | :---------------------------------------------------------------- |
| 7-6   | -            | R     | Unused (read as 1)                                                |
| **5** | **ACTN_SEL** | **W** | **Write 0 to select Action Buttons (A, B, Select, Start)**        |
| **4** | **DPAD_SEL** | **W** | **Write 0 to select Directional Buttons (Right, Left, Up, Down)** |
| **3** | **BTN_3**    | **R** | **Input for Down / Start**                                        |
| **2** | **BTN_2**    | **R** | **Input for Up / Select**                                         |
| **1** | **BTN_1**    | **R** | **Input for Left / B**                                            |
| **0** | **BTN_0**    | **R** | **Input for Right / A**                                           |

### **Reading the Joypad**

1. **Select a Button Column:** Write to F000 to select a column.
   - To read the D-Pad, write 0x10 (sets bit 4 to 0).
   - To read the Action Buttons, write 0x20 (sets bit 5 to 0).
2. **Read the Button State:** Read from F000. The lower 4 bits will reflect the state of the selected buttons. For example, if the D-Pad was selected and the player is pressing **Up** and **Left**, reading the register will return a value where bits 2 and 1 are 0.

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

To support games larger than the CPU's addressable ROM space (24 KiB), the console uses a cartridge-based mapper for bank switching. The mapper hardware resides on the game cartridge and is controlled by writing to I/O registers. The primary register for this is **MPR_BANK** at address F010.

### **MPR_BANK (F010) - Half-Select Mechanism**

The switchable ROM window at 4000-5FFF is 8 KiB. To provide finer control, the mapper uses a "half-select" system based on the 8-bit value written to **MPR_BANK**.

- **Bits 1-7 (Upper 7 bits):** These bits select a 16 KiB bank from the physical ROM on the cartridge.
- **Bit 0 (Lowest bit):** This bit selects which 8 KiB half of that 16 KiB bank to map into the 4000-5FFF window.
  - **0:** Selects the first half (offset +0 KiB).
  - **1:** Selects the second half (offset +8 KiB).

## **APU Registers (F500-F5FF)**

| Address Range | Name          | Description                                                 |
| :------------ | :------------ | :---------------------------------------------------------- |
| F500-F57F     | **APU Core**  | **Registers for the 4 sound channels (Pulse, Wave, Noise)** |
| F580-F59F     | **APU Mixer** | **Master volume, panning, and mixing controls**             |
| F5A0-F5FF     | **APU DSP**   | **Control registers for the DSP (echo/delay)**              |
