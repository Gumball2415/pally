use nes_ppu_cvbs::{self, PpuPixel};
fn main() {
    println!("black is {}!", nes_ppu_cvbs::CVBS_BLACK);

    let pix = PpuPixel::Active(0b000_00_0001.into());
    let samples = nes_ppu_cvbs::encode_cvbs_pixel(&pix, &mut 0,12, false);
    println!("the samples generated from color {:}: {:?}", pix, samples)
}
