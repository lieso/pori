pub enum ContentType {
    Digest,
}

pub struct UI {
    content_type: Option<ContentType>,
}

impl UI {
    pub fn new() -> Self {
        UI {
            content_type: None,
        }
    }
}
