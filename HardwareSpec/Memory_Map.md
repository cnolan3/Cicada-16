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

- The Boot ROM is mapped to addresses `0x0000-0x3FFF`, temporarily overlaying the game cartridge.
- After the boot sequence finishes its hardware setup and verification, it commands the MMU to unmap the Boot ROM.
- The MMU then maps the game cartridge into the `0x0000-7FFF` range, and the CPU jumps to the game's entry point.

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

## **IO Registers (F000–F03F)**

| Address | Name          | Description                                                       |
| :------ | :------------ | :---------------------------------------------------------------- |
| F000    | **JOYP**      | **Joypad: read buttons, write column select**                     |
| F002    | **SB**        | **Serial Buffer (R/W)**                                           |
| F003    | **SC**        | **Serial Control (R/W)**                                          |
| F004    | **DIV0**      | **32-bit free-running divider (byte 0, LSB)**                     |
| F005    | **DIV1**      | **32-bit free-running divider (byte 1)**                          |
| F006    | **DIV2**      | **32-bit free-running divider (byte 2)**                          |
| F007    | **DIV3**      | **32-bit free-running divider (byte 3, MSB)**                     |
| F008    | **TIMA**      | **8-bit timer counter (IRQ on overflow → IF.TMR)**                |
| F009    | **TMA**       | **8-bit timer modulo (reload value on overflow)**                 |
| F00A    | **TAC**       | **Timer control: bit2=EN, bits1..0=clock sel**                    |
| F00B    | **DMA_SRC_L** | **DMA source address low**                                        |
| F00C    | **DMA_SRC_H** | **DMA source address high**                                       |
| F00D    | **DMA_DST_L** | **DMA destination low**                                           |
| F00E    | **DMA_DST_H** | **DMA destination high**                                          |
| F00F    | **DMA_LEN**   | **DMA length in bytes (0 => special 256/512 default)**            |
| F010    | **DMA_CTL**   | **DMA control: bit0=START, bit1=DIR, bit2=VRAM_ONLY, etc.**       |
| F011    | **MPR_BANK**  | **ROM bank select for 4000-7FFF window**                          |
| F012    | **RAM_BANK**  | **Bank select for banked Cart RAM (if enabled)**                  |
| F013    | **WE_LATCH**  | **Write-enable latch for battery RAM (write key)**                |
| F014    | **VRAM_BANK** | **VRAM Bank Select (0-3 for 9000-AFFF window)**                   |
| F015    | **WRAM_BANK** | **WRAM Bank Select (0-5 for D000-DFFF window -> maps banks 1-6)** |
| F018    | **RTC_SEC**   | **0..59 (latched)**                                               |
| F019    | **RTC_MIN**   | **0..59 (latched)**                                               |
| F01A    | **RTC_HOUR**  | **0..23 (latched)**                                               |
| F01B    | **RTC_DAY_L** | **day counter low (latched)**                                     |
| F01C    | **RTC_DAY_H** | **day counter high (latched)**                                    |
| F01D    | **RTC_CTL**   | **bit0=HALT, bit1=LATCH (1=latch snapshot)**                      |
| F01E    | **RTC_STS**   | **bit0=LATCHED, bit1=BAT_OK (optional)**                          |
| F020    | **IE**        | **Interrupt Enable Register**                                     |
| F021    | **IF**        | **Interrupt Flag Register**                                       |

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

## **Divider Registers (DIV0-DIV3)**

The DIV0-DIV3 registers (F004-F007) together form a single, 32-bit, free-running counter that increments on every system clock cycle (T-cycle). This counter is read-only and cannot be stopped or reset by the game software. Writing to these registers has no effect.

- **DIV0 (F004):** Byte 0 (LSB)
- **DIV1 (F005):** Byte 1
- **DIV2 (F006):** Byte 2
- **DIV3 (F007):** Byte 3 (MSB)

Because the counter is constantly running, it provides a simple, persistent time reference. The programmable timer (TIMA) uses specific bits from this divider as its clock source.

### **Use-Cases**

1.  **Basic Timing:** While the main timer (TIMA) is better for precise, interrupt-driven timing, the DIV registers can be used for simple, low-resolution time measurements. A game could read the value at the start and end of an operation to get a rough estimate of elapsed time.
2.  **Pseudo-Random Number Generation:** The ever-changing value of the DIV registers makes them a common and effective source of entropy for generating pseudo-random numbers. By reading any of the DIV registers at an unpredictable time (e.g., when the player presses a button), the game can get a seed value for a random number algorithm.

## **Programmable Timer (TIMA, TMA, TAC)**

The console provides one 8-bit programmable timer that can be configured to fire an interrupt when it overflows. This system is controlled by three registers: TIMA, TMA, and TAC, now located at F008-F00A.

- **TIMA (F008 - Timer Counter):** This is the main 8-bit counter. It increments at a frequency selected by the TAC register. When TIMA overflows (increments past 255), it is automatically reloaded with the value from TMA and requests a Timer Interrupt by setting bit 2 of the IF register.
- **TMA (F009 - Timer Modulo):** This 8-bit register holds the value that TIMA will be reset to after it overflows. This allows the game to control the starting point of the count, and thus the period of the timer interrupt. For example, if TMA is set to 200, the timer will count from 200 to 255 (56 ticks) before overflowing and firing an interrupt.
- **TAC (F00A - Timer Control):** This register controls the timer's operation. It has been expanded to allow for a much wider range of timer frequencies.

| Bit     | Name        | Type    | Description                                  |
| :------ | :---------- | :------ | :------------------------------------------- |
| 7-6     | -           | R/W     | Unused                                       |
| **5**   | **TMR_EN**  | **R/W** | **Timer Enable (0 = Stop, 1 = Start)**       |
| **4-0** | **CLK_SEL** | **R/W** | **Clock Select (determines TIMA frequency)** |

### **Clock Selection (CLK_SEL)**

The 5-bit value in CLK_SEL selects the clock source for the timer by directly mapping to a bit in the 32-bit DIV counter. The selected CLK_SEL value directly corresponds to the index of the DIV bit that is "tapped into", ie. CLK_SEL = 0 selects bit 0 of the DIV, CLK_SEL = 1 selects bit 1 of the DIV, and so on.

| CLK_SEL | Frequency Calculation (System Clock / Divisor = Result) | Period (Time for one 0→1→0 cycle) |
| :------ | :------------------------------------------------------ | :-------------------------------- |
| 0       | System Clock / 2 = 4,194,304 Hz                         | ~0.238 microseconds               |
| 1       | System Clock / 4 = 2,097,152 Hz                         | ~0.477 microseconds               |
| 2       | System Clock / 8 = 1,048,576 Hz                         | ~0.954 microseconds               |
| 3       | System Clock / 16 = 524,288 Hz                          | ~1.907 microseconds               |
| 4       | System Clock / 32 = 262,144 Hz                          | ~3.815 microseconds               |
| 5       | System Clock / 64 = 131,072 Hz                          | ~7.629 microseconds               |
| 6       | System Clock / 128 = 65,536 Hz                          | ~15.26 microseconds               |
| 7       | System Clock / 256 = 32,768 Hz                          | ~30.52 microseconds               |
| 8       | System Clock / 512 = 16,384 Hz                          | ~61.04 microseconds               |
| 9       | System Clock / 1,024 = 8,192 Hz                         | ~122.1 microseconds               |
| 10      | System Clock / 2,048 = 4,096 Hz                         | ~244.1 microseconds               |
| 11      | System Clock / 4,096 = 2,048 Hz                         | ~488.3 microseconds               |
| 12      | System Clock / 8,192 = 1,024 Hz                         | ~0.977 milliseconds               |
| 13      | System Clock / 16,384 = 512 Hz                          | ~1.953 milliseconds               |
| 14      | System Clock / 32,768 = 256 Hz                          | ~3.906 milliseconds               |
| 15      | System Clock / 65,536 = 128 Hz                          | ~7.813 milliseconds               |
| 16      | System Clock / 131,072 = 64 Hz                          | ~15.63 milliseconds               |
| 17      | System Clock / 262,144 = 32 Hz                          | ~31.25 milliseconds               |
| 18      | System Clock / 524,288 = 16 Hz                          | 62.5 milliseconds                 |
| 19      | System Clock / 1,048,576 = 8 Hz                         | 125 milliseconds                  |
| 20      | System Clock / 2,097,152 = 4 Hz                         | 250 milliseconds                  |
| 21      | System Clock / 4,194,304 = 2 Hz                         | 0.5 seconds                       |
| 22      | System Clock / 8,388,608 = 1 Hz                         | 1 second                          |
| 23      | System Clock / 16,777,216 = 0.5 Hz                      | 2 seconds                         |
| 24      | System Clock / 33,554,432 = 0.25 Hz                     | 4 seconds                         |
| 25      | System Clock / 67,108,864 = 0.125 Hz                    | 8 seconds                         |
| 26      | System Clock / 134,217,728 = 0.0625 Hz                  | 16 seconds                        |
| 27      | System Clock / 268,435,456 = 0.03125 Hz                 | 32 seconds                        |
| 28      | System Clock / 536,870,912 = 0.015625 Hz                | 64 seconds                        |
| 29      | System Clock / 1,073,741,824 = ~0.0078 Hz               | 128 seconds (~2.13 minutes)       |
| 30      | System Clock / 2,147,483,648 = ~0.0039 Hz               | 256 seconds (~4.27 minutes)       |
| 31      | System Clock / 4,294,967,296 = ~0.0020 Hz               | 512 seconds (~8.53 minutes)       |

### **Timer Operation Flow**

1.  **Configure:** Set the desired reload value in **TMA** and the clock frequency in **TAC**.
2.  **Enable:** Set bit 5 of **TAC** to start the timer.
3.  **Counting:** **TIMA** increments at the selected frequency.
4.  **Overflow:** When **TIMA** counts past 255, it overflows.
5.  **Interrupt & Reload:** On overflow, two things happen simultaneously:
    - The Timer Interrupt Flag (bit 2 in **IF**) is set to 1.
    - **TIMA** is reloaded with the value from **TMA**.

### **Use-Cases**

1.  **Scheduled Game Events:** The timer interrupt can be used to drive regularly scheduled game logic, such as updating animations, moving non-player characters, or checking for game state changes at a fixed interval.
2.  **Controlling Music/Sound Tempo:** The timer can provide a steady beat for a software-driven music engine.
3.  **Implementing Time-based Mechanics:** A countdown timer for a level, a temporary power-up that wears off, or a day/night cycle can all be implemented using the timer interrupt.

## **Real-Time Clock (RTC)**

The console includes a battery-backed Real-Time Clock (RTC) that keeps track of time even when the console is powered off. This feature is available on cartridges that include the necessary RTC hardware and a battery. The RTC is controlled by a set of I/O registers from F018 to F01E.

| Address | Name          | Description                           |
| :------ | :------------ | :------------------------------------ |
| F018    | **RTC_SEC**   | Seconds (0-59)                        |
| F019    | **RTC_MIN**   | Minutes (0-59)                        |
| F01A    | **RTC_HOUR**  | Hours (0-23)                          |
| F01B    | **RTC_DAY_L** | Lower 8 bits of a 16-bit day counter. |
| F01C    | **RTC_DAY_H** | Upper 8 bits of a 16-bit day counter. |
| F01D    | **RTC_CTL**   | Control register for the RTC.         |
| F01E    | **RTC_STS**   | Status register for the RTC.          |

### **RTC_CTL (F01D) Bit Assignments**

| Bit   | Name      | Type    | Description                                 |
| :---- | :-------- | :------ | :------------------------------------------ |
| 7-2   | -         | R/W     | Unused                                      |
| **1** | **LATCH** | **R/W** | **Latch RTC Snapshot (1=latch, 0=release)** |
| **0** | **HALT**  | **R/W** | **Halt Clock (1=Stop, 0=Run)**              |

### **RTC_STS (F01E) Bit Assignments**

| Bit   | Name        | Type  | Description                                     |
| :---- | :---------- | :---- | :---------------------------------------------- |
| 7-2   | -           | R     | Unused                                          |
| **1** | **BAT_OK**  | **R** | **Battery Status (1=OK, 0=Fail, optional)**     |
| **0** | **LATCHED** | **R** | **Snapshot Latched (1=Latched, 0=Not Latched)** |

### **Reading the RTC Registers (Latching)**

To prevent reading inconsistent time values (e.g., reading the minutes just as the seconds roll over to 0), the RTC uses a latching mechanism. To safely read the time, the game must perform the following sequence:

1.  Write a `1` to the **LATCH bit (bit 1)** of the **RTC_CTL** register (F01D). This copies the current state of all RTC counter registers into a separate, stable set of latched registers.
2.  Read the time values from the RTC registers (F018-F01C). These reads will access the latched snapshot, not the live counters.
3.  Write a `0` to the **LATCH bit** to release the latch and allow the snapshot to be updated again on the next latch request.

The **LATCHED bit (bit 0)** in the **RTC_STS** register (F01E) will be set to 1 by the hardware after a successful latch, confirming the data is ready to be read.

### **Controlling the RTC**

- **Halting the Clock:** Setting the **HALT bit (bit 0)** in **RTC_CTL** will stop the RTC from incrementing. This is typically only done when the game needs to set the time.
- **Battery Status:** The optional **BAT_OK bit (bit 1)** in **RTC_STS** can be checked to see if the cartridge battery is still good. A value of 1 indicates the battery is okay.

### **Use-Cases**

1.  **Persistent In-Game Time:** The most common use is for games where in-game events are tied to the real-world passage of time, such as animal-crossing-style life simulation games or farming games where crops grow over several real days.
2.  **Time-Locked Events:** A game could unlock special content or characters only on certain dates or at certain times of day.
3.  **Player Progress Tracking:** The RTC can be used to log when a player last played or to track how much real time has been spent in the game.

## **Cartridge Mapper and Bank Switching**

To support games larger than the CPU's addressable ROM space, the console uses a cartridge-based mapper for bank switching. The mapper hardware resides on the game cartridge and is controlled by writing to I/O registers. The primary register for this is **MPR_BANK** at address F011. The value written to this register directly selects which 16 KiB ROM bank is mapped into the `4000-7FFF` address window.

---

© 2025 Connor Nolan. This work is licensed under a
[Creative Commons Attribution-ShareAlike 4.0 International License](http://creativecommons.org/licenses/by-sa/4.0/).
