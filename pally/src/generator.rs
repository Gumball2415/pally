//! Responsible for interacting with the composite encoder and decoder, and
//! returning the final encoded color.

use nes_ppu_cvbs::*;
use cvbs_decode::*;

/// File output format.
pub enum FileFormatType {
    /// .pal uint8
    PalUint8,
    /// .pal double
    PalDouble,
    /// .pal Jasc
    PalJasc,
    /// .gpl
    Gpl,
    /// .png
    Png,
    /// .txt HTML hex
    TxtHtmlHex,
    /// .txt MediaWiki
    TxtMediaWiki,
    /// C header .h uint8_t
    HeaderUint8T,
}

/// Method for clipping out-of-range RGB colors.
pub enum ClipType {
    /// If any of the RGB channels are greater than 1.0, subtract all channels
    /// by delta of highest value.
    /// 
    /// Algorithm by DragWx.
    Darken,
    /// If any of the RGB channels are greater than 1.0, desaturate the color
    /// until all channels are within range.
    /// 
    /// Algorithm by DragWx.
    Desaturate,
}

/// Method for scaling out-of-range RGB colors into gamut.
pub enum NormalizeType {
    /// Scale all RGB values within 0.0 to 1.0.
    Scale,
    /// Clips all negative RGB values, then scales them within 0.0 to 1.0.
    ScaleClipNegative,
}

/// Settings for file I/O and additional color processing
pub struct PallyGenConfig {
    /// File output format. Default = `FileFormatType::PalUint8`
    pub file_format: FileFormatType,
    /// Include emphasis entries in output. Default = `true`
    pub render_emphasis: bool,
    /// Method for clipping out-of-range RGB colors. Default = `None`
    pub clip: Option<ClipType>,
    /// Method for scaling out-of-range RGB colors into gamut. Default = `None`
    pub normalize: Option<NormalizeType>,
}

impl PallyGenConfig {
    pub fn new() -> Self {
        Self {
            file_format: FileFormatType::PalUint8,
            render_emphasis: true,
            clip: None,
            normalize: None,
        }
    }
}

impl Default for PallyGenConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Converts the floating-point palette tuple vector into u8
/// 
/// Converting range of `0.0, 1.0` to `0, 255`.
pub fn palette_to_u8(palette: &[(f64, f64, f64)]) -> Vec<(u8, u8, u8)> {
    palette.iter().map(|color|
        color_to_u8(*color)
    ).collect()
}

/// Converts the floating-point palette tuple vector into u8
/// 
/// Converting range of `0.0, 1.0` to `0, 255`.
pub fn color_to_u8((r, g, b): (f64, f64, f64)) -> (u8, u8, u8) {
    (
        (r*255.0).round() as u8,
        (g*255.0).round() as u8,
        (b*255.0).round() as u8,
    )
}

/// Converts a given PPU color (in u16 form) into a composite signal.
pub fn pixel_to_cvbs(encoder: &NesPpuCvbs, pixel: u16, length: u8) -> Vec<f64> {
    encoder.encode_cvbs_pixel(
        &PpuPixel::Active(pixel.into()),
        &mut 0,
        length,
        false
    )
}

/// Decodes a given composite signal and colorburst
/// reference signal to a final 8bpc RGB value.
/// 
/// Returns an `(r, g, b)` `f64` tuple.
pub fn cvbs_to_rgb(
    cvbs: &[f64],
    cb: &[f64],
    config: &DecodeConfig
) -> (f64, f64, f64) {
    let (r, g, b) = yuv_to_rgb(
        decode_area(cvbs, cb, config)
    );

    // TODO: colorimetry, normalization, clipping

    let r = r.clamp(0.0, 1.0);
    let g = g.clamp(0.0, 1.0);
    let b = b.clamp(0.0, 1.0);

    (r, g, b)
}

/// Converts a given PPU pixel into a single 8bpc RGB color.
///
/// Returns an `(r, g, b)` `u8` tuple.
pub fn pixel_to_rgb(
    encoder: &NesPpuCvbs,
    decoder: &DecodeConfig,
    pixel: u16,
) -> (f64, f64, f64) {
    let length = 12;

    cvbs_to_rgb(
        &pixel_to_cvbs(encoder, pixel, length),
    &encoder.encode_cvbs_pixel(
            &PpuPixel::Colorburst(0x08),
            &mut 0,
            length,
            false
        ),
        decoder
    )
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_cvbs() {
        let encoder = NesPpuCvbs {
            lut: LvlCVBSTable::new_normalized(),
            ..Default::default()
        };

        let decoder = DecodeConfig::new();

        for hue in 0x20..=0x2F {
            print!("{} ", PpuColor::from(hue));
            let (r, g, b) = color_to_u8(
                pixel_to_rgb(&encoder, &decoder, hue)
            );
            println!("#{r:02X}{g:02X}{b:02X}")
        }
    }
}
