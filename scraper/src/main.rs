mod category;
mod client;
mod df;
mod fetcher;
mod output_writer;
mod pirates;
mod ships;
mod types;
mod utils;

use std::sync::Arc;

use category::CategoryScraper;
use clap::Parser;
use client::HttpClientWrapper;
use df::scraper::{DfScrapable, DfScraper};
use fetcher::HtmlFetcher;
use log::{debug, info};
use output_writer::OutputWriter;
use pirates::scraper::PirateScraper;

/// OPDB Scrapper program
#[derive(Parser)]
#[command(version, about)]
struct MainArgs {
    /// Set the scrapped output directory path
    #[arg(short, long, default_value = "data")]
    output_dir: String,
    category: Option<String>,
}

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    debug!("Starting...");
    let args = MainArgs::parse();

    let base_url = "https://onepiece.fandom.com";
    let output_dir = args.output_dir;
    let category = args.category.as_ref();

    let client = HttpClientWrapper(reqwest::Client::builder().build().unwrap());
    let fetcher = HtmlFetcher::new(client, base_url);
    let cat_crawler = Arc::new(CategoryScraper::new(fetcher.clone()));
    let writer = OutputWriter::new(output_dir);

    if category.is_none() || category.is_some_and(|c| c == "df") {
        let df_s = DfScraper::new(fetcher.clone());
        let df_type_infos = df_s.get_dftype_info().await.unwrap();
        let df_result = df_s.get_df_list().await.unwrap();
        writer.write(&df_type_infos, "df_type_infos").await.unwrap();
        writer.write(&df_result, "df_list").await.unwrap();
    }

    if category.is_none() || category.is_some_and(|c| c == "pirate") {
        let pirate_s = PirateScraper::new(fetcher.clone(), cat_crawler.clone());
        let pirates = pirate_s.scrape().await.unwrap();
        writer.write(&pirates, "pirates").await.unwrap();
    }

    if category.is_none() || category.is_some_and(|c| c == "ship") {
        let ship_s = ships::scraper::ShipScraper::new(fetcher.clone(), cat_crawler.clone());
        let ships = ship_s.scrape().await.unwrap();
        writer.write(&ships, "ships").await.unwrap();
    }
}
