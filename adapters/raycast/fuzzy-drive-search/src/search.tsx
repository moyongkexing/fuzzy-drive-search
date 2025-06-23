import { ActionPanel, Action, Icon, List, showToast, Toast } from "@raycast/api";
import { useCachedPromise } from "@raycast/utils";
import { execSync } from "child_process";
import { useState } from "react";

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

const binaryPath =
  "/Users/suenagakatsuyuki/Documents/claude-desktop/fuzzy-drive-search/target/release/fuzzy-drive-search";

const executeCommand = (command: string, options: { timeout?: number } = {}) => {
  return execSync(`${binaryPath} ${command}`, {
    encoding: "utf8",
    timeout: options.timeout || 10_000,
  });
};

const parseSearchOutput = (output: string): SearchResult[] => {
  if (!output.trim()) return [];

  try {
    const jsonStart = output.indexOf("{");
    if (jsonStart === -1) return [];

    const jsonPart = output.substring(jsonStart);
    const results: SearchResults = JSON.parse(jsonPart);
    return results.items || [];
  } catch (error) {
    console.error("JSON解析エラー:", error);
    return [];
  }
};

const getIconForMimeType = (filename: string) => {
  const ext = filename.split(".").pop()?.toLowerCase() || "";
  const iconMap: Record<string, Icon> = {
    pdf: Icon.Document,
    doc: Icon.Text,
    docx: Icon.Text,
    xls: Icon.BarChart,
    xlsx: Icon.BarChart,
    ppt: Icon.Monitor,
    pptx: Icon.Monitor,
    png: Icon.Image,
    jpg: Icon.Image,
    jpeg: Icon.Image,
    txt: Icon.Text,
  };
  return iconMap[ext] || Icon.Document;
};

export default function Command() {
  const [searchText, setSearchText] = useState("");

  const {
    data: results = [],
    isLoading,
    mutate,
  } = useCachedPromise(
    async (query: string) => {
      if (!query.trim()) return [];

      try {
        const escapedQuery = query.replace(/'/g, "'\"'\"'");
        const output = executeCommand(`search '${escapedQuery}'`);
        return parseSearchOutput(output);
      } catch (error) {
        showToast({
          style: Toast.Style.Failure,
          title: "検索エラー",
          message: error instanceof Error ? error.message : "検索に失敗しました",
        });
        return [];
      }
    },
    [searchText],
    {
      keepPreviousData: true,
      execute: searchText.trim().length > 0,
      failureToastOptions: { title: "検索エラー" },
    },
  );

  const initialize = async () => {
    try {
      showToast({
        style: Toast.Style.Animated,
        title: "初期化中...",
        message: "Google Drive認証を開始します",
      });

      executeCommand("init", { timeout: 60_000 });

      showToast({
        style: Toast.Style.Success,
        title: "初期化完了",
        message: "Google Drive認証が完了しました",
      });
    } catch (error) {
      showToast({
        style: Toast.Style.Failure,
        title: "初期化エラー",
        message: error instanceof Error ? error.message : "初期化に失敗しました",
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

      executeCommand("sync", { timeout: 30_000 });

      showToast({
        style: Toast.Style.Success,
        title: "同期完了",
        message: "ファイル一覧を更新しました",
      });

      mutate();
    } catch (error) {
      showToast({
        style: Toast.Style.Failure,
        title: "同期エラー",
        message: error instanceof Error ? error.message : "同期に失敗しました",
      });
    }
  };

  return (
    <List
      isLoading={isLoading}
      searchBarPlaceholder="Google Drive内のファイルを検索..."
      onSearchTextChange={setSearchText}
      throttle={false}
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
      ) : results.length > 0 ? (
        <List.Section title={`検索結果 (${results.length}件)`}>
          {results.map((item) => (
            <List.Item
              key={item.uid}
              title={item.title}
              subtitle={item.subtitle}
              icon={getIconForMimeType(item.title)}
              actions={
                <ActionPanel>
                  <Action.OpenInBrowser title="ブラウザで開く" icon={Icon.Globe} url={item.arg} />
                  <Action.CopyToClipboard title="URLをコピー" icon={Icon.Clipboard} content={item.arg} />
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
      ) : (
        <List.EmptyView
          title="検索結果が見つかりません"
          description={`"${searchText}" に一致するファイルがありません`}
          icon={Icon.MagnifyingGlass}
        />
      )}
    </List>
  );
}
