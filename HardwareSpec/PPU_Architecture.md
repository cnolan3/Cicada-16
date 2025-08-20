# **Cricket-16 PPU \- Architecture**

This document describes the design of the Picture Processing Unit (PPU) for the Cricket-16 fantasy console. The PPU is responsible for rendering all graphics to the screen. The design is inspired by 16-bit era consoles, featuring multiple background layers, sprites, and a palette-based color system.

## **1\. Core Components**

- **VRAM (Video RAM):** 8 KiB of dedicated RAM located at 6000-7FFF. VRAM holds the tile data (the 8x8 pixel building blocks for graphics) and the tilemaps (the data that arranges tiles into background layers).
- **OAM (Object Attribute Memory):** 512 bytes of dedicated RAM at F600-F7FF used to store the attributes for all 64 hardware sprites (position, tile index, palette, etc.).
- **CRAM (Color RAM / Palette RAM):** 512 bytes of internal PPU memory that holds the color palette data. This is not directly mapped to the CPU's address space but is accessed via PPU registers.
- **Screen Resolution:** 240x160 pixels.
- **Refresh Rate:** 60 Hz.

## **2\. Color System**

- **Master Palette:** The PPU can display **256** colors on screen at once.
- **Color RAM (CRAM):** The 256-color master palette is stored in CRAM. Each color entry is a 16-bit value, defining the color in RGB555 format (5 bits for Red, 5 for Green, 5 for Blue).
  - RRRRRGGGGGBBBBB
- **Sub-palettes:** The 256 colors in CRAM are divided into 16 sub-palettes of 16 colors each. Background layers and sprites can each be assigned one of these sub-palettes.
- **Color 0:** In any 16-color sub-palette, color 0 is treated as transparent. This allows sprites and background layers to show content behind them.

## **3\. Background Layers**

The PPU can render **two** independent background layers, referred to as **BG0** and **BG1**. This allows for effects like parallax scrolling.

- **Tilemaps:** Each background layer is constructed from a tilemap stored in VRAM. A tilemap is a 2D array of entries that specify which tile to draw at each position on the grid.
  - Tilemap sizes can be configured (e.g., 32x32 tiles, 64x32 tiles) to create large scrolling worlds.
- **Tiles:** The building blocks of backgrounds. Each tile is an 8x8 pixel graphic. The color data for tiles is stored in VRAM. We'll assume a 4-bit-per-pixel (4bpp) format, where each pixel's value is an index into one of the 16-color sub-palettes.
- **Scrolling:** Each background layer has its own horizontal and vertical scroll registers (SCX, SCY), allowing them to be moved independently.

### **3.1. Window Layer**

For displaying static UI elements like HUDs or dialogue boxes, the PPU supports a **Window Layer**.

- **Function:** The Window is a rectangular area that is not affected by background scrolling. It is rendered on top of the background layers but can be behind or in front of sprites.
- **Implementation:** The Window is not a true third layer. It re-uses the tile data and tilemap of one of the background layers (typically BG0). The PPU is instructed to draw a portion of this tilemap at a fixed screen position defined by the WINY and WINX registers, ignoring the regular scroll values for that area.

## **4\. Sprites (Objects)**

The PPU can render up to **64** sprites on screen at once.

- **Attributes:** Each sprite's data is stored in OAM as an 8-byte entry:
  - **Byte 0:** Y-Position (vertical screen coordinate)
  - **Byte 1:** X-Position (horizontal screen coordinate)
  - **Byte 2:** **Size & Shape attribute** \- Defines the sprite's dimensions (e.g., 8x8, 16x16, 8x16, 16x8). This is now a per-sprite setting.
  - **Byte 3:** Tile Index (which 8x8 tile from VRAM to use)
  - **Byte 4:** Flags (Palette select, H-Flip, V-Flip, Priority vs. Backgrounds)
  - **Bytes 5-7:** Reserved for future use (e.g., rotation/scaling data).
- **Scanline Limit:** The PPU can render a maximum of **16** sprites per horizontal scanline. If more than 16 sprites are on a line, the additional ones will not be drawn.

## **5\. PPU Registers (F100-F1FF)**

These registers, mapped to the CPU's address space, control the PPU's operation.

| Address | Name      | Description                                                                                                                                                                              |
| :------ | :-------- | :--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| F100    | LCDC      | **LCD Control:** Master switch for the PPU. Contains bits to enable/disable the screen, background layers, sprites, and select tilemap/tiledata base addresses.                          |
| F101    | STAT      | **LCD Status:** Contains flags indicating the PPU's current mode (H-Blank, V-Blank, OAM Scan) and can be configured to trigger interrupts on specific events.                            |
| F104    | SCY0      | Background 0 \- Vertical Scroll                                                                                                                                                          |
| F105    | SCX0      | Background 0 \- Horizontal Scroll                                                                                                                                                        |
| F106    | SCY1      | Background 1 \- Vertical Scroll                                                                                                                                                          |
| F107    | SCX1      | Background 1 \- Horizontal Scroll                                                                                                                                                        |
| F108    | WINY      | **Window Y-Position:** The top edge of the Window layer.                                                                                                                                 |
| F109    | WINX      | **Window X-Position:** The left edge of the Window layer.                                                                                                                                |
| F10A    | LY        | **LCD Y-Coordinate:** Indicates the current vertical scanline being drawn (Read-only). Ranges from 0 to \~180.                                                                           |
| F10B    | LYC       | **LY Compare:** The PPU compares LY with this value. If they match, a flag is set in the STAT register, which can trigger an interrupt. Useful for scanline-based effects.               |
| F110    | CRAM_ADDR | **Color RAM Address:** The CPU writes a CRAM index (0-255) to this register.                                                                                                             |
| F111    | CRAM_DATA | **Color RAM Data:** The CPU writes a 16-bit color via **two consecutive 8-bit writes** to this address (low byte, then high byte). The CRAM_ADDR auto-increments after the second write. |
| F112    | BG_PAL    | **Background Palettes:** Selects which of the 16 sub-palettes to use for each background layer.                                                                                          |

## **6\. LCDC Register (F100)**

This 8-bit register is the primary control for the PPU. Writing to this address configures most of the PPU's rendering features.

| Bit   | Name         | Function                                                                                |
| :---- | :----------- | :-------------------------------------------------------------------------------------- |
| **7** | PPU_ENABLE   | 1: PPU is on and drawing to the screen. 0: PPU is off (screen is blank).                |
| **6** | SPR_ENABLE   | 1: Sprites are enabled and will be drawn. 0: Sprites are disabled.                      |
| **5** | BG1_ENABLE   | 1: Background Layer 1 is enabled. 0: Background Layer 1 is disabled.                    |
| **4** | BG0_ENABLE   | 1: Background Layer 0 is enabled. 0: Background Layer 0 is disabled.                    |
| **3** | TILE_SEL     | Selects the base address for tile data. 0: Tiles from 0x6000. 1: Tiles from 0x6800.     |
| **2** | BG1_TMAP_SEL | Selects the base address for BG1's tilemap. 0: Tilemap at 0x7000. 1: Tilemap at 0x7800. |
| **1** | BG0_TMAP_SEL | Selects the base address for BG0's tilemap. 0: Tilemap at 0x7400. 1: Tilemap at 0x7C00. |
| **0** | WIN_ENABLE   | 1: The Window layer is enabled and will be drawn. 0: The Window is disabled.            |

## **7\. STAT Register (F101)**

This 8-bit register provides information about the PPU's current state and allows the CPU to request interrupts based on PPU events.

| Bit     | Name        | Function                                                                                                                              |
| :------ | :---------- | :------------------------------------------------------------------------------------------------------------------------------------ |
| **7**   | UNUSED      | Unused, always reads 0\.                                                                                                              |
| **6**   | LYC_INT_EN  | 1: Enable interrupt when LY \== LYC. 0: Disable.                                                                                      |
| **5**   | OAM_INT_EN  | 1: Enable interrupt when PPU enters OAM Scan mode. 0: Disable.                                                                        |
| **4**   | VBLK_INT_EN | 1: Enable interrupt when PPU enters V-Blank. 0: Disable.                                                                              |
| **3**   | HBLK_INT_EN | 1: Enable interrupt when PPU enters H-Blank. 0: Disable.                                                                              |
| **2**   | LYC_FLAG    | **(Read-only)** 1 if LY \== LYC. 0 otherwise.                                                                                         |
| **1-0** | MODE_FLAG   | **(Read-only)** Indicates the PPU's current mode: \<br\> 00: H-Blank \<br\> 01: V-Blank \<br\> 10: OAM Scan \<br\> 11: Drawing Pixels |
