use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::error::Error;
use std::iter::once;

use crate::generator::palette_to_u8;

pub fn output_binary_uint8(path: &Path, palette: &[(f64, f64, f64)]) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(path)?;

    // flattening an array of tuples
    // https://users.rust-lang.org/t/flattening-a-vector-of-tuples/11409/4
    let buf: Vec<u8> = palette_to_u8(palette)
        .iter()
        .flat_map(
            |color| {
                once(color.0).chain(once(color.1).chain(once(color.2)))
            }
        )
        .collect();

    file.write_all(&buf)?;
    Ok(())
}
