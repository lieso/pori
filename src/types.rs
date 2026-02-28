#[derive(Clone, Debug)]
pub enum Errors {
    UnexpectedError(String),
    ProviderError(String),
    BrowserError(String),
    TranslationError(String),
}

#[derive(Clone, Debug)]
pub enum Mode {
    Interaction,
    Navigation,
    NavigationInput,
}

impl Mode {
    pub fn as_str(&self) -> &str {
        match self {
            Mode::Navigation => "navigation",
            Mode::NavigationInput => "input",
            Mode::Interaction => "interaction",
        }
    }
}
