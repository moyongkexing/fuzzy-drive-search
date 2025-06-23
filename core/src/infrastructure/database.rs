use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Row};
use std::path::Path;

use crate::domain::entities::DriveFile;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let db = Self { conn };
        db.create_tables()?;
        Ok(db)
    }

    fn create_tables(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS drive_files (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                web_view_link TEXT NOT NULL,
                modified_time TEXT NOT NULL,
                mime_type TEXT NOT NULL,
                parents TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS sync_info (
                id INTEGER PRIMARY KEY,
                last_sync TEXT NOT NULL,
                sync_token TEXT
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS folders (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL
            )",
            [],
        )?;

        // 検索用のインデックスを作成
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_file_name ON drive_files(name)",
            [],
        )?;

        Ok(())
    }

    pub fn save_files(&self, files: &[DriveFile]) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;

        // 既存のファイルを削除
        tx.execute("DELETE FROM drive_files", [])?;

        // 新しいファイルを挿入
        for file in files {
            let parents_json = serde_json::to_string(&file.parents)?;
            tx.execute(
                "INSERT INTO drive_files (id, name, web_view_link, modified_time, mime_type, parents, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    file.id,
                    file.name,
                    file.web_view_link,
                    file.modified_time.to_rfc3339(),
                    file.mime_type,
                    parents_json,
                    Utc::now().to_rfc3339()
                ],
            )?;
        }

        tx.commit()?;
        println!("データベースに{}件のファイルを保存しました", files.len());
        Ok(())
    }

    pub fn search_files(&self, query: &str) -> Result<Vec<DriveFile>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, web_view_link, modified_time, mime_type, parents
             FROM drive_files 
             WHERE name LIKE ?1 
             ORDER BY name"
        )?;

        let search_pattern = format!("%{}%", query);
        let rows = stmt.query_map(params![search_pattern], |row| {
            self.row_to_drive_file(row)
        })?;

        let mut files = Vec::new();
        for row in rows {
            files.push(row?);
        }

        Ok(files)
    }

    pub fn get_all_files(&self) -> Result<Vec<DriveFile>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, web_view_link, modified_time, mime_type, parents
             FROM drive_files 
             ORDER BY name"
        )?;

        let rows = stmt.query_map([], |row| {
            self.row_to_drive_file(row)
        })?;

        let mut files = Vec::new();
        for row in rows {
            files.push(row?);
        }

        Ok(files)
    }

    fn row_to_drive_file(&self, row: &Row) -> rusqlite::Result<DriveFile> {
        let parents_json: String = row.get(5)?;
        let parents: Vec<String> = serde_json::from_str(&parents_json)
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                5, rusqlite::types::Type::Text, Box::new(e)
            ))?;

        let modified_time_str: String = row.get(3)?;
        let modified_time = DateTime::parse_from_rfc3339(&modified_time_str)
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                3, rusqlite::types::Type::Text, Box::new(e)
            ))?
            .with_timezone(&Utc);

        Ok(DriveFile {
            id: row.get(0)?,
            name: row.get(1)?,
            web_view_link: row.get(2)?,
            modified_time,
            mime_type: row.get(4)?,
            parents,
        })
    }

    pub fn save_sync_info(&self, sync_token: Option<&str>) -> Result<()> {
        self.conn.execute("DELETE FROM sync_info", [])?;
        
        self.conn.execute(
            "INSERT INTO sync_info (last_sync, sync_token) VALUES (?1, ?2)",
            params![Utc::now().to_rfc3339(), sync_token],
        )?;

        Ok(())
    }

    pub fn get_sync_info(&self) -> Result<Option<(DateTime<Utc>, Option<String>)>> {
        let mut stmt = self.conn.prepare("SELECT last_sync, sync_token FROM sync_info ORDER BY id DESC LIMIT 1")?;
        
        let mut rows = stmt.query_map([], |row| {
            let last_sync_str: String = row.get(0)?;
            let sync_token: Option<String> = row.get(1)?;
            
            let last_sync = DateTime::parse_from_rfc3339(&last_sync_str)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                    0, rusqlite::types::Type::Text, Box::new(e)
                ))?
                .with_timezone(&Utc);
            
            Ok((last_sync, sync_token))
        })?;

        if let Some(row) = rows.next() {
            Ok(Some(row?))
        } else {
            Ok(None)
        }
    }

    pub fn get_file_count(&self) -> Result<usize> {
        let mut stmt = self.conn.prepare("SELECT COUNT(*) FROM drive_files")?;
        let count: i64 = stmt.query_row([], |row| row.get(0))?;
        Ok(count as usize)
    }

    pub fn save_folder_names(&self, folder_names: &std::collections::HashMap<String, String>) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        
        tx.execute("DELETE FROM folders", [])?;
        
        for (folder_id, folder_name) in folder_names {
            tx.execute(
                "INSERT INTO folders (id, name) VALUES (?1, ?2)",
                params![folder_id, folder_name],
            )?;
        }
        
        tx.commit()?;
        Ok(())
    }
    
    pub fn get_folder_names(&self) -> Result<std::collections::HashMap<String, String>> {
        let mut stmt = self.conn.prepare("SELECT id, name FROM folders")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        
        let mut folder_names = std::collections::HashMap::new();
        for row in rows {
            let (id, name) = row?;
            folder_names.insert(id, name);
        }
        
        Ok(folder_names)
    }
}