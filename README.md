# Pally

Previously known as `palgen_persune`, `palgen-persune`.

Yet another NES palette generator, made with Rust!

My first big Rust project.

![A diagram of a generated palette, with CIE XY chromaticities, color phase, a test image, and color swatches.](docs/diagrams/palette_preview.png)
![An animated diagram showing the voltage of a period of a given color.](docs/diagrams/waveform_phase.gif)
![An animated diagram showing QAM demodulation of a given color.](docs/diagrams/QAM_phase.gif)
![An animated diagram of a generated palette undergoing different emphasis attenuations, with CIE XY chromaticities, color phase, a test image, and color swatches.](docs/diagrams/palette_preview_emphasis.gif)

Something to note: there _is_ no one true NES palette, but this generator
can pretty much approach colors that looks good enough for use in gaming, art,
and color work. feel free to adjust!

![Art of Addie by yoeynsf](docs/diagrams/addie.png)
![Art of Minae by forple](docs/diagrams/minae.png)

## Usage

To execute, run `cargo run --release`.

To access documentation, run `cargo doc --open`.

```sh
Yet another NES color palette generator.

Usage: pally [OPTIONS] <FILE_OUTPUT>

Arguments:
  <FILE_OUTPUT>  File output. Format set by `--file-format`

Options:
  -f, --file-format <FILE_FORMAT>
          File output format. Default = `pal-uint8` [default: pal-uint8] [possible values: pal-uint8, pal-double, pal-jasc, gpl, png, txt-html-hex, txt-mediawiki, header-uint8-t]
  -e, --emphasis
          Include emphasis entries in output
      --clip <CLIP>
          Alternate method for clipping out-of-range RGB colors [possible values: darken, desaturate]
      --normalize <NORMALIZE>
          Alternate method for scaling out-of-range RGB colors into gamut [possible values: scale, scale-clip-negative]
      --black-point <BLACK_POINT>
          Black point, in IRE units. If not defined, will default to use the voltage level of `$1D`
      --white-point <WHITE_POINT>
          White point, in IRE units. If not defined, will default to use the voltage level of `$30`
  -b, --brightness <BRIGHTNESS>
          Luma brightness delta in IRE units [default: 0]
  -c, --contrast <CONTRAST>
          Luma contrast factor [default: 1]
      --hue <HUE>
          Chroma hue angle delta, in degrees [default: 0]
  -s, --saturation <SATURATION>
          Chroma saturation factor [default: 1]
  -g, --gain <GAIN>
          Gain adjustment to signal before decoding, in IRE units [default: 0]
      --gamma <GAMMA>
          If defined, will apply a simple OETF gamma transfer function instead, where the EOTF function is assumed to be gamma 2.2
      --decode-type <DECODE_TYPE>
          Chooses what decoding to use. Not used in area-mode decoding [default: fir] [possible values: fir, comb2-line, comb3-line]
      --ppu <PPU>
          Settings for adjusting the encoding of signals PPU chip used for generating colors [default: 2c02] [possible values: 2c02, 2c03, 2c04-0000, 2c04-0001, 2c04-0002, 2c04-0003, 2c04-0004, 2c05-99, 2c07]
  -p, --phase-distortion <PHASE_DISTORTION>
          Amount of voltage-dependent impedance for RC lowpass, where 'RC = amount * (level/composite_white) * 1e-8' [default: 4]
  -h, --help
          Print help (see more with '--help')
  -V, --version
          Print version
```

## License

This work is licensed under the MIT license.

Copyright (C) Persune 2025.

## Credits

Dedicated to:

- NewRisingSun
- L. Spiro
- lidnariq
- PinoBatch
- jekuthiel
- _aitchFactor
- zeta0134

This would have not been possible without their help!
