#!/bin/bash

# Fuzzy Drive Search 簡易セットアップスクリプト
# macOS専用

set -e

# 色付きログ出力
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo "${GREEN}✅ $1${NC}"
}

log_warning() {
    echo "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo "${RED}❌ $1${NC}"
}

# スクリプトのディレクトリを取得
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "🚀 Fuzzy Drive Search セットアップを開始します..."
echo "📁 プロジェクトディレクトリ: $PROJECT_ROOT"

# 1. 依存関係のチェックとインストール
log_info "依存関係をチェックしています..."

# Homebrewのチェック
if ! command -v brew &> /dev/null; then
    log_error "Homebrewがインストールされていません"
    log_info "Homebrewをインストールしてください: https://brew.sh/"
    exit 1
fi
log_success "Homebrew: OK"

# Node.jsのチェック
if ! command -v node &> /dev/null; then
    log_warning "Node.jsがインストールされていません。インストールします..."
    brew install node
fi
log_success "Node.js: OK ($(node --version))"

# npmのチェック
if ! command -v npm &> /dev/null; then
    log_error "npmがインストールされていません"
    exit 1
fi
log_success "npm: OK ($(npm --version))"

# 2. Raycastのチェックとインストール
log_info "Raycastをチェックしています..."

if [ ! -d "/Applications/Raycast.app" ]; then
    log_warning "Raycastがインストールされていません"
    read -p "Raycastをインストールしますか? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        # Raycastをインストール
        log_info "📦 Homebrewを使用してRaycastをインストールします..."
        if brew install --cask raycast; then
            log_success "Raycastのインストールが完了しました"
            log_info "📌 Raycastを起動して初期設定を行ってください"
            log_info "📌 システム設定でRaycastにアクセシビリティ権限を付与してください"
            
            # Raycastを起動
            log_info "🚀 Raycastを起動します..."
            open /Applications/Raycast.app
            
            log_info "Raycastの初期設定を完了してから、このスクリプトを再実行してください"
            exit 0
        else
            log_error "Raycastのインストールに失敗しました"
            exit 1
        fi
    else
        log_error "Raycastが必要です。手動でインストールしてください"
        exit 1
    fi
fi
log_success "Raycast: OK"

# 3. Raycast拡張機能のセットアップ
log_info "Raycast拡張機能をセットアップしています..."
cd "$PROJECT_ROOT/clients/raycast"

# npm依存関係のインストール
if npm install; then
    log_success "npm依存関係インストール完了"
else
    log_error "npm依存関係のインストールに失敗しました"
    exit 1
fi

# 4. セットアップ完了案内
echo ""
echo "🎉 セットアップが完了しました！"
echo ""
echo "📋 使用方法:"
echo "   1. Raycastを開く (Cmd + Space)"
echo "   2. 'Drive File Search' と入力"
echo "   3. 初回は認証設定を行ってください"
echo "   4. Google Driveフォルダを同期してください"
echo ""
echo "🔧 拡張機能の場所:"
echo "   プロジェクト: $PROJECT_ROOT"
echo "   設定: ~/Library/Application Support/fuzzy-drive-search/"
echo ""
echo "❓ 問題がある場合は、README.mdを確認してください"
echo ""
log_success "セットアップ完了！🚀"
echo ""

# 開発サーバーの起動
log_info "開発サーバーを起動しています..."
log_warning "開発サーバーが起動します。Raycastに拡張機能が登録されます"
log_info "停止したい場合は Ctrl+C を押してください"
echo ""

npm run dev