use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DriveApiFile {
    pub id: String,
    pub name: String,
    #[serde(rename = "webViewLink")]
    pub web_view_link: Option<String>,
    #[serde(rename = "modifiedTime")]
    pub modified_time: String,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    pub parents: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct DriveFilesResponse {
    pub files: Vec<DriveApiFile>,
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
}

pub struct GoogleDriveClient {
    client: Client,
    access_token: String,
}

impl GoogleDriveClient {
    pub fn new(access_token: String) -> Self {
        Self {
            client: Client::new(),
            access_token,
        }
    }

    pub async fn list_files(&self, page_token: Option<String>) -> Result<DriveFilesResponse> {
        let mut params = vec![
            ("fields", "files(id,name,webViewLink,modifiedTime,mimeType,parents),nextPageToken"),
            ("pageSize", "100"),
        ];
        
        if let Some(ref token) = page_token {
            params.push(("pageToken", token));
        }

        let response = self
            .client
            .get("https://www.googleapis.com/drive/v3/files")
            .header("Authorization", format!("Bearer {}", self.access_token))
            .query(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Drive API エラー: {}", error_text);
        }

        let files_response: DriveFilesResponse = response.json().await?;
        Ok(files_response)
    }

    pub async fn test_connection(&self) -> Result<bool> {
        let response = self
            .client
            .get("https://www.googleapis.com/drive/v3/about")
            .header("Authorization", format!("Bearer {}", self.access_token))
            .query(&[("fields", "user")])
            .send()
            .await?;

        Ok(response.status().is_success())
    }
}

