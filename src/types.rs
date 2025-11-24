#[derive(Clone, Debug)]
pub enum Errors {
    UnexpectedError(String),
    ProviderError(String),
    BrowserError(String),
}

#[derive(Clone, Debug)]
pub enum Mode {
    Normal,
    Search
}
