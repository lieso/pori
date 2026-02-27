mod digest;

use crate::content::digest::Digest;
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
}
