use anyhow::Result;
use chrono::Local;
use fern::colors::{Color, ColoredLevelConfig};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use log::{LevelFilter, Log, Metadata, Record, SetLoggerError};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, Once};

// シングルトンパターンを使ってグローバルプログレスバーを管理
static PROGRESS_INIT: Once = Once::new();
static mut MULTI_PROGRESS: Option<MultiProgress> = None;
static mut MAIN_PROGRESS: Option<ProgressBar> = None;

// プログレスバーの上に表示されるカスタムロガー
pub struct ProgressBarLogger {
    file: Mutex<File>,
    colors: ColoredLevelConfig,
    log_level: LevelFilter,
}

impl ProgressBarLogger {
    pub fn new(log_file: &PathBuf, log_level: LevelFilter) -> Result<Self> {
        let file = File::create(log_file)?;

        let colors = ColoredLevelConfig::new()
            .error(Color::Red)
            .warn(Color::Yellow)
            .info(Color::Green)
            .debug(Color::Blue)
            .trace(Color::Magenta);

        Ok(Self {
            file: Mutex::new(file),
            colors,
            log_level,
        })
    }

    // グローバルロガーとして初期化
    pub fn init(self) -> std::result::Result<(), SetLoggerError> {
        let max_level = self.log_level;
        log::set_boxed_logger(Box::new(self))?;
        log::set_max_level(max_level);
        Ok(())
    }
}

impl Log for ProgressBarLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.log_level
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        // ターミナルへの出力 - プログレスバーを考慮
        let level_str = self.colors.color(record.level()).to_string();
        let log_message = format!(
            "{} [{}] {} ({}:{})",
            Local::now().format("[%H:%M:%S]"),
            level_str,
            record.args(),
            record.file().unwrap_or("unknown"),
            record.line().unwrap_or(0)
        );

        // 修正: &raw const を使う場合は参照外し（dereference）が必要
        unsafe {
            // 生ポインタを取得して参照外し
            let main_progress_ptr = &raw const MAIN_PROGRESS;
            // 参照外し演算子 * を使って、ポインタの中身のOptionにアクセス
            if let Some(pb) = (*main_progress_ptr).as_ref() {
                pb.suspend(|| {
                    println!("{}", log_message);
                });
            } else {
                println!("{}", log_message);
            }
        }

        // ファイルへのログ出力
        if let Ok(mut file) = self.file.lock() {
            let file_log = format!(
                "{} - {} - {} - {}:{}\n",
                Local::now().format("[%H:%M:%S]"),
                record.level(),
                record.args(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
            );
            let _ = file.write_all(file_log.as_bytes());
            let _ = file.flush();
        }
    }

    fn flush(&self) {
        if let Ok(mut file) = self.file.lock() {
            let _ = file.flush();
        }
    }
}

/// ロギングとプログレスバーを統合して設定
pub fn setup_logging_and_progress(total_items: usize) -> Result<PathBuf> {
    let current_datetime = Local::now();
    let formatted_date = current_datetime.format("%Y-%m-%d_%H-%M-%S").to_string();

    let log_dir = PathBuf::from("logs").join("fix_jpeg").join(formatted_date);
    fs::create_dir_all(&log_dir)?;

    let log_file = log_dir.join("fix_jpeg.log");

    // ロガーを初期化
    let logger = ProgressBarLogger::new(&log_file, LevelFilter::Info)?;
    logger.init()?;

    // グローバルプログレスバーを初期化（一度だけ実行）
    PROGRESS_INIT.call_once(|| {
        let mp = MultiProgress::new();
        let pb = mp.add(ProgressBar::new(total_items as u64));

        pb.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:30.cyan/blue}] {pos}/{len} {msg}",
                )
                .unwrap()
                .progress_chars("=>-"),
        );
        pb.set_message("処理中...");

        unsafe {
            MULTI_PROGRESS = Some(mp);
            MAIN_PROGRESS = Some(pb);
        }
    });

    Ok(log_dir)
}

/// メインプログレスバーを取得
pub fn get_progress_bar() -> ProgressBar {
    unsafe {
        // 初期化されていない場合はダミーを返す
        let main_progress_ptr = &raw const MAIN_PROGRESS;
        match (*main_progress_ptr).as_ref() {
            Some(pb) => pb.clone(),
            None => ProgressBar::hidden(),
        }
    }
}
