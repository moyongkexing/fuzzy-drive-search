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

## テスト実行

**通常のテスト:**
```bash
cargo test
```

**Google Drive API疎通テスト（実際のAPIを使用）:**
```bash
# 環境変数にアクセストークンを設定
export GOOGLE_ACCESS_TOKEN="your_access_token_here"

# 疎通テストを実行
cargo test --ignored
```

**注意:** 疎通テストには有効なGoogle Drive APIアクセストークンが必要です。