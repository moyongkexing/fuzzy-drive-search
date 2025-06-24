import { showToast, Toast, environment, LaunchType } from "@raycast/api";
import { execSync } from "child_process";
import { join } from "path";

const binaryPath = join(__dirname, "../../../bin/fuzzy-drive-search");

const executeCommandAsync = async (command: string, options: { timeout?: number } = {}) => {
  return new Promise<string>((resolve, reject) => {
    setTimeout(() => {
      try {
        const result = execSync(`${binaryPath} ${command}`, {
          encoding: "utf8",
          timeout: options.timeout || 10_000,
        });
        resolve(result);
      } catch (error) {
        reject(error);
      }
    }, 0);
  });
};

export function useSync() {
  const syncFiles = async (onComplete?: () => void) => {
    const isBackground = environment.launchType === LaunchType.Background;
    
    try {
      // バックグラウンド実行時はToastを表示しない
      if (!isBackground) {
        showToast({
          style: Toast.Style.Animated,
          title: "同期中...",
          message: "Google Driveのファイル一覧を更新しています",
        });
      }

      await executeCommandAsync("sync", { timeout: 30_000 });

      if (!isBackground) {
        showToast({
          style: Toast.Style.Success,
          title: "同期完了",
          message: "ファイル一覧を更新しました",
        });
      }

      // 同期完了後のコールバック実行
      if (onComplete) {
        onComplete();
      }
    } catch (error) {
      if (!isBackground) {
        showToast({
          style: Toast.Style.Failure,
          title: "同期エラー",
          message: error instanceof Error ? error.message : "同期に失敗しました",
        });
      }
    }
  };

  return {
    syncFiles,
  };
}
