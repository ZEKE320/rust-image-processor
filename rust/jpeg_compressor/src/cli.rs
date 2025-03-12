use clap::{Parser, ValueEnum};
use std::fmt;

/// コマンドライン引数の解析のためのデータ構造
#[derive(Parser, Debug)]
#[command(author, version, about = "JPEGファイルを圧縮するユーティリティ", long_about = None)]
pub struct Cli {
    /// 入力ディレクトリのパス
    #[arg(
        short,
        long,
        default_value = "./data/input_images",
        help = "圧縮するJPEGファイルが格納されているディレクトリのパスを指定します。"
    )]
    pub input_dir: String,

    /// 出力ディレクトリのパス（指定しない場合は自動生成）
    #[arg(
        short,
        long,
        help = "出力ディレクトリのパスを指定します。指定しない場合は自動生成されます。"
    )]
    pub output_dir: Option<String>,

    /// 圧縮の品質（1-100、高いほど高品質）
    #[arg(short, long, default_value = "90", value_parser = quality_validator, help="圧縮品質を指定します。1から100の範囲で指定してください。デフォルトは90です。")]
    pub quality: u8,

    /// 処理を自動実行（プロンプトなし）
    #[arg(
        short,
        long,
        default_value_t = false,
        help = "処理を自動実行します。プロンプトなしで既存のファイルを上書きします。"
    )]
    pub yes: bool,

    /// 並列処理のスレッド数（0=自動、システムの論理プロセッサ数に基づいて自動設定、デフォルトは0）
    #[arg(short, long, default_value_t = 0, value_parser = threads_validator, help = "使用するスレッド数を指定します。0の場合は自動で設定されます（デフォルトは0）。")]
    pub threads: usize,

    /// ログレベル（error=エラーのみ, warn=警告, info=情報, debug=デバッグ, trace=詳細デバッグ）
    #[arg(short, long, value_enum, default_value_t = LogLevel::Info, help = "ログレベルを指定します。error=エラーのみ, warn=警告, info=情報, debug=デバッグ, trace=詳細デバッグ（デフォルトはinfo）")]
    pub log_level: LogLevel,

    /// エンコーダーの種類（mozjpeg=高品質・高圧縮率, image=imageクレートのJpegEncoder）
    #[arg(short = 'e', long, value_enum, default_value_t = EncoderType::Mozjpeg, help = "エンコーダーの種類を指定します。Mozjpeg（デフォルト）は高品質・高圧縮率、ImageはimageクレートのJpegEncoderを使用します。")]
    pub encoder: EncoderType,
}

/// ログレベルの列挙型
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

/// エンコーダーの種類の列挙型
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum EncoderType {
    /// mozjpegエンコーダー（高品質・高圧縮率）
    Mozjpeg,
    /// imageクレートのJpegEncoder
    Image,
}

impl fmt::Display for EncoderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EncoderType::Mozjpeg => write!(f, "mozjpeg"),
            EncoderType::Image => write!(f, "image"),
        }
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Error => write!(f, "error"),
            LogLevel::Warn => write!(f, "warn"),
            LogLevel::Info => write!(f, "info"),
            LogLevel::Debug => write!(f, "debug"),
            LogLevel::Trace => write!(f, "trace"),
        }
    }
}

/// 品質パラメータのバリデーション（1-100の範囲内であることを確認）
fn quality_validator(s: &str) -> Result<u8, String> {
    s.parse::<u8>()
        .map_err(|_| format!("`{}` は有効な数値ではありません", s))
        .and_then(|quality| {
            if (1..=100).contains(&quality) {
                Ok(quality)
            } else {
                Err("品質は1から100の間である必要があります".to_string())
            }
        })
}

/// スレッド数パラメータのバリデーション
fn threads_validator(s: &str) -> Result<usize, String> {
    s.parse::<usize>()
        .map_err(|_| format!("`{}` は有効な数値ではありません", s))
        .and_then(|threads| {
            if threads <= 256 {
                Ok(threads)
            } else {
                Err("スレッド数は0（自動）から256までの間である必要があります".to_string())
            }
        })
}
