use std::collections::HashMap;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{Line, Text, Span},
    style::{
        palette::tailwind::{GREEN, BLUE, GRAY},
        Style,
    },
    widgets::{List, ListItem, ListState, StatefulWidget},
};

use crate::prelude::*;
use crate::content::digest::Digest;

struct EntryList {
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
                state: ListState::default()
            },
            column_ratios: HashMap::new(),
            column_count: 0,
            selected_column_index: 0,
        }
    }

    pub fn run(&mut self, digest: Digest) {
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

    pub fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let Some(digest) = &self.digest else { return; };

        let width = area.width;
        let column_ratios_total = self.column_ratios.values().fold(0, |acc, &r| acc + r);
        let column_widths: HashMap<String, u16> = self.column_ratios
            .iter()
            .map(|(k, &v)| {
                (k.to_string(), width * (v as u16) / (column_ratios_total as u16))
            })
            .collect();

        let items: Vec<ListItem> = digest
            .entries
            .iter()
            .enumerate()
            .map(|(index, entry)| {
                let is_row_selected = self.entry_list.state.selected() == Some(index);

                let title = entry
                    .title
                    .clone()
                    .unwrap_or_else(|| "Untitled".to_string());

                let title_line = Line::styled(
                    title,
                    Style::default().fg(GRAY.c300).bold()
                );

                let mut spans = Vec::new();

                let col_style = |col_index: usize, base: Style| {
                    if is_row_selected && col_index == self.selected_column_index {
                        base.bold()
                    } else {
                        base
                    }
                };

                if let Some(url) = &entry.url {
                    let minimized_url = minimize_url(&url);
                    let width = column_widths.get("url").unwrap();
                    let style = col_style(spans.len(), Style::default().fg(BLUE.c500));
                    spans.push(Span::styled(fit_to_width(&minimized_url, *width as usize), style));
                }

                if let Some(score) = &entry.score {
                    let width = column_widths.get("score").unwrap();
                    let style = col_style(spans.len(), Style::default().fg(GREEN.c500));
                    spans.push(Span::styled(fit_to_width(&score, *width as usize), style));
                }

                if let Some(content) = &entry.content {
                    let width = column_widths.get("content").unwrap();
                    let style = col_style(spans.len(), Style::default().fg(GREEN.c500));
                    spans.push(Span::styled(fit_to_width(&content, *width as usize), style));
                }

                if let Some(discussion_url) = &entry.discussion_url {
                    let width = column_widths.get("discussion_url").unwrap();
                    let style = col_style(spans.len(), Style::default().fg(BLUE.c500));
                    spans.push(Span::styled(fit_to_width(&discussion_url, *width as usize), style));
                }

                if let Some(timestamp) = &entry.timestamp {
                    let width = column_widths.get("timestamp").unwrap();
                    let style = col_style(spans.len(), Style::default().fg(GREEN.c500));
                    spans.push(Span::styled(fit_to_width(&timestamp, *width as usize), style));
                }

                if let Some(author) = &entry.author {
                    if let Some(author_name) = &author.name {
                        let width = column_widths.get("author").unwrap();
                        let style = col_style(spans.len(), Style::default().fg(GREEN.c500));
                        spans.push(Span::styled(fit_to_width(&author_name, *width as usize), style));
                    }
                }

                let details_line = Line::from(spans.clone());

                let text = Text::from(vec![title_line, details_line, Line::from("")]);

                ListItem::new(text)
            })
        .collect();

        let list = List::new(items)
            .highlight_symbol(">>")
            .repeat_highlight_symbol(false);

        StatefulWidget::render(list, area, buf, &mut self.entry_list.state);
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('h') => {
                self.select_previous_column();
            },
            KeyCode::Char('j') => {
                self.select_next();
            },
            KeyCode::Char('k') => {
                self.select_previous();
            },
            KeyCode::Char('l') => {
                self.select_next_column();
            },
            _ => {}
        }
    }

    fn select_previous(&mut self) {
        self.entry_list.state.select_previous();
    }

    fn select_next(&mut self) {
        self.entry_list.state.select_next();
    }

    fn select_previous_column(&mut self) {
        self.selected_column_index = self.selected_column_index - 1;
    }

    fn select_next_column(&mut self) {
        self.selected_column_index = self.selected_column_index + 1;
    }
}

fn fit_to_width(s: &str, width: usize) -> String {
    if s.len() >= width { format!("{:.prec$}...", s, prec=width-1) }
    else { format!("{:<width$}", s, width=width) }
}
