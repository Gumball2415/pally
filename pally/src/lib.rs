use std::path::Path;
use std::error::Error;

mod generator;
mod fileio;
use crate::generator::*;
use nes_ppu_cvbs::*;
use cvbs_decode::*;


/// Runs the CLI
pub fn run_cli() -> Result<(), Box<dyn Error>> {
    // parse arguments

    // Get default black/white points?
    // construct encoder and decoder
    let pally = PallyGenConfig::new();

    // generate colors
    let palette = generate_colors(&pally);

    let path: &Path = Path::new("test.pal");
    // save colors
    save_colors(path, &palette)
}

/// Generates the palette entries.
/// 
/// Returns a vector of `(r, g, b)` `f64` tuples.
pub fn generate_colors(pally: &PallyGenConfig) -> Vec<(f64, f64, f64)> {

    // presumably we would get the black/whitepoints from the interface
    let encoder = LvlCVBSTable::new();
    let black = encoder.get_black();
    let white = encoder.get_white();

    let encoder = NesPpuCvbs {
        lut: LvlCVBSTable::new().normalize(white, black),
        ..Default::default()
    };

    let decoder = DecodeConfig::new();

    let max: u16 = if pally.render_emphasis {
        0b111_11_1111
    } else {
        0b000_11_1111
    };

    (0..=max).map(
        |hue| pixel_to_rgb(&encoder, &decoder, hue)
    ).collect()
}

/// Saves the generated palette to a file, with a provided path.
pub fn save_colors(path: &Path, palette: &[(f64, f64, f64)]) -> Result<(), Box<dyn Error>> {
    // TODO: switching between different outputs based on enum and trait?
    fileio::output_binary_uint8(path, palette)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    // TODO: tests
}