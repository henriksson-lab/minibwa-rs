
use std::process::Command;

use crate::ksw2_ll_sse::{ksw_ll_i16_core, ksw_ll_qinit, ksw_ll_u8_core, ksw_llrst_t};

const MAT: [i8; 25] = [
    2, -4, -4, -4, -1, -4, 2, -4, -4, -1, -4, -4, 2, -4, -1, -4, -4, -4, 2, -1, -1, -1, -1, -1, -1,
];
const ALT_MAT: [i8; 25] = [
    3, -2, -2, -2, -2, -2, 3, -2, -2, -2, -2, -2, 3, -2, -2, -2, -2, -2, 3, -2, -2, -2, -2, -2, -2,
];
const PEAK_MAT: [i8; 25] = [
    7, -3, -3, -3, -2, -3, 7, -3, -3, -2, -3, -3, 7, -3, -2, -3, -3, -3, 7, -2, -2, -2, -2, -2, -2,
];

#[test]
#[ignore = "strict original-C ksw ll verifier; requires gcc, original C sources, and x86 SSE2"]
fn ksw_ll_original_c_conformance_harness() {
    if !cfg!(any(target_arch = "x86", target_arch = "x86_64")) {
        eprintln!("skipping original ksw ll SSE harness on non-x86 target");
        return;
    }

    let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let exe = std::env::temp_dir().join(format!(
        "minibwa_ksw_ll_conformance_{}_{}",
        std::process::id(),
        option_env!("CARGO_PKG_VERSION").unwrap_or("dev")
    ));
    let output = Command::new("gcc")
        .current_dir(&manifest)
        .args([
            "-std=c99",
            "-O2",
            "-msse2",
            "-I",
            "minibwa",
            "tools/ksw_ll_conformance.c",
            "minibwa/ksw2_ll_sse.c",
            "-o",
        ])
        .arg(&exe)
        .output()
        .expect("failed to execute gcc for original ksw ll harness");
    assert!(
        output.status.success(),
        "gcc failed for original ksw ll harness\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let output = Command::new(&exe)
        .output()
        .expect("failed to run original ksw ll harness");
    assert!(
        output.status.success(),
        "original ksw ll harness failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("harness output is not UTF-8");
    let mut total = 0usize;
    let mut mismatches = Vec::new();
    for line in stdout.lines().filter(|line| !line.trim().is_empty()) {
        total += 1;
        let parts = line.split_whitespace().collect::<Vec<_>>();
        assert_eq!(parts.len(), 15, "bad harness line: {line}");
        let mat_id = parts[1].parse::<i32>().expect("mat_id");
        let size = parts[2].parse::<i32>().expect("size");
        let qlen = parts[3].parse::<i32>().expect("qlen");
        let tlen = parts[4].parse::<i32>().expect("tlen");
        let gapo = parts[5].parse::<i32>().expect("gapo");
        let gape = parts[6].parse::<i32>().expect("gape");
        let xtra = parts[7].parse::<i32>().expect("xtra");
        let query = parts[8]
            .bytes()
            .map(|b| {
                assert!((b'0'..=b'4').contains(&b), "bad sequence digit in {line}");
                b - b'0'
            })
            .collect::<Vec<_>>();
        let target = parts[9]
            .bytes()
            .map(|b| {
                assert!((b'0'..=b'4').contains(&b), "bad sequence digit in {line}");
                b - b'0'
            })
            .collect::<Vec<_>>();
        let expected = ksw_llrst_t {
            score: parts[10].parse().expect("score"),
            te: parts[11].parse().expect("te"),
            qe: parts[12].parse().expect("qe"),
            score2: parts[13].parse().expect("score2"),
            te2: parts[14].parse().expect("te2"),
        };
        let mat = match mat_id {
            0 => &MAT,
            1 => &ALT_MAT,
            2 => &PEAK_MAT,
            _ => panic!("unknown ksw ll conformance matrix in {line}"),
        };
        let q = ksw_ll_qinit((), size, qlen, &query, 5, mat);
        let actual = if size == 1 {
            ksw_ll_u8_core(&q, tlen, &target, gapo, gape, xtra)
        } else {
            ksw_ll_i16_core(&q, tlen, &target, gapo, gape, xtra)
        };
        if actual != expected {
            mismatches.push(format!(
                "case line: {}\nexpected: {:?}\nactual:   {:?}",
                line, expected, actual
            ));
        }
    }
    assert!(
        total >= 450,
        "original ksw ll harness produced only {total} cases"
    );
    assert!(
        mismatches.is_empty(),
        "{} / {total} original ksw ll cases mismatched\n{}",
        mismatches.len(),
        mismatches
            .iter()
            .take(12)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n\n")
    );
}
