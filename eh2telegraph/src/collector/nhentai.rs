/// nhentai collector.
/// Host matching: nhentai.to or nhentai.net
///
/// Since nhentai.net always enable CloudFlare Firewall, so we will
/// use nhapi.cat42.uk(there will be some syncing latency).
use again::RetryPolicy;
use rand::seq::SliceRandom;
use reqwest::Response;
use serde::Deserialize;
use std::time::Duration;

use crate::{
    http_client::{GhostClient, GhostClientBuilder},
    stream::AsyncStream,
    util::get_bytes,
};

use super::{AlbumMeta, Collector, ImageData, ImageMeta};

const NHAPI: &str = "https://nhapi.cat42.uk/gallery/";

lazy_static::lazy_static! {
    static ref RETRY_POLICY: RetryPolicy = RetryPolicy::fixed(Duration::from_millis(200))
        .with_max_retries(5)
        .with_jitter(true);
}

const DOMAIN_LIST: [&str; 0] = [];
const NH_CDN_LIST: [&str; 5] = [
    "https://i.nhentai.net/galleries",
    "https://i2.nhentai.net/galleries",
    "https://i3.nhentai.net/galleries",
    "https://i5.nhentai.net/galleries",
    "https://i7.nhentai.net/galleries",
];

#[derive(Debug, Clone, Default)]
pub struct NHCollector {
    client: GhostClient,
}

impl NHCollector {
    pub fn new() -> Self {
        Self {
            client: GhostClientBuilder::default()
                .with_cf_resolve(&DOMAIN_LIST)
                .build(None),
        }
    }

    pub fn new_from_config() -> anyhow::Result<Self> {
        Ok(Self::new())
    }
}

#[derive(Deserialize)]
struct NhAlbum {
    // id: u32,
    media_id: String,
    title: Title,
    images: Images,
    // tags: Vec<Tag>,
    // num_pages: usize,
}

#[derive(Deserialize)]
struct Title {
    pretty: Option<String>,
    english: Option<String>,
    japanese: Option<String>,
}

impl Title {
    fn title(&self, f: impl Fn() -> String) -> String {
        if let Some(pretty) = &self.pretty {
            return pretty.clone();
        }
        if let Some(english) = &self.english {
            return english.clone();
        }
        if let Some(japanese) = &self.japanese {
            return japanese.clone();
        }
        f()
    }
}

#[derive(Deserialize)]
struct Images {
    pages: Vec<Image>,
}

#[derive(Deserialize, Clone, Copy)]
struct Image {
    t: ImageType,
}

#[derive(Debug, Deserialize, Clone, Copy)]
enum ImageType {
    #[serde(rename = "j")]
    Jpg,
    #[serde(rename = "p")]
    Png,
    #[serde(rename = "g")]
    Gif,
}

impl ImageType {
    fn as_str(&self) -> &'static str {
        match self {
            ImageType::Jpg => ".jpg",
            ImageType::Png => ".png",
            ImageType::Gif => ".gif",
        }
    }
}

// #[derive(Deserialize)]
// struct Tag {
//     #[serde(rename = "type")]
//     typ: String,
//     name: String,
// }

impl Collector for NHCollector {
    type FetchError = anyhow::Error;
    type StreamError = anyhow::Error;
    type ImageStream = NHImageStream;

    #[inline]
    fn name() -> &'static str {
        "nhentai"
    }

    async fn fetch(
        &self,
        path: String,
    ) -> Result<(AlbumMeta, Self::ImageStream), Self::FetchError> {
        // normalize url
        let mut parts = path.trim_matches(|c| c == '/').split('/');
        let g = parts.next();
        let album_id = parts.next();
        let album_id = match (g, album_id) {
            (Some("g"), Some(album_id)) => album_id,
            _ => {
                return Err(anyhow::anyhow!("invalid input path({path}), gallery url is expected(like https://nhentai.net/g/333678)"));
            }
        };
        // Note: Since nh enables CF firewall, we use nhentai.to instead.
        let api_url = format!("{NHAPI}{album_id}");
        let original_url = format!("https://nhentai.net/g/{album_id}");
        tracing::info!("[nhentai] process {api_url}(original url {original_url})");

        // clone client to force changing ip
        let client = self.client.clone();
        let album: NhAlbum = client
            .get(&api_url)
            .send()
            .await
            .and_then(Response::error_for_status)?
            .json()
            .await?;
        let title = album.title.title(|| format!("Nhentai-{album_id}"));
        let image_urls = album
            .images
            .pages
            .iter()
            .enumerate()
            .map(|(idx, page)| ImageURL::new(album.media_id.clone(), idx + 1, page.t))
            .collect::<Vec<_>>()
            .into_iter();

        Ok((
            AlbumMeta {
                link: original_url,
                name: title,
                class: None,
                description: None,
                authors: None,
                tags: None,
            },
            NHImageStream { client, image_urls },
        ))
    }
}

#[derive(Debug)]
struct ImageURL {
    raw: String,
    media: String,
    id: usize,
    typ: ImageType,
}

impl ImageURL {
    fn new(media: String, id: usize, typ: ImageType) -> Self {
        Self {
            raw: Self::random_cdn_link(&media, id, typ),
            media,
            id,
            typ,
        }
    }

    fn raw(&self) -> &str {
        &self.raw
    }

    fn fallback(&self) -> String {
        Self::random_cdn_link(&self.media, self.id, self.typ)
    }

    fn random_cdn_link(media: &str, id: usize, typ: ImageType) -> String {
        let cdn = NH_CDN_LIST
            .choose(&mut rand::thread_rng())
            .expect("empty CDN list");
        format!("{cdn}/{media}/{id}{}", typ.as_str())
    }
}

#[derive(Debug)]
pub struct NHImageStream {
    client: GhostClient,
    image_urls: std::vec::IntoIter<ImageURL>,
}

impl NHImageStream {
    async fn load_image(client: GhostClient, link: &str) -> anyhow::Result<(ImageMeta, ImageData)> {
        let image_data = RETRY_POLICY
            .retry(|| async { get_bytes(&client, link).await })
            .await?;

        tracing::trace!(
            "download nhentai image with size {}, link: {link}",
            image_data.len()
        );
        let meta = ImageMeta {
            id: link.to_string(),
            url: link.to_string(),
            description: None,
        };
        Ok((meta, image_data))
    }
}

impl AsyncStream for NHImageStream {
    type Item = anyhow::Result<(ImageMeta, ImageData)>;

    type Future = impl std::future::Future<Output = Self::Item>;

    fn next(&mut self) -> Option<Self::Future> {
        let link = self.image_urls.next()?;
        let client = self.client.clone();
        Some(async move {
            match Self::load_image(client.clone(), link.raw()).await {
                Ok(r) => Ok(r),
                Err(e) => {
                    tracing::error!("fallback for nh image {link:?}: {e}");
                    Self::load_image(client, &link.fallback()).await
                }
            }
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.image_urls.size_hint()
    }
}
