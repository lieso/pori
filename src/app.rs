use std::io;
use std::time::Duration;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    symbols::border,
    text::{Line, Text, Span},
    style::{
        palette::tailwind::{GREEN, BLUE},
        Color, Style, Stylize,
    },
    widgets::{Block, Paragraph, Widget, List, ListItem, ListState, StatefulWidget},
    DefaultTerminal, Frame,
    prelude::*
};
use tokio::sync::mpsc;

use crate::prelude::*;
use crate::context::Context;
use crate::digest::{Digest, minimize_url};

const ENTRY_DETAILS_FG_COLOR: Color = GREEN.c500;

struct EntryListItem {
    title: String,
}

struct EntryList {
    items: Vec<EntryListItem>,
    state: ListState,
}

pub struct App {
    context: Context,
    digest: Option<Digest>,
    exit: bool,
    loading: bool,
    entry_list: EntryList,
    digest_tx: mpsc::UnboundedSender<Digest>,
    digest_rx: mpsc::UnboundedReceiver<Digest>,
}

impl App {
    pub fn new(context: Context) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        Self {
            context,
            exit: false,
            loading: false,
            digest: None,
            entry_list: EntryList {
                items: vec![],
                state: ListState::default(),
            },
            digest_tx: tx,
            digest_rx: rx,
        }
    }

    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events().await?;

            if let Ok(digest) = self.digest_rx.try_recv() {
                self.digest = Some(digest);
                self.loading = false;
                self.context.set_mode(Mode::Interaction);
            }
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    async fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event).await
                }
                _ => {}
            };
        }
        Ok(())
    }

    async fn handle_navigation_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => {
                self.context.append_char('q');
            },
            KeyCode::Esc => {
                self.context.set_mode(Mode::Interaction);
            }
            KeyCode::Char('/') => {
                self.context.append_char('/');
            }
            KeyCode::Char(c) => {
                self.context.append_char(c);
            }
            KeyCode::Backspace => {
                self.context.remove_last_char();
            }
            KeyCode::Enter => {
                self.navigate().await;
            },
            _ => {}
        }
    }

    async fn handle_interaction_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('j') => {
                self.select_next();
            },
            KeyCode::Char('k') => {
                self.select_previous();
            },
            KeyCode::Char('q') => {
                self.exit();
            },
            KeyCode::Char('r') => {
                self.refresh().await;
            },
            KeyCode::Char('/') => {
                self.context.set_mode(Mode::Navigation);
            }
            _ => {}
        }
    }

    async fn handle_key_event(&mut self, key_event: KeyEvent) {
        match self.context.get_mode().clone() {
            Mode::Navigation => self.handle_navigation_key_event(key_event).await,
            Mode::Interaction => self.handle_interaction_key_event(key_event).await,
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    async fn refresh(&mut self) {
        self.navigate().await;
    }

    async fn navigate(&mut self) {
        self.loading = true;

        let context_clone = self.context.clone();
        let tx_clone = self.digest_tx.clone();

        tokio::spawn(async move {
            let digest: Digest = context_clone.visit().await.expect("Could not visit");

            tx_clone.send(digest).unwrap();
        });
    }

    fn select_previous(&mut self) {
        self.entry_list.state.select_previous();
    }

    fn select_next(&mut self) {
        self.entry_list.state.select_next();
    }
}

impl App {
    fn render_header(&mut self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" pori ".bold());
        let block = Block::bordered()
            .title(title.centered())
            .border_set(border::THICK);

        let url = self.context.url_to_string();

        let search_text = if let Mode::Navigation = self.context.get_mode() {
            Text::from(vec![Line::from(vec![
                Span::raw("Navigate: ").white(),
                Span::raw(url),
            ])])
        } else {
            Text::from(vec![Line::from(vec![
                Span::raw(url),
            ])])
        };

        Paragraph::new(search_text)
            .centered()
            .block(block)
            .render(area, buf);
    }

    fn render_body(&mut self, area: Rect, buf: &mut Buffer) {
        if self.loading {
            let loading_text = Paragraph::new("⏳ Loading...")
                .block(Block::bordered().title("Status"))
                .centered();
            loading_text.render(area, buf);
        } else if let Some(digest) = &self.digest {
            let items: Vec<ListItem> = digest
                .entries
                .iter()
                .enumerate()
                .map(|(index, entry)| {
                    let title = entry
                        .title
                        .clone()
                        .unwrap_or_else(|| "Untitled".to_string());

                    let title_line = Line::styled(
                        title,
                        Style::default().bold()
                    );

                    let mut spans = Vec::new();

                    if let Some(url) = &entry.url {
                        let minimized_url = minimize_url(&url);
                        spans.push(
                            Span::styled(format!("{}", minimized_url), Style::default().fg(BLUE.c500))
                        );
                    }

                    if let Some(score) = &entry.score {
                        spans.push(
                            Span::styled(format!(" {}", score), Style::default().fg(GREEN.c500))
                        );
                    }

                    if let Some(content) = &entry.content {
                        spans.push(
                            Span::styled(format!(" {}", content), Style::default().fg(GREEN.c500))
                        );
                    }

                    if let Some(discussion_url) = &entry.discussion_url {
                        spans.push(
                            Span::styled(format!(" {}", discussion_url), Style::default().fg(BLUE.c500))
                        );
                    }

                    if let Some(timestamp) = &entry.timestamp {
                        spans.push(
                            Span::styled(format!(" {}", timestamp), Style::default().fg(GREEN.c500))
                        );
                    }

                    if let Some(author) = &entry.author {
                        if let Some(author_name) = &author.name {
                            spans.push(
                                Span::styled(format!(" {}", author_name), Style::default().fg(GREEN.c500))
                            );
                        }
                    }

                    
                    let details_line = Line::from(spans.clone());

                    let text = Text::from(vec![title_line, details_line]);

                    ListItem::new(text)
                })
                .collect();

            let list = List::new(items)
                .block(Block::bordered().title("Entries"))
                .highlight_style(Style::new().italic())
                .highlight_symbol(">>")
                .repeat_highlight_symbol(true);

            StatefulWidget::render(list, area, buf, &mut self.entry_list.state);
        }
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(3),
                Constraint::Min(0)
            ])
            .split(area);

        self.render_header(layout[0], buf);
        self.render_body(layout[1], buf);
    }
}
