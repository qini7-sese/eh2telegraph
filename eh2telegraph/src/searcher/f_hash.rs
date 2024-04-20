use ipnet::Ipv6Net;
use regex::Regex;

use crate::{
    collector::exhentai::EXCollector,
    http_client::{GhostClient, GhostClientBuilder},
    util::{get_string, match_first_group},
};

lazy_static::lazy_static! {
    static ref EHENTAI_URL_RE: Regex = Regex::new(r#"<a href="(https://e(-|x)hentai\.org/g/\w+/[\w-]+)/">"#).unwrap();
}
/// FHashConverter can convert f-hash(usually comes from a search result) to the first gallery url.
/// Works for both e-hentai and ex-hentai.
pub struct FHashConvertor {
    client: GhostClient,
    raw_client: reqwest::Client,
}

impl FHashConvertor {
    pub fn new(prefix: Option<Ipv6Net>) -> Self {
        Self {
            client: GhostClientBuilder::default()
                .with_cf_resolve(&["e-hentai.org"])
                .build(prefix),
            raw_client: EXCollector::new_from_config()
                .expect("unable to build ex-client")
                .get_client(),
        }
    }

    pub fn new_from_config() -> Self {
        Self {
            client: GhostClientBuilder::default()
                .with_cf_resolve(&["e-hentai.org"])
                .build_from_config()
                .expect("unable to build client for f-hash convertor"),
            raw_client: EXCollector::new_from_config()
                .expect("unable to build ex-client")
                .get_client(),
        }
    }

    // TODO: impl a trait?
    pub async fn convert_to_gallery(&self, f_hash: &str) -> anyhow::Result<String> {
        tracing::info!("[f-hash] converting hash {f_hash}");
        // find in e-hentai
        let url = format!("https://e-hentai.org/?f_shash={f_hash}&f_sh=on&f_sname=on&f_stags=on&f_sh=on&f_spf=&f_spt=&f_sfl=on&f_sfu=on&f_sft=on");
        let text = get_string(&self.client, &url).await?;

        if let Some(url) = match_first_group(&EHENTAI_URL_RE, &text) {
            tracing::info!("[f-hash] hash {f_hash} -> {url}");
            return Ok(url.to_string());
        }

        // find in exhentai
        let url = format!("https://exhentai.org/?f_shash={f_hash}&f_sh=on&f_sname=on&f_stags=on&f_sh=on&f_spf=&f_spt=&f_sfl=on&f_sfu=on&f_sft=on");
        let text = get_string(&self.raw_client, &url).await?;

        if let Some(url) = match_first_group(&EHENTAI_URL_RE, &text) {
            tracing::info!("[f-hash] hash {f_hash} -> {url}");
            return Ok(url.to_string());
        }

        tracing::info!("[f-hash] hash {f_hash} not found");
        Err(anyhow::anyhow!("not found in e-hentai or exhentai"))
    }
}
