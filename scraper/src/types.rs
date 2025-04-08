use async_trait::async_trait;
use thiserror::Error;

use crate::output_writer::OutputWriter;

pub trait UrlTyped {
    fn get_path(&self) -> String;
}

#[derive(Debug, Clone, Error)]
pub enum Error {
    #[error("Request error: {0}")]
    RequestError(String),
    #[error("Invalid html structure: {0}")]
    InvalidStructure(String),
}

#[async_trait(?Send)]
pub trait ScrapeTask {
    async fn run<T: OutputWriter>(&self, writer: T) -> Result<(), Error>;
}
