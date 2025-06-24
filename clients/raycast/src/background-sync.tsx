import { environment, LaunchType } from "@raycast/api";
import { useSync } from "./hooks/useSync";

export default function BackgroundSync() {
  const { syncFiles } = useSync();

  // バックグラウンド実行の場合のみ同期を実行
  if (environment.launchType === LaunchType.Background) {
    syncFiles();
  }

  // no-viewモードなので何も表示しない
  return null;
}