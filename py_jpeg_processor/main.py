import logging
from datetime import datetime
from logging.handlers import RotatingFileHandler
from pathlib import Path

from PIL import Image, UnidentifiedImageError
from pillow_heif import register_heif_opener
from rich.logging import RichHandler
from rich.progress import track
from typer import Typer

register_heif_opener()

app = Typer()

project_dir = Path(__file__).parent.resolve()


# 入力・出力フォルダを定数として定義
INPUT_DIR = (
    project_dir / ".." / "data" / "受領画像"
)  # 実際の入力フォルダパスに変更してください
OUTPUT_DIR = (
    project_dir / "output" / "fixed_jpeg"
)  # 実際の出力フォルダパスに変更してください

current_datetime = datetime.now()
logging_dir = (
    project_dir / f"logs/fix_jpeg/{current_datetime.strftime('%Y-%m-%d_%H-%M-%S')}"
)
logging_dir.mkdir(exist_ok=True, parents=True)

file_handler = RotatingFileHandler(
    logging_dir / "fix_jpeg.log",
    maxBytes=5 * 1024 * 1024,  # 5MB
    backupCount=3,
)
file_handler.setFormatter(
    logging.Formatter(
        "%(asctime)s - %(levelname)s - %(message)s - %(filename)s:%(lineno)d",
        datefmt="[%X]",
    )
)

logging.basicConfig(
    level=logging.INFO,
    format="%(message)s",
    datefmt="[%X]",
    handlers=[RichHandler(rich_tracebacks=True), file_handler],
)

logger = logging.getLogger("fix_jpeg")


def process_jpeg_files():
    """
    定義された入力ディレクトリを再帰的に探索し、JPEGファイルを無劣化で出力ディレクトリに保存します。
    ディレクトリ構造は維持されます。HEIFフォーマットが.jpegとして保存されている場合も
    JPEGとして出力します。
    """
    input_path = Path(INPUT_DIR)
    output_path = Path(OUTPUT_DIR)

    # 入力ディレクトリが存在することを確認
    if not input_path.exists():
        raise FileNotFoundError(f"入力ディレクトリが見つかりません: {INPUT_DIR}")

    # 出力ディレクトリを作成（もし存在しなければ）
    output_path.mkdir(exist_ok=True, parents=True)

    # 入力ディレクトリを再帰的に探索
    filepaths = tuple(input_path.glob("**/*"))
    for file_path in track(filepaths, description="画像を処理中..."):
        # 相対パスを取得（入力ディレクトリからの相対パス）
        relative_path = file_path.relative_to(input_path)
        output_file = output_path / relative_path

        # ディレクトリの場合、出力先に同じディレクトリ構造を作成
        if file_path.is_dir():
            output_file.mkdir(exist_ok=True, parents=True)
            continue

        # JPEGファイルかどうかを確認
        if file_path.suffix.lower() not in [".jpg", ".jpeg"]:
            logging.info(f"スキップ: {file_path} は JPEG 形式ではありません")
            continue

        # 出力先のディレクトリがまだ存在しない場合は作成
        output_file.parent.mkdir(exist_ok=True, parents=True)

        try:
            # 画像を開く
            img = Image.open(file_path)
            actual_format = img.format

            logging.info(f"ファイル: {file_path}, 検出された形式: {actual_format}")

            # どのような形式でもJPEGとして保存
            img.save(output_file, format="JPEG", quality=100, subsampling=0)
            logging.info(f"JPEG として保存しました: {output_file}")

        except UnidentifiedImageError as e:
            # 画像として開けない場合はエラーを報告
            logging.error(f"エラー: {file_path} は画像として認識できませんでした")
            logging.error(e, exc_info=True)

        except Exception as e:
            logging.error(f"エラー: {file_path} の処理中にエラーが発生しました - {e}")
            logging.error(e, exc_info=True)


@app.command()
def main():
    # PIL が HEIF 形式をサポートしているか確認するメッセージ
    logging.info(
        "注意: HEIFフォーマットを処理するには 'pillow-heif' パッケージが必要な場合があります。"
    )
    logging.info("インストールするには: pip install pillow-heif")
    logging.info("処理を開始します...")

    process_jpeg_files()
    logging.info("処理が完了しました。")


if __name__ == "__main__":
    app()
