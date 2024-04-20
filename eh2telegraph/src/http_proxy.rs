use std::time::Duration;

use reqwest::header::HeaderValue;

use crate::config;

const CONFIG_KEY: &str = "proxy";
const TIMEOUT: Duration = Duration::from_secs(30);

#[derive(serde::Deserialize, Clone, Debug, Default)]
struct ProxyConfig {
    endpoint: String,
    authorization: String,
}

/// RequestBuilder helps create a Request with proxy.
/// Note: Users should not replace headers.
#[derive(Debug, Clone, Default)]
pub struct ProxiedClient {
    proxy: Option<Proxy>,
    inner: reqwest::Client,
}

#[derive(Debug, Clone)]
pub struct Proxy {
    endpoint: reqwest::Url,
    authorization: HeaderValue,
}

impl ProxiedClient {
    pub fn new(endpoint: &str, authorization: &str) -> Self {
        let proxy = Some(Proxy {
            endpoint: endpoint.parse().expect("unable to parse proxy endpoint"),
            authorization: authorization
                .parse()
                .expect("unable to parse proxy authorization"),
        });
        Self {
            proxy,
            inner: reqwest::Client::builder()
                .timeout(TIMEOUT)
                .build()
                .expect("unable to build reqwest client"),
        }
    }

    pub fn new_from_config() -> Self {
        match config::parse::<ProxyConfig>(CONFIG_KEY)
            .expect("unable to parse proxy config(key is {CONFIG_KEY})")
        {
            Some(cfg) => Self::new(&cfg.endpoint, &cfg.authorization),
            None => {
                tracing::warn!("initialized ProxiedClient without proxy config");
                Self::default()
            }
        }
    }

    pub fn with_default_headers(self, headers: reqwest::header::HeaderMap) -> Self {
        Self {
            inner: reqwest::Client::builder()
                .timeout(TIMEOUT)
                .default_headers(headers)
                .build()
                .expect("unable to build reqwest client"),
            ..self
        }
    }
}

macro_rules! impl_method {
    ($method: ident) => {
        pub fn $method(&self, url: &str) -> reqwest::RequestBuilder {
            match &self.proxy {
                Some(p) => self
                    .inner
                    .$method(p.endpoint.clone())
                    .header("X-Forwarded-For", url)
                    .header("X-Authorization", p.authorization.clone()),
                None => self.inner.$method(url),
            }
        }
    };
}

impl ProxiedClient {
    impl_method!(get);
    impl_method!(post);
    impl_method!(head);
    impl_method!(put);
    impl_method!(delete);
    impl_method!(patch);

    pub fn request(&self, method: reqwest::Method, url: &str) -> reqwest::RequestBuilder {
        match &self.proxy {
            Some(p) => self
                .inner
                .request(method, p.endpoint.clone())
                .header("X-Forwarded-For", url)
                .header("X-Authorization", p.authorization.clone()),
            None => self.inner.request(method, url),
        }
    }
}
