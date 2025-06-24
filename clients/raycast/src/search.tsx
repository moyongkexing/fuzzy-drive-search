import { ActionPanel, Action, Icon, List } from "@raycast/api";
import InitForm from "./init-form";
import { useSearch } from "./hooks/useSearch";
import { useInitialization } from "./hooks/useInitialization";
import { useSync } from "./hooks/useSync";

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
  const { searchText, setSearchText, results, isLoading, refreshSearch } = useSearch();
  const { initialize } = useInitialization();
  const { syncFiles } = useSync();

  const handleSync = () => {
    syncFiles(refreshSearch);
  };

  return (
    <List
      isLoading={isLoading}
      searchBarPlaceholder="Google Drive内のファイルを検索..."
      searchText={searchText}
      onSearchTextChange={setSearchText}
      throttle={false}
      filtering={false}
    >
      {results.length === 0 && searchText.trim() === "" ? (
        <List.Section title="操作">
          <List.Item
            title="初期設定"
            subtitle="Google Drive認証と初回同期を実行"
            icon={Icon.Gear}
            actions={
              <ActionPanel>
                <Action.Push title="初期設定フォームを開く" icon={Icon.Gear} target={<InitForm />} />
                <Action
                  title="既存の設定で初期化"
                  icon={Icon.ArrowClockwise}
                  onAction={initialize}
                  shortcut={{ modifiers: ["cmd"], key: "i" }}
                />
              </ActionPanel>
            }
          />
          <List.Item
            title="手動同期"
            subtitle="Google Driveのファイル一覧を強制更新"
            icon={Icon.RotateClockwise}
            actions={
              <ActionPanel>
                <Action title="同期を実行" icon={Icon.RotateClockwise} onAction={handleSync} />
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
                    onAction={handleSync}
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
