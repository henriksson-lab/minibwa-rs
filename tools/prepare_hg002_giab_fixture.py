#!/usr/bin/env python3
"""Prepare a larger paired HG002 GIAB FASTQ fixture.

The source index has paired FASTQ URLs and MD5s. This script downloads missing
members, reuses any matching files already staged in a cache directory, counts
records, and writes concatenated gzip streams for R1/R2. Concatenated gzip is
intentional: minibwa-rs uses MultiGzDecoder and original minibwa uses zlib's
gzread, both of which consume multiple gzip members.
"""

from __future__ import annotations

import argparse
import csv
import gzip
import hashlib
import os
import shutil
import subprocess
import sys
from pathlib import Path
from urllib.parse import urlparse


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--index", required=True, type=Path)
    parser.add_argument("--cache-dir", required=True, type=Path)
    parser.add_argument("--reuse-dir", action="append", default=[], type=Path)
    parser.add_argument("--out-dir", required=True, type=Path)
    parser.add_argument("--target-reads", required=True, type=int)
    parser.add_argument("--name", default=None)
    parser.add_argument("--no-download", action="store_true")
    parser.add_argument("--skip-md5", action="store_true")
    return parser.parse_args()


def basename(url: str) -> str:
    return Path(urlparse(url).path).name


def md5_file(path: Path) -> str:
    h = hashlib.md5()
    with path.open("rb") as fh:
        for chunk in iter(lambda: fh.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def ensure_file(
    url: str,
    expected_md5: str,
    cache_dir: Path,
    reuse_dirs: list[Path],
    skip_md5: bool,
    no_download: bool,
) -> Path:
    name = basename(url)
    for path in [cache_dir / name, *[reuse_dir / name for reuse_dir in reuse_dirs]]:
        if not path.exists():
            continue
        if not skip_md5 and expected_md5 and md5_file(path) != expected_md5:
            raise SystemExit(f"MD5 mismatch for existing {path}")
        return path
    path = cache_dir / name
    if no_download:
        raise SystemExit(f"Missing {path} and --no-download was set")
    tmp = path.with_suffix(path.suffix + ".part")
    print(f"download {url}", flush=True)
    subprocess.run(["curl", "-L", "--fail", "--retry", "5", "-o", str(tmp), url], check=True)
    if not skip_md5 and expected_md5 and md5_file(tmp) != expected_md5:
        tmp.unlink(missing_ok=True)
        raise SystemExit(f"MD5 mismatch after download: {path}")
    tmp.replace(path)
    return path


def count_reads_gz(path: Path) -> int:
    lines = 0
    with gzip.open(path, "rb") as fh:
        for _ in fh:
            lines += 1
    if lines % 4 != 0:
        raise SystemExit(f"FASTQ line count is not divisible by 4: {path}: {lines}")
    return lines // 4


def concat_gzip(paths: list[Path], out: Path) -> None:
    tmp = out.with_suffix(out.suffix + ".part")
    with tmp.open("wb") as dst:
        for src in paths:
            with src.open("rb") as fh:
                shutil.copyfileobj(fh, dst, length=1024 * 1024)
    tmp.replace(out)


def symlink_force(src: Path, dst: Path) -> None:
    dst.unlink(missing_ok=True)
    os.symlink(src, dst)


def main() -> int:
    args = parse_args()
    args.cache_dir.mkdir(parents=True, exist_ok=True)
    args.out_dir.mkdir(parents=True, exist_ok=True)
    name = args.name or f"hg002_{args.target_reads // 1_000_000}m"

    selected: list[tuple[Path, Path, int]] = []
    total = 0
    with args.index.open(newline="") as fh:
        reader = csv.DictReader(fh, delimiter="\t")
        for row in reader:
            r1 = ensure_file(
                row["FASTQ"],
                row.get("FASTQ_MD5", ""),
                args.cache_dir,
                args.reuse_dir,
                args.skip_md5,
                args.no_download,
            )
            r2 = ensure_file(
                row["PAIRED_FASTQ"],
                row.get("PAIRED_FASTQ_MD5", ""),
                args.cache_dir,
                args.reuse_dir,
                args.skip_md5,
                args.no_download,
            )
            n1 = count_reads_gz(r1)
            n2 = count_reads_gz(r2)
            if n1 != n2:
                raise SystemExit(f"Pair count mismatch: {r1}={n1}, {r2}={n2}")
            selected.append((r1, r2, n1))
            total += n1
            print(f"selected {r1.name} / {r2.name}: {n1} reads, total {total}", flush=True)
            if total >= args.target_reads:
                break

    if total < args.target_reads:
        raise SystemExit(f"Only found {total} reads, wanted {args.target_reads}")

    r1_out = args.out_dir / f"{name}_R1.fastq.gz"
    r2_out = args.out_dir / f"{name}_R2.fastq.gz"
    concat_gzip([p[0] for p in selected], r1_out)
    concat_gzip([p[1] for p in selected], r2_out)

    manifest = args.out_dir / f"{name}.manifest.tsv"
    with manifest.open("w") as out:
        out.write("r1\tr2\treads\n")
        for r1, r2, reads in selected:
            out.write(f"{r1}\t{r2}\t{reads}\n")
        out.write(f"TOTAL\tTOTAL\t{total}\n")

    symlink_dir = Path(".tmp/large-real") / name
    symlink_dir.mkdir(parents=True, exist_ok=True)
    symlink_force(r1_out, symlink_dir / r1_out.name)
    symlink_force(r2_out, symlink_dir / r2_out.name)
    symlink_force(manifest, symlink_dir / manifest.name)
    print(f"prepared {r1_out}")
    print(f"prepared {r2_out}")
    print(f"reads_per_mate {total}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
