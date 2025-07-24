//! Simple composite video decoder

pub enum DecoderType {
    /// FIR non-complementary lowpass
    FIR,
    /// 2-line comb filtering. Also PAL delay line
    Comb2Line,
    /// 3-line comb filtering.
    Comb3Line,
}

/// Settings for adjusting decoding
pub struct DecodeConfig {
    /// Black point, in IRE units, default = `0.0`
    pub black_point: f64,
    /// White point, in IRE units, default = `100.0`
    pub white_point: f64,
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
    /// If nonzero, will apply a simple OETF gamma transfer function instead,
    /// where the EOTF function is assumed to be gamma 2.2. Default = `0.0`
    pub gamma: f64,
    /// Chooses what decoding to use. Not used in area-mode decoding.
    /// Default = `DecoderType::FIR`
    pub decode_type: DecoderType,
}

impl Default for DecodeConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl DecodeConfig {
    pub fn new() -> Self {
        Self {
            black_point: 0.0,
            white_point: 100.0,
            brightness: 0.0,
            contrast: 1.0,
            hue: 0.0,
            saturation: 1.0,
            gain: 0.0,
            gamma: 0.0,
            decode_type: DecoderType::FIR,
        }
    }
}

/// Rounds up a float to `n` decimal digits of precision.
pub fn round_up(f: f64, n: u32) -> f64 {
    let decimal = 10u32.pow(n) as f64;
    (f * decimal).round() / decimal
}

/// Converts a signal RGB color to signal YUV
/// via SMPTE 170M.
/// 
/// The conversion matrix is only accurate within 6 digits due to the precision
/// of the reduction factors, but this is fine because this is finer than the
/// final 8bpc precision of `1/255`, or `0.003922`.
/// 
/// Valid range for `R`, `G`, and `B`: `0.0` to `1.0`
/// 
/// Returns `(Y, U, V)` tuple.
pub fn rgb_to_yuv(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    // coefficients taken from
    // https://www.nesdev.org/wiki/NTSC_video#Converting_YUV_to_signal_RGB
    (
          r*0.299 + g*0.587 + b*0.114,
        (-r*0.299 - g*0.587 + b*0.886) * 0.492111,
        ( r*0.701 - g*0.587 - b*0.114) * 0.877283,
    )
}

/// Converts a signal YUV color to signal RGB
/// via SMPTE 170M.
/// 
/// The conversion matrix is only accurate within 6 digits due to the precision
/// of the reduction factors, but this is fine because this is finer than the
/// final 8bpc precision of `1/255`, or `0.003922`.
/// 
/// Valid range for `Y`, `U`, and `V`: `0.0` to `1.0`.
/// 
/// Returns `(R, G, B)` tuple.
pub fn yuv_to_rgb(y: f64, u: f64, v: f64) -> (f64, f64, f64) {
    // coefficients taken from
    // https://www.nesdev.org/wiki/NTSC_video#Converting_YUV_to_signal_RGB
    let r = y + v*1.139883;
    let b = y + u*2.032062;
    let g = (y - r*0.299 - b*0.114) / 0.587;
    ( r, g, b )
}

/// Saturation
static SATURATION_CORRECTION: f64 = 2.0;

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
    v.atan2(u)
}

/// Decodes a given composite signal, assuming it is encoded from a single
/// patch of color.
/// 
/// A colorburst reference must also be provided for decoding.
/// 
/// Both input composite and colorburst reference signals must be of the same
/// length.
/// 
/// Returns a `(y, u, v)` tuple.
pub fn decode_area(
    cvbs: &[f64],
    cb: &[f64],
    cfg: &DecodeConfig
) -> (f64, f64, f64) {
    // determine colorburst phase
    let cb_phase = qam_phase(cb);

    let signal_len = cvbs.len();

    // FIXME: it's a mystery why the phase is always offset like this
    // offset by 90 degrees + 30 degrees
    let phase_adjust = consts::FRAC_PI_2 + consts::FRAC_PI_6;

    // generate decoding waveforms
    let u_decode: Vec<f64> = (0..signal_len)
        .map(|i|  {
            f64::sin(
                consts::TAU * (i as f64) / 12.0
                - cb_phase
                + f64::to_radians(cfg.hue)
                - phase_adjust
            ) * cfg.saturation * SATURATION_CORRECTION
        })
        .collect();

    let v_decode: Vec<f64> = (0..signal_len)
        .map(|i|  {
            // TODO: investigate inverted V fix
            -(
                f64::cos(
                    consts::TAU * (i as f64) / 12.0
                    - cb_phase
                    + f64::to_radians(cfg.hue)
                    - phase_adjust
                ) * cfg.saturation * SATURATION_CORRECTION
            )
        })
        .collect();

    // QAM decode!
    let y: f64 = cvbs
        .iter()
        .map(|sample| {
            sample / (signal_len as f64)
        }).sum();

    let u: f64 = cvbs
        .iter()
        .enumerate()
        .map(|(i, sample)| {
            u_decode[i] * sample / (signal_len as f64)
        }).sum();

    let v: f64 = cvbs
        .iter()
        .enumerate()
        .map(|(i, sample)| {
            v_decode[i] * sample / (signal_len as f64)
        }).sum();

    (y, u, v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yuv_rgb_transform() {
        let (r, g, b) = (1.0, 0.000001, 0.5);
        let (y, u, v) = rgb_to_yuv(r, g, b);
        let (r2, g2, b2) = yuv_to_rgb(y, u, v);

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
