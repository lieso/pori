use std::collections::HashMap;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    symbols::border,
    text::{Line, Text, Span},
    style::{
        palette::tailwind::{GREEN, BLUE, GRAY},
        Color, Style, Stylize,
    },
    widgets::{Block, Paragraph, Widget, List, ListItem, ListState, StatefulWidget},
    DefaultTerminal, Frame,
    prelude::*
};

use crate::content::digest::Digest;

struct EntryListItem {
    title: String,
}

struct EntryList {
    items: Vec<EntryListItem>,
    state: ListState,
}

pub struct DigestApp {
    digest: Option<Digest>,
    entry_list: EntryList,
    column_ratios: HashMap<String, u32>,
    column_count: usize,
    selected_column_index: usize,
}

impl DigestApp {
    pub fn new() -> Self {
        Self {
            digest: None,
            entry_list: EntryList {
                items: vec![],
                state: ListState::default()
            },
            column_ratios: HashMap::new(),
            column_count: 0,
            selected_column_index: 0,
        }
    }

    pub fn on_digest(&mut self, digest: Digest) {
        let column_count = digest
            .entries
            .iter()
            .fold(0, |acc, entry| {
                let field_presence: Vec<bool> = vec![
                    entry.content.is_some(),
                    entry.url.is_some(),
                    entry.discussion_url.is_some(),
                    entry.author.is_some(),
                    entry.timestamp.is_some(),
                    entry.score.is_some(),
                ];

                let count = field_presence.iter().fold(0, |acc, &present| {
                    if present { acc + 1 } else { acc }
                });

                if count > acc {
                    count
                } else {
                    acc
                }
            });

        let total_lengths: HashMap<String, usize> = digest
            .entries
            .iter()
            .fold(HashMap::new(), |mut acc, entry| {
                if let Some(content) = &entry.content {
                    *acc.entry("content".to_string()).or_insert(0) += content.len();
                }

                if let Some(url) = &entry.url {
                    *acc.entry("url".to_string()).or_insert(0) += url.len();
                }

                if let Some(discussion_url) = &entry.discussion_url {
                    *acc.entry("discussion_url".to_string()).or_insert(0) += discussion_url.len();
                }

                if let Some(author) = &entry.author {
                    let mut total = 0;
                    if let Some(name) = &author.name {
                        total += name.len();
                    }
                    if let Some(url) = &author.url {
                        total += url.len();
                    }

                    *acc.entry("author".to_string()).or_insert(0) += total;
                }

                if let Some(timestamp) = &entry.timestamp {
                    *acc.entry("timestamp".to_string()).or_insert(0) += timestamp.len();
                }

                if let Some(score) = &entry.score {
                    *acc.entry("score".to_string()).or_insert(0) += score.len();
                }

                acc
            });

        let average_lengths: HashMap<String, f64> = total_lengths
            .into_iter()
            .map(|(field_name, total_len)| {
                (field_name, total_len as f64 / digest.entries.len() as f64)
            })
            .collect();

        let min_average = average_lengths
            .values()
            .fold(f64::INFINITY, |a, &b| a.min(b));

        let column_ratios: HashMap<String, u32> = average_lengths
            .into_iter()
            .map(|(k, val)| {
                let normalized = if min_average > 0.0 {
                    (val / min_average).round()
                } else {
                    0.0
                };
                (k, normalized as u32)
            })
            .collect();

        log::info!("Using column ratios: {:?}", column_ratios);
        log::info!("Using column count: {}", column_count);

        self.column_ratios = column_ratios;
        self.column_count = column_count;
        self.digest = Some(digest);
    }
}
