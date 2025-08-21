# Project Notes & Future Ideas

**Disclaimer:** This document contains thoughts and potential ideas for the Cricket-16 project. The notes here are **not** part of the official hardware specification unless explicitly integrated into the main `HardwareSpec` documents.

---

## Idea: switch the sizes of ROM0 and ROM1

Since we have freed up space in the cartridge ROM by dedicating a memory space to the System Library and eliminating the need for boilerplate functions to be stored in ROM0, we can reduce the size of ROM0 and increase the size of the ROM1 switchable window. I'm proposing making ROM0 8KiB and ROM1 16KiB and eliminating the half-select system for ROM1.

