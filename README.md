# Google Drive連携曖昧検索アプリケーション

Google Driveのファイルを曖昧検索し、Raycastを含む各ツールから直接ブラウザで開けるツール

## 主要機能

- **バックグラウンド同期**: 1時間ごとにDrive API叩いてメタデータ更新
- **曖昧検索**: ファイル名に対する高速fuzzy search
- **Raycast連携**: 検索結果をRaycast形式で出力
- **直接アクセス**: 選択時にブラウザでDriveファイルを開く

## アーキテクチャ

```
core/
├── domain/          # DriveFile, DriveIndex エンティティ
├── application/     # 同期・検索ユースケース
└── infrastructure/ # Drive API client, SQLite永続化
adapters/
└── raycast/        # Raycast JSON出力 + バイナリ
```

## 技術スタック

- **Drive API**: `reqwest` + OAuth2認証
- **永続化**: SQLite（`rusqlite`）
- **バックグラウンド処理**: `tokio` スケジューラー
- **曖昧検索**: `skim` または `fuzzy-matcher`

## 実装順序

1. Google Drive API認証・クライアント実装
2. ファイルメタデータ同期機能
3. SQLiteでの永続化
4. 曖昧検索エンジン
5. バックグラウンド同期スケジューラー
6. Raycast アダプター
7. 統合テスト

## 認証フロー

- **初回**: OAuth2 → アクセストークン取得 → リフレッシュトークン保存
- **以降**: リフレッシュトークンで自動更新

## ユースケース例

1. ユーザーがRaycastで「画面　ログイン」と検索
2. アプリがローカルDBから関連ファイルを曖昧検索
3. 「ログイン画面_設計書.docx」「ログイン仕様書.pdf」等を表示
4. ユーザーが選択 → ファイル形式に応じて適切なアプリで開く（PDF、スプレッドシート、ドキュメント等）




**注意:** 疎通テストには有効なGoogle Drive APIアクセストークンが必要です。

## セットアップ手順

### 1. Google Drive API設定

1. [Google Cloud Console](https://console.cloud.google.com/) でプロジェクトを作成
2. Google Drive API を有効化
3. OAuth 2.0 クライアントIDとシークレットを作成
4. リダイレクトURIに `http://localhost:8080/callback` を追加

### 2. ビルドとインストール

```bash
# ビルド
cargo build --release

# バイナリをシステムパスにコピー
sudo cp target/release/fuzzy-drive-search /usr/local/bin/

# 実行権限を付与
sudo chmod +x /usr/local/bin/fuzzy-drive-search
```

### 3. 初期設定

```bash
# 初期設定（設定ファイルが作成される）
fuzzy-drive-search init
```

設定ファイル `~/.config/fuzzy-drive-search/config.toml` を編集：

```toml
target_folder_id = ["your_google_drive_folder_id", "your_google_drive_folder_id"]
google_client_id = "your_client_id.apps.googleusercontent.com"
google_client_secret = "your_client_secret"
```

### 4. Raycast連携

1. Raycastエクステンションのインストール:
   ```bash
   cd adapters/raycast
   npm install
   ray develop
   ```

2. Raycastで「Drive File Search」コマンドを使用

## 使用方法

- **検索**: Raycastで「Drive File Search」を開き、ファイル名を入力
- **同期**: `Cmd+R` で手動同期
- **初期設定**: 初回はRaycast内で「初期設定」を実行