//! Responsible for interacting with the composite encoder and decoder, and
//! returning the final encoded color.

use nes_ppu_cvbs::*;
use cvbs_decode::*;

use std::fmt;
use clap::ValueEnum;

/// File output format.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
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

/// Needed for default clap setting
impl fmt::Display for FileFormatType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Self::PalUint8 => ".pal uint8",
            Self::PalDouble => ".pal double",
            Self::PalJasc => ".pal Jasc",
            Self::Gpl => ".gpl",
            Self::Png => ".png",
            Self::TxtHtmlHex => ".txt HTML hex",
            Self::TxtMediaWiki => ".txt MediaWiki",
            Self::HeaderUint8T => ".h uint8_t",
        })
    }
}

/// Settings for file I/O and additional color processing
pub struct PallyGenConfig {
    /// File output format. Default = `FileFormatType::PalUint8`
    pub file_format: FileFormatType,
    /// Include emphasis entries in output. Default = `true`
    pub render_emphasis: bool,
}

impl PallyGenConfig {
    pub fn new() -> Self {
        Self {
            file_format: FileFormatType::PalUint8,
            render_emphasis: true,
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
    cfg: &DecodeConfig
) -> (f64, f64, f64) {
    clip_normalize_colors(
        normalize_color(
            yuv_to_rgb(
                decode_area(cvbs, cb, cfg)
            ),
            cfg.black_point, cfg.white_point
        ), cfg
    )
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
