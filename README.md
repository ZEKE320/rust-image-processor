# JPEG画像処理ユーティリティ

ディレクトリ内のJPEG画像を高品質で処理するユーティリティです。ディレクトリ構造を維持しながら、任意の品質でJPEG形式に変換・保存します。

## 特徴

- マルチスレッド並列処理による高速な画像変換
- 画像処理エンジンの自動選択:
  - ImageMagick (利用可能な場合)
  - Rust内蔵の`image`クレート (フォールバック)
- ディレクトリ構造の維持
- カラフルなログ出力とプログレスバー表示
- 最高品質（品質100%）でのJPEG保存

## 必要条件

- Rust 1.70.0以上
- Cargo
- 推奨:
  - ImageMagick (`convert`コマンドが利用可能な場合、処理が高速化)

## インストール

```bash
# リポジトリをクローン
git clone <repository-url>
cd sugi-img

# ImageMagick（推奨、オプション）
# Debianベースのシステム
sudo apt install imagemagick

# Red Hat/CentOSベースのシステム
sudo yum install ImageMagick

# macOS
brew install imagemagick

# プログラムをビルド
cd rust_jpeg_processor
cargo build --release
```

## 実行方法

```bash
# デフォルト設定で実行
cargo run --release

# 特定のディレクトリを指定して実行
cargo run --release -- --input-dir="../data/受領画像" --output-dir="output/fixed_jpeg"
```

## コマンドラインオプション

```sh
USAGE:
    sugi-img [OPTIONS]

OPTIONS:
    --input-dir <INPUT_DIR>    入力ディレクトリパス [default: data/受領画像]
    --output-dir <OUTPUT_DIR>  出力ディレクトリパス [default: output/fixed_jpeg]
    -h, --help                 ヘルプメッセージを表示
    -V, --version              バージョン情報を表示
```

## 注意事項

- **HEIF 形式サポート**:

  - Python 版: `pillow-heif`パッケージにより HEIF 形式をサポート
  - Rust 版: 標準では制限されたサポート。完全な HEIF 対応には追加設定が必要

- **ログファイル**:
  - 処理のログは `logs/fix_jpeg/日時/` ディレクトリに保存されます
  - コンソールにも出力されます

## ライセンス

[ライセンス情報を記載]
