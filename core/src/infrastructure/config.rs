use anyhow::Result;
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use super::auth::TokenInfo;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub target_folder_ids: Vec<String>,
    pub google_client_id: String,
    pub google_client_secret: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            target_folder_ids: vec![],
            google_client_id: "your_client_id_here".to_string(),
            google_client_secret: "your_client_secret_here".to_string(),
        }
    }
}

pub struct ConfigManager {
    config_dir: PathBuf,
}

impl ConfigManager {
    pub fn new() -> Result<Self> {
        let config_dir = config_dir()
            .ok_or_else(|| anyhow::anyhow!("設定ディレクトリが見つかりません"))?
            .join("fuzzy-drive-search");

        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
            println!("設定ディレクトリを作成しました: {:?}", config_dir);
        }

        Ok(Self { config_dir })
    }

    pub fn load_config(&self) -> Result<AppConfig> {
        let config_path = self.config_dir.join("config.toml");

        if !config_path.exists() {
            println!("設定ファイルが存在しません。初期設定を作成します。");
            let default_config = AppConfig::default();
            self.save_config(&default_config)?;
            return Ok(default_config);
        }

        let content = fs::read_to_string(&config_path)?;
        let config: AppConfig = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save_config(&self, config: &AppConfig) -> Result<()> {
        let config_path = self.config_dir.join("config.toml");
        let content = toml::to_string_pretty(config)?;
        fs::write(&config_path, content)?;
        println!("設定ファイルを保存しました: {:?}", config_path);
        Ok(())
    }

    pub fn load_tokens(&self) -> Result<Option<TokenInfo>> {
        let tokens_path = self.config_dir.join("tokens.json");

        if !tokens_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&tokens_path)?;
        let tokens: TokenInfo = serde_json::from_str(&content)?;
        Ok(Some(tokens))
    }

    pub fn save_tokens(&self, tokens: &TokenInfo) -> Result<()> {
        let tokens_path = self.config_dir.join("tokens.json");
        let content = serde_json::to_string_pretty(tokens)?;
        fs::write(&tokens_path, content)?;
        println!("認証トークンを保存しました");
        Ok(())
    }

    pub fn get_database_path(&self) -> PathBuf {
        self.config_dir.join("search_index.db")
    }

    pub fn setup_initial_config(&self) -> Result<AppConfig> {
        println!("初期設定を開始します。");
        
        let config = self.load_config()?;

        // Google API認証情報の設定確認
        if config.google_client_id == "your_client_id_here" {
            println!("\nGoogle Drive API の設定が必要です。");
            println!("Google Cloud Console でプロジェクトを作成し、Drive API を有効にしてください。");
            println!("OAuth 2.0 クライアントIDとシークレットを取得してください。");
            println!("\n設定ファイルを編集してください: {:?}", self.config_dir.join("config.toml"));
            println!("client_id と client_secret を正しい値に変更した後、再度実行してください。");
            return Err(anyhow::anyhow!("Google API認証情報の設定が必要です"));
        }

        // 検索対象フォルダIDの設定確認
        if config.target_folder_ids.is_empty() {
            println!("\n検索対象のGoogle Driveフォルダを設定してください。");
            println!("フォルダのURLから ID を取得してください。");
            println!("例: https://drive.google.com/drive/folders/1ABCDefGHijKLmnOPqrStUVwxyz");
            println!("この場合、フォルダID は「1ABCDefGHijKLmnOPqrStUVwxyz」です。");
            println!("\n設定ファイルを編集してください: {:?}", self.config_dir.join("config.toml"));
            println!("target_folder_ids を配列で設定した後、再度実行してください。");
            println!("例: target_folder_ids = [\"1ABCDefGHijKLmnOPqrStUVwxyz\", \"1XYZabcdefghijklmnopqrst\"]");
            return Err(anyhow::anyhow!("検索対象フォルダIDの設定が必要です"));
        }

        Ok(config)
    }
}