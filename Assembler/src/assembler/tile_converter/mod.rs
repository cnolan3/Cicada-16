/*
Copyright 2025 Connor Nolan

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

use crate::errors::AssemblyError;
use image::{DynamicImage, GenericImageView, ImageError, Rgba};
use std::path::Path;

/// Represents a single 8x8 tile with 4-bit color indices
struct Tile8x8 {
    pixels: [[u8; 8]; 8],
}

pub struct TileConverter;

impl TileConverter {
    /// Convert an image file to Cicada-16 4bpp planar tile data
    pub fn convert_image_to_tiles(
        image_path: &Path,
        x_pixels: Option<u16>,
        y_pixels: Option<u16>,
        width_pixels: Option<u16>,
        height_pixels: Option<u16>,
        line: usize,
    ) -> Result<Vec<u8>, AssemblyError> {
        // Load the image
        let img = Self::load_image(image_path, line)?;

        // Determine the region to extract
        let (x, y, width, height) = if let (Some(x), Some(y), Some(w), Some(h)) =
            (x_pixels, y_pixels, width_pixels, height_pixels)
        {
            (x as u32, y as u32, w as u32, h as u32)
        } else {
            (0, 0, img.width(), img.height())
        };

        // Validate dimensions
        Self::validate_dimensions(x, y, width, height, img.width(), img.height(), line)?;

        // Extract tiles from the region
        let tiles = Self::extract_tiles(&img, x, y, width, height, line)?;

        // Convert each tile to 4bpp planar format
        let mut output = Vec::new();
        for tile in tiles {
            output.extend(Self::convert_tile_to_4bpp_planar(&tile));
        }

        Ok(output)
    }

    /// Load an image from a file path
    fn load_image(path: &Path, line: usize) -> Result<DynamicImage, AssemblyError> {
        image::open(path).map_err(|e| match e {
            ImageError::IoError(io_err) => AssemblyError::ImageError {
                line,
                reason: format!("Failed to read image file '{}': {}", path.display(), io_err),
            },
            ImageError::Decoding(dec_err) => AssemblyError::ImageError {
                line,
                reason: format!("Failed to decode image '{}': {}", path.display(), dec_err),
            },
            ImageError::Unsupported(unsup_err) => AssemblyError::ImageError {
                line,
                reason: format!(
                    "Unsupported image format '{}': {}",
                    path.display(),
                    unsup_err
                ),
            },
            _ => AssemblyError::ImageError {
                line,
                reason: format!("Image error for '{}': {}", path.display(), e),
            },
        })
    }

    /// Validate that dimensions are appropriate for tile extraction
    fn validate_dimensions(
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        img_width: u32,
        img_height: u32,
        line: usize,
    ) -> Result<(), AssemblyError> {
        // Check 8-pixel alignment
        if x % 8 != 0 {
            return Err(AssemblyError::ImageDimensionError {
                line,
                reason: format!("X coordinate {} must be a multiple of 8", x),
            });
        }
        if y % 8 != 0 {
            return Err(AssemblyError::ImageDimensionError {
                line,
                reason: format!("Y coordinate {} must be a multiple of 8", y),
            });
        }
        if width % 8 != 0 {
            return Err(AssemblyError::ImageDimensionError {
                line,
                reason: format!("Width {} must be a multiple of 8", width),
            });
        }
        if height % 8 != 0 {
            return Err(AssemblyError::ImageDimensionError {
                line,
                reason: format!("Height {} must be a multiple of 8", height),
            });
        }

        // Check bounds
        if x + width > img_width {
            return Err(AssemblyError::ImageDimensionError {
                line,
                reason: format!(
                    "Region exceeds image width (x={}, width={}, image_width={})",
                    x, width, img_width
                ),
            });
        }
        if y + height > img_height {
            return Err(AssemblyError::ImageDimensionError {
                line,
                reason: format!(
                    "Region exceeds image height (y={}, height={}, image_height={})",
                    y, height, img_height
                ),
            });
        }

        Ok(())
    }

    /// Extract 8x8 tiles from the specified region
    fn extract_tiles(
        img: &DynamicImage,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        line: usize,
    ) -> Result<Vec<Tile8x8>, AssemblyError> {
        let mut tiles = Vec::new();
        let tiles_wide = width / 8;
        let tiles_high = height / 8;

        for tile_y in 0..tiles_high {
            for tile_x in 0..tiles_wide {
                let tile = Self::extract_single_tile(
                    img,
                    x + tile_x * 8,
                    y + tile_y * 8,
                    line,
                )?;
                tiles.push(tile);
            }
        }

        Ok(tiles)
    }

    /// Extract a single 8x8 tile starting at the given position
    fn extract_single_tile(
        img: &DynamicImage,
        start_x: u32,
        start_y: u32,
        line: usize,
    ) -> Result<Tile8x8, AssemblyError> {
        let mut pixels = [[0u8; 8]; 8];

        for row in 0..8 {
            for col in 0..8 {
                let pixel = img.get_pixel(start_x + col, start_y + row);
                let color_index = Self::rgba_to_4bit_index(&pixel, line)?;
                pixels[row as usize][col as usize] = color_index;
            }
        }

        Ok(Tile8x8 { pixels })
    }

    /// Convert RGBA pixel to 4-bit color index
    /// This is a simplified conversion - in a real implementation you might want
    /// to use a palette or quantization algorithm
    fn rgba_to_4bit_index(pixel: &Rgba<u8>, line: usize) -> Result<u8, AssemblyError> {
        // Simple grayscale conversion to 4-bit (0-15)
        // Formula: (0.299*R + 0.587*G + 0.114*B) / 16
        let r = pixel[0] as u32;
        let g = pixel[1] as u32;
        let b = pixel[2] as u32;

        let gray = (299 * r + 587 * g + 114 * b) / 1000;
        let index = (gray / 17) as u8;  // Map 0-255 to 0-15

        if index > 15 {
            Err(AssemblyError::ImageError {
                line,
                reason: format!("Color index {} exceeds 4-bit range (0-15)", index),
            })
        } else {
            Ok(index)
        }
    }

    /// Convert a single tile to 4bpp planar format (32 bytes)
    fn convert_tile_to_4bpp_planar(tile: &Tile8x8) -> Vec<u8> {
        let mut output = Vec::with_capacity(32);

        // For each of 4 bit planes
        for plane_bit in 0..4 {
            // For each of 8 rows
            for row in 0..8 {
                let mut byte = 0u8;
                // For each of 8 pixels in the row
                for col in 0..8 {
                    let pixel = tile.pixels[row][col];
                    let bit = (pixel >> plane_bit) & 1;
                    byte |= bit << (7 - col); // MSB = leftmost pixel
                }
                output.push(byte);
            }
        }

        output
    }

    /// Calculate the size in bytes that a given image region will produce
    /// This is used during symbol table building to reserve space
    pub fn calculate_size(
        image_path: &Path,
        x_pixels: Option<u16>,
        y_pixels: Option<u16>,
        width_pixels: Option<u16>,
        height_pixels: Option<u16>,
        line: usize,
    ) -> Result<u32, AssemblyError> {
        // Load image to get dimensions
        let img = Self::load_image(image_path, line)?;

        // Determine the region
        let (x, y, width, height) = if let (Some(x), Some(y), Some(w), Some(h)) =
            (x_pixels, y_pixels, width_pixels, height_pixels)
        {
            (x as u32, y as u32, w as u32, h as u32)
        } else {
            (0, 0, img.width(), img.height())
        };

        // Validate dimensions
        Self::validate_dimensions(x, y, width, height, img.width(), img.height(), line)?;

        // Calculate number of tiles
        let tiles_wide = width / 8;
        let tiles_high = height / 8;
        let num_tiles = tiles_wide * tiles_high;

        // Each tile is 32 bytes
        Ok(num_tiles * 32)
    }
}
