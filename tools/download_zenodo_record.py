#!/usr/bin/env python3
"""Download all files from a Zenodo record with checksum verification."""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path


DEFAULT_RECORD = "20097931"
API_ROOT = "https://zenodo.org/api/records"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Download and verify every file attached to a Zenodo record."
    )
    parser.add_argument("--record", default=DEFAULT_RECORD, help="Zenodo record id")
    parser.add_argument(
        "-o",
        "--out-dir",
        default=f".tmp/zenodo-{DEFAULT_RECORD}",
        help="directory for downloaded files",
    )
    parser.add_argument("--retries", type=int, default=5, help="download retry count")
    parser.add_argument(
        "--timeout", type=float, default=60.0, help="HTTP timeout in seconds"
    )
    parser.add_argument(
        "--force", action="store_true", help="redownload files even when verified"
    )
    return parser.parse_args()


def fetch_json(url: str, timeout: float) -> dict:
    request = urllib.request.Request(url, headers={"User-Agent": "minibwa-rs/zenodo"})
    with urllib.request.urlopen(request, timeout=timeout) as response:
        return json.load(response)


def checksum(path: Path, algorithm: str) -> str:
    digest = hashlib.new(algorithm)
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def verified(path: Path, expected: str | None) -> bool:
    if not path.exists() or expected is None:
        return False
    algorithm, value = expected.split(":", 1) if ":" in expected else ("md5", expected)
    return checksum(path, algorithm) == value


def download_file(url: str, dest: Path, size: int, retries: int, timeout: float) -> None:
    part = dest.with_suffix(dest.suffix + ".part")
    dest.parent.mkdir(parents=True, exist_ok=True)

    for attempt in range(1, retries + 1):
        resume_at = part.stat().st_size if part.exists() else 0
        headers = {"User-Agent": "minibwa-rs/zenodo"}
        if resume_at:
            headers["Range"] = f"bytes={resume_at}-"
        request = urllib.request.Request(url, headers=headers)

        try:
            with urllib.request.urlopen(request, timeout=timeout) as response:
                status = getattr(response, "status", 200)
                mode = "ab" if resume_at and status == 206 else "wb"
                if mode == "wb" and part.exists():
                    part.unlink()
                with part.open(mode) as handle:
                    copied = part.stat().st_size if part.exists() else 0
                    while True:
                        chunk = response.read(1024 * 1024)
                        if not chunk:
                            break
                        handle.write(chunk)
                        copied += len(chunk)
                        if size:
                            pct = copied * 100.0 / size
                            print(
                                f"\r  {dest.name}: {copied}/{size} bytes ({pct:.1f}%)",
                                end="",
                                file=sys.stderr,
                            )
            print(file=sys.stderr)
            part.replace(dest)
            return
        except (urllib.error.URLError, TimeoutError) as error:
            if attempt == retries:
                raise
            wait = min(30, 2**attempt)
            print(
                f"retrying {dest.name} after {error!s} ({attempt}/{retries}); sleeping {wait}s",
                file=sys.stderr,
            )
            time.sleep(wait)


def main() -> int:
    args = parse_args()
    out_dir = Path(args.out_dir)
    record = fetch_json(f"{API_ROOT}/{args.record}", args.timeout)
    files = record.get("files", [])
    if not files:
        print(f"record {args.record} has no files", file=sys.stderr)
        return 1

    out_dir.mkdir(parents=True, exist_ok=True)
    manifest_path = out_dir / "zenodo-record.json"
    manifest_path.write_text(json.dumps(record, indent=2, sort_keys=True) + "\n")

    for entry in files:
        name = entry["key"]
        dest = out_dir / name
        expected = entry.get("checksum")
        if not args.force and verified(dest, expected):
            print(f"verified {dest}")
            continue

        url = entry.get("links", {}).get("self")
        if not url:
            print(f"missing download URL for {name}", file=sys.stderr)
            return 1
        print(f"downloading {name} -> {dest}")
        download_file(url, dest, int(entry.get("size", 0)), args.retries, args.timeout)
        if expected and not verified(dest, expected):
            print(f"checksum mismatch for {dest}", file=sys.stderr)
            return 1
        print(f"verified {dest}")

    print(f"wrote manifest {manifest_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
