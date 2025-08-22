# **Cricket-16 Interrupt System**

## **1. Introduction**

Interrupts are a mechanism that allows the CPU to pause its current task to handle a high-priority hardware event, such as the PPU finishing a frame (V-Blank). This allows for efficient, event-driven programming and is the primary way that games on the Cricket-16 handle graphical updates, sound, and other timed events.

## **2. The Interrupt Process**

The process for triggering and handling an interrupt or CPU fault is as follows:

1.  A specific hardware event occurs (e.g., the V-Blank period begins) OR a CPU fault condition is detected (e.g., unaligned memory access).
2.  For hardware interrupts, the hardware automatically sets the corresponding flag bit in the **`IF` (Interrupt Flag)** register at `F201`. For CPU faults, the fault condition is directly detected by the CPU.
3.  At the end of each instruction cycle, the CPU checks for pending interrupts or faults.
    *   **For Hardware Interrupts:** An interrupt will be serviced if three conditions are met:
        a. The master interrupt switch is enabled (which is done by the `EI` instruction).
        b. The specific interrupt's bit is set to `1` in the **`IE` (Interrupt Enable)** register at `F200`.
        c. The specific interrupt's bit is set to `1` in the **`IF` (Interrupt Flag)** register.
    *   **For CPU Faults:** Faults are generally non-maskable and will be serviced immediately, regardless of the master interrupt switch or `IE`/`IF` registers.
4.  If conditions are met, the CPU performs the following actions:
    a. Disables further interrupts (master interrupt switch is cleared).
    b. Pushes the current 16-bit Program Counter (PC) to the stack.
    c. Pushes the current 16-bit Flags register (F) to the stack.
    d. Jumps to the address specified for that interrupt or fault in the **Interrupt Vector Table**.
5.  The code at that address, the Interrupt Service Routine (ISR) or Fault Handler, is executed.
6.  Inside the ISR, the programmer is responsible for manually clearing the interrupt flag in the `IF` register by writing a `1` to its bit. This prevents the interrupt from immediately re-triggering after the ISR is complete. (This step is not applicable for CPU Faults, as they are not tied to `IF` bits).
7.  The ISR/Fault Handler must end with the `RETI` (Return from Interrupt) instruction, which pops the original F register, then the PC from the stack, and re-enables the master interrupt switch.

## **3. Interrupt Sources**

The Cricket-16 has five hardware interrupt sources, controlled via the `IE` and `IF` registers.

| Bit | Name        | Description                                       |
| :-- | :---------- | :------------------------------------------------ |
| 0   | **V-Blank** | Occurs when the PPU enters the V-Blank period.    |
| 1   | **H-Blank** | Occurs when the PPU enters the H-Blank period.    |
| 2   | **Timer**   | Occurs when the TIMA timer overflows.             |
| 3   | **Serial**  | Occurs when a serial data transfer is complete.   |
| 4   | **Joypad**  | Occurs when a joypad button is pressed **(on the 1-to-0 transition only)**. |

## **4. Interrupt Vector Table**

The Interrupt Vector Table is a 16-byte block of memory containing the 16-bit addresses for each ISR or Fault Handler. The layout below is defined by its offset from the table's base address. The order also determines priority if multiple interrupts or faults occur simultaneously.

| Vector Address Offset | Interrupt Source | Priority |
| :-------------------- | :--------------- | :------- |
| `+0x0` (`xxF0-xxF1`)  | `RESET`          | Highest  |
| `+0x2` (`xxF2-xxF3`)  | **Bus Error**    | **Fault**|
| `+0x4` (`xxF4-xxF5`)  | `V-Blank`        | 1        |
| `+0x6` (`xxF6-xxF7`)  | `H-Blank`        | 2        |
| `+0x8` (`xxF8-xxF9`)  | `Timer`          | 3        |
| `+0xA` (`xxFA-xxFB`)  | `Serial`         | 4        |
| `+0xC` (`xxFC-xxFD`)  | `Joypad`         | 5        |
| `+0xE` (`xxFE-xxFF`)  | `(Reserved)`     | -        | 

## **5. Vector Table Location Modes**

The Cricket-16 supports two modes for the location of the interrupt vector table, determined by a flag in the cartridge header. This provides a choice between simplicity and advanced functionality.

### **Standard Mode (ROM-Based)**
This is the default and simplest mode.

-   **Cartridge Header Flag**: The "Interrupt Mode" bit (Bit 7 of byte `0x002C`) is set to `0`.
-   **CPU Behavior**: The CPU is hardwired to look for the interrupt vector table at a fixed location within the cartridge ROM: **`0x00F0 - 0x00FF`**.
-   **Implementation**: The game developers place a static list of 16-bit addresses at this location in their ROM file. The interrupt handlers are fixed for the lifetime of the game.

### **Enhanced Mode (RAM-Based)**
This mode enables advanced programming techniques by allowing the game to modify its interrupt handlers on the fly.

-   **Cartridge Header Flag**: The "Interrupt Mode" bit is set to `1`.
-   **Console Boot Behavior**: The boot ROM reads this flag and performs two actions:
    1.  It sets a hardware latch that re-routes the CPU's interrupt vector lookups to a fixed location in Work RAM Bank 0 (WRAM0): **`0xC000 - 0xC00F`**.
    2.  It uses the DMA controller to automatically copy the 16-byte vector table from the cartridge ROM (`0x00F0`) to WRAM (`0xC000`) as a default starting point.
-   **Flexibility**: Because the interrupt table is in RAM, the game can overwrite any of these vector addresses at any time to point to different handler routines, allowing for dynamic, state-based interrupt handling.