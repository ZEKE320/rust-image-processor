import logging
from collections import Counter
from datetime import datetime
from pathlib import Path

from rich.console import Console
from rich.logging import RichHandler
from rich.table import Table
from typer import Typer

logging.basicConfig(
    level=logging.INFO,
    format="%(message)s",
    handlers=[
        RichHandler(rich_tracebacks=True),
    ],
)
logger = logging.getLogger("jpeg_counter")

app = Typer()
module_dir = Path(__file__).resolve().parent


@app.command()
def main(
    input_directory: Path = module_dir / "../../../data/受領画像_整理済み",
) -> None:
    """
    指定したディレクトリ内のJPEG画像を再帰的にカウントし、一意のファイル名の数を表示する
    サブディレクトリも含めて処理します

    Args:
        input_directory: 検索対象のディレクトリ
    """
    start_time = datetime.now()

    # 入力パスの解決
    input_path = Path(input_directory).resolve()
    logger.info(f"JPEG画像の解析を開始: {input_path}")

    if not input_path.exists():
        logger.error(f"指定されたディレクトリが存在しません: {input_path}")
        return

    # すべてのファイルを再帰的に取得し、JPEGのみをフィルタリング（サブディレクトリも検索）
    all_files = input_path.rglob("*")  # rglob は再帰的に全ファイルを検索
    jpeg_files = [
        f for f in all_files if f.is_file() and f.suffix.lower() in (".jpg", ".jpeg")
    ]

    # Counter を直接活用してファイル名を集計
    filename_counter = Counter(f.stem for f in jpeg_files)

    # パスとファイル名のマッピングを作成（重複表示用）
    filename_paths = {}
    for file_path in jpeg_files:
        stem = file_path.stem
        if stem not in filename_paths:
            filename_paths[stem] = []
        filename_paths[stem].append(file_path)

    # 処理時間の計測
    processing_time = (datetime.now() - start_time).total_seconds()

    # 結果の表示
    logger.info("\nJPEG画像カウント結果")
    logger.info(f"解析ディレクトリ: {input_path}")
    logger.info(f"総JPEG画像ファイル数: {len(jpeg_files)}")
    logger.info(f"一意のファイル名数: {len(filename_counter)}")
    logger.info(
        f"重複ファイル名数: {sum(1 for c in filename_counter.values() if c > 1)}"
    )
    logger.info(f"処理時間: {processing_time:.2f}秒")

    # 重複ファイル名の詳細表示（most_common を活用）
    duplicates = [
        (name, count) for name, count in filename_counter.most_common() if count > 1
    ]

    if duplicates:
        logger.info("\n重複ファイル名一覧")

        # テーブルを作成
        table = Table(show_header=True, header_style="bold magenta", expand=True)
        table.add_column("ファイル名", style="dim")
        table.add_column("出現回数", justify="right")
        table.add_column("ファイルパス (すべて)", style="dim", no_wrap=False)

        # most_common はすでに降順ソート済み
        for name, count in duplicates:
            # 全てのパスをテーブルに表示
            all_paths = "\n".join([str(p) for p in filename_paths[name]])
            table.add_row(name, str(count), all_paths)

        # コンソールを作成してテーブルをレンダリング
        console = Console(width=150)  # 幅を広げて省略を防止
        console.print(table)

        # テーブル内容をログに記録（詳細表示）
        logger.info("重複ファイルの詳細パス:")
        for name, count in duplicates:
            logger.info(f"- {name} (出現回数: {count}):")
            for path in filename_paths[name]:
                logger.info(f"  - {path}")

        logger.info(f"重複ファイル名の詳細: {len(duplicates)}件")

    logger.info("JPEG画像カウント処理が完了しました")


if __name__ == "__main__":
    app()
