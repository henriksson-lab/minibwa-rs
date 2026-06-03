#!/usr/bin/env python3
"""Benchmark minibwa-rs against original C minibwa on Zenodo record 20097931."""

from __future__ import annotations

import argparse
import csv
import filecmp
import hashlib
import json
import os
import re
import subprocess
import sys
from dataclasses import asdict, dataclass
from pathlib import Path


DEFAULT_DATA_DIR = Path(".tmp/zenodo-20097931")
DEFAULT_OUT_DIR = Path(".tmp/zenodo-20097931-bench")
DEFAULT_REF = Path(".tmp/large-real/human_grch38/ref.fa.gz")


@dataclass
class Result:
    step: str
    dataset: str
    impl: str
    index_algorithm: str
    status: int
    wall_seconds: float | None
    user_seconds: float | None
    system_seconds: float | None
    max_rss_kb: int | None
    stdout: str
    stderr: str
    time_file: str


@dataclass
class Parity:
    step: str
    dataset: str
    left: str
    right: str
    equal: bool
    left_sha256: str | None
    right_sha256: str | None


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Build C/Rust minibwa, index a reference, map every read dataset from "
            "Zenodo 20097931, record speed/RSS, and compare output parity."
        )
    )
    parser.add_argument("--data-dir", type=Path, default=DEFAULT_DATA_DIR)
    parser.add_argument("--out-dir", type=Path, default=DEFAULT_OUT_DIR)
    parser.add_argument("--ref", type=Path, default=DEFAULT_REF)
    parser.add_argument("--threads", type=int, default=1)
    parser.add_argument("--rust-bin", type=Path, default=Path("target/release/minibwa-rs"))
    parser.add_argument("--c-bin", type=Path, default=Path("minibwa/minibwa"))
    parser.add_argument("--skip-build", action="store_true")
    parser.add_argument("--skip-index", action="store_true")
    parser.add_argument(
        "--allocator",
        choices=["system", "mimalloc", "c-default"],
        default="system",
        help=(
            "allocator mode for builds: system builds Rust without mimalloc and C with "
            "mimalloc=0; mimalloc enables Rust mimalloc and C Makefile default; "
            "c-default preserves the C default while Rust uses --rust-features"
        ),
    )
    parser.add_argument(
        "--rust-features",
        default=None,
        help="override Rust cargo features for the benchmark build",
    )
    parser.add_argument(
        "--index-algorithm",
        choices=["lowmem", "libsais"],
        default="lowmem",
        help=(
            "index construction algorithm to compare; lowmem avoids the very high "
            "RSS of in-memory libsais on whole-genome methylation indexes"
        ),
    )
    parser.add_argument(
        "--index-block-size",
        default=None,
        help="block size passed to low-memory indexing with -b, for example 50m",
    )
    parser.add_argument("--datasets", nargs="*", help="subset: wgs hic hifi ont meth")
    parser.add_argument(
        "--sam",
        action="store_true",
        help="emit SAM for mapping cases instead of default PAF where applicable",
    )
    return parser.parse_args()


def run(args: list[str], stdout: Path, stderr: Path, time_file: Path) -> int:
    stdout.parent.mkdir(parents=True, exist_ok=True)
    command = ["/usr/bin/time", "-v", "-o", str(time_file), *args]
    with stdout.open("wb") as out, stderr.open("wb") as err:
        proc = subprocess.run(command, stdout=out, stderr=err)
    return proc.returncode


def build(args: argparse.Namespace) -> None:
    if args.skip_build:
        return
    rust_features = args.rust_features
    if rust_features is None:
        rust_features = "cli,mimalloc" if args.allocator == "mimalloc" else "cli"
    cargo_cmd = ["cargo", "build", "--release", "--features", rust_features]
    make_cmd = ["make", "-C", "minibwa"]
    if args.allocator == "system":
        make_cmd.append("mimalloc=0")
    subprocess.run(cargo_cmd, check=True)
    subprocess.run(make_cmd, check=True)


def build_metadata(args: argparse.Namespace) -> dict[str, str | None]:
    rust_features = args.rust_features
    if rust_features is None:
        rust_features = "cli,mimalloc" if args.allocator == "mimalloc" else "cli"
    return {
        "allocator": args.allocator,
        "rust_features": rust_features,
        "rustflags": os.environ.get("RUSTFLAGS"),
        "cflags": os.environ.get("CFLAGS"),
        "c_make_allocator": "mimalloc=0" if args.allocator == "system" else "default",
    }


def parse_time(path: Path) -> tuple[float | None, float | None, float | None, int | None]:
    text = path.read_text(errors="replace") if path.exists() else ""
    user = find_float(text, r"User time \(seconds\): ([0-9.]+)")
    system = find_float(text, r"System time \(seconds\): ([0-9.]+)")
    rss = find_int(text, r"Maximum resident set size \(kbytes\): ([0-9]+)")
    wall = parse_elapsed(find_text(text, r"Elapsed \(wall clock\) time.*: ([^\n]+)"))
    return wall, user, system, rss


def find_text(text: str, pattern: str) -> str | None:
    match = re.search(pattern, text)
    return match.group(1).strip() if match else None


def find_float(text: str, pattern: str) -> float | None:
    value = find_text(text, pattern)
    return float(value) if value is not None else None


def find_int(text: str, pattern: str) -> int | None:
    value = find_text(text, pattern)
    return int(value) if value is not None else None


def parse_elapsed(value: str | None) -> float | None:
    if value is None:
        return None
    parts = value.split(":")
    try:
        if len(parts) == 3:
            hours, minutes, seconds = parts
            return int(hours) * 3600 + int(minutes) * 60 + float(seconds)
        if len(parts) == 2:
            minutes, seconds = parts
            return int(minutes) * 60 + float(seconds)
        return float(value)
    except ValueError:
        return None


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def compare(step: str, dataset: str, left: Path, right: Path) -> Parity:
    equal = left.exists() and right.exists() and filecmp.cmp(left, right, shallow=False)
    return Parity(
        step=step,
        dataset=dataset,
        left=str(left),
        right=str(right),
        equal=equal,
        left_sha256=sha256(left) if left.exists() else None,
        right_sha256=sha256(right) if right.exists() else None,
    )


def indexed_file(prefix: Path, ext: str) -> Path:
    return Path(f"{prefix}.{ext}")


def record_result(
    results: list[Result],
    index_algorithm: str,
    step: str,
    dataset: str,
    impl: str,
    status: int,
    stdout: Path,
    stderr: Path,
    time_file: Path,
) -> None:
    wall, user, system, rss = parse_time(time_file)
    results.append(
        Result(
            step=step,
            dataset=dataset,
            impl=impl,
            index_algorithm=index_algorithm,
            status=status,
            wall_seconds=wall,
            user_seconds=user,
            system_seconds=system,
            max_rss_kb=rss,
            stdout=str(stdout),
            stderr=str(stderr),
            time_file=str(time_file),
        )
    )


def require(path: Path) -> Path:
    if not path.exists():
        raise FileNotFoundError(path)
    return path


def require_index(prefix: Path, mode: str) -> None:
    required = ["l2b", "mbw"]
    if mode == "meth":
        required.append("meth.mbw")
    for ext in required:
        require(indexed_file(prefix, ext))


def dataset_plan(data_dir: Path) -> dict[str, dict[str, object]]:
    return {
        "wgs": {
            "reads": [
                require(data_dir / "HG002.WGS-1M_1.fq.gz"),
                require(data_dir / "HG002.WGS-1M_2.fq.gz"),
            ],
            "extra": [],
            "index": "normal",
        },
        "hic": {
            "reads": [
                require(data_dir / "HG002.HiC-1M_1.fq.gz"),
                require(data_dir / "HG002.HiC-1M_2.fq.gz"),
            ],
            "extra": ["--hic"],
            "index": "normal",
        },
        "hifi": {
            "reads": [require(data_dir / "HG002.HiFi-10k.fa.gz")],
            "extra": ["-x", "lr"],
            "index": "normal",
        },
        "ont": {
            "reads": [require(data_dir / "HG002.ONT-10k.fa.gz")],
            "extra": ["-x", "lr"],
            "index": "normal",
        },
        "meth": {
            "reads": [
                require(data_dir / "NA12878-meth-1M_1.fa.gz"),
                require(data_dir / "NA12878-meth-1M_2.fa.gz"),
            ],
            "extra": ["--meth"],
            "index": "meth",
        },
    }


def run_index(
    args: argparse.Namespace, impl: str, bin_path: Path, mode: str, results: list[Result]
) -> Path:
    prefix = args.out_dir / "index" / impl / f"ref.{mode}"
    prefix.parent.mkdir(parents=True, exist_ok=True)
    stdout = args.out_dir / "index" / f"{impl}.{mode}.stdout"
    stderr = args.out_dir / "index" / f"{impl}.{mode}.stderr"
    time_file = args.out_dir / "index" / f"{impl}.{mode}.time"
    cmd = [
        str(bin_path),
        "index",
        "-t",
        str(args.threads),
    ]
    if args.index_algorithm == "lowmem":
        cmd.append("-l")
        if args.index_block_size:
            cmd.extend(["-b", args.index_block_size])
    if mode == "meth":
        cmd.append("--meth")
    cmd.extend([str(args.ref), str(prefix)])
    print("running", " ".join(cmd), file=sys.stderr)
    status = run(cmd, stdout, stderr, time_file)
    record_result(
        results,
        args.index_algorithm,
        "index",
        mode,
        impl,
        status,
        stdout,
        stderr,
        time_file,
    )
    if status != 0:
        raise subprocess.CalledProcessError(status, cmd)
    if args.index_algorithm == "lowmem" and mode == "meth":
        run_l2b_restore(args, impl, bin_path, mode, prefix, results)
    require_index(prefix, mode)
    return prefix


def run_l2b_restore(
    args: argparse.Namespace,
    impl: str,
    bin_path: Path,
    mode: str,
    prefix: Path,
    results: list[Result],
) -> None:
    stdout = args.out_dir / "index" / f"{impl}.{mode}.restore-l2b.stdout"
    stderr = args.out_dir / "index" / f"{impl}.{mode}.restore-l2b.stderr"
    time_file = args.out_dir / "index" / f"{impl}.{mode}.restore-l2b.time"
    cmd = [str(bin_path), "fa2bit", str(args.ref), f"{prefix}.l2b"]
    print("running", " ".join(cmd), file=sys.stderr)
    status = run(cmd, stdout, stderr, time_file)
    record_result(
        results,
        args.index_algorithm,
        "index-restore-l2b",
        mode,
        impl,
        status,
        stdout,
        stderr,
        time_file,
    )
    if status != 0:
        raise subprocess.CalledProcessError(status, cmd)


def run_map(
    args: argparse.Namespace,
    dataset: str,
    spec: dict[str, object],
    impl: str,
    bin_path: Path,
    index_prefix: Path,
    results: list[Result],
) -> Path:
    suffix = "sam" if args.sam else "paf"
    stdout = args.out_dir / "map" / dataset / f"{impl}.{suffix}"
    stderr = args.out_dir / "map" / dataset / f"{impl}.stderr"
    time_file = args.out_dir / "map" / dataset / f"{impl}.time"
    cmd = [str(bin_path), "map"]
    if args.sam:
        cmd.append("-a")
    cmd.extend(spec["extra"])
    cmd.extend(["-t", str(args.threads), str(index_prefix)])
    cmd.extend(str(path) for path in spec["reads"])
    print("running", " ".join(cmd), file=sys.stderr)
    status = run(cmd, stdout, stderr, time_file)
    record_result(
        results,
        args.index_algorithm,
        "map",
        dataset,
        impl,
        status,
        stdout,
        stderr,
        time_file,
    )
    if status != 0:
        raise subprocess.CalledProcessError(status, cmd)
    return stdout


def write_summary(
    out_dir: Path,
    results: list[Result],
    parities: list[Parity],
    metadata: dict[str, str | None],
) -> None:
    out_dir.mkdir(parents=True, exist_ok=True)
    with (out_dir / "summary.csv").open("w", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(asdict(results[0]).keys()))
        writer.writeheader()
        for result in results:
            writer.writerow(asdict(result))
    with (out_dir / "parity.csv").open("w", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(asdict(parities[0]).keys()))
        writer.writeheader()
        for parity in parities:
            writer.writerow(asdict(parity))
    (out_dir / "summary.json").write_text(
        json.dumps(
            {
                "metadata": metadata,
                "results": [asdict(result) for result in results],
                "parity": [asdict(parity) for parity in parities],
            },
            indent=2,
            sort_keys=True,
        )
        + "\n"
    )


def main() -> int:
    args = parse_args()
    os.chdir(Path(__file__).resolve().parents[1])
    args.data_dir = args.data_dir.resolve()
    args.out_dir = args.out_dir.resolve()
    args.ref = args.ref.resolve()
    require(args.ref)

    build(args)
    require(args.c_bin)
    require(args.rust_bin)

    selected = set(args.datasets) if args.datasets else None
    plan = dataset_plan(args.data_dir)
    if selected:
        unknown = selected.difference(plan)
        if unknown:
            raise ValueError(f"unknown datasets: {', '.join(sorted(unknown))}")
        plan = {name: spec for name, spec in plan.items() if name in selected}

    results: list[Result] = []
    parities: list[Parity] = []

    if args.skip_index:
        c_normal = args.out_dir / "index" / "c" / "ref.normal"
        rust_normal = args.out_dir / "index" / "rust" / "ref.normal"
        c_meth = args.out_dir / "index" / "c" / "ref.meth"
        rust_meth = args.out_dir / "index" / "rust" / "ref.meth"
        require_index(c_normal, "normal")
        require_index(rust_normal, "normal")
        if any(spec["index"] == "meth" for spec in plan.values()):
            require_index(c_meth, "meth")
            require_index(rust_meth, "meth")
    else:
        c_normal = run_index(args, "c", args.c_bin, "normal", results)
        rust_normal = run_index(args, "rust", args.rust_bin, "normal", results)
        for ext in ["l2b", "mbw"]:
            parities.append(
                compare(
                    "index",
                    f"normal.{ext}",
                    indexed_file(c_normal, ext),
                    indexed_file(rust_normal, ext),
                )
            )

        needs_meth = any(spec["index"] == "meth" for spec in plan.values())
        if needs_meth:
            c_meth = run_index(args, "c", args.c_bin, "meth", results)
            rust_meth = run_index(args, "rust", args.rust_bin, "meth", results)
            for ext in ["l2b", "mbw", "meth.mbw"]:
                parities.append(
                    compare(
                        "index",
                        f"meth.{ext}",
                        indexed_file(c_meth, ext),
                        indexed_file(rust_meth, ext),
                    )
                )
        else:
            c_meth = rust_meth = Path()

    for dataset, spec in plan.items():
        c_index = c_meth if spec["index"] == "meth" else c_normal
        rust_index = rust_meth if spec["index"] == "meth" else rust_normal
        c_out = run_map(args, dataset, spec, "c", args.c_bin, c_index, results)
        rust_out = run_map(args, dataset, spec, "rust", args.rust_bin, rust_index, results)
        parities.append(compare("map", dataset, c_out, rust_out))

    write_summary(args.out_dir, results, parities, build_metadata(args))
    failed = [p for p in parities if not p.equal]
    print(f"wrote {args.out_dir / 'summary.csv'}")
    print(f"wrote {args.out_dir / 'parity.csv'}")
    if failed:
        print(f"{len(failed)} parity comparisons failed", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
