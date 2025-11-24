use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct Context {
    url: Option<String>,
    mode: Mode,
}

impl Context {
    pub fn get_url(&self) -> Option<String> {
        self.url.clone()
    }

    pub fn set_url(&mut self, url: String) {
        self.url = Some(url);
    }
    
    pub fn has_url(&self) -> bool {
        self.url.is_some()
    }

    pub fn new() -> Self {
        Context {
            url: None,
            mode: Mode::Search
        }
    }
}
