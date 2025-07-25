//! # NES PPU composite video encoder
//!
//! `nes_ppu_cvbs` handles the encoding of a given PPU pixel to its composite
//! counterpart, given a 9-bit PPU pixel and a phase value.
//!
//! For more information on the terminology used for representing the NES PPU
//! colors, visit:
//! <https://www.nesdev.org/wiki/PPU_palettes#Color_Value_Significance_(Hue_/_Value)>

use clap::ValueEnum;

use std::fmt;

/// Represents the 9-bit framebuffer pixel value used in most emulators.
/// 
/// Alongside the color index, the emphasis flags are included as follows:
///
/// ```txt
/// bgr vv hhhh
/// ||| || ++++-- Hue phase column.
/// ||| ++------- Value row.
/// +++---------- Red, green, and blue PPUMASK emphasis bits.
/// ```
/// 
/// Note:
/// 
/// - It is up to the emulator to swizzle between red and green emphasis
///   tint bits for PAL/Dendy PPUs.
/// - Sometimes Hue and Value (from
///   [HSL](https://en.wikipedia.org/wiki/HSL_and_HSV) terminology) is also
///   referred to as Chroma and Luma.
#[derive(Debug)]
pub struct PpuColor (u16);

impl PpuColor {
    fn get_no_emphasis(&self) -> u8 {
        (self.0 & 0b000_11_1111) as u8
    }
    fn get_hue(&self) -> u8 {
        (self.0 & 0b000_00_1111) as u8
    }
    fn get_value(&self) -> u8 {
        ((self.0 & 0b000_11_0000) >> 4) as u8
    }
    fn get_emphasis(&self) -> u8 {
        ((self.0 & 0b111_00_0000) >> 6) as u8
    }
}

impl From<u16> for PpuColor {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl fmt::Display for PpuColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "${:02X} {:03b}", self.get_no_emphasis(), self.get_emphasis())
    }
}

/// Represents all the types of pixels in a given composite video signal.
pub enum PpuPixel {
    Active(PpuColor),
    /// Sync voltage level.
    Sync,
    /// Blanking voltage level.
    Blank,
    /// Colorburst voltage levels, with a given chroma phase.
    Colorburst(u8),
}

impl fmt::Display for PpuPixel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Active(color) => color.fmt(f),
            Self::Blank => write!(f, "blank"),
            Self::Sync => write!(f, "sync"),
            Self::Colorburst(phase) =>
                write!(f, "colorburst with phase {phase:02X}", )
        }
    }
}

/// Holds the attenuated and non-attenuated signal of a given NES PPU composite
/// level.
#[derive(Debug)]
struct LvlEmph {
    /// "Nominal": No attenuation from any emphasis bits.
    n: f64,
    /// "Emphasis": Attenuated signal level from any given emphasis bits.
    e: f64,
}


/// Normalizes a given value to a range `0.0` to `1.0`, given the min and max
/// points.
/// 
/// This does not perform any clipping within the range.
pub fn normalize(val: f64, max: f64, min: f64) -> f64 {
    (val-min) / (max-min)
}

impl LvlEmph {
    fn select_level(&self, attenuate: bool) -> f64 {
        if attenuate { self.e } else { self.n }
    }

    fn normalize(self, white_point: f64, black_point: f64) -> Self {
        LvlEmph {
            n: normalize(self.n, white_point, black_point),
            e: normalize(self.e, white_point, black_point),
        }
    }
}

/// Holds two levels of a given chroma signal waveform.
#[derive(Debug)]
struct LvlAmp {
    /// High voltage level `$x0`.
    _0: LvlEmph,
    /// Low voltage level `$xD`.
    _d: LvlEmph,
}

impl LvlAmp {
    fn select_level(&self, high: bool) -> &LvlEmph {
        if high { &self._0 } else { &self._d }
    }

    fn normalize(self, white_point: f64, black_point: f64) -> Self {
        LvlAmp {
            _0: self._0.normalize(white_point, black_point),
            _d: self._d.normalize(white_point, black_point)
        }
    }
}

/// Holds the signal lookup table for all possible types of `PpuPixel`s.
#[derive(Debug)]
pub struct LvlCVBSTable {
    /// Signal levels of all colors in the `$0x` row.
    s_0: LvlAmp,
    /// Signal levels of all colors in the `$1x` row.
    s_1: LvlAmp,
    /// Signal levels of all colors in the `$2x` row.
    s_2: LvlAmp,
    /// Signal levels of all colors in the `$3x` row.
    s_3: LvlAmp,
    /// Signal levels of the colorburst.
    s_cb: LvlAmp,
    /// Signal levels of sync and blanking.
    s_bl: LvlAmp,
}

impl Default for LvlCVBSTable {
    fn default() -> Self {
        Self::new()
    }
}

impl LvlCVBSTable {
    /// Returns the default values of the voltage lookup table
    ///
    /// Voltages taken
    /// from <https://forums.nesdev.org/viewtopic.php?p=159266#p159266>
    ///
    /// $0x-$3x, $x0/$xD, no emphasis/emphasis
    pub fn new() -> Self {
        Self {
            s_0: LvlAmp {
                _0: LvlEmph { n: 0.616, e: 0.500 },
                _d: LvlEmph { n: 0.228, e: 0.192 },
            },
            s_1: LvlAmp {
                _0: LvlEmph { n: 0.840, e: 0.676 },
                _d: LvlEmph { n: 0.312, e: 0.256 },
            },
            s_2: LvlAmp {
                _0: LvlEmph { n: 1.100, e: 0.896 },
                _d: LvlEmph { n: 0.552, e: 0.448 },
            },
            s_3: LvlAmp {
                _0: LvlEmph { n: 1.100, e: 0.896 },
                _d: LvlEmph { n: 0.880, e: 0.712 },
            },
            // colorburst high, colorburst low
            s_cb: LvlAmp {
                _0: LvlEmph { n: 0.524, e: 0.524 },
                _d: LvlEmph { n: 0.148, e: 0.148 },
            },
            // blank level, sync level
            s_bl: LvlAmp {
                _0: LvlEmph { n: 0.312, e: 0.312 },
                _d: LvlEmph { n: 0.048, e: 0.048 },
            },
        }
    }

    /// Returns a pre-normalized lookup table, with the black point set at level
    /// `$1D`, and the white point set at level `$30`.
    ///
    /// Voltages taken
    /// from <https://forums.nesdev.org/viewtopic.php?p=159266#p159266>
    ///
    /// $0x-$3x, $x0/$xD, no emphasis/emphasis
    pub fn new_normalized() -> Self {
        let new = Self::new();
        let black_point = new.get_black();
        let white_point = new.get_white();
        new.normalize(white_point, black_point)
    }

    /// Normalizes the signal lookup table, given a black point and a
    /// white point.
    /// 
    /// This skips the signal normalization when performance is critical.
    pub fn normalize(self, white_point: f64, black_point: f64) -> Self {
        LvlCVBSTable {
            s_0: self.s_0.normalize(white_point, black_point),
            s_1: self.s_1.normalize(white_point, black_point),
            s_2: self.s_2.normalize(white_point, black_point),
            s_3: self.s_3.normalize(white_point, black_point),
            s_cb: self.s_cb.normalize(white_point, black_point),
            s_bl: self.s_bl.normalize(white_point, black_point)
        }
    }

    /// Blank/black level of the composite signal. Used for brightness
    /// normalization functions.
    pub fn get_black(&self) -> f64 {
        self.s_1._d.n
    }

    /// White level of the composite signal. Used for brightness normalization
    /// functions.
    pub fn get_white(&self) -> f64 {
        self.s_3._0.n
    }

    fn select_level(&self, value: u8) -> &LvlAmp {
        match value {
            0 => &self.s_0,
            1 => &self.s_1,
            2 => &self.s_2,
            3 => &self.s_3,
            invalid => panic!("not a valid value: {invalid}")
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum PpuType {
    _2C02,
    _2C07,
}

/// Settings for adjusting the encoding of signals
pub struct EncodeConfig {
    /// PPU chip used for generating colors. Default = `PpuType::_2C02`
    pub ppu: PpuType,
    
    /// Amount of voltage-dependent impedance for RC lowpass,
    /// where RC = `amount * (level/composite_white) * 1e-8`.
    /// 
    /// This will also desaturate and hue shift the resulting colors
    /// nonlinearly. a value of 4 very roughly corresponds to a -5 degree delta
    /// per luma row.
    /// 
    /// Default = `0.0`
    pub phase_distortion: f64,
}

impl Default for EncodeConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl EncodeConfig {
    pub fn new() -> Self {
        Self {
            ppu: PpuType::_2C02,
            phase_distortion: 0.0,
        }
    }
}

/// Holds the signal lookup table and the encoder configurations, as well as the
/// methods for encoding a PPU pixel to a composite signal.
pub struct NesPpuCvbs {
    pub lut: LvlCVBSTable,
    pub cfg: EncodeConfig,
}

impl Default for NesPpuCvbs {
    fn default() -> Self {
        Self::new()
    }
}

impl NesPpuCvbs {
    pub fn new() -> Self {
        Self {
            lut: LvlCVBSTable::new(),
            cfg: EncodeConfig::new(),
        }
    }

    /// Encodes a given `PpuPixel` into the current composite video sample.
    /// 
    /// Must also include the current sample phase and the colorburst hue.
    /// 
    /// This calls into the respective associated functions to return a sample
    /// value.
    pub fn encode_cvbs_sample(
        &self,
        pixel: &PpuPixel,
        sample_phase: u8,
        alternate_line: bool
    ) -> f64 {
        match pixel {
            PpuPixel::Sync =>
                self.lut.s_bl._d.n,
            PpuPixel::Blank =>
                self.lut.s_bl._0.n,
            PpuPixel::Colorburst(cburst_hue) => {
                self.lut.s_cb.select_level(
                    // subcarrier generation is 180 degrees offset
                    !color_phase(*cburst_hue, sample_phase, alternate_line)
                ).select_level(false)
            },
            PpuPixel::Active(color) => {
                let hue = color.get_hue();
                // Colors `$xE-$xF` always remain blanking
                if hue > 0xD {
                    self.lut.get_black()
                } else {
                    let value = color.get_value();
                    let emphasis = color.get_emphasis();
                    let in_phase = color_phase(hue, sample_phase, alternate_line);
                    let attenuate = attenuate(hue, emphasis, sample_phase, alternate_line);
                    self.lut
                        .select_level(value)
                        .select_level(in_phase)
                        .select_level(attenuate)
                }
            }
        }
    }

    /// Encodes a given PPU pixel with a given signal length.
    /// 
    /// Returns a vector of samples. `sample_phase` will be updated to
    /// reflect the new phase after the pixel.
    pub fn encode_cvbs_pixel(
        &self,
        pixel: &PpuPixel,
        sample_phase: &mut u8,
        length: u8,
        alternate_line: bool
    ) -> Vec<f64> {
        let mut output: Vec<f64> = Vec::new();
        for phase in 0..length {
            output.push(
                self.encode_cvbs_sample(pixel, (*sample_phase + phase)%12, alternate_line));
        }
        *sample_phase = (*sample_phase + length) % 12;
        output
    }
}

/// Precalculates the emphasis phase for further processing.
fn attenuate(
    hue: u8,
    emphasis: u8,
    sample_phase: u8,
    alternate_line: bool
) -> bool {
    let r = (emphasis & 0b001 != 0)
        && color_phase(0xC, sample_phase, alternate_line);
    let g = (emphasis & 0b010 != 0)
        && color_phase(0x4, sample_phase, alternate_line);
    let b = (emphasis & 0b100 != 0)
        && color_phase(0x8, sample_phase, alternate_line);
    (r || g || b)
    // Colors `$xE-$xF` are not affected by emphasis.
    && (hue < 0xE)
}

/// Original algorithm by Bisqwit.
/// 
/// Determines the waveform level by comparing the hue with the current
/// sample phase.
/// 
/// If the hue value is not chromatic (within `0x0..0xC`), it returns a constant
/// wave value.
/// 
/// If true, the phase is at the high part. Else, it is within the low part.
/// 
/// Valid hue range: `0x0..0xF`\
/// Valid chromatic phase range: `0..11`
fn color_phase(
    hue: u8,
    sample_phase: u8,
    alternate_line: bool) -> bool {
    match pal_phase(hue, alternate_line) {
        0x0 => true,
        0x1..=0xC => (hue + sample_phase) % 12 >= 6,
        0xD..=0xF => false,
        invalid => panic!("not a valid hue value: {invalid}")
    }
}

/// Alternates the V phase of a given hue in the same manner as 2C07s.
/// 
/// If not a chromatic hue, it returns the same hue value.
/// 
/// Valid hue range: `0x0..0xF`\
/// Valid chromatic phase range: `0..11`
fn pal_phase(hue: u8, alternate_line: bool) -> u8 {
    static ALT_PHASE: [u8; 12] = [
        4, 3, 2, 1,
        12, 11, 10, 9,
        8, 7, 6, 5,
    ];

    if (1..=12).contains(&hue) && alternate_line {
        ALT_PHASE[(hue-1) as usize]
    } else {
        hue
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Given an expected waveform `Vec<f64>`, a `PpuPixel` to be encoded, and a
    /// length, test if the encoding works as expected.
    fn encode_cvbs(
        encoder: &NesPpuCvbs,
        expected: &Vec<f64>,
        color: PpuPixel,
        length: u8
    ) {
        let mut sample_phase: u8 = 0;
        let out = encoder.encode_cvbs_pixel(&color, &mut sample_phase, length, false);

        assert_eq!(*expected, out);
        assert_eq!((length % 12), sample_phase);
    }

    #[test]
    /// Test encoding of a given color `$18`
    fn encode_cvbs_color() {
        let encoder: NesPpuCvbs = NesPpuCvbs::new();
        let length= 24;
        let color = PpuPixel::Active(PpuColor(0b000_01_1000));
        let sig_hi = encoder.lut.s_1._0.n;
        let sig_lo = encoder.lut.s_1._d.n;

        // color $01
        let expected = vec![
            sig_hi, sig_hi, sig_hi, sig_hi, sig_lo, sig_lo,
            sig_lo, sig_lo, sig_lo, sig_lo, sig_hi, sig_hi,
            sig_hi, sig_hi, sig_hi, sig_hi, sig_lo, sig_lo,
            sig_lo, sig_lo, sig_lo, sig_lo, sig_hi, sig_hi,
        ];

        encode_cvbs(&encoder, &expected, color, length);
    }

    #[test]
    /// Test encoding of a given color `$18`, with emphasis red
    fn encode_cvbs_emphasis_color() {
        let encoder: NesPpuCvbs = NesPpuCvbs::new();
        let length= 24;
        let color = PpuPixel::Active(PpuColor(0b001_01_1000));
        let sig_hi = encoder.lut.s_1._0.n;
        let sig_lo = encoder.lut.s_1._d.n;

        let sig_he = encoder.lut.s_1._0.e;
        let sig_le = encoder.lut.s_1._d.e;

        // color $38, red emphasis
        let expected = vec![
            sig_hi, sig_hi, sig_hi, sig_hi, sig_lo, sig_lo,
            sig_le, sig_le, sig_le, sig_le, sig_he, sig_he,
            sig_hi, sig_hi, sig_hi, sig_hi, sig_lo, sig_lo,
            sig_le, sig_le, sig_le, sig_le, sig_he, sig_he,
        ];

        encode_cvbs(&encoder, &expected, color, length);
    }

    #[test]
    fn encode_cvbs_emphasis_0() {
        let encoder: NesPpuCvbs = NesPpuCvbs::new();
        let length= 24;
        let color = PpuPixel::Active(PpuColor(0b001_00_0000));
        let sig_hi = encoder.lut.s_0._0.n;
        let sig_lo = encoder.lut.s_0._0.e;
        let expected = vec![
            sig_hi, sig_hi, sig_hi, sig_hi, sig_hi, sig_hi,
            sig_lo, sig_lo, sig_lo, sig_lo, sig_lo, sig_lo,
            sig_hi, sig_hi, sig_hi, sig_hi, sig_hi, sig_hi,
            sig_lo, sig_lo, sig_lo, sig_lo, sig_lo, sig_lo,
        ];
        encode_cvbs(&encoder, &expected, color, length);
    }

    #[test]
    fn encode_cvbs_emphasis_d() {
        let encoder: NesPpuCvbs = NesPpuCvbs::new();
        let length= 24;
        let color = PpuPixel::Active(PpuColor(0b001_00_1101));
        let sig_hi = encoder.lut.s_0._d.n;
        let sig_lo = encoder.lut.s_0._d.e;
        let expected = vec![
            sig_hi, sig_hi, sig_hi, sig_hi, sig_hi, sig_hi,
            sig_lo, sig_lo, sig_lo, sig_lo, sig_lo, sig_lo,
            sig_hi, sig_hi, sig_hi, sig_hi, sig_hi, sig_hi,
            sig_lo, sig_lo, sig_lo, sig_lo, sig_lo, sig_lo,
        ];
        encode_cvbs(&encoder, &expected, color, length);
    }

    #[test]
    fn encode_cvbs_emphasis_f() {
        let encoder: NesPpuCvbs = NesPpuCvbs::new();
        let black = encoder.lut.get_black();
        let length= 24;
        let color = PpuPixel::Active(PpuColor(0b001_00_1111));
        let expected = vec![
            black, black, black, black,
            black, black, black, black,
            black, black, black, black,
            black, black, black, black,
            black, black, black, black,
            black, black, black, black,
        ];
        encode_cvbs(&encoder, &expected, color, length);
    }

    #[test]
    fn encode_cvbs_emphasis_red() {
        let encoder: NesPpuCvbs = NesPpuCvbs::new();
        let length= 24;
        let color = PpuPixel::Active(PpuColor(0b001_01_0000));
        let sig_hi = encoder.lut.s_1._0.n;
        let sig_lo = encoder.lut.s_1._0.e;
        let expected = vec![
            sig_hi, sig_hi, sig_hi, sig_hi, sig_hi, sig_hi,
            sig_lo, sig_lo, sig_lo, sig_lo, sig_lo, sig_lo,
            sig_hi, sig_hi, sig_hi, sig_hi, sig_hi, sig_hi,
            sig_lo, sig_lo, sig_lo, sig_lo, sig_lo, sig_lo,
        ];
        encode_cvbs(&encoder, &expected, color, length);
    }

    #[test]
    fn encode_cvbs_emphasis_green() {
        let encoder: NesPpuCvbs = NesPpuCvbs::new();
        let length= 24;
        let color = PpuPixel::Active(PpuColor(0b010_01_0000));
        let sig_hi = encoder.lut.s_1._0.n;
        let sig_lo = encoder.lut.s_1._0.e;
        let expected = vec![
            sig_hi, sig_hi, sig_lo, sig_lo, sig_lo, sig_lo,
            sig_lo, sig_lo, sig_hi, sig_hi, sig_hi, sig_hi,
            sig_hi, sig_hi, sig_lo, sig_lo, sig_lo, sig_lo,
            sig_lo, sig_lo, sig_hi, sig_hi, sig_hi, sig_hi,
        ];
        encode_cvbs(&encoder, &expected, color, length);
    }

    #[test]
    fn encode_cvbs_emphasis_blue() {
        let encoder: NesPpuCvbs = NesPpuCvbs::new();
        let length= 24;
        let color = PpuPixel::Active(PpuColor(0b100_01_0000));
        let sig_hi = encoder.lut.s_1._0.n;
        let sig_lo = encoder.lut.s_1._0.e;
        let expected = vec![
            sig_lo, sig_lo, sig_lo, sig_lo, sig_hi, sig_hi,
            sig_hi, sig_hi, sig_hi, sig_hi, sig_lo, sig_lo,
            sig_lo, sig_lo, sig_lo, sig_lo, sig_hi, sig_hi,
            sig_hi, sig_hi, sig_hi, sig_hi, sig_lo, sig_lo,
        ];
        encode_cvbs(&encoder, &expected, color, length);
    }

    #[test]
    fn deconstruct_ppu_color() {
        let pix = PpuColor(0b100_10_1000);
        assert_eq!(0b1000, pix.get_hue());
        assert_eq!(0b100, pix.get_emphasis());
        assert_eq!(0b10, pix.get_value());
    }
}
