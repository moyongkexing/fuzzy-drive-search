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

export function useInitialization() {
  const initialize = async () => {
    const isBackground = environment.launchType === LaunchType.Background;
    
    try {
      if (!isBackground) {
        showToast({
          style: Toast.Style.Animated,
          title: "初期化中...",
          message: "Google Drive認証を開始します",
        });
      }

      await executeCommandAsync("init", { timeout: 60_000 });

      if (!isBackground) {
        showToast({
          style: Toast.Style.Success,
          title: "初期化完了",
          message: "Google Drive認証が完了しました",
        });
      }
    } catch (error) {
      if (!isBackground) {
        showToast({
          style: Toast.Style.Failure,
          title: "初期化エラー",
          message: error instanceof Error ? error.message : "初期化に失敗しました",
        });
      }
    }
  };

  return {
    initialize,
  };
}
