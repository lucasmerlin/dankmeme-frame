use std::{collections::HashSet, fs::File, io::Write, path::Path};

use dither::{color::palette, prelude::*};
use image::ImageFormat;

pub async fn dither(imageUrl: &str) -> Result<Vec<u8>> {

    let img_url = format!("https://img.malmal.io/insecure/w:600/h:448/rt:fill/plain/{imageUrl}@png");

    let result = reqwest::get(img_url).await.unwrap();

    let data = result.bytes().await.unwrap();

    println!("Loaded png data: {}", data.len());

    let img = image::load_from_memory_with_format(&data, ImageFormat::Png).unwrap().to_rgb();


    println!("Image size {} {}", img.width(), img.height());

    let img = img.pixels().map(|p| RGB::from(p.0));



    let img: Img<RGB<f64>> = Img::<RGB<u8>>::new(img, 600).unwrap()
        .convert_with(|rgb| rgb.convert_with(f64::from));

    let quantize = dither::create_quantize_n_bits_func(4)?;
    let plt: Vec<_> = palette::parse(include_str!("../../palette.plt")).unwrap();

    let dithered_img = ditherer::ATKINSON
        .dither(img, palette::quantize(plt.as_slice()))
        .convert_with(|rgb| rgb.convert_with(clamp_f64_to_u8));

    dithered_img.clone().save(Path::new("preview.png")).unwrap();

    let img_4bit = dithered_img
        .into_vec()
        .chunks_exact(2)
        .map(|chunk| {
            plt.iter().position(|col| chunk[1] == *col).unwrap() as u8
                | ((plt.iter().position(|col| chunk[0] == *col).unwrap() as u8) << 4)
        })
        .collect::<Vec<_>>();

    let mut output = File::create("image.bin").unwrap();
    output.write_all(img_4bit.as_slice()).unwrap();


    Ok(img_4bit)
}
