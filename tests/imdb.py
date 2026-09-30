#!/usr/bin/env -S uv run --script
# /// script
# dependencies = ["polars", "tqdm"]
# ///

from pathlib import Path
from tqdm import tqdm
import logging
import polars as pl

logging.basicConfig(level=logging.DEBUG)
logger = logging.getLogger(__name__)

out_dir = Path.cwd() / "imdb"
out_dir.mkdir(exist_ok=True)

splits = {
    "train": "plain_text/train-00000-of-00001.parquet",
    "test": "plain_text/test-00000-of-00001.parquet",
    "unsupervised": "plain_text/unsupervised-00000-of-00001.parquet",
}
file_nr = 0
with tqdm(unit="file", unit_scale=True, total=1e12, leave=True) as progress:
    progress.total = 0
    for split in splits.values():
        df = pl.scan_parquet("hf://datasets/stanfordnlp/imdb/" + split).select("text")

        nr_rows = df.select(pl.len()).collect().item()
        logger.debug("split %s has %s rows", split, nr_rows)
        progress.total += nr_rows

        for batch in df.collect_batches():
            for row in batch.iter_rows():
                out_path = out_dir / f"{file_nr}.txt"
                with out_path.open("w") as out_file:
                    out_file.write(row[0])
                progress.update(1)
                file_nr += 1
