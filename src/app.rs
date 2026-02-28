use std::io;
use std::time::Duration;
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
use tokio::sync::mpsc;
use std::collections::HashMap;

use crate::prelude::*;
use crate::context::Context;
use crate::content::digest::Digest;
use crate::ui::{UI, ContentType};
use crate::content::ContentPayload;

pub struct App {
    context: Context,
    ui: UI,
    exit: bool,
    loading: bool,
    tx: mpsc::UnboundedSender<ContentPayload>,
    rx: mpsc::UnboundedReceiver<ContentPayload>,
}

impl App {
    pub fn new(context: Context) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        Self {
            context,
            ui: UI::new(),
            exit: false,
            loading: false,
            tx: tx,
            rx: rx,
        }
    }

    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events().await?;

            if let Ok(content) = self.rx.try_recv() {
                self.ui.run(content);
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

    fn handle_navigation_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => {
                self.exit();
            },
            KeyCode::Char('r') => {
                self.refresh();
            },
            KeyCode::Enter => {
                self.navigate();
            },
            _ => {}
        }
    }

    fn handle_navigation_input_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char(c) => {
                self.context.append_char(c);
            }
            KeyCode::Backspace => {
                self.context.remove_last_char();
            }
            KeyCode::Enter => {
                self.navigate();
            },
            _ => {}
        }
    }

    fn handle_universal_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc => {
                self.context.set_mode(Mode::Navigation);
            }
            KeyCode::Char('/') => {
                self.context.set_mode(Mode::NavigationInput);
            }
            _ => {}
        }
    }

    async fn handle_key_event(&mut self, key_event: KeyEvent) {
        self.handle_universal_key_event(key_event);

        match self.context.get_mode().clone() {
            Mode::Navigation => self.handle_navigation_key_event(key_event),
            Mode::Interaction => self.ui.handle_key_event(key_event),
            Mode::NavigationInput => self.handle_navigation_input_key_event(key_event),
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn refresh(&mut self) {
        self.navigate();
    }

    fn navigate(&mut self) {
        self.loading = true;

        // TODO: infer content type ******************************************
        let content_type = ContentType::Digest;
        self.ui.set_content_type(content_type);
        // ******************************************

        let context_clone = self.context.clone();
        let tx_clone = self.tx.clone();
        let schema_clone = self.ui.get_json_schema().to_string();

        tokio::spawn(async move {
            let digest: Digest = context_clone
                .visit(&schema_clone)
                .await
                .expect("Could not visit");

            let content_payload = ContentPayload::Digest(digest);

            tx_clone.send(content_payload).unwrap();
        });
    }
}

impl App {
    fn render_header(&mut self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" pori ".bold());
        let block = Block::bordered()
            .title(title.centered())
            .border_set(border::THICK);

        let url = self.context.url_to_string();

        let search_text = if let Mode::NavigationInput = self.context.get_mode() {
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
        } else {
            self.ui.render(area, buf);
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
