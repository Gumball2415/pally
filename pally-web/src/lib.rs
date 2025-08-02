use wasm_bindgen::prelude::*;

use js_sys::JsString;

use cvbs_decode::*;
use nes_ppu_cvbs::*;
use pally::*;

// logging capability
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// const PALETTE_SIZE: usize = 64 * 8 * 3;
// static mut PALETTE_BUFFER: [u8; PALETTE_SIZE] = [0; PALETTE_SIZE];

// #[wasm_bindgen]
// pub fn get_palette_loc() -> *const u8 {
//     unsafe {
//         (*(&raw mut PALETTE_BUFFER)).as_ptr()
//     }
// }

#[wasm_bindgen]
pub fn generate_palette(
    render_emphasis: bool,
    clip: JsString,
    normalize: JsString,
    black_point: Option<f64>,
    white_point: Option<f64>,
    brightness: f64,
    contrast: f64,
    hue: f64,
    saturation: f64,
    gain: f64,
    gamma: Option<f64>,
    ppu: JsString,
    phase_distortion: f64,
) -> Vec<u8> {
    // console_log!("render_emphasis: {render_emphasis:?}");
    // console_log!("clip: {clip:?}");
    // console_log!("normalize: {normalize:?}");
    // console_log!("black_point: {black_point:?}");
    // console_log!("white_point: {white_point:?}");
    // console_log!("brightness: {brightness}");
    // console_log!("contrast: {contrast:?}");
    // console_log!("hue: {hue:?}");
    // console_log!("saturation: {saturation:?}");
    // console_log!("gain: {gain:?}");
    // console_log!("gamma: {gamma:?}");
    // console_log!("ppu: {ppu:?}");
    // console_log!("phase_distortion: {phase_distortion:?}");

    // TODO: need a better way to parse settings across boundaries
    let settings = &PallySettings {
        render_emphasis,
        clip: {
            let clip: String = clip.into();
            match clip.as_str() {
                "Darken" => Some(ClipType::Darken),
                "Desaturate" => Some(ClipType::Desaturate),
                _ => None,
            }
        },
        normalize: {
            let normalize: String = normalize.into();
            match normalize.as_str() {
                "Scale" => Some(NormalizeType::Scale),
                "ScaleClipNegative" => Some(NormalizeType::ScaleClipNegative),
                _ => None,
            }
        },
        black_point,
        white_point,
        brightness,
        contrast,
        hue,
        saturation,
        gain,
        gamma,
        ppu: {
            let ppu: String = ppu.into();
            match ppu.as_str() {
                "2C02" => PpuType::_2C02,
                "2C07" => PpuType::_2C07,
                _ => Default::default(),
            }
        },
        phase_distortion,
        ..Default::default()
    };
    // console_log!("{settings:?}");
    let pally = &parse_pally_config(settings);

    let (encoder, decoder) = &parse_encoder_decoder(settings);

    // generate colors
    let buf = pally::fileio::color_tuple_u8_array_to_vec_u8(
        &pally::generator::palette_to_u8(
                &generate_colors(pally.render_emphasis, encoder, decoder),
        )
    );
    // console_log!("{buf:?}");
    buf
}


#[cfg(test)]
pub mod tests{
    use super::*;
    use wasm_bindgen_test::*;

    #[test]
    #[wasm_bindgen_test]
    fn palgen_test() {
        wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);
        generate_palette(
            true,
            "".into(),
            "".into(),
            None,
            None,
            0.0,
            1.0,
            0.0,
            1.0,
            0.0,
            None,
            "2C02".into(),
            4.0
        );
    }
}
