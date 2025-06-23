use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
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

    pub async fn list_files_in_folder(&self, _folder_id: &str, page_token: Option<String>) -> Result<DriveFilesResponse> {
        // 全ファイルを取得（フォルダも含む）してから階層的にフィルタリング
        let query = "trashed=false".to_string();
        
        println!("検索クエリ: {} (全ファイルを取得後、フォルダ階層でフィルタリング)", query);
        
        let mut params = vec![
            ("fields", "files(id,name,webViewLink,modifiedTime,mimeType,parents),nextPageToken"),
            ("pageSize", "1000"),
            ("q", query.as_str()),
            ("supportsAllDrives", "true"),
            ("includeItemsFromAllDrives", "true"),
            ("corpora", "allDrives"),
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

        let response_text = response.text().await?;
        println!("APIレスポンス（最初の500文字）: {}", &response_text[..response_text.len().min(500)]);
        
        let files_response: DriveFilesResponse = serde_json::from_str(&response_text)?;
        println!("取得したファイル数: {}", files_response.files.len());
        
        Ok(files_response)
    }

    pub async fn list_files_in_folders_directly(&self, folder_ids: &[String]) -> Result<Vec<DriveApiFile>> {
        let mut all_files = Vec::new();

        println!("{}個のフォルダの直下ファイルを取得中...", folder_ids.len());

        for (index, folder_id) in folder_ids.iter().enumerate() {
            println!("フォルダ {}/{}: {} の直下ファイルを取得中...", 
                index + 1, folder_ids.len(), folder_id);

            let mut page_token = None;
            loop {
                let response = self.list_folder_contents(folder_id, page_token).await?;
                
                for file in response.files {
                    // ファイルのみを追加（フォルダは除外）
                    if file.mime_type != "application/vnd.google-apps.folder" {
                        all_files.push(file);
                    }
                }

                if response.next_page_token.is_none() {
                    break;
                }
                page_token = response.next_page_token;
            }
        }

        println!("直下ファイル取得完了: {}件のファイルを発見", all_files.len());
        Ok(all_files)
    }

    async fn list_folder_contents(&self, folder_id: &str, page_token: Option<String>) -> Result<DriveFilesResponse> {
        let query = format!("'{}' in parents and trashed=false", folder_id);
        
        let mut params = vec![
            ("fields", "files(id,name,webViewLink,modifiedTime,mimeType,parents),nextPageToken"),
            ("pageSize", "1000"),
            ("q", query.as_str()),
            ("supportsAllDrives", "true"),
            ("includeItemsFromAllDrives", "true"),
            ("corpora", "allDrives"),
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

        let response_text = response.text().await?;
        let files_response: DriveFilesResponse = serde_json::from_str(&response_text)?;
        
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
    
    pub async fn get_folder_info(&self, folder_id: &str) -> Result<String> {
        let response = self
            .client
            .get(&format!("https://www.googleapis.com/drive/v3/files/{}", folder_id))
            .header("Authorization", format!("Bearer {}", self.access_token))
            .query(&[
                ("fields", "id,name,mimeType,webViewLink"),
                ("supportsAllDrives", "true"),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            println!("フォルダ情報取得エラー: {}", error_text);
            anyhow::bail!("フォルダ情報の取得に失敗しました");
        }

        let folder_info = response.text().await?;
        println!("フォルダ情報: {}", folder_info);
        Ok(folder_info)
    }
}

