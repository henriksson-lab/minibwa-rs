# minibwa-rs

This is a Rust translation of minibwa (commit: e89baa0732b4c69a80b029a0569e14b0c1ae03ef)

## This is an LLM-mediated faithful (hopefully) translation, not the original code! 

Most users should probably first see if the existing original code works for them, unless they have reason otherwise. The original source
may have newer features and it has had more love in terms of fixing bugs. In fact, we aim to replicate bugs if they are present, for the
sake of reproducibility! (but then we might have added a few more in the process)

There are however cases when you might prefer this Rust version. We generally agree with [this manifesto](https://rewrites.bio/) but more specifically:
* We have had many issues with ensuring that our software works using existing containers (Docker, PodMan, Singularity). One size does not fit all and it eats our resources trying to keep up with every way of delivering software
* Common package managers do not work well. It was great when we had a few Linux distributions with stable procedures, but now there are just too many ecosystems (Homebrew, Conda). Conda has an NP-complete resolver which does not scale. Homebrew is only so-stable. And our dependencies in Python still break. These can no longer be considered professional serious options. Meanwhile, Cargo enables multiple versions of packages to be available, even within the same program(!)
* The future is the web. We deploy software in the web browser, and until now that has meant Javascript. This is a language where even the == operator is broken. Typescript is one step up, but a game changer is the ability to compile Rust code into webassembly, enabling performance and sharing of code with the backend. Translating code to Rust enables new ways of deployment and running code in the browser has especial benefits for science - researchers do not have deep pockets to run servers, so pushing compute to the user enables deployment that otherwise would be impossible
* Old CLI-based utilities are bad for the environment(!). A large amount of compute resources are spent creating and communicating via small files, which we can bypass by using code as libraries. Even better, we can avoid frequent reloading of databases by hoisting this stage, with up to 100x speedups in some cases. Less compute means faster compute and less electricity wasted
* LLM-mediated translations may actually be safer to use than the original code. This article shows that [running the same code on different operating systems can give somewhat different answers](https://doi.org/10.1038/nbt.3820). This is a gap that Rust+Cargo can reduce. Typesafe interfaces also reduce coding mistakes and error handling, as opposed to typical command-line scripting

But:

* **This approach should still be considered experimental**. The LLM technology is immature and has sharp corners. But there are opportunities to reap, and the genie is not going back into the bottle. This translation is as much aimed to learn how to improve the technology and get feedback on the results.
* Translations are not endorsed by the original authors unless otherwise noted. **Do not send bug reports to the original developers**. Use our Github issues page instead.
* **Do not trust the benchmarks on this page**. They are used to help evaluate the translation. If you want improved performance, you generally have to use this code as a library, and use the additional tricks it offers. We generally accept performance losses in order to reduce our dependency issues
* **Check the original Github pages for information about the package**. This README is kept sparse on purpose. It is not meant to be the primary source of information
* **If you are the author of the original code and wish to move to Rust, you can obtain ownership of this repository and crate**. Until then, our commitment is to offer an as-faithful-as-possible translation of a snapshot of your code. If we find serious bugs, we will report them to you. Otherwise we will just replicate them, to ensure comparability across studies that claim to use package XYZ v.666. Think of this like a fancy Ubuntu .deb-package of your software - that is how we treat it

This blurb might be out of date. Go to [this page](https://github.com/henriksson-lab/rustification) for the latest information and further information about how we approach translation

## Benchmark snapshot

This is a development benchmark used to check translation parity, not a general
performance claim. The run mapped realistic human read fixtures from Zenodo
record 20097931 against whole-genome GRCh38 indexes, with 20 mapping threads
and PAF output enabled. Timings were captured on 2026-06-03 with
`/usr/bin/time -v`; both binaries were release builds using mimalloc, and the
normal indexes were built with the low-memory indexer:

```sh
TMPDIR=/big/temp/minibwa \
python3 tools/benchmark_zenodo_20097931.py \
  --data-dir .tmp/zenodo-20097931 \
  --out-dir /big/temp/minibwa/zenodo-20097931-bench-mimalloc-t20-readme-20260603-v1 \
  --ref .tmp/large-real/human_grch38/ref.fa.gz \
  --threads 20 \
  --allocator mimalloc \
  --datasets wgs hic hifi ont \
  --skip-build \
  --skip-index
```

For each dataset, the C and Rust PAF outputs were byte-identical.

20 mapping threads:

| Dataset | C wall | Rust wall | Rust/C wall | C user | Rust user | C system | Rust system | C RSS | Rust RSS | Rust/C RSS | Parity |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| WGS paired-end 1M | 14.64 s | 15.14 s | 1.03x | 225.11 s | 239.53 s | 3.04 s | 2.79 s | 8,655,788 KB | 8,109,976 KB | 0.94x | identical |
| Hi-C paired-end 1M | 19.21 s | 19.29 s | 1.00x | 314.83 s | 322.07 s | 3.35 s | 2.88 s | 8,932,968 KB | 8,216,752 KB | 0.92x | identical |
| HiFi 10k | 11.26 s | 11.68 s | 1.04x | 142.88 s | 155.82 s | 3.29 s | 4.19 s | 9,760,572 KB | 10,052,532 KB | 1.03x | identical |
| ONT 10k | 13.54 s | 14.06 s | 1.04x | 198.14 s | 212.37 s | 3.92 s | 4.90 s | 9,827,808 KB | 9,939,756 KB | 1.01x | identical |

30 mapping threads:

| Dataset | C wall | Rust wall | Rust/C wall | C user | Rust user | C system | Rust system | C RSS | Rust RSS | Rust/C RSS | Parity |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| WGS paired-end 1M | 13.95 s | 14.12 s | 1.01x | 311.28 s | 328.22 s | 3.47 s | 2.94 s | 8,736,640 KB | 8,183,580 KB | 0.94x | identical |
| Hi-C paired-end 1M | 18.21 s | 18.16 s | 1.00x | 438.91 s | 448.07 s | 3.68 s | 3.19 s | 8,963,144 KB | 8,278,780 KB | 0.92x | identical |
| HiFi 10k | 11.01 s | 11.33 s | 1.03x | 207.31 s | 227.77 s | 5.93 s | 5.25 s | 11,309,104 KB | 10,536,768 KB | 0.93x | identical |
| ONT 10k | 12.80 s | 13.57 s | 1.06x | 276.23 s | 304.79 s | 4.40 s | 5.59 s | 10,648,100 KB | 10,675,120 KB | 1.00x | identical |

In these runs, `minibwa-rs` is near wall-time parity on WGS and Hi-C, 3-4%
slower on HiFi, and 4-6% slower on ONT. Peak RSS remains lower on WGS and Hi-C,
lower on 30-thread HiFi, and roughly at parity on ONT.

## Possible upstream bugs

C bugs the Rust port silently fixes (i.e. the Rust translation deviates from
upstream C behavior here because the upstream behavior is undefined):

5. `mb_escape` buffer-read overrun on a trailing backslash
   (`minibwa/format.c:52-66`). With input ending in `\`, the inner `++p` walks
   onto the NUL, the `if/else if` chain doesn't match, and the outer `for`'s
   post-increment then walks one byte past the buffer end, which the next loop
   test reads. Reachable via `-R "...\\"` style read-group lines.
6. `krealloc` NULL-deref plus use-after-free under OOM
   (`minibwa/kalloc.c:168-185`). On `kmalloc` failure the code calls `memcpy`
   on the NULL return and then `kfree`s the old buffer, violating C `realloc`'s
   contract that failure must leave the original intact.


## Cargo Features

`minibwa-rs` is a library crate by default. The command-line binary is optional
and is disabled unless the `cli` feature is requested.

```toml
[dependencies]
minibwa-rs = "0.1"
```

To build or install the CLI, enable the feature explicitly:

```sh
cargo build --release --features cli
cargo install minibwa-rs --features cli
```

The same feature also exposes `minibwa_rs::cli` for library users that want to
embed the command dispatcher instead of spawning a process.

The packaged CLI binary uses `mimalloc` as its global allocator. This is
intentional: minibwa's mapping path performs many short-lived allocations, and
the original C benchmark builds also use mimalloc. Library users keep control of
their process-wide allocator, but high-throughput applications should strongly
consider enabling mimalloc in their own binary:

```rust
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;
```

with:

```toml
mimalloc = { version = "0.1.50", default-features = false }
```

The translated Rust KSW2 alignment path is always used; the crate does not link
the original C KSW2 kernels.

### SIMD and supported architectures

Only `x86_64` is currently supported. The KSW2 alignment kernels
(`src/ksw2_*_sse.rs`) use SSE2/SSE4.1 intrinsics via
`std::arch::x86_64` and do not yet have non-x86 fallbacks, so the crate will
not build on `aarch64`, `riscv64`, etc. The portable SIMD shim in
`src/s2n_lite.rs` does include scalar emulation for non-x86 targets and could
be extended to cover the KSW2 helpers; NEON and other native SIMD backends are
not implemented.

For local development from this repository:

```sh
cargo run --release --features cli -- map -t 4 ref_prefix reads.fq > out.paf
```

## Library Examples

The most stable library entry points are currently close to the translated C
surface. For CLI-compatible behavior from Rust code, call the subcommand
functions directly and provide the argument vector that would normally follow
the executable name.

```rust
use std::io::BufWriter;

fn main() {
    let mut out = BufWriter::new(Vec::new());
    let args = vec![
        "map".to_string(),
        "-t".to_string(),
        "4".to_string(),
        "ref_prefix".to_string(),
        "reads.fq".to_string(),
    ];

    let (status, message) = minibwa_rs::map_main::main_map_write(&args, &mut out);
    assert_eq!(status, 0, "{message}");

    let paf = String::from_utf8(out.into_inner().unwrap()).unwrap();
    println!("{paf}");
}
```

Call indexing the same way when you want command-compatible indexing from a
library context:

```rust
fn main() {
    let args = vec![
        "index".to_string(),
        "-t".to_string(),
        "4".to_string(),
        "reference.fa".to_string(),
        "ref_prefix".to_string(),
    ];

    let (status, message) = minibwa_rs::index::main_index(&args);
    assert_eq!(status, 0, "{message}");
}
```

If you specifically want the CLI dispatcher from a library, enable the `cli`
feature and pass an argv vector including the executable name:

```toml
[dependencies]
minibwa-rs = { version = "0.1", features = ["cli"] }
```

```rust
fn main() {
    let argv = vec![
        "minibwa-rs".to_string(),
        "version".to_string(),
    ];
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let status = minibwa_rs::cli::run_with_writers(&argv, &mut stdout, &mut stderr)
        .expect("failed to run minibwa CLI dispatcher");
    assert_eq!(status, 0);
}
```

For long-running applications, load the index once and pass your own Rayon pool
down to mapping. This avoids rebuilding a thread pool for every call and lets
the embedding application own scheduling.

```rust
use minibwa_rs::{map_main, mbidx, options};
use rayon::ThreadPoolBuilder;

fn main() {
    let mut opt = options::mb_opt_t::default();
    options::mb_opt_init(&mut opt);
    opt.n_threads = 4;

    let idx = mbidx::mb_idx_load("ref_prefix", 0).expect("failed to load index");
    let pool = ThreadPoolBuilder::new()
        .num_threads(opt.n_threads as usize)
        .build()
        .unwrap();

    let mut out = Vec::new();
    pool.install(|| {
        map_main::mb_map_file_with_pool(
            &idx,
            &mut opt,
            1,
            &["reads.fq"],
            &mut out,
            Some(&pool),
        );
    });
}
```

## CLI Examples

Build the binary with the optional feature:

```sh
cargo build --release --features cli
```

Build an index:

```sh
target/release/minibwa-rs index -t 4 reference.fa ref_prefix
```

Map reads to PAF:

```sh
target/release/minibwa-rs map -t 4 ref_prefix reads.fq > reads.paf
```

Map paired-end reads to SAM:

```sh
target/release/minibwa-rs map -a -t 4 ref_prefix reads_R1.fq reads_R2.fq > reads.sam
```

Run long-read chaining presets:

```sh
target/release/minibwa-rs map -x lr --chain-only -t 4 ref_prefix reads.fq > chains.paf
```


# Citing

TODO!
