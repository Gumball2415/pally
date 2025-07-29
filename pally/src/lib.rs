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
    #[arg(value_enum, short, long, default_value_t = FileFormatType::PalUint8)]
    pub file_format: FileFormatType,

    /// Include emphasis entries in output.
    #[arg(short='e', long="emphasis")]
    pub render_emphasis: bool,

    /// Alternate method for clipping out-of-range RGB colors.
    #[arg(value_enum, long)]
    pub clip: Option<ClipType>,

    /// Alternate method for scaling out-of-range RGB colors into gamut.
    #[arg(value_enum, long)]
    pub normalize: Option<NormalizeType>,

    // Settings for adjusting decoding

    /// Black point, in IRE units.
    /// If not defined, will default to use the voltage level of `$1D`.
    #[arg(long)]
    pub black_point: Option<f64>,

    /// White point, in IRE units.
    /// If not defined, will default to use the voltage level of `$30`.
    #[arg(long)]
    pub white_point: Option<f64>,

    /// Luma brightness delta in IRE units.
    #[arg(short, long, default_value_t = 0.0)]
    pub brightness: f64,

    /// Luma contrast factor.
    #[arg(short, long, default_value_t = 1.0)]
    pub contrast: f64,

    /// Chroma hue angle delta, in degrees.
    #[arg(long, default_value_t = 0.0)]
    pub hue: f64,

    /// Chroma saturation factor.
    #[arg(short, long, default_value_t = 1.0)]
    pub saturation: f64,

    /// Gain adjustment to signal before decoding, in IRE units.
    #[arg(short, long, default_value_t = 0.0)]
    pub gain: f64,

    /// If defined, will apply a simple OETF gamma transfer function instead,
    /// where the EOTF function is assumed to be gamma 2.2.
    #[arg(long)]
    pub gamma: Option<f64>,

    /// Chooses what decoding to use. Not used in area-mode decoding.
    #[arg(value_enum, long, default_value_t = DecoderType::FIR)]
    pub decode_type: DecoderType,

    /// Settings for adjusting the encoding of signals

    /// PPU chip used for generating colors.
    #[arg(value_enum, long, default_value_t = PpuType::_2C02)]
    pub ppu: PpuType,

    /// Amount of voltage-dependent impedance for RC lowpass,
    /// where 'RC = amount * (level/composite_white) * 1e-8'.
    #[arg(short, long, default_value_t = 0.0)]
    pub phase_distortion: f64,
}


/// Runs the CLI
pub fn run_cli() -> Result<(), Box<dyn Error>> {
    // parse arguments
    let pally_cli = &PallyCli::parse();

    let pally = &parse_pally_config(pally_cli);
    let encoder = &parse_encoder(pally_cli);
    let decoder = &parse_decoder(pally_cli);

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



/// Grabs the relevant fields from the parser.
/// 
/// Returns a new `PallyGenConfig` encoder configuration.
fn parse_pally_config(cli: &PallyCli) -> PallyGenConfig {
    PallyGenConfig {
        file_format: cli.file_format,
        clip: cli.clip,
        normalize: cli.normalize,
        render_emphasis: cli.render_emphasis,
    }
}

/// Grabs the relevant fields from the parser.
/// 
/// Returns a new `NesPpuCvbs` encoder configuration.
fn parse_encoder(cli: &PallyCli) -> NesPpuCvbs {

    let mut lut = LvlCVBSTable::new();

    // Get
    if (None, None) == (cli.black_point, cli.white_point) {
        let white = lut.get_white();
        let black = lut.get_black();
        lut = lut.normalize(white, black);
    }

    let cfg: EncodeConfig = EncodeConfig {
        ppu: cli.ppu,
        phase_distortion: cli.phase_distortion,
    };

    NesPpuCvbs {
        lut,
        cfg,
    }
}

fn parse_decoder(cli: &PallyCli) -> DecodeConfig {
    DecodeConfig {
        black_point: cli.black_point,
        white_point: cli.white_point,
        brightness: cli.brightness,
        contrast: cli.contrast,
        hue: cli.hue,
        saturation: cli.saturation,
        gain: cli.gain,
        gamma: cli.gamma,
        decode_type: cli.decode_type,
    }
}

#[cfg(test)]
mod tests {
    // TODO: tests
}