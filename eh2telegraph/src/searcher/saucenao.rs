use std::{borrow::Cow, str::FromStr};

use futures::Future;
use ipnet::Ipv6Net;
use regex::Regex;
use reqwest::{
    multipart::{self, Part},
    Response,
};

use crate::http_client::{GhostClient, HttpRequestBuilder};

use super::ImageSearcher;

lazy_static::lazy_static! {
    static ref SEARCH_ELEMENT_RE: Regex = Regex::new(r#"<tr><td class="resulttableimage">(.*?)</tr>"#).unwrap();
    static ref S_URL_RE: Regex = Regex::new(r#"src="(https://.*?)""#).unwrap();
    static ref TITLE_RE: Regex = Regex::new(r#"<div class="resulttitle"><strong>(.*?)</strong>"#).unwrap();
    static ref SIM_RE: Regex = Regex::new(r#"<div class="resultsimilarityinfo">(\d+)\.?\d*%</div>"#).unwrap();
    static ref SITE_PARSE_RE: Regex = Regex::new(r#"saucenao\.com/(res/pixiv(_historical)?/\d+/manga/(?P<pixiv_id>\d+)_)|(ehentai/\w+/\w+/(?P<ehentai_fhash>\w+))|(res/nhentai/(?P<nhentai_id>\d+))"#).unwrap();
}

macro_rules! extract_first {
    ($re: expr, $input: expr, $err_msg: expr) => {
        $re.captures($input)
            .ok_or_else(|| anyhow::anyhow!($err_msg))?
            .get(1)
            .expect("regexp is matched but no group 1 found")
            .as_str()
    };
}

macro_rules! extract_first_opt {
    ($re: expr, $input: expr, $default: expr) => {
        match $re.captures($input) {
            Some(t) => t
                .get(1)
                .expect("regexp is matched but no group 1 found")
                .as_str(),
            None => $default,
        }
    };
}

/// Saucenao searcher.
/// Note: even saucenao resolves to an ipv6 address, we still use force resolving.
#[derive(Debug, Clone)]
pub struct SaucenaoSearcher {
    client: GhostClient,
}

impl SaucenaoSearcher {
    pub fn new(prefix: Option<Ipv6Net>) -> Self {
        Self {
            client: GhostClient::builder()
                .with_cf_resolve(&["saucenao.com", "e-hentai.org"])
                .build(prefix),
        }
    }

    pub fn new_from_config() -> Self {
        Self {
            client: GhostClient::builder()
                .with_cf_resolve(&["saucenao.com", "e-hentai.org"])
                .build_from_config()
                .expect("unable to build client for saucenao"),
        }
    }

    async fn do_search<C: HttpRequestBuilder>(
        client: &C,
        file: Part,
    ) -> anyhow::Result<SaucenaoOutput> {
        let response = client
            .post_builder("https://saucenao.com/search.php")
            .multipart(multipart::Form::new().part("file", file))
            .send()
            .await
            .and_then(Response::error_for_status)?
            .text()
            .await?;
        // check if the response is as expected
        if !response.contains("<title>Sauce Found?</title>") {
            return Err(anyhow::anyhow!("saucenao response is not as expected"));
        }
        SaucenaoOutput::from_str(&response)
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum SaucenaoParsed {
    EHentai(String),
    NHentai(String),
    Pixiv(String),
    Other,
}

#[derive(Debug, Clone)]
pub struct SaucenaoOuputElement {
    pub raw_url: String,
    pub name: String,
    pub similarity: u8,

    pub parsed: SaucenaoParsed,
}

#[derive(Debug, Clone)]
pub struct SaucenaoOutput {
    pub data: Vec<SaucenaoOuputElement>,
}

impl IntoIterator for SaucenaoOutput {
    type Item = <Vec<SaucenaoOuputElement> as IntoIterator>::Item;
    type IntoIter = <Vec<SaucenaoOuputElement> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl<T> ImageSearcher<T> for SaucenaoSearcher
where
    T: Into<Cow<'static, [u8]>>,
{
    type SeacheError = anyhow::Error;
    type SearchOutput = SaucenaoOutput;
    type FetchFuture = impl Future<Output = Result<Self::SearchOutput, Self::SeacheError>>;

    fn search(&self, data: T) -> Self::FetchFuture {
        let file_part = Part::bytes(data).file_name("image.jpg");
        let client = self.client.clone();
        async move { Self::do_search(&client, file_part).await }
    }
}

impl FromStr for SaucenaoOutput {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut data = Vec::new();

        // match all
        for cap in SEARCH_ELEMENT_RE.captures_iter(s) {
            let s = cap
                .get(1)
                .expect("regexp is matched but no group 1 found")
                .as_str();
            let element = SaucenaoOuputElement::from_str(s)?;
            data.push(element);
        }
        // sort
        data.sort_unstable_by(|a, b| b.similarity.cmp(&a.similarity));

        Ok(Self { data })
    }
}

impl FromStr for SaucenaoOuputElement {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // raw_url examples:
        // https://img1.saucenao.com/res/pixiv/7594/manga/75943246_p1.jpg?auth=dKnHvUUPQ0wi8G6yv-HWZQ&exp=1645560000
        // https://img1.saucenao.com/res/seiga_illust/157/1574075.jpg?auth=KKGjLqCUyouLUKieJ5g4Rw&exp=1645560000
        // https://img3.saucenao.com/ehentai/c5/17/c517710f0654ea883df1e0fea7117c671fb03bc1.jpg?auth=Hu-H_4c3lTKdh_rtZJv50w&exp=1645560000
        let raw_url =
            extract_first!(S_URL_RE, s, "unable to parse saucenao result url").to_string();
        let name = extract_first_opt!(TITLE_RE, s, "NO TITLE").to_string();
        let similarity =
            extract_first!(SIM_RE, s, "unable to parse saucenao result similarity").parse()?;

        let parsed = SITE_PARSE_RE
            .captures(&raw_url)
            .and_then(|cap| {
                if let Some(pixiv) = cap.name("pixiv_id") {
                    return Some(SaucenaoParsed::Pixiv(pixiv.as_str().to_string()));
                }
                if let Some(eh) = cap.name("ehentai_fhash") {
                    return Some(SaucenaoParsed::EHentai(eh.as_str().to_string()));
                }
                if let Some(nh) = cap.name("nhentai_id") {
                    return Some(SaucenaoParsed::NHentai(nh.as_str().to_string()));
                }
                None
            })
            .unwrap_or(SaucenaoParsed::Other);

        Ok(Self {
            raw_url,
            name,
            similarity,
            parsed,
        })
    }
}
