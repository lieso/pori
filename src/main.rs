use ratatui::{text::Text, Frame};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use headless_chrome::{Browser, LaunchOptions};
use clap::{Arg, App};

mod context;
mod prelude;
mod types;

use crate::prelude::*;
use crate::context::Context;

const VERSION: &str = "0.0.0";
const PROGRAM_NAME: &str = "pori";

fn parse_arguments() -> clap::ArgMatches {
    App::new(PROGRAM_NAME)
        .version(VERSION)
        .arg(Arg::with_name("version")
            .short('v')
            .long("version")
            .help("Display program version"))
        .get_matches()
}

fn draw(frame: &mut Frame) {
    let text = Text::raw("Hello World!");
    frame.render_widget(text, frame.area());
}

async fn run() -> Result<(), Errors> {



    let matches = parse_arguments();

    if matches.is_present("version") {
        println!("{} {}", PROGRAM_NAME, VERSION);
        return Ok(());
    }


    let mut terminal = ratatui::init();






    let browser = Browser::new(
       LaunchOptions {
           headless: true,
           ..Default::default()
       }
    ).expect("Could not launch browser");

    let tab = browser.new_tab().expect("Could not create new tab");

    //tab.navigate_to("https://www.rust-lang.org/").expect("Could not navigate");
    //tab.wait_until_navigated().expect("Could not wait until navigate");







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

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        println!("Error occurred: {:?}", e);
        std::process::exit(1);
    }
    std::process::exit(0);
}
