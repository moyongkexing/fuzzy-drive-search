use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DriveFile {
    pub id: String,
    pub name: String,
    pub web_view_link: String,
    pub modified_time: DateTime<Utc>,
    pub mime_type: String,
    pub parents: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveIndex {
    pub files: Vec<DriveFile>,
    pub last_sync: DateTime<Utc>,
    pub sync_token: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SearchResult {
    pub file: DriveFile,
    pub score: f64,
    pub matched_ranges: Vec<(usize, usize)>,
}

impl DriveFile {
    pub fn new(
        id: String,
        name: String,
        web_view_link: String,
        modified_time: DateTime<Utc>,
        mime_type: String,
        parents: Vec<String>,
    ) -> Self {
        Self {
            id,
            name,
            web_view_link,
            modified_time,
            mime_type,
            parents,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn ドライブファイルを正常に作成できる() {
        let file = DriveFile::new(
            "test_id_123".to_string(),
            "ログイン画面_設計書.docx".to_string(),
            "https://drive.google.com/file/d/test_id_123/view".to_string(),
            Utc::now(),
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string(),
            vec!["folder_123".to_string()],
        );

        assert_eq!(file.id, "test_id_123");
        assert_eq!(file.name, "ログイン画面_設計書.docx");
        assert!(file.web_view_link.contains("test_id_123"));
    }

    #[test]
    fn ドライブファイルが等しいかどうか判定できる() {
        let now = Utc::now();
        let file1 = DriveFile::new(
            "same_id".to_string(),
            "同じファイル.pdf".to_string(),
            "https://drive.google.com/file/d/same_id/view".to_string(),
            now,
            "application/pdf".to_string(),
            vec![],
        );
        
        let file2 = DriveFile::new(
            "same_id".to_string(),
            "同じファイル.pdf".to_string(),
            "https://drive.google.com/file/d/same_id/view".to_string(),
            now,
            "application/pdf".to_string(),
            vec![],
        );

        assert_eq!(file1, file2);
    }

    #[test]
    fn ドライブインデックスを作成できる() {
        let files = vec![
            DriveFile::new(
                "file1".to_string(),
                "テスト資料.docx".to_string(),
                "https://drive.google.com/file/d/file1/view".to_string(),
                Utc::now(),
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string(),
                vec![],
            ),
            DriveFile::new(
                "file2".to_string(),
                "設計書.pdf".to_string(),
                "https://drive.google.com/file/d/file2/view".to_string(),
                Utc::now(),
                "application/pdf".to_string(),
                vec![],
            ),
        ];

        let index = DriveIndex {
            files,
            last_sync: Utc::now(),
            sync_token: Some("token_123".to_string()),
        };

        assert_eq!(index.files.len(), 2);
        assert_eq!(index.sync_token, Some("token_123".to_string()));
        assert_eq!(index.files[0].name, "テスト資料.docx");
        assert_eq!(index.files[1].name, "設計書.pdf");
    }

    #[test]
    fn 検索結果を作成できる() {
        let file = DriveFile::new(
            "search_test".to_string(),
            "ログイン機能仕様書.pdf".to_string(),
            "https://drive.google.com/file/d/search_test/view".to_string(),
            Utc::now(),
            "application/pdf".to_string(),
            vec![],
        );

        let result = SearchResult {
            file: file.clone(),
            score: 0.85,
            matched_ranges: vec![(0, 3), (4, 6)], // "ログイン" と "機能" の位置
        };

        assert_eq!(result.file, file);
        assert_eq!(result.score, 0.85);
        assert_eq!(result.matched_ranges, vec![(0, 3), (4, 6)]);
    }
}