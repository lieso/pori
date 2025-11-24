#[derive(Clone, Debug)]
pub enum Errors {
    UnexpectedError(String),
    ProviderError(String),
}

#[derive(Clone, Debug)]
pub enum Mode {
    Normal,
    Search
}
