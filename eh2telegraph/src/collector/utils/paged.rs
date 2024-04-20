use reqwest::Response;

use crate::http_client::HttpRequestBuilder;

pub trait PageFormatter {
    fn format_n(&self, n: usize) -> String;
}

pub trait PageIndicator {
    fn is_last_page(&self, content: &str, next_page: usize) -> bool;
}

#[derive(thiserror::Error, Debug)]
pub enum PagedError {
    #[error("reqwest error")]
    Reqwest(#[from] reqwest::Error),
}

pub struct Paged<T> {
    next_page: usize,
    page_indicator: T,
}

impl<T> Paged<T> {
    pub fn new(init_page: usize, page_indicator: T) -> Self {
        Self {
            next_page: init_page,
            page_indicator,
        }
    }
}

impl<T> Paged<T>
where
    T: PageFormatter,
{
    pub async fn next<C>(&mut self, client: &C) -> Result<String, PagedError>
    where
        C: HttpRequestBuilder,
    {
        let url = self.page_indicator.format_n(self.next_page);

        let content = client
            .get_builder(&url)
            .send()
            .await
            .and_then(Response::error_for_status)?
            .text()
            .await?;
        self.next_page += 1;
        Ok(content)
    }
}

impl<T> Paged<T>
where
    T: PageFormatter + PageIndicator,
{
    /// pages returns at least one element if it is Ok
    pub async fn pages<C>(&mut self, client: &C) -> Result<Vec<String>, PagedError>
    where
        C: HttpRequestBuilder,
    {
        let mut results = Vec::new();
        loop {
            let content = self.next(client).await?;
            let terminated = self.page_indicator.is_last_page(&content, self.next_page);
            results.push(content);
            if terminated {
                return Ok(results);
            }
        }
    }
}
