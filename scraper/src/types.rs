use async_trait::async_trait;
use serde::Serialize;
use thiserror::Error;

use crate::output_writer::{JsonWriter, OutputWriter};

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct NamedUrl {
    pub name: String,
    url: String,
}

impl NamedUrl {
    pub fn new(name: String, url: String) -> Self {
        Self { name, url }
    }
}

impl UrlTyped for NamedUrl {
    fn get_path(&self) -> String {
        self.url.clone()
    }
}

#[derive(Debug, Clone, Default)]
pub struct NamedJpEn {
    pub name: String,
    pub en_name: String,
    pub description: String,
}

impl NamedJpEn {
    pub fn new(name: String, en_name: String, description: String) -> Self {
        Self {
            name,
            en_name,
            description,
        }
    }
}

#[async_trait(?Send)]
pub trait ScrapeTask<T = JsonWriter>
where
    T: OutputWriter,
{
    async fn run(&self, writer: T) -> Result<(), Error>;
}
