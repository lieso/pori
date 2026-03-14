use headless_chrome::Browser;
use parversion::document::DocumentType;
use parversion::organization::organize_text_to_basis_graph;
use parversion::prelude::{ExecutionContext, Metadata, Options};
use parversion::provider::yaml::YamlFileProvider;
use parversion::translation;
use std::sync::Arc;

use crate::content::{Content, ContentPayload, ContentType};
use crate::prelude::*;

#[derive(Clone)]
pub struct Context {
    browser: Browser,
    provider: Arc<YamlFileProvider>,
    url: Option<String>,
    mode: Mode,
}

impl Context {
    pub fn new(provider: Arc<YamlFileProvider>, browser: Browser) -> Self {
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

        // Do we regenerate when guessing content type? How to avoid wasteful re-interpretation?
        let options = Options {
            origin: Some(url.clone()),
            date: None,
            regenerate: false,
        };

        let metadata = Metadata {
            document_type: Some(DocumentType::Html),
            origin: url.clone(),
        };

        let meta_context = organize_text_to_basis_graph(
            self.provider.clone(),
            document.clone(),
            &options,
            &metadata,
            ExecutionContext::new(),
        )
        .await
        .expect("Could not obtain basis graph");

        let basis_graph = meta_context.read().unwrap().get_basis_graph();

        let mut content_names = basis_graph.clone().unwrap().aliases.clone();
        content_names.push(basis_graph.clone().unwrap().name.clone());

        log::info!("Content has the following names: {:?}", content_names);

        let content_type: Option<ContentType> = Content::match_content_names(content_names);

        if let Some(content_type) = content_type {
            let json_schema = Content::get_json_schema_by_content_type(&content_type);

            let options = Options {
                origin: Some(url.clone()),
                date: None,
                regenerate,
            };

            let result = translation::translate_text_to_package(
                self.provider.clone(),
                document,
                &options,
                &metadata,
                json_schema,
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
