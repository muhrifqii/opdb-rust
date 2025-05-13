use serde::Serialize;
use serde_json::to_vec_pretty;
use strum::{Display, EnumIter, EnumString, IntoEnumIterator};
#[cfg(any(not(test), rust_analyzer))]
use tokio::fs::{create_dir_all, File};
use tokio::io::AsyncWriteExt;

#[cfg(all(test, not(rust_analyzer)))]
use mocks::{create_dir_all, File};

#[derive(Debug, EnumString, EnumIter, Display)]
#[strum(serialize_all = "snake_case")]
pub enum OutputFormat {
    Json,
}

#[derive(Debug)]
pub struct OutputWriter {
    format: Vec<OutputFormat>,
    dir: String,
}

impl OutputWriter {
    pub fn new(dir: String) -> Self {
        Self {
            format: OutputFormat::iter().collect(),
            dir,
        }
    }

    pub async fn write<T>(&self, data: &T, file_name: &str) -> std::io::Result<()>
    where
        T: ?Sized + Serialize,
    {
        create_dir_all(&self.dir).await?;
        for fmt in self.format.iter() {
            let mut file = File::create(format!("{}/{}.{}", &self.dir, file_name, fmt)).await?;
            file.write_all(&serialize(data, fmt)).await?;
            file.flush().await?;
        }

        Ok(())
    }
}

fn serialize<T>(data: &T, format: &OutputFormat) -> Vec<u8>
where
    T: ?Sized + Serialize,
{
    match format {
        OutputFormat::Json => to_vec_pretty(data).expect("Failed to serialize JSON"),
    }
}

#[cfg(test)]
#[allow(dead_code)]
mod mocks {
    use std::cell::RefCell;

    pub async fn create_dir_all(_path: &str) -> std::io::Result<()> {
        Ok(())
    }

    #[derive(Clone, Default)]
    pub struct File {
        pub expexted: String,
    }

    thread_local! {
        pub static FILE: RefCell<File> = RefCell::new(File::default());
    }

    impl File {
        pub async fn create(path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
            let self_f = FILE.with(|f| {
                let bfile = f.borrow().to_owned();
                let path = path.as_ref().to_owned();
                assert_eq!(path.to_str().unwrap(), bfile.expexted);
                bfile
            });
            Ok(self_f)
        }

        pub async fn write_all(&mut self, src: &[u8]) -> std::io::Result<()> {
            assert!(!src.is_empty());
            Ok(())
        }

        pub async fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        df::models::DfTypeInfo,
        output_writer::mocks::{self, File},
    };

    use super::OutputWriter;

    #[tokio::test]
    async fn write_to_json() {
        let devils = vec![DfTypeInfo::default()];
        let writer = OutputWriter::new("folder".to_string());
        mocks::FILE.with_borrow_mut(|f| {
            *f = File {
                expexted: "folder/output.json".to_string(),
            }
        });
        writer.write(&devils, "output").await.unwrap();
        mocks::FILE.with_borrow_mut(|f| *f = File::default());
    }
}
