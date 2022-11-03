use std::{collections::HashSet, fs::File, io::Write, path::Path};

use dither::{color::palette, prelude::*};

fn main() {
    dither().unwrap();
}

fn dither() -> Result<()> {
    let img: Img<RGB<f64>> = Img::<RGB<u8>>::load(&Path::new("./image.png"))?
        .convert_with(|rgb| rgb.convert_with(f64::from));

    let quantize = dither::create_quantize_n_bits_func(4)?;
    let plt: Vec<_> = palette::parse(include_str!("../../palette.plt")).unwrap();

    let dithered_img = ditherer::ATKINSON
        .dither(img, palette::quantize(plt.as_slice()))
        .convert_with(|rgb| rgb.convert_with(clamp_f64_to_u8));

    dithered_img.clone().save(Path::new("preview.png")).unwrap();

    let mut output = File::create("image.bin").unwrap();
    output
        .write_all(
            dithered_img
                .into_vec()
                .chunks_exact(2)
                .map(|chunk| {
                    plt.iter().position(|col| chunk[1] == *col).unwrap() as u8
                    | ((plt.iter().position(|col| chunk[0] == *col).unwrap() as u8) << 4)
                })
                .collect::<Vec<_>>()
                .as_slice(),
        )
        .unwrap();

    Ok(())
}
