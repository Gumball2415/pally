//! Composite video decoder


pub struct DecodeConfig {
    black_point: f64,
    white_point: f64,
    brightness: f64,
    contrast: f64,
    hue: f64,
    saturation: f64,
    gain: f64,
    gamma: f64,
}

pub struct YUVColor {
    Y: f64,
    U: f64,
    V: f64,
}

impl YUVColor {
    /// Converts a signal YUV color to signal RGB
    /// via SMPTE 170M
    /// 
    /// Valid range for `Y`, `U`, and `V`: `0.0` to `1.0`
    fn to_rgb(&self) -> RGBColor {
        // coefficients taken from
        // https://www.nesdev.org/wiki/NTSC_video#Converting_YUV_to_signal_RGB
        RGBColor {
            R: self.Y + self.V*1.139883,
            G: self.Y - self.U*0.394642 + self.V*0.580622,
            B: self.Y + self.U*2.032062
        }
    }
}

pub struct RGBColor {
    R: f64,
    G: f64,
    B: f64,
}

/// Saturation
static SATURATION_CORRECTION: f64 = 2.0;

/// Given a sinusoidal signal, calculate its in-phase and quadrature phases.
fn qam_phase(signal: &[f64]) -> f64 {
    let len: f64 = signal.len() as f64;
    let u: f64 = signal.iter().enumerate().map(|(i, sample)| {
        sample * f64::sin(std::f64::consts::TAU * (i as f64) / 12.0)
    }).sum();

    let v: f64 = signal.iter().enumerate().map(|(i, sample)| {
        sample * f64::cos(std::f64::consts::TAU * (i as f64) / 12.0)
    }).sum();
    f64::atan2(v/len, u/len)
}

/// Decodes a given composite signal, assuming it is encoded from a single
/// patch of color.
/// 
/// A colorburst reference must also be provided for decoding.
/// 
/// Both input composite and colorburst reference signals must be of the same
/// length.
/// 
/// Returns a single `Y`, `U`, and `V` value.
pub fn decode_area(cvbs: &[f64], cb: &[f64], cfg: &DecodeConfig) -> YUVColor {

    // determine colorburst phase
    let cb_phase = qam_phase(cb);
    let signal_len = cvbs.len();

    // generate decoding waveforms
    let u_decode: Vec<f64> = (0..signal_len).into_iter()
        .map(|i|  {
            f64::sin(
                std::f64::consts::TAU * (i as f64) / 12.0
                - cb_phase
                + f64::to_radians(cfg.hue)
            ) * cfg.saturation * SATURATION_CORRECTION
        })
        .collect();

    let v_decode: Vec<f64> = (0..signal_len).into_iter()
        .map(|i|  {
            f64::cos(
                std::f64::consts::TAU * (i as f64) / 12.0
                - cb_phase
                + f64::to_radians(cfg.hue)
            ) * cfg.saturation * SATURATION_CORRECTION
        })
        .collect();

    // QAM decode!

    let y: f64 = cvbs
        .into_iter()
        .map(|sample| {
            sample / (signal_len as f64)
        }).sum();

    let u: f64 = cvbs
        .into_iter()
        .enumerate()
        .map(|(i, sample)| {
            u_decode[i] * sample / (signal_len as f64)
        }).sum();

    let v: f64 = cvbs
        .into_iter()
        .enumerate()
        .map(|(i, sample)| {
            v_decode[i] * sample / (signal_len as f64)
        }).sum();

    YUVColor{
        Y: y,
        U: u,
        V: v
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
