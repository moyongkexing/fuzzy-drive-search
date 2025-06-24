use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::models::DriveFile;

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonStorageFile {
    pub id: String,
    pub name: String,
    pub web_view_link: String,
    pub modified_time: DateTime<Utc>,
    pub mime_type: String,
    pub parents: Vec<String>,
    pub parent_folder_name: String,
    pub keywords: Vec<String>,
    pub romaji_keywords: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonStorageData {
    pub files: Vec<JsonStorageFile>,
    pub folders: HashMap<String, String>,
    pub last_sync: DateTime<Utc>,
    pub sync_token: Option<String>,
}

pub struct JsonStorage {
    storage_path: PathBuf,
}

impl JsonStorage {
    pub fn new(storage_path: PathBuf) -> Result<Self> {
        if let Some(parent) = storage_path.parent() {
            fs::create_dir_all(parent)?;
        }

        Ok(Self {
            storage_path,
        })
    }

    pub fn save_data(
        &mut self,
        files: &[DriveFile],
        folder_names: &HashMap<String, String>,
        sync_token: Option<String>,
    ) -> Result<()> {
        let mut storage_files = Vec::new();

        for file in files {
            let (keywords, romaji_keywords) = self.extract_keywords(&file.name)?;

            let parent_folder_name = file.parents.first()
                .and_then(|parent_id| folder_names.get(parent_id))
                .cloned()
                .unwrap_or_else(|| "不明なフォルダ".to_string());

            storage_files.push(JsonStorageFile {
                id: file.id.clone(),
                name: file.name.clone(),
                web_view_link: file.web_view_link.clone(),
                modified_time: file.modified_time,
                mime_type: file.mime_type.clone(),
                parents: file.parents.clone(),
                parent_folder_name,
                keywords,
                romaji_keywords,
            });
        }

        let storage_data = JsonStorageData {
            files: storage_files,
            folders: folder_names.clone(),
            last_sync: Utc::now(),
            sync_token,
        };

        let json_data = serde_json::to_string_pretty(&storage_data)?;
        fs::write(&self.storage_path, json_data)?;

        println!("JSONストレージに{}件のファイルを保存しました", files.len());
        Ok(())
    }

    pub fn load_data(&self) -> Result<Option<JsonStorageData>> {
        if !self.storage_path.exists() {
            return Ok(None);
        }

        let json_data = fs::read_to_string(&self.storage_path)?;
        let storage_data: JsonStorageData = serde_json::from_str(&json_data)?;
        Ok(Some(storage_data))
    }

    pub fn get_files(&self) -> Result<Vec<DriveFile>> {
        if let Some(data) = self.load_data()? {
            let files: Vec<DriveFile> = data.files.into_iter().map(|f| {
                DriveFile::new(
                    f.id,
                    f.name,
                    f.web_view_link,
                    f.modified_time,
                    f.mime_type,
                    f.parents,
                )
            }).collect();
            Ok(files)
        } else {
            Ok(Vec::new())
        }
    }

    pub fn get_file_count(&self) -> Result<usize> {
        if let Some(data) = self.load_data()? {
            Ok(data.files.len())
        } else {
            Ok(0)
        }
    }

    pub fn get_sync_info(&self) -> Result<Option<(DateTime<Utc>, Option<String>)>> {
        if let Some(data) = self.load_data()? {
            Ok(Some((data.last_sync, data.sync_token)))
        } else {
            Ok(None)
        }
    }

    pub fn get_folder_names(&self) -> Result<HashMap<String, String>> {
        if let Some(data) = self.load_data()? {
            Ok(data.folders)
        } else {
            Ok(HashMap::new())
        }
    }

    fn extract_keywords(&mut self, text: &str) -> Result<(Vec<String>, Vec<String>)> {
        let keywords = vec![text.to_string()];
        let romaji_keywords = self.simple_kana_to_romaji(text);
        Ok((keywords, romaji_keywords))
    }

    fn simple_kana_to_romaji(&self, text: &str) -> Vec<String> {
        let mut romaji_parts = Vec::new();
        let mut current_kana = String::new();
        
        for c in text.chars() {
            match c {
                // ひらがな
                'あ' => current_kana.push_str("a"),
                'い' => current_kana.push_str("i"),
                'う' => current_kana.push_str("u"),
                'え' => current_kana.push_str("e"),
                'お' => current_kana.push_str("o"),
                'か' => current_kana.push_str("ka"),
                'き' => current_kana.push_str("ki"),
                'く' => current_kana.push_str("ku"),
                'け' => current_kana.push_str("ke"),
                'こ' => current_kana.push_str("ko"),
                'が' => current_kana.push_str("ga"),
                'ぎ' => current_kana.push_str("gi"),
                'ぐ' => current_kana.push_str("gu"),
                'げ' => current_kana.push_str("ge"),
                'ご' => current_kana.push_str("go"),
                'さ' => current_kana.push_str("sa"),
                'し' => current_kana.push_str("shi"),
                'す' => current_kana.push_str("su"),
                'せ' => current_kana.push_str("se"),
                'そ' => current_kana.push_str("so"),
                'ざ' => current_kana.push_str("za"),
                'じ' => current_kana.push_str("ji"),
                'ず' => current_kana.push_str("zu"),
                'ぜ' => current_kana.push_str("ze"),
                'ぞ' => current_kana.push_str("zo"),
                'た' => current_kana.push_str("ta"),
                'ち' => current_kana.push_str("chi"),
                'つ' => current_kana.push_str("tsu"),
                'て' => current_kana.push_str("te"),
                'と' => current_kana.push_str("to"),
                'だ' => current_kana.push_str("da"),
                'ぢ' => current_kana.push_str("di"),
                'づ' => current_kana.push_str("du"),
                'で' => current_kana.push_str("de"),
                'ど' => current_kana.push_str("do"),
                'な' => current_kana.push_str("na"),
                'に' => current_kana.push_str("ni"),
                'ぬ' => current_kana.push_str("nu"),
                'ね' => current_kana.push_str("ne"),
                'の' => current_kana.push_str("no"),
                'は' => current_kana.push_str("ha"),
                'ひ' => current_kana.push_str("hi"),
                'ふ' => current_kana.push_str("fu"),
                'へ' => current_kana.push_str("he"),
                'ほ' => current_kana.push_str("ho"),
                'ば' => current_kana.push_str("ba"),
                'び' => current_kana.push_str("bi"),
                'ぶ' => current_kana.push_str("bu"),
                'べ' => current_kana.push_str("be"),
                'ぼ' => current_kana.push_str("bo"),
                'ぱ' => current_kana.push_str("pa"),
                'ぴ' => current_kana.push_str("pi"),
                'ぷ' => current_kana.push_str("pu"),
                'ぺ' => current_kana.push_str("pe"),
                'ぽ' => current_kana.push_str("po"),
                'ま' => current_kana.push_str("ma"),
                'み' => current_kana.push_str("mi"),
                'む' => current_kana.push_str("mu"),
                'め' => current_kana.push_str("me"),
                'も' => current_kana.push_str("mo"),
                'や' => current_kana.push_str("ya"),
                'ゆ' => current_kana.push_str("yu"),
                'よ' => current_kana.push_str("yo"),
                'ら' => current_kana.push_str("ra"),
                'り' => current_kana.push_str("ri"),
                'る' => current_kana.push_str("ru"),
                'れ' => current_kana.push_str("re"),
                'ろ' => current_kana.push_str("ro"),
                'わ' => current_kana.push_str("wa"),
                'ゐ' => current_kana.push_str("wi"),
                'ゑ' => current_kana.push_str("we"),
                'を' => current_kana.push_str("wo"),
                'ん' => current_kana.push_str("n"),
                // カタカナ
                'ア' => current_kana.push_str("a"),
                'イ' => current_kana.push_str("i"),
                'ウ' => current_kana.push_str("u"),
                'エ' => current_kana.push_str("e"),
                'オ' => current_kana.push_str("o"),
                'カ' => current_kana.push_str("ka"),
                'キ' => current_kana.push_str("ki"),
                'ク' => current_kana.push_str("ku"),
                'ケ' => current_kana.push_str("ke"),
                'コ' => current_kana.push_str("ko"),
                'サ' => current_kana.push_str("sa"),
                'シ' => current_kana.push_str("shi"),
                'ス' => current_kana.push_str("su"),
                'セ' => current_kana.push_str("se"),
                'ソ' => current_kana.push_str("so"),
                'タ' => current_kana.push_str("ta"),
                'チ' => current_kana.push_str("chi"),
                'ツ' => current_kana.push_str("tsu"),
                'テ' => current_kana.push_str("te"),
                'ト' => current_kana.push_str("to"),
                'ナ' => current_kana.push_str("na"),
                'ニ' => current_kana.push_str("ni"),
                'ヌ' => current_kana.push_str("nu"),
                'ネ' => current_kana.push_str("ne"),
                'ノ' => current_kana.push_str("no"),
                'ハ' => current_kana.push_str("ha"),
                'ヒ' => current_kana.push_str("hi"),
                'フ' => current_kana.push_str("fu"),
                'ヘ' => current_kana.push_str("he"),
                'ホ' => current_kana.push_str("ho"),
                'マ' => current_kana.push_str("ma"),
                'ミ' => current_kana.push_str("mi"),
                'ム' => current_kana.push_str("mu"),
                'メ' => current_kana.push_str("me"),
                'モ' => current_kana.push_str("mo"),
                'ヤ' => current_kana.push_str("ya"),
                'ユ' => current_kana.push_str("yu"),
                'ヨ' => current_kana.push_str("yo"),
                'ラ' => current_kana.push_str("ra"),
                'リ' => current_kana.push_str("ri"),
                'ル' => current_kana.push_str("ru"),
                'レ' => current_kana.push_str("re"),
                'ロ' => current_kana.push_str("ro"),
                'ワ' => current_kana.push_str("wa"),
                'ヲ' => current_kana.push_str("wo"),
                'ン' => current_kana.push_str("n"),
                _ => {
                    if !current_kana.is_empty() {
                        romaji_parts.push(current_kana.clone());
                        current_kana.clear();
                    }
                }
            }
        }
        
        if !current_kana.is_empty() {
            romaji_parts.push(current_kana);
        }
        
        romaji_parts.sort();
        romaji_parts.dedup();
        romaji_parts
    }

}