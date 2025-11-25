use std::sync::{Arc, RwLock};
use std::io;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text, Span},
    widgets::{Block, Borders, Paragraph, Widget, List, ListDirection, ListItem},
    DefaultTerminal, Frame,
    prelude::*
};

use crate::prelude::*;
use crate::context::Context;
use crate::digest::Digest;

pub struct App {
    context: Arc<RwLock<Context>>,
    digest: Option<Digest>,
    exit: bool,
}

impl App {
    pub fn new(context: Arc<RwLock<Context>>) -> Self {
        App {
            context,
            exit: false,
            digest: None,
        }
    }

    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events().await?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
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

    async fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => {
                let mode;
                {
                    let mut lock = write_lock!(self.context);
                    mode = lock.get_mode().clone();

                    if let Mode::Search = mode {
                        lock.append_char('q');
                    }
                }

                if let Mode::Normal = mode {
                    self.exit();
                }
            },
            KeyCode::Esc => {
                let mut lock = write_lock!(self.context);
                lock.set_mode(Mode::Normal);
            }
            KeyCode::Char('/') => {
                let mut lock = write_lock!(self.context);
                if let Mode::Normal = lock.get_mode() {
                    lock.set_mode(Mode::Search);
                }
                if let Mode::Search = lock.get_mode() {
                    lock.append_char('/');
                }
            }
            KeyCode::Char(c) => {
                let mut lock = write_lock!(self.context);
                if let Mode::Search = lock.get_mode() {
                    lock.append_char(c);
                }
            }
            KeyCode::Backspace => {
                let mut lock = write_lock!(self.context);
                if let Mode::Search = lock.get_mode() {
                    lock.remove_last_char();
                }
            }
            KeyCode::Enter => {
                let mut lock = write_lock!(self.context);
                if let Mode::Search = lock.get_mode() {
                    let digest = lock.visit().await.expect("Could not visit");

                    self.digest = Some(digest);
                }
            },
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Percentage(5),
                Constraint::Percentage(95),
            ])
            .split(area);





        let title = Line::from(" pori ".bold());
        let block = Block::bordered()
            .title(title.centered())
            .border_set(border::THICK);

        let url = {
            let lock = read_lock!(self.context);
            lock.url_to_string()
        };

        let search_text = Text::from(vec![Line::from(vec![
            Span::raw("Search: "),
            Span::raw(url),
        ])]);

        Paragraph::new(search_text)
            .centered()
            .block(block)
            .render(layout[0], buf);


        if let Some(digest) = &self.digest {

            let items: Vec<String> = digest.entries.iter().map(|entry| {
                entry
                    .title
                    .clone()
                    .unwrap_or_else(|| "Untitled".to_string())
            }).collect();

            let list = List::new(items)
                .block(Block::bordered().title("Entries"))
                .style(Style::new().white())
                .highlight_style(Style::new().italic())
                .highlight_symbol(">>")
                .repeat_highlight_symbol(true)
                .direction(ListDirection::TopToBottom);


            Widget::render(list, layout[1], buf);

        }
    }
}
