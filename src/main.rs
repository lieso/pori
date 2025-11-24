use std::sync::{Arc, RwLock};
use ratatui;
use headless_chrome::{Browser, LaunchOptions};
use clap::{Arg, App as ClapApp};

mod app;
mod context;
mod macros;
mod prelude;
mod types;

use crate::prelude::*;
use crate::context::Context;
use crate::app::App;

const VERSION: &str = "0.0.0";
const PROGRAM_NAME: &str = "pori";

fn parse_arguments() -> clap::ArgMatches {
    ClapApp::new(PROGRAM_NAME)
        .version(VERSION)
        .arg(Arg::with_name("version")
            .short('v')
            .long("version")
            .help("Display program version"))
        .get_matches()
}

async fn run() -> Result<(), Errors> {



    let matches = parse_arguments();

    if matches.is_present("version") {
        println!("{} {}", PROGRAM_NAME, VERSION);
        return Ok(());
    }


    let mut terminal = ratatui::init();



    let mut context = Arc::new(RwLock::new(Context::new()));




    let app_result = App::new(context).run(&mut terminal);






    let browser = Browser::new(
       LaunchOptions {
           headless: true,
           ..Default::default()
       }
    ).expect("Could not launch browser");

    let tab = browser.new_tab().expect("Could not create new tab");

    //tab.navigate_to("https://www.rust-lang.org/").expect("Could not navigate");
    //tab.wait_until_navigated().expect("Could not wait until navigate");









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
