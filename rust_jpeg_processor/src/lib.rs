//! JPEG画像処理ライブラリ
//!
//! このライブラリは画像ファイルをJPEG形式で最適化して保存する機能を提供します。

pub mod cli;
pub mod logger;
pub mod processor;

// 主要な型やユーティリティを再エクスポート
pub use crate::processor::process_images;
