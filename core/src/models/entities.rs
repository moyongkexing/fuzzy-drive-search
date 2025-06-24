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

