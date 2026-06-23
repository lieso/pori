use headless_chrome::Browser;
use parversion::prelude::{ExecutionContext, Metadata, Options};
use parversion::provider::sqlite::SqliteProvider;
use parversion::translation;
use parversion::normalization;
use parversion::document_format::DocumentFormat;
use parversion::document::{DocumentType, DocumentRole};
use std::sync::Arc;

use crate::content::{Content, ContentPayload, ContentType};
use crate::prelude::*;

#[derive(Clone)]
pub struct Context {
    browser: Browser,
    provider: Arc<SqliteProvider>,
    url: Option<String>,
    mode: Mode,
}

impl Context {
    pub fn new(provider: Arc<SqliteProvider>, browser: Browser) -> Self {
        Context {
            browser,
            provider,
            url: None,
            mode: Mode::NavigationInput,
        }
    }

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

    pub fn open_using_system(&self, url: String) {
        let _ = std::process::Command::new("open").arg(&url).spawn();
    }

    pub async fn open(
        &self,
        execution_context: Arc<ExecutionContext>,
        regenerate: bool,
    ) -> Result<ContentPayload, Errors> {
        log::trace!("In open");

        let document = self.fetch_document()?;
        let url = self
            .get_url()
            .ok_or_else(|| Errors::UnexpectedError("URL not found".into()))?;

        let options = Options {
            origin: Some(url.clone()),
            date: None,
            regenerate,
        };

        let metadata = Metadata {
            document_type: Some(DocumentType::Html),
            origin: url.clone(),
            role: DocumentRole::Instance,
        };

        let document_format = DocumentFormat::default();

        let normalized_document = normalization::normalize_text_to_document(
            self.provider.clone(),
            document.clone(),
            &options,
            &metadata,
            &document_format,
            execution_context.clone()
        )
        .await
        .map_err(|e| {
            Errors::NormalizationError(format!("Could not normalize paage: {:?}", e))
        })?;

        let content_type: Option<ContentType> = Content::match_content_names(normalized_document.metadata.semantic_content_types.unwrap());

        if let Some(content_type) = content_type {
            let json_schema = Content::get_json_schema_by_content_type(&content_type);

            let options = Options {
                origin: Some(url.clone()),
                date: None,
                regenerate,
            };

            let schema_metadata = Metadata {
                document_type: Some(DocumentType::Json),
                origin: url.clone(),
                role: DocumentRole::Schema,
            };

            let result = translation::translate_text_to_package(
                self.provider.clone(),
                (document, &metadata),
                (json_schema.to_string(), &schema_metadata),
                &options,
                &document_format,
                execution_context.clone(),
            )
            .await
            .map_err(|e| {
                Errors::TranslationError(format!("Could not translate content: {:?}", e))
            })?;

            let payload = Content::content_data_to_payload(&content_type, &result.document.data)
                .expect("Could not deserialize translated content");

            Ok(payload)
        } else {
            Err(Errors::UnexpectedContentType(
                "Could not match content type with any of expected types".to_string(),
            ))
        }
    }

    fn fetch_document(&self) -> Result<String, Errors> {
        let url = self
            .get_url()
            .ok_or_else(|| Errors::UnexpectedError("URL not found".into()))?;

        if !is_valid_url(&url) {
            log::warn!("url is not valid: {}", url);
            return Err(Errors::InvalidUrl);
        }

        let tab = self
            .browser
            .new_tab()
            .map_err(|e| Errors::BrowserError(format!("Could not create new tab: {}", e)))?;

        tab.navigate_to(&url)
            .map_err(|e| Errors::BrowserError(format!("Could not navigate: {}", e)))?;

        tab.wait_until_navigated()
            .map_err(|e| Errors::BrowserError(format!("Could not wait: {}", e)))?;

        let document = tab
            .evaluate("document.documentElement.outerHTML", false)
            .map_err(|e| Errors::BrowserError(format!("Could not evaluate JavaScript: {}", e)))?
            .value
            .ok_or_else(|| Errors::BrowserError("No content returned".into()))?
            .as_str()
            .ok_or_else(|| Errors::BrowserError("Content is not a string".into()))?
            .to_string();

        Ok(document)
    }
}
