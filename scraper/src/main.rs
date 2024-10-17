mod df;
mod output_writer;
mod types;

use clap::{Parser, Subcommand};
use df::df_scraper::{DfScrapable, DfScraper};
use log::debug;

/// OPDB Scrapper program
#[derive(Parser, Debug)]
#[command(version, about)]
struct MainArgs {
    /// Set the scrapped output directory path
    #[arg(short, long)]
    output: Option<String>,
}

#[derive(Subcommand)]
enum Commands {}

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    debug!("Starting...");
    let args = MainArgs::parse();

    let base_url = "https://onepiece.fandom.com";
    if let Some(output_dir) = args.output.as_deref() {}

    let df_s = DfScraper::new(base_url, reqwest::Client::new());
    df_s.get_dftype_info().await.unwrap();
}
