use crate::logger;
use anyhow::{Context, Result};
use log::{info, warn};
use rayon::prelude::*;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use walkdir::WalkDir;

/// 外部コマンドを使ったJPEGファイル処理
pub fn process_images(input_dir: &Path, output_dir: &Path) -> Result<()> {
    // 画像処理ツールの存在チェック
    check_external_tools()?;

    // 入力ディレクトリの存在確認
    if !input_dir.exists() {
        return Err(anyhow::anyhow!(
            "入力ディレクトリが見つかりません: {:?}",
            input_dir
        ));
    }

    // 出力ディレクトリを作成
    fs::create_dir_all(output_dir)?;

    // 処理対象のファイル一覧を取得
    let files: Vec<_> = collect_jpeg_files(input_dir);
    let total_files = files.len();

    info!("合計 {} ファイルを処理します", total_files);

    // プログレスバーを取得
    let progress_bar = logger::get_progress_bar();
    progress_bar.set_length(total_files as u64);
    progress_bar.set_message("画像処理中...");
    progress_bar.enable_steady_tick(Duration::from_millis(200));

    // 処理成功/失敗のカウンター
    let success_count = Arc::new(AtomicUsize::new(0));
    let error_count = Arc::new(AtomicUsize::new(0));

    // スレッド数を制限（コア数の1/2を使用）
    let num_threads = std::cmp::max(1, num_cpus::get() / 2);
    info!("{}個のスレッドで並列処理を実行します", num_threads);

    // スレッドプールの設定
    rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()
        .unwrap_or_else(|e| warn!("スレッドプール設定エラー: {}", e));

    // ファイルを処理
    files.par_iter().for_each(|file_path| {
        let relative_path = match file_path.strip_prefix(input_dir) {
            Ok(path) => path,
            Err(e) => {
                warn!("相対パス作成失敗: {} - {}", file_path.display(), e);
                progress_bar.inc(1);
                return;
            }
        };

        let output_file = output_dir.join(relative_path);

        // 出力先ディレクトリを作成
        if let Some(parent) = output_file.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                warn!("ディレクトリ作成失敗: {} - {}", parent.display(), e);
                progress_bar.inc(1);
                return;
            }
        }

        let start_time = Instant::now();
        match process_jpeg_file(file_path, &output_file) {
            Ok(_) => {
                let elapsed = start_time.elapsed();
                if elapsed.as_secs() > 5 {
                    warn!(
                        "処理に時間がかかりました: {} ({:.1}秒)",
                        file_path.file_name().unwrap_or_default().to_string_lossy(),
                        elapsed.as_secs_f32()
                    );
                }

                let count = success_count.fetch_add(1, Ordering::SeqCst) + 1;
                if count % 10 == 0 {
                    info!("成功: {}/{} ファイル処理済み", count, total_files);
                }
            }
            Err(e) => {
                error_count.fetch_add(1, Ordering::SeqCst);
                warn!(
                    "変換失敗: {} - {}",
                    file_path.file_name().unwrap_or_default().to_string_lossy(),
                    e
                );
            }
        }

        progress_bar.inc(1);
    });

    let success = success_count.load(Ordering::SeqCst);
    let errors = error_count.load(Ordering::SeqCst);

    progress_bar.finish_with_message(format!("処理完了：成功 {}, 失敗 {}", success, errors));
    info!("処理結果: 成功 {}, 失敗 {}", success, errors);

    Ok(())
}

/// JPEGファイルをリストアップする
fn collect_jpeg_files(dir: &Path) -> Vec<PathBuf> {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .filter(|p| {
            matches!(
                p.extension().and_then(|e| e.to_str()),
                Some("jpg" | "jpeg" | "JPG" | "JPEG")
            )
        })
        .collect()
}

/// 外部コマンドが利用可能かチェック
fn check_external_tools() -> Result<()> {
    // まずImageMagickを確認
    match Command::new("convert").arg("-version").output() {
        Ok(_) => {
            info!("ImageMagick が利用可能です");
            return Ok(());
        }
        Err(e) if e.kind() == ErrorKind::NotFound => {
            // ImageMagickがなければ次を試す
        }
        Err(e) => {
            return Err(anyhow::anyhow!("ImageMagickのチェックに失敗: {}", e));
        }
    }

    // 何も見つからなかった場合
    info!("外部画像処理ツールが見つかりません。Rust内蔵のimage crateを使用します");
    Ok(())
}

/// 画像変換処理を実行する（Python実装に合わせた処理）
fn process_jpeg_file(input_path: &Path, output_path: &Path) -> Result<()> {
    // ImageMagickが使えるかチェック（最も一般的）
    if Command::new("convert")
        .arg("-version")
        .stdout(Stdio::null())
        .status()
        .is_ok()
    {
        // ImageMagickで処理（無損失処理）
        let status = Command::new("convert")
            .arg(input_path)
            .arg("-quality")
            .arg("100") // 無損失でJPEGを出力 (py_jpeg_processorと同じ)
            .arg(output_path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .with_context(|| {
                format!("ImageMagickの実行に失敗しました: {}", input_path.display())
            })?;

        if status.success() {
            return Ok(());
        } else {
            return Err(anyhow::anyhow!(
                "ImageMagickがエラーコード {} で終了しました",
                status
            ));
        }
    }

    // どちらも使えない場合はRust内蔵機能を使用
    process_with_rust_image(input_path, output_path)
}

/// Rustのimage crateを使ってJPEG画像を処理
fn process_with_rust_image(input_path: &Path, output_path: &Path) -> Result<()> {
    use image::codecs::jpeg::JpegEncoder;
    use image::io::Reader as ImageReader;

    let img = ImageReader::open(input_path)
        .with_context(|| format!("ファイルを開けませんでした: {}", input_path.display()))?
        .with_guessed_format()
        .with_context(|| "フォーマット推測に失敗しました")?
        .decode()
        .with_context(|| format!("画像デコードに失敗しました: {}", input_path.display()))?;

    // 検出された形式を記録
    let format = match ImageReader::open(input_path)?.with_guessed_format()? {
        reader => match reader.format() {
            Some(fmt) => format!("{:?}", fmt),
            None => "不明".to_string(),
        },
    };
    info!(
        "ファイル: {}, 検出された形式: {}",
        input_path.display(),
        format
    );

    let output_file = fs::File::create(output_path).with_context(|| {
        format!(
            "出力ファイルを作成できませんでした: {}",
            output_path.display()
        )
    })?;

    // py_jpeg_processor と同様に最高品質設定を使用
    let mut encoder = JpegEncoder::new_with_quality(std::io::BufWriter::new(output_file), 100);
    encoder
        .encode_image(&img)
        .with_context(|| "JPEG エンコードに失敗しました")?;

    info!("JPEG として保存しました: {}", output_path.display());
    Ok(())
}
