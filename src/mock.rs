use std::collections::HashMap;
use serde_json::Value;
use crate::prelude::*;
use crate::digest::{Digest, deserialize_to_digest};

/// Mock data provider for testing UI without parversion
pub struct MockProvider {
    test_data: HashMap<String, String>,
}

impl MockProvider {
    pub fn new() -> Result<Self, Errors> {
        let test_data_json = include_str!("../test_digests.json");
        let data: Value = serde_json::from_str(test_data_json)
            .map_err(|e| Errors::UnexpectedError(format!("Failed to parse test data: {}", e)))?;

        let mut test_data = HashMap::new();

        if let Some(obj) = data.as_object() {
            for (key, value) in obj.iter() {
                test_data.insert(
                    key.clone(),
                    serde_json::to_string(value)
                        .map_err(|e| Errors::UnexpectedError(format!("Failed to serialize test data: {}", e)))?
                );
            }
        }

        Ok(MockProvider { test_data })
    }

    /// Get mock digest data based on URL pattern matching
    pub fn get_digest_for_url(&self, url: &str) -> Result<Digest, Errors> {
        // Simple URL matching - you can customize this logic
        let key = if url.contains("news.ycombinator") || url.contains("hackernews") || url.contains("hn") {
            "hacker_news"
        } else if url.contains("reddit.com/r/programming") || url.contains("reddit") {
            "reddit_programming"
        } else if url.contains("blog") || url.contains("tech") {
            "tech_blog"
        } else if url.contains("minimal") {
            "minimal"
        } else if url.contains("empty") {
            "empty"
        } else {
            // Default to hacker_news if no pattern matches
            "hacker_news"
        };

        let json_data = self.test_data.get(key)
            .ok_or_else(|| Errors::UnexpectedError(format!("No test data found for key: {}", key)))?;

        deserialize_to_digest(json_data)
            .map_err(|e| Errors::TranslationError(format!("Failed to deserialize mock data: {}", e)))
    }

    /// Get all available test data keys
    pub fn available_keys(&self) -> Vec<String> {
        self.test_data.keys().cloned().collect()
    }

    /// Get digest by explicit key (for testing specific scenarios)
    pub fn get_digest_by_key(&self, key: &str) -> Result<Digest, Errors> {
        let json_data = self.test_data.get(key)
            .ok_or_else(|| Errors::UnexpectedError(format!("No test data found for key: {}", key)))?;

        deserialize_to_digest(json_data)
            .map_err(|e| Errors::TranslationError(format!("Failed to deserialize mock data: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_provider_loads() {
        let provider = MockProvider::new().unwrap();
        assert!(provider.available_keys().len() > 0);
    }

    #[test]
    fn test_get_digest_for_url() {
        let provider = MockProvider::new().unwrap();

        // Test HN URL
        let digest = provider.get_digest_for_url("https://news.ycombinator.com").unwrap();
        assert_eq!(digest.title, Some("Hacker News".to_string()));
        assert!(digest.entries.len() > 0);

        // Test Reddit URL
        let digest = provider.get_digest_for_url("https://reddit.com/r/programming").unwrap();
        assert_eq!(digest.title, Some("r/programming".to_string()));
    }

    #[test]
    fn test_get_digest_by_key() {
        let provider = MockProvider::new().unwrap();

        let digest = provider.get_digest_by_key("minimal").unwrap();
        assert_eq!(digest.entries.len(), 1);

        let digest = provider.get_digest_by_key("empty").unwrap();
        assert_eq!(digest.entries.len(), 0);
    }
}
