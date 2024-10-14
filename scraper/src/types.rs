use core::error;

use thiserror::Error;

pub trait UrlTyped {
    fn get_path(&self) -> &'static str;
}

#[derive(Debug, Clone, Error)]
pub enum Error {
    #[error("Request error: {0}")]
    RequestError(String),
    #[error("Invalid html structure: {0}")]
    InvalidStructure(String),
}
