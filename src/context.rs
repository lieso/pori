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
    
    pub fn url_to_string(&self) -> String {
        if let Some(url) = &self.url {
            url.clone()
        } else {
            String::new()
        }
    }

    pub fn append_char(&mut self, ch: char) {
        if self.url.is_none() {
            self.url = Some(String::new());
        }

        if let Some(url) = &mut self.url {
            url.push(ch);
        }
    }

    pub fn remove_last_char(&mut self) {
        if let Some(url) = &mut self.url {
            url.pop();
        }
    }

    pub fn get_mode(&self) -> &Mode {
        &self.mode
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    pub fn new() -> Self {
        Context {
            url: None,
            mode: Mode::Search
        }
    }
}
