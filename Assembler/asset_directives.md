# Asset Directives Design Document

This document outlines potential new directives for the Cicada-16 assembler to improve the asset pipeline for palette data, tilemaps, and sprite metadata.

## Table of Contents

- [Palette Data Directives](#palette-data-directives)
- [Tilemap Data Directives](#tilemap-data-directives)
- [Advanced Directive Ideas](#advanced-directive-ideas)
- [File Format Support](#file-format-support)
- [Design Philosophy](#design-philosophy)
- [Implementation Priority](#implementation-priority)

---

## Palette Data Directives

### Option 1: `.inc_palette` - Direct Palette File Import

Import palettes from external files in various standard formats.

**Syntax:**
```assembly
; Import a palette from a JSON, PNG, or custom format
.inc_palette "game_palette.json"
.inc_palette "reference_image.png", 0, 16  ; Extract first 16 colors from PNG
```

**Benefits:**
- Artists can define palettes in their image editor
- Extract palettes directly from reference art
- Support for standard formats (GIMP .gpl, Aseprite .ase)

**Implementation Considerations:**
- JSON format: `[{"r":255,"g":0,"b":0}, ...]` or hex strings `["#FF0000", ...]`
- PNG extraction: Read first N unique colors from an indexed PNG's palette
- Convert RGB888 to RGB555 for Cicada-16's format
- Validate max 16 colors per directive call

**Example JSON Palette Format:**
```json
{
  "name": "main_palette",
  "colors": [
    "#000000",
    "#FFFFFF",
    "#FF0000",
    "#00FF00",
    "#0000FF"
  ]
}
```

---

### Option 2: `.palette` - Inline Palette Definition

Define palettes directly in assembly code for self-contained files.

**Syntax:**
```assembly
; Define palette inline with RGB values
.palette_start palette_0
    .rgb 0, 0, 0           ; Color 0: black
    .rgb 255, 255, 255     ; Color 1: white
    .rgb 128, 64, 32       ; Color 2: brown
    ; ... up to 16 colors
.palette_end

; Or hex format
.palette_start palette_1
    .rgb555 0x0000         ; Raw RGB555 value
    .rgb555 0x7FFF
    .rgb555 0x3DEF
.palette_end

; Or shorthand
.palette palette_2, 0x0000, 0x7FFF, 0x3DEF, 0x294A  ; List of RGB555 values
```

**Benefits:**
- Self-contained assembly files
- Easy to tweak colors without external tools
- Clear documentation of what each color is
- Good for procedural/programmatic color generation

**Notes:**
- `.rgb` variant converts RGB888 (0-255) to RGB555 automatically
- `.rgb555` variant uses raw hardware format
- Shorthand syntax for simple cases

---

### Option 3: `.palette_extract` - Extract from Tiledata Source

Automatically extract palette from the same PNG used for tiles.

**Syntax:**
```assembly
; Automatically extract palette from the same PNG used for tiles
.palette_extract "sprite.png"  ; Gets all colors from indexed PNG palette

; With range specification
.palette_extract "sprite.png", 0, 8   ; Extract colors 0-7 only
```

**Benefits:**
- DRY principle - don't repeat the image filename
- Guarantees palette matches the tile data
- Automatic workflow for artists
- Reduces chance of palette/tile mismatches

**Example Usage:**
```assembly
; Load both tiles and palette from same source
sprite_tiles:
    .inc_tiledata "player.png"

sprite_palette:
    .palette_extract "player.png"  ; Auto-extracts the palette
```

---

## Tilemap Data Directives

### Option 1: `.inc_tilemap` - Import Tilemap from Tiled/LDtk

Import tilemaps from industry-standard level editors.

**Syntax:**
```assembly
; Import a tilemap from Tiled's JSON export
.inc_tilemap "level1.tmj", "Background"      ; Layer name
.inc_tilemap "level1.tmj", 0                 ; Layer index

; Import from LDtk
.inc_tilemap "world.ldtk", "Level_0", "Ground"

; With options block
.inc_tilemap_start "level1.tmj", "Background"
    .flip_support true      ; Include flip bits
    .palette_offset 2       ; Add 2 to all palette indices
    .priority_layer true    ; Set priority bit on all tiles
.inc_tilemap_end
```

**Implementation:**
- Parse JSON from Tiled or LDtk
- Extract tile indices and arrangement
- Convert to Cicada-16's tilemap format (16-bit entries)
- Support for tile flipping (if hardware supports it)
- Handle layer ordering and visibility

**Tiled TMJ Format Example:**
```json
{
  "layers": [
    {
      "name": "Background",
      "width": 32,
      "height": 20,
      "data": [1, 2, 3, 4, ...]
    }
  ]
}
```

---

### Option 2: `.tilemap` - Inline Tilemap Definition

Define tilemaps directly in assembly for small maps or procedural templates.

**Syntax:**
```assembly
; Define a tilemap inline with tile indices
.tilemap_start level1_bg, 32, 20  ; 32 tiles wide, 20 tiles tall
    ; Row-major order, each value is tile index
    .tile_row 0, 0, 0, 0, 1, 1, 1, 1, ...  ; 32 values
    .tile_row 0, 0, 2, 3, 4, 5, 6, 7, ...
    ; ... 20 rows total
.tilemap_end

; Or with attributes
.tilemap_start level1_fg, 32, 20
    .tile 0 | HFLIP         ; Tile 0, horizontally flipped
    .tile 1 | VFLIP         ; Tile 1, vertically flipped
    .tile 2 | PALETTE(3)    ; Tile 2, using palette 3
    .tile 3 | PRIORITY      ; Tile 3, high priority
    ; ...
.tilemap_end

; Compact 2D syntax
.tilemap 8, 8, [
    0, 0, 0, 1, 1, 0, 0, 0,
    0, 0, 1, 2, 2, 1, 0, 0,
    0, 1, 2, 3, 3, 2, 1, 0,
    1, 2, 3, 4, 4, 3, 2, 1,
    1, 2, 3, 4, 4, 3, 2, 1,
    0, 1, 2, 3, 3, 2, 1, 0,
    0, 0, 1, 2, 2, 1, 0, 0,
    0, 0, 0, 1, 1, 0, 0, 0
]
```

**Benefits:**
- Small tilemaps can be defined in code
- Good for procedural generation templates
- Easy to visualize structure
- No external files needed for simple cases

**Attribute Flags (if supported by hardware):**
- `HFLIP` - Horizontal flip
- `VFLIP` - Vertical flip
- `PALETTE(n)` - Palette selection
- `PRIORITY` - Priority/layering

---

### Option 3: `.tilemap_rle` - RLE Compressed Tilemap

Define tilemaps with run-length encoding for compression.

**Syntax:**
```assembly
; Define tilemap with run-length encoding
.tilemap_rle_start bg_layer, 64, 32
    .run 128, 0          ; 128 tiles of index 0 (empty)
    .run 16, 1           ; 16 tiles of index 1
    .tile 5              ; Single tile of index 5
    .run 32, 0           ; 32 more empty tiles
    .tile 10
    .tile 11
    .run 64, 2
    ; ...
.tilemap_rle_end
```

**Benefits:**
- Compact for large tilemaps with repeating patterns
- Natural for backgrounds with lots of empty space
- Assembler can auto-decompress at build time
- Reduces ROM usage significantly

**Use Cases:**
- Large scrolling backgrounds
- Tilemaps with lots of empty/sky tiles
- Repeating patterns (water, ground, etc.)

---

## Advanced Directive Ideas

### `.sprite_metadata` - Sprite Metadata Tables

Define comprehensive sprite metadata alongside tile data.

**Syntax:**
```assembly
; Define sprite metadata alongside tile data
.sprite_def player
    .tiles "player.png"              ; Load tile data
    .palette "player_palette.json"   ; Load palette
    .size 16, 16                     ; 16x16 pixels
    .origin 8, 15                    ; Origin point (feet)
    .hitbox 4, 2, 8, 14             ; x, y, w, h
.sprite_end

; Generates:
; - player_tiles: tile data bytes
; - player_palette: palette data
; - player_metadata: struct with size, origin, hitbox
```

**Generated Constants:**
```assembly
; Assembler auto-generates these
player_tiles:           ; Address of tile data
    .inc_tiledata "player.png"

player_palette:         ; Address of palette data
    .inc_palette "player_palette.json"

player_metadata:        ; Metadata structure
    .word 16            ; Width
    .word 16            ; Height
    .word 8             ; Origin X
    .word 15            ; Origin Y
    .word 4             ; Hitbox X
    .word 2             ; Hitbox Y
    .word 8             ; Hitbox Width
    .word 14            ; Hitbox Height

; Also generates helpful constants
.define PLAYER_WIDTH 16
.define PLAYER_HEIGHT 16
.define PLAYER_ORIGIN_X 8
.define PLAYER_ORIGIN_Y 15
```

**Benefits:**
- All sprite data in one place
- Auto-generates helpful constants
- Type-safe metadata access
- Reduces boilerplate

---

### `.metatile` - Metatile Definitions

Define metatiles (groups of tiles treated as a single unit).

**Syntax:**
```assembly
; Define metatiles (2x2 or larger tile groups)
.metatile grass_block, 2, 2
    1, 2,     ; Top-left, top-right tile indices
    3, 4      ; Bottom-left, bottom-right
.end

.metatile tree, 3, 4   ; 3 tiles wide, 4 tiles tall
    10, 11, 12,
    13, 14, 15,
    16, 17, 18,
    19, 20, 21
.end

; Use in tilemap
.tilemap_metatile level1, 16, 16
    grass_block, grass_block, tree, grass_block, ...
.end
```

**Benefits:**
- Common pattern in retro games
- Reduces tilemap size significantly
- Easier level editing
- Logical grouping of related tiles

**Example Usage:**
```assembly
; Define metatiles
.metatile empty, 2, 2
    0, 0,
    0, 0
.end

.metatile platform, 2, 2
    16, 17,
    18, 19
.end

; Use them in a level
.tilemap_metatile level1, 10, 8
    empty, empty, empty, empty, empty, empty, empty, empty, empty, empty,
    empty, empty, empty, empty, empty, empty, empty, empty, empty, empty,
    empty, empty, platform, platform, empty, empty, empty, empty, empty, empty,
    empty, empty, empty, empty, empty, empty, platform, platform, empty, empty,
    ; ...
.end
```

---

### `.animated_tiles` - Animation Sequences

Define animated tile sequences with frame timing.

**Syntax:**
```assembly
; Define animated tile sequences
.anim_tiles water, 4          ; 4 frames
    .frame 10, 8              ; Tile 10, 8 frames duration
    .frame 11, 8
    .frame 12, 8
    .frame 13, 8
.end

.anim_tiles torch, 3
    .frame 20, 4
    .frame 21, 4
    .frame 22, 4
.end

; Generates animation metadata table
```

**Generated Data:**
```assembly
water_anim:
    .byte 4              ; Frame count
    .byte 10, 8          ; Frame 0: tile 10, duration 8
    .byte 11, 8          ; Frame 1: tile 11, duration 8
    .byte 12, 8          ; Frame 2: tile 12, duration 8
    .byte 13, 8          ; Frame 3: tile 13, duration 8
```

**Runtime Usage:**
```assembly
; Update animated tiles (pseudo-code)
update_animations:
    LDI R0, water_anim
    CALL animate_tile
    LDI R0, torch_anim
    CALL animate_tile
    RET
```

---

### `.collision_map` - Collision Data

Define collision data alongside tilemaps.

**Syntax:**
```assembly
; Define collision alongside tilemap
.collision_map level1, 32, 20
    .solid_tiles 1, 2, 3, 4, 5          ; These tile IDs are solid
    .slope_tiles 10, 11, 12             ; Slope tiles
    .damage_tiles 20, 21                ; Damage tiles
    .platform_tiles 30, 31, 32          ; One-way platforms
.end

; Or bitmap approach
.collision_bitmap level1, 32, 20
    0, 0, 0, 1, 1, 1, ...  ; 0=passable, 1=solid
.end

; Or attribute-based
.collision_attr level1, 32, 20
    .attr SOLID | DAMAGE    ; Tile 0 attributes
    .attr PASSABLE          ; Tile 1 attributes
    .attr PLATFORM          ; Tile 2 attributes
    ; ...
.end
```

**Generated Lookup Table:**
```assembly
level1_collision:
    .byte 0x03  ; Tile 1: SOLID
    .byte 0x03  ; Tile 2: SOLID
    .byte 0x03  ; Tile 3: SOLID
    .byte 0x05  ; Tile 10: SLOPE
    .byte 0x09  ; Tile 20: DAMAGE
    ; ...
```

**Benefits:**
- Collision data stored with level data
- Type-safe collision attributes
- Easy to modify and maintain
- Supports multiple collision types

---

## File Format Support

### Format Priority

1. **Aseprite .ase/.aseprite** - **RECOMMENDED** - Reliably preserves indexed color mode and palette data
2. **PNG (indexed)** - Supported with limitations (validation is best-effort, may not reliably detect indexed vs grayscale)
3. **JSON** - Easy to parse, human-readable, tool-friendly
4. **Tiled TMJ/TMX** - Industry standard for tilemaps
5. **LDtk JSON** - Modern, popular level editor
6. **GIMP GPL** - Standard palette format

### JSON Metadata File Format Example

```json
{
  "palettes": [
    {
      "name": "main_palette",
      "colors": [
        "#000000",
        "#FFFFFF",
        "#FF0000",
        "#00FF00",
        "#0000FF",
        "#FFFF00",
        "#FF00FF",
        "#00FFFF"
      ]
    },
    {
      "name": "night_palette",
      "colors": [
        "#000000",
        "#1A1A2E",
        "#16213E",
        "#0F3460",
        "#533483"
      ]
    }
  ],
  "sprites": [
    {
      "name": "player",
      "file": "player.png",
      "palette": "main_palette",
      "size": [16, 16],
      "origin": [8, 15],
      "hitbox": [4, 2, 8, 14],
      "frames": [
        {"x": 0, "y": 0, "w": 16, "h": 16, "duration": 8},
        {"x": 16, "y": 0, "w": 16, "h": 16, "duration": 8},
        {"x": 32, "y": 0, "w": 16, "h": 16, "duration": 8}
      ]
    },
    {
      "name": "enemy",
      "file": "enemy.png",
      "palette": "main_palette",
      "size": [16, 16],
      "origin": [8, 15],
      "hitbox": [2, 0, 12, 16]
    }
  ],
  "levels": [
    {
      "name": "level1",
      "tilemap": "level1.tmj",
      "background_layer": "Background",
      "foreground_layer": "Foreground",
      "collision_layer": "Collision"
    }
  ]
}
```

### Tiled TMJ Format Support

**Supported Features:**
- Multiple layers
- Tile flipping/rotation
- Tile properties
- Layer visibility
- Infinite maps (with chunking)

**Example Integration:**
```assembly
.inc_tilemap "dungeon.tmj", "Background"
.inc_tilemap "dungeon.tmj", "Foreground"
.inc_tilemap "dungeon.tmj", "Collision"
```

### LDtk Format Support

**Supported Features:**
- Multi-level worlds
- Entity placement
- Auto-tiling rules
- Enum-based tile properties

**Example Integration:**
```assembly
.inc_tilemap "world.ldtk", "Level_0", "Ground"
.inc_tilemap "world.ldtk", "Level_0", "Decorations"
```

---

## Assembly Output Organization

### Recommended Memory Layout

```assembly
; Auto-generated organization
.bank 0
.org 0x0100

; Entry point
start:
    JMP main

.org 0x1000
; === Sprite Tiles ===
sprites:
    player_tiles:
        .inc_tiledata "player.png"

    enemy_tiles:
        .inc_tiledata "enemy.png"

    item_tiles:
        .inc_tiledata "items.png"

.org 0x2000
; === Background Tiles ===
bg_tiles:
    tileset_tiles:
        .inc_tiledata "tileset.png"

.org 0x3000
; === Palettes ===
palettes:
    palette_0:
        .inc_palette "main.json"

    palette_1:
        .inc_palette "night.json"

    palette_2:
        .palette player_pal, 0x0000, 0x7FFF, 0x3DEF, 0x294A

.bank 1
.org 0x4000
; === Tilemaps ===
tilemaps:
    level1_bg:
        .inc_tilemap "level1.tmj", "Background"

    level1_fg:
        .inc_tilemap "level1.tmj", "Foreground"

    level2_bg:
        .inc_tilemap "level2.tmj", "Background"

.org 0x5000
; === Sprite Metadata ===
sprite_metadata:
    .sprite_def player
        .tiles "player.png"
        .palette "player_palette.json"
        .size 16, 16
        .origin 8, 15
        .hitbox 4, 2, 8, 14
    .sprite_end

    .sprite_def enemy
        .tiles "enemy.png"
        .palette "enemy_palette.json"
        .size 16, 16
        .origin 8, 15
        .hitbox 2, 0, 12, 16
    .sprite_end

.org 0x6000
; === Animation Data ===
animations:
    .anim_tiles water, 4
        .frame 10, 8
        .frame 11, 8
        .frame 12, 8
        .frame 13, 8
    .end

    .anim_tiles fire, 3
        .frame 20, 4
        .frame 21, 4
        .frame 22, 4
    .end
```

---

## Design Philosophy

### Core Principles

1. **Separate Concerns**
   - Tiles, palettes, and tilemaps should be separate directives
   - Each directive has a single, clear responsibility
   - Composition over monolithic directives

2. **Support Both Inline and File-Based**
   - Small data can be defined inline
   - Large data loaded from files
   - Flexibility for different workflows

3. **Standard Formats First**
   - Use existing formats (JSON, PNG, TMJ) before inventing new ones
   - Interoperate with existing tools
   - Lower learning curve for developers

4. **Validation at Assembly Time**
   - Catch palette/tile mismatches early
   - Verify dimensions and constraints
   - Clear error messages with actionable fixes

5. **Clear Error Messages**
   - Like the tiledata rework, guide users to fix issues
   - Provide file/line context
   - Suggest solutions, not just problems

6. **Optional Compression**
   - Let users choose between size and simplicity
   - RLE for tilemaps that benefit
   - Raw data for easy editing

7. **Metadata Generation**
   - Auto-generate size/count constants for runtime use
   - Reduce boilerplate in game code
   - Type-safe access to asset properties

### Error Message Examples

**Good Error Messages:**
```
Error on line 42: Palette 'main_palette.json' has 18 colors, but max 16 are supported.
  Hint: Reduce the number of colors in your palette file, or split into multiple 16-color palettes.

Error on line 56: Tilemap 'level1.tmj' layer 'Background' references tile index 128, but tileset only has 64 tiles.
  Hint: Check that your Tiled map is using the correct tileset.

Error on line 78: Sprite 'player.png' uses palette indices 0-23, exceeding the 4-bit limit (0-15).
  Hint: Convert sprite to indexed color with max 16 colors. See MIGRATION_GUIDE.md
```

---

## Implementation Priority

If implementing these directives, the recommended order is:

### Phase 1: Core Palette Support
1. **`.inc_palette`** - Most needed, complements `.inc_tiledata` well
2. **`.palette`** - Inline fallback for quick prototyping

**Rationale:** Palette support is the natural next step after tile data. Together they provide a complete graphics asset pipeline.

### Phase 2: Tilemap Integration
3. **`.inc_tilemap`** - Tiled integration is huge for productivity
4. **`.tilemap`** - Inline tilemaps for small/procedural use

**Rationale:** With tiles and palettes done, tilemaps complete the basic asset triangle.

### Phase 3: Advanced Features
5. **`.sprite_def`** - Higher-level abstraction for better DX
6. **`.metatile`** - Once tilemaps are stable
7. **`.animated_tiles`** - Animation metadata
8. **`.collision_map`** - Collision data

**Rationale:** These build on the foundation and provide quality-of-life improvements.

### Phase 4: Optimization
9. **`.tilemap_rle`** - Compression for large maps
10. **`.palette_extract`** - DRY improvements

**Rationale:** Once the core workflow is solid, add optimizations and conveniences.

---

## Example Complete Workflow

### Artist/Designer Workflow

1. **Create graphics in Aseprite**
   - Design sprites in indexed color mode (16 colors max)
   - Export as PNG

2. **Create levels in Tiled**
   - Use the exported tiles as tileset
   - Design level layout
   - Export as TMJ (JSON)

3. **Define metadata** (optional)
   - Create JSON file with sprite metadata
   - Or use inline `.sprite_def` in assembly

### Developer Workflow

```assembly
; game.asm - Main assembly file

.include "constants.asm"
.include "assets.asm"

.org 0x0100
start:
    CALL init_graphics
    CALL load_level1
    JMP main_loop

; assets.asm - Asset definitions
.bank 1
.org 0x4000

; === Graphics Assets ===
player_gfx:
    .sprite_def player
        .tiles "player.png"
        .palette "palettes/main.json"
        .size 16, 16
        .origin 8, 15
        .hitbox 4, 2, 8, 14
    .sprite_end

; === Level Data ===
level1_data:
    .inc_tilemap "levels/level1.tmj", "Background"

level1_collision:
    .collision_map level1, 64, 32
        .solid_tiles 1, 2, 3, 4, 5, 6, 7, 8
        .platform_tiles 16, 17, 18
    .end
```

### Build Process

```bash
# Assemble the game
casm game.asm -o game.bin

# The assembler:
# 1. Reads player.png and converts to 4bpp planar tiles
# 2. Reads palettes/main.json and converts to RGB555
# 3. Reads levels/level1.tmj and extracts tilemap data
# 4. Generates metadata constants
# 5. Outputs final ROM
```

---

## Future Considerations

### Potential Extensions

- **Sprite sheet packing**: Auto-pack multiple sprites into tileset
- **Palette optimization**: Auto-generate shared palettes
- **Map compression**: Additional compression algorithms beyond RLE
- **Asset validation**: Cross-check tile/palette/map consistency
- **Watch mode**: Rebuild on asset file changes
- **Asset report**: Generate size/usage statistics

### Tool Integration

- **Aseprite plugin**: Export directly to Cicada-16 format
- **Tiled plugin**: Custom properties for Cicada-16 hardware
- **VS Code extension**: Syntax highlighting for new directives
- **Asset viewer**: Preview assembled graphics data

---

## Summary

This comprehensive asset directive system provides:

✅ **Complete Pipeline**: Tiles → Palettes → Tilemaps → Sprites
✅ **Tool Integration**: Works with Aseprite, Tiled, LDtk, GIMP
✅ **Flexibility**: Both inline and file-based workflows
✅ **Validation**: Catch errors at assembly time
✅ **Optimization**: Optional compression and DRY features
✅ **Developer Experience**: Clear errors, auto-generated constants

The goal is to make asset integration seamless while maintaining the assembler's philosophy of clear, predictable behavior.
