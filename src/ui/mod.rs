use crossterm::event::KeyEvent;
use ratatui::{buffer::Buffer, layout::Rect};

mod digest;

use crate::content::ContentPayload;
use crate::content::digest::Digest;
use crate::prelude::*;
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
            }
        }
    }

    //pub fn get_json_schema(&self) -> &str {
    //    match &self.content_type.as_ref().unwrap() {
    //        ContentType::Digest => Digest::get_json_schema(),
    //    }
    //}

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
            }
        }
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer) {
        match self.content_type {
            Some(ContentType::Digest) => {
                if let Some(app) = &mut self.digest {
                    app.render(area, buf);
                }
            }
            None => {}
        }
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> Option<Action> {
        match self.content_type {
            Some(ContentType::Digest) => {
                if let Some(app) = &mut self.digest {
                    return app.handle_key_event(key_event);
                }
            }
            None => {}
        }

        None
    }
}
