use anyhow::Result;
use bytes::Bytes;
use reqwest::multipart;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use super::Client;

/// File that has been uploaded to Moonraker.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UploadResponseItem {
    /// Path of the file relative to the root directory.
    pub path: String,

    /// Root folder. Currently only a limted set are supported,
    /// check the moonraker docs for more information. This code
    /// assumes everything is `gcodes` for now.
    pub root: String,
}

/// Response to an upload request.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UploadResponse {
    /// `gcode` file uploaded to the printer.
    pub item: UploadResponseItem,

    /// Has this print been started?
    pub print_started: bool,

    /// Has this print been enqueued?
    pub print_queued: bool,
}

/// File that has been deleted from Moonraker.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DeleteResponseItem {
    /// Path of the file relative to the root directory.
    pub path: String,

    /// Root folder. Currently only a limted set are supported,
    /// check the moonraker docs for more information. This code
    /// assumes everything is `gcodes` for now.
    pub root: String,
}

/// Response to a delete request.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DeleteResponse {
    /// `gcode` file that has been deleted.
    pub item: DeleteResponseItem,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct DeleteResponseWrapper {
    result: DeleteResponse,
}

impl Client {
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
        let resp: DeleteResponseWrapper = client
            .delete(format!("{}/server/files/gcodes/{}", self.url_base, file_name))
            .send()
            .await?
            .json()
            .await?;
        Ok(resp.result)
    }
}
