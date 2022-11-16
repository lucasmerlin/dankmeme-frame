use serde::{Deserialize, Serialize};
use warp::{Filter, reply::Response};
use rand::{Rng, prelude::SliceRandom};

#[tokio::main]
async fn main() {
    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let hello = warp::path!("malmal")
    .then(|| async {Response::new(load_malmal_image().await.unwrap().into())});
    let memes = warp::path!("memes")
    .then(|| async {Response::new(load_reddit_meme().await.unwrap().into())});

    warp::serve(hello.or(memes))
    .run(([0, 0, 0, 0], 8080))
    .await;
}


async fn load_malmal_image() -> anyhow::Result<Vec<u8>> {

    let url = "https://malmal.io/api/gallery/entries?order=TOP&minDate=2021-11-09T22:40:49.187Z&limit=200&offset=0";

    let response = reqwest::get(url).await?;

    let body = response.text().await?;

    let images: Vec<GalleryEntry> = serde_json::from_str(&body)?;

    let image = &images[rand::random::<usize>() % images.len()];

    println!("Using https://malmal.io/gallery/{}", image.id);

    Ok(dank_image::dither(&image.imageUrl).await?)
}

async fn load_reddit_meme() -> anyhow::Result<Vec<u8>> {
    let subreddit = roux::Subreddit::new("dankmemes+me_irl+meirl");

    let hot = subreddit.hot(100, None).await?;

    let images = hot.data.children.iter().filter_map(|post| {
        post.data.url.as_ref().and_then(|url| if [".png", ".jpg", ".webp"].iter().any(|ext| url.ends_with(ext)) {
            Some(&post.data)
        } else {
            None
        })
    }).collect::<Vec<_>>();

    let image = images.choose(&mut rand::thread_rng()).unwrap();

    Ok(dank_image::dither(&image.url.as_ref().unwrap()).await?)
}

#[derive(Serialize, Deserialize)]
struct GalleryEntry {
    id: u64,
    imageUrl: String,
}