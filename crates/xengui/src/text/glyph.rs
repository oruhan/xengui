// SPDX-License-Identifier: Apache-2.0

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// Data and behavior represented by `GlyphKey`.
pub struct GlyphKey {
    /// The `font` value carried by this type.
    pub font: u32,
    /// The `glyph_id` value carried by this type.
    pub glyph_id: u32,
    /// The `size` value carried by this type.
    pub size: u32,
}

#[derive(Clone, Debug)]
/// Data and behavior represented by `GlyphBitmap`.
pub struct GlyphBitmap {
    /// The `width` value carried by this type.
    pub width: u32,
    /// The `height` value carried by this type.
    pub height: u32,

    /// The `left` value carried by this type.
    pub left: i32,
    /// The `top` value carried by this type.
    pub top: i32,

    /// The `advance` value carried by this type.
    pub advance: f32,

    // 8-bit alpha bitmap.
    /// The `pixels` value carried by this type.
    pub pixels: Vec<u8>,
}
