mod utils;

#[macro_use]
extern crate lazy_static;

use std::{path::Path};

use dither::{color::palette, prelude::*};
use image::{
    ImageBuffer, Rgb, RgbImage,
};

lazy_static! {
    static ref PALETTE: Vec<RGB<u8>> = palette::parse(include_str!("../../palette.plt")).unwrap();
}

pub async fn dither(image: Vec<u8>) -> Result<Vec<u8>> {
    let img: RgbImage = {
        let mut target = ImageBuffer::from_pixel(600, 448, Rgb { 0: [255, 255, 255] });
        let image = image::load_from_memory(&image).unwrap().into_rgb8();
        utils::fit_image(&mut target, &image);
        target
    };

    let img = img.pixels().map(|p| RGB::from(p.0));

    let img: Img<RGB<f64>> = Img::<RGB<u8>>::new(img, 600)
        .unwrap()
        .convert_with(|rgb| rgb.convert_with(f64::from));

    let dithered_img = ditherer::ATKINSON
        .dither(img, palette::quantize(&PALETTE))
        .convert_with(|rgb| rgb.convert_with(clamp_f64_to_u8));

    dithered_img.clone().save(Path::new("preview.png")).unwrap();

    let img_4bit = dithered_img
        .into_vec()
        .chunks_exact(2)
        .map(|chunk| {
            ((PALETTE.iter().position(|col| chunk[0] == *col).unwrap() as u8) << 4)
                | PALETTE.iter().position(|col| chunk[1] == *col).unwrap() as u8
        })
        .collect::<Vec<_>>();

    Ok(img_4bit)
}
