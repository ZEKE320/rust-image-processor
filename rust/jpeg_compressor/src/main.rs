use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context, Result};
use chrono::Local;
use clap::Parser;
use log::info;

// パッケージ名をjpeg_compressorに変更
use jpeg_compressor::{compress_jpeg_directory, logger, Cli};

fn main() -> Result<()> {
    // コマンドライン引数の解析
    let cli = Cli::parse();

    // ログ設定
    logger::init_logger(cli.log_level)?;

    // 入力パスの検証
    let input_dir = PathBuf::from(&cli.input_dir)
        .canonicalize()
        .with_context(|| format!("入力ディレクトリが見つかりません: {}", cli.input_dir))?;

    // 出力ディレクトリの設定
    let output_dir = if let Some(output_path) = &cli.output_dir {
        PathBuf::from(output_path)
    } else {
        // 現在の日時を文字列に変換
        let datetime = Local::now().format("%Y%m%d_%H%M%S");
        let input_dirname = input_dir.file_name().unwrap_or_default().to_string_lossy();

        PathBuf::from(format!(
            "output/compressed/{}_{}_{}",
            datetime, input_dirname, cli.quality
        ))
    };

    // 圧縮設定の表示
    display_compression_config(&cli, &input_dir, &output_dir);

    // 出力ディレクトリが存在する場合の確認
    if output_dir.exists() && !cli.yes {
        println!("出力ディレクトリが既に存在します: {}", output_dir.display());
        print!("既存のファイルを上書きしますか？ (y/n): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() != "y" {
            println!("処理を中止しました。");
            return Ok(());
        }
    }

    // 自動実行でない場合はユーザー確認
    if !cli.yes {
        print!("圧縮処理を実行しますか？ (y/n): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() != "y" {
            println!("処理を中止しました。");
            return Ok(());
        }
    }

    // 開始時刻の記録
    let start_time = Instant::now();
    info!("処理開始時刻: {}", Local::now().format("%Y-%m-%d %H:%M:%S"));

    // スレッド数の設定
    let threads = if cli.threads == 0 {
        // CPUコア数の半分か、最低1スレッドを使用
        std::cmp::max(1, num_cpus::get() / 2)
    } else {
        cli.threads
    };
    info!(
        "設定スレッド数: {} (CPU: {} コア)",
        threads,
        num_cpus::get()
    );

    // 圧縮処理の実行
    let stats = compress_jpeg_directory(&input_dir, &output_dir, cli.quality, threads, cli.encoder)
        .with_context(|| "JPEG圧縮処理中にエラーが発生しました")?;

    // 処理時間の計算
    let elapsed = start_time.elapsed();
    let elapsed_sec = elapsed.as_secs_f64();

    // 処理速度の計算
    let speed = if elapsed_sec > 0.0 {
        stats.processed_files as f64 / elapsed_sec
    } else {
        0.0
    };

    // 結果の表示
    display_compression_results(output_dir, stats, elapsed_sec, speed);

    info!("処理終了時刻: {}", Local::now().format("%Y-%m-%d %H:%M:%S"));

    Ok(())
}

fn display_compression_config(cli: &Cli, input_dir: &Path, output_dir: &Path) {
    info!("==================================================");
    info!("JPEG圧縮ユーティリティ v{}", env!("CARGO_PKG_VERSION"));
    info!("==================================================");
    info!("実行時設定:");
    info!(" - 入力ディレクトリ: {}", input_dir.display());
    info!(" - 出力ディレクトリ: {}", output_dir.display());
    info!(" - 圧縮品質: {}/100", cli.quality);
    info!(
        " - スレッド数: {}",
        if cli.threads == 0 {
            "自動".to_string()
        } else {
            cli.threads.to_string()
        }
    );
    info!(" - エンコーダー: {}", cli.encoder);
    info!(" - ログレベル: {}", cli.log_level);
    info!("--------------------------------------------------");
}

/// バイト数を人間が読みやすい形式にフォーマットする
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

fn display_compression_results(
    output_dir: PathBuf,
    stats: jpeg_compressor::compressor::CompressionStats,
    elapsed_sec: f64,
    speed: f64,
) {
    info!("========================================");
    info!("JPEG圧縮処理 完了");
    info!("----------------------------------------");
    info!(
        "処理時間: {:.2}秒 ({:.2}分)",
        elapsed_sec,
        elapsed_sec / 60.0
    );
    info!("処理速度: {:.1}ファイル/秒", speed);
    info!("処理ファイル数: {}", stats.processed_files);
    info!("スキップファイル数: {}", stats.skipped_files);
    info!("エラーファイル数: {}", stats.error_files);

    if stats.processed_files > 0 {
        info!("平均圧縮率: {:.1}%", stats.get_compression_ratio() * 100.0);
        info!(
            "容量削減: {} → {} ({:.1}% 削減)",
            format_bytes(stats.original_size),
            format_bytes(stats.compressed_size),
            (1.0 - stats.get_size_ratio()) * 100.0
        );

        // 圧縮結果の詳細情報
        let saved_bytes = if stats.original_size > stats.compressed_size {
            stats.original_size - stats.compressed_size
        } else {
            0
        };

        info!("節約容量: {}", format_bytes(saved_bytes));
        info!("出力ディレクトリ: {}", output_dir.display());
    }
}
