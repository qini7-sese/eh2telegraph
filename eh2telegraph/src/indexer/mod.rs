// Indexer + Filters(FilterType+Value) -> EntryStream

#[derive(Debug, Clone)]
pub enum Filter {
    Name(String),
    Category(String),
}

#[derive(Debug, Clone)]
pub enum OrderBy {
    TimeDesc,
    ClickDesc,
}

pub trait Indexer {}
