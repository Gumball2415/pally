use nes_ppu_cvbs::{self, PpuPixel, PpuColor};
fn main() {
    println!("black is {}!", nes_ppu_cvbs::CVBS_BLACK);

    let pix = PpuPixel::Color(PpuColor(0b000_00_0001));
    let samples = nes_ppu_cvbs::encode_cvbs_pixel(&pix, &mut 0,12, false);
    println!("the samples generated from color {:}: {:?}", pix, samples)
}
