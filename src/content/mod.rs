use crate::prelude::*;

pub mod digest;

use crate::content::digest::{Digest, deserialize_to_digest};

#[derive(Debug, Clone)]
pub enum ContentType {
    Digest,
}

const DIGEST_NAMES: &[&str] = &["digest", "feed", "aggregator", "list"];

#[derive(Debug, Clone)]
pub enum ContentPayload {
    Digest(digest::Digest),
}

pub struct Content {}

impl Content {
    pub fn match_content_names(content_names: Vec<String>) -> Option<ContentType> {
        let known_names = [
            (DIGEST_NAMES, ContentType::Digest)
        ];

        for (names, content_type) in known_names {
            let has_match = content_names.iter().any(|item| names.contains(&item.to_lowercase().as_str()));

            if has_match {
                return Some(content_type);
            }
        }

        None
    }

    pub fn get_json_schema_by_content_type(content_type: &ContentType) -> &str {
        match content_type {
            ContentType::Digest => Digest::get_json_schema(),
        }
    }

    pub fn content_data_to_payload(content_type: &ContentType, data: &str) -> Result<ContentPayload, Errors> {
        match content_type {
            ContentType::Digest => {
                let digest: Digest = deserialize_to_digest(data).map_err(|e| {
                    Errors::TranslationError(format!("Could not deserialize translated content: {}", e))
                })?;

                Ok(ContentPayload::Digest(digest))
            }
        }
    }
}
