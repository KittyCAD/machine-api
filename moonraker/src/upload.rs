use anyhow::Result;
use bytes::Bytes;
use reqwest::multipart;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use super::PrintManager;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UploadResponseItem {
    pub path: String,
    pub root: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UploadResponse {
    pub item: UploadResponseItem,
    pub print_started: bool,
    pub print_queued: bool,
    pub action: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DeleteResponseItem {
    pub path: String,
    pub root: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DeleteResponseInner {
    pub item: DeleteResponseItem,
    pub action: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DeleteResponse {
    result: DeleteResponseInner,
}

impl PrintManager {
    /// Upload a file with some gcode to the server.
    pub async fn upload_file(&self, file_name: &Path) -> Result<UploadResponse> {
        self.upload(
            &PathBuf::from(file_name.file_name().unwrap().to_str().unwrap()),
            &std::fs::read(file_name)?,
        )
        .await
    }

    /// Upload a byte array of gcode to the print queue.
    pub async fn upload(&self, file_name: &Path, gcode: &[u8]) -> Result<UploadResponse> {
        let file_name = file_name.to_str().unwrap();
        let gcode = multipart::Part::bytes(gcode.to_owned())
            .file_name(file_name.to_owned())
            .mime_str("text/x-gcode")?;

        let client = reqwest::Client::new();

        // TODO: include checksum

        Ok(client
            .post(format!("{}/server/files/upload", self.url_base))
            .multipart(multipart::Form::new().text("root", "gcodes").part("file", gcode))
            .send()
            .await?
            .json()
            .await?)
    }

    /// Get the contents of an uploaded file.
    pub async fn get(&self, file_name: &Path) -> Result<Bytes> {
        let file_name = file_name.to_str().unwrap();
        let client = reqwest::Client::new();
        Ok(client
            .get(format!("{}/server/files/gcodes/{}", self.url_base, file_name))
            .send()
            .await?
            .bytes()
            .await?)
    }

    /// Delete an uploaded file from the print queue.
    pub async fn delete(&self, file_name: &Path) -> Result<DeleteResponse> {
        let file_name = file_name.to_str().unwrap();
        let client = reqwest::Client::new();
        Ok(client
            .delete(format!("{}/server/files/gcodes/{}", self.url_base, file_name))
            .send()
            .await?
            .json()
            .await?)
    }
}
