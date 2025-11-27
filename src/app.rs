use std::sync::{Arc, RwLock};
use std::io;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    symbols::border,
    text::{Line, Text, Span},
    style::{
        palette::tailwind::{BLUE, GREEN, SLATE},
        Color, Modifier, Style, Stylize,
    },
    widgets::{Block, Borders, Paragraph, Widget, List, ListDirection, ListItem, ListState, StatefulWidget},
    DefaultTerminal, Frame,
    prelude::*
};

use crate::prelude::*;
use crate::context::Context;
use crate::digest::Digest;

const TEXT_FG_COLOR: Color = SLATE.c200;

struct EntryListItem {
    title: String,
}

struct EntryList {
    items: Vec<EntryListItem>,
    state: ListState,
}

pub struct App {
    context: Arc<RwLock<Context>>,
    digest: Option<Digest>,
    exit: bool,
    entry_list: EntryList,
}

impl App {
    pub fn new(context: Arc<RwLock<Context>>) -> Self {
        Self {
            context,
            exit: false,
            digest: None,
            entry_list: EntryList {
                items: vec![],
                state: ListState::default(),
            }
        }
    }

    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events().await?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    async fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event).await
            }
            _ => {}
        };
        Ok(())
    }

    async fn handle_navigation_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => {
                let mut lock = write_lock!(self.context);
                lock.append_char('q');
            },
            KeyCode::Esc => {
                let mut lock = write_lock!(self.context);
                lock.set_mode(Mode::Interaction);
            }
            KeyCode::Char('/') => {
                let mut lock = write_lock!(self.context);
                lock.append_char('/');
            }
            KeyCode::Char(c) => {
                let mut lock = write_lock!(self.context);
                lock.append_char(c);
            }
            KeyCode::Backspace => {
                let mut lock = write_lock!(self.context);
                lock.remove_last_char();
            }
            KeyCode::Enter => {
                let mut lock = write_lock!(self.context);

                let digest = lock.visit().await.expect("Could not visit");

                self.digest = Some(digest);

                lock.set_mode(Mode::Interaction);
            },
            _ => {}
        }
    }

    async fn handle_interaction_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => {
                self.exit();
            },
            KeyCode::Char('/') => {
                let mut lock = write_lock!(self.context);
                lock.set_mode(Mode::Navigation);
            }
            _ => {}
        }
    }

    async fn handle_key_event(&mut self, key_event: KeyEvent) {
        let current_mode = {
            let lock = read_lock!(self.context);

            lock.get_mode().clone()
        };

        match current_mode {
            Mode::Navigation => self.handle_navigation_key_event(key_event).await,
            Mode::Interaction => self.handle_interaction_key_event(key_event).await,
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl App {
    fn render_header(&mut self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" pori ".bold());
        let block = Block::bordered()
            .title(title.centered())
            .border_set(border::THICK);

        let url = {
            let lock = read_lock!(self.context);
            lock.url_to_string()
        };

        let search_text = {
            let lock = read_lock!(self.context);
            
            if let Mode::Navigation = lock.get_mode() {
                Text::from(vec![Line::from(vec![
                    Span::raw("Navigate: ").white(),
                    Span::raw(url),
                ])])
            } else {
                Text::from(vec![Line::from(vec![
                    Span::raw(url),
                ])])
            }
        };

        Paragraph::new(search_text)
            .centered()
            .block(block)
            .render(area, buf);
    }

    fn render_entries(&mut self, area: Rect, buf: &mut Buffer) {
        if let Some(digest) = &self.digest {
            let items: Vec<ListItem> = digest
                .entries
                .iter()
                .map(|entry| {

                    let title = entry
                        .title
                        .clone()
                        .unwrap_or_else(|| "Untitled".to_string());

                    let line = Line::styled(title, Style::default().fg(TEXT_FG_COLOR));

                    ListItem::new(line)
                })
                .collect();

            let list = List::new(items)
                .block(Block::bordered().title("Entries"))
                .style(Style::new().bold())
                .highlight_style(Style::new().italic())
                .highlight_symbol(">>")
                .repeat_highlight_symbol(true)
                .direction(ListDirection::TopToBottom);

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

        if let Some(digest) = &self.digest {
            self.render_entries(layout[1], buf);
        }
    }
}
