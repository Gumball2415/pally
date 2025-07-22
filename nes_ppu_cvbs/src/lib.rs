//! # NES PPU composite video encoder
//!
//! `nes_ppu_cvbs` handles the encoding of a given PPU pixel to its composite
//! counterpart, given a 9-bit PPU pixel and a phase value.
//!
//! Version: 0.2.0

use std::fmt;

/// Represents the 9-bit framebuffer pixel value used in most emulators.
///
/// ```txt
/// bgrvv hhhh
/// ||||| ++++-- Hue phase
/// |||++------- Luma
/// +++--------- Red, green, and blue PPUMASK emphasis bits
/// ```
///
/// Note: It is up to the emulator to swizzle between red and green emphasis
/// tint bits for PAL/Dendy PPUs.
#[derive(Debug)]
pub struct PpuColor (pub u16);

impl PpuColor {
    fn get_no_emphasis(&self) -> u8 {
        (self.0 & 0b000_11_1111) as u8
    }
    fn get_hue(&self) -> u8 {
        (self.0 & 0b000_00_1111) as u8
    }
    fn get_luma(&self) -> u8 {
        ((self.0 & 0b000_11_0000) >> 4) as u8
    }
    fn get_emphasis(&self) -> u8 {
        ((self.0 & 0b111_00_0000) >> 6) as u8
    }
}

impl fmt::Display for PpuColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "${:02X} {:03b}", self.get_no_emphasis(), self.get_emphasis())
    }
}

/// Represents all the types of pixels in a given composite video signal.
pub enum PpuPixel {
    Color(PpuColor),
    /// Sync voltage level
    Sync,
    /// Blanking voltage level
    Blank,
    /// Colorburst voltage levels, with a given hue phase
    Colorburst(u8),
}

impl fmt::Display for PpuPixel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Color(color) => color.fmt(f),
            Self::Blank => write!(f, "blank"),
            Self::Sync => write!(f, "sync"),
            Self::Colorburst(phase) => write!(f, "colorburst with phase {phase:02X}", )
        }
    }
}

/// Holds two versions of a given NES PPU composite level.
#[derive(Debug)]
struct LvlEmph {
    /// No attenuation from any emphasis bits.
    n: f64,
    /// Emphasis attenuated signal level.
    e: f64,
}

impl LvlEmph {
    fn emphasis(&self,
        attenuate: bool
    ) -> f64 {
        if attenuate {
            self.e
        } else {
            self.n
        }
    }
}

/// Holds two levels of a given hue waveform.
#[derive(Debug)]
struct LvlAmp {
    /// High voltage level `$x0`.
    _0: LvlEmph,
    /// Low voltage level `$xD`.
    _d: LvlEmph,
}

impl LvlAmp {
    /// Encodes a given PPU row color at a given single composite sample
    /// point.
    /// 
    /// Valid hue range: `0x0..0xF`\
    /// Valid sample phase range: `0..11`
    fn encode_sample(
        &self,
        hue: u8,
        sample_phase: u8,
        attenuate: bool,
        alternate_line: bool
    ) -> f64 {
        let in_phase = color_phase(hue, sample_phase, alternate_line);
        if in_phase {
            self._0.emphasis(attenuate)
        } else {
            self._d.emphasis(attenuate)
        }
    }
}

/// Holds the signal lookup table for all possible types of `PpuPixel`s.
#[derive(Debug)]
pub struct LvlCVBSTable {
    /// voltage levels of `$0x` colors
    s_0: LvlAmp,
    /// voltage levels of `$1x` colors
    s_1: LvlAmp,
    /// voltage levels of `$2x` colors
    s_2: LvlAmp,
    /// voltage levels of `$3x` colors
    s_3: LvlAmp,
    /// voltage levels of colorburst
    s_cb: LvlAmp,
    /// voltage levels of sync and blank
    s_bl: LvlAmp,
}

impl LvlCVBSTable {
    /// Encodes a given PPU `PpuColor` at a given single composite sample
    /// point.
    /// 
    /// Valid sample phase range: `0..11`
    fn encode_sample(
        &self,
        color: &PpuColor,
        sample_phase: u8,
        alternate_line: bool
    ) -> f64 {
        let luma = color.get_luma();
        let hue = color.get_hue();
        let emphasis = color.get_emphasis();
        let wave = match luma {
            0 => &self.s_0,
            1 => &self.s_1,
            2 => &self.s_2,
            3 => &self.s_3,
            invalid => panic!("not a valid luma value: {invalid}")
        };

        // Colors `$xE-$xF` always remain blanking
        if hue > 0xD {
            CVBS_BLACK
        } else {
            wave.encode_sample(
                hue,
                sample_phase,
                attenuate(hue, emphasis, sample_phase, alternate_line),
                alternate_line
            )
       }
    }
}

/// The voltage signal lookup table, for generating composite video
///
/// Voltages taken
/// from <https://forums.nesdev.org/viewtopic.php?p=159266#p159266>
///
/// $0x-$3x, $x0/$xD, no emphasis/emphasis
/// 5th index is purely colorburst
pub static SIGNAL_TABLE: LvlCVBSTable = LvlCVBSTable {
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
};

/// Blank/black level of the composite signal. Used for brightness
/// normalization functions.
pub static CVBS_BLACK: f64 = SIGNAL_TABLE.s_1._d.n;

/// White level of the composite signal. Used for brightness normalization
/// functions.
pub static CVBS_WHITE: f64 = SIGNAL_TABLE.s_3._0.n;

/// Precalculates the emphasis phase for further processing.
fn attenuate(
    hue: u8,
    emphasis: u8,
    sample_phase: u8,
    alternate_line: bool
) -> bool {
    let r = (emphasis & 0b001 != 0) && !color_phase(0xC, sample_phase, alternate_line);
    let g = (emphasis & 0b010 != 0) && !color_phase(0x4, sample_phase, alternate_line);
    let b = (emphasis & 0b100 != 0) && !color_phase(0x8, sample_phase, alternate_line);
    (r || g || b) && (hue < 0xE)
}

/// Original algorithm by Bisqwit.
/// 
/// Determines the waveform level by comparing the hue with the current
/// sample phase.
/// 
/// If the hue value is not chromatic (within `0x0..0xC`), it returns a constant wave value.
/// 
/// If true, the phase is at the high part. Else, it is within the low part.
/// 
/// Valid hue range: `0x0..0xF`\
/// Valid chromatic phase range: `0..11`
fn color_phase(
    hue: u8,
    sample_phase: u8,
    alternate_line: bool) -> bool {
    let in_phase = match pal_phase(hue, alternate_line) {
        0x0 => true,
        0x1..=0xC => (hue + sample_phase) % 12 >= 6,
        0xD..=0xF => false,
        invalid => panic!("not a valid hue value: {invalid}")
    };
    in_phase
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

    if (hue >= 1 && hue <= 12) && alternate_line {
        ALT_PHASE[(hue-1) as usize]
    } else {
        hue
    }
}

/// Encodes a given `PpuPixel` into the current composite video sample.
/// 
/// Must also include the current sample phase and the colorburst hue.
/// 
/// This calls into the respective associated functions to return a sample
/// value.
pub fn encode_cvbs_sample(
    pixel: &PpuPixel,
    sample_phase: u8,
    alternate_line: bool
) -> f64 {
    match pixel {
        PpuPixel::Sync =>
            SIGNAL_TABLE.s_bl._d.n,
        PpuPixel::Blank =>
            SIGNAL_TABLE.s_bl._0.n,
        PpuPixel::Colorburst(cburst_hue) =>
            SIGNAL_TABLE.s_cb.encode_sample(
                *cburst_hue, sample_phase, false, alternate_line
            ),
        PpuPixel::Color(color) =>
            SIGNAL_TABLE.encode_sample(color, sample_phase, alternate_line)
    }
}

/// Encodes a given PPU pixel with a given signal length.
/// 
/// Returns a vector of samples. `sample_phase` will be updated to
/// reflect the new phase after the pixel.
pub fn encode_cvbs_pixel(
    pixel: &PpuPixel,
    sample_phase: &mut u8,
    length: u8,
    alternate_line: bool
) -> Vec<f64> {
    let mut output: Vec<f64> = Vec::new();
    for phase in 0..length {
        output.push(encode_cvbs_sample(pixel, (*sample_phase + phase)%12, alternate_line));
    }
    *sample_phase = (*sample_phase + length) % 12;
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Given an expected waveform `Vec<f64>`, a `PpuPixel` to be encoded, and a
    /// length, test if the encoding works as expected.
    fn encode_cvbs(
        expected: &Vec<f64>,
        color: PpuPixel,
        length: u8
    ) {
        let mut sample_phase: u8 = 0;
        let out = encode_cvbs_pixel(&color, &mut sample_phase, length, false);

        assert_eq!(*expected, out);
        assert_eq!((length % 12), sample_phase);
    }

    #[test]
    /// Test encoding of a given color `$18`
    fn encode_cvbs_color() {
        let length= 24;
        let color = PpuPixel::Color(PpuColor(0b000_01_1000));
        let sig_hi = SIGNAL_TABLE.s_1._0.n;
        let sig_lo = SIGNAL_TABLE.s_1._d.n;

        // color $01
        let expected = vec![
            sig_hi, sig_hi, sig_hi, sig_hi, sig_lo, sig_lo,
            sig_lo, sig_lo, sig_lo, sig_lo, sig_hi, sig_hi,
            sig_hi, sig_hi, sig_hi, sig_hi, sig_lo, sig_lo,
            sig_lo, sig_lo, sig_lo, sig_lo, sig_hi, sig_hi,
        ];

        encode_cvbs(&expected, color, length);
    }

    #[test]
    /// Test encoding of a given color `$18`, with emphasis red
    fn encode_cvbs_color_emph() {
        let length= 24;
        let color = PpuPixel::Color(PpuColor(0b001_01_1000));
        let sig_hi = SIGNAL_TABLE.s_1._0.n;
        let sig_lo = SIGNAL_TABLE.s_1._d.n;

        let sig_he = SIGNAL_TABLE.s_1._0.e;
        let sig_le = SIGNAL_TABLE.s_1._d.e;

        // color $38, red emphasis
        let expected = vec![
            sig_he, sig_he, sig_he, sig_he, sig_le, sig_le,
            sig_lo, sig_lo, sig_lo, sig_lo, sig_hi, sig_hi,
            sig_he, sig_he, sig_he, sig_he, sig_le, sig_le,
            sig_lo, sig_lo, sig_lo, sig_lo, sig_hi, sig_hi,
        ];

        encode_cvbs(&expected, color, length);
    }

    #[test]
    fn encode_cvbs_emphasis_0() {
        let length= 24;
        let color = PpuPixel::Color(PpuColor(0b001_00_0000));
        let sig_hi = SIGNAL_TABLE.s_0._0.n;
        let sig_lo = SIGNAL_TABLE.s_0._0.e;
        let expected = vec![
            sig_lo, sig_lo, sig_lo, sig_lo, sig_lo, sig_lo,
            sig_hi, sig_hi, sig_hi, sig_hi, sig_hi, sig_hi,
            sig_lo, sig_lo, sig_lo, sig_lo, sig_lo, sig_lo,
            sig_hi, sig_hi, sig_hi, sig_hi, sig_hi, sig_hi,
        ];
        encode_cvbs(&expected, color, length);
    }

    #[test]
    fn encode_cvbs_emphasis_d() {
        let length= 24;
        let color = PpuPixel::Color(PpuColor(0b001_00_1101));
        let sig_hi = SIGNAL_TABLE.s_0._d.n;
        let sig_lo = SIGNAL_TABLE.s_0._d.e;
        let expected = vec![
            sig_lo, sig_lo, sig_lo, sig_lo, sig_lo, sig_lo,
            sig_hi, sig_hi, sig_hi, sig_hi, sig_hi, sig_hi,
            sig_lo, sig_lo, sig_lo, sig_lo, sig_lo, sig_lo,
            sig_hi, sig_hi, sig_hi, sig_hi, sig_hi, sig_hi,
        ];
        encode_cvbs(&expected, color, length);
    }

    #[test]
    fn encode_cvbs_emphasis_f() {
        let length= 24;
        let color = PpuPixel::Color(PpuColor(0b001_00_1111));
        let expected = vec![
            CVBS_BLACK, CVBS_BLACK, CVBS_BLACK, CVBS_BLACK,
            CVBS_BLACK, CVBS_BLACK, CVBS_BLACK, CVBS_BLACK,
            CVBS_BLACK, CVBS_BLACK, CVBS_BLACK, CVBS_BLACK,
            CVBS_BLACK, CVBS_BLACK, CVBS_BLACK, CVBS_BLACK,
            CVBS_BLACK, CVBS_BLACK, CVBS_BLACK, CVBS_BLACK,
            CVBS_BLACK, CVBS_BLACK, CVBS_BLACK, CVBS_BLACK,
        ];
        encode_cvbs(&expected, color, length);
    }

    #[test]
    fn encode_cvbs_emphasis_red() {
        let length= 24;
        let color = PpuPixel::Color(PpuColor(0b001_01_0000));
        let sig_hi = SIGNAL_TABLE.s_1._0.n;
        let sig_lo = SIGNAL_TABLE.s_1._0.e;
        let expected = vec![
            sig_lo, sig_lo, sig_lo, sig_lo, sig_lo, sig_lo,
            sig_hi, sig_hi, sig_hi, sig_hi, sig_hi, sig_hi,
            sig_lo, sig_lo, sig_lo, sig_lo, sig_lo, sig_lo,
            sig_hi, sig_hi, sig_hi, sig_hi, sig_hi, sig_hi,
        ];
        encode_cvbs(&expected, color, length);
    }

    #[test]
    fn encode_cvbs_emphasis_green() {
        let length= 24;
        let color = PpuPixel::Color(PpuColor(0b010_01_0000));
        let sig_hi = SIGNAL_TABLE.s_1._0.n;
        let sig_lo = SIGNAL_TABLE.s_1._0.e;
        let expected = vec![
            sig_lo, sig_lo, sig_hi, sig_hi, sig_hi, sig_hi,
            sig_hi, sig_hi, sig_lo, sig_lo, sig_lo, sig_lo,
            sig_lo, sig_lo, sig_hi, sig_hi, sig_hi, sig_hi,
            sig_hi, sig_hi, sig_lo, sig_lo, sig_lo, sig_lo,
        ];
        encode_cvbs(&expected, color, length);
    }

    #[test]
    fn encode_cvbs_emphasis_blue() {
        let length= 24;
        let color = PpuPixel::Color(PpuColor(0b100_01_0000));
        let sig_hi = SIGNAL_TABLE.s_1._0.n;
        let sig_lo = SIGNAL_TABLE.s_1._0.e;
        let expected = vec![
            sig_hi, sig_hi, sig_hi, sig_hi, sig_lo, sig_lo,
            sig_lo, sig_lo, sig_lo, sig_lo, sig_hi, sig_hi,
            sig_hi, sig_hi, sig_hi, sig_hi, sig_lo, sig_lo,
            sig_lo, sig_lo, sig_lo, sig_lo, sig_hi, sig_hi,
        ];
        encode_cvbs(&expected, color, length);
    }

    #[test]
    fn deconstruct_ppu_color() {
        let pix = PpuColor(0b100_10_1000);
        assert_eq!(0b1000, pix.get_hue());
        assert_eq!(0b100, pix.get_emphasis());
        assert_eq!(0b10, pix.get_luma());
    }
}
