use std::sync::{Arc, RwLock};
use std::io;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text, Span},
    widgets::{Block, Paragraph, Widget},
    DefaultTerminal, Frame,
};

use crate::prelude::*;
use crate::context::Context;

#[derive(Debug)]
pub struct App {
    context: Arc<RwLock<Context>>,
    exit: bool,
}

impl App {
    pub fn new(context: Arc<RwLock<Context>>) -> Self {
        App {
            context,
            exit: false,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
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
                // No-op
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
        let title = Line::from(" pori ".bold());
        let instructions = Line::from(vec![
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
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
            .render(area, buf);
    }
}
