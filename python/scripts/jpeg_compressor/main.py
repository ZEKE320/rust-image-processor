import logging
from datetime import datetime
from pathlib import Path

from PIL import Image
from rich.logging import RichHandler
from rich.progress import track
from typer import Typer

app = Typer()
module_dir = Path(__file__).resolve().parent

datetime_str = datetime.now().strftime("%Y%m%d_%H%M%S")

logging.basicConfig(
    level=logging.INFO,
    format="%(message)s",
    handlers=[
        RichHandler(rich_tracebacks=True),
    ],
)
logger = logging.getLogger("jpeg_compressor")


def compress_jpeg(input_path: Path, output_path: Path, quality: int):
    """
    フォルダ内のJPEG画像を圧縮し、元のフォルダ構成を維持しながら出力する

    Args:
        input_dir (str): 入力ディレクトリのパス
        output_dir (str): 出力ディレクトリのパス
        quality (int): 圧縮の品質（0-100）、80は元の80%の品質を意味する
    """

    # 出力ディレクトリが存在しない場合は作成
    output_path.mkdir(parents=True, exist_ok=True)

    target_path = tuple(input_path.glob("**/*"))

    # 再帰的にすべてのファイルを処理
    for file_path in track(target_path):
        # ディレクトリはスキップ
        if file_path.is_dir():
            logger.info(f"ディレクトリはスキップ: {file_path}")
            continue

        # 入力パスからの相対パスを計算
        relative_path = file_path.relative_to(input_path)
        # 出力先のファイルパスを作成
        output_file_path = output_path / relative_path
        # 出力先ディレクトリを確保
        output_file_path.parent.mkdir(parents=True, exist_ok=True)

        # JPEGファイルのみ処理
        if file_path.suffix.lower() not in (".jpg", ".jpeg"):
            logger.warning(f"JPEGファイルではありません: {file_path}")
            continue

        try:
            # 圧縮前のファイルサイズを取得
            original_size = file_path.stat().st_size

            with Image.open(file_path) as img:
                # 最適化フラグを追加してJPEGとして保存（圧縮）
                img.save(output_file_path, "JPEG", quality=quality, optimize=True)

                # 圧縮後のファイルサイズを取得
                compressed_size = output_file_path.stat().st_size
                ratio = compressed_size / original_size * 100

                logger.info(
                    f"圧縮: {file_path} -> {output_file_path} "
                    f"({original_size:,} bytes -> {compressed_size:,} bytes, "
                    f"{ratio:.1f}% of original)"
                )
        except Exception as e:
            logger.error(f"エラー処理 {file_path}: {e}", exc_info=True)


@app.command()
def main(
    input_directory: Path = module_dir / "../../../data/受領画像_整理済み",
    quality: int = 90,
) -> None:
    # 文字列が渡された場合に Path オブジェクトに変換
    input_path = Path(input_directory).resolve()

    # 入力パスの存在確認
    if not input_path.exists():
        logger.error(f"入力パスが存在しません: {input_path}")
        return

    # 出力パスの作成
    input_dirname = input_path.name
    output_path = (
        module_dir
        / "output"
        / "compressed"
        / f"{datetime_str}_{input_dirname}_{quality}"
    )

    # 出力パスの存在確認
    if output_path.exists():
        logger.warning(
            f"出力ディレクトリが既に存在します。確認してください: {output_path}"
        )
        return

    logger.info(f"入力ディレクトリ: {input_path}")
    logger.info(f"出力ディレクトリ: {output_path}")
    logger.info(f"圧縮品質: {quality}")

    if input("圧縮を実行しますか？ (y/n): ").lower() != "y":
        logger.info("処理がキャンセルされました。")
        return

    compress_jpeg(input_path, output_path, quality)
    logger.info("圧縮処理が完了しました。")


if __name__ == "__main__":
    app()
