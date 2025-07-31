use std::path::PathBuf;

use std::error::Error;

mod generator;
mod fileio;
use crate::generator::*;
use crate::fileio::*;
use nes_ppu_cvbs::*;
use cvbs_decode::*;

use clap::Parser;

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

#[derive(Parser)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = env!("CARGO_PKG_DESCRIPTION"), long_about = None)]
pub struct PallyCli {
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
    #[arg(short, long, default_value_t = 4.0)]
    pub phase_distortion: f64,
}


/// Runs the CLI
pub fn run_cli() -> Result<(), Box<dyn Error>> {
    // parse arguments
    let pally_cli = &PallyCli::parse();

    let pally = &parse_pally_config(pally_cli);
    let (encoder, decoder) = &parse_encoder_decoder(pally_cli);

    // generate colors
    let palette = generate_colors(pally.render_emphasis, encoder, decoder);

    // save colors
    save_colors(pally_cli, &palette)
}

/// Generates the palette entries.
/// 
/// Returns a vector of `(r, g, b)` `f64` tuples.
pub fn generate_colors(
    render_emphasis: bool,
    encoder: &NesPpuCvbs,
    decoder: &DecodeConfig
) -> Vec<(f64, f64, f64)> {

    let max: u16 = if render_emphasis {
        0b111_11_1111
    } else {
        0b000_11_1111
    };

    (0..=max).map(
        |hue| pixel_to_rgb(encoder, decoder, hue)
    ).collect()
}

/// Saves the generated palette to a file, with a provided path.
pub fn save_colors(
    pallycli: &PallyCli,
    palette: &[(f64, f64, f64)]
) -> Result<(), Box<dyn Error>> {
    // TODO: switching between different outputs based on enum and trait?
    fileio::output_file(
        &pallycli.file_output,
        palette,
        pallycli.file_format
    )?;

    Ok(())
}



/// Grabs the relevant fields from the parser.
/// 
/// Returns a new `PallyGenConfig` encoder configuration.
fn parse_pally_config(pallycli: &PallyCli) -> PallyGenConfig {
    PallyGenConfig {
        file_format: pallycli.file_format,
        render_emphasis: pallycli.render_emphasis,
    }
}

/// Grabs the relevant fields from the parser.
/// 
/// Returns a new encoder and decoder configuration.
fn parse_encoder_decoder(pallycli: &PallyCli) -> (NesPpuCvbs, DecodeConfig) {
    let lut = LvlCVBSTable::new();

    // Get LUT's own black and white points if none is provided
    let black_point = pallycli.black_point.unwrap_or(0.0);
    let blank_point = lut.get_black() * 140.0;
    let white_point = pallycli.white_point.unwrap_or(
        (lut.get_white() * 140.0) - blank_point
    );

    (
        NesPpuCvbs {
            lut,
            cfg: EncodeConfig {
                ppu: pallycli.ppu,
                phase_distortion: pallycli.phase_distortion,
                ..Default::default()
            }.initialize_clock_freq(),
        },
        DecodeConfig {
            black_point,
            white_point,
            blank_point,
            brightness: pallycli.brightness,
            contrast: pallycli.contrast,
            hue: pallycli.hue,
            saturation: pallycli.saturation,
            gain: pallycli.gain,
            gamma: pallycli.gamma,
            decode_type: pallycli.decode_type,
            clip: pallycli.clip,
            normalize: pallycli.normalize,
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    // TODO: tests
    #[test]
    fn generate() {
            let pally = &PallyGenConfig::new();
            let encoder = &NesPpuCvbs::new().initialize_clock_freq();
            let decoder = &DecodeConfig::new();

            // generate colors
            generate_colors(pally.render_emphasis, encoder, decoder);
    }
}