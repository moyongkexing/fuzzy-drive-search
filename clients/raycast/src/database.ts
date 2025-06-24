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

  // スペース分割によるAND検索をサポート
  const queryLower = query.toLowerCase();
  const words = queryLower.split(/\s+/).filter(word => word.length > 0);
  const isMultiWord = words.length > 1;

  const results = cachedFiles.filter((file) => {
    if (isMultiWord) {
      // 複数単語の場合：すべての単語が含まれているかチェック
      return words.every((word) => {
        const fileName = file.name.toLowerCase();
        const inFileName = fileName.includes(word);
        const inKeywords = file.keywords?.some((keyword) => keyword.toLowerCase().includes(word)) || false;
        const inRomaji = file.romaji_keywords?.some((romaji) => romaji.toLowerCase().includes(word)) || false;
        
        return inFileName || inKeywords || inRomaji;
      });
    } else {
      // 単一単語の場合：従来の方法
      const fileName = file.name.toLowerCase();
      if (fileName.includes(queryLower)) {
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
    }
  });

  // 関連度でソート（複数単語の場合も考慮）
  results.sort((a, b) => {
    if (isMultiWord) {
      // 複数単語の場合：より多くの単語がファイル名に含まれる方を優先
      const aScore = words.filter(word => a.name.toLowerCase().includes(word)).length;
      const bScore = words.filter(word => b.name.toLowerCase().includes(word)).length;
      if (aScore !== bScore) return bScore - aScore;
    } else {
      // 単一単語の場合：ファイル名に含まれる場合を優先
      const aInName = a.name.toLowerCase().includes(queryLower);
      const bInName = b.name.toLowerCase().includes(queryLower);
      if (aInName && !bInName) return -1;
      if (!aInName && bInName) return 1;
    }
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
