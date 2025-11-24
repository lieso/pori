use std::sync::{Arc, RwLock};
use ratatui;
use headless_chrome::{Browser, LaunchOptions};
use clap::{Arg, App as ClapApp};
use parversion::provider::{Provider};
use parversion::provider::yaml::{YamlFileProvider};
use std::fs;
use std::path::PathBuf;

mod app;
mod context;
mod macros;
mod prelude;
mod schema;
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

async fn init_provider() -> Result<Arc<YamlFileProvider>, Errors> {
    log::info!("Initializing data provider...");

    log::info!("Using yaml file provider");

    let data_dir: PathBuf = dirs::data_dir()
     .ok_or_else(|| Errors::ProviderError("Could not find data
directory".into()))?;

    let provider_path = data_dir.join(PROGRAM_NAME).join("provider.yaml");
    
    if let Some(parent_dir) = provider_path.parent() {
        fs::create_dir_all(parent_dir).expect("Unable to create directory");
    }

    log::debug!("provider_path: {}", provider_path.display());

    Ok(Arc::new(YamlFileProvider::new(provider_path.to_string_lossy().into_owned())))
}

async fn init_browser() -> Result<Browser, Errors> {
    log::info!("Initializing web browser...");

    Browser::new(
       LaunchOptions {
           headless: true,
           ..Default::default()
       }
    ).map_err(|e| Errors::BrowserError(format!("Could not start web browser: {}",
 e)))
}

async fn run() -> Result<(), Errors> {
    let matches = parse_arguments();

    if matches.is_present("version") {
        println!("{} {}", PROGRAM_NAME, VERSION);
        return Ok(());
    }

    let provider = init_provider().await?;

    let browser = init_browser().await?;

    let mut context = Arc::new(RwLock::new(Context::new(
        provider,
        browser
    )));

    let mut terminal = ratatui::init();
    let app_result = App::new(context).run(&mut terminal);

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
