use std::path::{Path, PathBuf};

use std::error::Error;

mod generator;
mod fileio;
use crate::generator::*;
use nes_ppu_cvbs::*;
use cvbs_decode::*;

use clap::Parser;

#[derive(Parser)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = env!("CARGO_PKG_DESCRIPTION"), long_about = None)]
struct PallyCli {
    /// File output. Format set by `--file-format`.
    pub file_output: PathBuf,

    // Settings for file I/O and additional color processing

    /// File output format. Default = `pal-uint8`
    #[arg(short, long)]
    pub file_format: Option<FileFormatType>,
    /// Include emphasis entries in output. Default = `true`
    #[arg(short='e', long="emphasis")]
    pub render_emphasis: Option<bool>,
    /// Method for clipping out-of-range RGB colors. Default = `None`
    #[arg(long)]
    pub clip: Option<ClipType>,
    /// Method for scaling out-of-range RGB colors into gamut. Default = `None`
    #[arg(long)]
    pub normalize: Option<NormalizeType>,

    // Settings for adjusting decoding

    /// Black point, in IRE units, default = `0.0`
    #[arg(long)]
    pub black_point: Option<f64>,
    /// White point, in IRE units, default = `100.0`
    #[arg(long)]
    pub white_point: Option<f64>,
    /// Luma brightness delta in IRE units, default = `0.0`
    #[arg(short, long)]
    pub brightness: Option<f64>,
    /// Luma contrast factor, default = `1.0`
    #[arg(short, long)]
    pub contrast: Option<f64>,
    /// Chroma hue angle delta, in degrees, default = `0.0`
    #[arg(long)]
    pub hue: Option<f64>,
    /// Chroma saturation factor, default = `1.0`
    #[arg(short, long)]
    pub saturation: Option<f64>,
    /// Gain adjustment to signal before decoding, in IRE units, default = `0.0`
    #[arg(short, long)]
    pub gain: Option<f64>,
    /// If nonzero, will apply a simple OETF gamma transfer function instead,
    /// where the EOTF function is assumed to be gamma 2.2. Default = `0.0`
    #[arg(long)]
    pub gamma: Option<f64>,
    /// Chooses what decoding to use. Not used in area-mode decoding.
    /// 
    /// Default = `fir`
    #[arg(long)]
    pub decode_type: Option<DecoderType>,

    /// Settings for adjusting the encoding of signals

    /// PPU chip used for generating colors.
    /// 
    /// Default = `2c02`
    #[arg(long)]
    pub ppu: Option<PpuType>,

    /// Amount of voltage-dependent impedance for RC lowpass,
    /// where 'RC = amount * (level/composite_white) * 1e-8'. 
    /// Default = `0.0`
    #[arg(short, long)]
    pub phase_distortion: Option<f64>,
}


/// Runs the CLI
pub fn run_cli() -> Result<(), Box<dyn Error>> {
    // parse arguments
    let pally_cli = PallyCli::parse();

    // Get default black/white points?
    // construct encoder and decoder
    let pally = &PallyGenConfig::new();

    // presumably we would get the black/whitepoints from the interface
    let encoder = LvlCVBSTable::new();
    let black = encoder.get_black();
    let white = encoder.get_white();

    let encoder = &NesPpuCvbs {
        lut: LvlCVBSTable::new().normalize(white, black),
        ..Default::default()
    };

    let decoder = &DecodeConfig::new();

    // generate colors
    let palette = generate_colors(pally, encoder, decoder);

    // save colors
    save_colors(&pally_cli.file_output, &palette)
}

/// Generates the palette entries.
/// 
/// Returns a vector of `(r, g, b)` `f64` tuples.
pub fn generate_colors(
    pally: &PallyGenConfig,
    encoder: &NesPpuCvbs,
    decoder: &DecodeConfig
) -> Vec<(f64, f64, f64)> {

    let max: u16 = if pally.render_emphasis {
        0b111_11_1111
    } else {
        0b000_11_1111
    };

    (0..=max).map(
        |hue| pixel_to_rgb(encoder, decoder, hue)
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