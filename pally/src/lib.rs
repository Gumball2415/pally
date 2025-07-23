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

    fn pix_to_cvbs(pixel: u16, length: u8) -> Vec<f64> {
        encode_cvbs_pixel(
            &PpuPixel::Active(pixel.into()),
            &mut 0,
            length,
            false
        )
    }

    #[test]
    fn decode_cvbs() {
        let length = 12;
        let config = DecodeConfig {
            ..cvbs_decode::DEFAULT_CFG
        };

        // colorburst
        let cb = encode_cvbs_pixel(
            &PpuPixel::Colorburst(0x08),
            &mut 0,
            length,
            false
        );

        for hue in 0..16 {
            print!("${hue:02X}: ");
            pixel_to_rgb(&pix_to_cvbs(hue, length), &cb, &config);
        }
    }
}
