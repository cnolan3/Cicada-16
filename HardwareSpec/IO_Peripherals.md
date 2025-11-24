# **Cicada-16 I/O Peripherals**

This document provides comprehensive documentation for all I/O peripherals and their associated memory-mapped registers in the Cicada-16 system. The I/O register block is located at addresses F000-F03F in the memory map.

## **IO Registers (F000–F03F)**

| Address | Name           | Description                                                          |
| :------ | :------------- | :------------------------------------------------------------------- |
| F000    | **SB**         | **Serial Buffer (R/W)**                                              |
| F001    | **SC**         | **Serial Control (R/W)**                                             |
| F002    | **DIV0**       | **32-bit free-running divider (byte 0, LSB)**                        |
| F003    | **DIV1**       | **32-bit free-running divider (byte 1)**                             |
| F004    | **DIV2**       | **32-bit free-running divider (byte 2)**                             |
| F005    | **DIV3**       | **32-bit free-running divider (byte 3, MSB)**                        |
| F006    | **JOYP**       | **Joypad: read buttons, write column select**                        |
| F007    | **TIMA**       | **8-bit timer counter (IRQ on overflow → IF.TMR)**                   |
| F008    | **TMA**        | **8-bit timer modulo (reload value on overflow)**                    |
| F009    | **TAC**        | **Timer control: bit2=EN, bits1..0=clock sel**                       |
| F00A    | **DMA_SRC_L**  | **DMA source address low**                                           |
| F00B    | **DMA_SRC_H**  | **DMA source address high**                                          |
| F00C    | **DMA_DST_L**  | **DMA destination low**                                              |
| F00D    | **DMA_DST_H**  | **DMA destination high**                                             |
| F00E    | **DMA_LEN_L**  | **DMA length/parameter low byte**                                    |
| F00F    | **DMA_LEN_H**  | **DMA length/parameter high byte**                                   |
| F010    | **DMA_CTL**    | **DMA control: bits5-3=MODE, bit2=VRAM_SAFE, bit1=ADDR_MODE, bit0=START** |
| F011    | **MPR_BANK**   | **ROM bank select for 4000-7FFF window**                             |
| F012    | **RAM_BANK**   | **Bank select for banked Cart RAM (if enabled)**                     |
| F013    | **WE_LATCH**   | Write-enable latch for save RAM (write key)                          |
| F014    | **VRAM_BANK**  | **VRAM Bank Select (0-3 for 9000-AFFF window)**                      |
| F015    | **WRAM_BANK**  | **WRAM Bank Select (0-5 for D000-DFFF window -> maps banks 1-6)**    |
| F018    | **RTC_SEC**    | **Seconds (0-59)**                                                   |
| F019    | **RTC_MIN**    | **Minutes (0-59)**                                                   |
| F01A    | **RTC_HOUR**   | **Hours (0-23)**                                                     |
| F01B    | **RTC_DAY**    | **Day of the month (1-31)**                                          |
| F01C    | **RTC_MONTH**  | **Month of the year (1-12)**                                         |
| F01D    | **RTC_YEAR_L** | **Lower 8 bits of the year**                                         |
| F01E    | **RTC_YEAR_H** | **Upper 8 bits of the year**                                         |
| F01F    | **RTC_CTL**    | **Control register (LATCH, HALT)**                                   |
| F020    | **IE**         | **Interrupt Enable Register**                                        |
| F021    | **IF**         | **Interrupt Flag Register**                                          |
| F022    | **BOOT_CTRL**  | **Boot Control: Write 1 to exit boot ROM.**                          |
| F023    | **TIMA1**      | **Timer 1: 8-bit counter (IRQ on overflow → IF.TMR1)**               |
| F024    | **TMA1**       | **Timer 1: 8-bit modulo (reload value on overflow)**                 |
| F025    | **TAC1**       | **Timer 1 control: bit5=EN, bits4..0=clock sel**                     |

## **Divider Registers (DIV0-DIV3)**

The DIV0-DIV3 registers (F004-F007) together form a single, 32-bit, free-running counter that increments on every system clock cycle (T-cycle). This counter is read-only and cannot be stopped or reset by the game software. Writing to these registers has no effect.

- **DIV0 (F002):** Byte 0 (LSB)
- **DIV1 (F003):** Byte 1
- **DIV2 (F004):** Byte 2
- **DIV3 (F005):** Byte 3 (MSB)

Because the counter is constantly running, it provides a simple, persistent time reference. The programmable timer (TIMA) uses specific bits from this divider as its clock source.

### **Use-Cases**

1.  **Basic Timing:** While the main timer (TIMA) is better for precise, interrupt-driven timing, the DIV registers can be used for simple, low-resolution time measurements. A game could read the value at the start and end of an operation to get a rough estimate of elapsed time.
2.  **Pseudo-Random Number Generation:** The ever-changing value of the DIV registers makes them a common and effective source of entropy for generating pseudo-random numbers. By reading any of the DIV registers at an unpredictable time (e.g., when the player presses a button), the game can get a seed value for a random number algorithm.

## **Joypad Register (JOYP)**

The JOYP register at F006 uses a matrix layout to read all 12 buttons (D-Pad, Action, Shoulder, and Utility buttons) using a small number of I/O bits. The game must first write to the register to select which group of four buttons it wants to read, and then read the register to get their state. Buttons that are currently pressed are represented by a 0 bit (active low).

### **JOYP (F006) Bit Assignments**

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

## **Programmable Timers**

The console provides two independent 8-bit programmable timers that can be configured to fire interrupts when they overflow. Each timer is controlled by three registers.

### **Timer 0 (TIMA, TMA, TAC)**

Timer 0 is controlled by three registers located at F007-F009.

- **TIMA (F007 - Timer 0 Counter):** This is the main 8-bit counter. It increments at a frequency selected by the TAC register. When TIMA overflows (increments past 255), it is automatically reloaded with the value from TMA and requests a Timer 0 Interrupt by setting bit 3 of the IF register.
- **TMA (F008 - Timer 0 Modulo):** This 8-bit register holds the value that TIMA will be reset to after it overflows. This allows the game to control the starting point of the count, and thus the period of the timer interrupt. For example, if TMA is set to 200, the timer will count from 200 to 255 (56 ticks) before overflowing and firing an interrupt.
- **TAC (F009 - Timer 0 Control):** This register controls the timer's operation. It has been expanded to allow for a much wider range of timer frequencies.

### **Timer 1 (TIMA1, TMA1, TAC1)**

Timer 1 is controlled by three registers located at F023-F025. It operates identically to Timer 0 but independently.

- **TIMA1 (F023 - Timer 1 Counter):** The 8-bit counter for Timer 1. When it overflows, it is automatically reloaded with the value from TMA1 and requests a Timer 1 Interrupt by setting bit 4 of the IF register.
- **TMA1 (F024 - Timer 1 Modulo):** The 8-bit reload value for Timer 1 after overflow.
- **TAC1 (F025 - Timer 1 Control):** Controls Timer 1's operation with the same configuration options as TAC.

### **Timer Control Register (TAC / TAC1) Bit Assignments**

Both TAC (F009) and TAC1 (F025) share the same bit layout:

| Bit     | Name        | Type    | Description                                  |
| :------ | :---------- | :------ | :------------------------------------------- |
| 7-6     | -           | R/W     | Unused                                       |
| **5**   | **TMR_EN**  | **R/W** | **Timer Enable (0 = Stop, 1 = Start)**       |
| **4-0** | **CLK_SEL** | **R/W** | **Clock Select (determines timer frequency)** |

### **Clock Selection (CLK_SEL)**

The 5-bit value in CLK_SEL selects the clock source for the timer by directly mapping to a bit in the 32-bit DIV counter. The selected CLK_SEL value directly corresponds to the index of the DIV bit that is "tapped into", ie. CLK_SEL = 0 selects bit 0 of the DIV, CLK_SEL = 1 selects bit 1 of the DIV, and so on.

| CLK_SEL | Frequency Calculation (System Clock / Divisor = Result) | Period (Time for one 0→1→0 cycle) |
| :------ | :------------------------------------------------------ | :-------------------------------- |
| 0       | System Clock / 2 = 8,388,608 Hz                         | ~0.119 microseconds               |
| 1       | System Clock / 4 = 4,194,304 Hz                         | ~0.238 microseconds               |
| 2       | System Clock / 8 = 2,097,152 Hz                         | ~0.477 microseconds               |
| 3       | System Clock / 16 = 1,048,576 Hz                        | ~0.954 microseconds               |
| 4       | System Clock / 32 = 524,288 Hz                          | ~1.907 microseconds               |
| 5       | System Clock / 64 = 262,144 Hz                          | ~3.815 microseconds               |
| 6       | System Clock / 128 = 131,072 Hz                         | ~7.629 microseconds               |
| 7       | System Clock / 256 = 65,536 Hz                          | ~15.26 microseconds               |
| 8       | System Clock / 512 = 32,768 Hz                          | ~30.52 microseconds               |
| 9       | System Clock / 1,024 = 16,384 Hz                        | ~61.04 microseconds               |
| 10      | System Clock / 2,048 = 8,192 Hz                         | ~122.1 microseconds               |
| 11      | System Clock / 4,096 = 4,096 Hz                         | ~244.1 microseconds               |
| 12      | System Clock / 8,192 = 2,048 Hz                         | ~488.3 microseconds               |
| 13      | System Clock / 16,384 = 1,024 Hz                        | ~0.977 milliseconds               |
| 14      | System Clock / 32,768 = 512 Hz                          | ~1.953 milliseconds               |
| 15      | System Clock / 65,536 = 256 Hz                          | ~3.906 milliseconds               |
| 16      | System Clock / 131,072 = 128 Hz                         | ~7.813 milliseconds               |
| 17      | System Clock / 262,144 = 64 Hz                          | ~15.63 milliseconds               |
| 18      | System Clock / 524,288 = 32 Hz                          | ~31.25 milliseconds               |
| 19      | System Clock / 1,048,576 = 16 Hz                        | 62.5 milliseconds                 |
| 20      | System Clock / 2,097,152 = 8 Hz                         | 125 milliseconds                  |
| 21      | System Clock / 4,194,304 = 4 Hz                         | 250 milliseconds                  |
| 22      | System Clock / 8,388,608 = 2 Hz                         | 0.5 seconds                       |
| 23      | System Clock / 16,777,216 = 1 Hz                        | 1 second                          |
| 24      | System Clock / 33,554,432 = 0.5 Hz                      | 2 seconds                         |
| 25      | System Clock / 67,108,864 = 0.25 Hz                     | 4 seconds                         |
| 26      | System Clock / 134,217,728 = 0.125 Hz                   | 8 seconds                         |
| 27      | System Clock / 268,435,456 = 0.0625 Hz                  | 16 seconds                        |
| 28      | System Clock / 536,870,912 = 0.03125 Hz                 | 32 seconds                        |
| 29      | System Clock / 1,073,741,824 = 0.015625 Hz              | 64 seconds                        |
| 30      | System Clock / 2,147,483,648 = ~0.0078 Hz               | 128 seconds (~2.13 minutes)       |
| 31      | System Clock / 4,294,967,296 = ~0.0039 Hz               | 256 seconds (~4.27 minutes)       |

### **Timer Operation Flow**

The operation flow is the same for both Timer 0 and Timer 1:

1.  **Configure:** Set the desired reload value in **TMA/TMA1** and the clock frequency in **TAC/TAC1**.
2.  **Enable:** Set bit 5 of **TAC/TAC1** to start the timer.
3.  **Counting:** **TIMA/TIMA1** increments at the selected frequency.
4.  **Overflow:** When **TIMA/TIMA1** counts past 255, it overflows.
5.  **Interrupt & Reload:** On overflow, two things happen simultaneously:
    - The Timer Interrupt Flag (bit 3 for Timer 0, bit 4 for Timer 1 in **IF**) is set to 1.
    - **TIMA/TIMA1** is reloaded with the value from **TMA/TMA1**.

### **Use-Cases**

Having two independent timers provides flexibility for managing multiple timing-critical tasks:

1.  **Scheduled Game Events:** Timer interrupts can be used to drive regularly scheduled game logic, such as updating animations, moving non-player characters, or checking for game state changes at a fixed interval.
2.  **Controlling Music/Sound Tempo:** One timer can provide a steady beat for a software-driven music engine while the other handles game logic timing.
3.  **Implementing Time-based Mechanics:** A countdown timer for a level, a temporary power-up that wears off, or a day/night cycle can all be implemented using timer interrupts.
4.  **Dual-rate Processing:** Use Timer 0 for high-frequency updates (e.g., audio processing) and Timer 1 for lower-frequency updates (e.g., enemy AI), reducing overhead compared to manually dividing a single timer.

## **Real-Time Clock (RTC)**

The console includes a Real-Time Clock (RTC) that keeps track of time even when the console is powered off. This feature is available on cartridges that include the necessary RTC hardware to maintain time. The RTC is controlled by a set of I/O registers from F018 to F01F.

The RTC's internal logic is responsible for handling all the rules of the calendar, including months of different lengths and leap years. When RTC_SEC rolls past 59, RTC_MIN is incremented. When RTC_MIN rolls past 59, RTC_HOUR is incremented. When RTC_HOUR rolls past 23, RTC_DAY is incremented. When RTC_DAY is incremented, the hardware checks the current RTC_MONTH and RTC_YEAR. If it's January 31st, the day resets to 1 and the month increments to February. If it's February 28th on a non-leap year, the day resets to 1 and the month increments to March. If it's February 28th on a leap year, the day simply increments to 29. This logic continues for all months. When December 31st rolls over, the day and month reset to 1, and the year is incremented.

| Address | Name           | Description                   |
| :------ | :------------- | :---------------------------- |
| F018    | **RTC_SEC**    | Seconds (0-59)                |
| F019    | **RTC_MIN**    | Minutes (0-59)                |
| F01A    | **RTC_HOUR**   | Hours (0-23)                  |
| F01B    | **RTC_DAY**    | Day of the month (1-31)       |
| F01C    | **RTC_MONTH**  | Month of the year (1-12)      |
| F01D    | **RTC_YEAR_L** | Lower 8 bits of the year      |
| F01E    | **RTC_YEAR_H** | Upper 8 bits of the year      |
| F01F    | **RTC_CTL**    | Control register for the RTC. |

### **RTC_CTL (F01F) Bit Assignments**

| Bit   | Name      | Type    | Description                                 |
| :---- | :-------- | :------ | :------------------------------------------ |
| 7-2   | -         | R/W     | Unused                                      |
| **1** | **LATCH** | **R/W** | **Latch RTC Snapshot (1=latch, 0=release)** |
| **0** | **HALT**  | **R/W** | **Halt Clock (1=Stop, 0=Run)**              |

### **Reading the RTC Registers (Latching)**

To prevent reading inconsistent time values (e.g., reading the minutes just as the seconds roll over to 0), the RTC uses a latching mechanism. To safely read the time, the game must perform the following sequence:

1.  Write a `1` to the **LATCH bit (bit 1)** of the **RTC_CTL** register (F01F). This copies the current state of all RTC counter registers into a separate, stable set of latched registers.
2.  Read the time values from the RTC registers (F018-F01E). These reads will access the latched snapshot, not the live counters.
3.  Write a `0` to the **LATCH bit** to release the latch and allow the snapshot to be updated again on the next latch request.

### **Controlling the RTC**

- **Halting the Clock:** Setting the **HALT bit (bit 0)** in **RTC_CTL** will stop the RTC from incrementing. This is typically only done when the game needs to set the time.

### **Setting the RTC**

To set the time and date, the following sequence must be performed:

1.  Set the **HALT bit (bit 0)** in the **RTC_CTL** register (F01F) to `1`. This stops the RTC from incrementing.
2.  Write the desired values to the `RTC_SEC`, `RTC_MIN`, `RTC_HOUR`, `RTC_DAY`, `RTC_MONTH`, `RTC_YEAR_L`, and `RTC_YEAR_H` registers.
3.  Set the **HALT bit** back to `0` to start the RTC again.

### **Use-Cases**

1.  **Persistent In-Game Time:** The most common use is for games where in-game events are tied to the real-world passage of time, such as animal-crossing-style life simulation games or farming games where crops grow over several real days.
2.  **Time-Locked Events:** A game could unlock special content or characters only on certain dates or at certain times of day.
3.  **Player Progress Tracking:** The RTC can be used to log when a player last played or to track how much real time has been spent in the game.

## **Cartridge Mapper and Bank Switching**

To support games larger than the CPU's addressable ROM space, the console uses a cartridge-based mapper for bank switching. The mapper hardware resides on the game cartridge and is controlled by writing to I/O registers. The primary register for this is **MPR_BANK** at address F011. The value written to this register directly selects which 16 KiB ROM bank is mapped into the `4000-7FFF` address window.

---

© 2025 Connor Nolan. This work is licensed under a
[Creative Commons Attribution-ShareAlike 4.0 International License](http://creativecommons.org/licenses/by-sa/4.0/).
