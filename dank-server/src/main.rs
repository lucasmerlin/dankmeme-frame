use serde::{Deserialize, Serialize};
use warp::{Filter, reply::Response};
use rand::Rng;

#[tokio::main]
async fn main() {
    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let hello = warp::path!()
    .then(|| async {Response::new(load_image().await.unwrap().into())});

    warp::serve(hello)
    .run(([0, 0, 0, 0], 8080))
    .await;
}


async fn load_image() -> anyhow::Result<Vec<u8>> {

    let url = "https://malmal.io/api/gallery/entries?order=TOP&minDate=Tue,%2004%20Oct%202022%2023:31:37%20GMT&limit=30&offset=0";

    let response = reqwest::get(url).await?;

    let body = response.text().await?;

    let images: Vec<GalleryEntry> = serde_json::from_str(&body)?;

    let image = &images[rand::random::<usize>() % images.len()];

    Ok(dank_image::dither(&image.imageUrl).await?)
}

#[derive(Serialize, Deserialize)]
struct GalleryEntry {
    imageUrl: String,
}