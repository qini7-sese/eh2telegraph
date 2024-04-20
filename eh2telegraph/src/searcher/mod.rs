pub mod f_hash;
pub mod saucenao;

pub trait ImageSearcher<T> {
    type SeacheError;
    type SearchOutput;
    type FetchFuture: std::future::Future<Output = Result<Self::SearchOutput, Self::SeacheError>>;

    fn search(&self, data: T) -> Self::FetchFuture;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[tokio::test]
    async fn demo() {
        let data = std::fs::read("./image.png").unwrap();
        let searcher = saucenao::SaucenaoSearcher::new(None);
        let r = searcher.search(data).await;
        println!("result: {r:?}");
    }
}
