
use std::process::Command;

use crate::ksw2::ksw_extz_t;
use crate::ksw2_extd2_sse::ksw_extd2_sse;
use crate::ksw2_extz2_sse::ksw_extz2_sse;

const MAT: [i8; 25] = [
    2, -4, -4, -4, -1, -4, 2, -4, -4, -1, -4, -4, 2, -4, -1, -4, -4, -4, 2, -1, -1, -1, -1, -1, -1,
];
const ALT_MAT: [i8; 25] = [
    3, -2, -2, -2, -2, -2, 3, -2, -2, -2, -2, -2, 3, -2, -2, -2, -2, -2, 3, -2, -2, -2, -2, -2, -2,
];
const PEAK_MAT: [i8; 25] = [
    7, -3, -3, -3, -2, -3, 7, -3, -3, -2, -3, -3, 7, -3, -2, -3, -3, -3, 7, -2, -2, -2, -2, -2, -2,
];
#[derive(Clone, Debug, PartialEq, Eq)]
struct Observed {
    max: u32,
    zdropped: u32,
    max_q: i32,
    max_t: i32,
    mqe: i32,
    mqe_t: i32,
    mte: i32,
    mte_q: i32,
    score: i32,
    m_cigar: i32,
    n_cigar: i32,
    reach_end: i32,
    cigar: Vec<u32>,
}

#[test]
#[ignore = "strict original-C ksw verifier; requires gcc, original C sources, and x86 SSE4.1"]
fn ksw_original_c_conformance_harness() {
    if !cfg!(any(target_arch = "x86", target_arch = "x86_64")) {
        eprintln!("skipping original ksw SSE harness on non-x86 target");
        return;
    }

    let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let exe = std::env::temp_dir().join(format!(
        "minibwa_ksw_conformance_{}_{}",
        std::process::id(),
        option_env!("CARGO_PKG_VERSION").unwrap_or("dev")
    ));
    let output = Command::new("gcc")
        .current_dir(&manifest)
        .args([
            "-std=c99",
            "-O2",
            "-msse4.1",
            "-I",
            "minibwa",
            "tools/ksw_conformance.c",
            "minibwa/ksw2_extz2_sse.c",
            "minibwa/ksw2_extd2_sse.c",
            "-o",
        ])
        .arg(&exe)
        .output()
        .expect("failed to execute gcc for original ksw harness");
    assert!(
        output.status.success(),
        "gcc failed for original ksw harness\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let output = Command::new(&exe)
        .output()
        .expect("failed to run original ksw harness");
    assert!(
        output.status.success(),
        "original ksw harness failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("harness output is not UTF-8");
    let mut total = 0usize;
    let mut mismatches = Vec::new();
    for line in stdout.lines().filter(|line| !line.trim().is_empty()) {
        total += 1;
        let parts = line.split_whitespace().collect::<Vec<_>>();
        assert!(parts.len() >= 27, "short harness line: {line}");
        let n_cigar = parts[25].parse::<i32>().expect("n_cigar");
        assert!(
            parts.len() == 27 + n_cigar.max(0) as usize,
            "bad cigar field count in harness line: {line}"
        );
        let mat_id = parts[2].parse::<i32>().expect("mat_id");
        let qlen = parts[3].parse::<i32>().expect("qlen");
        let tlen = parts[4].parse::<i32>().expect("tlen");
        let q = parts[5].parse::<i8>().expect("q");
        let e = parts[6].parse::<i8>().expect("e");
        let q2 = parts[7].parse::<i8>().expect("q2");
        let e2 = parts[8].parse::<i8>().expect("e2");
        let w = parts[9].parse::<i32>().expect("w");
        let zdrop = parts[10].parse::<i32>().expect("zdrop");
        let end_bonus = parts[11].parse::<i32>().expect("end_bonus");
        let flag = parts[12].parse::<i32>().expect("flag");
        let query = parts[13]
            .bytes()
            .map(|b| {
                assert!((b'0'..=b'4').contains(&b), "bad sequence digit in {line}");
                b - b'0'
            })
            .collect::<Vec<_>>();
        let target = parts[14]
            .bytes()
            .map(|b| {
                assert!((b'0'..=b'4').contains(&b), "bad sequence digit in {line}");
                b - b'0'
            })
            .collect::<Vec<_>>();
        let expected = Observed {
            max: parts[15].parse().expect("max"),
            zdropped: parts[16].parse().expect("zdropped"),
            max_q: parts[17].parse().expect("max_q"),
            max_t: parts[18].parse().expect("max_t"),
            mqe: parts[19].parse().expect("mqe"),
            mqe_t: parts[20].parse().expect("mqe_t"),
            mte: parts[21].parse().expect("mte"),
            mte_q: parts[22].parse().expect("mte_q"),
            score: parts[23].parse().expect("score"),
            m_cigar: parts[24].parse().expect("m_cigar"),
            n_cigar,
            reach_end: parts[26].parse().expect("reach_end"),
            cigar: parts[27..]
                .iter()
                .map(|s| s.parse().expect("cigar"))
                .collect(),
        };
        let mut ez = ksw_extz_t::default();
        let mat = match mat_id {
            0 => &MAT,
            1 => &ALT_MAT,
            2 => &PEAK_MAT,
            _ => panic!("unknown ksw conformance matrix in {line}"),
        };
        match parts[0] {
            "z" => ksw_extz2_sse(
                (),
                qlen,
                &query,
                tlen,
                &target,
                5,
                mat,
                q,
                e,
                w,
                zdrop,
                end_bonus,
                flag,
                &mut ez,
            ),
            "d" => ksw_extd2_sse(
                (),
                qlen,
                &query,
                tlen,
                &target,
                5,
                mat,
                q,
                e,
                q2,
                e2,
                w,
                zdrop,
                end_bonus,
                flag,
                &mut ez,
            ),
            _ => panic!("unknown ksw conformance kind in {line}"),
        }
        let actual = Observed {
            max: ez.max,
            zdropped: ez.zdropped,
            max_q: ez.max_q,
            max_t: ez.max_t,
            mqe: ez.mqe,
            mqe_t: ez.mqe_t,
            mte: ez.mte,
            mte_q: ez.mte_q,
            score: ez.score,
            m_cigar: ez.m_cigar,
            n_cigar: ez.n_cigar,
            reach_end: ez.reach_end,
            cigar: ez.cigar[..ez.n_cigar.max(0) as usize].to_vec(),
        };
        if actual != expected {
            mismatches.push(format!(
                "case line: {}\nexpected: {:?}\nactual:   {:?}",
                line, expected, actual
            ));
        }
    }
    assert!(
        !stdout.trim().is_empty(),
        "original ksw harness produced no cases"
    );
    assert!(
        total >= 1534,
        "original ksw harness produced only {total} cases"
    );
    assert!(
        mismatches.is_empty(),
        "{} / {total} original ksw cases mismatched\n{}",
        mismatches.len(),
        mismatches
            .iter()
            .take(12)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n\n")
    );
}
