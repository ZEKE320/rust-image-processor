# JPEG画像処理ユーティリティ（外部ツール対応版）

このツールは、ディレクトリ内のJPEG画像を高速に処理し、最適化された方法で保存します。外部ツールと連携することで、より広範なファイル形式と処理オプションをサポートします。

## 主な機能

- マルチスレッド並列処理
- 複数の画像処理エンジンサポート:
  - OpenCV
  - ImageMagick
  - libjpeg-progs
- 自動フォールバック処理（最適なツールを自動選択）
- ディレクトリ構造の保持
- カラフルなログ出力
- 効率的なJPEG変換

## 必要条件

- Rust 1.70.0以上
- Cargo
- 推奨：
  - OpenCV 4.x（より高速かつ高品質な処理）
  - または ImageMagick
  - または libjpeg-progs

## インストール

```bash
# 依存ライブラリをインストール（オプション・推奨）
# Debianベースのシステム
sudo apt install libopencv-dev # OpenCV
# または
sudo apt install imagemagick libjpeg-progs

# Red Hat/CentOSベースのシステム
sudo yum install opencv-devel # OpenCV
# または
sudo yum install ImageMagick libjpeg-turbo-utils

# macOS
brew install opencv # OpenCV
# または
brew install imagemagick jpeg

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

## 処理フロー

1. システム上の利用可能な画像処理ツールを検出
2. 最適なツールを自動選択（OpenCV > ImageMagick > libjpeg-progs > Rust内蔵機能）
3. 画像を並列処理
4. 結果を出力

## トラブルシューティング

頻繁に発生するデコードエラーは、次のような原因が考えられます:

1. 実際にはHEIF/HEICフォーマットなのにJPEG拡張子が付いている
2. 画像ファイルが破損している
3. メタデータが特殊または大きすぎる

OpenCVやImageMagickを使用すると、これらの問題の多くが解決します。特にOpenCVは様々な形式のファイルに対応可能です。
