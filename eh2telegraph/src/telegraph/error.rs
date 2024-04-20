// Partly borrowed from https://github.com/Aloxaf/telegraph-rs/blob/master/src/error.rs

use serde::Deserialize;

use super::types::MediaInfo;

#[derive(thiserror::Error, Debug)]
pub enum TelegraphError {
    #[error("api error {0}")]
    Api(String),
    #[error("reqwest error {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("unexpected server result")]
    Server,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum ApiResult<T> {
    Ok { result: T },
    Err { error: String },
}

impl<T> From<ApiResult<T>> for Result<T, TelegraphError> {
    fn from(r: ApiResult<T>) -> Self {
        match r {
            ApiResult::Ok { result: v } => Ok(v),
            ApiResult::Err { error: e, .. } => Err(TelegraphError::Api(e)),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum UploadResult {
    Ok(Vec<MediaInfo>),
    Err { error: String },
}

impl From<UploadResult> for Result<Vec<MediaInfo>, TelegraphError> {
    fn from(r: UploadResult) -> Self {
        match r {
            UploadResult::Ok(v) => Ok(v),
            UploadResult::Err { error } => Err(TelegraphError::Api(error)),
        }
    }
}
