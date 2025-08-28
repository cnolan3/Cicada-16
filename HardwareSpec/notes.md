# Project Notes & Future Ideas

**Disclaimer:** This document contains thoughts and potential ideas for the Cicada-16 project. The notes here are **not** part of the official hardware specification unless explicitly integrated into the main `HardwareSpec` documents.

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
