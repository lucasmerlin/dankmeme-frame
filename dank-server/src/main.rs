use warp::{Filter, reply::Response};

#[tokio::main]
async fn main() {
    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let hello = warp::path!()
    .map(|| Response::new(dank_image::dither().unwrap().into()));

    warp::serve(hello)
    .run(([0, 0, 0, 0], 8080))
    .await;
}