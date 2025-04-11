mod category;
mod df;
mod fetcher;
mod output_writer;
mod pirates;
mod types;
mod utils;

use category::CategoryScraper;
use clap::Parser;
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

    let fetcher = HtmlFetcher::new(reqwest::Client::builder().build().unwrap());
    let cat_crawler = Box::new(CategoryScraper::new(fetcher.clone(), base_url));
    let df_s = DfScraper::new(fetcher.clone(), base_url);
    let pirate_s = PirateScraper::new(fetcher.clone(), cat_crawler, base_url);

    let (pirates, ships) = pirate_s.get().await.unwrap();
    info!("pirates: {:?}\nships: {:?}", pirates, ships);

    // let df_type_infos = df_s.get_dftype_info().await.unwrap();
    // let df_result = df_s.get_df_list().await.unwrap();
    // info!("result size: {}", df_result.len());

    // let writer = JsonWriter;
    // writer
    //     .write(&df_type_infos, &output_dir, "df_type_infos")
    //     .await
    //     .unwrap();
    // writer
    //     .write(&df_result, &output_dir, "df_list")
    //     .await
    //     .unwrap();
}
