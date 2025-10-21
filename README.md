# Cicada-16

**A fantasy console for crafting retro games with modern ambition.**

Cicada-16 is an open-source journey back to the golden age of 16-bit game development. It's a complete, from-scratch **specification for a virtual machine** designed to replicate the experience of creating games for consoles of the 90s and early 2000s. Our goal is to blend the creative focus of classic hardware limitations with a toolset powerful enough to bring your most ambitious retro-inspired ideas to life.

Whether you're a seasoned developer nostalgic for the past or a newcomer eager to learn the fundamentals of how games are made, the Cicada-16 specification provides a fun, educational, and capable framework to explore, create, and play.

## What is a Fantasy Console?

A fantasy console is like a video game console that doesn't physically exist. It's a set of carefully chosen limitations (like screen resolution, color palette, and memory size) and a standardized toolset designed to simulate the experience of developing for a real retro machine. The goal is to provide a stable, focused platform that encourages creativity by forcing developers to work within its unique constraints.

## Core Concepts

The Cicada-16 is built from the ground up with a custom architecture and a complete software stack.

- **Custom 16-bit CPU**: A unique CISC instruction set that is easy to learn but provides powerful capabilities for game logic.
- **Memory Banking**: A classic technique to allow games to expand beyond the 16-bit address space, enabling larger and more complex worlds.
- **Dedicated Graphics & Audio Hardware**: Detailed specifications for a Picture Processing Unit (PPU) and Audio Processing Unit (APU) to handle sprites, tilemaps, and sound generation.
- **Complete Toolchain**: Everything you need to get started, including an official assembler and detailed documentation.

## The Semikit Emulator: Coming Soon!

The Cicada-16 is a set of blueprints for a machine. To bring these blueprints to life, we are developing **Semikit**, the official emulator for the Cicada-16 platform. Semikit will accurately simulate the Cicada-16's custom hardware, allowing you to run, test, and play games developed with the `casm` assembler.

Stay tuned for its release!

## Folder Structure

This repository contains all the components that make up the Cicada-16 ecosystem. Here's a look at how it's organized:

- `Assembler/`

  - Home of **casm**, the official assembler for the Cicada-16. Written from the ground up in Rust, this is the tool that translates your assembly language source code into bytecode the console can execute.

- `Assets/`

  - Contains official console assets, such as the data for the boot-up logo, default system fonts, and other resources that define the look and feel of the Cicada-16 platform.

- `BootROM/`

  - Holds the source code and compiled binary for the console's Boot ROM. This is the very first program that runs on startup, responsible for initializing hardware and loading a game cartridge.

- `ProgrammingDocs/`

  - Your go-to reference for writing Cicada-16 assembly! Here you'll find detailed guides on the instruction set, CPU registers, addressing modes, assembler directives, and example programs.

- `HardwareSpec/`
  - The blueprints of the machine. These documents contain the detailed technical specifications for the CPU, Picture Processing Unit (PPU), Audio Processing Unit (APU), memory map, and other core hardware components.

## Versioning

The Cicada-16 project uses a **hybrid versioning system**:

- **The Hardware Specification version is the primary version** for the repository and serves as the authoritative version for compatibility.
- Each subdirectory (`Assembler/`, `ProgrammingDocs/`, `BootROM/`, etc.) maintains its own independent version number.
- **Git tags are prefixed** to indicate which component they version: `HardwareSpec_v0.3.10`, `cicasm_v0.1.10`, `ProgrammingDocs_v1.2.0`, etc.
- Each component directory contains a compatibility document (typically named `COMPATIBILITY.md` or included in its README) that relates its version to the Hardware Specification versions it supports.

For example:

- Git tag: `HardwareSpec_v0.4.0` → Hardware Specification v0.4.0
- Git tag: `cicasm_v1.3.0` → Assembler v1.3.0 (compatible with Hardware Spec v0.3.0 - v0.4.0)
- Git tag: `ProgrammingDocs_v2.1.0` → Programming Docs v2.1.0 (for Hardware Spec v0.4.0+)

**The Hardware Specification version is the authoritative version for compatibility.** When developing games or tools, always reference which Hardware Spec version you are targeting.

---

_note: (AUG-27, 2025) License modification, decided on final license structure outlined in [LICENSE.md](./LICENSE.md) in the root of this repository_
