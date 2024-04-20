// A wrapper for reqwest to provide ability to bind to random ip.
// Since apparently I can not afford a ipv4 subnet, here I assume ipv6.
// Using he.net tunnel broker works fine.
// Setup:
// 1. sudo ip add add local 2001:x:x::/48 dev lo
// 2. sudo ip route add local 2001:x:x::/48 dev he-ipv6
// 3. Set net.ipv6.ip_nonlocal_bind=1

pub const UAS: [&str; 45] = [
    "Mozilla/5.0 (X11; Linux x86_64; rv:123.0) Gecko/20100101 Firefox/123.0",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:107.0) Gecko/20100101 Firefox/107.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 12.3; x64; rv:107.0) Gecko/20100101 Firefox/107.0",
    "Mozilla/5.0 (Linux; Android 12; SM-G988B Build/SP1A.210812.016; wv) Gecko/20100101 Firefox/107.0 Mobile/15E148",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_4 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) FxiOS/107.0 Mobile/15E148",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_4 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) CriOS/108.0.5359.95 Mobile/15E148 Safari/604.1",
    "Mozilla/5.0 (Linux; Android 12; SM-G988B Build/SP1A.210812.016; wv) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/108.0.5359.95 Mobile Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 12.3; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/108.0.5359.95 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/108.0.5359.95 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.3",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_4) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.3.1 Safari/605.1.15",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 12.3; x64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.4 Safari/605.1.15",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36 Edg/122.0.2365.80",
    "Mozilla/5.0 (X11; CrOS x86_64 15633.69.0) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.6045.212 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36 OPR/108.0.0.0",
    "Mozilla/5.0 (Linux; Android 14) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.6261.105 Mobile Safari/537.36",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 17_4 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.3.1 Mobile/15E148 Safari/604.1",
    "Mozilla/4.0 (compatible; MSIE 7.0; Windows NT 5.1; Trident/4.0; TencentTraveler 4.0; .NET CLR 2.0.50727)",
    "Mozilla/5.0 (Linux; U; Android 11; zh-cn; PDRM00 Build/RKQ1.200903.002) AppleWebKit/537.36 (KHTML, like Gecko) Version/4.0 Chrome/70.0.3538.80 Mobile Safari/537.36 HeyTapBrowser/40.7.27.2",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/108.0.5359.71 Safari/537.36 Edg/108.0.1462.42",
    "Mozilla/5.0 (Linux; U; Android 12; zh-cn; 2201122C Build/SKQ1.211006.001) AppleWebKit/537.36 (KHTML, like Gecko) Version/4.0 Chrome/89.0.4389.116 Mobile Safari/537.36 XiaoMi/MiuiBrowser/15.9.18 swan-mibrowser",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/108.0.5359.95 Safari/537.36 OPR/74.0.3911.104",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/98.0.4758.102 Safari/537.36 QQBrowser/10.8.4313.400",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/98.0.4758.102 Safari/537.36 360SE/13.0.1920.1000",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/98.0.4758.102 Safari/537.36 UCBrowser/13.2.8.1300",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_4 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Mobile/15E148 QQ/10.8.4313.400 NetType/WIFI",
    "Mozilla/5.0 (Linux; Android 12; SM-G988B Build/SP1A.210812.016; wv) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/98.0.4758.102 Mobile Safari/537.36 QQBrowser/10.8.4313.400",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 12.3; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/98.0.4758.102 Safari/537.36 QQBrowser/10.8.4313.400",
    "Mozilla/5.0 (Linux; Android 6.0; Nexus 5 Build/MRA58N) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/99.0.4844.51 Mobile Safari/537.36 MicroMessenger/7.0.1",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/110.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/109.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 6.1; WOW64; rv:54.0) Gecko/20100101 Firefox/74.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 13.3; rv:109.0) Gecko/20100101 Firefox/109.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 13_2_1) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.4 Safari/605.1.15",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 12_6_2) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/110.0.5481.77 Safari/537.36",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/110.0.5481.77 Safari/537.36",
    "Mozilla/5.0 (X11; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/109.0",
    "Mozilla/5.0 (X11; Ubuntu; Linux i686; rv:109.0) Gecko/20100101 Firefox/109.0",
    "Mozilla/5.0 (Linux; Android 13; SM-G973F) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/110.0.5481.63 Mobile Safari/537.36",
    "Mozilla/5.0 (Linux; Android 9; SM-G960F) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/109.0.5414.117 Mobile Safari/537.36",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 16_3_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.3 Mobile/15E148 Safari/604.1",
    "Mozilla/5.0 (iPad; CPU OS 16_3_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.0 Mobile/15E148 Safari/604.1",
    "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/70.0.3538.102 YaBrowser/19.1.3.322 Yowser/2.5 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/70.0.3538.25 Safari/537.36 Core/1.70.3706.400 QQBrowser/10.4.3620.400",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/80.0.3987.132 Whale/2.7.99.13 Safari/537.36",
];
const CONFIG_KEY: &str = "http";
const TIMTOUT: Duration = Duration::from_secs(30);

use std::{
    net::{IpAddr, Ipv6Addr, SocketAddr},
    ops::{Deref, DerefMut},
    sync::Arc,
    time::Duration,
};

use ipnet::Ipv6Net;
use reqwest::header;
use rustls::ClientConfig;

use crate::{config, tls::WhitelistVerifier};

const CF_ADDR: Ipv6Addr = Ipv6Addr::new(0x2606, 0x4700, 0x4700, 0, 0, 0, 0, 0x1111);
const TG_ADDR: Ipv6Addr = Ipv6Addr::new(0x2001, 0x67c, 0x4e8, 0x1033, 0x1, 0x100, 0, 0xa);

pub fn rand_ua() -> &'static str {
    use rand::seq::SliceRandom;
    use rand::thread_rng;
    UAS.choose(&mut thread_rng()).expect("Empty UA List!")
}

pub trait HttpRequestBuilder {
    fn get_builder(&self, url: &str) -> reqwest::RequestBuilder;
    fn post_builder(&self, url: &str) -> reqwest::RequestBuilder;
}

macro_rules! gen_impl {
    ($ty: ty) => {
        impl HttpRequestBuilder for $ty {
            #[inline]
            fn get_builder(&self, url: &str) -> reqwest::RequestBuilder {
                self.get(url).header(reqwest::header::USER_AGENT, rand_ua())
            }

            #[inline]
            fn post_builder(&self, url: &str) -> reqwest::RequestBuilder {
                self.post(url)
                    .header(reqwest::header::USER_AGENT, rand_ua())
            }
        }
    };
}

gen_impl!(reqwest::Client);
gen_impl!(crate::http_proxy::ProxiedClient);
gen_impl!(GhostClient);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, derive_more::From, derive_more::Into)]
pub struct Ipv6Net2(Ipv6Net);

impl<'de> serde::Deserialize<'de> for Ipv6Net2 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use std::str::FromStr;
        let data = String::deserialize(deserializer)?;
        Ipv6Net::from_str(&data)
            .map(Ipv6Net2)
            .map_err(serde::de::Error::custom)
    }
}

#[derive(serde::Deserialize, Clone, Debug, Default)]
struct HTTPConfig {
    ipv6_prefix: Option<Ipv6Net2>,
}

#[derive(Debug, Default)]
pub struct GhostClientBuilder {
    mapping: Vec<(&'static str, SocketAddr)>,
    headers: Option<header::HeaderMap>,
}

impl GhostClientBuilder {
    pub fn with_default_headers(self, headers: header::HeaderMap) -> Self {
        Self {
            headers: Some(headers),
            ..self
        }
    }

    pub fn with_cf_resolve(mut self, domains: &[&'static str]) -> Self {
        let cf = SocketAddr::new(IpAddr::V6(CF_ADDR), 443);
        for &domain in domains.iter() {
            self.mapping.push((domain, cf));
        }
        self
    }

    #[deprecated = "telegra.ph has fixed it and returns 501 when using ipv6"]
    pub fn with_tg_resolve(mut self) -> Self {
        let tg = SocketAddr::new(IpAddr::V6(TG_ADDR), 443);
        self.mapping.push(("telegra.ph", tg));
        self.mapping.push(("api.telegra.ph", tg));
        self
    }

    pub fn build(self, prefix: Option<Ipv6Net>) -> GhostClient {
        let inner = GhostClient::build_raw(&prefix, &self.mapping, self.headers.clone());
        GhostClient {
            prefix,
            mapping: Arc::new(self.mapping),
            headers: self.headers,
            inner,
        }
    }

    pub fn build_from_config(self) -> anyhow::Result<GhostClient> {
        let config: HTTPConfig = config::parse(CONFIG_KEY)?.unwrap_or_default();
        let prefix = config.ipv6_prefix.map(Into::into);
        Ok(self.build(prefix))
    }
}

#[derive(Debug, Default)]
pub struct GhostClient {
    prefix: Option<Ipv6Net>,
    mapping: Arc<Vec<(&'static str, SocketAddr)>>,
    headers: Option<header::HeaderMap>,

    inner: reqwest::Client,
}

impl GhostClient {
    pub fn builder() -> GhostClientBuilder {
        GhostClientBuilder::default()
    }
}

impl Clone for GhostClient {
    fn clone(&self) -> Self {
        let inner = Self::build_raw(&self.prefix, &self.mapping, self.headers.clone());
        Self {
            prefix: self.prefix,
            mapping: self.mapping.clone(),
            headers: self.headers.clone(),
            inner,
        }
    }
}

impl Deref for GhostClient {
    type Target = reqwest::Client;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for GhostClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl GhostClient {
    fn build_raw(
        net: &Option<Ipv6Net>,
        mapping: &[(&'static str, SocketAddr)],
        headers: Option<header::HeaderMap>,
    ) -> reqwest::Client {
        let mut builder = reqwest::Client::builder().timeout(TIMTOUT);

        if let Some(headers) = headers {
            builder = builder.default_headers(headers);
        }

        if let Some(net) = net {
            let addr: u128 = net.addr().into();
            let prefix_len = net.prefix_len();
            let mask = !u128::MAX
                .checked_shl((128 - prefix_len) as u32)
                .unwrap_or(u128::MIN);

            // use random ipv6
            let rand: u128 = rand::Rng::gen(&mut rand::thread_rng());
            let addr = IpAddr::V6(Ipv6Addr::from(rand & mask | addr));
            builder = builder.local_address(addr);

            // apply resolve
            for (domain, addr) in mapping {
                builder = builder.resolve(domain, *addr);
            }

            // not add preconfigured tls
            // let tls_config = TLS_CFG.clone();
            // builder = builder.use_preconfigured_tls(tls_config);
        }

        builder.build().expect("build reqwest client failed")
    }

    pub fn refresh(&mut self) {
        self.inner = Self::build_raw(&self.prefix, &self.mapping, self.headers.clone());
    }
}

lazy_static::lazy_static! {
    // here we only meet telegra.ph with wrong tls config, so we write them as fixed values.
    static ref TLS_CFG: ClientConfig = WhitelistVerifier::new(["telegram.org"]).into();
}

#[cfg(test)]
mod tests {
    use super::TLS_CFG;

    #[ignore]
    #[tokio::test]
    async fn test_tls() {
        let tls_config = TLS_CFG.clone();
        // use a telegram.org ip address(normally it fails in browser)
        let cli = reqwest::Client::builder()
            .resolve("api.telegra.ph", "149.154.167.99:443".parse().unwrap())
            .use_preconfigured_tls(tls_config)
            .build()
            .unwrap();
        let resp = cli
            .get("https://api.telegra.ph/getPage")
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
    }
}
