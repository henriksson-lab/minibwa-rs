use std::process::Command;

use crate::ksw2::{
    ksw_apply_zdrop, ksw_backtrack, ksw_extz_t, ksw_gen_nt4_mat, ksw_reset_extz, KSW_NEG_INF,
};

#[test]
#[ignore = "strict original-C ksw helper verifier; requires gcc and original C headers"]
fn ksw_helpers_original_c_conformance_harness() {
    let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let exe = std::env::temp_dir().join(format!(
        "minibwa_ksw_helpers_conformance_{}_{}",
        std::process::id(),
        option_env!("CARGO_PKG_VERSION").unwrap_or("dev")
    ));
    let output = Command::new("gcc")
        .current_dir(&manifest)
        .args([
            "-std=c99",
            "-O2",
            "-I",
            "minibwa",
            "tools/ksw_helpers_conformance.c",
            "-o",
        ])
        .arg(&exe)
        .output()
        .expect("failed to execute gcc for original ksw helper harness");
    assert!(
        output.status.success(),
        "gcc failed for original ksw helper harness\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let output = Command::new(&exe)
        .output()
        .expect("failed to run original ksw helper harness");
    assert!(
        output.status.success(),
        "original ksw helper harness failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("harness output is not UTF-8");
    let mut total = 0usize;
    for line in stdout.lines().filter(|line| !line.trim().is_empty()) {
        total += 1;
        let parts = line.split_whitespace().collect::<Vec<_>>();
        match parts.first().copied() {
            Some("mat") => {
                assert_eq!(parts.len(), 31, "bad mat line: {line}");
                let mut mat = [0i8; 25];
                ksw_gen_nt4_mat(
                    &mut mat,
                    parts[2].parse::<i32>().expect("match field") as i8,
                    parts[3].parse::<i32>().expect("mismatch field") as i8,
                    parts[4].parse::<i32>().expect("ambiguous field") as i8,
                    parts[5].parse::<i32>().expect("wildcard field") as i8,
                );
                let expected = parts[6..]
                    .iter()
                    .map(|s| s.parse::<i32>().expect("mat field") as i8)
                    .collect::<Vec<_>>();
                assert_eq!(mat.to_vec(), expected, "mat mismatch for {line}");
            }
            Some("reset") => {
                assert_eq!(parts.len(), 13, "bad reset line: {line}");
                let mut ez = ksw_extz_t {
                    max: 123,
                    zdropped: 1,
                    max_q: 7,
                    max_t: 8,
                    mqe: 9,
                    mqe_t: 10,
                    mte: 11,
                    mte_q: 12,
                    score: 13,
                    m_cigar: 14,
                    n_cigar: 15,
                    reach_end: 1,
                    cigar: Vec::new(),
                };
                ksw_reset_extz(&mut ez);
                let actual = [
                    ez.max as i32,
                    ez.zdropped as i32,
                    ez.max_q,
                    ez.max_t,
                    ez.mqe,
                    ez.mqe_t,
                    ez.mte,
                    ez.mte_q,
                    ez.score,
                    ez.n_cigar,
                    ez.reach_end,
                ];
                let expected = parts[2..]
                    .iter()
                    .map(|s| s.parse::<i32>().expect("reset field"))
                    .collect::<Vec<_>>();
                assert_eq!(actual.to_vec(), expected, "reset mismatch for {line}");
                assert_eq!(ez.score, KSW_NEG_INF);
            }
            Some("zdrop") => {
                assert_eq!(parts.len(), 16, "bad zdrop line: {line}");
                let mut ez = ksw_extz_t {
                    max: parts[2].parse().expect("max field"),
                    max_t: parts[3].parse().expect("max_t field"),
                    max_q: parts[4].parse().expect("max_q field"),
                    ..Default::default()
                };
                let ret = ksw_apply_zdrop(
                    &mut ez,
                    parts[5].parse().expect("off field"),
                    parts[6].parse().expect("e field"),
                    parts[7].parse().expect("i field"),
                    parts[8].parse().expect("score field"),
                    parts[9].parse().expect("zdrop field"),
                    parts[10].parse::<i32>().expect("e2 field") as i8,
                );
                let actual = [
                    ret as i64,
                    ez.max as i64,
                    ez.zdropped as i64,
                    ez.max_t as i64,
                    ez.max_q as i64,
                ];
                let expected = parts[11..]
                    .iter()
                    .map(|s| s.parse::<i64>().expect("zdrop output"))
                    .collect::<Vec<_>>();
                assert_eq!(actual.to_vec(), expected, "zdrop mismatch for {line}");
            }
            Some("bt") => {
                assert!(parts.len() >= 13, "short bt line: {line}");
                let is_rot = parts[2].parse::<i32>().expect("is_rot field");
                let is_rev = parts[3].parse::<i32>().expect("is_rev field");
                let min_intron_len = parts[4].parse::<i32>().expect("min_intron_len field");
                let n_col = parts[5].parse::<i32>().expect("n_col field");
                let i0 = parts[6].parse::<i32>().expect("i0 field");
                let j0 = parts[7].parse::<i32>().expect("j0 field");
                let off_len = parts[8].parse::<usize>().expect("off_len field");
                let p_len = parts[9].parse::<usize>().expect("p_len field");
                let mut pos = 10;
                let off = parts[pos..pos + off_len]
                    .iter()
                    .map(|s| s.parse::<i32>().expect("off field"))
                    .collect::<Vec<_>>();
                pos += off_len;
                let has_off_end = parts[pos].parse::<i32>().expect("has_off_end field") != 0;
                pos += 1;
                let off_end = if has_off_end {
                    let v = parts[pos..pos + off_len]
                        .iter()
                        .map(|s| s.parse::<i32>().expect("off_end field"))
                        .collect::<Vec<_>>();
                    pos += off_len;
                    Some(v)
                } else {
                    None
                };
                let p = parts[pos..pos + p_len]
                    .iter()
                    .map(|s| s.parse::<i32>().expect("p field") as u8)
                    .collect::<Vec<_>>();
                pos += p_len;
                let expected_m = parts[pos].parse::<i32>().expect("expected_m field");
                pos += 1;
                let expected_n = parts[pos].parse::<i32>().expect("expected_n field");
                pos += 1;
                assert_eq!(
                    parts.len(),
                    pos + expected_n.max(0) as usize,
                    "bad bt cigar count: {line}"
                );
                let expected_cigar = parts[pos..]
                    .iter()
                    .map(|s| s.parse::<u32>().expect("cigar field"))
                    .collect::<Vec<_>>();
                let mut m = 0;
                let mut n = 0;
                let mut cigar = Vec::new();
                ksw_backtrack(
                    (),
                    is_rot,
                    is_rev,
                    min_intron_len,
                    &p,
                    &off,
                    off_end.as_deref(),
                    n_col,
                    i0,
                    j0,
                    &mut m,
                    &mut n,
                    &mut cigar,
                );
                assert_eq!(
                    (m, n),
                    (expected_m, expected_n),
                    "bt sizes mismatch for {line}"
                );
                assert_eq!(
                    cigar[..n as usize].to_vec(),
                    expected_cigar,
                    "bt cigar mismatch for {line}"
                );
            }
            _ => panic!("unknown ksw helper harness line: {line}"),
        }
    }
    assert!(
        total >= 16,
        "ksw helper harness produced only {total} cases"
    );
}
