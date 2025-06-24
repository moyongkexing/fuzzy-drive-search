use fuzzy_drive_search_core::services::SearchService;
use serde_json;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    // 引数の解析
    if args.len() < 2 {
        print_help();
        return Ok(());
    }

    let command = &args[1];
    
    match command.as_str() {
        "init" => {
            handle_init(&args[2..]).await?;
        }
        "sync" => {
            handle_sync().await?;
        }
        "--help" | "-h" | "help" => {
            print_help();
        }
        _ => {
            print_help();
        }
    }

    Ok(())
}

fn print_help() {
    let help_items = vec![
        serde_json::json!({
            "title": "Fuzzy Drive Search - ヘルプ",
            "subtitle": "使用方法: fuzzy-drive-search [init|sync]",
            "valid": false
        }),
        serde_json::json!({
            "title": "init - 初期設定",
            "subtitle": "Google Drive認証と初回同期を実行します",
            "valid": false
        }),
        serde_json::json!({
            "title": "sync - 手動同期",
            "subtitle": "設定された複数フォルダの直下ファイル一覧を強制同期します",
            "valid": false
        })
    ];
    
    let output = serde_json::json!({
        "items": help_items
    });
    println!("{}", serde_json::to_string_pretty(&output).unwrap_or_default());
}

async fn handle_init(args: &[String]) -> anyhow::Result<()> {
    println!("Fuzzy Drive Search の初期化を開始します...");
    
    let (client_id, client_secret) = parse_auth_args(args)?;
    let mut service = SearchService::new()?;
    
    if client_id.is_some() || client_secret.is_some() {
        service.initialize_with_overrides(client_id, client_secret).await?;
    } else {
        service.ensure_initialized().await?;
    }
    
    println!("初期化が完了しました");
    Ok(())
}

fn parse_auth_args(args: &[String]) -> anyhow::Result<(Option<String>, Option<String>)> {
    let mut client_id = None;
    let mut client_secret = None;
    
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        
        if let Some(value) = arg.strip_prefix("--client-id=") {
            client_id = Some(value.to_string());
            i += 1;
        } else if let Some(value) = arg.strip_prefix("--client-secret=") {
            client_secret = Some(value.to_string());
            i += 1;
        } else if arg == "--client-id" {
            if i + 1 >= args.len() {
                return Err(anyhow::anyhow!("--client-id には値が必要です"));
            }
            client_id = Some(args[i + 1].clone());
            i += 2;
        } else if arg == "--client-secret" {
            if i + 1 >= args.len() {
                return Err(anyhow::anyhow!("--client-secret には値が必要です"));
            }
            client_secret = Some(args[i + 1].clone());
            i += 2;
        } else {
            i += 1;
        }
    }
    
    Ok((client_id, client_secret))
}


async fn handle_sync() -> anyhow::Result<()> {
    let mut service = SearchService::new()?;
    service.sync_files().await?;
    println!("同期が完了しました");
    Ok(())
}