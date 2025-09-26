# Cicada-16

**A fantasy console for crafting retro games with modern ambition.**

Cicada-16 is an open-source journey back to the golden age of 16-bit game development. It's a complete, from-scratch platform designed to replicate the experience of creating games for consoles of the 90s and early 2000s. Our goal is to blend the creative focus of classic hardware limitations with a toolset powerful enough to bring your most ambitious retro-inspired ideas to life.

Whether you're a seasoned developer nostalgic for the past or a newcomer eager to learn the fundamentals of how games are made, Cicada-16 provides a fun, educational, and capable environment to explore, create, and play.

## Core Concepts

The Cicada-16 is built from the ground up with a custom architecture and a complete software stack.

*   **Custom 16-bit CPU**: A unique CISC instruction set that is easy to learn but provides powerful capabilities for game logic.
*   **Memory Banking**: A classic technique to allow games to expand beyond the 16-bit address space, enabling larger and more complex worlds.
*   **Dedicated Graphics & Audio Hardware**: Detailed specifications for a Picture Processing Unit (PPU) and Audio Processing Unit (APU) to handle sprites, tilemaps, and sound generation.
*   **Complete Toolchain**: Everything you need to get started, including an official assembler and detailed documentation.

## Folder Structure

This repository contains all the components that make up the Cicada-16 ecosystem. Here's a look at how it's organized:

*   `Assembler/`
    *   Home of **casm**, the official assembler for the Cicada-16. Written from the ground up in Rust, this is the tool that translates your assembly language source code into bytecode the console can execute.

*   `Assets/`
    *   Contains official console assets, such as the data for the boot-up logo, default system fonts, and other resources that define the look and feel of the Cicada-16 platform.

*   `BootROM/`
    *   Holds the source code and compiled binary for the console's Boot ROM. This is the very first program that runs on startup, responsible for initializing hardware and loading a game cartridge.

*   `ProgrammingDocs/`
    *   Your go-to reference for writing Cicada-16 assembly! Here you'll find detailed guides on the instruction set, CPU registers, addressing modes, assembler directives, and example programs.

*   `HardwareSpec/`
    *   The blueprints of the machine. These documents contain the detailed technical specifications for the CPU, Picture Processing Unit (PPU), Audio Processing Unit (APU), memory map, and other core hardware components.

---

*note: (AUG-27, 2025) License modification, decided on final license structure outlined in [LICENSE.md](./LICENSE.md) in the root of this repository*

