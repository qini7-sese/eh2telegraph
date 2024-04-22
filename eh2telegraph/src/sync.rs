use crate::{
    buffer::{DataSized, ImageBuffer},
    collector::{
        AlbumMeta, Collector, ImageData, ImageMeta, Param, Registry, URL_FROM_TEXT_RE,
        URL_FROM_URL_RE,
    },
    http_proxy::ProxiedClient,
    storage::{cloudflare_kv::CFStorage, KVStorage},
    stream::{AsyncStream, Buffered},
    telegraph::{
        types::{Node, NodeElement, NodeElementAttr, Page, PageCreate, Tag},
        RandomAccessToken, Telegraph, TelegraphError, MAX_SINGLE_FILE_SIZE,
    },
    util::match_first_group,
};

const ERR_THRESHOLD: usize = 10;
const BATCH_LEN_THRESHOLD: usize = 20;
const BATCH_SIZE_THRESHOLD: usize = 5 * 1024 * 1024;
const DEFAULT_CONCURRENT: usize = 20;

#[derive(thiserror::Error, Debug)]
pub enum UploadError<SE> {
    #[error("stream error {0}")]
    Stream(SE),
    #[error("telegraph error {0}")]
    Reqwest(#[from] TelegraphError),
}

pub struct Synchronizer<C = CFStorage> {
    tg: Telegraph<RandomAccessToken, ProxiedClient>,
    limit: Option<usize>,

    author_name: Option<String>,
    author_url: Option<String>,
    cache_ttl: Option<usize>,

    registry: Registry,
    cache: C,
}

impl<CACHE> Synchronizer<CACHE>
where
    CACHE: KVStorage<String>,
{
    // cache ttl is 45 days
    const DEFAULT_CACHE_TTL: usize = 3600 * 24 * 45;

    pub fn new(
        tg: Telegraph<RandomAccessToken, ProxiedClient>,
        registry: Registry,
        cache: CACHE,
    ) -> Self {
        Self {
            tg,
            limit: None,
            author_name: None,
            author_url: None,
            cache_ttl: None,
            registry,
            cache,
        }
    }

    pub fn with_concurrent_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn with_author<S: Into<String>>(mut self, name: Option<S>, url: Option<S>) -> Self {
        self.author_name = name.map(Into::into);
        self.author_url = url.map(Into::into);
        self
    }

    pub fn with_cache_ttl(mut self, ttl: Option<usize>) -> Self {
        self.cache_ttl = ttl;
        self
    }

    pub async fn delete_cache(&self, key: &str) -> anyhow::Result<()> {
        self.cache.delete(key).await
    }

    pub async fn sync<C: Collector>(&self, path: String) -> anyhow::Result<String>
    where
        Registry: Param<C>,
        C::FetchError: Into<anyhow::Error> + Send + 'static,
        C::StreamError:
            Into<anyhow::Error> + std::fmt::Debug + std::fmt::Display + Send + Sync + 'static,
        C::ImageStream: Send + 'static,
        <C::ImageStream as AsyncStream>::Future: Send + 'static,
    {
        // check cache
        let cache_key = format!("{}|{}", C::name(), path);
        if let Ok(Some(v)) = self.cache.get(&cache_key).await {
            tracing::info!("[cache] hit key {cache_key}");
            return Ok(v);
        }
        tracing::info!("[cache] miss key {cache_key}");

        let collector: &C = self.registry.get();
        let (meta, stream) = collector.fetch(path).await.map_err(Into::into)?;
        let page = self
            .sync_stream(meta, stream)
            .await
            .map_err(anyhow::Error::from)?;

        // set cache
        let _ = self
            .cache
            .set(
                cache_key,
                page.url.clone(),
                Some(self.cache_ttl.unwrap_or(Self::DEFAULT_CACHE_TTL)),
            )
            .await;
        Ok(page.url)
    }

    pub async fn sync_stream<S, SE>(
        &self,
        meta: AlbumMeta,
        stream: S,
    ) -> Result<Page, UploadError<SE>>
    where
        SE: Send + std::fmt::Debug + 'static,
        S: AsyncStream<Item = Result<(ImageMeta, ImageData), SE>>,
        S::Future: Send + 'static,
    {
        let buffered_stream = Buffered::new(stream, self.limit.unwrap_or(DEFAULT_CONCURRENT));
        let r = self.inner_sync_stream(meta, buffered_stream).await;
        match &r {
            Ok(p) => {
                tracing::info!("[sync] sync success with url {}", p.url);
            }
            Err(e) => {
                tracing::error!("[sync] sync fail! {e:?}");
            }
        }
        r
    }

    async fn inner_sync_stream<S, SE>(
        &self,
        meta: AlbumMeta,
        mut stream: S,
    ) -> Result<Page, UploadError<SE>>
    where
        S: AsyncStream<Item = Result<(ImageMeta, ImageData), SE>>,
    {
        let mut err_count = 0;
        let mut uploaded = Vec::new();

        let mut buffer = ImageBuffer::new();

        // in this big loop, we will download images, and upload them in batch.
        // then, all meta info will be saved in `uploaded`.
        loop {
            // TODO: load images one by one is too slow!
            // We can spawn a background task(FuturesUnordered) and use channel, but expose as AsyncStream,
            // which does not require changes on consuming side.

            // 1. download images in batch
            while let Some(fut) = stream.next() {
                let data = match fut.await {
                    Err(e) => {
                        err_count += 1;
                        if err_count > ERR_THRESHOLD {
                            return Err(UploadError::Stream(e));
                        }
                        continue;
                    }
                    Ok(d) => {
                        err_count = 0;
                        d
                    }
                };

                // if the data size is too big to upload, we will discard it.
                if data.1.len() >= MAX_SINGLE_FILE_SIZE {
                    tracing::error!("Too big file, discarded. Meta: {:?}", data.0);
                    continue;
                }

                buffer.push(data);
                if buffer.len() > BATCH_LEN_THRESHOLD || buffer.size() > BATCH_SIZE_THRESHOLD {
                    break;
                }
            }
            // all data is uploaded, and no data to process.
            // just break the big loop.
            if buffer.is_empty() {
                break;
            }

            // 2. upload the batch
            let (full_data, size) = buffer.swap();
            let image_count = full_data.len();
            tracing::debug!("download {image_count} images with size {size}, will upload them",);

            let (meta, data) = full_data
                .into_iter()
                .map(|(a, b)| (a, b.as_ref().to_owned()))
                .unzip::<_, _, Vec<_>, Vec<_>>();
            let medium = self.tg.upload(data).await?;
            err_count = 0;

            // 3. add to uploaded
            tracing::debug!("upload {image_count} images with size {size}, medium: {medium:?}");
            uploaded.extend(
                meta.into_iter()
                    .zip(medium.into_iter().map(|x| x.src))
                    .map(|(meta, src)| UploadedImage { meta, src }),
            );
        }

        // create telegraph page, or multi pages
        // Telegraph has 64K limit, since our estimate is not accurate, here we use 48K.
        const PAGE_SIZE_LIMIT: usize = 48 * 1024;
        let mut chunks = Vec::with_capacity(8);
        chunks.push(Vec::new());
        let mut last_chunk_size = 0;
        for item in uploaded.into_iter().map(Into::<Node>::into) {
            let item_size = item.estimate_size();
            if last_chunk_size + item_size > PAGE_SIZE_LIMIT {
                chunks.push(Vec::new());
                last_chunk_size = 0;
            }
            last_chunk_size += item_size;
            chunks.last_mut().unwrap().push(item);
        }

        let mut last_page: Option<Page> = None;
        let title = meta.name.replace('|', "");
        while let Some(last_chunk) = chunks.pop() {
            let mut content = last_chunk;
            write_footer(
                &mut content,
                meta.link.as_str(),
                last_page.as_ref().map(|p| p.url.as_str()),
            );
            let title = match chunks.len() {
                0 => title.clone(),
                n => format!("{}-Page{}", title, n + 1),
            };
            tracing::debug!("create page with content: {content:?}");
            let page = self
                .tg
                .create_page(&PageCreate {
                    title,
                    content,
                    author_name: self
                        .author_name
                        .clone()
                        .or_else(|| meta.authors.as_ref().map(|x| x.join(", "))),
                    author_url: self.author_url.clone(),
                })
                .await
                .map_err(UploadError::Reqwest)?;

            last_page = Some(page);
        }
        Ok(last_page.unwrap())
    }
}

fn write_footer(content: &mut Vec<Node>, original_link: &str, next_page: Option<&str>) {
    if let Some(page) = next_page {
        content.push(np!(na!(@page, nt!("Next Page"))));
    }
    content.push(np!(
        nt!("Generated by "),
        na!(@"https://github.com/qini7-sese/eh2telegraph", nt!("eh2telegraph"))
    ));
    content.push(np!(
        nt!("Original link: "),
        na!(@original_link, nt!(original_link))
    ));
}

impl Synchronizer {
    pub fn match_url_from_text(content: &str) -> Option<&str> {
        match_first_group(&URL_FROM_TEXT_RE, content)
    }

    pub fn match_url_from_url(content: &str) -> Option<&str> {
        match_first_group(&URL_FROM_URL_RE, content)
    }
}

impl DataSized for (ImageMeta, ImageData) {
    #[inline]
    fn size(&self) -> usize {
        self.1.size()
    }
}

struct UploadedImage {
    #[allow(unused)]
    meta: ImageMeta,
    src: String,
}

// Size: {"tag":"img","attrs":{"src":"https://telegra.ph..."}}
impl From<UploadedImage> for Node {
    fn from(i: UploadedImage) -> Self {
        Node::new_image(format!("https://telegra.ph{}", i.src))
    }
}
