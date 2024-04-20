/// nhentai collector.
/// Host matching: e-hentai.org
use crate::{
    http_client::{GhostClient, GhostClientBuilder},
    stream::AsyncStream,
    util::match_first_group,
    util::{get_bytes, get_string},
};
use again::RetryPolicy;
use ipnet::Ipv6Net;
use regex::Regex;
use reqwest::header;

use std::time::Duration;

use super::{
    utils::paged::{PageFormatter, PageIndicator, Paged},
    AlbumMeta, Collector, ImageData, ImageMeta,
};

lazy_static::lazy_static! {
    static ref PAGE_RE: Regex = Regex::new(r#"<a href="(https://e-hentai\.org/s/\w+/[\w-]+)">"#).unwrap();
    static ref IMG_RE: Regex = Regex::new(r#"<img id="img" src="(.*?)""#).unwrap();
    static ref TITLE_RE: Regex = Regex::new(r#"<h1 id="gn">(.*?)</h1>"#).unwrap();

    static ref RETRY_POLICY: RetryPolicy = RetryPolicy::fixed(Duration::from_millis(200))
        .with_max_retries(5)
        .with_jitter(true);
}
const TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Default)]
pub struct EHCollector {
    client: GhostClient,
    raw_client: reqwest::Client,
}

impl EHCollector {
    pub fn new(prefix: Option<Ipv6Net>) -> Self {
        let mut request_headers = header::HeaderMap::new();
        request_headers.insert(
            header::COOKIE,
            header::HeaderValue::from_str("nw=1").unwrap(),
        );

        Self {
            client: GhostClientBuilder::default()
                .with_default_headers(request_headers)
                .with_cf_resolve(&["e-hentai.org"])
                .build(prefix),
            raw_client: reqwest::Client::builder().timeout(TIMEOUT).build().unwrap(),
        }
    }

    pub fn new_from_config() -> anyhow::Result<Self> {
        let mut request_headers = header::HeaderMap::new();
        request_headers.insert(
            header::COOKIE,
            header::HeaderValue::from_str("nw=1").unwrap(),
        );

        Ok(Self {
            client: GhostClientBuilder::default()
                .with_default_headers(request_headers)
                .with_cf_resolve(&["e-hentai.org"])
                .build_from_config()?,
            raw_client: reqwest::Client::builder().timeout(TIMEOUT).build().unwrap(),
        })
    }
}

impl Collector for EHCollector {
    type FetchError = anyhow::Error;
    type StreamError = anyhow::Error;
    type ImageStream = EHImageStream;

    #[inline]
    fn name() -> &'static str {
        "e-hentai"
    }

    async fn fetch(
        &self,
        path: String,
    ) -> Result<(AlbumMeta, Self::ImageStream), Self::FetchError> {
        // normalize url
        let mut parts = path.trim_matches(|c| c == '/').split('/');
        let g = parts.next();
        let album_id = parts.next();
        let album_token = parts.next();
        let (album_id, album_token) = match (g, album_id, album_token) {
            (Some("g"), Some(album_id), Some(album_token)) => (album_id, album_token),
            _ => {
                return Err(anyhow::anyhow!("invalid input path({path}), gallery url is expected(like https://e-hentai.org/g/2127986/da1deffea5)"));
            }
        };
        let url = format!("https://e-hentai.org/g/{album_id}/{album_token}");
        tracing::info!("[e-hentai] process {url}");

        // clone client to force changing ip
        let client = self.client.clone();
        let mut paged = Paged::new(0, EHPageIndicator { base: url.clone() });
        let gallery_pages = paged.pages(&client).await?;

        // Since paged returns at least one page, we can safely get it.
        let title = match_first_group(&TITLE_RE, &gallery_pages[0])
            .unwrap_or("No Title")
            .to_string();

        let mut image_page_links = Vec::new();
        for gallery_page in gallery_pages.iter() {
            PAGE_RE.captures_iter(gallery_page).for_each(|c| {
                let matching = c.get(1).expect("regexp is matched but no group 1 found");
                image_page_links.push(matching.as_str().to_string());
            });
        }

        if image_page_links.is_empty() {
            return Err(anyhow::anyhow!(
                "invalid url, maybe resource has been deleted."
            ));
        }

        Ok((
            AlbumMeta {
                link: url,
                name: title,
                class: None,
                description: None,
                authors: None,
                tags: None,
            },
            EHImageStream {
                client,
                raw_client: self.raw_client.clone(),
                image_page_links: image_page_links.into_iter(),
            },
        ))
    }
}

#[derive(Debug)]
pub struct EHImageStream {
    client: GhostClient,
    raw_client: reqwest::Client,
    image_page_links: std::vec::IntoIter<String>,
}

impl EHImageStream {
    async fn load_image(
        client: &GhostClient,
        raw_client: &reqwest::Client,
        link: String,
    ) -> anyhow::Result<(ImageMeta, ImageData)> {
        let content = RETRY_POLICY
            .retry(|| async { get_string(client, &link).await })
            .await?;
        let img_url = match_first_group(&IMG_RE, &content)
            .ok_or_else(|| anyhow::anyhow!("unable to find image in page"))?;
        let image_data = RETRY_POLICY
            .retry(|| async { get_bytes(raw_client, img_url).await })
            .await?;

        tracing::trace!(
            "download e-hentai image with size {}, link: {link}",
            image_data.len()
        );
        let meta = ImageMeta {
            id: link,
            url: img_url.to_string(),
            description: None,
        };
        Ok((meta, image_data))
    }
}

impl AsyncStream for EHImageStream {
    type Item = anyhow::Result<(ImageMeta, ImageData)>;

    type Future = impl std::future::Future<Output = Self::Item>;

    fn next(&mut self) -> Option<Self::Future> {
        let link = self.image_page_links.next()?;
        let client = self.client.clone();
        let raw_client = self.raw_client.clone();
        Some(async move { Self::load_image(&client, &raw_client, link).await })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.image_page_links.size_hint()
    }
}

struct EHPageIndicator {
    base: String,
}

impl PageFormatter for EHPageIndicator {
    fn format_n(&self, n: usize) -> String {
        format!("{}/?p={}", self.base, n)
    }
}

impl PageIndicator for EHPageIndicator {
    fn is_last_page(&self, content: &str, next_page: usize) -> bool {
        let html = format!(
            "<a href=\"{}/?p={}\" onclick=\"return false\">",
            self.base, next_page
        );
        !content.contains(&html)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[tokio::test]
    async fn demo() {
        let collector = EHCollector {
            raw_client: Default::default(),
            client: Default::default(),
        };
        let (album, mut image_stream) = collector
            .fetch("/g/2122174/fd2525031e".to_string())
            .await
            .unwrap();
        println!("album: {album:?}");

        let maybe_first_image = image_stream.next().unwrap().await;
        if let Ok((meta, data)) = maybe_first_image {
            println!("first image meta: {meta:?}");
            println!("first image data length: {}", data.len());
        }
    }

    #[ignore]
    #[test]
    fn regex_match() {
        // test page: https://e-hentai.org/g/2122174/fd2525031e
        let r = Regex::new(r#"<a href="(https://e-hentai\.org/s/\w+/[\w-]+)">"#).unwrap();
        let h = r#"<div class="gdtm" style="height:170px"><div style="margin:1px auto 0; width:100px; height:140px; background:transparent url(https://ehgt.org/m/002122/2122174-00.jpg) -600px 0 no-repeat"><a href="https://e-hentai.org/s/bd2b37d829/2122174-7"><img alt="007" title="Page 7: 2.png" src="https://ehgt.org/g/blank.gif" style="width:100px; height:139px; margin:-1px 0 0 -1px" /></a></div></div><div class="gdtm" style="height:170px"><div style="margin:1px auto 0; width:100px; height:100px; background:transparent url(https://ehgt.org/m/002122/2122174-00.jpg) -700px 0 no-repeat"><a href="https://e-hentai.org/s/4ca72f757d/2122174-8"><img alt="008" title="Page 8: 3.png" src="https://ehgt.org/g/blank.gif" style="width:100px; height:99px; margin:-1px 0 0 -1px" />"#;

        let mut iter = r.captures_iter(h);
        let first = iter.next().unwrap();
        println!("{}", first.get(1).unwrap().as_str());

        let second = iter.next().unwrap();
        println!("{}", second.get(1).unwrap().as_str());
    }
}
