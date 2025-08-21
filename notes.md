# Project Notes & Future Ideas

**Disclaimer:** This document contains thoughts and potential ideas for the Cricket-16 project. The notes here are **not** part of the official hardware specification unless explicitly integrated into the main `HardwareSpec` documents.

---

## **Idea: Memory-Mapped Color Palette RAM (CRAM)**

**Date:** 2025-08-20

**Context:** Currently, the 512-byte CRAM is internal to the PPU and accessed indirectly via I/O ports `F110` (CRAM_ADDR) and `F111` (CRAM_DATA). The main memory map has a large reserved block at `F202-F4FF`.

**Proposal:** Consider moving CRAM into the main memory map, for instance at location `F300-F4FF`.

**Pros:**
*   **Performance:** This would allow for much faster palette updates. The CPU could use `LD`/`ST` instructions or even DMA to modify palette data, which is significantly faster than the current port-based access. This would make full-screen fades and other palette effects easier to implement.
*   **Simplicity:** The programming model becomes simpler, as CRAM is just another piece of memory.
*   **CPU Read Access:** The CPU could directly read palette values.

**Implementation Caveat / Hardware Rule:**
*   To prevent bus contention between the CPU and PPU, a hardware rule would be needed: **The CPU should only be allowed to safely write to CRAM during non-rendering periods (i.e., V-Blank or H-Blank).** Writing during active rendering could cause visual artifacts.

**Conclusion:**
This change was deferred to keep the large reserved memory block available for other potential uses during development. If this space is still free near the project's completion, this idea should be reconsidered as a significant performance and usability improvement. If implemented, the PPU registers `F110` and `F111` would become obsolete.