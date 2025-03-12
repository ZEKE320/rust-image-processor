use std::fs;
use std::io::{BufWriter, Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use anyhow::{Context, Result};
use image::codecs::jpeg::JpegEncoder;
use indicatif::{ProgressBar, ProgressStyle};
use log::{error, info, warn};
use rayon::prelude::*;
use walkdir::WalkDir;

use crate::cli::EncoderType;
use mozjpeg::{ColorSpace, Compress};

use crate::util::formatter::str_formatter::format_bytes;
use crate::util::time::time_util::estimate_remaining_time;

/// 圧縮処理の統計情報
#[derive(Debug)]
pub struct CompressionStats {
    pub processed_files: usize,
    pub skipped_files: usize,
    pub error_files: usize,
    pub original_size: u64,
    pub compressed_size: u64,
    pub start_time: Instant,
}

impl Default for CompressionStats {
    fn default() -> Self {
        Self {
            processed_files: 0,
            skipped_files: 0,
            error_files: 0,
            original_size: 0,
            compressed_size: 0,
            start_time: Instant::now(),
        }
    }
}

impl CompressionStats {
    /// 新しい統計情報インスタンスを作成
    pub fn new() -> Self {
        Self::default()
    }

    /// 圧縮率（圧縮後のサイズ / 元のサイズ）を計算
    pub fn get_size_ratio(&self) -> f64 {
        if self.original_size == 0 {
            return 0.0;
        }
        self.compressed_size as f64 / self.original_size as f64
    }

    /// 圧縮率（%表示用）を計算
    pub fn get_compression_ratio(&self) -> f64 {
        1.0 - self.get_size_ratio()
    }

    /// 処理速度（ファイル/秒）を計算
    pub fn get_processing_speed(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.processed_files as f64 / elapsed
        } else {
            0.0
        }
    }

    /// 現在の処理状況のサマリーを取得
    pub fn get_summary(&self) -> String {
        let total_files = self.processed_files + self.error_files + self.skipped_files;
        let progress_pct = if total_files > 0 {
            (self.processed_files as f64 / total_files as f64) * 100.0
        } else {
            0.0
        };

        let elapsed = self.start_time.elapsed();
        let elapsed_secs = elapsed.as_secs();
        let elapsed_str = format!(
            "{}:{:02}:{:02}",
            elapsed_secs / 3600,
            (elapsed_secs % 3600) / 60,
            elapsed_secs % 60
        );

        format!(
            "処理状況: {} 完了, {} エラー ({:.1}%), 経過時間: {}, {:.1}ファイル/秒, 圧縮率: {:.1}%, 容量: {} → {}",
            self.processed_files,
            self.error_files,
            progress_pct,
            elapsed_str,
            self.get_processing_speed(),
            self.get_compression_ratio() * 100.0,
            format_bytes(self.original_size),
            format_bytes(self.compressed_size)
        )
    }
}

/// フォルダ内のJPEGファイルを圧縮する
pub fn compress_jpeg_directory(
    input_dir: &Path,
    output_dir: &Path,
    quality: u8,
    thread_count: usize,
    encoder_type: EncoderType,
) -> Result<CompressionStats> {
    info!("JPEGファイルをスキャンしています...");

    // 出力ディレクトリの作成
    fs::create_dir_all(output_dir).with_context(|| {
        format!(
            "出力ディレクトリの作成に失敗しました: {}",
            output_dir.display()
        )
    })?;

    // ファイルのパスを収集
    let mut files = Vec::new();
    for entry in WalkDir::new(input_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        // ディレクトリはスキップ（ディレクトリ構造は後で作成）
        if path.is_dir() {
            continue;
        }

        // JPEGファイルのみ処理対象にする
        if let Some(ext) = path.extension() {
            let ext_lower = ext.to_string_lossy().to_lowercase();
            if ext_lower == "jpg" || ext_lower == "jpeg" {
                files.push(path.to_path_buf());
            }
        }
    }

    // ファイルが見つからない場合
    if files.is_empty() {
        warn!("JPEG形式のファイルが見つかりませんでした");
        return Ok(CompressionStats::default());
    }

    let total_files = files.len();
    info!("合計 {} 個のJPEGファイルを検出しました", total_files);

    // ディレクトリごとのファイル数を集計
    let mut dir_counts = std::collections::HashMap::new();
    for file in &files {
        if let Some(parent) = file.parent() {
            let relative = parent.strip_prefix(input_dir).unwrap_or(parent);
            *dir_counts.entry(relative.to_path_buf()).or_insert(0) += 1;
        }
    }

    // ディレクトリ情報を表示（最大5つ）
    let mut dirs: Vec<_> = dir_counts.iter().collect();
    dirs.sort_by_key(|(_, count)| std::cmp::Reverse(**count));

    info!("処理対象ディレクトリ:");
    for (i, (dir, count)) in dirs.iter().take(5).enumerate() {
        info!(" - {}: {} ({} ファイル)", i + 1, dir.display(), count);
    }
    if dirs.len() > 5 {
        info!(" - その他 {} ディレクトリ", dirs.len() - 5);
    }

    // プログレスバーの設定
    {
        // プログレスバー表示を有効化
        let mut manager = crate::logger::PROGRESS_MANAGER.lock().unwrap();
        manager.enable();
    }

    let progress_style = ProgressStyle::default_bar()
        .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) {msg}")
        .unwrap()
        .progress_chars("█▓▒░ ");

    let progress_bar = ProgressBar::new(total_files as u64);
    progress_bar.set_style(progress_style);
    progress_bar.set_message("処理開始準備中...");

    // 統計情報の共有
    let stats = Arc::new(Mutex::new(CompressionStats::new()));
    let stats_clone = Arc::clone(&stats);
    let pb_clone = progress_bar.clone();

    // 定期的な更新のためのスレッド
    let update_thread = std::thread::spawn(move || {
        let mut last_count = 0;
        let mut last_update = Instant::now();

        while !pb_clone.is_finished() {
            std::thread::sleep(std::time::Duration::from_secs(1));

            let lock_result = stats_clone.try_lock();
            if let Ok(stats) = lock_result {
                let current_count = stats.processed_files;

                // 前回の更新から変化があった場合のみ表示を更新
                if current_count > last_count {
                    let elapsed = last_update.elapsed().as_secs_f64();
                    let speed = if elapsed > 0.0 {
                        (current_count - last_count) as f64 / elapsed
                    } else {
                        0.0
                    };

                    pb_clone.set_message(format!(
                        "{:.1}枚/秒 (圧縮率:{:.1}%, 残り約{:.0}分)",
                        speed,
                        stats.get_compression_ratio() * 100.0,
                        estimate_remaining_time(
                            stats.processed_files,
                            total_files,
                            stats.start_time.elapsed().as_secs_f64()
                        )
                    ));

                    last_count = current_count;
                    last_update = Instant::now();
                }

                pb_clone.set_position((stats.processed_files + stats.error_files) as u64);
            }
        }
    });

    info!(
        "圧縮処理を開始します（{}スレッド, 品質: {}, エンコーダー: {}）",
        thread_count, quality, encoder_type
    );

    // スレッドプールを設定してRayonの並列処理を実行
    rayon::ThreadPoolBuilder::new()
        .num_threads(thread_count)
        .build_global()?;

    // 並列処理で圧縮を実行
    files.par_iter().for_each(|file_path| {
        let relative_path = file_path.strip_prefix(input_dir).unwrap_or(file_path);
        let output_file = output_dir.join(relative_path);

        // 出力ディレクトリが存在しない場合は作成
        if let Some(parent) = output_file.parent() {
            if !parent.exists() {
                if let Err(e) = fs::create_dir_all(parent) {
                    error!(
                        "ディレクトリの作成に失敗しました {}: {}",
                        parent.display(),
                        e
                    );
                    let mut stats = stats.lock().unwrap();
                    stats.error_files += 1;
                    return;
                }
            }
        }

        // 圧縮を実行 - エンコーダータイプに基づいて関数を選択
        let start = Instant::now();
        let compression_result = match encoder_type {
            EncoderType::Mozjpeg => compress_jpeg_mozjpeg(file_path, &output_file, quality),
            EncoderType::Image => compress_jpeg_image(file_path, &output_file, quality),
        };

        // 圧縮結果の処理
        match compression_result {
            Ok((original_size, compressed_size)) => {
                let ratio = if original_size > 0 {
                    compressed_size as f64 / original_size as f64 * 100.0
                } else {
                    0.0
                };

                // サイズの大きな変化があった場合のみログを出力
                let size_change_pct = if original_size > 0 {
                    (1.0 - (compressed_size as f64 / original_size as f64)) * 100.0
                } else {
                    0.0
                };

                if size_change_pct > 70.0
                    || original_size > 10 * 1024 * 1024
                    || start.elapsed().as_secs() > 5
                {
                    info!(
                        "注目ファイル: {} ({} → {}, {:.1}%, {:.1}秒)",
                        relative_path.display(),
                        format_bytes(original_size),
                        format_bytes(compressed_size),
                        ratio,
                        start.elapsed().as_secs_f64()
                    );
                }

                // 統計情報を更新
                let mut stats = stats.lock().unwrap();
                stats.processed_files += 1;
                stats.original_size += original_size;
                stats.compressed_size += compressed_size;

                // 処理後にプログレスバーを更新
                progress_bar.inc(1);
            }
            Err(e) => {
                error!("圧縮エラー {}: {}", file_path.display(), e);
                let mut stats = stats.lock().unwrap();
                stats.error_files += 1;
                progress_bar.inc(1);
            }
        }
    });

    // プログレスバー終了処理
    progress_bar.finish_with_message("処理完了");
    let _ = update_thread.join();

    // プログレスバーを無効化
    {
        let mut manager = crate::logger::PROGRESS_MANAGER.lock().unwrap();
        manager.disable();
    }

    let final_stats = Arc::try_unwrap(stats).unwrap().into_inner().unwrap();

    // トータル統計情報のみログに出力
    info!(
        "処理サマリー: {}ファイル処理, {}エラー, 圧縮率: {:.1}%, {} → {} ({:.1}% 削減)",
        final_stats.processed_files,
        final_stats.error_files,
        final_stats.get_compression_ratio() * 100.0,
        format_bytes(final_stats.original_size),
        format_bytes(final_stats.compressed_size),
        final_stats.get_compression_ratio() * 100.0
    );

    Ok(final_stats)
}

/// imageクレートのJpegEncoderを使用した圧縮実装（既存の実装）
fn compress_jpeg_image(input_path: &Path, output_path: &Path, quality: u8) -> Result<(u64, u64)> {
    // 元のファイルサイズを取得
    let original_size = fs::metadata(input_path)
        .with_context(|| {
            format!(
                "ファイルのメタデータを取得できません: {}",
                input_path.display()
            )
        })?
        .len();

    // バイナリデータとして読み込む
    let mut input_file = fs::File::open(input_path)
        .with_context(|| format!("入力ファイルを開けませんでした: {}", input_path.display()))?;

    // 画像全体をメモリに読み込む
    let mut buffer = Vec::new();
    input_file
        .read_to_end(&mut buffer)
        .with_context(|| format!("ファイルの読み込みに失敗しました: {}", input_path.display()))?;

    // 画像をデコード
    let img = image::load_from_memory(&buffer)
        .with_context(|| format!("画像データの解析に失敗しました: {}", input_path.display()))?;

    // バッファ付きの書き込み
    let output_file = fs::File::create(output_path).with_context(|| {
        format!(
            "出力ファイルを作成できませんでした: {}",
            output_path.display()
        )
    })?;
    let buffered_output = BufWriter::new(output_file);

    // JpegEncoderを直接使用して品質パラメータを適用
    let mut encoder = JpegEncoder::new_with_quality(buffered_output, quality);
    encoder
        .encode_image(&img)
        .with_context(|| format!("画像のエンコードに失敗しました: {}", input_path.display()))?;

    // 圧縮後のファイルサイズを取得
    let compressed_size = fs::metadata(output_path)
        .with_context(|| {
            format!(
                "圧縮ファイルのメタデータを取得できません: {}",
                output_path.display()
            )
        })?
        .len();

    Ok((original_size, compressed_size))
}

/// mozjpegを使用した高品質圧縮実装
#[cfg(feature = "mozjpeg-encoder")]
fn compress_jpeg_mozjpeg(input_path: &Path, output_path: &Path, quality: u8) -> Result<(u64, u64)> {
    // 元のファイルサイズを取得
    let original_size = fs::metadata(input_path)
        .with_context(|| {
            format!(
                "ファイルのメタデータを取得できません: {}",
                input_path.display()
            )
        })?
        .len();

    // 画像を読み込む
    let img = image::open(input_path)
        .with_context(|| format!("画像ファイルを開けませんでした: {}", input_path.display()))?;

    // RGBに変換
    let rgb_img = img.to_rgb8();
    let width = rgb_img.width() as usize;
    let height = rgb_img.height() as usize;
    let pixels = rgb_img.into_raw();

    // 出力ファイルを準備
    let output_file = fs::File::create(output_path).with_context(|| {
        format!(
            "出力ファイルを作成できませんでした: {}",
            output_path.display()
        )
    })?;
    let mut buffered_output = BufWriter::new(output_file);

    // mozjpeg圧縮設定 - 実際のAPI（0.10.13）に合わせた実装
    let mut comp = Compress::new(ColorSpace::JCS_RGB);
    comp.set_size(width, height);
    comp.set_quality(quality as f32);
    comp.set_optimize_coding(true);

    // 正しくAPIを使用する
    let mut comp_started = comp
        .start_compress(&mut buffered_output)
        .with_context(|| format!("mozjpegの圧縮開始に失敗しました: {}", input_path.display()))?;
    // ピクセルデータを書き込む
    comp_started.write_scanlines(&pixels).with_context(|| {
        format!(
            "画像データの書き込みに失敗しました: {}",
            input_path.display()
        )
    })?;
    // 終了処理（非推奨のfinish_compressではなくfinishを使用）
    comp_started
        .finish()
        .with_context(|| format!("mozjpegの圧縮完了に失敗しました: {}", input_path.display()))?;

    // バッファをフラッシュ
    buffered_output
        .flush()
        .with_context(|| "出力バッファのフラッシュに失敗しました")?;

    // 圧縮後のファイルサイズを取得
    let compressed_size = fs::metadata(output_path)
        .with_context(|| {
            format!(
                "圧縮ファイルのメタデータを取得できません: {}",
                output_path.display()
            )
        })?
        .len();

    Ok((original_size, compressed_size))
}

/// エンコーダーが利用できない場合の代替実装
#[cfg(not(feature = "mozjpeg-encoder"))]
fn compress_jpeg_mozjpeg(input_path: &Path, output_path: &Path, quality: u8) -> Result<(u64, u64)> {
    // mozjpegが利用できない場合は標準のエンコーダーを使用
    warn!("mozjpegエンコーダーが利用できません。標準のimageエンコーダーを使用します。");
    info!(
        "fallback: {} を圧縮します (品質: {})",
        input_path.display(),
        quality
    );
    compress_jpeg_image(input_path, output_path, quality)
}
