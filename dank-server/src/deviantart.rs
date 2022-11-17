use anyhow::Context;
use rand::prelude::{IteratorRandom};
use rss::Channel;

use crate::load_url;

pub async fn laod_deviantart_iamge() -> anyhow::Result<Vec<u8>> {
    let content =
        reqwest::get("https://backend.deviantart.com/rss.xml?type=deviation&q=topic%3Aphotography")
            .await?
            .bytes()
            .await?;

    let channel = Channel::read_from(&content[..])?;

    println!(
        "{:?}",
        channel
            .items()
            .get(0)
            .unwrap()
            .extensions
            .get("media")
            .context("no media")?
            .get("content")
            .context("no content")?
            .get(0)
            .context("no index")?
    );

    let item = channel
        .items()
        .iter()
        .filter_map(|item| {
            item.extensions
                .get("media")
                .and_then(|media| media.get("content"))
                .and_then(|content| content.get(0))
                .and_then(|content| content.attrs().get("url"))
        })
        .choose(&mut rand::thread_rng())
        .context("No Content Found")?;

    load_url(&item).await
}
