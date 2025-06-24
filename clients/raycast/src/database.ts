import { readFileSync, existsSync } from "fs";
import { homedir } from "os";
import { join } from "path";

interface DriveFile {
  id: string;
  name: string;
  web_view_link: string;
  mime_type: string;
  parents: string[];
  parent_folder_name: string;
  keywords?: string[];
  romaji_keywords?: string[];
}

interface DriveStorageData {
  files: DriveFile[];
  folders: Record<string, string>;
  last_sync: string;
  sync_token?: string;
}

let cachedFiles: DriveFile[] = [];
let lastLoadTime = 0;

function loadCacheFromJson(): void {
  const storagePath = join(homedir(), "Library", "Application Support", "fuzzy-drive-search", "drive_files.json");

  try {
    if (!existsSync(storagePath)) {
      console.warn("JSONストレージファイルが見つかりません:", storagePath);
      return;
    }

    const content = readFileSync(storagePath, "utf8");
    const storageData: DriveStorageData = JSON.parse(content);

    cachedFiles = storageData.files;
    lastLoadTime = Date.now();
  } catch (error) {
    console.error("JSONストレージ読み込みエラー:", error);
  }
}

// fuse.jsを使用した高性能ファジー検索
export function fuzzySearchFiles(query: string): Array<{
  title: string;
  subtitle: string;
  arg: string;
  uid: string;
  valid: boolean;
  mimeType?: string;
}> {
  if (!query.trim()) return [];

  // 5分以上経過していたらJSONを再読み込み
  if (Date.now() - lastLoadTime > 5 * 60 * 1000) {
    loadCacheFromJson();
  }

  // 初回読み込み
  if (cachedFiles.length === 0) {
    loadCacheFromJson();
  }

  // シンプルなファジー検索を実装
  const queryLower = query.toLowerCase();
  const results = cachedFiles.filter((file) => {
    // ファイル名での検索
    if (file.name.toLowerCase().includes(queryLower)) {
      return true;
    }
    // キーワードでの検索
    if (file.keywords?.some((keyword) => keyword.toLowerCase().includes(queryLower))) {
      return true;
    }
    // ローマ字キーワードでの検索
    if (file.romaji_keywords?.some((romaji) => romaji.toLowerCase().includes(queryLower))) {
      return true;
    }
    return false;
  });

  // 関連度でソート（ファイル名に含まれる場合を優先）
  results.sort((a, b) => {
    const aInName = a.name.toLowerCase().includes(queryLower);
    const bInName = b.name.toLowerCase().includes(queryLower);
    if (aInName && !bInName) return -1;
    if (!aInName && bInName) return 1;
    return 0;
  });

  // 上位20件を返す
  return results.slice(0, 20).map((file) => ({
    title: file.name,
    subtitle: file.parent_folder_name,
    arg: file.web_view_link,
    uid: file.id,
    valid: true,
    mimeType: file.mime_type,
  }));
}
