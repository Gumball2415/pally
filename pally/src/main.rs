
fn main() {
    use nes_ppu_cvbs::*;

    let ppu_filter = NesPpuCvbs::new();

    println!("black is {}!", ppu_filter.lut.get_black());

    let pix = PpuPixel::Active(0b000_00_0001.into());
    let samples = ppu_filter.encode_cvbs_pixel(&pix, &mut 0,12, false);
    println!("the samples generated from color {:}: {:?}", pix, samples)
}
