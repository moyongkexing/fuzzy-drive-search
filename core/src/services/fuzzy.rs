use crate::models::{DriveFile, SearchResult};

pub struct FuzzySearchEngine {
    threshold: f64,
}

impl FuzzySearchEngine {
    pub fn new(threshold: f64) -> Self {
        Self { threshold }
    }

    pub fn search(&self, query: &str, files: &[DriveFile]) -> Vec<SearchResult> {
        if query.trim().is_empty() {
            return files
                .iter()
                .take(50) // 空クエリの場合は最初の50件を返す
                .map(|file| SearchResult {
                    file: file.clone(),
                    score: 1.0,
                    matched_ranges: vec![],
                })
                .collect();
        }

        // 非インタラクティブ環境では単純検索を使用
        self.simple_search(query, files)
    }

    fn simple_search(&self, query: &str, files: &[DriveFile]) -> Vec<SearchResult> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        for file in files {
            let file_name_lower = file.name.to_lowercase();
            
            if file_name_lower.contains(&query_lower) {
                let score = self.calculate_score(&query_lower, &file_name_lower);
                
                if score >= self.threshold {
                    let matched_ranges = self.find_matched_ranges(query, &file.name);
                    
                    results.push(SearchResult {
                        file: file.clone(),
                        score,
                        matched_ranges,
                    });
                }
            }
        }

        // スコア順でソート
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.truncate(20);

        results
    }

    fn calculate_score(&self, query: &str, file_name: &str) -> f64 {
        let query_lower = query.to_lowercase();
        let name_lower = file_name.to_lowercase();

        // 完全一致
        if name_lower == query_lower {
            return 1.0;
        }

        // 前方一致
        if name_lower.starts_with(&query_lower) {
            return 0.9;
        }

        // 含有一致
        if name_lower.contains(&query_lower) {
            return 0.7;
        }

        // 文字の一致度を計算
        let match_ratio = self.calculate_match_ratio(&query_lower, &name_lower);
        match_ratio * 0.6
    }

    fn calculate_match_ratio(&self, query: &str, text: &str) -> f64 {
        let query_chars: Vec<char> = query.chars().collect();
        let text_chars: Vec<char> = text.chars().collect();
        
        let mut matches = 0;
        let mut text_index = 0;

        for query_char in &query_chars {
            while text_index < text_chars.len() {
                if text_chars[text_index] == *query_char {
                    matches += 1;
                    text_index += 1;
                    break;
                }
                text_index += 1;
            }
        }

        matches as f64 / query_chars.len() as f64
    }

    fn find_matched_ranges(&self, query: &str, text: &str) -> Vec<(usize, usize)> {
        let query_lower = query.to_lowercase();
        let text_lower = text.to_lowercase();
        let mut ranges = Vec::new();

        if let Some(start) = text_lower.find(&query_lower) {
            ranges.push((start, start + query_lower.len()));
        }

        ranges
    }
}