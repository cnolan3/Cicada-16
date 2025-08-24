# **Cricket-16 System Library**

This document describes the functions that are available in the permanent System Library, which is located in its own dedicated, read-only RAM region at **`E800-EFFF`**.

These functions are copied from the internal Boot ROM to the System Library RAM by the boot process and are available for any game to use. They provide standardized, optimized routines for common tasks. After the boot sequence completes, this memory region is made read-only to protect the library's integrity.

_This document will be updated with a full list of available functions, their addresses, and their usage instructions as they are defined._

## **Music and Sound Routines**

### **Note Frequency Table**

To simplify music creation, the System Library provides a pre-calculated lookup table containing the 16-bit `FREQ_REG` values for a range of standard musical notes. This allows developers to play specific pitches without needing to perform the frequency calculation manually.

- **Location:** The table resides at a fixed address within the System Library space.
- **Range:** The table covers 5 octaves, from C2 to B6.
- **Format:** The table is a simple array of 16-bit unsigned integers. Each entry corresponds to a note in the chromatic scale.

**Example Usage (Conceptual):**

A developer could access the table using constants defined in an official include file.

```assembly
; Load the value for A-4 into R1
LD.w R1, (NOTE_A4_ADDR)

; Write the value to the APU frequency register for Channel 0
ST.w (CH0_FREQ), R1
```

**Note:** The exact addresses and constant names will be finalized in the official toolchain documentation.

## **Serial Communication Functions**

To simplify serial communication, the System Library provides functions to handle the low-level boilerplate of exchanging single bytes.

### `serialExchangeByte`

This function, intended to be called by the **Master** console, simultaneously sends one byte and receives one byte.

-   **Input:** A byte to send, typically passed in a general-purpose register (e.g., the low byte of `R0`).
-   **Action:**
    1.  Waits until the serial port is not busy (i.e., the `START` bit in the `SC` register is 0).
    2.  Writes the input byte to the **`SB`** (Serial Buffer) register at `F002`.
    3.  Sets the `START` bit in the **`SC`** (Serial Control) register at `F003` to begin the transfer.
    4.  Waits for the transfer to complete (i.e., for the `START` bit to be cleared by hardware).
-   **Output:** The byte received from the other console during the exchange. This is read from the `SB` register after the transfer and returned, typically in a general-purpose register.

**Note:** This is a blocking function. It will halt CPU execution until the transfer is complete. It should only be called by the Master console (`CLK_SRC = 0`). The Slave console must have its outgoing byte pre-loaded into its `SB` register before this function is called by the Master.

### `serialByteWrite`

This non-blocking function simply writes a byte to the serial buffer. It is intended to be used by the **Slave** console to prepare the next byte for transmission before the Master initiates an exchange.

-   **Input:** A byte to write, typically passed in a general-purpose register (e.g., the low byte of `R0`).
-   **Action:**
    1.  Writes the input byte to the **`SB`** (Serial Buffer) register at `F002`.
-   **Output:** None.

### `serialByteRecv`

This function reads a single byte from the serial buffer. It is designed to be lightweight and is typically called from within the Serial Transfer Complete interrupt service routine, primarily on the **Slave** console.

-   **Input:** None.
-   **Action:**
    1.  Reads the byte from the **`SB`** (Serial Buffer) register at `F002`.
-   **Output:** The received byte, typically returned in a general-purpose register (e.g., the low byte of `R0`).

**Example ISR Usage on Slave Console (Conceptual):**

```assembly
; Serial Transfer Complete ISR (on Slave)
Serial_ISR:
    ; The master has just finished a transfer.
    ; Our outgoing byte was sent, and we have received a byte from the master.
    
    ; Call the library function to get the byte from the buffer.
    CALL serialByteRecv

    ; R0 now holds the received byte.
    ; Process the byte (e.g., store it in a RAM buffer).
    ...

    ; Pre-load the SB register for the *next* transfer using the new function.
    LD.b R1, (next_byte_to_send) ; Load the next byte into a register
    CALL serialByteWrite        ; Call the library function to write it to SB

    RETI ; Return from interrupt
```