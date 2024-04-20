use bytes::Bytes;
use regex::Regex;
use reqwest::Response;

use crate::http_client::HttpRequestBuilder;

#[inline]
pub fn match_first_group<'a>(regexp: &'a Regex, content: &'a str) -> Option<&'a str> {
    regexp.captures(content).map(|c| {
        c.get(1)
            .expect("regexp is matched but no group 1 found")
            .as_str()
    })
}

#[inline]
pub async fn get_bytes<C: HttpRequestBuilder>(client: &C, link: &str) -> reqwest::Result<Bytes> {
    client
        .get_builder(link)
        .send()
        .await
        .and_then(Response::error_for_status)?
        .bytes()
        .await
}

#[inline]
pub async fn get_string<C: HttpRequestBuilder>(client: &C, link: &str) -> reqwest::Result<String> {
    client
        .get_builder(link)
        .send()
        .await
        .and_then(Response::error_for_status)?
        .text()
        .await
}
