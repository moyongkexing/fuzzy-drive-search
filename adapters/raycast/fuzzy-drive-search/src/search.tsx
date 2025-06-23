import { ActionPanel, Action, Icon, List, showToast, Toast } from "@raycast/api";
import { useState, useEffect } from "react";

import { execSync } from "child_process";

interface SearchResult {
  title: string;
  subtitle: string;
  arg: string;
  uid: string;
  valid: boolean;
  icon?: {
    type: string;
    path: string;
  };
}

interface SearchResults {
  items: SearchResult[];
}

export default function Command(): Element {
  const [searchText, setSearchText] = useState("");
  const [results, setResults] = useState<SearchResult[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const binaryPath = "/Users/suenagakatsuyuki/Documents/claude-desktop/fuzzy-drive-search/target/release/fuzzy-drive-search";

  const performSearch = async (query: string) => {
    setIsLoading(true);
    try {
      // 特殊文字をエスケープしてシェルインジェクションを防ぐ
      const escapedQuery = query.replace(/'/g, "'\"'\"'");
      const output = execSync(`${binaryPath} search '${escapedQuery}'`, {
        encoding: "utf8",
        timeout: 10000,
      });
      
      // 空の出力や無効なJSONをチェック
      if (!output.trim()) {
        setResults([]);
        return;
      }
      
      // 出力をデバッグ表示
      console.log("Raw output:", JSON.stringify(output));
      
      // JSONデータの開始位置を見つける（{で始まる部分）
      const jsonStart = output.indexOf('{');
      
      if (jsonStart === -1) {
        console.log("JSON形式のデータが見つかりません。出力:", output);
        setResults([]);
        return;
      }
      
      // JSONの開始位置から最後まで抽出
      const jsonPart = output.substring(jsonStart).trim();
      
      // 複数行にわたるJSONの場合、最後の}まで取得
      let jsonData = '';
      let braceCount = 0;
      for (let i = 0; i < jsonPart.length; i++) {
        const char = jsonPart[i];
        jsonData += char;
        
        if (char === '{') {
          braceCount++;
        } else if (char === '}') {
          braceCount--;
          if (braceCount === 0) {
            break;
          }
        }
      }
      
      console.log("Extracted JSON:", JSON.stringify(jsonData));
      
      const searchResults: SearchResults = JSON.parse(jsonData);
      setResults(searchResults.items || []);
    } catch (error) {
      console.error("検索エラー:", error);
      
      // JSONパースエラーの場合は詳細なメッセージを表示
      let errorMessage = "検索でエラーが発生しました";
      if (error instanceof Error) {
        if (error.message.includes("JSON")) {
          errorMessage = "検索結果の解析に失敗しました。出力形式を確認してください。";
        } else {
          errorMessage = error.message;
        }
      }
      
      showToast({
        style: Toast.Style.Failure,
        title: "検索エラー",
        message: errorMessage,
      });
      setResults([]);
    } finally {
      setIsLoading(false);
    }
  };

  const initialize = async () => {
    try {
      showToast({
        style: Toast.Style.Animated,
        title: "初期化中...",
        message: "Google Drive認証を開始します",
      });

      execSync(`${binaryPath} init`, {
        encoding: "utf8",
        timeout: 60000,
      });

      showToast({
        style: Toast.Style.Success,
        title: "初期化完了",
        message: "Google Drive認証が完了しました",
      });
    } catch (error) {
      showToast({
        style: Toast.Style.Failure,
        title: "初期化エラー",
        message: `初期化に失敗しました: ${error instanceof Error ? error.message : String(error)}`,
      });
    }
  };

  const syncFiles = async () => {
    try {
      showToast({
        style: Toast.Style.Animated,
        title: "同期中...",
        message: "Google Driveのファイル一覧を更新しています",
      });

      execSync(`${binaryPath} sync`, {
        encoding: "utf8",
        timeout: 30000,
      });

      showToast({
        style: Toast.Style.Success,
        title: "同期完了",
        message: "ファイル一覧を更新しました",
      });

      if (searchText.trim()) {
        performSearch(searchText);
      }
    } catch (error) {
      showToast({
        style: Toast.Style.Failure,
        title: "同期エラー",
        message: `同期に失敗しました: ${error instanceof Error ? error.message : String(error)}`,
      });
    }
  };

  const getIconForMimeType = (mimeType: string) => {
    switch (mimeType) {
      case "application/pdf":
        return Icon.Document;
      case "application/vnd.openxmlformats-officedocument.wordprocessingml.document":
        return Icon.Text;
      case "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet":
        return Icon.BarChart;
      case "application/vnd.openxmlformats-officedocument.presentationml.presentation":
        return Icon.Monitor;
      case "image/png":
      case "image/jpg":
      case "image/jpeg":
        return Icon.Image;
      case "text/plain":
        return Icon.Text;
      default:
        return Icon.Document;
    }
  };

  useEffect(() => {
    const timeoutId = setTimeout(() => {
      performSearch(searchText);
    }, 300);

    return () => clearTimeout(timeoutId);
  }, [searchText]);

  return (
    <List
      isLoading={isLoading}
      onSearchTextChange={setSearchText}
      searchBarPlaceholder="Google Drive内のファイルを検索..."
      throttle
    >
      {results.length === 0 && searchText.trim() === "" ? (
        <List.Section title="操作">
          <List.Item
            title="初期設定"
            subtitle="Google Drive認証と初回同期を実行"
            icon={Icon.Gear}
            actions={
              <ActionPanel>
                <Action title="初期設定を実行" icon={Icon.Gear} onAction={initialize} />
              </ActionPanel>
            }
          />
          <List.Item
            title="手動同期"
            subtitle="Google Driveのファイル一覧を強制更新"
            icon={Icon.RotateClockwise}
            actions={
              <ActionPanel>
                <Action title="同期を実行" icon={Icon.RotateClockwise} onAction={syncFiles} />
              </ActionPanel>
            }
          />
        </List.Section>
      ) : (
        <List.Section title={`検索結果 (${results.length}件)`}>
          {results.map((item: SearchResult) => (
            <List.Item
              key={item.uid}
              title={item.title}
              subtitle={item.subtitle}
              icon={getIconForMimeType(item.title.split('.').pop() || "")}
              actions={
                <ActionPanel>
                  <Action.OpenInBrowser
                    title="ブラウザで開く"
                    icon={Icon.Globe}
                    url={item.arg}
                  />
                  <Action.CopyToClipboard
                    title="URLをコピー"
                    icon={Icon.Clipboard}
                    content={item.arg}
                  />
                  <Action
                    title="手動同期"
                    icon={Icon.RotateClockwise}
                    onAction={syncFiles}
                    shortcut={{ modifiers: ["cmd"], key: "r" }}
                  />
                </ActionPanel>
              }
            />
          ))}
        </List.Section>
      )}
    </List>
  );
}
