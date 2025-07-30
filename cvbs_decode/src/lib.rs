//! Simple composite video decoder

use std::fmt;

use clap::ValueEnum;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum DecoderType {
    /// FIR non-complementary lowpass
    FIR,
    /// 2-line comb filtering. Also PAL delay line
    Comb2Line,
    /// 3-line comb filtering.
    Comb3Line,
}

impl fmt::Display for DecoderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Self::Comb2Line => "2-line",
            Self::Comb3Line => "3-line",
            Self::FIR => "FIR"
        })
    }
}

/// Method for clipping out-of-range RGB colors.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
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
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum NormalizeType {
    /// Scale all RGB values within 0.0 to 1.0.
    Scale,
    /// Clips all negative RGB values, then scales them within 0.0 to 1.0.
    ScaleClipNegative,
}

/// Settings for adjusting decoding
pub struct DecodeConfig {
    /// Black point, in IRE units, default = `0.0`
    pub black_point: f64,

    /// White point, in IRE units, default = `100.0`
    pub white_point: f64,

    /// Blank point, in IRE units, default = `7.5`
    pub blank_point: f64,

    /// Luma brightness delta in IRE units, default = `0.0`
    pub brightness: f64,

    /// Luma contrast factor, default = `1.0`
    pub contrast: f64,

    /// Chroma hue angle delta, in degrees, default = `0.0`
    pub hue: f64,

    /// Chroma saturation factor, default = `1.0`
    pub saturation: f64,

    /// Gain adjustment to signal before decoding, in IRE units, default = `0.0`
    pub gain: f64,

    /// If defined, will apply a simple OETF gamma transfer function with the
    /// specified gamma, where the EOTF function is assumed to be gamma 2.2.
    /// Default = `None`
    pub gamma: Option<f64>,

    /// Chooses what decoding to use. Not used in area-mode decoding.
    /// Default = `DecoderType::FIR`
    pub decode_type: DecoderType,

    /// Method for clipping out-of-range RGB colors. Default = `None`
    pub clip: Option<ClipType>,

    /// Method for scaling out-of-range RGB colors into gamut. Default = `None`
    pub normalize: Option<NormalizeType>,
}

impl DecodeConfig {
    pub fn new() -> Self {
        Self {
            black_point: 0.0,
            white_point: 100.0,
            blank_point: 7.5,
            brightness: 0.0,
            contrast: 1.0,
            hue: 0.0,
            saturation: 1.0,
            gain: 0.0,
            gamma: None,
            decode_type: DecoderType::FIR,
            clip: None,
            normalize: None,
        }
    }
}

impl Default for DecodeConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Rounds up a float to `n` decimal digits of precision.
pub fn round_up(f: f64, n: u32) -> f64 {
    let decimal = 10u32.pow(n) as f64;
    (f * decimal).round() / decimal
}

/// Converts a signal RGB color to signal YUV via SMPTE 170M.
/// 
/// The conversion matrix is only accurate within 6 digits due to the precision
/// of the reduction factors, but this is fine because this is finer than the
/// final 8bpc precision of `1/255`, or `0.003922`.
/// 
/// Valid range for `r`, `g`, and `b`: `0.0` to `1.0`
/// 
/// Returns a `(y, u, v)` `u8` tuple.
pub fn rgb_to_yuv((r, g, b): (f64, f64, f64)) -> (f64, f64, f64) {
    // coefficients taken from
    // https://www.nesdev.org/wiki/NTSC_video#Converting_YUV_to_signal_RGB
    (
          r*0.299 + g*0.587 + b*0.114,
        (-r*0.299 - g*0.587 + b*0.886) * 0.492111,
        ( r*0.701 - g*0.587 - b*0.114) * 0.877283,
    )
}

/// Converts a signal YUV color to signal RGB via SMPTE 170M.
/// 
/// The conversion matrix is only accurate within 6 digits due to the precision
/// of the reduction factors, but this is fine because this is finer than the
/// final 8bpc precision of `1/255`, or `0.003922`.
/// 
/// Valid range for `y`, `u`, and `v`: `0.0` to `1.0`.
/// 
/// Returns an `(r, g, b)` `u8` tuple.
pub fn yuv_to_rgb((y, u, v): (f64, f64, f64)) -> (f64, f64, f64) {
    // coefficients taken from
    // https://www.nesdev.org/wiki/NTSC_video#Converting_YUV_to_signal_RGB
    let r = y + v*1.139883;
    let b = y + u*2.032062;
    let g = (y - r*0.299 - b*0.114) / 0.587;
    ( r, g, b )
}

/// Saturation
const SATURATION_CORRECTION: f64 = 2.0;

/// Decodes a given composite signal, assuming it is encoded from a single
/// patch of color.
/// 
/// A colorburst reference must also be provided for decoding.
/// 
/// Both input composite and colorburst reference signals must be of the same
/// length.
/// 
/// Returns a raw `(y, u, v)` `f64` tuple, in IRE.
/// 
/// # Panics
/// 
/// This function will panic if `cvbs` and `cb` are not of the same length.
pub fn decode_area(
    cvbs: &[f64],
    cb: &[f64],
    cfg: &DecodeConfig
) -> (f64, f64, f64) {
    assert_eq!(
        cvbs.len(), cb.len(),
        "cvbs ({}) and cb ({}) are not of equal size",
        cvbs.len(), cb.len(),
    );

    // determine colorburst phase
    let cb_phase = qam_phase(cb);

    let signal_len = cvbs.len();

    // FIXME: it's a mystery why the phase is always offset like this
    let phase_adjust = -consts::FRAC_PI_2;

    // generate decoding waveforms
    let u_decode: Vec<f64> = (0..signal_len)
        .map(|i|  {
            f64::sin(
                consts::TAU * (i as f64) / 12.0
                - cb_phase
                - phase_adjust
                + f64::to_radians(cfg.hue)
            ) * cfg.saturation * SATURATION_CORRECTION
        })
        .collect();

    let v_decode: Vec<f64> = (0..signal_len)
        .map(|i|  {
            f64::cos(
                consts::TAU * (i as f64) / 12.0
                - cb_phase
                - phase_adjust
                + f64::to_radians(cfg.hue)
            ) * cfg.saturation * SATURATION_CORRECTION
        })
        .collect();

    // FIXME: need to do this better
    let mut signal: Vec<f64> = cvbs.to_vec();

    // convert to IRE
    signal = signal.iter().map(|x| x * 140.0).collect();

    // blank = 0.0
    signal = signal.iter().map(|x| x - cfg.blank_point).collect();

    // apply gain
    signal = signal.iter().map(|x| x + cfg.gain).collect();

    // decode!
    let y: f64 = signal
        .iter()
        .map(|sample| {
            sample / (signal_len as f64)
        }).sum();

    // apply brightnes and contrast
    let y = y * cfg.contrast + cfg.brightness;

    let u: f64 = signal
        .iter()
        .enumerate()
        .map(|(i, sample)| {
            u_decode[i] * sample / (signal_len as f64)
        }).sum();

    let v: f64 = signal
        .iter()
        .enumerate()
        .map(|(i, sample)| {
            v_decode[i] * sample / (signal_len as f64)
        }).sum();

    (y, u, v)
}

/// Given a three-channeled signal and min/max value points, scale the values,
/// such that `min = 0.0`, and `max == 1.0`
pub fn normalize_color(
    (r, g, b): (f64, f64, f64),
    min: f64,
    max: f64,
) -> (f64, f64, f64) {
    (
        normalize_channel(r, min, max),
        normalize_channel(g, min, max),
        normalize_channel(b, min, max),
    )
}

/// Scale the signal such that `min = 0.0`, and `max == 1.0`
fn normalize_channel(
    c: f64,
    min: f64,
    max: f64,
) -> f64 {
    (c - min) / (max - min)
}

/// Apply clipping and renormalization, if defined.
pub fn clip_normalize_colors(
    (r, g, b): (f64, f64, f64),
    cfg: &DecodeConfig,
) -> (f64, f64, f64) {
    let (r, g, b) = if let Some(clip) = cfg.clip {
        // clip takes priority over normalize
        match clip {
            ClipType::Darken => color_clip_darken((r, g, b)),
            ClipType::Desaturate => color_clip_desaturate((r, g ,b))
        }
    }
    else if let Some(norm) = cfg.normalize {
            let (min, max) = match norm {
                NormalizeType::ScaleClipNegative => (0.0, r.max(g.max(b))),
                NormalizeType::Scale => (r.min(g.min(b)), r.max(g.max(b))),
            };
            normalize_color((r, g, b), min, max)
    } else {
        (r, g, b)
    };
    clip_color((r, g, b), 0.0, 1.0)
}

/// Helper function for `f64::clamp()` for a color tuple.
fn clip_color(
    (r, g, b): (f64, f64, f64),
    min: f64,
    max: f64,
) -> (f64, f64, f64) {
    (
        r.clamp(min, max),
        g.clamp(min, max),
        b.clamp(min, max),
    )
}

/// Algorithm by DragWx.
/// 
/// If any of the RGB channels are greater than 1, subtract all channels by
/// delta of greatest channel
fn color_clip_darken(
    (r, g, b): (f64, f64, f64),
) -> (f64, f64, f64) {
    let darken_factor = r.max(g.max(b));
    normalize_color((r, g, b), 0.0, darken_factor)
}


/// Algorithm by DragWx.
/// 
/// If any of the RGB channels are greater than 1, desaturate until all channels
/// are within range.
fn color_clip_desaturate(
    (r, g, b): (f64, f64, f64),
) -> (f64, f64, f64) {
    let darken_factor = r.max(g.max(b));
    let (y, _, _) = rgb_to_yuv((r, g, b));
    let (r, g, b) = (r-y, g-y, b-y);
    let (r, g, b) = (r/darken_factor, g/darken_factor, b/darken_factor);
    (r+y, g+y, b+y)
}

use std::f64::consts;

/// Given a sinusoidal signal, calculate its in-phase and quadrature phases.
fn qam_phase(signal: &[f64]) -> f64 {
    let len: f64 = signal.len() as f64;
    let u: f64 = signal.iter().enumerate().map(|(i, sample)| {
        sample * f64::sin(consts::TAU * (i as f64) / 12.0) / len
    }).sum();

    let v: f64 = signal.iter().enumerate().map(|(i, sample)| {
        sample * f64::cos(consts::TAU * (i as f64) / 12.0) / len
    }).sum();
    u.atan2(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yuv_rgb_transform() {
        let (r, g, b) = (1.0, 0.000001, 0.5);
        let (r2, g2, b2) = yuv_to_rgb(rgb_to_yuv((r, g, b)));

        // The results should remain exact within 6 digits of precision.

        let r2 = round_up(r2, 6);
        let g2 = round_up(g2, 6);
        let b2 = round_up(b2, 6);
        let r = round_up(r, 6);
        let g = round_up(g, 6);
        let b = round_up(b, 6);

        assert_eq!((r, g, b), (r2, g2, b2));
    }
}
