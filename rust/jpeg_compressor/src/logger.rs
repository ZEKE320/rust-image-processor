use anyhow::Result;
use chrono::Local;
use colored::*;
use fern::colors::{Color, ColoredLevelConfig};
use log::LevelFilter;
use regex::Regex;
use std::io::Write;
use std::sync::Mutex;

use crate::cli::LogLevel;

// プログレスバーを管理するためのグローバル変数
pub static PROGRESS_MANAGER: Mutex<ProgressManager> = Mutex::new(ProgressManager::new());

// プログレスバー管理のための構造体
pub struct ProgressManager {
    pub enabled: bool,
}

impl Default for ProgressManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressManager {
    pub const fn new() -> Self {
        Self { enabled: false }
    }

    // プログレスバーが有効になっているかを確認する
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    // プログレスバーを有効にする
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    // プログレスバーを無効にする
    pub fn disable(&mut self) {
        self.enabled = false;
    }
}

/// ロガーを初期化する
pub fn init_logger(log_level: LogLevel) -> Result<()> {
    // ログレベルの設定
    let level_filter = match log_level {
        LogLevel::Error => LevelFilter::Error,
        LogLevel::Warn => LevelFilter::Warn,
        LogLevel::Info => LevelFilter::Info,
        LogLevel::Debug => LevelFilter::Debug,
        LogLevel::Trace => LevelFilter::Trace,
    };

    // カラー設定
    let colors = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::Green)
        .debug(Color::Cyan)
        .trace(Color::BrightBlack);

    // ファイルパスの汎用正規表現（多様なパス形式と拡張子に対応）
    let path_pattern = Regex::new(
        r#"((^|[\s\(\["'])([A-Za-z]:)?[/\\]([^<>:"\|?*\r\n]|[^\x00-\x7F])*\.[a-zA-Z0-9]+)"#,
    )?;

    // 数値パターンの汎用正規表現（単位、分数、パーセンテージなど様々な形式に対応）
    let number_pattern =
        Regex::new(r"(\b\d+(\.\d+)?([KkMmGgTtPp]i?[Bb]|%|秒|分|時間|日|枚/秒)?\b)|(\d+/\d+)")?;

    // メインディスパッチの設定
    let dispatch = fern::Dispatch::new()
        .format(move |out, message, record| {
            // プログレスバーが有効な場合は一時的にクリア
            let progress_active = PROGRESS_MANAGER.lock().unwrap().is_enabled();
            if progress_active {
                // プログレスバーをクリアするための改行
                print!("\r\x1B[K");
                let _ = std::io::stdout().flush();
            }

            // 時間を青色で表示
            let timestamp = Local::now()
                .format("[%Y-%m-%d %H:%M:%S]")
                .to_string()
                .blue()
                .bold();

            // モジュール名を黄色で表示
            let module = record.target();
            let target_parts: Vec<&str> = module.split("::").collect();
            let target = if target_parts.len() > 1 {
                format!(
                    "[{}::{}]",
                    target_parts[0].yellow(),
                    target_parts[1..].join("::").yellow().bold()
                )
            } else {
                format!("[{}]", module.yellow())
            };

            // ログレベルはfernの標準カラー
            let level = colors.color(record.level());

            // メッセージ内の色付け - パターンを先に処理
            let mut colored_message = message.to_string();

            // パスを緑色に - ファイル名をハイライト
            colored_message = path_pattern
                .replace_all(&colored_message, |caps: &regex::Captures| {
                    caps[0].green().to_string()
                })
                .to_string();

            // 数値を紫色に
            colored_message = number_pattern
                .replace_all(&colored_message, |caps: &regex::Captures| {
                    caps[0].purple().bold().to_string()
                })
                .to_string();

            // フォーマット済みの最終メッセージ
            let formatted = format!("{} {} [{}] {}", timestamp, target, level, colored_message);

            // 直接stdoutに出力してフォーマット制御
            println!("{}", formatted);

            // 改行を追加してプログレスバーとの間隔を確保
            if progress_active {
                let _ = std::io::stdout().flush();
            }

            // fernのログ出力用（ファイル出力用）
            let plain_timestamp = Local::now().format("[%Y-%m-%d %H:%M:%S]");
            out.finish(format_args!(
                "{} [{}] [{}] {}",
                plain_timestamp,
                record.target(),
                record.level(),
                message
            ));
        })
        .level(level_filter)
        .chain(fern::log_file("jpeg_compressor.log")?);

    // フィルタと適用
    dispatch.apply()?;

    Ok(())
}
