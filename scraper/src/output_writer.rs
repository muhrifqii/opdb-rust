use serde::Serialize;
use serde_json::to_string_pretty;
use tokio::{fs::create_dir_all, fs::File, io::AsyncWriteExt};

pub trait OutputWriter {
    async fn write<T>(&self, data: &T, path: &str, file_name: &str) -> std::io::Result<()>
    where
        T: Serialize;
}

pub struct JsonWriter {}

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
