/// # NES PPU composite video encoder
///
/// `ppu_cvbs` handles the encoding of a given PPU pixel to its composite
/// counterpart, given a 9-bit PPU pixel and a phase value.
///
/// Version: 0.2.0
pub mod ppu_cvbs {
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
    pub struct PpuColor (u16);

    impl PpuColor {
        fn get_hue(&self) -> u8 {
            (self.0 & 0b000_00_1111) as u8
        }
        fn get_luma(&self) -> u8 {
            (self.0 & 0b000_11_0000 >> 4) as u8
        }
        fn get_emphasis(&self) -> u8 {
            (self.0 & 0b111_00_0000 >> 6) as u8
        }
    }

    /// Represents all the types of pixels in a given composite video signal.
    pub enum PpuPixel {
        Color(PpuColor),
        Sync,
        Blank,
        Colorburst,
    }

    /// Holds two versions of a given NES PPU composite level.
    struct LvlEmph {
        /// No attenuation from any emphasis bits.
        n: f64,
        /// Emphasis attenuated signal level.
        e: f64,
    }

    impl LvlEmph {
        fn emphasis(&self,
            emphasis: u8,
            hue: u8,
            sample_phase: u8,
            alternate_line: bool
        ) -> f64 {
            if (
                // Red emphasis
                (emphasis & 0b001 != 0)
                && !color_phase(0xC, sample_phase, alternate_line)

                // Green emphasis
                || (emphasis & 0b010 != 0)
                && !color_phase(0x4, sample_phase, alternate_line)

                // Blue emphasis
                || (emphasis & 0b010 != 0)
                && !color_phase(0x8, sample_phase, alternate_line)
            ) && (hue < 0xE) {
                self.e
            } else {
                self.n
            }
        }
    }

    /// Holds two levels of a given hue waveform.
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
            emphasis: u8,
            alternate_line: bool
        ) -> f64 {
            if color_phase(hue, sample_phase, alternate_line) {
                self._0.emphasis(emphasis, hue, sample_phase, alternate_line)
            } else {
                self._d.emphasis(emphasis, hue, sample_phase, alternate_line)
            }
        }
    }

    /// Holds the signal lookup table for all possible types of `PpuPixel`s.
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
            color: PpuColor,
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

            wave.encode_sample(hue, sample_phase, emphasis, alternate_line)
        }
    }

    /// Alternates the V phase of a given hue in the same manner as 2C07s, if
    /// a valid hue.
    /// 
    /// If not a valid hue, returns the old hue value.
    /// 
    /// Valid range for alternation: 1-12
    fn pal_phase(hue: u8, alternate_line: bool) -> u8 {
        static ALT_PHASE: [u8; 12] = [
            4, 3, 2, 1,
            12, 11, 10, 9,
            8, 7, 6, 5,
        ];
        if hue >= 1 && hue <= 12 && alternate_line {
            ALT_PHASE[(hue-1) as usize]
        } else {
            hue
        }
    }

    /// Original algorithm by Bisqwit.
    /// 
    /// Determines the waveform level by comparing the hue with the current
    /// sample phase.
    /// 
    /// If the hue value is not chromatic (within `0x0..0xC`), it returns a constant wave value.
    /// 
    /// If true, the waveform is within high phase. Else, it is within the low
    /// phase.
    /// 
    /// Valid hue range: `0x0..0xF`\
    /// Valid phase range: `0..11`
    fn color_phase(hue: u8, phase: u8, alternate_line: bool) -> bool {
        match pal_phase(hue, alternate_line) {
            0x0 => true,
            0x1..=0xC => (hue + phase % 12) > 6,
            0xD..=0xF => false,
            invalid => panic!("not a valid hue value: {invalid}")
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

    /// Encodes a given `PpuPixel` into the current composite video sample.
    /// 
    /// Must also include the current sample phase and the colorburst hue.
    /// 
    /// This calls into the respective associated functions to return a sample
    /// value.
    pub fn encode_cvbs_sample(
        pixel: PpuPixel,
        sample_phase: u8,
        cburst_hue: u8,
        alternate_line: bool
    ) -> f64 {
        match pixel {
            PpuPixel::Sync =>
                SIGNAL_TABLE.s_bl._d.n,
            PpuPixel::Blank =>
                SIGNAL_TABLE.s_bl._0.n,
            PpuPixel::Colorburst =>
                SIGNAL_TABLE.s_cb.encode_sample(
                    cburst_hue, sample_phase, 0, alternate_line
                ),
            PpuPixel::Color(color) =>
                SIGNAL_TABLE.encode_sample(color, sample_phase, alternate_line)
        }
    }
}
