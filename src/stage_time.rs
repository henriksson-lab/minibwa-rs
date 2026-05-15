
use std::cell::Cell;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Instant;

pub static ENABLED: AtomicBool = AtomicBool::new(false);

#[derive(Copy, Clone)]
#[repr(usize)]
pub enum Bucket {
    ReadIo = 0,
    Encode = 1,
    Seed = 2,
    Anchor = 3,
    Chain = 4,
    Align = 5,
    MapqPost = 6,
    Pair = 7,
    Output = 8,
}
pub const N: usize = 9;
const NAMES: [&str; N] = [
    "read_io",
    "encode",
    "seed",
    "anchor",
    "chain",
    "align",
    "mapq+post",
    "pair",
    "output",
];

static TOTALS: [AtomicU64; N] = [
    AtomicU64::new(0),
    AtomicU64::new(0),
    AtomicU64::new(0),
    AtomicU64::new(0),
    AtomicU64::new(0),
    AtomicU64::new(0),
    AtomicU64::new(0),
    AtomicU64::new(0),
    AtomicU64::new(0),
];

thread_local! {
    static LOCAL: [Cell<u64>; N] = const {
        [
            Cell::new(0), Cell::new(0), Cell::new(0),
            Cell::new(0), Cell::new(0), Cell::new(0),
            Cell::new(0), Cell::new(0), Cell::new(0),
        ]
    };
}

pub fn init_from_env() {
    if std::env::var("MBWA_TIME_STAGES").ok().as_deref() == Some("1") {
        ENABLED.store(true, Ordering::Relaxed);
    }
}

#[inline(always)]
pub fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

#[inline(always)]
pub fn measure<R>(b: Bucket, f: impl FnOnce() -> R) -> R {
    if !ENABLED.load(Ordering::Relaxed) {
        return f();
    }
    let t0 = Instant::now();
    let r = f();
    let ns = t0.elapsed().as_nanos() as u64;
    LOCAL.with(|arr| arr[b as usize].set(arr[b as usize].get() + ns));
    r
}

pub fn flush_local() {
    if !ENABLED.load(Ordering::Relaxed) {
        return;
    }
    LOCAL.with(|arr| {
        for i in 0..N {
            let v = arr[i].get();
            if v > 0 {
                TOTALS[i].fetch_add(v, Ordering::Relaxed);
                arr[i].set(0);
            }
        }
    });
}

#[inline]
pub fn accumulate_global(b: Bucket, ns: u64) {
    if !ENABLED.load(Ordering::Relaxed) {
        return;
    }
    TOTALS[b as usize].fetch_add(ns, Ordering::Relaxed);
}

pub fn report() {
    if !ENABLED.load(Ordering::Relaxed) {
        return;
    }
    let mut total: u64 = 0;
    let mut vals = [0u64; N];
    for i in 0..N {
        vals[i] = TOTALS[i].load(Ordering::Relaxed);
        total += vals[i];
    }
    eprintln!("[stage] cumulative thread-time per phase (ms across all worker threads):");
    for i in 0..N {
        let pct = if total > 0 {
            100.0 * vals[i] as f64 / total as f64
        } else {
            0.0
        };
        eprintln!(
            "[stage]   {:<10} {:>10} ms  {:>5.1}%",
            NAMES[i],
            vals[i] / 1_000_000,
            pct
        );
    }
    eprintln!("[stage]   {:<10} {:>10} ms", "TOTAL", total / 1_000_000);
}
