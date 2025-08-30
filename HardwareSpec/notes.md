# Project Notes & Future Ideas

**Disclaimer:** This document contains thoughts and potential ideas for the Cicada-16 project. The notes here are **not** part of the official hardware specification unless explicitly integrated into the main `HardwareSpec` documents.

## Idea: refactor the RTC chip

### How a Calendar RTC Works

Instead of counting the number of days since an epoch, the hardware would have dedicated registers for each component of the date. The RTC's internal logic would be responsible for handling all the rules of the calendar, including months of different lengths and leap years.

The hardware would automatically handle the entire rollover chain:

When RTC_SEC rolls past 59, RTC_MIN is incremented.

When RTC_MIN rolls past 59, RTC_HOUR is incremented.

When RTC_HOUR rolls past 23, RTC_DAY is incremented.

Here's the key change: When RTC_DAY is incremented, the hardware checks the current RTC_MONTH and RTC_YEAR.

If it's January 31st, the day resets to 1 and the month increments to February.

If it's February 28th on a non-leap year, the day resets to 1 and the month increments to March.

If it's February 28th on a leap year, the day simply increments to 29.

This logic continues for all months. When December 31st rolls over, the day and month reset to 1, and the year is incremented.

### Proposed Hardware and Register Changes

To implement this, we would need to expand the RTC's register space slightly to accommodate the new date components.

New Address Name Description
F018 RTC_SEC Seconds (0-59)
F019 RTC_MIN Minutes (0-59)
F01A RTC_HOUR Hours (0-23)
F01B RTC_DAY Day of the month (1-31)
F01C RTC_MONTH Month of the year (1-12)
F01D RTC_YEAR_L Lower 8 bits of the year (e.g., for 2025, this would be 0xE9)
F01E RTC_YEAR_H Upper 8 bits of the year (e.g., for 2025, this would be 0x07)
F01F RTC_CTL Control register (LATCH, HALT)

Export to Sheets
This expanded layout uses the previously defined RTC registers and extends into the reserved I/O space, which is a perfectly acceptable hardware modification.

### Advantages and Trade-offs

Advantages 👍
Maximum Simplicity for Developers: This is the biggest benefit. To get the current year, a developer simply reads the RTC_YEAR registers. There are no library calls or calculations needed. The date is always directly and instantly available.

Simplified Boot ROM: The first-time setup routine becomes trivial. When the user enters "2025-08-29", the boot ROM simply writes 2025 to the year registers, 8 to the month register, and 29 to the day register.

Trade-offs 🤔
Increased Hardware Complexity: The internal circuitry of the RTC chip is more complex because it must contain the state machine for the full calendar logic, including leap year calculations. This is the primary trade-off.

Uses More I/O Space: This design requires a few more bytes in the memory-mapped I/O space, but this is negligible given the available space in the current memory map.

In summary, if hardware changes are on the table, the Calendar RTC is the superior design. It aligns perfectly with the console's philosophy of making things easier for the developer by handling complex operations at a lower level.

## Possible Sequel Console

The following ideas are things that I think may be appropriate to add to a possible sequel console ("Cicada-16 pro"?).

- Double buffered OAM/CRAM/VRAM
  - Double buffering all of the RAM areas shared between the CPU and the PPU, removes the need to "race the beam", no need to try to fit all visual update logic into the V-blank timing of each frame.
- Additional background layer
- Additional APU channels
- Enhanced WAV channel
  - increase wave sample size from 4 bits to 8, increase wave ram
- Advanced DSP effects
- Hardware multiplication/division

---

© 2025 Connor Nolan. This work is licensed under a
[Creative Commons Attribution-ShareAlike 4.0 International License](http://creativecommons.org/licenses/by-sa/4.0/).
