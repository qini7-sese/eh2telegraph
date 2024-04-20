#![feature(type_alias_impl_trait)]
#![feature(impl_trait_in_assoc_type)]

#[macro_use]
pub mod telegraph;

pub mod buffer;
pub mod collector;
pub mod config;
pub mod http_client;
pub mod http_proxy;
pub mod indexer;
pub mod searcher;
pub mod storage;
pub mod stream;
pub mod sync;
pub mod tls;
pub mod util;
