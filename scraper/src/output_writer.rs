#[cfg(all(test, not(rust_analyzer)))]
use mocks::{create_dir_all, File};
use serde::Serialize;
use serde_json::to_string_pretty;
#[cfg(any(not(test), rust_analyzer))]
use tokio::fs::{create_dir_all, File};
use tokio::io::AsyncWriteExt;

pub trait OutputWriter {
    async fn write<T>(&self, data: &T, path: &str, file_name: &str) -> std::io::Result<()>
    where
        T: Serialize;
}

#[derive(Default, Debug)]
pub struct JsonWriter;

impl OutputWriter for JsonWriter {
    async fn write<T>(&self, data: &T, path: &str, file_name: &str) -> std::io::Result<()>
    where
        T: Serialize,
    {
        create_dir_all(path).await?;
        let mut file = File::create(path.to_string() + "/" + file_name + ".json").await?;
        let json = to_string_pretty(data).expect("Failed to serialize JSON");
        file.write_all(json.as_bytes()).await?;
        file.flush().await?;

        Ok(())
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

    use super::{JsonWriter, OutputWriter};

    #[tokio::test]
    async fn write_to_json() {
        let devils = vec![DfTypeInfo::default()];
        let writer = JsonWriter;
        mocks::FILE.with_borrow_mut(|f| {
            *f = File {
                expexted: "folder/output.json".to_string(),
            }
        });
        writer.write(&devils, "folder", "output").await.unwrap();
        mocks::FILE.with_borrow_mut(|f| *f = File::default());
    }
}
