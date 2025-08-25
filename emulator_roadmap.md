# Cricket-16 Emulator Development Roadmap

## Phase 1: The Core Foundation

This phase focuses on creating a program that can successfully execute machine code instructions and manage memory, without worrying about graphics, sound, or input.

**1. Choose a Language and Set Up the Project:**
*   **Language:** Decide on a language. C++ or Rust are common for emulators due to their performance. C# or Go could also work.
*   **Project Structure:** Create a basic directory structure. A `src/` for your source code, a `roms/` for test programs, and a `build/` or `bin/` directory for the compiled emulator.

**2. Implement the CPU Core:**
*   **Goal:** Create a CPU object or module that can execute instructions.
*   **Reference:** `HardwareSpec/CPU_ISA.md` and `HardwareSpec/CPU_Opcodes.md`.
*   **Steps:**
    *   Define the CPU's internal state: registers (A, B, X, Y, etc.), Program Counter (PC), and Stack Pointer (SP).
    *   Implement the main execution loop: `Fetch -> Decode -> Execute`.
    *   Start by implementing a few of the simplest opcodes first. Don't try to do them all at once. Good candidates to start with are `NOP`, `LD` (load), `ST` (store), and basic arithmetic like `ADD`.

**3. Implement the Memory Map and Bus:**
*   **Goal:** Create a way for the CPU to read from and write to memory.
*   **Reference:** `HardwareSpec/Memory_Map.md`.
*   **Steps:**
    *   Create an array (or a more complex object) to represent the console's 64KB address space.
    *   Write `read_byte(address)` and `write_byte(address, value)` functions.
    *   This "bus" will initially just interact with the RAM array, but will later be expanded to route memory access to the PPU, APU, and cartridge ROM based on the address.

**4. Develop a Cartridge Loader:**
*   **Goal:** Load a game file into the emulator's memory.
*   **Reference:** `HardwareSpec/Cartridge_ROM.md`.
*   **Steps:**
    *   Write a function that reads a binary file (your "ROM") from your computer's hard drive.
    *   Load the contents of that file into the appropriate memory region of your emulator, as defined by the memory map (e.g., starting at `0x8000`).

## Phase 2: First Light - Running Code

This phase is about verifying that the core components from Phase 1 are working together correctly.

**5. Create a "Hello, World" Test ROM:**
*   **Goal:** Create a very simple program in Cricket-16 assembly to test the CPU.
*   **Steps:**
    *   You will need a basic assembler. You can either write a very simple one or, for the first test, **manually assemble the bytes by hand**.
    *   The program should be trivial. For example:
        1.  `LD A, #$42` (Load the value 66 into register A).
        2.  `ST A, $0200` (Store the value from register A into RAM address `0x0200`).
        3.  `HLT` (Halt the CPU).
*   This test ROM will be the first file you load with your cartridge loader.

**6. Debug and Verify:**
*   **Goal:** Prove that your test ROM ran correctly.
*   **Steps:**
    *   Run the emulator with your test ROM.
    *   After the emulator halts, add code to inspect the state of the system.
    *   Check if register A contains `0x42`.
    *   Check if memory address `0x0200` contains `0x42`.
    *   If they do, you have successfully emulated a CPU executing code. This is a major milestone.
