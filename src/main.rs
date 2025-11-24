use ratatui::{text::Text, Frame};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use headless_chrome::{Browser, LaunchOptions};

fn main() {
    let mut terminal = ratatui::init();

    let mut input = String::new();





    let browser = Browser::new(
       LaunchOptions {
           headless: true,
           ..Default::default()
       }
    ).expect("Could not launch browser");

    let tab = browser.new_tab().expect("Could not create new tab");

    tab.navigate_to("https://www.rust-lang.org/").expect("Could not navigate");
    tab.wait_until_navigated().expect("Could not wait until navigate");







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
