# JPEG 圧縮ユーティリティ

このツールは、ディレクトリ内の JPEG 画像を高速に圧縮し、ディレクトリ構造を維持しながら出力するユーティリティです。Python の`jpeg_compressor`を Rust で再実装したもので、パフォーマンスと安全性が向上しています。

## 主な機能

- マルチスレッド並列処理による高速圧縮
- 進捗状況のリアルタイム表示（プログレスバー）
- 品質レベル指定（1-100）
- mozjpegとimage（標準）の両方のエンコーダーをサポート
- ディレクトリ構造の保持
- 詳細な統計情報の出力
- 圧縮前後のファイルサイズ比較
- 色付きログ出力

## 使用方法

```bash
# 基本的な使用法
cargo run --release -- --input-dir="../data/受領画像_整理済み" --quality=85

# mozjpegエンコーダーを指定する場合
cargo run --release -- --input-dir="../data/受領画像_整理済み" --quality=85 --encoder=mozjpeg

# imageエンコーダーを強制的に使用する場合
cargo run --release -- --input-dir="../data/受領画像_整理済み" --quality=85 --encoder=image

# ヘルプを表示
cargo run --release -- --help
```

### コマンドライン引数

```
Usage: jpeg_compressor [OPTIONS]

Options:
  -i, --input-dir <INPUT_DIR>    入力ディレクトリのパス [default: ../data/受領画像_整理済み]
  -o, --output-dir <OUTPUT_DIR>  出力ディレクトリのパス
  -q, --quality <QUALITY>        圧縮の品質（1-100） [default: 90]
  -y, --yes                      処理を自動実行（プロンプトなし）
  -t, --threads <THREADS>        並列処理のスレッド数（0=自動） [default: 0]
  -e, --encoder <ENCODER>        エンコーダーの種類 [default: image] [possible values: mozjpeg, image]
  -l, --log-level <LOG_LEVEL>    ログレベル [default: info] [possible values: error, warn, info, debug, trace]
  -h, --help                     Print help
  -V, --version                  Print version
```

## 圧縮品質について

- `100`: 無損失圧縮（最高品質、ファイルサイズは大きい）
- `90`: 高品質（視覚的な違いはほとんどない、適度な圧縮）
- `80-85`: 推奨値（良好な品質と圧縮率のバランス）
- `70-75`: 標準的な Web 用（ファイルサイズ優先）
- `60以下`: 低品質（明らかな画質劣化、小さなファイルサイズ）

## エンコーダーについて

- `mozjpeg`: mozjpegエンコーダーを使用（高品質・高圧縮率、DCT最適化）
- `image`: Rustのimageクレートのエンコーダーを使用（標準的な品質）

mozjpegエンコーダーを使用するには、ビルド時にmozjpegライブラリが必要です。

## ビルド方法

```bash
# リリースビルド（推奨）
cargo build --release

# mozjpegサポートなしでビルド
cargo build --release --no-default-features --features="image-encoder"
```

## 推奨環境

- Rust 1.70.0 以上
- マルチコア CPU（並列処理の恩恵を受けるため）
- mozjpegをサポートするには:
  - Linux: build-essential, cmake, nasm
  - macOS: brew install cmake nasm
  - Windows: CMake, Visual Studio Build Tools, NASM

## シーケンス図

```mermaid
sequenceDiagram
    autonumber
    actor User as ユーザー
    participant Main as メインプロセス
    participant CLI as 引数解析
    participant Logger as ロガー
    participant Compressor as 圧縮処理
    participant FileSystem as ファイルシステム
    participant Threads as 並列処理スレッド

    %% 初期化フェーズ
    User->>Main: プログラム起動 (cargo run)
    Note over Main,CLI: 初期化フェーズ
    Main->>CLI: 引数解析の要求
    CLI->>CLI: 引数値のバリデーション
    CLI-->>Main: 解析済み引数 (Cliオブジェクト)

    %% ロガー初期化
    Main->>Logger: ロガー初期化
    Logger->>Logger: ログレベル設定 (error/warn/info/debug/trace)
    Logger->>Logger: カラースキーム設定
    Logger-->>Main: ロガー初期化完了

    %% パス検証と設定
    Main->>FileSystem: 入力ディレクトリの検証
    FileSystem-->>Main: パス情報
    Main->>Main: 出力ディレクトリの設定
    Note right of Main: 日時を含む出力フォルダの自動生成

    %% 実行情報の表示
    Main->>Logger: 実行設定の表示要求
    Logger-->>User: 設定情報の表示 (ディレクトリ、品質、スレッド数)

    %% ユーザー確認
    alt 出力ディレクトリが存在 && 自動実行でない
        Main->>User: 上書き確認プロンプト
        User-->>Main: 応答 (y/n)
    end

    alt 自動実行でない
        Main->>User: 実行確認プロンプト
        User-->>Main: 応答 (y/n)

        alt 応答 == "n"
            Main-->>User: 処理中止
            Note over Main: プログラム終了
        end
    end

    %% 圧縮処理開始
    Main->>Logger: 処理開始ログ
    Main->>Compressor: compress_directory()呼び出し
    Note over Compressor: 圧縮処理フェーズ開始

    %% ファイルスキャン
    Compressor->>FileSystem: JPEGファイルのスキャン開始
    FileSystem-->>Compressor: 検出ファイル一覧
    Compressor->>Compressor: ディレクトリ集計
    Compressor->>Logger: ファイル検出結果ログ

    %% プログレスバー初期化
    Compressor->>Compressor: プログレスバー初期化
    Compressor->>Threads: 進捗状況監視スレッド作成

    %% 並列処理の開始
    Compressor->>Threads: スレッドプール構成 (CPUコア数に基づく)

    par 並列圧縮処理
        loop 各ファイルの並列処理
            Threads->>FileSystem: JPEGファイルの読み込み
            FileSystem-->>Threads: バイナリデータ
            Threads->>Threads: 画像のデコード
            Threads->>FileSystem: 圧縮された画像の保存
            Threads->>Compressor: 処理結果・統計情報の更新
            Compressor->>Threads: 進捗表示の更新
        end
    end

    %% 進捗表示
    loop 定期的な更新
        Threads->>User: プログレスバーの更新表示
    end

    %% 完了とクリーンアップ
    Threads-->>Compressor: すべてのファイル処理完了
    Compressor->>Compressor: プログレスバー完了状態に変更
    Compressor->>Threads: 進捗状況監視スレッド終了
    Compressor-->>Main: 圧縮統計情報の返却

    %% 結果表示
    Main->>Logger: 圧縮結果のログ出力要求
    Logger-->>User: 処理結果の詳細表示 (時間、ファイル数、圧縮率)
    Main-->>User: 処理終了
```
