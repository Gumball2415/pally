use std::{
    fs::File,
    io::Write,
    path::PathBuf,
    error::Error,
    fmt,
};

use clap::ValueEnum;
use nes_ppu_cvbs::PpuColor;

use crate::generator::palette_to_u8;

/// File output format.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum FileFormatType {
    /// .pal uint8
    #[default]
    PalUint8,
    /// .pal double
    PalDouble,
    /// .pal Jasc
    PalJasc,
    /// .gpl
    Gpl,
    /// .png
    Png,
    /// .txt HTML hex
    TxtHtmlHex,
    /// .txt MediaWiki
    TxtMediawiki,
    /// C header .h uint8_t
    HeaderUint8T,
}

/// Needed for default clap setting
impl fmt::Display for FileFormatType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Self::PalUint8 => ".pal uint8",
            Self::PalDouble => ".pal double",
            Self::PalJasc => ".pal Jasc",
            Self::Gpl => ".gpl",
            Self::Png => ".png",
            Self::TxtHtmlHex => ".txt HTML hex",
            Self::TxtMediawiki => ".txt MediaWiki",
            Self::HeaderUint8T => ".h uint8_t",
        })
    }
}

#[derive(Debug)]
enum FileIoError {
    Unimplemented(FileFormatType),
}

impl Error for FileIoError {}

impl fmt::Display for FileIoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        
        write!(f, "{}", match self {
            Self::Unimplemented(format) => format!("unimplemented file type: {format}"),
            // _ => "general file IO handler error".to_string()
        })
    }
}

pub fn output_file(
    path: &PathBuf,
    palette: &[(f64, f64, f64)],
    format: FileFormatType
) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    // TODO: trait-ify>
    match format {
        FileFormatType::PalUint8 => output_binary_uint8(file, palette),
        FileFormatType::PalDouble => output_binary_f64(file, palette),
        FileFormatType::Gpl => output_gimp_pal(file, palette),
        // TODO: png?
        FileFormatType::PalJasc => output_jasc_pal(file, palette),
        FileFormatType::TxtHtmlHex => output_html_hex(file, palette),
        FileFormatType::TxtMediawiki => output_mediawiki_table(file, palette),
        FileFormatType::HeaderUint8T => output_c_array(file, palette),

        // Unimplemented
        format => {
            
            Err(FileIoError::Unimplemented(format).into())
        }
    }
}

/// Flattens the `u8` color tuple array to a `u8` byte array.
/// Stores the color channels in order: `r: u8, g: u8, b: u8`
pub fn color_tuple_u8_array_to_vec_u8(buf: &[(u8, u8, u8)]) -> Vec<u8> {
    use std::iter::once;
    // flattening an array of tuples
    // https://users.rust-lang.org/t/flattening-a-vector-of-tuples/11409/4
    buf
        .iter()
        .flat_map(
            |color| {
                once(color.0).chain(once(color.1).chain(once(color.2)))
            }
        )
        .collect()
}


/// Flattens the `f64` color tuple array to a `u8` byte array, in little endian.
/// Stores the color channels in order:
/// `r: f64::to_le_bytes(), g: f64::to_le_bytes(), b: f64::to_le_bytes()`
fn color_tuple_f64_array_to_vec_u8(buf: &[(f64, f64, f64)]) -> Vec<u8> {
    let mut outbuffer: Vec<u8> = Vec::new();
    buf.iter().for_each(|(r, g, b)| {
        r.to_le_bytes().iter().for_each(|x| outbuffer.push(*x));
        g.to_le_bytes().iter().for_each(|x| outbuffer.push(*x));
        b.to_le_bytes().iter().for_each(|x| outbuffer.push(*x));
    });
    outbuffer
}

fn output_binary_uint8(
    mut file: File,
    palette: &[(f64, f64, f64)]
) -> Result<(), Box<dyn Error>> {

    let buf: Vec<u8> = color_tuple_u8_array_to_vec_u8(&palette_to_u8(palette));

    file.write_all(&buf)?;
    Ok(())
}

fn output_binary_f64(
    mut file: File,
    palette: &[(f64, f64, f64)]
) -> Result<(), Box<dyn Error>> {

    let buf: Vec<u8> = color_tuple_f64_array_to_vec_u8(palette);

    file.write_all(&buf)?;
    Ok(())
}

fn output_gimp_pal(
    mut file: File,
    palette: &[(f64, f64, f64)]
) -> Result<(), Box<dyn Error>> {

    let palette = palette_to_u8(palette);

    writeln!(file, "GIMP Palette")?;
    writeln!(file, "Name: generated NES/FC palette")?;
    // the columns are always 16
    writeln!(file, "Columns: 16")?;
    writeln!(file, "# https://github.com/Gumball2415/pally")?;

    // The colors are always written in the same order
    for (i, (r, g, b)) in palette.iter().enumerate() {
        let emph = (i & 0b111_00_0000) >> 6;
        let color_byte = i & 0b000_11_1111;

        writeln!(file,
            "{r:>4}{g:>4}{b:>4} ${color_byte:02X} emphasis {emph:03b}")?;
    }

    Ok(())
}

fn output_jasc_pal(
    mut file: File,
    palette: &[(f64, f64, f64)]
) -> Result<(), Box<dyn Error>> {

    let palette = palette_to_u8(palette);
    writeln!(file, "JASC-PAL")?;
    writeln!(file, "0100")?;
    writeln!(file, "{}", palette.len())?;

    // JASC-PAL stops loading when it encounters #000000??
    let mut black_entry_exists = false;

    for (r, g, b) in palette {
        if r == 0 && g == 0 && b == 0 {
            black_entry_exists = true;
        }
        else {
            writeln!(file, "{r:>0} {g:>0} {b:>0}")?;
        }
    }
    if black_entry_exists {
        writeln!(file, "0 0 0")?;
    }

    Ok(())
}

fn output_html_hex(
    mut file: File,
    palette: &[(f64, f64, f64)]
) -> Result<(), Box<dyn Error>> {
    let palette = palette_to_u8(palette);
    // TODO: add method to convert to rgb8bpc value?
    for (r, g, b) in palette {
        writeln!(file, "#{r:02X}{g:02X}{b:02X}")?;
    }
    Ok(())
}

fn output_mediawiki_table(
    mut file: File,
    palette: &[(f64, f64, f64)]
) -> Result<(), Box<dyn Error>> {
    let palette = palette_to_u8(palette);

    writeln!(file, "{{|class=\"wikitable\"")?;

    for (i, (r, g, b)) in palette.iter().enumerate() {
        let color: PpuColor = (i as u16).into();
        let (hue, _value, emph, byte) = color.deconstruct();

        if hue == 0 {
            writeln!(file, "|-")?;
        }
        let contrast = if 
            ((*r as u32)*299 + (*g as u32)*587 + (*b as u32)*114) <= 127500
        {
            0xFFF
        } else { 0x000 };

        // $0D color warning
        let label = if byte == 0x0D && emph == 0 {
            format!("[[Color_$0D_games#Effects|<s style=\"color:red\">${byte:02X}</s>]]")
        }
        else {
            format!("${byte:02X}")
        };

        // TODO: add method to convert to rgb8bpc value?
        writeln!(file, "|style=\"border:0px;background-color:#{r:02X}{g:02X}{b:02X};width:32px;height:32px;color:#{contrast:03X};text-align:center\"|{label}")?;
    }

    writeln!(file, "|}}")?;

    Ok(())
}



fn output_c_array(
    mut file: File,
    palette: &[(f64, f64, f64)]
) -> Result<(), Box<dyn Error>> {
    let palette = palette_to_u8(palette);
    for (r, g, b) in palette {
        writeln!(file, "0x{r:02X}, 0x{g:02X}, 0x{b:02X},")?;
    }

    Ok(())
}