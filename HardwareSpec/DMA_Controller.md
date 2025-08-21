# **Cricket-16 DMA - Architecture**

This document describes the design and functionality of the Direct Memory Access (DMA) controller for the Cricket-16 console. The DMA unit is responsible for performing fast memory copy operations, which significantly offloads the CPU and is essential for high-performance graphics.

## **1. DMA Registers (F00A–F00F)**

The DMA system is controlled by a set of memory-mapped I/O registers.

| Address   | Name        | Description                                      |
| :-------- | :---------- | :----------------------------------------------- |
| F00A-F00B | `DMA_SRC`   | The 16-bit source address for the transfer.      |
| F00C-F00D | `DMA_DST`   | The 16-bit destination address for the transfer. |
| F00E      | `DMA_LEN`   | The length of the transfer in bytes.             |
| F00F      | `DMA_CTL`   | The DMA Control Register.                        |

## **2. The DMA Transfer Process**

A DMA transfer is a multi-step process initiated by the CPU.

1.  **Set Source:** Write the 16-bit source address to `DMA_SRC` (low byte to `F00A`, high byte to `F00B`).
2.  **Set Destination:** Write the 16-bit destination address to `DMA_DST` (low byte to `F00C`, high byte to `F00D`).
3.  **Set Length:** Write the number of bytes to copy to `DMA_LEN`. (See Section 6 for the special OAM DMA mode).
4.  **Configure Control:** Set the desired transfer properties (e.g., address mode, safe transfer) in the `DMA_CTL` register.
5.  **Start Transfer:** Write a `1` to the `START` bit (bit 0) of the `DMA_CTL` register. The transfer will begin immediately (unless `VRAM_SAFE` is enabled).

## **3. CPU State and Timing**

To prevent conflicts over the system bus, the DMA controller uses a **CPU Halting** model.

-   **CPU Halt:** The moment the `START` bit is written, the CPU is completely frozen and relinquishes control of the memory bus.
-   **Timing:** The DMA transfer takes **4 system clock cycles per byte** copied. For example, copying 256 bytes takes `256 * 4 = 1024` clock cycles.
-   **Completion:** Once the specified number of bytes has been transferred, the DMA controller clears the `START` bit in `DMA_CTL` to `0`, releases the memory bus, and the CPU resumes execution on the next clock cycle.

## **4. Control Register (`DMA_CTL`)**

The `DMA_CTL` register at `F00F` configures the behavior of the transfer.

| Bit | Name        | Function                                                                                                                                                                 |
| :-- | :---------- | :----------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 7-3 | -           | Reserved.                                                                                                                                                                |
| 2   | `VRAM_SAFE` | **VRAM Safe Transfer.** If this bit is `1`, the DMA transfer will not begin until the PPU enters a non-rendering period (H-Blank or V-Blank). This prevents visual artifacts when writing to VRAM or CRAM, but adds a small, variable delay to the start of the transfer. If `0`, the transfer begins immediately. |
| 1   | `ADDR_MODE` | **Address Mode.** If `0`, the source and destination addresses increment after each byte is copied. If `1`, the addresses decrement.                                         |
| 0   | `START`     | **Start/Status Bit.** Writing a `1` to this bit initiates the transfer. This bit will read as `1` while the transfer is in progress and is automatically cleared to `0` by the hardware upon completion. |

## **5. Length Register (`DMA_LEN`) and OAM DMA**

The `DMA_LEN` register at `F00E` typically holds the number of bytes (1-255) for a standard transfer. However, it has a special function when set to zero.

-   **Standard DMA:** If `DMA_LEN` is set to a non-zero value (e.g., `N`), the DMA controller will copy `N` bytes.
-   **OAM DMA:** If `DMA_LEN` is set to `0`, this triggers a special, high-speed **OAM DMA**. This is the primary way to update sprite data. In this mode:
    -   The DMA controller will automatically copy **512 bytes** from the `DMA_SRC` address.
    -   The destination is fixed to the start of OAM memory (`F600`). The `DMA_DST` register is ignored.
