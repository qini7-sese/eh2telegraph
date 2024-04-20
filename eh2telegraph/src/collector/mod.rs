//! Built-in collectors and trait.

use once_cell::sync::Lazy;
use regex::Regex;
use std::future::Future;

use crate::stream::AsyncStream;

use self::{e_hentai::EHCollector, exhentai::EXCollector, nhentai::NHCollector};

pub mod utils;

pub mod e_hentai;
pub mod exhentai;
pub mod nhentai;
pub mod pixiv;

#[derive(Debug, Clone)]
pub struct ImageMeta {
    pub id: String,
    pub url: String,
    pub description: Option<String>,
}

pub type ImageData = bytes::Bytes;

#[derive(Debug, Clone)]
pub struct AlbumMeta {
    pub link: String,
    pub name: String,
    pub class: Option<String>,
    pub description: Option<String>,
    pub authors: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
}

/// Generic collector.
/// The `async fetch` returns the result of `AlbumMeta` and `ImageStream`.
/// By exposing `ImageStream`, we can fetch the images lazily. For low
/// memory VM, it will keep only a small amount in memory.
pub trait Collector {
    type FetchError;
    type StreamError;
    type ImageStream: AsyncStream<Item = Result<(ImageMeta, ImageData), Self::StreamError>>;

    fn name() -> &'static str;
    fn fetch(
        &self,
        path: String,
    ) -> impl Future<Output = Result<(AlbumMeta, Self::ImageStream), Self::FetchError>>;
}

pub(crate) static URL_FROM_TEXT_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"((https://exhentai\.org/g/\w+/[\w-]+)|(https://e-hentai\.org/g/\w+/[\w-]+)|(https://nhentai\.net/g/\d+)|(https://nhentai\.to/g/\d+))"#).unwrap()
});
pub(crate) static URL_FROM_URL_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^((https://exhentai\.org/g/\w+/[\w-]+)|(https://e-hentai\.org/g/\w+/[\w-]+)|(https://nhentai\.net/g/\d+)|(https://nhentai\.to/g/\d+))"#).unwrap()
});

#[derive(Debug, Clone)]
pub struct Registry {
    eh: EHCollector,
    nh: NHCollector,
    ex: EXCollector,
}

pub trait Param<T> {
    fn get(&self) -> &T;
}

impl Param<EHCollector> for Registry {
    fn get(&self) -> &EHCollector {
        &self.eh
    }
}

impl Param<NHCollector> for Registry {
    fn get(&self) -> &NHCollector {
        &self.nh
    }
}

impl Param<EXCollector> for Registry {
    fn get(&self) -> &EXCollector {
        &self.ex
    }
}

impl Registry {
    pub fn new_from_config() -> Self {
        Self {
            eh: EHCollector::new_from_config().expect("unable to build e-hentai collector"),
            nh: NHCollector::new_from_config().expect("unable to build nhentai collector"),
            ex: EXCollector::new_from_config().expect("unable to build exhentai collector"),
        }
    }
}
