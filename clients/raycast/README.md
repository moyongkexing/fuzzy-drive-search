# Fuzzy Drive Search

Google Driveのファイルを高速に曖昧検索できるRaycast拡張機能

## 概要

この拡張機能は、Google Drive内のファイルを日本語・英語・ローマ字で高速に検索できるツールです。ファイル名だけでなく、ファイルに設定されたキーワードでも検索可能です。

### 主な機能

- 日本語、英語、ローマ字での曖昧検索
- ファイル名とキーワードによる検索
- 親フォルダ名の表示
- 高速なローカルキャッシュ検索

## インストール方法

### 社内配布版のセットアップ

1. リポジトリをクローン
```bash
git clone <リポジトリURL>
cd fuzzy-drive-search/clients/raycast
```

2. 依存関係をインストール
```bash
npm install
```

3. 開発モードで起動（初回のみ）
```bash
npm run dev
```

4. Raycastで「Drive File Search」を検索して使用開始

**注意**: 一度 `npm run dev` を実行すると、開発サーバーを停止してもRaycastに拡張機能が登録され、PC再起動後も使用可能です。

## 使い方

1. Raycastを開く（デフォルト: `Cmd + Space`）
2. 「Drive File Search」と入力またはショートカットキーを使用
3. 検索したいファイル名やキーワードを入力
4. Enterキーでファイルを開く

### 初回セットアップ

1. 拡張機能を初めて使用する際は「初期設定」画面が表示されます
2. 「Google Drive認証を開始」をクリック
3. ブラウザでGoogleアカウントにログイン
4. 同期したいフォルダのURLを入力（複数可）
5. 「同期開始」をクリック

## データ保存場所

この拡張機能は以下の場所にデータを保存します：

**macOS**: `~/Library/Application Support/fuzzy-drive-search/`

保存されるファイル：
- `files_cache.json` - Google Driveファイルのキャッシュ（検索用）
- `config.json` - 同期対象フォルダなどの設定
- `token.json` - Google認証トークン（自動更新）

## トラブルシューティング

### 検索結果が表示されない

1. 同期が完了しているか確認
2. 設定画面から再同期を実行

### 認証エラーが発生する

1. `~/Library/Application Support/fuzzy-drive-search/token.json` を削除
2. 拡張機能を再起動して再認証

## ライセンス

MIT