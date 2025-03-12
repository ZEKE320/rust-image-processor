/// JPEG圧縮ユーティリティのライブラリクレート
///
/// 指定されたディレクトリ内のJPEG画像を並列に圧縮し、
/// ディレクトリ構造を維持しながら出力します。
pub mod cli;
pub mod compressor;
pub mod logger;
pub mod util;

pub use cli::Cli;
pub use compressor::compress_jpeg_directory;
