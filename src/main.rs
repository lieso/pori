use ratatui::{text::Text, Frame};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};

fn main() {
    let mut terminal = ratatui::init();

    let mut input = String::new();

    loop {
        terminal.draw(draw).expect("failed to draw frame");

        match event::read().expect("Could not read event") {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Char('q') => break,
                _ => {}
            },
            _ => {}
        }

    }
    ratatui::restore();
}

fn draw(frame: &mut Frame) {
    let text = Text::raw("Hello World!");
    frame.render_widget(text, frame.area());
}
