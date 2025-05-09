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
use output_writer::{JsonWriter, OutputWriter};
use pirates::scraper::PirateScraper;

/// OPDB Scrapper program
#[derive(Parser, Debug)]
#[command(version, about)]
struct MainArgs {
    /// Set the scrapped output directory path
    #[arg(short, long)]
    output: Option<String>,
}

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    debug!("Starting...");
    let args = MainArgs::parse();

    let base_url = "https://onepiece.fandom.com";
    let default_output_dir = String::from("data");
    let output_dir = args.output.unwrap_or(default_output_dir);

    let client = HttpClientWrapper(reqwest::Client::builder().build().unwrap());
    let fetcher = HtmlFetcher::new(client, base_url);
    let cat_crawler = Arc::new(CategoryScraper::new(fetcher.clone()));

    let df_s = DfScraper::new(fetcher.clone());
    let pirate_s = PirateScraper::new(fetcher.clone(), cat_crawler.clone());
    let ship_s = ships::scraper::ShipScraper::new(fetcher.clone(), cat_crawler.clone());

    // let df_type_infos = df_s.get_dftype_info().await.unwrap();
    // let df_result = df_s.get_df_list().await.unwrap();
    // let pirates = pirate_s.scrape().await.unwrap();
    let ships = ship_s.scrape().await.unwrap();
    info!("{:?}", ships);

    // let writer = JsonWriter;
    // writer
    //     .write(&df_type_infos, &output_dir, "df_type_infos")
    //     .await
    //     .unwrap();
    // writer
    //     .write(&df_result, &output_dir, "df_list")
    //     .await
    //     .unwrap();
    // writer
    //     .write(&pirates, &output_dir, "pirates")
    //     .await
    //     .unwrap();
    // writer.write(&ships, &output_dir, "ships").await.unwrap();
}
