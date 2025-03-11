mod cli;
mod logger;
mod processor;

use anyhow::Result;
use log::info;
use std::path::PathBuf;
use std::time::Instant;

fn main() {
    if let Err(e) = run() {
        eprintln!("エラー: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    // コマンドライン引数をパース
    let args = cli::parse_args();

    // 入力・出力ディレクトリのパスを解決
    let current_dir = std::env::current_dir()?;
    let input_dir = resolve_path(&args.input_dir, &current_dir);
    let output_dir = resolve_path(&args.output_dir, &current_dir);

    // 処理対象のファイル数を概算（ロギング設定の前に必要）
    let file_count = estimate_file_count(&input_dir)?;

    // スマートロギングを設定（プログレスバー付き）
    logger::setup_logging_and_progress(file_count)?;

    info!("JPEG画像処理ユーティリティを開始します");
    info!("入力ディレクトリ: {}", input_dir.display());
    info!("出力ディレクトリ: {}", output_dir.display());

    let start_time = Instant::now();

    // 改良された処理器を使用
    processor::process_images(&input_dir, &output_dir)?;

    let elapsed = start_time.elapsed();
    info!(
        "処理が完了しました。所要時間: {:.2}秒",
        elapsed.as_secs_f64()
    );

    Ok(())
}

/// 相対パスまたは絶対パスを適切に解決する
fn resolve_path(path: &PathBuf, current_dir: &PathBuf) -> PathBuf {
    if path.is_absolute() {
        path.clone()
    } else {
        current_dir.join(path)
    }
}

/// 対象ファイル数を概算（正確さよりも速度を優先）
fn estimate_file_count(dir: &PathBuf) -> Result<usize> {
    use walkdir::WalkDir;

    if !dir.exists() {
        return Ok(0);
    }

    let mut count = 0;
    for entry in WalkDir::new(dir)
        .max_depth(10)
        .into_iter()
        .filter_map(|res| res.ok())
    {
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension() {
                if let Some(ext_str) = ext.to_str() {
                    let lower_ext = ext_str.to_lowercase();
                    if lower_ext == "jpg" || lower_ext == "jpeg" {
                        count += 1;
                    }
                }
            }
        }
    }

    Ok(count)
}
