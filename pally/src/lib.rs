
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

pub enum ClipType {
    Darken,
    Desaturate,
}

pub enum NormalizeType {
    Scale,
    ScaleClipNegative,
}

/// Settings for file I/O and additional color processing
pub struct PallyConfig {
    /// File output format. Default = `FileFormatType::PalUint8`
    pub file_format: FileFormatType,
    /// Include emphasis entries in output. Default = `true`
    pub render_emphasis: bool,
    /// Method for clipping out-of-range RGB colors. Default = `None`
    pub clip: Option<ClipType>,
    /// Method for scaling out-of-range RGB colors into gamut. Default = `None`
    pub normalize: Option<NormalizeType>,
}

impl Default for PallyConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl PallyConfig {
    pub fn new() -> Self {
        Self {
            file_format: FileFormatType::PalUint8,
            render_emphasis: true,
            clip: None,
            normalize: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use nes_ppu_cvbs::*;
    use cvbs_decode::*;

    fn pixel_to_rgb(cvbs: &[f64], cb: &[f64], config: &DecodeConfig) {
        let (y, u, v) = decode_area(&cvbs, &cb, &config);
        let (r, g, b) = yuv_to_rgb(y, u, v);
        let r = (r.clamp(0.0, 1.0)*255.0).round() as u8;
        let g = (g.clamp(0.0, 1.0)*255.0).round() as u8;
        let b = (b.clamp(0.0, 1.0)*255.0).round() as u8;
        println!("#{b:02X}{g:02X}{r:02X}")
    }

    fn pix_to_cvbs(encoder: &NesPpuCvbs, pixel: u16, length: u8) -> Vec<f64> {
        encoder.encode_cvbs_pixel(
            &PpuPixel::Active(pixel.into()),
            &mut 0,
            length,
            false
        )
    }

    #[test]
    fn decode_cvbs() {
        let mut encoder = NesPpuCvbs::new();
        let decoder = DecodeConfig::new();
        let length = 12;

        let black_point = encoder.lut.get_black();
        let white_point = encoder.lut.get_white();
        encoder.lut = encoder.lut.normalize(white_point, black_point);

        // colorburst
        let cb = encoder.encode_cvbs_pixel(
            &PpuPixel::Colorburst(0x08),
            &mut 0,
            length,
            false
        );

        for hue in 0x20..=0x2F {
            print!("${hue:02X}: ");
            pixel_to_rgb(
                &pix_to_cvbs(&encoder, hue, length),
                &cb,
                &decoder
            );
        }
    }
}
