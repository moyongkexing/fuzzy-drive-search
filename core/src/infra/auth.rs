use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use url::Url;

const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const REDIRECT_URI: &str = "http://localhost:8080/callback";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenInfo {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: u64,
    pub token_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: u64,
    token_type: String,
}

pub struct OAuth2Client {
    client_id: String,
    client_secret: String,
    client: Client,
}

impl OAuth2Client {
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client_id,
            client_secret,
            client: Client::new(),
        }
    }

    pub async fn authorize(&self) -> Result<TokenInfo> {
        println!("Google Drive認証を開始します...");

        // 認証URLを生成
        let auth_url = self.build_auth_url()?;
        println!("ブラウザで以下のURLを開いてください:");
        println!("{}", auth_url);

        // ブラウザを開く
        if let Err(e) = open::that(&auth_url) {
            println!("ブラウザの自動起動に失敗しました: {}", e);
            println!("手動で上記URLをブラウザで開いてください。");
        }

        // ローカルサーバーでコールバックを待機
        let auth_code = self.wait_for_callback()?;
        println!("認証コードを受信しました");

        // トークンを取得
        let token_info = self.exchange_code_for_token(&auth_code).await?;
        println!("アクセストークンを取得しました");

        Ok(token_info)
    }

    pub async fn refresh_token(&self, refresh_token: &str) -> Result<TokenInfo> {
        let params = [
            ("client_id", &self.client_id),
            ("client_secret", &self.client_secret),
            ("refresh_token", &refresh_token.to_string()),
            ("grant_type", &"refresh_token".to_string()),
        ];

        let response = self
            .client
            .post(GOOGLE_TOKEN_URL)
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("トークンの更新に失敗しました: {}", error_text));
        }

        let token_response: TokenResponse = response.json().await?;
        Ok(TokenInfo {
            access_token: token_response.access_token,
            refresh_token: Some(refresh_token.to_string()), // 既存のリフレッシュトークンを保持
            expires_in: token_response.expires_in,
            token_type: token_response.token_type,
        })
    }

    fn build_auth_url(&self) -> Result<String> {
        let mut url = Url::parse(GOOGLE_AUTH_URL)?;
        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id)
            .append_pair("redirect_uri", REDIRECT_URI)
            .append_pair("response_type", "code")
            .append_pair("scope", "https://www.googleapis.com/auth/drive.readonly https://www.googleapis.com/auth/drive.metadata.readonly")
            .append_pair("access_type", "offline")
            .append_pair("prompt", "consent");

        Ok(url.to_string())
    }

    fn wait_for_callback(&self) -> Result<String> {
        let listener = TcpListener::bind("127.0.0.1:8080")?;
        println!("認証コールバックを待機中...");

        for stream in listener.incoming() {
            let stream = stream?;
            if let Some(code) = self.handle_callback(stream)? {
                return Ok(code);
            }
        }

        Err(anyhow!("認証コードの取得に失敗しました"))
    }

    fn handle_callback(&self, mut stream: TcpStream) -> Result<Option<String>> {
        let buf_reader = BufReader::new(&mut stream);
        let request_line = buf_reader.lines().next().unwrap()?;

        if let Some(query_start) = request_line.find("?") {
            let query = &request_line[query_start + 1..];
            if let Some(path_end) = query.find(" ") {
                let query = &query[..path_end];
                let url = Url::parse(&format!("http://localhost:8080/callback?{}", query))?;

                for (key, value) in url.query_pairs() {
                    if key == "code" {
                        // 成功レスポンスを送信
                        let response = "HTTP/1.1 200 OK\r\n\r\n<html><body><h1>認証が完了しました！</h1><p>このタブを閉じてください。</p></body></html>";
                        stream.write_all(response.as_bytes())?;
                        return Ok(Some(value.to_string()));
                    }
                }
            }
        }

        // エラーレスポンスを送信
        let response = "HTTP/1.1 400 Bad Request\r\n\r\n<html><body><h1>認証エラー</h1></body></html>";
        stream.write_all(response.as_bytes())?;
        Ok(None)
    }

    async fn exchange_code_for_token(&self, code: &str) -> Result<TokenInfo> {
        let params = [
            ("client_id", &self.client_id),
            ("client_secret", &self.client_secret),
            ("code", &code.to_string()),
            ("grant_type", &"authorization_code".to_string()),
            ("redirect_uri", &REDIRECT_URI.to_string()),
        ];

        let response = self
            .client
            .post(GOOGLE_TOKEN_URL)
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("トークンの取得に失敗しました: {}", error_text));
        }

        let token_response: TokenResponse = response.json().await?;
        Ok(TokenInfo {
            access_token: token_response.access_token,
            refresh_token: token_response.refresh_token,
            expires_in: token_response.expires_in,
            token_type: token_response.token_type,
        })
    }
}