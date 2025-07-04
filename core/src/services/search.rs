use anyhow::Result;
use chrono::{Duration, Utc};

use crate::infra::{
    ConfigManager, JsonStorage, GoogleDriveClient, OAuth2Client,
};

pub struct SearchService {
    config_manager: ConfigManager,
    json_storage: JsonStorage,
}

impl SearchService {
    pub fn new() -> Result<Self> {
        let config_manager = ConfigManager::new()?;
        
        // JSONストレージのパスを設定
        let storage_path = config_manager.config_dir.join("drive_files.json");
        let json_storage = JsonStorage::new(storage_path)?;
        
        Ok(Self {
            config_manager,
            json_storage,
        })
    }

    pub async fn ensure_initialized(&mut self) -> Result<()> {
        self.initialize_with_overrides(None, None).await
    }

    pub async fn initialize_with_overrides(
        &mut self,
        client_id_override: Option<String>,
        client_secret_override: Option<String>
    ) -> Result<()> {
        // 設定ファイルの初期化確認
        let config = self.config_manager.setup_initial_config_with_overrides(
            client_id_override,
            client_secret_override
        )?;
        
        // 認証トークンの確認・取得
        self.ensure_authenticated(&config).await?;
        
        // JSONストレージの初期化確認
        let file_count = self.json_storage.get_file_count()?;
        if file_count == 0 {
            println!("初回同期を実行します...");
            self.sync_files().await?;
        } else {
            println!("ストレージに{}件のファイルがあります", file_count);
        }

        Ok(())
    }

    async fn ensure_authenticated(&self, config: &crate::infra::AppConfig) -> Result<()> {
        let oauth_client = OAuth2Client::new(
            config.google_client_id.clone(),
            config.google_client_secret.clone(),
        );

        // 既存のトークンを確認
        if let Some(tokens) = self.config_manager.load_tokens()? {
            // トークンの有効性を確認（簡易版）
            let drive_client = GoogleDriveClient::new(tokens.access_token.clone());
            
            if drive_client.test_connection().await.unwrap_or(false) {
                println!("既存の認証トークンが有効です");
                return Ok(());
            }

            // リフレッシュトークンで更新を試行
            if let Some(ref refresh_token) = tokens.refresh_token {
                if let Ok(new_tokens) = oauth_client.refresh_token(refresh_token).await {
                    println!("認証トークンを更新しました");
                    self.config_manager.save_tokens(&new_tokens)?;
                    return Ok(());
                }
            }
        }

        // 新規認証
        println!("認証が必要です。OAuth2フローを開始します...");
        let tokens = oauth_client.authorize().await?;
        self.config_manager.save_tokens(&tokens)?;
        println!("認証が完了しました");

        Ok(())
    }

    pub async fn sync_files(&mut self) -> Result<()> {
        let config = self.config_manager.load_config()?;
        if config.target_folder_ids.is_empty() {
            return Err(anyhow::anyhow!("検索対象フォルダIDが設定されていません"));
        }

        let tokens = self.config_manager.load_tokens()?
            .ok_or_else(|| anyhow::anyhow!("認証トークンが見つかりません"))?;

        let drive_client = GoogleDriveClient::new(tokens.access_token);

        println!("Google Driveから{}個のフォルダの直下ファイルを取得中...", config.target_folder_ids.len());
        
        // 各フォルダ情報を確認
        for folder_id in &config.target_folder_ids {
            println!("\nフォルダID {} の情報を確認中...", folder_id);
            if let Err(e) = drive_client.get_folder_info(folder_id).await {
                println!("フォルダ情報取得エラー: {}", e);
            }
        }

        // 複数フォルダの直下ファイルのみを取得
        let api_files = drive_client.list_files_in_folders_directly(&config.target_folder_ids).await?;
        
        let mut all_files = Vec::new();
        for api_file in api_files {
            let file = crate::models::DriveFile::new(
                api_file.id,
                api_file.name,
                api_file.web_view_link.unwrap_or_default(),
                chrono::DateTime::parse_from_rfc3339(&api_file.modified_time)?
                    .with_timezone(&Utc),
                api_file.mime_type,
                api_file.parents.unwrap_or_default(),
            );
            all_files.push(file);
        }

        // フォルダ名を取得
        let folder_names = self.fetch_folder_names_for_sync(&drive_client, &config.target_folder_ids).await?;
        
        // JSONストレージに保存
        self.json_storage.save_data(&all_files, &folder_names, None)?;

        println!("同期が完了しました。{}件のファイルを取得しました", all_files.len());
        Ok(())
    }


    pub fn get_folder_names(&self) -> Result<std::collections::HashMap<String, String>> {
        self.json_storage.get_folder_names()
    }

    async fn fetch_folder_names_for_sync(&self, drive_client: &GoogleDriveClient, folder_ids: &[String]) -> Result<std::collections::HashMap<String, String>> {
        let mut folder_names = std::collections::HashMap::new();
        
        for folder_id in folder_ids {
            if let Ok(folder_info) = drive_client.get_folder_info(folder_id).await {
                if let Ok(folder_data) = serde_json::from_str::<serde_json::Value>(&folder_info) {
                    if let Some(name) = folder_data["name"].as_str() {
                        folder_names.insert(folder_id.clone(), name.to_string());
                    }
                }
            }
        }
        
        Ok(folder_names)
    }

    pub async fn check_and_sync(&mut self) -> Result<()> {
        // 最後の同期から1時間以上経過している場合のみ同期
        if let Some((last_sync, _)) = self.json_storage.get_sync_info()? {
            let now = Utc::now();
            let sync_interval = Duration::hours(1);
            
            if now.signed_duration_since(last_sync) < sync_interval {
                return Ok(());
            }
        }

        println!("定期同期を実行します...");
        self.sync_files().await
    }
}