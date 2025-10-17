# **Cicada-16 PPU - Architecture**

This document describes the design of the Picture Processing Unit (PPU) for the Cicada-16 fantasy console. The PPU is responsible for rendering all graphics to the screen. The design is inspired by 16-bit era consoles, featuring multiple background layers, sprites, and a palette-based color system.

## **1. Core Components**

- **VRAM (Video RAM):** 32 KiB of dedicated, bank-switched RAM. An 8 KiB window of VRAM is accessible to the CPU at any time at `9000-AFFF` for writing and reading data; the active bank for this window is selected via the `VRAM_BANK` I/O register. For rendering, the PPU has a wider internal address bus and can access the entire 32 KiB of VRAM simultaneously. VRAM holds the tile data (the 8x8 pixel building blocks for graphics) and the tilemaps (the data that arranges tiles into the three graphical layers: BG0, BG1, and Window).
- **OAM (Object Attribute Memory):** 512 bytes of dedicated RAM at F400-F5FF used to store the attributes for all 64 hardware sprites (position, tile index, palette, etc.).

### **1.1. Memory Access Conflicts**

Accessing OAM/VRAM/CRAM (reading or writing) by the CPU is generally safe during V-Blank and H-Blank periods. However, if the CPU attempts to write to OAM/VRAM/CRAM while the PPU is in **Mode 2 (OAM Scan)** or **Mode 3 (Drawing Pixels)**, the write will still occur, but the PPU may read corrupted or inconsistent data for the current frame, leading to **graphical glitches** on the screen. This behavior is intentional and requires developers to synchronize OAM updates with the PPU's rendering cycle.

- **CRAM (Color RAM / Palette RAM):** 512 bytes of RAM located at `F200-F3FF` in the main memory map. It holds the 256 16-bit color palette entries.
- **Screen Resolution:** 240x160 pixels.
- **Refresh Rate:** ~59.739 Hz.

## **2. Color System**

- **Master Palette:** The PPU can display **256** colors on screen at once.
- **Color RAM (CRAM):** The 256-color master palette is stored in CRAM. Each color entry is a 16-bit value, defining the color in RGB555 format (5 bits for Red, 5 for Green, 5 for Blue).
  - RRRRRGGGGGBBBBB
- **Sub-palettes:** The 256 colors in CRAM are divided into 16 sub-palettes of 16 colors each. Background layers and sprites can each be assigned one of these sub-palettes.
- **Color 0:** In all layers except BG0 color 0 of each 16 color sub-palette is treated as transparent, no matter what color the entry actually corresponds to. This allows sprites and background layers to show content behind them. BG0 treats color 0 the same as all other palette colors and renders it as the color that it specifies.

## **3. VRAM Layout and Tilemaps**

The PPU's graphics are built from layers of tiles, which are configured by data structures in VRAM called Tilemaps. The layout of VRAM is highly flexible and controlled by a set of registers.

### **3.1. VRAM Organization**

By hardware rule, the **Tile Graphics Area** is always assumed to begin at address `0x0000` in VRAM. The developer can then place their **Tilemap Areas** anywhere else in VRAM. The PPU supports three independent tilemaps (BG0, BG1, and Window), each with its location and size defined by PPU registers (`BG_TMB` and `WIN_TMB`), giving the developer full control over how VRAM is partitioned.

### **3.2. Tilemap Data Format**

A tilemap is a 2D grid of 16-bit entries. Each entry tells the PPU which tile to draw at a position, and how to draw it.
Notice that the PAL data field is only 3 bits, so it is only able to index 8 out of the total 16 sub-palettes stored in the master palette in CRAM. Background tiles are limited to using the first 8 sub-palettes (0-7), while sprites have access to all 16.

| Bit(s) | Name             | Description                         |
| :----- | :--------------- | :---------------------------------- |
| 15     | **P (Priority)** | Priority vs. Sprites. (1 bit)       |
| 14     | **V (V-Flip)**   | Vertical Flip. (1 bit)              |
| 13     | **H (H-Flip)**   | Horizontal Flip. (1 bit)            |
| 10-12  | **PAL**          | **Palette Select (0-7).** (3 bits)  |
| **9**  | **`INDEX_9`**    | **The 10th bit of the tile index.** |
| **8**  | **`INDEX_8`**    | **The 9th bit of the tile index.**  |
| 7-0    | **`INDEX_7_0`**  | The lower 8 bits of the tile index. |

### **3.3. Tile Graphics Data Format (4bpp Planar)**

Each 8x8 tile requires 32 bytes of storage in VRAM. These 32 bytes are organized into four **bit planes**. Each plane holds one bit of the 4-bit color index for all 64 pixels in the tile.

The 32 bytes of a single tile are laid out in VRAM as follows:

- **Bytes 0-7:** **Bit Plane 0** (The least significant bit of the color index for all 64 pixels)
- **Bytes 8-15:** **Bit Plane 1**
- **Bytes 16-23:** **Bit Plane 2**
- **Bytes 24-31:** **Bit Plane 3** (The most significant bit of the color index)

Within each 8-byte plane, each byte represents one row of 8 pixels. For example, in Bit Plane 0:

- Byte 0 holds the LSB for the 8 pixels in Row 0 of the tile.
- Byte 1 holds the LSB for the 8 pixels in Row 1 of the tile.
- ...and so on up to Byte 7 for Row 7.

#### How the PPU Reconstructs a Pixel

To find the color of a single pixel, the PPU gathers one bit from each of the four planes and combines them into a 4-bit number.

For example, to get the color for the pixel at coordinate (2, 5) (the 3rd pixel of the 6th row) within a tile:

1.  It reads the 3rd bit from the 6th byte of **Bit Plane 0**.
2.  It reads the 3rd bit from the 6th byte of **Bit Plane 1**.
3.  It reads the 3rd bit from the 6th byte of **Bit Plane 2**.
4.  It reads the 3rd bit from the 6th byte of **Bit Plane 3**.

These four bits are combined to form the final 4-bit color index (a value from 0-15), which is then used to look up the actual color in the sub-palette assigned to that tile.

## **4. Background Layers**

The PPU can render **two** independent scrollable background layers, referred to as **BG0** and **BG1**, plus a third non-scrolling **Window** layer for static UI elements.

### **4.1. BG0 and BG1 (Scrollable Backgrounds)**

- **Tilemaps:** Each background layer is constructed from a tilemap stored in VRAM. A tilemap is a 2D array of 16-bit entries that specify which tile to draw at each position on the grid.
  - Tilemap sizes can be configured (e.g., 32x32 tiles, 64x32 tiles) to create large scrolling worlds.
- **Tiles:** The building blocks of backgrounds. Each tile is an 8x8 pixel graphic. The color data for tiles is stored in VRAM in a 4-bit-per-pixel (4bpp) planar format, where each pixel's value is an index into one of the 16-color sub-palettes.
- **Scrolling:** Each background layer has its own horizontal and vertical scroll registers (SCX0, SCY0 for BG0; SCX1, SCY1 for BG1), allowing them to be moved independently for effects like parallax scrolling.

### **4.2. Window Layer**

The Window is a third independent tilemap layer used for static UI elements like HUDs, dialogue boxes, and borders. Unlike BG0 and BG1, the Window is not affected by scrolling and remains at a fixed screen position.

- **Tilemap Storage:** The Window has its own dedicated tilemap in VRAM. The tilemap location is specified by the `WIN_TMB` register (similar to `BG_TMB`), occupying one 2 KiB VRAM slot. The tilemap has a fixed size of 32×32 tiles (256×256 pixels).
- **Positioning:** The Window's position on screen is controlled by the `WINX` and `WINY` registers (default: 0, 0 at the top-left corner). These coordinates specify where the top-left pixel of the Window's tilemap appears on screen. Any portion extending beyond the 240×160 screen boundaries is clipped.
- **Tilemap Format:** Uses the same 16-bit tilemap entry format as BG0/BG1, with Priority, V-Flip, H-Flip flags, palette select (0-7, limited to first 8 sub-palettes), and 10-bit tile index.
- **Tile Graphics:** Shares the same tile graphics pool as BG0 and BG1. All tiles are stored starting at VRAM address 0x0000, and Window tilemap entries reference these shared tiles via tile index.
- **Transparency:** Color #0 of any sub-palette is transparent, allowing BG0 and BG1 to show through.

## **5. Sprites (Objects)**

The PPU can render up to **64** sprites on screen at once.

- **Attributes:** Each sprite's data is stored in OAM as an 8-byte entry:

  - **Byte 0:** Y-Position (vertical screen coordinate)
  - **Byte 1:** X-Position (horizontal screen coordinate)
  - **Byte 2:** **Size & Shape attribute** - Defines the sprite's dimensions (e.g., 8x8, 16x16, 8x16, 16x8).
  - **Byte 3:** Tile Index (which 8x8 tile from VRAM to use)
  - **Byte 4:** Attribute Flags, defined below.
  - **Bytes 5-7:** Reserved for future use (e.g., rotation/scaling data).

  **OAM Byte 2 (Size & Shape Attribute):**

  This byte controls the dimensions of the sprite. The lower four bits are divided into two 2-bit fields that work together to define the final size. The upper four bits are reserved for future use.

| Bit(s) | Name       | Description                                                                        |
| :----- | :--------- | :--------------------------------------------------------------------------------- |
| 7-4    | (Reserved) | Unused.                                                                            |
| 3-2    | **SHAPE**  | `00`: Square, `01`: Horizontal Rectangle, `10`: Vertical Rectangle, `11`: Reserved |
| 1-0    | **SIZE**   | Selects the size from a shape-dependent table.                                     |

**Sprite Dimensions based on SHAPE and SIZE:**

| SHAPE             | SIZE=`00` | SIZE=`01` | SIZE=`10` | SIZE=`11` |
| :---------------- | :-------- | :-------- | :-------- | :-------- |
| **00 (Square)**   | 8x8       | 16x16     | 32x32     | 64x64     |
| **01 (H-Rect)**   | 16x8      | 32x8      | 32x16     | 64x32     |
| **10 (V-Rect)**   | 8x16      | 8x32      | 16x32     | 32x64     |
| **11 (Reserved)** | Invalid   | Invalid   | Invalid   | Invalid   |

For sprites larger than 8x8, the tile index in Byte 3 points to the top-left 8x8 tile of the sprite. The PPU then automatically fetches the required adjacent tiles from VRAM to construct the full sprite. For example, a 16x16 sprite will be composed of a 2x2 block of four tiles starting at the specified tile index.

**OAM Byte 4 (Sprite Attribute Flags):**

| Bit(s) | Name         | Description                                                                          |
| :----- | :----------- | :----------------------------------------------------------------------------------- |
| 7      | **Priority** | `1`=Sprite is in front of high-priority background tiles. `0`=Sprite is behind them. |
| 6      | **V-Flip**   | `1`=Flip sprite vertically.                                                          |
| 5      | **H-Flip**   | `1`=Flip sprite horizontally.                                                        |
| 4      | (Reserved)   | Unused.                                                                              |
| 3-0    | **PAL**      | **Palette Select (0-15).** A 4-bit value selecting any of the 16 sub-palettes.       |

- **Scanline Limit:** The PPU can render a maximum of **16** sprites per horizontal scanline. If more than 16 sprites are on a line, the additional ones will not be drawn.

### **5.1. Sprite Priority**

When multiple sprites overlap on the same scanline, their rendering priority is determined by their index in OAM. Sprites with a **lower OAM index** (e.g., sprite 0) are drawn on top of sprites with a **higher OAM index** (e.g., sprite 1). This provides a deterministic and predictable rendering order for overlapping sprites.

## **6. Layer Priority and Transparency**

The PPU renders the final image by drawing the graphical layers in a specific order from back to front with defined transparency rules.

### **6.1. Rendering Order**

Layers are drawn in the following order:

1.  **BG0 (Backdrop Layer):** BG0 is the rearmost background layer. It is always fully opaque. Even if a tile uses color #0 of a sub-palette, the actual color value stored in CRAM for that entry will be drawn to the screen. This ensures the game always has a solid backdrop color.

2.  **BG1 (Overlay Layer):** BG1 is drawn on top of BG0. Color #0 of its assigned sub-palette is treated as **transparent**, allowing the pixels of the BG0 layer to show through.

3.  **Window Layer:** The Window is drawn on top of BG0 and BG1. Color #0 of any tile drawn as part of the Window layer is treated as **transparent**, allowing background layers to show through transparent pixels.

4.  **Sprites (Objects):** Sprites are the topmost layer, rendered over all background and window layers. Color #0 of a sprite's assigned sub-palette is always **transparent**. The sprite's priority flag (in its OAM data) can cause it to be rendered behind high-priority background tiles, but it is always drawn on top of low-priority tiles.

## **7. PPU Registers (F040-F07F)**

These registers, mapped to the CPU's address space, control the PPU's operation.

| Address   | Name    | Description                                                                                                                                                                |
| :-------- | :------ | :------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| F040      | LCDC    | **LCD Control:** Master switch for the PPU. Contains bits to enable/disable the screen, background layers, sprites, and the window.                                        |
| F041      | STAT    | **LCD Status:** Contains flags indicating the PPU's current mode (H-Blank, V-Blank, OAM Scan) and can be configured to trigger interrupts on specific events.              |
| F042      | SCY0    | Background 0 - Vertical Scroll                                                                                                                                             |
| F043      | SCX0    | Background 0 - Horizontal Scroll                                                                                                                                           |
| F044      | SCY1    | Background 1 - Vertical Scroll                                                                                                                                             |
| F045      | SCX1    | Background 1 - Horizontal Scroll                                                                                                                                           |
| F046      | WINY    | **Window Y-Position:** The top edge of the Window layer.                                                                                                                   |
| F047      | WINX    | **Window X-Position:** The left edge of the Window layer.                                                                                                                  |
| F048      | LY      | **LCD Y-Coordinate:** Indicates the current vertical scanline being drawn (Read-only). Ranges from 0 to 215.                                                               |
| F049      | LYC     | **LY Compare:** The PPU compares LY with this value. If they match, a flag is set in the STAT register, which can trigger an interrupt. Useful for scanline-based effects. |
| F04A      | BG_MODE | **Background Mode:** Configures the size (dimensions) of the BG0 and BG1 tilemaps.                                                                                         |
| F04B      | BG_TMB  | **Background Tilemap Base:** Sets the 2KiB-aligned starting slot in VRAM for BG0 (bits 3-0) and BG1 (bits 7-4).                                                            |
| F04C      | WIN_TMB | **Window Tilemap Base:** Sets the 2KiB-aligned starting slot in VRAM for the Window tilemap (bits 3-0). Bits 7-4 are reserved.                                            |
| F04D-F07F | ---     | **Reserved**                                                                                                                                                               |

### **7.1. Accessing Color RAM (CRAM)**

Since CRAM is mapped directly to the CPU's address space (`F200-F3FF`), there are no registers for indirect access. This allows for very fast reads and writes. However, to prevent visual artifacts caused by modifying palette data while the PPU is actively drawing, all CPU writes to this memory region should be performed **only during non-rendering periods (V-Blank or H-Blank)**. Reading from CRAM is safe at any time.

### **7.2. Configuring Tilemap Base Addresses**

The `BG_TMB` register at `F04B` and `WIN_TMB` register at `F04C` provide an efficient way to set the starting addresses for all three tilemaps. The 32 KiB of VRAM is divided into 16 slots of 2 KiB each. These registers use a 4-bit value to specify which slot each tilemap begins in.

**BG_TMB Register (F04B):**
- **BG0:** The lower 4 bits (bits 3-0) select the starting slot (0-15) for the BG0 tilemap.
- **BG1:** The upper 4 bits (bits 7-4) select the starting slot (0-15) for the BG1 tilemap.

**WIN_TMB Register (F04C):**
- **Window:** The lower 4 bits (bits 3-0) select the starting slot (0-15) for the Window tilemap.
- **Bits 7-4:** Reserved, should be set to 0.

The PPU calculates the final base address using the formula: `base_address = slot_id × 2048`. For example:
- If `BG_TMB` = `0x42`: BG0's tilemap starts at slot 2 (address `0x1000`), and BG1's tilemap starts at slot 4 (address `0x2000`).
- If `WIN_TMB` = `0x06`: Window's tilemap starts at slot 6 (address `0x3000`).

## **8. LCDC Register (F040)**

This 8-bit register is the primary control for the PPU.

| Bit   | Name       | Function                                                                     |
| :---- | :--------- | :--------------------------------------------------------------------------- |
| **7** | PPU_ENABLE | 1: PPU is on and drawing to the screen. 0: PPU is off (screen is blank).     |
| **6** | SPR_ENABLE | 1: Sprites are enabled and will be drawn. 0: Sprites are disabled.           |
| **5** | BG1_ENABLE | 1: Background Layer 1 is enabled. 0: Background Layer 1 is disabled.         |
| **4** | BG0_ENABLE | 1: Background Layer 0 is enabled. 0: Background Layer 0 is disabled.         |
| **3** | (Reserved) | Unused.                                                                      |
| **2** | (Reserved) | Unused.                                                                      |
| **1** | (Reserved) | Unused.                                                                      |
| **0** | WIN_ENABLE | 1: The Window layer is enabled and will be drawn. 0: The Window is disabled. |

## **9. STAT Register (F041)**

This 8-bit register provides information about the PPU's current state and allows the CPU to request interrupts based on PPU events.

| Bit     | Name        | Function                                                                                                                      |
| :------ | :---------- | :---------------------------------------------------------------------------------------------------------------------------- |
| **7**   | UNUSED      | Unused, always reads 0.                                                                                                       |
| **6**   | LYC_INT_EN  | 1: Enable interrupt when LY == LYC. 0: Disable.                                                                               |
| **5**   | (Reserved)  | Unused.                                                                                                                       |
| **4**   | VBLK_INT_EN | 1: Enable interrupt when PPU enters V-Blank. 0: Disable.                                                                      |
| **3**   | HBLK_INT_EN | 1: Enable interrupt when PPU enters H-Blank. 0: Disable.                                                                      |
| **2**   | LYC_FLAG    | **(Read-only)** 1 if LY == LYC. 0 otherwise.                                                                                  |
| **1-0** | MODE_FLAG   | **(Read-only)** Indicates the PPU's current mode: <br> 00: H-Blank <br> 01: V-Blank <br> 10: OAM Scan <br> 11: Drawing Pixels |

## **10. PPU Frame Rendering Cycle**

The PPU renders the 240x160 screen one horizontal line (or scanline) at a time. The entire process is driven by a master clock and is meticulously timed. A full frame consists of 216 scanlines (LY 0-215) and takes 280,800 CPU cycles to complete, resulting in a refresh rate of approximately 59.739 Hz.

The rendering process is divided into two main phases: the period when pixels are drawn to the screen, and the Vertical Blank (V-Blank) period, which is the idle time between frames.

### **10.1. Phase 1: Visible Scanline Rendering (LY 0-159)**

For each of the 160 visible scanlines, the PPU performs the exact same sequence of operations, which takes a total of 1,300 CPU cycles. This sequence is divided into three modes, which correspond to the `MODE_FLAG` in the `STAT` register.

#### **Mode 2: OAM Scan (160 Cycles)**

At the beginning of a scanline, the PPU determines which sprites need to be drawn on this specific line.

- **Action:** The PPU iterates through all 64 sprite entries in OAM (Object Attribute Memory, `F400-F5FF`).
- **Condition:** It checks if the current scanline (`LY` register) falls within the vertical range of each sprite (i.e., `sprite.y <= LY < sprite.y + sprite.height`).
- **Result:** The PPU builds a temporary internal list of up to 16 sprites that are visible on this line. If more than 16 sprites are on the line, the additional ones are ignored for this frame. This list contains the sprite's X-position, tile index, attributes, and OAM index (for priority).
- **Status:** The `STAT` register's mode flag (bits 1-0) is set to `10`.

#### **Mode 3: Drawing Pixels (960 Cycles)**

This is the core of the rendering process, where the PPU composes the final color for each of the 240 horizontal pixels on the scanline. It processes one pixel every four CPU cycles.

- **Status:** The `STAT` register mode flag is set to `11`. Accessing VRAM, OAM, or CRAM during this mode can cause visual glitches.
- **The Pixel Pipeline:** For each pixel X from 0 to 239, the PPU performs the following logic:
  1.  **Fetch Pixels:** The PPU fetches a single pixel's data (a 4-bit color index) from all relevant layers that are enabled in the `LCDC` register. This includes:
      - **BG0 & BG1:** Based on the scroll registers (`SCX0`, `SCY0`, `SCX1`, `SCY1`) plus the current `LY` and `X` coordinates.
      - **Window:** If the current (`X`, `LY`) is within the window's rectangle (`WINX`, `WINY`).
      - **Sprites:** From the list of 16 sprites generated during the OAM scan.
  2.  **Layer & Priority Mixing:** The PPU combines these pixels using a fixed priority system to determine the final pixel to be drawn.
      - **Step A (Backdrop):** The pixel from BG0 is always the starting point. It is always opaque, even if its color index is 0.
      - **Step B (Backgrounds):** The pixel from BG1 is drawn on top of the BG0 pixel, but only if its color index is not 0 (color 0 is transparent).
      - **Step C (Window):** If the Window is active for this pixel, its pixel is drawn on top of the backgrounds, again treating color 0 as transparent.
      - **Step D (Sprites):** If a sprite pixel is present at this location, its priority is checked against the background pixel beneath it. Sprite-on-background priority is determined by the sprite's Priority flag (OAM Byte 4, Bit 7) and the background tile's Priority flag (Tilemap entry, Bit 15). A sprite with its priority flag set to `0` will be drawn behind a background tile with its priority flag set to `1`. Sprite-on-sprite priority is determined by OAM index: a sprite with a lower index (e.g., sprite #0) is always drawn on top of a sprite with a higher index (e.g., sprite #1).
  3.  **Final Color Lookup:** The result of the mixing logic is a 4-bit color index and a palette select value. The PPU uses these to look up the final 16-bit RGB555 color value from CRAM (`F200-F3FF`).

#### **Mode 0: Horizontal Blank (H-Blank) (180 Cycles)**

After the last pixel of a scanline is drawn, the PPU enters a short idle period before starting the next line.

- **Action:** The PPU is idle. This is a safe period for the CPU to write to VRAM, CRAM, and OAM without causing visual artifacts.
- **Status:** The `STAT` register mode flag is set to `00`. An H-Blank interrupt can be triggered if enabled.

### **10.2. Phase 2: Vertical Blank (V-Blank) Period (LY 160-215)**

After all 160 visible scanlines have been drawn, the PPU enters the V-Blank period, which lasts for 56 scanlines' worth of time.

#### **Mode 1: V-Blank (56 \* 1,300 Cycles)**

- **Action:** The PPU is completely idle and does no drawing. This is the main safe period for the CPU to perform lengthy graphics updates, such as updating tilemaps, copying new tile graphics to VRAM, or updating palettes in CRAM.
- **Status:** As soon as `LY` becomes 160, the `STAT` register mode flag is set to `01`.
- **V-Blank Interrupt:** The PPU sets the V-Blank flag in the `IF` register (`F021`, Bit 0). This is the most important interrupt in the system, as it signals to the game that it is time to prepare all the data for the next frame.

After the V-Blank period ends, the `LY` register wraps back to 0, and the entire frame rendering process begins again.

---

© 2025 Connor Nolan. This work is licensed under a
[Creative Commons Attribution-ShareAlike 4.0 International License](http://creativecommons.org/licenses/by-sa/4.0/).
