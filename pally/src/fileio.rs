use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::error::Error;

use crate::generator::palette_to_u8;

pub fn output_binary_uint8(path: &Path, palette: &[(f64, f64, f64)]) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(path)?;

    let buf: Vec<u8> = color_tuple_u8_array_to_vec_u8(&palette_to_u8(palette));

    file.write_all(&buf)?;
    Ok(())
}


fn color_tuple_u8_array_to_vec_u8(buf: &[(u8, u8, u8)]) -> Vec<u8> {
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

// fn color_tuple_f64_array_to_vec_u8(buf: &[(f64, f64, f64)]) -> Vec<u8> {
//     let mut outbuffer: Vec<u8> = Vec::new();
//     for (r, g, b) in buf {
//         outbuffer.push(r.to_le_bytes());
//     };
//     outbuffer
// }