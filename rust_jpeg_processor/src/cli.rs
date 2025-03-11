use clap::Parser;
use std::path::PathBuf;

/// JPEG画像処理ユーティリティ
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// 入力ディレクトリパス
    #[arg(long, default_value = "../data/受領画像")]
    pub input_dir: PathBuf,

    /// 出力ディレクトリパス
    #[arg(long, default_value = "output/fixed_jpeg")]
    pub output_dir: PathBuf,
}

/// コマンドライン引数をパースする
pub fn parse_args() -> Args {
    Args::parse()
}
