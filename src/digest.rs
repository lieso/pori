use serde::Deserialize;
use serde_json::{self, Value};
use std::collections::HashMap;
use std::error::Error;

use crate::prelude::*;

pub const JSON_SCHEMA: &str = r#"
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "digest",
  "type": "object",
  "description": "A digest is a type of digital platform or webpage that aggregates and presents a curated collection of content, typically organized in a concise and accessible format. It may be algorithmically generated, or derived from user submissions. These often feature summaries, snippets, or headlines that provide a quick overview of the content. Digests are designed to facilitate easy browsing and discovery, allowing users to quickly scan through a variety of topics and delve deeper into those of interest. They often include user interactions such as comments, votes, or recommendations",
  "properties": {
    "title": {
      "type": "string",
      "description": "The title of the website or page."
    },
    "entries": {
      "type": "array",
      "description": "A list of content items on the page.",
      "items": {
        "type": "object",
        "description": "A content item object",
        "properties": {
          "title": {
            "type": "string",
            "description": "The title of the content item."
          },
          "content": {
            "type": "string",
            "description": "The main content or description of the item."
          },
          "url": {
            "type": "string",
            "format": "uri",
            "description": "The URL to the full content or external link."
          },
          "discussionUrl": {
            "type": "string",
            "format": "uri",
            "description": "The URL to the comments or discussion of the content item."
          },
          "author": {
            "type": "object",
            "description": "Information about the author of the content item.",
            "properties": {
              "name": {
                "type": "string",
                "description": "The name of the author."
              },
              "url": {
                "type": "string",
                "format": "uri",
                "description": "The URL to the author's profile page."
              }
            }
          },
          "timestamp": {
            "type": "string",
            "format": "date-time",
            "description": "The date and time when the item was published or created."
          },
          "score": {
            "type": "string",
            "description": "The score or ranking of the content item, if applicable."
          },
          "tags": {
            "type": "array",
            "description": "A list of tags or categories associated with the content item.",
            "items": {
              "type": "string",
              "description": "A tag item"
            }
          }
        }
      }
    }
  }
}
"#;

#[derive(Deserialize, Debug)]
pub struct Digest {
    pub title: Option<String>,
    pub entries: Vec<ContentItem>,
}

impl Digest {
    pub fn get_json_schema() -> &'static str {
        JSON_SCHEMA
    }
}

#[derive(Deserialize, Debug)]
pub struct ContentItem {
    pub title: Option<String>,
    pub content: Option<String>,
    pub url: Option<String>,
    pub discussion_url: Option<String>,
    pub author: Option<Author>,
    pub timestamp: Option<String>,
    pub score: Option<String>,
    pub tags: Option<String>,
}

impl ContentItem {
    pub fn to_details_string(&self) -> String {
        let mut result = String::new();

        if let Some(content) = &self.content {
            result = format!("{} - {}", result, content);
        }

        if let Some(url) = &self.url {
            result = format!("{} - {}", result, url);
        }

        if let Some(discussion_url) = &self.discussion_url {
            result = format!("{} - {}", result, discussion_url);
        }

        if let Some(timestamp) = &self.timestamp {
            result = format!("{} - {}", result, timestamp);
        }

        if let Some(score) = &self.score {
            result = format!("{} - {}", result, score);
        }

        result
    }
}

#[derive(Deserialize, Debug)]
pub struct Author {
    pub name: Option<String>,
    pub url: Option<String>,
}

pub fn deserialize_to_digest(json_data: &str) -> Result<Digest, Box<dyn Error>> {
    let value: Value = serde_json::from_str(json_data)?;

    if let Some(obj) = value.as_object() {
        if let Some(digest_value) = obj.get("digest") {
            return serde_json::from_value(digest_value.clone()).map_err(|e| e.into());
        }
    }

    serde_json::from_value(value).map_err(|e| e.into())
}

