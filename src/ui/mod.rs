use ratatui::{
    buffer::Buffer,
    layout::Rect,
};
use crossterm::event::{KeyEvent};

mod digest;

use crate::content::digest::Digest;
use crate::content::ContentPayload;
use digest::DigestApp;

pub enum ContentType {
    Digest,
}

pub struct UI {
    content_type: Option<ContentType>,
    digest: Option<DigestApp>,
}

impl UI {
    pub fn new() -> Self {
        UI {
            content_type: None,
            digest: None,
        }
    }

    pub fn set_content_type(&mut self, content_type: ContentType) {
        match content_type {
            ContentType::Digest => {
                self.content_type = Some(ContentType::Digest);
                self.digest = Some(DigestApp::new());
            },
        }
    }
    
    pub fn get_json_schema(&self) -> &str {
        match &self.content_type.as_ref().unwrap() {
            ContentType::Digest => Digest::get_json_schema(),
        }
    }

    pub fn run(&mut self, content_payload: ContentPayload) {
        match content_payload {
            ContentPayload::Digest(digest) => {
                if let Some(app) = &mut self.digest {
                    app.run(digest);
                } else {
                    self.set_content_type(ContentType::Digest);
                    let app = &mut self.digest.as_mut().unwrap();
                    app.run(digest);
                }
            },
        }
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer) {
        match self.content_type {
            Some(ContentType::Digest) => {
                if let Some(app) = &mut self.digest {
                    app.render(area, buf);
                }
            },
            None => {}
        }
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        match self.content_type {
            Some(ContentType::Digest) => {
                if let Some(app) = &mut self.digest {
                    app.handle_key_event(key_event);
                }
            },
            None => {}
        }
    }
}
