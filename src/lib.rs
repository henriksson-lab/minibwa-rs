//! Rust translation of the original minibwa C surface.
//!
//! Source inventory was extracted with `ccc-rs analyze` from the sibling `minibwa/` tree.
//! Vendored `libsais` is provided by the local `libsais-rs` dependency, and vendored
//! `mimalloc` is linked by the CLI for parity with the original C build.
#![allow(
    clippy::absurd_extreme_comparisons,
    clippy::almost_swapped,
    clippy::approx_constant,
    clippy::collapsible_if,
    clippy::drop_non_drop,
    clippy::excessive_precision,
    clippy::field_reassign_with_default,
    clippy::if_same_then_else,
    clippy::int_plus_one,
    clippy::items_after_test_module,
    clippy::manual_div_ceil,
    clippy::manual_is_multiple_of,
    clippy::manual_pattern_char_comparison,
    clippy::manual_range_contains,
    clippy::missing_safety_doc,
    clippy::missing_transmute_annotations,
    clippy::needless_range_loop,
    clippy::needless_return,
    clippy::ptr_arg,
    clippy::too_many_arguments,
    clippy::unnecessary_map_or,
    clippy::unnecessary_cast
)]

pub mod QSufSort {
    #![allow(unused_variables, dead_code, non_snake_case)]

    pub const INSERT_SORT_NUM_ITEM: i64 = 16;

    /// Original C global function `QSufSortSuffixSort` from `minibwa/QSufSort.c:56`.
    pub fn QSufSortSuffixSort(
        V: &mut [i64],
        I: &mut [i64],
        numChar: i64,
        largestInputSymbol: i64,
        smallestInputSymbol: i64,
        skipTransform: i32,
    ) {
        let mut numSortedPos = 1i64;
        if skipTransform == 0 {
            let mut numSymbolAggregated = 0;
            let newAlphabetSize = QSufSortTransform(
                V,
                I,
                numChar,
                largestInputSymbol,
                smallestInputSymbol,
                numChar,
                &mut numSymbolAggregated,
            );
            QSufSortBucketSort(V, I, numChar, newAlphabetSize);
            I[0] = -1;
            V[numChar as usize] = 0;
            numSortedPos = numSymbolAggregated;
        }

        while I[0] >= -numChar {
            let mut i = 0i64;
            let mut negatedSortedGroupLength = 0i64;
            loop {
                let s = I[i as usize];
                if s < 0 {
                    i -= s;
                    negatedSortedGroupLength += s;
                } else {
                    if negatedSortedGroupLength != 0 {
                        I[(i + negatedSortedGroupLength) as usize] = negatedSortedGroupLength;
                        negatedSortedGroupLength = 0;
                    }
                    let j = V[s as usize] + 1;
                    QSufSortSortSplit(V, I, i, j - 1, numSortedPos);
                    i = j;
                }
                if i > numChar {
                    break;
                }
            }
            if negatedSortedGroupLength != 0 {
                I[(i + negatedSortedGroupLength) as usize] = negatedSortedGroupLength;
            }
            numSortedPos *= 2;
        }
    }

    /// Original C global function `QSufSortGenerateSaFromInverse` from `minibwa/QSufSort.c:101`.
    pub fn QSufSortGenerateSaFromInverse(V: &[i64], I: &mut [i64], numChar: i64) {
        for i in 0..=numChar as usize {
            I[V[i] as usize] = i as i64 + 1;
        }
    }

    /// Original C static function `QSufSortSortSplit` from `minibwa/QSufSort.c:113`.
    pub fn QSufSortSortSplit(
        V: &mut [i64],
        I: &mut [i64],
        lowestPos: i64,
        highestPos: i64,
        numSortedChar: i64,
    ) {
        let numItem = highestPos - lowestPos + 1;
        if numItem <= INSERT_SORT_NUM_ITEM {
            QSufSortInsertSortSplit(V, I, lowestPos, highestPos, numSortedChar);
            return;
        }

        let v = QSufSortChoosePivot(V, I, lowestPos, highestPos, numSortedChar);
        let mut a = lowestPos;
        let mut b = lowestPos;
        let mut c = highestPos;
        let mut d = highestPos;

        loop {
            while c >= b && V[(I[b as usize] + numSortedChar) as usize] <= v {
                if V[(I[b as usize] + numSortedChar) as usize] == v {
                    I.swap(a as usize, b as usize);
                    a += 1;
                }
                b += 1;
            }
            while c >= b && V[(I[c as usize] + numSortedChar) as usize] >= v {
                if V[(I[c as usize] + numSortedChar) as usize] == v {
                    I.swap(c as usize, d as usize);
                    d -= 1;
                }
                c -= 1;
            }
            if b > c {
                break;
            }
            I.swap(b as usize, c as usize);
            b += 1;
            c -= 1;
        }

        let mut s = a - lowestPos;
        let mut t = b - a;
        s = s.min(t);
        let mut l = lowestPos;
        let mut m = b - s;
        while m < b {
            I.swap(l as usize, m as usize);
            l += 1;
            m += 1;
        }

        s = d - c;
        t = highestPos - d;
        s = s.min(t);
        l = b;
        m = highestPos - s + 1;
        while m <= highestPos {
            I.swap(l as usize, m as usize);
            l += 1;
            m += 1;
        }

        s = b - a;
        t = d - c;
        if s > 0 {
            QSufSortSortSplit(V, I, lowestPos, lowestPos + s - 1, numSortedChar);
        }

        a = lowestPos + s;
        b = highestPos - t;
        if a == b {
            V[I[a as usize] as usize] = a;
            I[a as usize] = -1;
        } else {
            c = a;
            while c <= b {
                V[I[c as usize] as usize] = b;
                c += 1;
            }
        }

        if t > 0 {
            QSufSortSortSplit(V, I, highestPos - t + 1, highestPos, numSortedChar);
        }
    }

    /// Original C static function `QSufSortChoosePivot` from `minibwa/QSufSort.c:194`.
    pub fn QSufSortChoosePivot(
        V: &[i64],
        I: &[i64],
        lowestPos: i64,
        highestPos: i64,
        numSortedChar: i64,
    ) -> i64 {
        let numItem = highestPos - lowestPos + 1;
        let m = lowestPos + numItem / 2;
        let s = numItem / 8;
        let key1 = V[(I[lowestPos as usize] + numSortedChar) as usize];
        let key2 = V[(I[(lowestPos + s) as usize] + numSortedChar) as usize];
        let key3 = V[(I[(lowestPos + 2 * s) as usize] + numSortedChar) as usize];
        let keyl = if key1 < key2 {
            if key2 < key3 {
                key2
            } else if key1 < key3 {
                key3
            } else {
                key1
            }
        } else if key2 > key3 {
            key2
        } else if key1 > key3 {
            key3
        } else {
            key1
        };
        let key1 = V[(I[(m - s) as usize] + numSortedChar) as usize];
        let key2 = V[(I[m as usize] + numSortedChar) as usize];
        let key3 = V[(I[(m + s) as usize] + numSortedChar) as usize];
        let keym = if key1 < key2 {
            if key2 < key3 {
                key2
            } else if key1 < key3 {
                key3
            } else {
                key1
            }
        } else if key2 > key3 {
            key2
        } else if key1 > key3 {
            key3
        } else {
            key1
        };
        let key1 = V[(I[(highestPos - 2 * s) as usize] + numSortedChar) as usize];
        let key2 = V[(I[(highestPos - s) as usize] + numSortedChar) as usize];
        let key3 = V[(I[highestPos as usize] + numSortedChar) as usize];
        let keyn = if key1 < key2 {
            if key2 < key3 {
                key2
            } else if key1 < key3 {
                key3
            } else {
                key1
            }
        } else if key2 > key3 {
            key2
        } else if key1 > key3 {
            key3
        } else {
            key1
        };
        if keyl < keym {
            if keym < keyn {
                keym
            } else if keyl < keyn {
                keyn
            } else {
                keyl
            }
        } else if keym > keyn {
            keym
        } else if keyl > keyn {
            keyn
        } else {
            keyl
        }
    }

    /// Original C static function `QSufSortInsertSortSplit` from `minibwa/QSufSort.c:227`.
    pub fn QSufSortInsertSortSplit(
        V: &mut [i64],
        I: &mut [i64],
        lowestPos: i64,
        highestPos: i64,
        numSortedChar: i64,
    ) {
        let numItem = highestPos - lowestPos + 1;
        let mut key = vec![0i64; numItem as usize];
        let mut pos = vec![0i64; numItem as usize];
        for i in 0..numItem as usize {
            pos[i] = I[lowestPos as usize + i];
            key[i] = V[(pos[i] + numSortedChar) as usize];
        }
        for i in 1..numItem as usize {
            let tmpKey = key[i];
            let tmpPos = pos[i];
            let mut j = i;
            while j > 0 && key[j - 1] > tmpKey {
                key[j] = key[j - 1];
                pos[j] = pos[j - 1];
                j -= 1;
            }
            key[j] = tmpKey;
            pos[j] = tmpPos;
        }

        let mut negativeSortedLength = -1i64;
        let mut i = numItem - 1;
        let mut groupNum = highestPos;
        while i > 0 {
            I[(i + lowestPos) as usize] = pos[i as usize];
            V[I[(i + lowestPos) as usize] as usize] = groupNum;
            if key[(i - 1) as usize] == key[i as usize] {
                negativeSortedLength = 0;
            } else {
                if negativeSortedLength < 0 {
                    I[(i + lowestPos) as usize] = negativeSortedLength;
                }
                groupNum = i + lowestPos - 1;
                negativeSortedLength -= 1;
            }
            i -= 1;
        }
        I[lowestPos as usize] = pos[0];
        V[I[lowestPos as usize] as usize] = groupNum;
        if negativeSortedLength < 0 {
            I[lowestPos as usize] = negativeSortedLength;
        }
    }

    /// Original C static function `QSufSortBucketSort` from `minibwa/QSufSort.c:288`.
    pub fn QSufSortBucketSort(V: &mut [i64], I: &mut [i64], numChar: i64, alphabetSize: i64) {
        for i in 0..alphabetSize as usize {
            I[i] = -1;
        }
        for i in 0..=numChar as usize {
            let c = V[i] as usize;
            V[i] = I[c];
            I[c] = i as i64;
        }
        let mut currentIndex = numChar;
        let mut i = alphabetSize;
        while i > 0 {
            let mut c = I[(i - 1) as usize];
            let mut d = V[c as usize];
            let groupNum = currentIndex;
            V[c as usize] = groupNum;
            if d >= 0 {
                I[currentIndex as usize] = c;
                while d >= 0 {
                    c = d;
                    d = V[c as usize];
                    V[c as usize] = groupNum;
                    currentIndex -= 1;
                    I[currentIndex as usize] = c;
                }
            } else {
                I[currentIndex as usize] = -1;
            }
            currentIndex -= 1;
            i -= 1;
        }
    }

    /// Original C static function `QSufSortTransform` from `minibwa/QSufSort.c:344`.
    pub fn QSufSortTransform(
        V: &mut [i64],
        I: &mut [i64],
        numChar: i64,
        largestInputSymbol: i64,
        smallestInputSymbol: i64,
        maxNewAlphabetSize: i64,
        numSymbolAggregated: &mut i64,
    ) -> i64 {
        let maxNumInputSymbol = largestInputSymbol - smallestInputSymbol + 1;
        let mut maxNumBit = 0i64;
        let mut i = maxNumInputSymbol;
        while i != 0 {
            maxNumBit += 1;
            i >>= 1;
        }
        let maxSymbol = i64::MAX >> maxNumBit;
        let mut c = maxNumInputSymbol;
        let mut a = 0i64;
        let mut minSymbolInChunk = 0i64;
        let mut maxSymbolInChunk = 0i64;
        while a < numChar && maxSymbolInChunk <= maxSymbol && c <= maxNewAlphabetSize {
            minSymbolInChunk =
                (minSymbolInChunk << maxNumBit) | (V[a as usize] - smallestInputSymbol + 1);
            maxSymbolInChunk = c;
            c = (maxSymbolInChunk << maxNumBit) | maxNumInputSymbol;
            a += 1;
        }

        let mask = (1i64 << ((a - 1) * maxNumBit)) - 1;
        V[numChar as usize] = smallestInputSymbol - 1;
        for i in 0..=maxSymbolInChunk as usize {
            I[i] = 0;
        }
        c = minSymbolInChunk;
        i = a;
        while i <= numChar {
            I[c as usize] = 1;
            c = ((c & mask) << maxNumBit) | (V[i as usize] - smallestInputSymbol + 1);
            i += 1;
        }
        i = 1;
        while i < a {
            I[c as usize] = 1;
            c = (c & mask) << maxNumBit;
            i += 1;
        }
        let mut newAlphabetSize = 1i64;
        for i in 0..=maxSymbolInChunk as usize {
            if I[i] != 0 {
                I[i] = newAlphabetSize;
                newAlphabetSize += 1;
            }
        }
        c = minSymbolInChunk;
        i = 0;
        let mut j = a;
        while j <= numChar {
            V[i as usize] = I[c as usize];
            c = ((c & mask) << maxNumBit) | (V[j as usize] - smallestInputSymbol + 1);
            i += 1;
            j += 1;
        }
        while i < numChar {
            V[i as usize] = I[c as usize];
            c = (c & mask) << maxNumBit;
            i += 1;
        }
        V[numChar as usize] = 0;
        *numSymbolAggregated = a;
        newAlphabetSize
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn generate_sa_from_inverse_follows_original_one_based_output() {
            let v = vec![2, 0, 3, 1];
            let mut i = vec![0; 4];
            QSufSortGenerateSaFromInverse(&v, &mut i, 3);
            assert_eq!(i, vec![2, 4, 1, 3]);
        }

        #[test]
        fn suffix_sort_matches_naive_suffix_order_for_small_text() {
            let text = [2, 1, 3, 1, 3, 1];
            let n = text.len() as i64;
            let mut v = vec![0i64; text.len() + 1];
            let mut i = vec![0i64; text.len() + 1];
            for (k, &b) in text.iter().enumerate() {
                v[k] = b;
            }
            QSufSortSuffixSort(&mut v, &mut i, n, 3, 1, 0);
            QSufSortGenerateSaFromInverse(&v, &mut i, n);
            let got: Vec<usize> = i.iter().map(|&x| x as usize - 1).collect();
            let mut expected: Vec<usize> = (0..=text.len()).collect();
            expected.sort_by(|&a, &b| text[a..].cmp(&text[b..]));
            assert_eq!(got, expected);
        }
    }
}

#[cfg(test)]
mod ksw_original_c_conformance {
    use std::process::Command;

    use crate::ksw2::ksw_extz_t;
    use crate::ksw2_extd2_sse::ksw_extd2_sse;
    use crate::ksw2_extz2_sse::ksw_extz2_sse;

    const MAT: [i8; 25] = [
        2, -4, -4, -4, -1, -4, 2, -4, -4, -1, -4, -4, 2, -4, -1, -4, -4, -4, 2, -1, -1, -1, -1, -1,
        -1,
    ];
    const ALT_MAT: [i8; 25] = [
        3, -2, -2, -2, -2, -2, 3, -2, -2, -2, -2, -2, 3, -2, -2, -2, -2, -2, 3, -2, -2, -2, -2, -2,
        -2,
    ];
    const PEAK_MAT: [i8; 25] = [
        7, -3, -3, -3, -2, -3, 7, -3, -3, -2, -3, -3, 7, -3, -2, -3, -3, -3, 7, -2, -2, -2, -2, -2,
        -2,
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
}

#[cfg(test)]
mod ksw_ll_original_c_conformance {
    use std::process::Command;

    use crate::ksw2_ll_sse::{ksw_ll_i16_core, ksw_ll_qinit, ksw_ll_u8_core, ksw_llrst_t};

    const MAT: [i8; 25] = [
        2, -4, -4, -4, -1, -4, 2, -4, -4, -1, -4, -4, 2, -4, -1, -4, -4, -4, 2, -1, -1, -1, -1, -1,
        -1,
    ];
    const ALT_MAT: [i8; 25] = [
        3, -2, -2, -2, -2, -2, 3, -2, -2, -2, -2, -2, 3, -2, -2, -2, -2, -2, 3, -2, -2, -2, -2, -2,
        -2,
    ];
    const PEAK_MAT: [i8; 25] = [
        7, -3, -3, -3, -2, -3, 7, -3, -3, -2, -3, -3, 7, -3, -2, -3, -3, -3, 7, -2, -2, -2, -2, -2,
        -2,
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
}

#[cfg(test)]
mod ksw_helpers_original_c_conformance {
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
}

pub mod align {
    #![allow(unused_variables, dead_code, non_snake_case)]

    use crate::cs::{mb_write_MD, mb_write_cs_ds};
    use crate::ksw2::{
        ksw_extz_t, ksw_push_cigar, ksw_reset_extz, KSW_EZ_APPROX_MAX, KSW_EZ_EXTZ_ONLY,
        KSW_EZ_GENERIC_SC, KSW_EZ_REV_CIGAR, KSW_EZ_RIGHT,
    };
    use crate::ksw2_extd2_sse::ksw_extd2_sse;
    use crate::ksw2_extz2_sse::ksw_extz2_sse;
    use crate::ksw2_ll_sse::{ksw_ll_i16, ksw_ll_qinit};
    use crate::l2bit::{l2b_getseq_meth, l2b_meth_rev, l2b_meth_t};
    use crate::lchain::mb_anchor_t;
    use crate::map_algo::{
        mb_cal_mblen, mb_filter_hits, mb_hit_sort, mb_idx_t, mb_split_hit, mb_squeeze_a,
        MB_PARENT_TMP_PRI, MB_PARENT_UNSET,
    };
    use crate::mbpriv::{
        mb_is_sr_mode, mb_log2, mb_seq_rev, KOM_DBG_FLAG, MB_DBG_ALN_SEQ, MB_DBG_AN_POS,
    };
    use crate::options::mb_opt_t;
    use crate::pe::mb_hit_t;
    use std::sync::atomic::Ordering;

    pub const MB_CIGAR_MATCH: u32 = 0;
    pub const MB_CIGAR_INS: u32 = 1;
    pub const MB_CIGAR_DEL: u32 = 2;
    pub const MB_CIGAR_N_SKIP: u32 = 3;
    pub const MB_CIGAR_SOFTCLIP: u32 = 4;
    pub const MB_CIGAR_HARDCLIP: u32 = 5;
    pub const MB_CIGAR_PADDING: u32 = 6;
    pub const MB_CIGAR_EQ_MATCH: u32 = 7;
    pub const MB_CIGAR_X_MISMATCH: u32 = 8;
    pub const MB_CIGAR_STR: &str = "MIDNSHP=XB";

    pub const MB_SEED_LONG_JOIN: u32 = 0x1;
    pub const MB_SEED_IGNORE: u32 = 0x2;

    /// Original C static function `update_max_zdrop` from `minibwa/align.c:11`.
    pub fn update_max_zdrop(
        score: i32,
        i: i32,
        j: i32,
        max: &mut i32,
        max_i: &mut i32,
        max_j: &mut i32,
        e: i32,
        max_zdrop: &mut i32,
        pos: &mut [[i32; 2]; 2],
    ) {
        if score < *max {
            let li = i - *max_i;
            let lj = j - *max_j;
            let diff = if li > lj { li - lj } else { lj - li };
            let z = *max - score - diff * e;
            if z > *max_zdrop {
                *max_zdrop = z;
                pos[0][0] = *max_i;
                pos[0][1] = i;
                pos[1][0] = *max_j;
                pos[1][1] = j;
            }
        } else {
            *max = score;
            *max_i = i;
            *max_j = j;
        }
    }

    /// Original C static function `mm_test_zdrop` from `minibwa/align.c:26`.
    pub fn mm_test_zdrop(
        km: (),
        opt: &mb_opt_t,
        qseq: &[u8],
        tseq: &[u8],
        n_cigar: u32,
        cigar: &[u32],
        mat: &[i8],
        is_sr: i32,
    ) -> i32 {
        let mut score = 0i32;
        let mut max = i32::MIN;
        let mut max_i = -1i32;
        let mut max_j = -1i32;
        let mut i = 0i32;
        let mut j = 0i32;
        let mut max_zdrop = 0i32;
        let mut pos = [[-1i32, -1i32], [-1i32, -1i32]];
        for &cg in cigar.iter().take(n_cigar as usize) {
            let op = cg & 0xf;
            let len = cg >> 4;
            if op == MB_CIGAR_MATCH {
                for l in 0..len as i32 {
                    score += mat
                        [(tseq[(i + l) as usize] as usize) * 5 + qseq[(j + l) as usize] as usize]
                        as i32;
                    update_max_zdrop(
                        score,
                        i + l,
                        j + l,
                        &mut max,
                        &mut max_i,
                        &mut max_j,
                        opt.e,
                        &mut max_zdrop,
                        &mut pos,
                    );
                }
                i += len as i32;
                j += len as i32;
            } else if op == MB_CIGAR_INS || op == MB_CIGAR_DEL || op == MB_CIGAR_N_SKIP {
                score -= opt.q + opt.e * len as i32;
                if op == MB_CIGAR_INS {
                    j += len as i32;
                } else {
                    i += len as i32;
                }
                update_max_zdrop(
                    score,
                    i,
                    j,
                    &mut max,
                    &mut max_i,
                    &mut max_j,
                    opt.e,
                    &mut max_zdrop,
                    &mut pos,
                );
            }
        }
        let q_len = pos[1][1] - pos[1][0];
        let t_len = pos[0][1] - pos[0][0];
        if is_sr == 0 && max_zdrop > opt.zdrop_inv && q_len < opt.max_gap && t_len < opt.max_gap {
            let mut qseq2 = vec![0u8; q_len as usize];
            for i in 0..q_len as usize {
                let c = qseq[pos[1][1] as usize - i - 1];
                qseq2[i] = if c >= 4 { 4 } else { 3 - c };
            }
            let mut q_off = 0;
            let mut t_off = 0;
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            {
                if let Some(c_score) = crate::ksw2_c_sse::maybe_ll_i16(
                    q_len,
                    &qseq2,
                    mat,
                    t_len,
                    &tseq[pos[0][0] as usize..],
                    opt.q,
                    opt.e,
                    &mut q_off,
                    &mut t_off,
                ) {
                    score = c_score;
                } else {
                    let qp = ksw_ll_qinit(km, 2, q_len, &qseq2, 5, mat);
                    score = ksw_ll_i16(
                        &qp,
                        t_len,
                        &tseq[pos[0][0] as usize..],
                        opt.q,
                        opt.e,
                        &mut q_off,
                        &mut t_off,
                    );
                }
            }
            #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
            {
                let qp = ksw_ll_qinit(km, 2, q_len, &qseq2, 5, mat);
                score = ksw_ll_i16(
                    &qp,
                    t_len,
                    &tseq[pos[0][0] as usize..],
                    opt.q,
                    opt.e,
                    &mut q_off,
                    &mut t_off,
                );
            }
            if score >= opt.min_chain_score * opt.a && score >= opt.min_dp_max * opt.a {
                return 2;
            }
        }
        (max_zdrop > opt.zdrop) as i32
    }

    /// Original C static function `mb_fix_cigar` from `minibwa/align.c:70`.
    pub fn mb_fix_cigar(
        r: &mut mb_hit_t,
        qseq: &[u8],
        tseq: &[u8],
        qshift: &mut i32,
        tshift: &mut i32,
    ) {
        *qshift = 0;
        *tshift = 0;
        let r_rev = r.rev();
        let Some(p) = r.p.as_mut() else {
            return;
        };
        let mut toff = 0i32;
        let mut qoff = 0i32;
        let mut to_shrink = 0;
        if p.n_cigar <= 1 {
            return;
        }
        for k in 0..p.n_cigar as usize {
            let op = p.cigar()[k] & 0xf;
            let len = p.cigar()[k] >> 4;
            if len == 0 {
                to_shrink = 1;
            }
            if op == MB_CIGAR_MATCH {
                toff += len as i32;
                qoff += len as i32;
            } else if op == MB_CIGAR_INS || op == MB_CIGAR_DEL {
                if k > 0
                    && k < p.n_cigar as usize - 1
                    && (p.cigar()[k - 1] & 0xf) == MB_CIGAR_MATCH
                    && (p.cigar()[k + 1] & 0xf) == MB_CIGAR_MATCH
                {
                    let prev_len = p.cigar()[k - 1] >> 4;
                    let mut l = 0u32;
                    if op == MB_CIGAR_INS {
                        while l < prev_len
                            && qseq[(qoff - 1 - l as i32) as usize]
                                == qseq[(qoff + len as i32 - 1 - l as i32) as usize]
                        {
                            l += 1;
                        }
                    } else {
                        while l < prev_len
                            && tseq[(toff - 1 - l as i32) as usize]
                                == tseq[(toff + len as i32 - 1 - l as i32) as usize]
                        {
                            l += 1;
                        }
                    }
                    if l > 0 {
                        p.cigar_mut()[k - 1] -= l << 4;
                        p.cigar_mut()[k + 1] += l << 4;
                        qoff -= l as i32;
                        toff -= l as i32;
                    }
                    if l == prev_len {
                        to_shrink = 1;
                    }
                }
                if op == MB_CIGAR_INS {
                    qoff += len as i32;
                } else {
                    toff += len as i32;
                }
            } else if op == MB_CIGAR_N_SKIP {
                toff += len as i32;
            }
        }
        assert_eq!(qoff, r.qe - r.qs);
        assert_eq!(toff, (r.te - r.ts) as i32);
        let mut k = 0usize;
        while k + 2 < p.n_cigar as usize {
            if (p.cigar()[k] & 0xf) > 0 && (p.cigar()[k] & 0xf) + (p.cigar()[k + 1] & 0xf) == 3 {
                let mut s = [0u32; 3];
                let mut l = k;
                while l < p.n_cigar as usize {
                    let op = p.cigar()[l] & 0xf;
                    if op == MB_CIGAR_INS || op == MB_CIGAR_DEL || p.cigar()[l] >> 4 == 0 {
                        s[op as usize] += p.cigar()[l] >> 4;
                    } else {
                        break;
                    }
                    l += 1;
                }
                if s[1] > 0 && s[2] > 0 && l - k > 2 {
                    p.cigar_mut()[k] = s[1] << 4 | MB_CIGAR_INS;
                    p.cigar_mut()[k + 1] = s[2] << 4 | MB_CIGAR_DEL;
                    let mut kk = k + 2;
                    while kk < l {
                        p.cigar_mut()[kk] &= 0xf;
                        kk += 1;
                    }
                    to_shrink = 1;
                }
                k = l;
            }
            k += 1;
        }
        if to_shrink != 0 {
            let mut new_cigar = Vec::new();
            for &cg in p.cigar().iter().take(p.n_cigar as usize) {
                if cg >> 4 != 0 {
                    if let Some(last) = new_cigar.last_mut() {
                        if (*last & 0xf) == (cg & 0xf) {
                            *last += (cg >> 4) << 4;
                            continue;
                        }
                    }
                    new_cigar.push(cg);
                }
            }
            p.set_cigar_from_vec(new_cigar);
            p.cap = p.cap.max(p.n_cigar as u32);
        }
        if p.n_cigar > 0
            && ((p.cigar()[0] & 0xf) == MB_CIGAR_INS || (p.cigar()[0] & 0xf) == MB_CIGAR_DEL)
        {
            let l = (p.cigar()[0] >> 4) as i32;
            if (p.cigar()[0] & 0xf) == MB_CIGAR_INS {
                if r_rev != 0 {
                    r.qe -= l;
                } else {
                    r.qs += l;
                }
                *qshift = l;
            } else {
                r.ts += l as i64;
                *tshift = l;
            }
            p.remove_cigar(0);
        }
    }

    /// Original C static function `mm_update_cigar_eqx` from `minibwa/align.c:148`.
    pub fn mm_update_cigar_eqx(r: &mut mb_hit_t, qseq: &[u8], tseq: &[u8]) {
        let Some(p0) = r.p.as_ref() else {
            return;
        };
        let mut n_eqx = 0u32;
        let mut n_m = 0u32;
        let mut toff = 0usize;
        let mut qoff = 0usize;
        for &cg in p0.cigar().iter().take(p0.n_cigar as usize) {
            let op = cg & 0xf;
            let mut len = (cg >> 4) as usize;
            if op == MB_CIGAR_MATCH {
                while len > 0 {
                    let mut l = 0usize;
                    while l < len && qseq[qoff + l] == tseq[toff + l] {
                        l += 1;
                    }
                    if l > 0 {
                        n_eqx += 1;
                        len -= l;
                        toff += l;
                        qoff += l;
                    }
                    l = 0;
                    while l < len && qseq[qoff + l] != tseq[toff + l] {
                        l += 1;
                    }
                    if l > 0 {
                        n_eqx += 1;
                        len -= l;
                        toff += l;
                        qoff += l;
                    }
                }
                n_m += 1;
            } else if op == MB_CIGAR_INS {
                qoff += len;
            } else if op == MB_CIGAR_DEL || op == MB_CIGAR_N_SKIP {
                toff += len;
            }
        }
        let p = r.p.as_mut().unwrap();
        if n_eqx == n_m {
            let n_cigar = p.n_cigar as usize;
            for cg in p.cigar_mut().iter_mut().take(n_cigar) {
                let op = *cg & 0xf;
                let len = *cg >> 4;
                if op == MB_CIGAR_MATCH {
                    *cg = len << 4 | MB_CIGAR_EQ_MATCH;
                }
            }
            return;
        }
        let old = p.cigar()[..p.n_cigar as usize].to_vec();
        let mut new_cigar = Vec::new();
        toff = 0;
        qoff = 0;
        for cg in old {
            let op = cg & 0xf;
            let mut len = (cg >> 4) as usize;
            if op == MB_CIGAR_MATCH {
                while len > 0 {
                    let mut l = 0usize;
                    while l < len && qseq[qoff + l] == tseq[toff + l] {
                        l += 1;
                    }
                    if l > 0 {
                        new_cigar.push((l as u32) << 4 | MB_CIGAR_EQ_MATCH);
                    }
                    len -= l;
                    toff += l;
                    qoff += l;
                    l = 0;
                    while l < len && qseq[qoff + l] != tseq[toff + l] {
                        l += 1;
                    }
                    if l > 0 {
                        new_cigar.push((l as u32) << 4 | MB_CIGAR_X_MISMATCH);
                    }
                    len -= l;
                    toff += l;
                    qoff += l;
                }
            } else {
                if op == MB_CIGAR_INS {
                    qoff += len;
                } else if op == MB_CIGAR_DEL || op == MB_CIGAR_N_SKIP {
                    toff += len;
                }
                new_cigar.push(cg);
            }
        }
        p.set_cigar_from_vec(new_cigar);
        p.cap = p.cap.max(p.n_cigar as u32);
    }

    /// Original C global function `mb_update_extra` from `minibwa/align.c:218`.
    pub fn mb_update_extra(
        km: (),
        r: &mut mb_hit_t,
        qseq: &[u8],
        tseq: &[u8],
        mat: &[i8],
        q: i8,
        e: i8,
        opt_flag: u64,
        log_gap: i32,
    ) {
        if r.p.is_none() {
            return;
        }
        let mut qshift = 0;
        let mut tshift = 0;
        mb_fix_cigar(r, qseq, tseq, &mut qshift, &mut tshift);
        let qseq = &qseq[qshift as usize..];
        let tseq = &tseq[tshift as usize..];
        let mut toff = 0usize;
        let mut qoff = 0usize;
        let mut s = 0.0f64;
        let mut max = 0.0f64;
        r.blen = 0;
        r.mlen = 0;
        {
            let p = r.p.as_mut().unwrap();
            let mut total_n_ambi = 0u32;
            let n_cigar = p.n_cigar.max(0) as usize;
            let cigar = p.cigar().as_ptr();
            for k in 0..n_cigar {
                let cg = unsafe { *cigar.add(k) };
                let op = cg & 0xf;
                let len = (cg >> 4) as usize;
                if op == MB_CIGAR_MATCH {
                    let mut n_ambi = 0i32;
                    let mut n_diff = 0i32;
                    let qptr = unsafe { qseq.as_ptr().add(qoff) };
                    let tptr = unsafe { tseq.as_ptr().add(toff) };
                    let mat_ptr = mat.as_ptr();
                    for l in 0..len {
                        let cq = unsafe { *qptr.add(l) } as usize;
                        let ct = unsafe { *tptr.add(l) } as usize;
                        if ct > 3 || cq > 3 {
                            n_ambi += 1;
                        } else if ct != cq {
                            n_diff += 1;
                        }
                        s += unsafe { *mat_ptr.add(ct * 5 + cq) } as f64;
                        if s < 0.0 {
                            s = 0.0;
                        } else if max < s {
                            max = s;
                        }
                    }
                    r.blen += len as i32 - n_ambi;
                    r.mlen += len as i32 - (n_ambi + n_diff);
                    total_n_ambi += n_ambi as u32;
                    toff += len;
                    qoff += len;
                } else if op == MB_CIGAR_INS {
                    let mut n_ambi = 0i32;
                    let qptr = unsafe { qseq.as_ptr().add(qoff) };
                    for l in 0..len {
                        if unsafe { *qptr.add(l) } > 3 {
                            n_ambi += 1;
                        }
                    }
                    r.blen += len as i32 - n_ambi;
                    total_n_ambi += n_ambi as u32;
                    if log_gap != 0 {
                        s -= q as f64 + e as f64 * mb_log2(1.0 + len as f32) as f64;
                    } else {
                        s -= q as f64 + e as f64;
                    }
                    if s < 0.0 {
                        s = 0.0;
                    }
                    qoff += len;
                } else if op == MB_CIGAR_DEL {
                    let mut n_ambi = 0i32;
                    let tptr = unsafe { tseq.as_ptr().add(toff) };
                    for l in 0..len {
                        if unsafe { *tptr.add(l) } > 3 {
                            n_ambi += 1;
                        }
                    }
                    r.blen += len as i32 - n_ambi;
                    total_n_ambi += n_ambi as u32;
                    if log_gap != 0 {
                        s -= q as f64 + e as f64 * mb_log2(1.0 + len as f32) as f64;
                    } else {
                        s -= q as f64 + e as f64;
                    }
                    if s < 0.0 {
                        s = 0.0;
                    }
                    toff += len;
                }
            }
            p.set_n_ambi(total_n_ambi);
            p.dp_max0 = (max + 0.499) as i32;
            p.dp_max = p.dp_max0;
        }
        assert_eq!(qoff as i32, r.qe - r.qs);
        assert_eq!(toff as i32, (r.te - r.ts) as i32);
        if (opt_flag & crate::options::MB_F_EQX) != 0 {
            mm_update_cigar_eqx(r, qseq, tseq);
        }
        if (opt_flag
            & (crate::options::MB_F_WRITE_DS
                | crate::options::MB_F_WRITE_CS
                | crate::options::MB_F_WRITE_MD))
            != 0
        {
            let mut s = crate::kommon::kstring_t {
                m: 256,
                s: vec![0; 256],
                ..Default::default()
            };
            if (opt_flag & (crate::options::MB_F_WRITE_DS | crate::options::MB_F_WRITE_CS)) != 0 {
                mb_write_cs_ds(
                    km,
                    &mut s,
                    tseq,
                    qseq,
                    r,
                    ((opt_flag & crate::options::MB_F_WRITE_DS) != 0) as i32,
                );
            } else {
                mb_write_MD(km, &mut s, tseq, qseq, r);
            }
            let p = r.p.as_mut().unwrap();
            p.set_cs(1);
            p.truncate_cigar(p.n_cigar as usize);
            let mut tag_words = Vec::with_capacity((s.l + 1).div_ceil(4));
            for chunk in s.s[..s.l + 1].chunks(4) {
                let mut w = 0u32;
                for (i, &b) in chunk.iter().enumerate() {
                    w |= (b as u32) << (i * 8);
                }
                tag_words.push(w);
            }
            p.set_tag_words_from_slice(&tag_words);
        }
    }

    /// Original C static function `mb_enlarge_cigar` from `minibwa/align.c:284`.
    pub fn mb_enlarge_cigar(r: &mut mb_hit_t, n_cigar: u32) {
        const MB_EXTRA_WORDS: u32 = (std::mem::size_of::<crate::pe::mb_extra_t>() / 4) as u32;
        fn cigar_capacity(n_cigar: u32) -> u32 {
            let mut cap = n_cigar.saturating_add(MB_EXTRA_WORDS).max(1);
            cap -= 1;
            cap |= cap >> 1;
            cap |= cap >> 2;
            cap |= cap >> 4;
            cap |= cap >> 8;
            cap |= cap >> 16;
            cap += 1;
            cap.saturating_sub(MB_EXTRA_WORDS).max(n_cigar)
        }
        if n_cigar == 0 {
            return;
        }
        if r.p.is_none() {
            let cap = cigar_capacity(n_cigar);
            r.p = Some(
                crate::pe::mb_extra_t {
                    cap,
                    ..Default::default()
                }
                .boxed(),
            );
        } else {
            let p = r.p.as_mut().unwrap();
            let needed = p.n_cigar as u32 + n_cigar;
            if needed > p.cap {
                let cap = cigar_capacity(needed);
                p.ensure_capacity(cap);
            }
        }
    }

    /// Original C global function `mb_append_cigar` from `minibwa/align.c:299`.
    pub fn mb_append_cigar(r: &mut mb_hit_t, n_cigar: u32, cigar: &[u32]) {
        if n_cigar == 0 {
            return;
        }
        mb_enlarge_cigar(r, n_cigar);
        let p = r.p.as_mut().unwrap();
        let n = n_cigar as usize;
        if p.n_cigar > 0 && (p.cigar()[p.n_cigar as usize - 1] & 0xf) == (cigar[0] & 0xf) {
            let last = p.n_cigar as usize - 1;
            p.cigar_mut()[last] += (cigar[0] >> 4) << 4;
            if n > 1 {
                p.extend_cigar_from_slice(&cigar[1..n]);
            }
        } else {
            p.extend_cigar_from_slice(&cigar[..n]);
        }
    }

    /// Original C static function `mb_min_int32` from `minibwa/align.c:315`.
    pub fn mb_min_int32(a: i32, b: i32) -> i32 {
        if a < b {
            a
        } else {
            b
        }
    }

    /// Original C static function `max_bw_from_mm` from `minibwa/align.c:320`.
    pub fn max_bw_from_mm(opt: &mb_opt_t, mm: i32) -> i32 {
        let x = mm * (opt.a + opt.b);
        let mut max2 = 0;
        let mut max1 = 0;
        if x >= opt.q + opt.e {
            max1 = (x - opt.q + opt.e - 1) / opt.e;
        }
        if x >= opt.q2 + opt.e2 {
            max2 = (x - opt.q2 + opt.e2 - 1) / opt.e2;
        }
        if max1 > max2 {
            max1
        } else {
            max2
        }
    }

    /// Original C static function `mb_align_pair` from `minibwa/align.c:328`.
    pub fn mb_align_pair(
        km: (),
        opt: &mb_opt_t,
        qlen: i32,
        qseq: &[u8],
        tlen: i32,
        tseq: &[u8],
        mat: &[i8],
        mut w: i32,
        end_bonus: i32,
        zdrop: i32,
        mut ksw_flag: i32,
        ez: &mut ksw_extz_t,
    ) {
        const MAX_BW_ADJ_LEN: i32 = 100;
        let mut n_mm = -1;
        if opt.b_ts != 0 && opt.b != opt.b_ts {
            ksw_flag |= KSW_EZ_GENERIC_SC;
        } else if (ksw_flag & KSW_EZ_EXTZ_ONLY) != 0 && tlen >= qlen {
            ksw_reset_extz(ez);
            ez.score = 0;
            ez.max = 0;
            for j in 0..qlen as usize {
                if qseq[j] >= 4 || tseq[j] >= 4 {
                    ez.score -= opt.b_ambi;
                    n_mm += 1;
                } else {
                    ez.score += if qseq[j] == tseq[j] { opt.a } else { -opt.b };
                    n_mm += (qseq[j] != tseq[j]) as i32;
                }
                if (ez.max as i32) < ez.score {
                    ez.max = ez.score as u32;
                    ez.max_q = j as i32;
                    ez.max_t = j as i32;
                }
            }
            if n_mm <= 2 {
                ez.mqe = ez.score;
                ez.mqe_t = qlen - 1;
                if ez.mqe + end_bonus >= ez.max as i32 {
                    ez.reach_end = 1;
                    ksw_push_cigar(
                        km,
                        &mut ez.n_cigar,
                        &mut ez.m_cigar,
                        &mut ez.cigar,
                        MB_CIGAR_MATCH,
                        qlen,
                    );
                    return;
                }
            }
        } else if qlen == tlen && (ksw_flag & KSW_EZ_EXTZ_ONLY) == 0 {
            let max_gapped_score = (qlen - 2) * opt.a - 2 * (opt.q + opt.e);
            ksw_reset_extz(ez);
            ez.score = 0;
            for j in 0..qlen as usize {
                if qseq[j] >= 4 || tseq[j] >= 4 {
                    ez.score -= opt.b_ambi;
                    n_mm += 1;
                } else {
                    ez.score += if qseq[j] == tseq[j] { opt.a } else { -opt.b };
                    n_mm += (qseq[j] != tseq[j]) as i32;
                }
            }
            if n_mm <= 3 || ez.score > max_gapped_score {
                ksw_push_cigar(
                    km,
                    &mut ez.n_cigar,
                    &mut ez.m_cigar,
                    &mut ez.cigar,
                    MB_CIGAR_MATCH,
                    qlen,
                );
                return;
            }
        }

        if n_mm >= 0 && mb_min_int32(qlen, tlen) < MAX_BW_ADJ_LEN {
            let max_bw = max_bw_from_mm(opt, n_mm);
            if w > max_bw + 4 {
                w = max_bw + 4;
            }
        }
        if opt.max_sw_mat > 0 && (tlen as i64) * (qlen as i64) > opt.max_sw_mat {
            ksw_reset_extz(ez);
            ez.zdropped = 1;
        } else if opt.q == opt.q2 && opt.e == opt.e2 {
            ksw_extz2_sse(
                km,
                qlen,
                qseq,
                tlen,
                tseq,
                5,
                mat,
                opt.q as i8,
                opt.e as i8,
                w,
                zdrop * opt.a,
                end_bonus,
                ksw_flag,
                ez,
            );
        } else {
            ksw_extd2_sse(
                km,
                qlen,
                qseq,
                tlen,
                tseq,
                5,
                mat,
                opt.q as i8,
                opt.e as i8,
                opt.q2 as i8,
                opt.e2 as i8,
                w,
                zdrop * opt.a,
                end_bonus,
                ksw_flag,
                ez,
            );
        }
        if (KOM_DBG_FLAG.load(Ordering::Relaxed) & MB_DBG_ALN_SEQ) != 0 {
            eprintln!(
                "===> q=({},{}), e=({},{}), bw={}, ksw_flag=0x{:x}, zdrop={}, end_bonus={} <===",
                opt.q, opt.q2, opt.e, opt.e2, w, ksw_flag, opt.zdrop, end_bonus
            );
            let alphabet = b"ACGTN";
            eprintln!(
                "{}",
                tseq.iter()
                    .take(tlen.max(0) as usize)
                    .map(|&c| alphabet[c.min(4) as usize] as char)
                    .collect::<String>()
            );
            eprintln!(
                "{}",
                qseq.iter()
                    .take(qlen.max(0) as usize)
                    .map(|&c| alphabet[c.min(4) as usize] as char)
                    .collect::<String>()
            );
            let cigar = ez
                .cigar
                .iter()
                .take(ez.n_cigar as usize)
                .map(|&c| {
                    format!(
                        "{}{}",
                        c >> 4,
                        MB_CIGAR_STR.as_bytes()[(c & 0xf) as usize] as char
                    )
                })
                .collect::<String>();
            eprintln!("score={}, max={}, cigar={}", ez.score, ez.max, cigar);
        }
    }

    /// Original C static function `collect_long_gaps` from `minibwa/align.c:390`.
    pub fn collect_long_gaps(
        km: (),
        as1: i32,
        cnt1: i32,
        a: &[mb_anchor_t],
        min_gap: i32,
        n_: &mut i32,
    ) -> Vec<i32> {
        *n_ = 0;
        let mut n = 0;
        for i in 1..cnt1 {
            let idx = (as1 + i) as usize;
            let prev = (as1 + i - 1) as usize;
            let gap = (a[idx].qpos - a[prev].qpos) as i64 - (a[idx].tpos - a[prev].tpos);
            if gap < -(min_gap as i64) || gap > min_gap as i64 {
                n += 1;
            }
        }
        if n <= 1 {
            return Vec::new();
        }
        let mut kvec = Vec::with_capacity(n as usize);
        for i in 1..cnt1 {
            let idx = (as1 + i) as usize;
            let prev = (as1 + i - 1) as usize;
            let gap = (a[idx].qpos - a[prev].qpos) as i64 - (a[idx].tpos - a[prev].tpos);
            if gap < -(min_gap as i64) || gap > min_gap as i64 {
                kvec.push(i);
            }
        }
        *n_ = kvec.len() as i32;
        kvec
    }

    /// Original C static function `mm_filter_bad_seeds` from `minibwa/align.c:409`.
    pub fn mm_filter_bad_seeds(
        km: (),
        as1: i32,
        cnt1: i32,
        a: &mut [mb_anchor_t],
        min_gap: i32,
        diff_thres: i32,
        max_ext_len: i32,
        max_ext_cnt: i32,
    ) {
        let mut n = 0;
        let kvec = collect_long_gaps(km, as1, cnt1, a, min_gap, &mut n);
        if kvec.is_empty() {
            return;
        }
        let mut max = 0;
        let mut max_st = -1;
        let mut max_en = -1;
        let mut k = 0i32;
        loop {
            if k == n || k >= max_en {
                if max_en > 0 {
                    for i in kvec[max_st as usize]..kvec[max_en as usize] {
                        a[(as1 + i) as usize].flag |= MB_SEED_IGNORE;
                    }
                }
                max = 0;
                max_st = -1;
                max_en = -1;
                if k == n {
                    break;
                }
            }
            let i = kvec[k as usize];
            let mut gap = (a[(as1 + i) as usize].qpos - a[(as1 + i - 1) as usize].qpos) as i64
                - (a[(as1 + i) as usize].tpos - a[(as1 + i - 1) as usize].tpos);
            let mut n_ins = 0i32;
            let mut n_del = 0i32;
            if gap > 0 {
                n_ins += gap as i32;
            } else {
                n_del += (-gap) as i32;
            }
            let qs = a[(as1 + i - 1) as usize].qpos;
            let ts = a[(as1 + i - 1) as usize].tpos;
            let mut max_diff = 0;
            let mut max_diff_l = -1;
            let mut l = k + 1;
            while l < n && l <= k + max_ext_cnt {
                let j = kvec[l as usize];
                if a[(as1 + j) as usize].qpos - a[(as1 + j) as usize].len - qs > max_ext_len
                    || a[(as1 + j) as usize].tpos - a[(as1 + j) as usize].len as i64 - ts
                        > max_ext_len as i64
                {
                    break;
                }
                gap = (a[(as1 + j) as usize].qpos - a[(as1 + j - 1) as usize].qpos) as i64
                    - (a[(as1 + j) as usize].tpos - a[(as1 + j - 1) as usize].tpos);
                if gap > 0 {
                    n_ins += gap as i32;
                } else {
                    n_del += (-gap) as i32;
                }
                let diff = n_ins + n_del - (n_ins - n_del).abs();
                if max_diff < diff {
                    max_diff = diff;
                    max_diff_l = l;
                }
                l += 1;
            }
            if max_diff > diff_thres && max_diff > max {
                max = max_diff;
                max_st = k;
                max_en = max_diff_l;
            }
            k += 1;
        }
    }

    /// Original C static function `mm_filter_bad_seeds_alt` from `minibwa/align.c:447`.
    pub fn mm_filter_bad_seeds_alt(
        km: (),
        as1: i32,
        cnt1: i32,
        a: &mut [mb_anchor_t],
        min_gap: i32,
        max_ext: i32,
    ) {
        let mut n = 0;
        let kvec = collect_long_gaps(km, as1, cnt1, a, min_gap, &mut n);
        if kvec.is_empty() {
            return;
        }
        let mut k = 0i32;
        while k < n {
            let i = kvec[k as usize];
            let mut gap1 = (a[(as1 + i) as usize].qpos - a[(as1 + i - 1) as usize].qpos) as i64
                - (a[(as1 + i) as usize].tpos - a[(as1 + i - 1) as usize].tpos);
            let mut te1 = a[(as1 + i) as usize].tpos;
            let mut qe1 = a[(as1 + i) as usize].qpos;
            let mut left_len = a[(as1 + i) as usize].len;
            gap1 = gap1.abs();
            let mut l = k + 1;
            while l < n {
                let j = kvec[l as usize];
                if a[(as1 + j) as usize].qpos - qe1 > max_ext
                    || a[(as1 + j) as usize].tpos - te1 > max_ext as i64
                {
                    break;
                }
                let mut gap2 = (a[(as1 + j) as usize].qpos - a[(as1 + j - 1) as usize].qpos) as i64
                    - (a[(as1 + j) as usize].tpos - a[(as1 + j - 1) as usize].tpos);
                let m_t = a[(as1 + j - 1) as usize].tpos - te1 + left_len as i64;
                let m_q = a[(as1 + j - 1) as usize].qpos - qe1 + left_len;
                let m = (m_t.min(m_q as i64)) as i32;
                gap2 = gap2.abs();
                if m as i64 > gap1 + gap2 {
                    break;
                }
                te1 = a[(as1 + j) as usize].tpos;
                qe1 = a[(as1 + j) as usize].qpos;
                left_len = a[(as1 + j) as usize].len;
                gap1 = gap2;
                l += 1;
            }
            if l > k + 1 {
                let end = kvec[(l - 1) as usize];
                for j in kvec[k as usize]..end {
                    a[(as1 + j) as usize].flag |= MB_SEED_IGNORE;
                }
                a[(as1 + end) as usize].flag |= MB_SEED_LONG_JOIN;
            }
            k = l;
        }
    }

    /// Original C static function `mm_fix_bad_ends` from `minibwa/align.c:484`.
    pub fn mm_fix_bad_ends(
        r: &mb_hit_t,
        a: &[mb_anchor_t],
        bw: i32,
        min_match: i32,
        as_: &mut i32,
        cnt: &mut i32,
    ) {
        *as_ = r.as_;
        *cnt = r.cnt;
        if r.cnt < 3 {
            return;
        }
        let mut l = a[r.as_ as usize].len;
        let mut m = l;
        for i in r.as_ + 1..r.as_ + r.cnt - 1 {
            let idx = i as usize;
            if (a[idx].flag & MB_SEED_LONG_JOIN) != 0 {
                break;
            }
            let lr = (a[idx].tpos - a[idx - 1].tpos) as i32;
            let lq = a[idx].qpos - a[idx - 1].qpos;
            let min = lr.min(lq);
            let max = lr.max(lq);
            if max - min > l >> 1 {
                *as_ = i;
            }
            l += min;
            m += min.min(a[idx].len);
            if l >= bw << 1 || (m >= min_match && m >= bw) || m >= r.mlen >> 1 {
                break;
            }
        }
        *cnt = r.as_ + r.cnt - *as_;
        l = a[(r.as_ + r.cnt - 1) as usize].len;
        m = l;
        let mut i = r.as_ + r.cnt - 2;
        while i > *as_ {
            let idx = i as usize;
            if (a[idx + 1].flag & MB_SEED_LONG_JOIN) != 0 {
                break;
            }
            let lr = (a[idx + 1].tpos - a[idx].tpos) as i32 - a[idx + 1].len + a[idx].len;
            let lq = a[idx + 1].qpos - a[idx].qpos - a[idx + 1].len + a[idx].len;
            let min = lr.min(lq);
            let max = lr.max(lq);
            if max - min > l >> 1 {
                *cnt = i + 1 - *as_;
            }
            l += min;
            m += min.min(a[idx].len);
            if l >= bw << 1 || (m >= min_match && m >= bw) || m >= r.mlen >> 1 {
                break;
            }
            i -= 1;
        }
    }

    /// Original C static function `mb_max_stretch` from `minibwa/align.c:520`.
    pub fn mb_max_stretch(r: &mb_hit_t, a: &[mb_anchor_t], as_: &mut i32, cnt: &mut i32) {
        *as_ = r.as_;
        *cnt = r.cnt;
        if r.cnt < 2 {
            return;
        }
        let mut max_score = -1;
        let mut max_i = -1;
        let mut max_len = 0;
        let mut score = a[r.as_ as usize].len;
        let mut len = 1;
        for i in r.as_ + 1..r.as_ + r.cnt {
            let idx = i as usize;
            let lr = (a[idx].tpos - a[idx - 1].tpos) as i32;
            let lq = a[idx].qpos - a[idx - 1].qpos;
            if lq == lr {
                score += lq.min(a[idx].len);
                len += 1;
            } else {
                if score > max_score {
                    max_score = score;
                    max_len = len;
                    max_i = i - len;
                }
                score = a[idx].len;
                len = 1;
            }
        }
        if score > max_score {
            max_i = r.as_ + r.cnt - len;
            max_len = len;
        }
        *as_ = max_i;
        *cnt = max_len;
    }

    /// Original C static function `mb_align1` from `minibwa/align.c:546`.
    pub fn mb_align1(
        km: (),
        opt: &mb_opt_t,
        mi: &mb_idx_t,
        qlen: i32,
        qseq0: &mut [&mut [u8]; 2],
        mut mt: l2b_meth_t,
        r: &mut mb_hit_t,
        r2: &mut mb_hit_t,
        n_a: i32,
        a: &mut [mb_anchor_t],
        ez: &mut ksw_extz_t,
        tseq: &mut Vec<u8>,
        mat: &[i8; 25],
    ) {
        let is_sr = mb_is_sr_mode(opt, qlen);
        let max_back = if is_sr != 0 { 0 } else { 10 };
        r2.cnt = 0;
        if r.cnt == 0 {
            return;
        }
        let rev = a[r.as_ as usize].sid & 1;
        let tid = (a[r.as_ as usize].sid >> 1) as i64;
        let ctg_len = mi.l2b.ctg[tid as usize].len as i64;
        let bw = (opt.bw as f64 * 1.5 + 1.0) as i32;
        let bw_long = if is_sr == 0 {
            ((opt.bw_long as f64 * 1.5 + 1.0) as i32).max(bw)
        } else {
            bw
        };
        let mut dropped = 0;
        let ksw_flag = 0;
        if r.rev() != 0 {
            mt = l2b_meth_rev(mt);
        }
        let mut as1 = r.as_;
        let mut cnt1 = r.cnt;
        if is_sr != 0 {
            mb_max_stretch(r, a, &mut as1, &mut cnt1);
        } else {
            mm_fix_bad_ends(r, a, opt.bw, opt.min_chain_score * 2, &mut as1, &mut cnt1);
            mm_filter_bad_seeds(km, as1, cnt1, a, 10, 40, opt.max_gap >> 1, 10);
            mm_filter_bad_seeds_alt(km, as1, cnt1, a, 30, opt.max_gap >> 1);
        }
        if cnt1 <= 0 {
            return;
        }
        let first = a[as1 as usize];
        let last = a[(as1 + cnt1 - 1) as usize];
        let mut ts =
            first.tpos + 1 - first.len as i64 + mb_min_int32(first.len >> 1, max_back) as i64;
        let mut qs = first.qpos + 1 - first.len + mb_min_int32(first.len >> 1, max_back);
        let te_seed = last.tpos + 1 - mb_min_int32(last.len >> 1, max_back) as i64;
        let qe_seed = last.qpos + 1 - mb_min_int32(last.len >> 1, max_back);
        if te_seed <= ts || qe_seed <= qs || qs < 0 || qe_seed > qlen {
            return;
        }
        if (KOM_DBG_FLAG.load(Ordering::Relaxed) & MB_DBG_AN_POS) != 0 {
            for i in 0..r.cnt {
                let cur = &a[(r.as_ + i) as usize];
                let gap = if i == 0 {
                    0
                } else {
                    let prev = &a[(r.as_ + i - 1) as usize];
                    (cur.qpos - prev.qpos) - (cur.tpos - prev.tpos) as i32
                };
                eprintln!(
                    "AF\t{}\t{}\t{}\t{}\t{}\t{}",
                    r.as_, mi.l2b.ctg[tid as usize].name, cur.tpos, cur.qpos, gap, cur.len
                );
            }
        }

        let (ts0, qs0, te0, qe0) = if is_sr != 0 {
            let qs0 = 0;
            let qe0 = qlen;
            let mut l = qs as i64;
            if l as i32 * opt.a + opt.end_bonus > opt.q {
                l += ((l as i32 * opt.a + opt.end_bonus - opt.q) / opt.e) as i64;
            }
            let ts0 = (ts - l).max(0);
            let mut l = (qlen - qe_seed) as i64;
            if l as i32 * opt.a + opt.end_bonus > opt.q {
                l += ((l as i32 * opt.a + opt.end_bonus - opt.q) / opt.e) as i64;
            }
            let te0 = (te_seed + l).min(ctg_len);
            (ts0, qs0, te0, qe0)
        } else {
            let mut ts0 = a[r.as_ as usize].tpos + 1 - a[r.as_ as usize].len as i64;
            let mut qs0 = a[r.as_ as usize].qpos + 1 - a[r.as_ as usize].len;
            if ts0 < 0 {
                ts0 = 0;
            }
            let mut ts1_tmp = 0i64;
            let mut qs1_tmp = 0i32;
            if qs > 0 && ts > 0 {
                let mut l = qs.min(opt.max_gap);
                qs1_tmp = qs1_tmp.max(qs - l);
                qs0 = qs0.min(qs1_tmp);
                if l * opt.a > opt.q {
                    l += (l * opt.a - opt.q) / opt.e;
                }
                l = l.min(opt.max_gap).min(ts as i32);
                ts1_tmp = ts1_tmp.max(ts - l as i64);
                ts0 = ts0.min(ts1_tmp).min(ts);
            } else {
                ts0 = ts;
                qs0 = qs;
            }

            let mut te0 = a[(r.as_ + r.cnt - 1) as usize].tpos + 1;
            let mut qe0 = a[(r.as_ + r.cnt - 1) as usize].qpos + 1;
            let mut te1_tmp = ctg_len;
            let mut qe1_tmp = qlen;
            if qe_seed < qlen && te_seed < ctg_len {
                let mut l = (qlen - qe_seed).min(opt.max_gap);
                qe1_tmp = qe1_tmp.min(qe_seed + l);
                qe0 = qe0.max(qe1_tmp);
                if l * opt.a > opt.q {
                    l += (l * opt.a - opt.q) / opt.e;
                }
                l = l.min(opt.max_gap).min((ctg_len - te_seed) as i32);
                te1_tmp = te1_tmp.min(te_seed + l as i64);
                te0 = te0.max(te1_tmp);
            } else {
                te0 = te_seed;
                qe0 = qe_seed;
            }
            (ts0, qs0, te0, qe0)
        };

        if te0 <= ts0 {
            return;
        }
        tseq.clear();
        tseq.resize((te0 - ts0) as usize, 0);
        r.p = None;

        let (ts1, qs1) = if qs > 0 && ts > 0 {
            l2b_getseq_meth(&mi.l2b, tid, ts0, ts, mt, &mut tseq[..(ts - ts0) as usize]);
            {
                let qseq = &mut qseq0[rev as usize][qs0 as usize..qs as usize];
                mb_seq_rev((qs - qs0) as u32, qseq);
                mb_seq_rev((ts - ts0) as u32, &mut tseq[..(ts - ts0) as usize]);
                mb_align_pair(
                    km,
                    opt,
                    qs - qs0,
                    qseq,
                    (ts - ts0) as i32,
                    &tseq[..(ts - ts0) as usize],
                    mat,
                    bw,
                    opt.end_bonus,
                    if r.split_inv() != 0 {
                        opt.zdrop_inv
                    } else {
                        opt.zdrop
                    },
                    ksw_flag | KSW_EZ_EXTZ_ONLY | KSW_EZ_RIGHT | KSW_EZ_REV_CIGAR,
                    ez,
                );
                if ez.n_cigar > 0 {
                    mb_append_cigar(r, ez.n_cigar as u32, &ez.cigar);
                    if let Some(p) = r.p.as_mut() {
                        p.dp_score += if ez.reach_end != 0 {
                            ez.mqe
                        } else {
                            ez.max as i32
                        };
                    }
                }
                mb_seq_rev((qs - qs0) as u32, qseq);
            }
            let ts1 = ts
                - if ez.reach_end != 0 {
                    ez.mqe_t + 1
                } else {
                    ez.max_t + 1
                } as i64;
            let qs1 = qs
                - if ez.reach_end != 0 {
                    qs - qs0
                } else {
                    ez.max_q + 1
                };
            (ts1, qs1)
        } else {
            (ts, qs)
        };
        let mut te1;
        let mut qe1;
        if qs1 < 0 || ts1 < 0 {
            return;
        }

        {
            let te =
                a[as1 as usize].tpos + 1 - mb_min_int32(a[as1 as usize].len >> 1, max_back) as i64;
            let qe = a[as1 as usize].qpos + 1 - mb_min_int32(a[as1 as usize].len >> 1, max_back);
            if te < ts || qe < qs || te - ts != (qe - qs) as i64 {
                return;
            }
            let cigar0 = ((te - ts) as u32) << 4 | MB_CIGAR_MATCH;
            mb_append_cigar(r, 1, &[cigar0]);
            if let Some(p) = r.p.as_mut() {
                p.dp_score += opt.a * (te - ts) as i32;
            }
            ts = te;
            qs = qe;
            te1 = te;
            qe1 = qe;
        }

        for i in 1..cnt1 {
            let ai = a[(as1 + i) as usize];
            if (ai.flag & MB_SEED_IGNORE) != 0 && i != cnt1 - 1 {
                continue;
            }
            te1 = ai.tpos + 1 - mb_min_int32(ai.len >> 1, max_back) as i64;
            qe1 = ai.qpos + 1 - mb_min_int32(ai.len >> 1, max_back);
            if i == cnt1 - 1
                || (ai.flag & MB_SEED_LONG_JOIN) != 0
                || (qe1 - qs >= opt.min_ksw_len && te1 - ts >= opt.min_ksw_len as i64)
            {
                let mut bw1 = bw_long;
                let mut d1 = 0i64;
                if ai.len > opt.min_len * 2 {
                    d1 = te1 - (ai.tpos + 1 - ai.len as i64);
                    d1 = d1.min((qe1 - qs) as i64).min(te1 - ts);
                    d1 -= opt.min_len as i64;
                    if d1 < opt.min_len as i64 {
                        d1 = 0;
                    }
                }
                let te = te1 - d1;
                let qe = qe1 - d1 as i32;
                if (ai.flag & MB_SEED_LONG_JOIN) != 0 {
                    bw1 = (qe - qs).max((te - ts) as i32);
                }
                l2b_getseq_meth(&mi.l2b, tid, ts, te, mt, &mut tseq[..(te - ts) as usize]);
                let qseq = &qseq0[rev as usize][qs as usize..qe as usize];
                mb_align_pair(
                    km,
                    opt,
                    qe - qs,
                    qseq,
                    (te - ts) as i32,
                    &tseq[..(te - ts) as usize],
                    mat,
                    bw1,
                    -1,
                    opt.zdrop,
                    ksw_flag | KSW_EZ_APPROX_MAX,
                    ez,
                );
                let zdrop_code = mm_test_zdrop(
                    km,
                    opt,
                    qseq,
                    &tseq[..(te - ts) as usize],
                    ez.n_cigar as u32,
                    &ez.cigar,
                    mat,
                    is_sr,
                );
                if zdrop_code != 0 {
                    mb_align_pair(
                        km,
                        opt,
                        qe - qs,
                        qseq,
                        (te - ts) as i32,
                        &tseq[..(te - ts) as usize],
                        mat,
                        bw1,
                        -1,
                        if zdrop_code == 2 {
                            opt.zdrop_inv
                        } else {
                            opt.zdrop
                        },
                        ksw_flag,
                        ez,
                    );
                }
                if (KOM_DBG_FLAG.load(Ordering::Relaxed) & MB_DBG_AN_POS) != 0 {
                    eprintln!(
                        "AD\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                        r.as_, ts, te, qs, qe, zdrop_code, ez.zdropped
                    );
                }
                if ez.n_cigar > 0 {
                    mb_append_cigar(r, ez.n_cigar as u32, &ez.cigar);
                }
                if ez.zdropped != 0 {
                    let mut j = i - 1;
                    loop {
                        if a[(as1 + j) as usize].tpos <= ts + ez.max_t as i64 {
                            break;
                        }
                        if j == 0 {
                            break;
                        }
                        j -= 1;
                    }
                    dropped = 1;
                    if let Some(p) = r.p.as_mut() {
                        p.dp_score += ez.max as i32;
                    }
                    te1 = ts + ez.max_t as i64 + 1;
                    qe1 = qs + ez.max_q + 1;
                    let mut blen = 0;
                    let mlen =
                        mb_cal_mblen(cnt1 - (j + 1), &a[(as1 + j + 1) as usize..], &mut blen);
                    if mlen >= opt.min_chain_score {
                        mb_split_hit(r, r2, as1 + j + 1 - r.as_, qlen, a, &mi.l2b);
                        if zdrop_code == 2 {
                            r2.set_split_inv(1);
                        }
                    }
                    break;
                } else if let Some(p) = r.p.as_mut() {
                    p.dp_score += ez.score;
                }
                if d1 > 0 {
                    let cigar0 = (d1 as u32) << 4 | MB_CIGAR_MATCH;
                    mb_append_cigar(r, 1, &[cigar0]);
                    if let Some(p) = r.p.as_mut() {
                        p.dp_score += opt.a * d1 as i32;
                    }
                    ts = te1;
                    qs = qe1;
                } else {
                    ts = te;
                    qs = qe;
                }
            }
        }

        if dropped == 0 && qe1 < qe0 && te1 < te0 {
            l2b_getseq_meth(
                &mi.l2b,
                tid,
                te1,
                te0,
                mt,
                &mut tseq[..(te0 - te1) as usize],
            );
            let qseq = &qseq0[rev as usize][qe1 as usize..qe0 as usize];
            mb_align_pair(
                km,
                opt,
                qe0 - qe1,
                qseq,
                (te0 - te1) as i32,
                &tseq[..(te0 - te1) as usize],
                mat,
                bw,
                opt.end_bonus,
                opt.zdrop,
                ksw_flag | KSW_EZ_EXTZ_ONLY,
                ez,
            );
            if ez.n_cigar > 0 {
                mb_append_cigar(r, ez.n_cigar as u32, &ez.cigar);
                if let Some(p) = r.p.as_mut() {
                    p.dp_score += if ez.reach_end != 0 {
                        ez.mqe
                    } else {
                        ez.max as i32
                    };
                }
            }
            te1 += if ez.reach_end != 0 {
                ez.mqe_t + 1
            } else {
                ez.max_t + 1
            } as i64;
            qe1 += if ez.reach_end != 0 {
                qe0 - qe1
            } else {
                ez.max_q + 1
            };
        }

        if qe1 > qlen {
            return;
        }
        r.ts = ts1;
        r.te = te1;
        if rev == 0 {
            r.qs = qs1;
            r.qe = qe1;
        } else {
            r.qs = qlen - qe1;
            r.qe = qlen - qs1;
        }

        if r.p.is_some() {
            l2b_getseq_meth(
                &mi.l2b,
                tid,
                ts1,
                te1,
                mt,
                &mut tseq[..(te1 - ts1) as usize],
            );
            let qslice = &qseq0[r.rev() as usize][qs1 as usize..qe1 as usize];
            mb_update_extra(
                km,
                r,
                qslice,
                &tseq[..(te1 - ts1) as usize],
                mat,
                opt.q as i8,
                opt.e as i8,
                opt.flag,
                (is_sr == 0) as i32,
            );
        }
        let _ = n_a;
    }

    /// Original C static function `mb_align1_inv` from `minibwa/align.c:756`.
    pub fn mb_align1_inv(
        km: (),
        opt: &mb_opt_t,
        mi: &mb_idx_t,
        qlen: i32,
        qseq0: &mut [&mut [u8]; 2],
        mut mt: l2b_meth_t,
        r1: &mb_hit_t,
        r2: &mb_hit_t,
        r_inv: &mut mb_hit_t,
        ez: &mut ksw_extz_t,
        tseq: &mut Vec<u8>,
        mat: &[i8; 25],
    ) -> i32 {
        *r_inv = mb_hit_t::default();
        if (r1.split() & 1) == 0 || (r2.split() & 2) == 0 {
            return 0;
        }
        if r1.id != r1.parent && r1.parent != MB_PARENT_TMP_PRI {
            return 0;
        }
        if r2.id != r2.parent && r2.parent != MB_PARENT_TMP_PRI {
            return 0;
        }
        if r1.tid != r2.tid || r1.rev() != r2.rev() {
            return 0;
        }
        let ql = if r1.rev() != 0 {
            r1.qs - r2.qe
        } else {
            r2.qs - r1.qe
        };
        let tl = (r2.ts - r1.te) as i32;
        if ql < opt.min_chain_score
            || ql > opt.max_gap
            || tl < opt.min_chain_score
            || tl > opt.max_gap
        {
            return 0;
        }
        tseq.clear();
        tseq.resize(tl as usize, 0);
        if r1.rev() == 0 {
            mt = l2b_meth_rev(mt);
        }
        l2b_getseq_meth(&mi.l2b, r1.tid, r1.te, r2.ts, mt, tseq);
        let qstart = if r1.rev() != 0 {
            r2.qe as usize
        } else {
            (qlen - r2.qs) as usize
        };
        let qseq = &mut qseq0[if r1.rev() != 0 { 0 } else { 1 }][qstart..qstart + ql as usize];
        mb_seq_rev(ql as u32, qseq);
        mb_seq_rev(tl as u32, tseq);
        let mut q_off = 0;
        let mut t_off = 0;
        let score;
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            score = crate::ksw2_c_sse::maybe_ll_i16(
                ql, qseq, mat, tl, tseq, opt.q, opt.e, &mut q_off, &mut t_off,
            )
            .unwrap_or_else(|| {
                let qp = ksw_ll_qinit(km, 2, ql, qseq, 5, mat);
                ksw_ll_i16(&qp, tl, tseq, opt.q, opt.e, &mut q_off, &mut t_off)
            });
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            let qp = ksw_ll_qinit(km, 2, ql, qseq, 5, mat);
            score = ksw_ll_i16(&qp, tl, tseq, opt.q, opt.e, &mut q_off, &mut t_off);
        }
        mb_seq_rev(ql as u32, qseq);
        mb_seq_rev(tl as u32, tseq);
        if score < opt.min_dp_max * opt.a {
            return 0;
        }
        q_off = ql - (q_off + 1);
        t_off = tl - (t_off + 1);
        mb_align_pair(
            km,
            opt,
            ql - q_off,
            &qseq[q_off as usize..],
            tl - t_off,
            &tseq[t_off as usize..],
            mat,
            (opt.bw as f64 * 1.5) as i32,
            -1,
            opt.zdrop,
            KSW_EZ_EXTZ_ONLY,
            ez,
        );
        if ez.n_cigar == 0 {
            return 0;
        }
        mb_append_cigar(r_inv, ez.n_cigar as u32, &ez.cigar);
        if let Some(p) = r_inv.p.as_mut() {
            p.dp_score = ez.max as i32;
        }
        r_inv.id = -1;
        r_inv.parent = MB_PARENT_UNSET;
        r_inv.set_inv(1);
        r_inv.set_rev((r1.rev() == 0) as u8);
        r_inv.tid = r1.tid;
        if r_inv.rev() == 0 {
            r_inv.qs = r2.qe + q_off;
            r_inv.qe = r_inv.qs + ez.max_q + 1;
        } else {
            r_inv.qe = r2.qs - q_off;
            r_inv.qs = r_inv.qe - (ez.max_q + 1);
        }
        r_inv.ts = r1.te + t_off as i64;
        r_inv.te = r_inv.ts + ez.max_t as i64 + 1;
        mb_update_extra(
            km,
            r_inv,
            &qseq[q_off as usize..],
            &tseq[t_off as usize..],
            mat,
            opt.q as i8,
            opt.e as i8,
            opt.flag,
            mb_is_sr_mode(opt, qlen),
        );
        1
    }

    /// Original C static function `mb_insert_reg` from `minibwa/align.c:812`.
    pub fn mb_insert_reg(r: &mb_hit_t, i: i32, n_regs: &mut i32, regs: &mut Vec<mb_hit_t>) {
        let insert_at = (i + 1) as usize;
        regs.insert(insert_at, r.clone());
        *n_regs += 1;
    }

    /// Original C static function `mb_count_gaps` from `minibwa/align.c:822`.
    pub fn mb_count_gaps(r: &mb_hit_t, n_gap_: &mut i32, n_gapo_: &mut i32) {
        *n_gap_ = -1;
        *n_gapo_ = -1;
        let Some(p) = &r.p else {
            return;
        };
        let mut n_gapo = 0;
        let mut n_gap = 0;
        for &cg in p.cigar().iter().take(p.n_cigar as usize) {
            let op = cg & 0xf;
            let len = cg >> 4;
            if op == MB_CIGAR_INS || op == MB_CIGAR_DEL {
                n_gapo += 1;
                n_gap += len as i32;
            }
        }
        *n_gap_ = n_gap;
        *n_gapo_ = n_gapo;
    }

    /// Original C static function `mb_event_identity` from `minibwa/align.c:836`.
    pub fn mb_event_identity(r: &mb_hit_t) -> f64 {
        let Some(p) = &r.p else {
            return -1.0;
        };
        let mut n_gap = 0;
        let mut n_gapo = 0;
        mb_count_gaps(r, &mut n_gap, &mut n_gapo);
        r.mlen as f64 / (r.blen + p.n_ambi() as i32 - n_gap + n_gapo) as f64
    }

    /// Original C static function `mb_recal_max_dp` from `minibwa/align.c:844`.
    pub fn mb_recal_max_dp(r: &mb_hit_t, b2: f64, match_sc: i32) -> i32 {
        let Some(p) = &r.p else {
            return -1;
        };
        let mut n_gap = 0i32;
        let mut gap_cost = 0.0;
        for &cg in p.cigar().iter().take(p.n_cigar as usize) {
            let op = cg & 0xf;
            let len = cg >> 4;
            if op == MB_CIGAR_INS || op == MB_CIGAR_DEL {
                gap_cost += b2 * mb_log2(1.0 + len as f32) as f64;
                n_gap += len as i32;
            }
        }
        let n_mis = r.blen + p.n_ambi() as i32 - r.mlen - n_gap;
        (match_sc as f64 * (r.mlen as f64 - b2 * n_mis as f64 - gap_cost) + 0.499) as i32
    }

    /// Original C global function `mb_update_dp_max` from `minibwa/align.c:861`.
    pub fn mb_update_dp_max(
        qlen: i32,
        n_regs: i32,
        regs: &mut [mb_hit_t],
        frac: f64,
        a: i32,
        b: i32,
    ) {
        let mut max = -1;
        let mut max2 = -1;
        let mut max_i = -1;
        let mut max2_i = -1;
        if n_regs < 2 {
            return;
        }
        for i in 0..n_regs as usize {
            let Some(p) = &regs[i].p else {
                continue;
            };
            if p.dp_max > max {
                max2 = max;
                max2_i = max_i;
                max = p.dp_max;
                max_i = i as i32;
            } else if p.dp_max > max2 {
                max2 = p.dp_max;
                max2_i = i as i32;
            }
        }
        if max_i < 0 || max2_i < 0 {
            return;
        }
        if regs[max_i as usize].qe - regs[max_i as usize].qs < (qlen as f64 * frac) as i32 {
            return;
        }
        if regs[max2_i as usize].qe - regs[max2_i as usize].qs
            < ((regs[max_i as usize].qe - regs[max_i as usize].qs) as f64 * frac.sqrt()) as i32
        {
            return;
        }
        let mut div = 1.0 - mb_event_identity(&regs[max_i as usize]);
        if div < 0.02 {
            div = 0.02;
        }
        let mut b2 = 0.5 / div;
        if b2 * (a as f64) < b as f64 {
            b2 = a as f64 / b as f64;
        }
        for i in 0..n_regs as usize {
            if regs[i].p.is_none() {
                continue;
            }
            let mut dp_max = mb_recal_max_dp(&regs[i], b2, a);
            if dp_max < 0 {
                dp_max = 0;
            }
            regs[i].p.as_mut().unwrap().dp_max = dp_max;
        }
    }

    /// Original C global function `mb_align_skeleton` from `minibwa/align.c:887`.
    pub fn mb_align_skeleton(
        km: (),
        opt: &mb_opt_t,
        mi: &mb_idx_t,
        qlen: i32,
        qseq: &[u8],
        mt: l2b_meth_t,
        n_regs_: &mut i32,
        regs: &mut Vec<mb_hit_t>,
        a: &mut Vec<mb_anchor_t>,
    ) {
        let mut tseq = Vec::new();
        let mut qseq0_buf = Vec::new();
        mb_align_skeleton_with_scratch(
            km,
            opt,
            mi,
            qlen,
            qseq,
            mt,
            n_regs_,
            regs,
            a,
            &mut tseq,
            &mut qseq0_buf,
        );
    }

    pub fn mb_align_skeleton_with_scratch(
        km: (),
        opt: &mb_opt_t,
        mi: &mb_idx_t,
        qlen: i32,
        qseq: &[u8],
        mt: l2b_meth_t,
        n_regs_: &mut i32,
        regs: &mut Vec<mb_hit_t>,
        a: &mut Vec<mb_anchor_t>,
        tseq: &mut Vec<u8>,
        qseq0_buf: &mut Vec<u8>,
    ) {
        let mut n_regs = *n_regs_;
        let qlen_usize = qlen as usize;
        qseq0_buf.clear();
        qseq0_buf.resize(qlen_usize * 2, 0);
        let (qseq_fwd, qseq_rev) = qseq0_buf.split_at_mut(qlen_usize);
        qseq_fwd.copy_from_slice(qseq);
        for (i, &c) in qseq.iter().enumerate() {
            qseq_rev[qlen_usize - 1 - i] = if c < 4 { 3 - c } else { 4 };
        }
        let mut qseq0 = [qseq_fwd, qseq_rev];
        let n_a = mb_squeeze_a(km, n_regs, regs, a);
        let mut ez = ksw_extz_t::default();
        let mut mat = [0i8; 25];
        crate::ksw2::ksw_gen_nt4_mat(
            &mut mat,
            opt.a as i8,
            opt.b as i8,
            opt.b_ts as i8,
            opt.b_ambi as i8,
        );
        tseq.clear();
        let mut i = 0usize;
        while i < n_regs as usize {
            let mut r2 = mb_hit_t::default();
            mb_align1(
                km,
                opt,
                mi,
                qlen,
                &mut qseq0,
                mt,
                &mut regs[i],
                &mut r2,
                n_a,
                a,
                &mut ez,
                tseq,
                &mat,
            );
            if r2.cnt > 0 {
                mb_insert_reg(&r2, i as i32, &mut n_regs, regs);
            }
            if i > 0 && regs[i].split_inv() != 0 {
                let mut rinv = mb_hit_t::default();
                let (left, right) = regs.split_at_mut(i);
                let r1 = &left[i - 1];
                let rcur = &right[0];
                if mb_align1_inv(
                    km, opt, mi, qlen, &mut qseq0, mt, r1, rcur, &mut rinv, &mut ez, tseq, &mat,
                ) != 0
                {
                    mb_insert_reg(&rinv, i as i32, &mut n_regs, regs);
                    i += 1;
                }
            }
            i += 1;
        }
        mb_filter_hits(opt, qlen, &mut n_regs, regs);
        if mb_is_sr_mode(opt, qlen) == 0 {
            mb_update_dp_max(qlen, n_regs, regs, 0.9, opt.a, opt.b);
            mb_filter_hits(opt, qlen, &mut n_regs, regs);
        }
        mb_hit_sort(km, &mut n_regs, regs);
        *n_regs_ = n_regs;
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::bwt::mb_bwt_init;
        use crate::l2bit::{l2b_ctg_t, l2b_meth_t, l2b_t};
        use crate::map_algo::mb_idx_t;
        use crate::options::{mb_opt_init, mb_opt_t};
        use crate::pe::mb_extra_t;

        #[test]
        fn cigar_constants_match_public_header() {
            assert_eq!(MB_CIGAR_MATCH, 0);
            assert_eq!(MB_CIGAR_INS, 1);
            assert_eq!(MB_CIGAR_DEL, 2);
            assert_eq!(MB_CIGAR_N_SKIP, 3);
            assert_eq!(MB_CIGAR_SOFTCLIP, 4);
            assert_eq!(MB_CIGAR_HARDCLIP, 5);
            assert_eq!(MB_CIGAR_PADDING, 6);
            assert_eq!(MB_CIGAR_EQ_MATCH, 7);
            assert_eq!(MB_CIGAR_X_MISMATCH, 8);
            assert_eq!(MB_CIGAR_STR, "MIDNSHP=XB");
        }

        #[test]
        fn zdrop_tracker_records_largest_drop() {
            let mut max = i32::MIN;
            let mut max_i = -1;
            let mut max_j = -1;
            let mut max_zdrop = 0;
            let mut pos = [[-1, -1], [-1, -1]];
            update_max_zdrop(
                10,
                4,
                4,
                &mut max,
                &mut max_i,
                &mut max_j,
                2,
                &mut max_zdrop,
                &mut pos,
            );
            update_max_zdrop(
                3,
                8,
                5,
                &mut max,
                &mut max_i,
                &mut max_j,
                2,
                &mut max_zdrop,
                &mut pos,
            );
            assert_eq!(max, 10);
            assert_eq!(max_zdrop, 1);
            assert_eq!(pos, [[4, 8], [4, 5]]);
        }

        #[test]
        fn bandwidth_and_gap_accounting_match_cigar_rules() {
            let mut opt = mb_opt_t::default();
            mb_opt_init(&mut opt);
            assert_eq!(mb_min_int32(-3, 4), -3);
            assert_eq!(max_bw_from_mm(&opt, 5), 27);

            let hit = mb_hit_t {
                mlen: 80,
                blen: 100,
                p: Some(
                    mb_extra_t {
                        n_ambi_cs: 2,
                        ..Default::default()
                    }
                    .with_cigar(&[
                        20 << 4 | MB_CIGAR_MATCH,
                        3 << 4 | MB_CIGAR_INS,
                        5 << 4 | MB_CIGAR_DEL,
                        72 << 4 | MB_CIGAR_MATCH,
                    ]),
                ),
                ..Default::default()
            };
            let mut n_gap = 0;
            let mut n_gapo = 0;
            mb_count_gaps(&hit, &mut n_gap, &mut n_gapo);
            assert_eq!((n_gap, n_gapo), (8, 2));
            let identity = mb_event_identity(&hit);
            assert!(identity > 0.82 && identity < 0.84);
            assert!(mb_recal_max_dp(&hit, 2.0, 2) > 0);
        }

        #[test]
        fn update_dp_max_recalculates_when_two_long_hits_exist() {
            let mut regs = vec![
                mb_hit_t {
                    qs: 0,
                    qe: 95,
                    mlen: 90,
                    blen: 100,
                    p: Some(
                        mb_extra_t {
                            dp_max: 120,
                            ..Default::default()
                        }
                        .with_cigar(&[100 << 4 | MB_CIGAR_MATCH]),
                    ),
                    ..Default::default()
                },
                mb_hit_t {
                    qs: 5,
                    qe: 96,
                    mlen: 85,
                    blen: 100,
                    p: Some(
                        mb_extra_t {
                            dp_max: 110,
                            ..Default::default()
                        }
                        .with_cigar(&[90 << 4 | MB_CIGAR_MATCH, 5 << 4 | MB_CIGAR_DEL]),
                    ),
                    ..Default::default()
                },
            ];
            mb_update_dp_max(100, regs.len() as i32, &mut regs, 0.9, 2, 8);
            assert_eq!(regs[0].p.as_ref().unwrap().dp_max, 80);
            assert!(regs[1].p.as_ref().unwrap().dp_max < 80);
        }

        #[test]
        fn zdrop_test_uses_cigar_score_drop() {
            let mut opt = mb_opt_t::default();
            mb_opt_init(&mut opt);
            opt.zdrop = 5;
            opt.zdrop_inv = 1000;
            let qseq = vec![0, 0, 0, 0, 0, 0];
            let tseq = vec![0, 0, 1, 1, 1, 1];
            let mat = vec![
                2, -4, -4, -4, -1, -4, 2, -4, -4, -1, -4, -4, 2, -4, -1, -4, -4, -4, 2, -1, -1, -1,
                -1, -1, -1,
            ];
            assert_eq!(
                mm_test_zdrop(
                    (),
                    &opt,
                    &qseq,
                    &tseq,
                    1,
                    &[6 << 4 | MB_CIGAR_MATCH],
                    &mat,
                    1
                ),
                1
            );
        }

        #[test]
        fn append_and_fix_cigar_merge_and_trim_edges() {
            let mut hit = mb_hit_t::default();
            mb_append_cigar(
                &mut hit,
                3,
                &[
                    2 << 4 | MB_CIGAR_MATCH,
                    1 << 4 | MB_CIGAR_INS,
                    2 << 4 | MB_CIGAR_MATCH,
                ],
            );
            mb_append_cigar(&mut hit, 1, &[3 << 4 | MB_CIGAR_MATCH]);
            assert_eq!(
                hit.p.as_ref().unwrap().cigar()[..hit.p.as_ref().unwrap().n_cigar as usize]
                    .to_vec(),
                vec![
                    2 << 4 | MB_CIGAR_MATCH,
                    1 << 4 | MB_CIGAR_INS,
                    5 << 4 | MB_CIGAR_MATCH
                ]
            );

            hit.qs = 0;
            hit.qe = 8;
            hit.ts = 0;
            hit.te = 7;
            let qseq = vec![0, 1, 2, 3, 0, 1, 2, 3];
            let tseq = vec![0, 1, 3, 0, 1, 2, 3];
            let mut qshift = 0;
            let mut tshift = 0;
            mb_fix_cigar(&mut hit, &qseq, &tseq, &mut qshift, &mut tshift);
            assert_eq!((qshift, tshift), (0, 0));
            assert_eq!(hit.p.as_ref().unwrap().n_cigar, 3);
        }

        #[test]
        fn eqx_update_splits_match_runs() {
            let mut hit = mb_hit_t {
                p: Some(
                    mb_extra_t {
                        ..Default::default()
                    }
                    .with_cigar(&[5 << 4 | MB_CIGAR_MATCH]),
                ),
                ..Default::default()
            };
            let qseq = vec![0, 1, 2, 3, 0];
            let tseq = vec![0, 1, 3, 3, 1];
            mm_update_cigar_eqx(&mut hit, &qseq, &tseq);
            assert_eq!(
                hit.p.as_ref().unwrap().cigar()[..hit.p.as_ref().unwrap().n_cigar as usize]
                    .to_vec(),
                vec![
                    2 << 4 | MB_CIGAR_EQ_MATCH,
                    1 << 4 | MB_CIGAR_X_MISMATCH,
                    1 << 4 | MB_CIGAR_EQ_MATCH,
                    1 << 4 | MB_CIGAR_X_MISMATCH,
                ]
            );
        }

        #[test]
        fn update_extra_recomputes_lengths_scores_and_eqx() {
            let mut hit = mb_hit_t {
                qs: 0,
                qe: 5,
                ts: 0,
                te: 5,
                p: Some(
                    mb_extra_t {
                        ..Default::default()
                    }
                    .with_cigar(&[5 << 4 | MB_CIGAR_MATCH]),
                ),
                ..Default::default()
            };
            let qseq = vec![0, 1, 2, 3, 0];
            let tseq = vec![0, 1, 3, 3, 4];
            let mat = vec![
                2, -4, -4, -4, -1, -4, 2, -4, -4, -1, -4, -4, 2, -4, -1, -4, -4, -4, 2, -1, -1, -1,
                -1, -1, -1,
            ];
            mb_update_extra(
                (),
                &mut hit,
                &qseq,
                &tseq,
                &mat,
                12,
                2,
                crate::options::MB_F_EQX,
                0,
            );
            let p = hit.p.as_ref().unwrap();
            assert_eq!((hit.blen, hit.mlen, p.n_ambi()), (4, 3, 1));
            assert_eq!(p.dp_max, 4);
            assert_eq!(
                p.cigar()[..p.n_cigar as usize].to_vec(),
                vec![
                    2 << 4 | MB_CIGAR_EQ_MATCH,
                    1 << 4 | MB_CIGAR_X_MISMATCH,
                    1 << 4 | MB_CIGAR_EQ_MATCH,
                    1 << 4 | MB_CIGAR_X_MISMATCH,
                ]
            );
        }

        #[test]
        fn long_gap_filters_mark_bad_seed_runs() {
            let mut anchors = vec![
                mb_anchor_t {
                    len: 10,
                    qpos: 9,
                    tpos: 9,
                    ..Default::default()
                },
                mb_anchor_t {
                    len: 10,
                    qpos: 1019,
                    tpos: 19,
                    ..Default::default()
                },
                mb_anchor_t {
                    len: 10,
                    qpos: 1029,
                    tpos: 1029,
                    ..Default::default()
                },
                mb_anchor_t {
                    len: 10,
                    qpos: 2039,
                    tpos: 1039,
                    ..Default::default()
                },
                mb_anchor_t {
                    len: 10,
                    qpos: 2049,
                    tpos: 2049,
                    ..Default::default()
                },
            ];
            let mut n = 0;
            let gaps = collect_long_gaps((), 0, anchors.len() as i32, &anchors, 30, &mut n);
            assert_eq!(gaps, vec![1, 2, 3, 4]);
            assert_eq!(n, 4);
            mm_filter_bad_seeds((), 0, anchors.len() as i32, &mut anchors, 30, 40, 5000, 10);
            assert_ne!(anchors[1].flag & MB_SEED_IGNORE, 0);
            assert_ne!(anchors[2].flag & MB_SEED_IGNORE, 0);
        }

        #[test]
        fn alternate_filter_marks_long_join_and_ignored_middle() {
            let mut anchors = vec![
                mb_anchor_t {
                    len: 20,
                    qpos: 20,
                    tpos: 20,
                    ..Default::default()
                },
                mb_anchor_t {
                    len: 20,
                    qpos: 1040,
                    tpos: 40,
                    ..Default::default()
                },
                mb_anchor_t {
                    len: 20,
                    qpos: 2060,
                    tpos: 60,
                    ..Default::default()
                },
                mb_anchor_t {
                    len: 20,
                    qpos: 2080,
                    tpos: 80,
                    ..Default::default()
                },
            ];
            mm_filter_bad_seeds_alt((), 0, anchors.len() as i32, &mut anchors, 30, 5000);
            assert_ne!(anchors[1].flag & MB_SEED_IGNORE, 0);
            assert_ne!(anchors[2].flag & MB_SEED_LONG_JOIN, 0);
        }

        #[test]
        fn bad_end_and_max_stretch_select_anchor_subchains() {
            let anchors = vec![
                mb_anchor_t {
                    len: 10,
                    qpos: 9,
                    tpos: 9,
                    ..Default::default()
                },
                mb_anchor_t {
                    len: 10,
                    qpos: 40,
                    tpos: 12,
                    ..Default::default()
                },
                mb_anchor_t {
                    len: 10,
                    qpos: 50,
                    tpos: 22,
                    ..Default::default()
                },
                mb_anchor_t {
                    len: 10,
                    qpos: 60,
                    tpos: 32,
                    ..Default::default()
                },
                mb_anchor_t {
                    len: 10,
                    qpos: 200,
                    tpos: 100,
                    ..Default::default()
                },
            ];
            let hit = mb_hit_t {
                as_: 0,
                cnt: anchors.len() as i32,
                mlen: 50,
                ..Default::default()
            };
            let mut as_ = 0;
            let mut cnt = 0;
            mb_max_stretch(&hit, &anchors, &mut as_, &mut cnt);
            assert_eq!((as_, cnt), (1, 3));

            mm_fix_bad_ends(&hit, &anchors, 20, 20, &mut as_, &mut cnt);
            assert!(as_ >= 1);
            assert!(cnt <= 4);
        }

        #[test]
        fn insert_reg_inserts_after_index_and_updates_count() {
            let mut regs = vec![
                mb_hit_t {
                    id: 0,
                    ..Default::default()
                },
                mb_hit_t {
                    id: 2,
                    ..Default::default()
                },
            ];
            let mut n = regs.len() as i32;
            mb_insert_reg(
                &mb_hit_t {
                    id: 1,
                    ..Default::default()
                },
                0,
                &mut n,
                &mut regs,
            );
            assert_eq!(n, 3);
            assert_eq!(regs.iter().map(|r| r.id).collect::<Vec<_>>(), vec![0, 1, 2]);
        }

        #[test]
        fn align_pair_uses_ungapped_fast_paths_and_memory_skip() {
            let mut opt = mb_opt_t::default();
            mb_opt_init(&mut opt);
            let mut mat = [0i8; 25];
            crate::ksw2::ksw_gen_nt4_mat(
                &mut mat,
                opt.a as i8,
                opt.b as i8,
                opt.b_ts as i8,
                opt.b_ambi as i8,
            );
            let q = vec![0, 1, 2, 3];
            let t = vec![0, 1, 2, 3];
            let mut ez = ksw_extz_t::default();
            mb_align_pair(
                (),
                &opt,
                q.len() as i32,
                &q,
                t.len() as i32,
                &t,
                &mat,
                50,
                opt.end_bonus,
                opt.zdrop,
                0,
                &mut ez,
            );
            assert_eq!(ez.n_cigar, 1);
            assert_eq!(ez.cigar[0], 4 << 4 | MB_CIGAR_MATCH);
            assert_eq!(ez.score, 8);

            mb_align_pair(
                (),
                &opt,
                5,
                &[0, 1, 1, 2, 3],
                4,
                &[0, 1, 2, 3],
                &mat,
                50,
                opt.end_bonus,
                opt.zdrop,
                KSW_EZ_GENERIC_SC,
                &mut ez,
            );
            assert!(ez.n_cigar >= 2);
            assert!(ez.cigar[..ez.n_cigar as usize]
                .iter()
                .any(|c| (*c & 0xf) == MB_CIGAR_INS));

            opt.max_sw_mat = 1;
            mb_align_pair(
                (),
                &opt,
                q.len() as i32,
                &q,
                t.len() as i32 + 1,
                &[0, 1, 2, 3, 0],
                &mat,
                50,
                opt.end_bonus,
                opt.zdrop,
                KSW_EZ_GENERIC_SC,
                &mut ez,
            );
            assert_eq!(ez.zdropped, 1);
            assert_eq!(ez.n_cigar, 0);
        }

        #[test]
        fn align_pair_matches_original_debug_extension_boundaries() {
            let mut opt = mb_opt_t::default();
            mb_opt_init(&mut opt);
            let mut mat = [0i8; 25];
            crate::ksw2::ksw_gen_nt4_mat(
                &mut mat,
                opt.a as i8,
                opt.b as i8,
                opt.b_ts as i8,
                opt.b_ambi as i8,
            );
            let target = "TCCCCATACCCAACCCCCTGGTCAACCTCAACCTAGGCCTCCTATTTATTCTAGCCACCTCTAGCCTAGCCGTTTACTCAATCCTCTGATCAGGGTGAGCATCAAACTC"
                .bytes()
                .map(|c| match c {
                    b'A' => 0,
                    b'C' => 1,
                    b'G' => 2,
                    b'T' => 3,
                    _ => 4,
                })
                .collect::<Vec<_>>();
            let query = "ACCCCATACCCAACCCCCGGGTCAACCTCAACCAAGGCCTCCGATTTATTCTATC"
                .bytes()
                .map(|c| match c {
                    b'A' => 0,
                    b'C' => 1,
                    b'G' => 2,
                    b'T' => 3,
                    _ => 4,
                })
                .collect::<Vec<_>>();
            let mut ez = ksw_extz_t::default();
            mb_align_pair(
                (),
                &opt,
                query.len() as i32,
                &query,
                target.len() as i32,
                &target,
                &mat,
                21,
                opt.end_bonus,
                opt.zdrop,
                KSW_EZ_EXTZ_ONLY,
                &mut ez,
            );
            assert_eq!(
                (
                    ez.score,
                    ez.max,
                    ez.max_q,
                    ez.max_t,
                    ez.mqe,
                    ez.mqe_t,
                    ez.reach_end,
                    ez.n_cigar
                ),
                (i32::MIN / 2, 66, 52, 52, 60, 54, 1, 1)
            );
            assert_eq!(ez.cigar[0], 55 << 4 | MB_CIGAR_MATCH);

            let target = "GACAATAGGGATCCCATTGAACAAGGCAACCAGTTCAATAACCTAGTTAACTCATATCATCAAGCGAAACTGACCACTTCAGAATCGTA"
                .bytes()
                .map(|c| match c {
                    b'A' => 0,
                    b'C' => 1,
                    b'G' => 2,
                    b'T' => 3,
                    _ => 4,
                })
                .collect::<Vec<_>>();
            let query = "TACAATATGGATCCCATTGAACATTGCTACAAGTTCTATAACCTA"
                .bytes()
                .map(|c| match c {
                    b'A' => 0,
                    b'C' => 1,
                    b'G' => 2,
                    b'T' => 3,
                    _ => 4,
                })
                .collect::<Vec<_>>();
            mb_align_pair(
                (),
                &opt,
                query.len() as i32,
                &query,
                target.len() as i32,
                &target,
                &mat,
                41,
                opt.end_bonus,
                opt.zdrop,
                KSW_EZ_EXTZ_ONLY | KSW_EZ_RIGHT | KSW_EZ_REV_CIGAR,
                &mut ez,
            );
            assert_eq!((ez.score, ez.max, ez.n_cigar), (i32::MIN / 2, 26, 1));
            assert_eq!(ez.cigar[0], 45 << 4 | MB_CIGAR_MATCH);
        }

        #[test]
        fn align_skeleton_aligns_seed_hit_on_in_memory_reference() {
            let mut opt = mb_opt_t::default();
            mb_opt_init(&mut opt);
            opt.min_chain_score = 1;
            opt.min_dp_max = 1;
            let mi = mb_idx_t {
                is_meth: 0,
                l2b: l2b_t {
                    tot_len: 32,
                    n_ctg: 1,
                    ctg: vec![l2b_ctg_t {
                        name: "ctg".to_string(),
                        len: 32,
                        off: 0,
                        ..Default::default()
                    }],
                    pac: vec![0],
                    ..Default::default()
                },
                bwt: mb_bwt_init(),
            };
            let qseq = vec![0u8; 12];
            let mut regs = vec![mb_hit_t {
                as_: 0,
                cnt: 1,
                score: 8,
                score0: 8,
                mlen: 8,
                blen: 8,
                ..Default::default()
            }];
            let mut anchors = vec![mb_anchor_t {
                sid: 0,
                len: 8,
                qpos: 7,
                tpos: 7,
                ..Default::default()
            }];
            let mut n_regs = regs.len() as i32;
            mb_align_skeleton(
                (),
                &opt,
                &mi,
                qseq.len() as i32,
                &qseq,
                l2b_meth_t::L2B_METH_NONE,
                &mut n_regs,
                &mut regs,
                &mut anchors,
            );
            assert_eq!(n_regs, 1);
            assert_eq!(
                (regs[0].qs, regs[0].qe, regs[0].ts, regs[0].te),
                (0, 12, 0, 12)
            );
            assert_eq!(regs[0].p.as_ref().unwrap().n_cigar, 1);
            assert_eq!(
                regs[0].p.as_ref().unwrap().cigar()[0],
                12 << 4 | MB_CIGAR_MATCH
            );
        }
    }
}

pub mod bseq {
    #![allow(unused_variables, dead_code, non_snake_case, non_camel_case_types)]

    use crate::kommon::kom_revcomp;
    use flate2::bufread::MultiGzDecoder;
    use std::alloc::{alloc, dealloc, handle_alloc_error, Layout};
    use std::fs::File;
    use std::io::{self, BufRead, BufReader, Read};
    use std::ptr::NonNull;

    const CHECK_PAIR_THRES: u64 = 1000000;

    pub struct mb_opt_str_t {
        ptr: NonNull<u8>,
    }

    unsafe impl Send for mb_opt_str_t {}
    unsafe impl Sync for mb_opt_str_t {}

    impl mb_opt_str_t {
        #[inline]
        pub fn from_string(s: String) -> Self {
            let bytes = s.into_bytes();
            Self::from_bytes(&bytes)
        }

        fn from_bytes(bytes: &[u8]) -> Self {
            let header = std::mem::size_of::<usize>();
            let size = header + bytes.len();
            let layout = Layout::from_size_align(size.max(header), std::mem::align_of::<usize>())
                .expect("valid optional string layout");
            let base = unsafe { alloc(layout) };
            let Some(base) = NonNull::new(base) else {
                handle_alloc_error(layout);
            };
            unsafe {
                (base.as_ptr() as *mut usize).write(bytes.len());
                std::ptr::copy_nonoverlapping(
                    bytes.as_ptr(),
                    base.as_ptr().add(header),
                    bytes.len(),
                );
                Self {
                    ptr: NonNull::new_unchecked(base.as_ptr().add(header)),
                }
            }
        }

        #[inline]
        fn len(&self) -> usize {
            unsafe {
                let header = std::mem::size_of::<usize>();
                *((self.ptr.as_ptr().sub(header)) as *const usize)
            }
        }

        #[inline]
        pub fn as_str(&self) -> &str {
            unsafe {
                std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                    self.ptr.as_ptr(),
                    self.len(),
                ))
            }
        }
    }

    impl Clone for mb_opt_str_t {
        fn clone(&self) -> Self {
            Self::from_bytes(self.as_str().as_bytes())
        }
    }

    impl Drop for mb_opt_str_t {
        fn drop(&mut self) {
            let header = std::mem::size_of::<usize>();
            let len = self.len();
            let layout =
                Layout::from_size_align((header + len).max(header), std::mem::align_of::<usize>())
                    .expect("valid optional string layout");
            unsafe {
                dealloc(self.ptr.as_ptr().sub(header), layout);
            }
        }
    }

    impl std::fmt::Debug for mb_opt_str_t {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.as_str().fmt(f)
        }
    }

    impl PartialEq for mb_opt_str_t {
        fn eq(&self, other: &Self) -> bool {
            self.as_str() == other.as_str()
        }
    }

    impl Eq for mb_opt_str_t {}

    impl From<String> for mb_opt_str_t {
        fn from(value: String) -> Self {
            Self::from_string(value)
        }
    }

    impl From<&str> for mb_opt_str_t {
        fn from(value: &str) -> Self {
            Self::from_bytes(value.as_bytes())
        }
    }

    impl std::ops::Deref for mb_opt_str_t {
        type Target = str;

        fn deref(&self) -> &Self::Target {
            self.as_str()
        }
    }

    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    pub struct mb_bseq1_t {
        pub l_seq: u64,
        pub id: u64,
        pub name: Box<str>,
        pub seq: Box<str>,
        pub qual: Option<mb_opt_str_t>,
        pub comment: Option<mb_opt_str_t>,
    }

    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    pub struct kvec_t<T> {
        pub n: usize,
        pub m: usize,
        pub a: Vec<T>,
    }

    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    pub struct kseq_t {
        pub name: String,
        pub comment: String,
        pub seq: String,
        pub qual: String,
    }

    pub struct mb_bseq_file_s {
        pub records: Vec<kseq_t>,
        pub pos: usize,
        pub s: Option<mb_bseq1_t>,
        reader: Option<Box<dyn BufRead + Send>>,
        pending_header: Option<(u8, String)>,
        last_record_name: Option<String>,
        eof: bool,
        pub parse_error: bool,
        pub parse_error_after: Option<String>,
        pub parse_error_reported: bool,
        pub suppress_parse_warnings: bool,
    }
    pub type mb_bseq_file_t = mb_bseq_file_s;

    /// Original C global function `kvec_t` from `minibwa/bseq.c:8`.
    pub fn kvec_t<T>() -> kvec_t<T> {
        kvec_t {
            n: 0,
            m: 0,
            a: Vec::new(),
        }
    }

    /// Original C static function `mb_qname_len` from `minibwa/bseq.h:26`.
    pub fn mb_qname_len(s: &str) -> i32 {
        let b = s.as_bytes();
        let l = b.len();
        if l >= 3 && b[l - 1].is_ascii_digit() && b[l - 2] == b'/' {
            (l - 2) as i32
        } else {
            l as i32
        }
    }

    /// Original C static function `mb_qname_same` from `minibwa/bseq.h:33`.
    pub fn mb_qname_same(s1: &str, s2: &str) -> i32 {
        let l1 = mb_qname_len(s1) as usize;
        let l2 = mb_qname_len(s2) as usize;
        (l1 == l2 && s1.as_bytes()[..l1] == s2.as_bytes()[..l2]) as i32
    }

    /// Original C static function `mb_revcomp_bseq` from `minibwa/bseq.h:41`.
    pub fn mb_revcomp_bseq(s: &mut mb_bseq1_t) {
        let mut seq = s.seq.as_bytes().to_vec();
        kom_revcomp(s.l_seq, &mut seq);
        s.seq = String::from_utf8(seq).unwrap().into_boxed_str();
        if let Some(qual) = &mut s.qual {
            let mut q = qual.as_bytes().to_vec();
            q.reverse();
            *qual = mb_opt_str_t::from_string(String::from_utf8(q).unwrap());
        }
    }

    /// Original C global function `mb_bseq_open` from `minibwa/bseq.c:43`.
    pub fn mb_bseq_open(fn_: Option<&str>) -> Option<mb_bseq_file_t> {
        let raw: Box<dyn Read + Send> = match fn_ {
            Some(path) if path != "-" => Box::new(File::open(path).ok()?),
            _ => Box::new(io::stdin()),
        };
        let mut buffered = BufReader::new(raw);
        let is_gzip = buffered.fill_buf().ok()?.starts_with(&[0x1f, 0x8b]);
        let reader: Box<dyn BufRead + Send> = if is_gzip {
            Box::new(BufReader::new(MultiGzDecoder::new(buffered)))
        } else {
            Box::new(buffered)
        };
        Some(mb_bseq_file_t {
            records: Vec::new(),
            pos: 0,
            s: None,
            reader: Some(reader),
            pending_header: None,
            last_record_name: None,
            eof: false,
            parse_error: false,
            parse_error_after: None,
            parse_error_reported: false,
            suppress_parse_warnings: false,
        })
    }

    /// Original C global function `mb_bseq_close` from `minibwa/bseq.c:55`.
    pub fn mb_bseq_close(fp: Option<mb_bseq_file_t>) {
        drop(fp);
    }

    /// Original C static function `kstrdup` from `minibwa/bseq.c:62`.
    pub fn kstrdup(s: &str) -> String {
        s.to_string()
    }

    /// Original C static function `kseq2bseq` from `minibwa/bseq.c:70`.
    pub fn kseq2bseq(ks: kseq_t, s: &mut mb_bseq1_t, with_qual: i32, with_comment: i32) {
        if ks.name.is_empty() {
            eprintln!("[WARNING]\u{1b}[1;31m empty sequence name in the input.\u{1b}[0m");
        }
        s.name = ks.name.into_boxed_str();
        let mut seq = ks.seq.into_bytes();
        for c in &mut seq {
            if *c == b'u' || *c == b'U' {
                *c -= 1;
            }
        }
        s.l_seq = seq.len() as u64;
        s.seq = String::from_utf8(seq).unwrap().into_boxed_str();
        s.qual = if with_qual != 0 && !ks.qual.is_empty() {
            Some(mb_opt_str_t::from_string(ks.qual))
        } else {
            None
        };
        s.comment = if with_comment != 0 && !ks.comment.is_empty() {
            Some(mb_opt_str_t::from_string(ks.comment))
        } else {
            None
        };
    }

    macro_rules! kseq_read {
        ($fp:expr) => {{
            (|| -> Option<kseq_t> {
                let fp = &mut *$fp;
                if fp.parse_error || fp.eof {
                    return None;
                }
                let (start_ch, header) = if let Some(header) = fp.pending_header.take() {
                    header
                } else {
                    let reader = fp.reader.as_mut()?;
                    let mut line = String::new();
                    loop {
                        line.clear();
                        let n = reader.read_line(&mut line).ok()?;
                        if n == 0 {
                            fp.eof = true;
                            return None;
                        }
                        let bytes = line.as_bytes();
                        if let Some(pos) = bytes.iter().position(|&c| c == b'>' || c == b'@') {
                            let start_ch = bytes[pos];
                            let mut header =
                                String::from_utf8_lossy(&bytes[pos + 1..]).into_owned();
                            while header.ends_with('\n') || header.ends_with('\r') {
                                header.pop();
                            }
                            break (start_ch, header);
                        }
                    }
                };
                let mut it = header.splitn(2, |c: char| c.is_ascii_whitespace());
                let name = it.next().unwrap_or("").to_string();
                let comment = it.next().unwrap_or("").to_string();
                let mut seq = Vec::new();
                let mut saw_plus = false;
                let mut line = String::new();
                loop {
                    line.clear();
                    let n = fp.reader.as_mut()?.read_line(&mut line).ok()?;
                    if n == 0 {
                        fp.eof = true;
                        break;
                    }
                    let bytes = line.as_bytes();
                    if bytes.iter().all(|&c| c == b'\n' || c == b'\r') {
                        continue;
                    }
                    if bytes[0] == b'>' || bytes[0] == b'@' {
                        let mut header = String::from_utf8_lossy(&bytes[1..]).into_owned();
                        while header.ends_with('\n') || header.ends_with('\r') {
                            header.pop();
                        }
                        fp.pending_header = Some((bytes[0], header));
                        break;
                    }
                    if bytes[0] == b'+' {
                        saw_plus = true;
                        break;
                    }
                    let mut end = bytes.len();
                    while end > 0 && matches!(bytes[end - 1], b'\n' | b'\r') {
                        end -= 1;
                    }
                    seq.extend_from_slice(&bytes[..end]);
                }
                let mut qual = Vec::new();
                if start_ch == b'@' && saw_plus {
                    while qual.len() < seq.len() {
                        line.clear();
                        let n = fp.reader.as_mut()?.read_line(&mut line).ok()?;
                        if n == 0 {
                            fp.eof = true;
                            break;
                        }
                        let bytes = line.as_bytes();
                        let mut end = bytes.len();
                        while end > 0 && matches!(bytes[end - 1], b'\n' | b'\r') {
                            end -= 1;
                        }
                        qual.extend_from_slice(&bytes[..end]);
                        if qual.len() >= seq.len() {
                            break;
                        }
                    }
                    if qual.len() != seq.len() {
                        fp.parse_error = true;
                        if fp.parse_error_after.is_none() {
                            fp.parse_error_after = fp.last_record_name.clone();
                        }
                        return None;
                    }
                }
                fp.pos += 1;
                let rec = kseq_t {
                    name,
                    comment,
                    seq: String::from_utf8(seq).unwrap(),
                    qual: String::from_utf8(qual).unwrap(),
                };
                fp.last_record_name = Some(rec.name.clone());
                Some(rec)
            })()
        }};
    }

    /// Original C global function `mb_bseq_read` from `minibwa/bseq.c:85`.
    pub fn mb_bseq_read(
        fp: &mut mb_bseq_file_t,
        chunk_size: i64,
        with_qual: i32,
        with_comment: i32,
        frag_mode: i32,
        min_cnt: i32,
        mut max_chunk_size: i64,
        n_: &mut i32,
    ) -> Vec<mb_bseq1_t> {
        let mut size = 0i64;
        let mut a = Vec::new();
        *n_ = 0;
        if let Some(s) = fp.s.take() {
            size = s.l_seq as i64;
            a.push(s);
        }
        if max_chunk_size < chunk_size {
            max_chunk_size = chunk_size;
        }
        while let Some(ks) = kseq_read!(fp) {
            let mut s = mb_bseq1_t::default();
            kseq2bseq(ks, &mut s, with_qual, with_comment);
            size += s.l_seq as i64;
            a.push(s);
            let to_stop = chunk_size <= 0
                || max_chunk_size <= 0
                || size >= max_chunk_size
                || (size >= chunk_size && a.len() >= min_cnt as usize);
            if to_stop {
                if frag_mode != 0
                    && a.last()
                        .map(|x| x.l_seq < CHECK_PAIR_THRES)
                        .unwrap_or(false)
                {
                    while let Some(ks) = kseq_read!(fp) {
                        let mut s = mb_bseq1_t::default();
                        kseq2bseq(ks, &mut s, with_qual, with_comment);
                        if mb_qname_same(&s.name, &a[a.len() - 1].name) != 0 {
                            a.push(s);
                        } else {
                            fp.s = Some(s);
                            break;
                        }
                    }
                }
                break;
            }
        }
        if fp.parse_error && !fp.parse_error_reported && !fp.suppress_parse_warnings {
            if let Some(name) = &fp.parse_error_after {
                eprintln!(
                    "[WARNING]\u{1b}[1;31m failed to parse the FASTA/FASTQ record next to '{}'. Continue anyway.\u{1b}[0m",
                    name
                );
            } else {
                eprintln!(
                    "[WARNING]\u{1b}[1;31m failed to parse the first FASTA/FASTQ record. Continue anyway.\u{1b}[0m"
                );
            }
            fp.parse_error_reported = true;
        }
        *n_ = a.len() as i32;
        a
    }

    /// Original C global function `mb_bseq_read_frag` from `minibwa/bseq.c:133`.
    pub fn mb_bseq_read_frag(
        n_fp: i32,
        fp: &mut [mb_bseq_file_t],
        chunk_size: i64,
        with_qual: i32,
        with_comment: i32,
        n_: &mut i32,
    ) -> Vec<mb_bseq1_t> {
        let mut size = 0i64;
        let mut a = Vec::new();
        *n_ = 0;
        if n_fp < 1 {
            return a;
        }
        loop {
            let mut read = Vec::with_capacity(n_fp as usize);
            let mut n_read = 0;
            for f in fp.iter_mut().take(n_fp as usize) {
                let rec = kseq_read!(f);
                if rec.is_some() {
                    n_read += 1;
                }
                read.push(rec);
            }
            if n_read < n_fp {
                if n_read > 0 {
                    eprintln!(
                        "[W::mb_bseq_read_frag]\u{1b}[1;31m query files have different number of records; extra records skipped.\u{1b}[0m"
                    );
                }
                break;
            }
            for rec in read.into_iter().flatten() {
                let mut s = mb_bseq1_t::default();
                kseq2bseq(rec, &mut s, with_qual, with_comment);
                size += s.l_seq as i64;
                a.push(s);
            }
            if size >= chunk_size {
                break;
            }
        }
        for f in fp.iter_mut().take(n_fp as usize) {
            if f.parse_error && !f.parse_error_reported && !f.suppress_parse_warnings {
                if let Some(name) = &f.parse_error_after {
                    eprintln!(
                        "[WARNING]\u{1b}[1;31m failed to parse the FASTA/FASTQ record next to '{}'. Continue anyway.\u{1b}[0m",
                        name
                    );
                } else {
                    eprintln!(
                        "[WARNING]\u{1b}[1;31m failed to parse the first FASTA/FASTQ record. Continue anyway.\u{1b}[0m"
                    );
                }
                f.parse_error_reported = true;
            }
        }
        *n_ = a.len() as i32;
        a
    }

    /// Original C global function `mb_bseq_eof` from `minibwa/bseq.c:163`.
    pub fn mb_bseq_eof(fp: &mb_bseq_file_t) -> i32 {
        (fp.eof && fp.s.is_none()) as i32
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn qname_helpers_strip_pair_suffixes() {
            assert_eq!(mb_qname_len("read/1"), 4);
            assert_eq!(mb_qname_len("read/x"), 6);
            assert_eq!(mb_qname_same("read/1", "read/2"), 1);
            assert_eq!(mb_qname_same("read/1", "readx/2"), 0);
        }

        #[test]
        fn revcomp_bseq_reverses_sequence_and_quality() {
            let mut s = mb_bseq1_t {
                l_seq: 4,
                seq: "ACGT".into(),
                qual: Some("abcd".into()),
                ..Default::default()
            };
            mb_revcomp_bseq(&mut s);
            assert_eq!(&*s.seq, "ACGT");
            assert_eq!(s.qual.as_deref(), Some("dcba"));
        }

        #[test]
        fn read_real_chrm_fasta_chunk() {
            let mut fp = mb_bseq_open(Some("minibwa/test/chrM-human.fa.gz")).expect("open fasta");
            let mut n = 0;
            let reads = mb_bseq_read(&mut fp, 1_000_000, 0, 1, 0, 1, 1_000_000, &mut n);
            assert_eq!(n, 1);
            assert_eq!(&*reads[0].name, "chrM");
            assert_eq!(reads[0].l_seq, 16569);
            assert!(reads[0].seq.starts_with("GATCACAGGTCTATCACCCT"));
            assert_eq!(mb_bseq_eof(&fp), 1);
        }

        #[test]
        fn read_real_paired_fragments_from_two_files() {
            let fp1 = mb_bseq_open(Some("minibwa/test/chrM-read_1.fa.gz")).expect("open read1");
            let fp2 = mb_bseq_open(Some("minibwa/test/chrM-read_2.fa.gz")).expect("open read2");
            let mut fps = vec![fp1, fp2];
            let mut n = 0;
            let reads = mb_bseq_read_frag(2, &mut fps, 200, 0, 0, &mut n);
            assert_eq!(n, 2);
            assert_eq!(reads.len(), 2);
            assert_eq!(mb_qname_same(&reads[0].name, &reads[1].name), 1);
            assert!(reads[0].l_seq > 0);
            assert!(reads[1].l_seq > 0);
        }

        #[test]
        fn read_sequence_keeps_internal_marker_characters() {
            let path = std::env::temp_dir().join(format!(
                "minibwa_rs_bseq_internal_markers_{}.fa",
                std::process::id()
            ));
            std::fs::write(&path, b">r0\nAC@GT+AC\n>r1\nTT\n").unwrap();
            let mut fp = mb_bseq_open(Some(&path.to_string_lossy())).expect("open marker fasta");
            let mut n = 0;
            let reads = mb_bseq_read(&mut fp, 1_000_000, 0, 0, 0, 1, 1_000_000, &mut n);
            assert_eq!(n, 2);
            assert_eq!(&*reads[0].name, "r0");
            assert_eq!(&*reads[0].seq, "AC@GT+AC");
            assert_eq!(&*reads[1].name, "r1");
            let _ = std::fs::remove_file(path);
        }
    }
}

pub mod bwt {
    #![allow(unused_variables, dead_code, non_snake_case, non_camel_case_types)]

    use std::fs::File;
    use std::io::{BufWriter, Read, Write};
    use std::path::Path;

    const MB_MAGIC: &[u8; 4] = b"MBW\x02";
    const BWT_CNT_SHIFT: u32 = 56;
    const BWT_CNT_MASK: u64 = (1u64 << BWT_CNT_SHIFT) - 1;

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct mb_sai_t {
        pub x: [u64; 2],
        pub size: u64,
        pub info: u64,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct mb_bwt_t {
        pub primary: u64,
        pub L2: [u64; 5],
        pub seq_len: u64,
        pub data_len: u64,
        pub data: Vec<u64>,
        pub cnt_table: [u32; 256],
        pub pre_len: u32,
        pub pre: Vec<mb_sai_t>,
        pub sa_bit: u32,
        pub n_sa: u64,
        pub sa: Vec<u64>,
    }

    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    pub struct mb_sai_v {
        pub n: usize,
        pub m: usize,
        pub a: Vec<mb_sai_t>,
    }

    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    pub struct mb_smem_entry_t {
        pub min_len: i32,
        pub min_occ: i32,
        pub st: i32,
        pub en: i32,
        pub q: Vec<u8>,
        pub v: mb_sai_v,
        pub stage: i32,
        pub x: i32,
        pub i: i32,
        pub kmer: i32,
        pub p: mb_sai_t,
    }

    #[derive(Debug)]
    pub struct mb_smem_entry_ref {
        pub min_len: i32,
        pub min_occ: i32,
        pub st: i32,
        pub en: i32,
        pub q: *const u8,
        pub v: *mut mb_sai_v,
        pub stage: i32,
        pub x: i32,
        pub i: i32,
        pub kmer: i32,
        pub p: mb_sai_t,
    }

    #[inline(always)]
    unsafe fn smem_q_ref(s: &mb_smem_entry_ref, i: i32) -> u8 {
        unsafe { *s.q.add(i as usize) }
    }

    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    pub struct tiny_queue_t {
        pub front: i32,
        pub count: i32,
        pub cap: i32,
        pub a: Vec<i32>,
    }

    /// Original C static function `bwt_gen_cnt_table` from `minibwa/bwt.c:14`.
    pub fn bwt_gen_cnt_table(cnt: &mut [u32; 256]) {
        for i in 0..256usize {
            let mut x = 0u32;
            for j in 0..4u32 {
                x |= (((i & 3) as u32 == j) as u32
                    + (((i >> 2) & 3) as u32 == j) as u32
                    + (((i >> 4) & 3) as u32 == j) as u32
                    + ((i >> 6) as u32 == j) as u32)
                    << (j << 3);
            }
            cnt[i] = x;
        }
    }

    /// Original C global function `mb_bwt_init` from `minibwa/bwt.c:25`.
    pub fn mb_bwt_init() -> mb_bwt_t {
        let mut bwt = mb_bwt_t {
            primary: 0,
            L2: [0; 5],
            seq_len: 0,
            data_len: 0,
            data: Vec::new(),
            cnt_table: [0; 256],
            pre_len: 0,
            pre: Vec::new(),
            sa_bit: u32::MAX,
            n_sa: 0,
            sa: Vec::new(),
        };
        bwt_gen_cnt_table(&mut bwt.cnt_table);
        bwt
    }

    /// Original C global function `mb_bwt_destroy` from `minibwa/bwt.c:34`.
    pub fn mb_bwt_destroy(bwt: Option<mb_bwt_t>) {
        drop(bwt);
    }

    /// Original C static function `mb_bwt_data_len` from `minibwa/bwt.c:50`.
    pub fn mb_bwt_data_len(len: u64) -> u64 {
        let bwt_len = (len + 127) / 128 * 4;
        let occ_len = ((len + 127) / 128 + 1) * 4;
        bwt_len + occ_len
    }

    /// Original C global function `mb_bwt_init_from_raw` from `minibwa/bwt.c:64`.
    pub fn mb_bwt_init_from_raw(is_byte: i32, raw: &[u8], len: u64, primary: u64) -> mb_bwt_t {
        let mut c = [0u64; 4];
        let mut x = [0u64; 4];
        let mut bwt = mb_bwt_init();
        bwt.primary = primary;
        bwt.seq_len = len;
        bwt.data_len = mb_bwt_data_len(len);
        bwt.data = Vec::with_capacity(bwt.data_len as usize);

        let mut last_c: Option<usize> = None;
        for i in 0..len {
            let a = if is_byte != 0 {
                raw[i as usize] & 3
            } else {
                let word_start = ((i >> 4) as usize) * 4;
                let b = u32::from_ne_bytes([
                    raw[word_start],
                    raw[word_start + 1],
                    raw[word_start + 2],
                    raw[word_start + 3],
                ]);
                ((b >> ((!i & 0xf) << 1)) & 3) as u8
            };
            if (i & 0x7f) == 0 {
                if i > 0 {
                    bwt.data.extend_from_slice(&x);
                }
                last_c = Some(bwt.data.len());
                bwt.data.extend_from_slice(&c);
                x = [0; 4];
            } else if (i & 0x3f) == 0 {
                if let Some(last_c) = last_c {
                    for j in 0..4usize {
                        bwt.data[last_c + j] |= (c[j] - bwt.data[last_c + j]) << BWT_CNT_SHIFT;
                    }
                }
            }
            c[a as usize] += 1;
            x[((i & 0x7f) >> 5) as usize] |= (a as u64) << ((i & 0x1f) << 1);
        }
        bwt.data.extend_from_slice(&x);
        bwt.data.extend_from_slice(&c);
        assert_eq!(bwt.data.len(), bwt.data_len as usize);
        bwt.L2[0] = 0;
        for i in 0..4usize {
            bwt.L2[i + 1] = bwt.L2[i] + c[i];
        }
        assert_eq!(bwt.L2[4], len);
        bwt
    }

    /// Original C static function `mb_bwt_set_intv` from `minibwa/bwt.h:71`.
    #[inline(always)]
    pub fn mb_bwt_set_intv(bwt: &mb_bwt_t, c: i32, ik: &mut mb_sai_t) {
        ik.x[0] = bwt.L2[c as usize] + 1;
        ik.x[1] = bwt.L2[(3 - c) as usize] + 1;
        ik.size = bwt.L2[c as usize + 1] - bwt.L2[c as usize];
        ik.info = 0;
    }

    /// Original C static function `mb_bwt_block_prefetch` from `minibwa/bwt.c:118`.
    #[inline(always)]
    pub fn mb_bwt_block_prefetch(bwt: &mb_bwt_t, k: u64) {
        if k > 0 && !bwt.data.is_empty() {
            let block = ((k - 1 - ((k - 1 >= bwt.primary) as u64)) >> 7) << 3;
            let cell = bwt.data.as_ptr().wrapping_add(block as usize);
            crate::s2n_lite::_mm_prefetch(cell as *const u8, 3);
        }
    }

    #[inline(always)]
    pub fn mb_bwt_pre_prefetch(bwt: &mb_bwt_t, kmer: i32) {
        if !bwt.pre.is_empty() {
            let p = unsafe { bwt.pre.as_ptr().add(kmer as usize) };
            crate::s2n_lite::_mm_prefetch(p as *const u8, 3);
        }
    }

    /// Original C static function `rank_aux1` from `minibwa/bwt.c:123`.
    #[inline(always)]
    pub fn rank_aux1(mut y: u64, c: u8) -> i32 {
        y = ((if (c & 2) != 0 { y } else { !y }) >> 1)
            & (if (c & 1) != 0 { y } else { !y })
            & 0x5555_5555_5555_5555u64;
        y.count_ones() as i32
    }

    /// Original C global function `mb_bwt_rank11` from `minibwa/bwt.c:136`.
    #[inline(always)]
    pub fn mb_bwt_rank11(bwt: &mb_bwt_t, mut k: u64, c: u8) -> u64 {
        if k == 0 {
            return 0;
        }
        if k == bwt.seq_len + 1 {
            return bwt.L2[c as usize + 1] - bwt.L2[c as usize];
        }
        k -= 1;
        k -= (k >= bwt.primary) as u64;
        let mask = if (k & 0x7f) >= 64 {
            (1u64 << (64 - BWT_CNT_SHIFT)) - 1
        } else {
            0
        };
        let block = ((k >> 7) << 3) as usize;
        let mut n = (bwt.data[block + c as usize] & BWT_CNT_MASK)
            + ((bwt.data[block + c as usize] >> BWT_CNT_SHIFT) & mask);
        let mut p = block + 4;
        let end = p + ((k & 0x7f) >> 5) as usize;
        if (k & 0x7f) >= 64 {
            p += 2;
        }
        if p < end {
            n += rank_aux1(bwt.data[p], c) as u64;
            p += 1;
        }
        n += rank_aux1(bwt.data[p] << ((!k & 0x1f) << 1), c) as u64;
        if c == 0 {
            n -= !k & 0x1f;
        }
        n
    }

    /// Original C static function `seek_block` from `minibwa/bwt.c:157`.
    #[inline(always)]
    pub fn seek_block(bwt: &mb_bwt_t, k: u64, cnt: &mut [u64; 4]) -> usize {
        let p = ((k >> 7) << 3) as usize;
        let mask = if (k & 0x7f) >= 64 {
            (1u64 << (64 - BWT_CNT_SHIFT)) - 1
        } else {
            0
        };
        cnt[0] = (bwt.data[p] & BWT_CNT_MASK) + ((bwt.data[p] >> BWT_CNT_SHIFT) & mask);
        cnt[1] = (bwt.data[p + 1] & BWT_CNT_MASK) + ((bwt.data[p + 1] >> BWT_CNT_SHIFT) & mask);
        cnt[2] = (bwt.data[p + 2] & BWT_CNT_MASK) + ((bwt.data[p + 2] >> BWT_CNT_SHIFT) & mask);
        cnt[3] = (bwt.data[p + 3] & BWT_CNT_MASK) + ((bwt.data[p + 3] >> BWT_CNT_SHIFT) & mask);
        (p + 4) * 2
    }

    /// Original C static function `rank_aux4` from `minibwa/bwt.c:168`.
    #[inline(always)]
    pub fn rank_aux4(bwt: &mb_bwt_t, x: u32) -> u32 {
        bwt.cnt_table[(x & 0xff) as usize]
            + bwt.cnt_table[((x >> 8) & 0xff) as usize]
            + bwt.cnt_table[((x >> 16) & 0xff) as usize]
            + bwt.cnt_table[(x >> 24) as usize]
    }

    #[inline(always)]
    unsafe fn bwt_word32(bwt: &mb_bwt_t, q: usize) -> u32 {
        unsafe { *(bwt.data.as_ptr() as *const u32).add(q) }
    }

    /// Original C global function `mb_bwt_rank1a` from `minibwa/bwt.c:173`.
    #[inline(always)]
    pub fn mb_bwt_rank1a(bwt: &mb_bwt_t, mut k: u64, cnt: &mut [u64; 4]) {
        if k == 0 {
            *cnt = [0; 4];
            return;
        }
        k -= 1;
        k -= (k >= bwt.primary) as u64;
        let mut q = seek_block(bwt, k, cnt);
        let end = q + ((k & 0x7f) >> 4) as usize;
        if (k & 0x7f) >= 64 {
            q += 4;
        }
        let mut x = 0u32;
        while q < end {
            x = x.wrapping_add(rank_aux4(bwt, unsafe { bwt_word32(bwt, q) }));
            q += 1;
        }
        let val = unsafe { bwt_word32(bwt, q) };
        let tmp = val << ((!k & 0xf) << 1);
        x = x.wrapping_add(rank_aux4(bwt, tmp).wrapping_sub((!k & 0xf) as u32));
        cnt[0] += (x & 0xff) as u64;
        cnt[1] += ((x >> 8) & 0xff) as u64;
        cnt[2] += ((x >> 16) & 0xff) as u64;
        cnt[3] += (x >> 24) as u64;
    }

    /// Original C global function `mb_bwt_rank2a` from `minibwa/bwt.c:192`.
    #[inline(always)]
    pub fn mb_bwt_rank2a(
        bwt: &mb_bwt_t,
        mut k: u64,
        mut l: u64,
        cntk: &mut [u64; 4],
        cntl: &mut [u64; 4],
    ) {
        let mut k1 = k.wrapping_sub(1);
        let mut l1 = l.wrapping_sub(1);
        k1 -= (k1 >= bwt.primary) as u64;
        l1 -= (l1 >= bwt.primary) as u64;
        mb_bwt_block_prefetch(bwt, k);
        if (k1 >> 7) != (l1 >> 7) || k == 0 || l == 0 {
            mb_bwt_block_prefetch(bwt, l);
            mb_bwt_rank1a(bwt, k, cntk);
            mb_bwt_rank1a(bwt, l, cntl);
        } else if l - k == 1 {
            let z = k - (k > bwt.primary) as u64;
            mb_bwt_rank1a(bwt, k, cntk);
            *cntl = *cntk;
            let base = (((z >> 7) << 3) + 4 + (((z & 127) >> 5) as u64)) as usize;
            let c = ((bwt.data[base] >> ((z & 31) << 1)) & 3) as usize;
            cntl[c] += 1;
        } else {
            k = k1;
            l = l1;
            let mut q = seek_block(bwt, k, cntk);
            let endk = q + ((k & 0x7f) >> 4) as usize;
            let endl = q + ((l & 0x7f) >> 4) as usize;
            if (k & 0x7f) >= 64 {
                q += 4;
            }
            let mut x = 0u32;
            while q < endk {
                x = x.wrapping_add(rank_aux4(bwt, unsafe { bwt_word32(bwt, q) }));
                q += 1;
            }
            let mut y = x;
            let val = unsafe { bwt_word32(bwt, q) };
            let tmp = val << ((!k & 0xf) << 1);
            x = x.wrapping_add(rank_aux4(bwt, tmp).wrapping_sub((!k & 0xf) as u32));
            while q < endl {
                y = y.wrapping_add(rank_aux4(bwt, unsafe { bwt_word32(bwt, q) }));
                q += 1;
            }
            let val = unsafe { bwt_word32(bwt, q) };
            let tmp = val << ((!l & 0xf) << 1);
            y = y.wrapping_add(rank_aux4(bwt, tmp).wrapping_sub((!l & 0xf) as u32));
            *cntl = *cntk;
            cntk[0] += (x & 0xff) as u64;
            cntk[1] += ((x >> 8) & 0xff) as u64;
            cntk[2] += ((x >> 16) & 0xff) as u64;
            cntk[3] += (x >> 24) as u64;
            cntl[0] += (y & 0xff) as u64;
            cntl[1] += ((y >> 8) & 0xff) as u64;
            cntl[2] += ((y >> 16) & 0xff) as u64;
            cntl[3] += (y >> 24) as u64;
        }
    }

    /// Original C global function `mb_bwt_extend` from `minibwa/bwt.c:234`.
    #[inline(always)]
    pub fn mb_bwt_extend(bwt: &mb_bwt_t, ik: &mb_sai_t, ok: &mut [mb_sai_t; 4], is_back: i32) {
        if is_back == 0 {
            mb_bwt_extend_forward(bwt, ik, ok);
            return;
        }
        mb_bwt_extend_back(bwt, ik, ok);
    }

    #[inline(always)]
    pub fn mb_bwt_extend_forward(bwt: &mb_bwt_t, ik: &mb_sai_t, ok: &mut [mb_sai_t; 4]) {
        let mut tk = [0u64; 4];
        let mut tl = [0u64; 4];
        mb_bwt_rank2a(bwt, ik.x[1], ik.x[1] + ik.size, &mut tk, &mut tl);
        ok[0].x[1] = bwt.L2[0] + 1 + tk[0];
        ok[1].x[1] = bwt.L2[1] + 1 + tk[1];
        ok[2].x[1] = bwt.L2[2] + 1 + tk[2];
        ok[3].x[1] = bwt.L2[3] + 1 + tk[3];
        tl[0] -= tk[0];
        tl[1] -= tk[1];
        tl[2] -= tk[2];
        tl[3] -= tk[3];
        ok[0].size = tl[0];
        ok[1].size = tl[1];
        ok[2].size = tl[2];
        ok[3].size = tl[3];
        ok[3].x[0] = ik.x[0] + ((ik.x[1] <= bwt.primary && ik.x[1] + ik.size > bwt.primary) as u64);
        ok[2].x[0] = ok[3].x[0] + tl[3];
        ok[1].x[0] = ok[2].x[0] + tl[2];
        ok[0].x[0] = ok[1].x[0] + tl[1];
    }

    #[inline(always)]
    pub fn mb_bwt_extend_back(bwt: &mb_bwt_t, ik: &mb_sai_t, ok: &mut [mb_sai_t; 4]) {
        let mut tk = [0u64; 4];
        let mut tl = [0u64; 4];
        mb_bwt_rank2a(bwt, ik.x[0], ik.x[0] + ik.size, &mut tk, &mut tl);
        ok[0].x[0] = bwt.L2[0] + 1 + tk[0];
        ok[1].x[0] = bwt.L2[1] + 1 + tk[1];
        ok[2].x[0] = bwt.L2[2] + 1 + tk[2];
        ok[3].x[0] = bwt.L2[3] + 1 + tk[3];
        tl[0] -= tk[0];
        tl[1] -= tk[1];
        tl[2] -= tk[2];
        tl[3] -= tk[3];
        ok[0].size = tl[0];
        ok[1].size = tl[1];
        ok[2].size = tl[2];
        ok[3].size = tl[3];
        ok[3].x[1] = ik.x[1] + ((ik.x[0] <= bwt.primary && ik.x[0] + ik.size > bwt.primary) as u64);
        ok[2].x[1] = ok[3].x[1] + tl[3];
        ok[1].x[1] = ok[2].x[1] + tl[2];
        ok[0].x[1] = ok[1].x[1] + tl[1];
    }

    /// Original C static function `mb_bwt_back` from `minibwa/bwt.c:250`.
    pub fn mb_bwt_back(
        f: &mb_bwt_t,
        q: &[u8],
        st: i64,
        pos: i64,
        min_occ: i64,
        p: &mut mb_sai_t,
    ) -> i64 {
        let mut i = pos - 1;
        let mut ok = [mb_sai_t::default(); 4];
        debug_assert!(q[pos as usize] < 4);
        if !f.pre.is_empty() && pos - st >= f.pre_len as i64 {
            let mut z = 0u64;
            let mut l = 0u32;
            i = pos;
            while l < f.pre_len {
                z = z << 2 | q[i as usize] as u64;
                i -= 1;
                l += 1;
            }
            debug_assert!(z < (1u64 << (f.pre_len * 2)));
            *p = f.pre[z as usize];
        } else {
            p.size = 0;
        }
        if p.size < min_occ as u64 {
            mb_bwt_set_intv(f, q[pos as usize] as i32, p);
            i = pos - 1;
        }
        while i >= st {
            let c = q[i as usize] as usize;
            if c > 3 {
                break;
            }
            mb_bwt_extend_back(f, p, &mut ok);
            if ok[c].size < min_occ as u64 {
                break;
            }
            *p = ok[c];
            i -= 1;
        }
        i
    }

    /// Original C global function `mb_bwt_smem` from `minibwa/bwt.c:277`.
    pub fn mb_bwt_smem(
        f: &mb_bwt_t,
        len: u32,
        q: &[u8],
        x: i64,
        min_len: i64,
        min_occ: i64,
        p: &mut mb_sai_t,
    ) -> i64 {
        let mut ik = mb_sai_t::default();
        let mut ok = [mb_sai_t::default(); 4];
        debug_assert!(len <= i32::MAX as u32);
        p.size = 0;
        ik.size = 0;
        if len as i64 - x < min_len {
            return len as i64;
        }
        let mut xn = -1i64;
        let mut i = x;
        while i < x + min_len {
            if q[i as usize] > 3 {
                xn = i;
            }
            i += 1;
        }
        if xn >= 0 {
            return xn + 1;
        }
        i = mb_bwt_back(f, q, x, x + min_len - 1, min_occ, &mut ik);
        if i >= x {
            return i + 1;
        }
        let mut j = x + min_len;
        while j < len as i64 {
            let c = 3 - q[j as usize] as i32;
            if q[j as usize] > 3 {
                break;
            }
            mb_bwt_extend_forward(f, &ik, &mut ok);
            if ok[c as usize].size < min_occ as u64 {
                break;
            }
            ik = ok[c as usize];
            j += 1;
        }
        *p = ik;
        p.info = (x as u64) << 32 | j as u64;
        if j == len as i64 {
            return len as i64;
        }
        i = if q[j as usize] > 3 {
            j
        } else {
            mb_bwt_back(f, q, x + 1, j, min_occ, &mut ik)
        };
        i + 1
    }

    #[inline(always)]
    pub fn tq_reset(_km: (), q: &mut tiny_queue_t, n: i32) {
        let mut cap = n;
        if cap <= 0 {
            q.cap = 0;
            q.front = 0;
            q.count = 0;
            return;
        }
        cap -= 1;
        cap |= cap >> 1;
        cap |= cap >> 2;
        cap |= cap >> 4;
        cap |= cap >> 8;
        cap |= cap >> 16;
        cap += 1;
        q.cap = cap;
        if q.a.len() < q.cap as usize {
            q.a.resize(q.cap as usize, 0);
        }
        q.front = 0;
        q.count = 0;
    }

    /// Original C static function `tq_init` from `minibwa/bwt.c:313`.
    #[inline(always)]
    pub fn tq_init(km: (), q: &mut tiny_queue_t, n: i32) {
        tq_reset(km, q, n);
    }

    /// Original C static function `tq_push` from `minibwa/bwt.c:321`.
    #[inline(always)]
    pub fn tq_push(q: &mut tiny_queue_t, x: i32) {
        q.a[((q.count + q.front) & (q.cap - 1)) as usize] = x;
        q.count += 1;
    }

    /// Original C static function `tq_shift` from `minibwa/bwt.c:326`.
    #[inline(always)]
    pub fn tq_shift(q: &mut tiny_queue_t) -> i32 {
        if q.count == 0 {
            return -1;
        }
        let x = q.a[q.front as usize];
        q.front += 1;
        q.front &= q.cap - 1;
        q.count -= 1;
        x
    }

    /// Original C static function `se_one_step_back` from `minibwa/bwt.c:336`.
    #[inline(always)]
    pub fn se_one_step_back(bwt: &mb_bwt_t, s: &mut mb_smem_entry_t) {
        let mut ok = [mb_sai_t::default(); 4];
        let c = s.q[s.i as usize] as i32;
        debug_assert!(c < 4);
        mb_bwt_extend_back(bwt, &s.p, &mut ok);
        if ok[c as usize].size < s.min_occ as u64 {
            s.x = s.i + 1;
            s.stage = 1;
        } else {
            s.p = ok[c as usize];
            s.i -= 1;
            mb_bwt_block_prefetch(bwt, s.p.x[0]);
            mb_bwt_block_prefetch(bwt, s.p.x[0] + s.p.size);
        }
    }

    /// Original C global function `mb_bwt_smem_batch` from `minibwa/bwt.c:353`.
    pub fn mb_bwt_smem_batch(km: (), bwt: &mb_bwt_t, n: i32, a: &mut [mb_smem_entry_t]) {
        let mut tq = tiny_queue_t::default();
        tq_init(km, &mut tq, n);
        for i in 0..n {
            let s = &mut a[i as usize];
            tq_push(&mut tq, i);
            s.stage = 1;
            s.x = s.st;
            if s.v.m < 64 {
                s.v.m = 64;
                s.v.a.reserve(s.v.m.saturating_sub(s.v.a.capacity()));
            }
        }

        while tq.count > 0 {
            let idx = tq_shift(&mut tq);
            let s = &mut a[idx as usize];
            if s.stage == 1 {
                if s.en - s.x < s.min_len {
                    continue;
                }
                let mut xn = -1;
                let mut i = s.x;
                while i < s.x + s.min_len {
                    if s.q[i as usize] > 3 {
                        xn = i;
                    }
                    i += 1;
                }
                if xn >= 0 {
                    s.x = xn + 1;
                } else {
                    s.i = s.x + s.min_len - 1;
                    if !bwt.pre.is_empty() && s.min_len >= bwt.pre_len as i32 {
                        s.kmer = 0;
                        let mut i = 0;
                        while i < bwt.pre_len as i32 {
                            s.kmer = s.kmer << 2 | s.q[s.i as usize] as i32;
                            s.i -= 1;
                            i += 1;
                        }
                        mb_bwt_pre_prefetch(bwt, s.kmer);
                        s.stage = 2;
                    } else {
                        mb_bwt_set_intv(bwt, s.q[s.i as usize] as i32, &mut s.p);
                        s.i -= 1;
                        s.stage = 3;
                    }
                }
            } else if s.stage == 2 || s.stage == 5 {
                s.p = bwt.pre[s.kmer as usize];
                if s.p.size < s.min_occ as u64 {
                    s.i += bwt.pre_len as i32;
                    mb_bwt_set_intv(bwt, s.q[s.i as usize] as i32, &mut s.p);
                    s.i -= 1;
                }
                s.stage += 1;
            } else if s.stage == 3 {
                if s.i < s.x {
                    mb_bwt_block_prefetch(bwt, s.p.x[1]);
                    mb_bwt_block_prefetch(bwt, s.p.x[1] + s.p.size);
                    s.i = s.x + s.min_len;
                    s.stage = 4;
                } else {
                    se_one_step_back(bwt, s);
                }
            } else if s.stage == 4 {
                if s.i == s.en {
                    s.p.info = (s.x as u64) << 32 | s.i as u64;
                    if s.v.n >= s.v.m {
                        s.v.m = s.v.n + 1;
                        s.v.m += (s.v.m >> 1) + 16;
                        s.v.a.reserve(s.v.m.saturating_sub(s.v.a.capacity()));
                    }
                    s.v.a.push(s.p);
                    s.v.n += 1;
                    continue;
                } else {
                    let c = 3 - s.q[s.i as usize] as i32;
                    let mut ok = [mb_sai_t::default(); 4];
                    if c >= 0 {
                        mb_bwt_extend_forward(bwt, &s.p, &mut ok);
                    }
                    if c >= 0 && ok[c as usize].size >= s.min_occ as u64 {
                        s.p = ok[c as usize];
                        s.i += 1;
                        mb_bwt_block_prefetch(bwt, s.p.x[1]);
                        mb_bwt_block_prefetch(bwt, s.p.x[1] + s.p.size);
                    } else {
                        s.p.info = (s.x as u64) << 32 | s.i as u64;
                        if s.v.n >= s.v.m {
                            s.v.m = s.v.n + 1;
                            s.v.m += (s.v.m >> 1) + 16;
                            s.v.a.reserve(s.v.m.saturating_sub(s.v.a.capacity()));
                        }
                        s.v.a.push(s.p);
                        s.v.n += 1;
                        if c < 0 {
                            s.x = s.i + 1;
                            s.stage = 1;
                        } else if !bwt.pre.is_empty() && s.i - s.x - 1 >= bwt.pre_len as i32 {
                            s.kmer = 0;
                            let mut i = 0;
                            while i < bwt.pre_len as i32 {
                                s.kmer = s.kmer << 2 | s.q[s.i as usize] as i32;
                                s.i -= 1;
                                i += 1;
                            }
                            mb_bwt_pre_prefetch(bwt, s.kmer);
                            s.stage = 5;
                        } else {
                            mb_bwt_set_intv(bwt, s.q[s.i as usize] as i32, &mut s.p);
                            s.i -= 1;
                            s.stage = 6;
                        }
                    }
                }
            } else if s.stage == 6 {
                if s.i < s.x + 1 {
                    s.x = s.i + 1;
                    s.stage = 1;
                } else {
                    se_one_step_back(bwt, s);
                }
            }
            tq_push(&mut tq, idx);
        }
    }

    #[inline(always)]
    pub fn se_one_step_back_ref(bwt: &mb_bwt_t, s: &mut mb_smem_entry_ref) {
        let mut ok = [mb_sai_t::default(); 4];
        let c = unsafe { smem_q_ref(s, s.i) } as i32;
        debug_assert!(c < 4);
        mb_bwt_extend_back(bwt, &s.p, &mut ok);
        if ok[c as usize].size < s.min_occ as u64 {
            s.x = s.i + 1;
            s.stage = 1;
        } else {
            s.p = ok[c as usize];
            s.i -= 1;
            mb_bwt_block_prefetch(bwt, s.p.x[0]);
            mb_bwt_block_prefetch(bwt, s.p.x[0] + s.p.size);
        }
    }

    pub fn mb_bwt_smem_batch_ref(
        km: (),
        bwt: &mb_bwt_t,
        n: i32,
        a: &mut [mb_smem_entry_ref],
        v: &mut [mb_sai_v],
    ) {
        let mut tq = tiny_queue_t::default();
        mb_bwt_smem_batch_ref_with_queue(km, bwt, n, a, v, &mut tq);
    }

    pub fn mb_bwt_smem_batch_ref_with_queue(
        km: (),
        bwt: &mb_bwt_t,
        n: i32,
        a: &mut [mb_smem_entry_ref],
        _v: &mut [mb_sai_v],
        tq: &mut tiny_queue_t,
    ) {
        tq_reset(km, tq, n);
        for i in 0..n {
            let s = unsafe { a.get_unchecked_mut(i as usize) };
            tq_push(tq, i);
            s.stage = 1;
            s.x = s.st;
            let sv = unsafe { &mut *s.v };
            if sv.m < 64 {
                sv.m = 64;
                sv.a.reserve(sv.m.saturating_sub(sv.a.capacity()));
            }
        }

        while tq.count > 0 {
            let idx = tq_shift(tq);
            let s = unsafe { a.get_unchecked_mut(idx as usize) };
            if s.stage == 1 {
                if s.en - s.x < s.min_len {
                    continue;
                }
                let mut xn = -1;
                let mut i = s.x;
                while i < s.x + s.min_len {
                    if unsafe { smem_q_ref(s, i) } > 3 {
                        xn = i;
                    }
                    i += 1;
                }
                if xn >= 0 {
                    s.x = xn + 1;
                } else {
                    s.i = s.x + s.min_len - 1;
                    if !bwt.pre.is_empty() && s.min_len >= bwt.pre_len as i32 {
                        s.kmer = 0;
                        let mut i = 0;
                        while i < bwt.pre_len as i32 {
                            s.kmer = s.kmer << 2 | unsafe { smem_q_ref(s, s.i) } as i32;
                            s.i -= 1;
                            i += 1;
                        }
                        mb_bwt_pre_prefetch(bwt, s.kmer);
                        s.stage = 2;
                    } else {
                        mb_bwt_set_intv(bwt, unsafe { smem_q_ref(s, s.i) } as i32, &mut s.p);
                        s.i -= 1;
                        s.stage = 3;
                    }
                }
            } else if s.stage == 2 || s.stage == 5 {
                s.p = bwt.pre[s.kmer as usize];
                if s.p.size < s.min_occ as u64 {
                    s.i += bwt.pre_len as i32;
                    mb_bwt_set_intv(bwt, unsafe { smem_q_ref(s, s.i) } as i32, &mut s.p);
                    s.i -= 1;
                }
                s.stage += 1;
            } else if s.stage == 3 {
                if s.i < s.x {
                    mb_bwt_block_prefetch(bwt, s.p.x[1]);
                    mb_bwt_block_prefetch(bwt, s.p.x[1] + s.p.size);
                    s.i = s.x + s.min_len;
                    s.stage = 4;
                } else {
                    se_one_step_back_ref(bwt, s);
                }
            } else if s.stage == 4 {
                if s.i == s.en {
                    s.p.info = (s.x as u64) << 32 | s.i as u64;
                    let sv = unsafe { &mut *s.v };
                    if sv.n >= sv.m {
                        sv.m = sv.n + 1;
                        sv.m += (sv.m >> 1) + 16;
                        sv.a.reserve(sv.m.saturating_sub(sv.a.capacity()));
                    }
                    sv.a.push(s.p);
                    sv.n += 1;
                    continue;
                } else {
                    let c = 3 - unsafe { smem_q_ref(s, s.i) } as i32;
                    let mut ok = [mb_sai_t::default(); 4];
                    if c >= 0 {
                        mb_bwt_extend_forward(bwt, &s.p, &mut ok);
                    }
                    if c >= 0 && ok[c as usize].size >= s.min_occ as u64 {
                        s.p = ok[c as usize];
                        s.i += 1;
                        mb_bwt_block_prefetch(bwt, s.p.x[1]);
                        mb_bwt_block_prefetch(bwt, s.p.x[1] + s.p.size);
                    } else {
                        s.p.info = (s.x as u64) << 32 | s.i as u64;
                        let sv = unsafe { &mut *s.v };
                        if sv.n >= sv.m {
                            sv.m = sv.n + 1;
                            sv.m += (sv.m >> 1) + 16;
                            sv.a.reserve(sv.m.saturating_sub(sv.a.capacity()));
                        }
                        sv.a.push(s.p);
                        sv.n += 1;
                        if c < 0 {
                            s.x = s.i + 1;
                            s.stage = 1;
                        } else if !bwt.pre.is_empty() && s.i - s.x - 1 >= bwt.pre_len as i32 {
                            s.kmer = 0;
                            let mut i = 0;
                            while i < bwt.pre_len as i32 {
                                s.kmer = s.kmer << 2 | unsafe { smem_q_ref(s, s.i) } as i32;
                                s.i -= 1;
                                i += 1;
                            }
                            mb_bwt_pre_prefetch(bwt, s.kmer);
                            s.stage = 5;
                        } else {
                            mb_bwt_set_intv(bwt, unsafe { smem_q_ref(s, s.i) } as i32, &mut s.p);
                            s.i -= 1;
                            s.stage = 6;
                        }
                    }
                }
            } else if s.stage == 6 {
                if s.i < s.x + 1 {
                    s.x = s.i + 1;
                    s.stage = 1;
                } else {
                    se_one_step_back_ref(bwt, s);
                }
            }
            tq_push(tq, idx);
        }
    }

    /// Original C static function `bwt_invPsi` from `minibwa/bwt.c:460`.
    #[inline(always)]
    pub fn bwt_invPsi(bwt: &mb_bwt_t, k: u64) -> u64 {
        let x = k - (k > bwt.primary) as u64;
        let block = (((x >> 7) << 3) + 4 + ((x & 127) >> 5)) as usize;
        let c = ((bwt.data[block] >> ((x & 31) << 1)) & 3) as u8;
        let next = bwt.L2[c as usize] + 1 + mb_bwt_rank11(bwt, k, c);
        if k == bwt.primary {
            0
        } else {
            next
        }
    }

    /// Original C global function `mb_bwt_gen_sa` from `minibwa/bwt.c:469`.
    pub fn mb_bwt_gen_sa(bwt: &mut mb_bwt_t, sa_bit: u32) {
        assert!(!bwt.data.is_empty());
        bwt.sa.clear();
        bwt.sa_bit = sa_bit;
        bwt.n_sa = (bwt.seq_len + (1u64 << sa_bit)) >> sa_bit;
        bwt.sa = vec![0; bwt.n_sa as usize];

        let mut isa = 0u64;
        let mut sa = bwt.seq_len;
        let mask = (1u64 << sa_bit) - 1;
        for _ in 0..bwt.seq_len {
            if (isa & mask) == 0 {
                bwt.sa[(isa >> bwt.sa_bit) as usize] = sa;
            }
            sa = sa.wrapping_sub(1);
            isa = bwt_invPsi(bwt, isa);
        }
        if (isa & mask) == 0 {
            bwt.sa[(isa >> bwt.sa_bit) as usize] = sa;
        }
        bwt.sa[0] = u64::MAX;
    }

    /// Original C global function `mb_bwt_sa` from `minibwa/bwt.c:490`.
    #[inline(always)]
    pub fn mb_bwt_sa(bwt: &mb_bwt_t, mut k: u64) -> u64 {
        let mut sa = 0u64;
        let mask = (1u64 << bwt.sa_bit) - 1;
        while (k & mask) != 0 {
            sa += 1;
            k = bwt_invPsi(bwt, k);
        }
        sa.wrapping_add(bwt.sa[(k >> bwt.sa_bit) as usize])
    }

    /// Original C global function `mb_bwt_sa_batch` from `minibwa/bwt.c:502`.
    #[inline]
    pub fn mb_bwt_sa_batch(km: (), bwt: &mb_bwt_t, n: i64, x: &mut [u64]) {
        let mut z = Vec::new();
        mb_bwt_sa_batch_with_scratch(km, bwt, n, x, &mut z);
    }

    #[inline]
    pub fn mb_bwt_sa_batch_with_scratch(
        _km: (),
        bwt: &mb_bwt_t,
        n: i64,
        x: &mut [u64],
        z: &mut Vec<(u64, u64)>,
    ) {
        let mask = (1u64 << bwt.sa_bit) - 1;
        if n <= 0 {
            return;
        }
        z.clear();
        z.resize(n as usize, (0, 0));
        let n = n as usize;
        let z_ptr = z.as_mut_ptr();
        let x_ptr = x.as_mut_ptr();
        let sa_ptr = bwt.sa.as_ptr();
        for i in 0..n {
            let x_i = unsafe { *x_ptr.add(i) };
            unsafe {
                (*z_ptr.add(i)).0 = x_i;
                (*z_ptr.add(i)).1 = i as u64;
            }
            if (x_i & mask) == 0 {
                let sa = unsafe { sa_ptr.add((x_i >> bwt.sa_bit) as usize) };
                crate::s2n_lite::_mm_prefetch(sa as *const u8, 3);
            } else {
                mb_bwt_block_prefetch(bwt, x_i);
            }
        }
        let mut step = 0u64;
        let mut r = n;
        while r > 0 {
            let r0 = r;
            r = 0;
            for i in 0..r0 {
                let zi = unsafe { *z_ptr.add(i) };
                if (zi.0 & mask) == 0 {
                    unsafe {
                        *x_ptr.add(zi.1 as usize) =
                            step.wrapping_add(*sa_ptr.add((zi.0 >> bwt.sa_bit) as usize));
                    }
                } else {
                    unsafe { *z_ptr.add(r) = zi };
                    r += 1;
                }
            }
            for i in 0..r {
                let x_i = bwt_invPsi(bwt, unsafe { (*z_ptr.add(i)).0 });
                unsafe {
                    (*z_ptr.add(i)).0 = x_i;
                }
                if (x_i & mask) == 0 {
                    let sa = unsafe { sa_ptr.add((x_i >> bwt.sa_bit) as usize) };
                    crate::s2n_lite::_mm_prefetch(sa as *const u8, 3);
                } else {
                    mb_bwt_block_prefetch(bwt, x_i);
                }
            }
            step += 1;
        }
    }

    /// Original C global function `mb_bwt_count_kmer` from `minibwa/bwt.c:543`.
    pub fn mb_bwt_count_kmer(bwt: &mb_bwt_t, depth: i32, s: &mut [mb_sai_t]) {
        #[derive(Clone, Copy, Debug, Default)]
        struct count_pair64_t {
            p: mb_sai_t,
            d: i32,
            c: i32,
        }

        let mut stack = [count_pair64_t::default(); 64];
        let mut s_top = 0usize;
        let mut str_ = [0u8; 16];
        assert!(depth <= 15);
        for a in 0..4i32 {
            mb_bwt_set_intv(bwt, a, &mut stack[s_top].p);
            stack[s_top].d = 1;
            stack[s_top].c = a;
            s_top += 1;
        }
        while s_top > 0 {
            s_top -= 1;
            let top = stack[s_top];
            let mut ok = [mb_sai_t::default(); 4];
            if top.d > 0 {
                str_[(depth - top.d) as usize] = top.c as u8;
            }
            mb_bwt_extend_back(bwt, &top.p, &mut ok);
            for a in 0..4i32 {
                str_[(depth - top.d - 1) as usize] = a as u8;
                if top.d != depth - 1 {
                    stack[s_top].p = ok[a as usize];
                    stack[s_top].d = top.d + 1;
                    stack[s_top].c = a;
                    s_top += 1;
                } else {
                    let mut x = 0u64;
                    for i in 0..depth {
                        x |= (str_[i as usize] as u64) << (i * 2);
                    }
                    s[x as usize] = ok[a as usize];
                }
            }
        }
    }

    fn mb_bwt_count_kmer_uninit(
        bwt: &mb_bwt_t,
        depth: i32,
        s: &mut [std::mem::MaybeUninit<mb_sai_t>],
    ) {
        #[derive(Clone, Copy, Debug, Default)]
        struct count_pair64_t {
            p: mb_sai_t,
            d: i32,
            c: i32,
        }

        let mut stack = [count_pair64_t::default(); 64];
        let mut s_top = 0usize;
        let mut str_ = [0u8; 16];
        assert!(depth <= 15);
        for a in 0..4i32 {
            mb_bwt_set_intv(bwt, a, &mut stack[s_top].p);
            stack[s_top].d = 1;
            stack[s_top].c = a;
            s_top += 1;
        }
        while s_top > 0 {
            s_top -= 1;
            let top = stack[s_top];
            let mut ok = [mb_sai_t::default(); 4];
            if top.d > 0 {
                str_[(depth - top.d) as usize] = top.c as u8;
            }
            mb_bwt_extend_back(bwt, &top.p, &mut ok);
            for a in 0..4i32 {
                str_[(depth - top.d - 1) as usize] = a as u8;
                if top.d != depth - 1 {
                    stack[s_top].p = ok[a as usize];
                    stack[s_top].d = top.d + 1;
                    stack[s_top].c = a;
                    s_top += 1;
                } else {
                    let mut x = 0u64;
                    for i in 0..depth {
                        x |= (str_[i as usize] as u64) << (i * 2);
                    }
                    s[x as usize].write(ok[a as usize]);
                }
            }
        }
    }

    /// Original C global function `mb_bwt_cache` from `minibwa/bwt.c:576`.
    pub fn mb_bwt_cache(bwt: &mut mb_bwt_t, len: i32) {
        bwt.pre.clear();
        bwt.pre_len = len as u32;
        let n = 1usize << (len * 2);
        let mut pre = Vec::<std::mem::MaybeUninit<mb_sai_t>>::with_capacity(n);
        let pre_slice = unsafe { std::slice::from_raw_parts_mut(pre.as_mut_ptr(), n) };
        mb_bwt_count_kmer_uninit(bwt, len, pre_slice);
        let ptr = pre.as_mut_ptr() as *mut mb_sai_t;
        let cap = pre.capacity();
        std::mem::forget(pre);
        let pre = unsafe { Vec::from_raw_parts(ptr, n, cap) };
        bwt.pre = pre;
    }

    /// Original C static function `read_huge` from `minibwa/bwt.c:588`.
    pub fn read_huge<R: Read>(fp: &mut R, size: u64, a: &mut [u8]) -> u64 {
        let bufsize = 0x1000000usize;
        let mut offset = 0usize;
        let mut remaining = size as usize;
        while remaining > 0 {
            let x = bufsize.min(remaining);
            match fp.read(&mut a[offset..offset + x]) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    remaining -= n;
                    offset += n;
                }
            }
        }
        offset as u64
    }

    fn read_huge_u64_vec<R: Read>(fp: &mut R, n: u64) -> Option<Vec<u64>> {
        let mut a = Vec::<u64>::with_capacity(n as usize);
        let bytes =
            unsafe { std::slice::from_raw_parts_mut(a.as_mut_ptr() as *mut u8, n as usize * 8) };
        if read_huge(fp, n.checked_mul(8)?, bytes) != n * 8 {
            return None;
        }
        unsafe {
            a.set_len(n as usize);
        }
        #[cfg(target_endian = "big")]
        {
            for x in &mut a {
                *x = u64::from_le(*x);
            }
        }
        Some(a)
    }

    /// Original C global function `mb_bwt_load_raw` from `minibwa/bwt.c:600`.
    pub fn mb_bwt_load_raw<P: AsRef<Path>>(fn_: P) -> Option<mb_bwt_t> {
        let mut fp = File::open(fn_).ok()?;
        let file_len = fp.metadata().ok()?.len();
        let raw_size = (file_len.checked_sub(std::mem::size_of::<u64>() as u64 * 5)?) >> 2;
        let mut hdr = [0u8; 40];
        fp.read_exact(&mut hdr).ok()?;
        let primary = u64::from_le_bytes(hdr[0..8].try_into().unwrap());
        let mut L2 = [0u64; 5];
        for i in 1..5usize {
            let start = i * 8;
            L2[i] = u64::from_le_bytes(hdr[start..start + 8].try_into().unwrap());
        }
        let mut raw = vec![0u8; (raw_size << 2) as usize];
        read_huge(&mut fp, raw_size << 2, &mut raw);
        Some(mb_bwt_init_from_raw(0, &raw, L2[4], primary))
    }

    /// Original C global function `mb_bwt_save` from `minibwa/bwt.c:622`.
    pub fn mb_bwt_save<P: AsRef<Path>>(fn_: P, bwt: &mb_bwt_t) -> i32 {
        let fp = match File::create(fn_) {
            Ok(fp) => fp,
            Err(_) => return -1,
        };
        let mut fp = BufWriter::with_capacity(1 << 20, fp);
        if fp.write_all(MB_MAGIC).is_err() {
            return -1;
        }
        if fp.write_all(&bwt.sa_bit.to_le_bytes()).is_err() {
            return -1;
        }
        if fp.write_all(&bwt.primary.to_le_bytes()).is_err() {
            return -1;
        }
        for i in 1..5usize {
            if fp.write_all(&bwt.L2[i].to_le_bytes()).is_err() {
                return -1;
            }
        }
        if write_u64_slice_le(&mut fp, &bwt.data[..bwt.data_len as usize]).is_err() {
            return -1;
        }
        if fp.write_all(&bwt.n_sa.to_le_bytes()).is_err() {
            return -1;
        }
        if bwt.sa_bit != u32::MAX && bwt.n_sa > 0 && !bwt.sa.is_empty() {
            if write_u64_slice_le(&mut fp, &bwt.sa[..bwt.n_sa as usize]).is_err() {
                return -1;
            }
        }
        if fp.flush().is_err() {
            -1
        } else {
            0
        }
    }

    fn write_u64_slice_le<W: Write>(writer: &mut W, words: &[u64]) -> std::io::Result<()> {
        #[cfg(target_endian = "little")]
        {
            let bytes = unsafe {
                std::slice::from_raw_parts(
                    words.as_ptr().cast::<u8>(),
                    std::mem::size_of_val(words),
                )
            };
            writer.write_all(bytes)
        }
        #[cfg(not(target_endian = "little"))]
        {
            for &word in words {
                writer.write_all(&word.to_le_bytes())?;
            }
            Ok(())
        }
    }

    /// Original C global function `mb_bwt_load` from `minibwa/bwt.c:639`.
    pub fn mb_bwt_load<P: AsRef<Path>>(fn_: P) -> Option<mb_bwt_t> {
        let mut fp = File::open(fn_).ok()?;
        let mut magic = [0u8; 4];
        fp.read_exact(&mut magic).ok()?;
        if &magic != MB_MAGIC {
            return None;
        }
        let mut sa_bit_buf = [0u8; 4];
        fp.read_exact(&mut sa_bit_buf).ok()?;
        let mut x = [0u8; 40];
        fp.read_exact(&mut x).ok()?;
        let mut bwt = mb_bwt_init();
        bwt.sa_bit = u32::from_le_bytes(sa_bit_buf);
        bwt.primary = u64::from_le_bytes(x[0..8].try_into().unwrap());
        for i in 1..5usize {
            let start = i * 8;
            bwt.L2[i] = u64::from_le_bytes(x[start..start + 8].try_into().unwrap());
        }
        bwt.seq_len = bwt.L2[4];
        bwt.data_len = mb_bwt_data_len(bwt.seq_len);
        bwt.data = read_huge_u64_vec(&mut fp, bwt.data_len)?;
        let mut n_sa = [0u8; 8];
        fp.read_exact(&mut n_sa).ok()?;
        bwt.n_sa = u64::from_le_bytes(n_sa);
        if bwt.sa_bit != u32::MAX && bwt.n_sa > 0 {
            bwt.sa = read_huge_u64_vec(&mut fp, bwt.n_sa)?;
        }
        Some(bwt)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn rank_matches_bruteforce_for_constructed_byte_bwt() {
            let raw: Vec<u8> = (0..257).map(|i| ((i * 7 + i / 3) & 3) as u8).collect();
            let primary = 73;
            let bwt = mb_bwt_init_from_raw(1, &raw, raw.len() as u64, primary);

            for k in 0..=raw.len() as u64 + 1 {
                let mut cnt = [0u64; 4];
                mb_bwt_rank1a(&bwt, k, &mut cnt);
                for c in 0..4u8 {
                    let mut brute = 0u64;
                    for j in 0..k {
                        if j == primary {
                            continue;
                        }
                        let z = j - (j > primary) as u64;
                        if raw[z as usize] == c {
                            brute += 1;
                        }
                    }
                    assert_eq!(mb_bwt_rank11(&bwt, k, c), brute, "rank11 k={k} c={c}");
                    assert_eq!(cnt[c as usize], brute, "rank1a k={k} c={c}");
                }
            }
        }

        #[test]
        fn rank2_matches_two_rank1_queries() {
            let raw: Vec<u8> = (0..513).map(|i| ((i * 5 + i / 11) & 3) as u8).collect();
            let bwt = mb_bwt_init_from_raw(1, &raw, raw.len() as u64, 211);

            for &(k, l) in &[
                (0, 0),
                (0, 1),
                (1, 2),
                (31, 32),
                (63, 65),
                (128, 191),
                (212, 320),
                (500, 514),
            ] {
                let mut cntk = [0u64; 4];
                let mut cntl = [0u64; 4];
                let mut expect_k = [0u64; 4];
                let mut expect_l = [0u64; 4];
                mb_bwt_rank2a(&bwt, k, l, &mut cntk, &mut cntl);
                mb_bwt_rank1a(&bwt, k, &mut expect_k);
                mb_bwt_rank1a(&bwt, l, &mut expect_l);
                assert_eq!(cntk, expect_k, "k={k} l={l}");
                assert_eq!(cntl, expect_l, "k={k} l={l}");
            }
        }

        #[test]
        fn real_chrM_index_rank_vectors_are_self_consistent() {
            let bytes = std::fs::read("minibwa/chrM-human.mbw").expect("read chrM-human.mbw");
            assert_eq!(&bytes[0..4], b"MBW\x02");
            let sa_bit = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
            let primary = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
            let mut bwt = mb_bwt_init();
            bwt.sa_bit = sa_bit;
            bwt.primary = primary;
            for i in 1..5usize {
                let start = 16 + (i - 1) * 8;
                bwt.L2[i] = u64::from_le_bytes(bytes[start..start + 8].try_into().unwrap());
            }
            bwt.seq_len = bwt.L2[4];
            bwt.data_len = mb_bwt_data_len(bwt.seq_len);
            let data_start = 48usize;
            bwt.data = Vec::with_capacity(bwt.data_len as usize);
            for i in 0..bwt.data_len as usize {
                let start = data_start + i * 8;
                bwt.data.push(u64::from_le_bytes(
                    bytes[start..start + 8].try_into().unwrap(),
                ));
            }

            for &k in &[
                0,
                1,
                17,
                63,
                64,
                65,
                127,
                128,
                129,
                primary,
                primary + 1,
                bwt.seq_len,
                bwt.seq_len + 1,
            ] {
                let mut cnt = [0u64; 4];
                mb_bwt_rank1a(&bwt, k, &mut cnt);
                for c in 0..4u8 {
                    assert_eq!(
                        cnt[c as usize],
                        mb_bwt_rank11(&bwt, k, c),
                        "real k={k} c={c}"
                    );
                }
            }
        }

        #[test]
        fn batched_smem_matches_repeated_single_smem_on_real_read() {
            let bytes = std::fs::read("minibwa/chrM-human.mbw").expect("read chrM-human.mbw");
            assert_eq!(&bytes[0..4], b"MBW\x02");
            let mut bwt = mb_bwt_init();
            bwt.sa_bit = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
            bwt.primary = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
            for i in 1..5usize {
                let start = 16 + (i - 1) * 8;
                bwt.L2[i] = u64::from_le_bytes(bytes[start..start + 8].try_into().unwrap());
            }
            bwt.seq_len = bwt.L2[4];
            bwt.data_len = mb_bwt_data_len(bwt.seq_len);
            for i in 0..bwt.data_len as usize {
                let start = 48 + i * 8;
                bwt.data.push(u64::from_le_bytes(
                    bytes[start..start + 8].try_into().unwrap(),
                ));
            }

            let read = b"ACTCACCTGAGTTGTAAAAAACTCCAGTTGACACAAAATAGACTACGAAAGTGGCTTTAACATATCTGAACACACAATAGCTAAGACCCAAACTGGGATTAGATACCCCACTATGCTTAGCCCTAAACCTCAACAGTTAAATCAACAAAAC";
            let q: Vec<u8> = read
                .iter()
                .map(|&b| match b {
                    b'A' | b'a' => 0,
                    b'C' | b'c' => 1,
                    b'G' | b'g' => 2,
                    b'T' | b't' => 3,
                    _ => 4,
                })
                .collect();
            let mut x = 0i64;
            let mut single = Vec::new();
            while x < q.len() as i64 {
                let mut p = mb_sai_t::default();
                x = mb_bwt_smem(&bwt, q.len() as u32, &q, x, 19, 1, &mut p);
                if p.size > 0 {
                    single.push(p);
                }
            }

            let mut entries = [mb_smem_entry_t {
                min_len: 19,
                min_occ: 1,
                st: 0,
                en: q.len() as i32,
                q,
                v: mb_sai_v::default(),
                stage: 0,
                x: 0,
                i: 0,
                kmer: 0,
                p: mb_sai_t::default(),
            }];
            mb_bwt_smem_batch((), &bwt, 1, &mut entries);
            assert_eq!(entries[0].v.a, single);
            assert_eq!(entries[0].v.n, single.len());
        }

        #[test]
        fn load_save_roundtrip_preserves_real_chrM_index() {
            let bwt = mb_bwt_load("minibwa/chrM-human.mbw").expect("load chrM-human.mbw");
            let path = std::env::temp_dir().join("minibwa-rs-roundtrip.mbw");
            assert_eq!(mb_bwt_save(&path, &bwt), 0);
            let loaded = mb_bwt_load(&path).expect("reload saved BWT");
            let _ = std::fs::remove_file(&path);
            assert_eq!(loaded.primary, bwt.primary);
            assert_eq!(loaded.L2, bwt.L2);
            assert_eq!(loaded.seq_len, bwt.seq_len);
            assert_eq!(loaded.data_len, bwt.data_len);
            assert_eq!(loaded.data, bwt.data);
            assert_eq!(loaded.sa_bit, bwt.sa_bit);
            assert_eq!(loaded.n_sa, bwt.n_sa);
            assert_eq!(loaded.sa, bwt.sa);
        }

        #[test]
        fn generated_sa_matches_stored_real_chrM_sa() {
            let mut generated = mb_bwt_load("minibwa/chrM-human.mbw").expect("load chrM-human.mbw");
            let stored_sa = generated.sa.clone();
            let sa_bit = generated.sa_bit;
            mb_bwt_gen_sa(&mut generated, sa_bit);
            assert_eq!(generated.sa, stored_sa);
        }

        #[test]
        fn sa_batch_matches_single_sa_queries_on_real_chrM() {
            let bwt = mb_bwt_load("minibwa/chrM-human.mbw").expect("load chrM-human.mbw");
            let mut xs = vec![
                0,
                1,
                2,
                15,
                16,
                17,
                63,
                64,
                65,
                bwt.primary.saturating_sub(1),
                bwt.primary,
                bwt.primary + 1,
                bwt.seq_len,
            ];
            let expected: Vec<u64> = xs.iter().map(|&x| mb_bwt_sa(&bwt, x)).collect();
            mb_bwt_sa_batch((), &bwt, xs.len() as i64, &mut xs);
            assert_eq!(xs, expected);
        }

        #[test]
        fn kmer_cache_matches_direct_kmer_counting() {
            let mut bwt = mb_bwt_load("minibwa/chrM-human.mbw").expect("load chrM-human.mbw");
            let mut expected = vec![mb_sai_t::default(); 1usize << 6];
            mb_bwt_count_kmer(&bwt, 3, &mut expected);
            mb_bwt_cache(&mut bwt, 3);
            assert_eq!(bwt.pre_len, 3);
            assert_eq!(bwt.pre, expected);
        }

        #[test]
        fn load_raw_matches_init_from_word_packed_raw() {
            let raw_bases: Vec<u8> = (0..201).map(|i| ((i * 13 + 1) & 3) as u8).collect();
            let mut packed = vec![0u8; ((raw_bases.len() + 15) / 16) * 4];
            for (i, &base) in raw_bases.iter().enumerate() {
                let word = i >> 4;
                let shift = ((!i & 0xf) << 1) as u32;
                let mut val =
                    u32::from_le_bytes(packed[word * 4..word * 4 + 4].try_into().unwrap());
                val |= (base as u32) << shift;
                packed[word * 4..word * 4 + 4].copy_from_slice(&val.to_le_bytes());
            }
            let expected = mb_bwt_init_from_raw(0, &packed, raw_bases.len() as u64, 37);
            let path = std::env::temp_dir().join("minibwa-rs-raw.mbw");
            {
                let mut fp = File::create(&path).expect("create raw BWT");
                fp.write_all(&37u64.to_le_bytes()).unwrap();
                fp.write_all(&expected.L2[1].to_le_bytes()).unwrap();
                fp.write_all(&expected.L2[2].to_le_bytes()).unwrap();
                fp.write_all(&expected.L2[3].to_le_bytes()).unwrap();
                fp.write_all(&expected.L2[4].to_le_bytes()).unwrap();
                fp.write_all(&packed).unwrap();
            }
            let loaded = mb_bwt_load_raw(&path).expect("load raw BWT");
            let _ = std::fs::remove_file(&path);
            assert_eq!(loaded, expected);
        }
    }
}

pub mod bwtgen {
    #![allow(unused_variables, dead_code, non_snake_case, non_camel_case_types)]

    use crate::QSufSort::QSufSortSuffixSort;

    pub type bgint_t = u64;
    pub type sbgint_t = i64;

    pub const ALPHABET_SIZE: usize = 4;
    pub const BIT_PER_CHAR: u32 = 2;
    pub const CHAR_PER_WORD: u64 = 16;
    pub const CHAR_PER_BYTE: u32 = 4;
    pub const BITS_IN_WORD: u32 = 32;
    pub const BITS_IN_BYTE: u32 = 8;
    pub const BYTES_IN_WORD: usize = 4;
    pub const ALL_ONE_MASK: u32 = 0xffff_ffff;
    pub const DNA_OCC_CNT_TABLE_SIZE_IN_WORD: usize = 65536;
    pub const BITS_PER_OCC_VALUE: u32 = 16;
    pub const OCC_VALUE_PER_WORD: u64 = 2;
    pub const OCC_INTERVAL: u64 = 256;
    pub const OCC_INTERVAL_MAJOR: u64 = 65536;
    pub const MIN_AVAILABLE_WORD: u64 = 0x10000;
    pub const BWTINC_INSERT_SORT_NUM_ITEM: i64 = 7;

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct BWT {
        pub textLength: bgint_t,
        pub inverseSa0: bgint_t,
        pub cumulativeFreq: Vec<bgint_t>,
        pub bwtCode: Vec<u32>,
        pub occValue: Vec<u32>,
        pub occValueMajor: Vec<bgint_t>,
        pub decodeTable: Vec<u32>,
        pub bwtSizeInWord: bgint_t,
        pub occSizeInWord: bgint_t,
        pub occMajorSizeInWord: bgint_t,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct BWTInc {
        pub bwt: BWT,
        pub numberOfIterationDone: u32,
        pub cumulativeCountInCurrentBuild: Vec<bgint_t>,
        pub availableWord: bgint_t,
        pub buildSize: bgint_t,
        pub initialMaxBuildSize: bgint_t,
        pub incMaxBuildSize: bgint_t,
        pub firstCharInLastIteration: u32,
        pub workingMemory: Vec<u32>,
        pub packedText: Vec<u32>,
        pub textBuffer: Vec<u8>,
        pub packedShift: Vec<u32>,
        pub packedTextOffset: usize,
        pub textBufferOffset: usize,
    }

    /// Original C static function `TextLengthFromBytePacked` from `minibwa/bwtgen.c:97`.
    pub fn TextLengthFromBytePacked(
        bytePackedLength: bgint_t,
        bitPerChar: u32,
        lastByteLength: u32,
    ) -> bgint_t {
        (bytePackedLength - 1) * (BITS_IN_BYTE / bitPerChar) as u64 + lastByteLength as u64
    }

    /// Original C static function `initializeVAL` from `minibwa/bwtgen.c:103`.
    pub fn initializeVAL(startAddr: &mut [u32], length: bgint_t, initValue: u32) {
        for x in startAddr.iter_mut().take(length as usize) {
            *x = initValue;
        }
    }

    /// Original C static function `initializeVAL_bg` from `minibwa/bwtgen.c:109`.
    pub fn initializeVAL_bg(startAddr: &mut [bgint_t], length: bgint_t, initValue: bgint_t) {
        for x in startAddr.iter_mut().take(length as usize) {
            *x = initValue;
        }
    }

    /// Original C static function `GenerateDNAOccCountTable` from `minibwa/bwtgen.c:115`.
    pub fn GenerateDNAOccCountTable(dnaDecodeTable: &mut [u32]) {
        for i in 0..DNA_OCC_CNT_TABLE_SIZE_IN_WORD {
            let mut c = i as u32;
            dnaDecodeTable[i] = 0;
            for _ in 0..8 {
                let t = c & 3;
                dnaDecodeTable[i] = dnaDecodeTable[i].wrapping_add(1 << (t * 8));
                c >>= 2;
            }
        }
    }

    /// Original C static function `BWTOccValueMajorSizeInWord` from `minibwa/bwtgen.c:131`.
    pub fn BWTOccValueMajorSizeInWord(numChar: bgint_t) -> bgint_t {
        let numOfOccValue = (numChar + OCC_INTERVAL - 1) / OCC_INTERVAL + 1;
        let numOfOccIntervalPerMajor = OCC_INTERVAL_MAJOR / OCC_INTERVAL;
        (numOfOccValue + numOfOccIntervalPerMajor - 1) / numOfOccIntervalPerMajor
            * ALPHABET_SIZE as u64
    }

    /// Original C static function `BWTOccValueMinorSizeInWord` from `minibwa/bwtgen.c:140`.
    pub fn BWTOccValueMinorSizeInWord(numChar: bgint_t) -> bgint_t {
        let numOfOccValue = (numChar + OCC_INTERVAL - 1) / OCC_INTERVAL + 1;
        (numOfOccValue + OCC_VALUE_PER_WORD - 1) / OCC_VALUE_PER_WORD * ALPHABET_SIZE as u64
    }

    /// Original C static function `BWTResidentSizeInWord` from `minibwa/bwtgen.c:147`.
    pub fn BWTResidentSizeInWord(numChar: bgint_t) -> bgint_t {
        let numCharRoundUpToOccInterval =
            (numChar + OCC_INTERVAL - 1) / OCC_INTERVAL * OCC_INTERVAL;
        (numCharRoundUpToOccInterval + CHAR_PER_WORD - 1) / CHAR_PER_WORD
    }

    /// Original C static function `BWTIncSetBuildSizeAndTextAddr` from `minibwa/bwtgen.c:158`.
    pub fn BWTIncSetBuildSizeAndTextAddr(bwtInc: &mut BWTInc) {
        let word_scale = (std::mem::size_of::<bgint_t>() / 4) as u64;
        let maxBuildSize;
        if bwtInc.bwt.textLength == 0 {
            maxBuildSize = (bwtInc.availableWord - (2 + OCC_INTERVAL / CHAR_PER_WORD) * word_scale)
                / (2 * CHAR_PER_WORD + 1)
                * CHAR_PER_WORD
                / word_scale;
            if bwtInc.initialMaxBuildSize > 0 {
                bwtInc.buildSize = bwtInc.initialMaxBuildSize.min(maxBuildSize);
            } else {
                bwtInc.buildSize = maxBuildSize;
            }
        } else {
            maxBuildSize = (bwtInc.availableWord
                - bwtInc.bwt.bwtSizeInWord
                - bwtInc.bwt.occSizeInWord
                - (3 + bwtInc.numberOfIterationDone as u64 * OCC_INTERVAL / BIT_PER_CHAR as u64)
                    * word_scale)
                / 3
                / word_scale;
            if maxBuildSize < CHAR_PER_WORD {
                eprintln!("BWTIncSetBuildSizeAndTextAddr(): Not enough space allocated to continue construction!");
                std::process::exit(1);
            }
            if bwtInc.incMaxBuildSize > 0 {
                bwtInc.buildSize = bwtInc.incMaxBuildSize.min(maxBuildSize);
            } else {
                bwtInc.buildSize = maxBuildSize;
            }
            if bwtInc.buildSize < CHAR_PER_WORD {
                bwtInc.buildSize = CHAR_PER_WORD;
            }
        }
        if bwtInc.buildSize < CHAR_PER_WORD {
            eprintln!("BWTIncSetBuildSizeAndTextAddr(): Not enough space allocated to continue construction!");
            std::process::exit(1);
        }
        bwtInc.buildSize = bwtInc.buildSize / CHAR_PER_WORD * CHAR_PER_WORD;
        bwtInc.packedTextOffset = (2 * (bwtInc.buildSize + 1) * word_scale) as usize;
        bwtInc.textBufferOffset = ((bwtInc.buildSize + 1) * word_scale) as usize;
    }

    /// Original C global function `leadingZero` from `minibwa/bwtgen.c:203`.
    pub fn leadingZero(input: u32) -> u32 {
        const LEADING_ZERO_8BIT: [u32; 256] = [
            8, 7, 6, 6, 5, 5, 5, 5, 4, 4, 4, 4, 4, 4, 4, 4, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
            3, 3, 3, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];

        if (input & 0xffff0000) != 0 {
            if (input & 0xff000000) != 0 {
                LEADING_ZERO_8BIT[(input >> 24) as usize]
            } else {
                8 + LEADING_ZERO_8BIT[(input >> 16) as usize]
            }
        } else if (input & 0x0000ff00) != 0 {
            16 + LEADING_ZERO_8BIT[(input >> 8) as usize]
        } else {
            24 + LEADING_ZERO_8BIT[input as usize]
        }
    }

    /// Original C static function `ceilLog2` from `minibwa/bwtgen.c:232`.
    pub fn ceilLog2(input: u32) -> u32 {
        if input <= 1 {
            0
        } else {
            BITS_IN_WORD - leadingZero(input - 1)
        }
    }

    /// Original C static function `BitPerBytePackedChar` from `minibwa/bwtgen.c:239`.
    pub fn BitPerBytePackedChar(alphabetSize: u32) -> u32 {
        let mut bitPerChar = ceilLog2(alphabetSize);
        if BITS_IN_BYTE / (BITS_IN_BYTE / bitPerChar) > bitPerChar {
            bitPerChar = BITS_IN_BYTE / (BITS_IN_BYTE / bitPerChar);
        }
        bitPerChar
    }

    /// Original C static function `BitPerWordPackedChar` from `minibwa/bwtgen.c:249`.
    pub fn BitPerWordPackedChar(alphabetSize: u32) -> u32 {
        ceilLog2(alphabetSize)
    }

    /// Original C static function `ConvertBytePackedToWordPacked` from `minibwa/bwtgen.c:254`.
    pub fn ConvertBytePackedToWordPacked(
        input: &[u8],
        output: &mut [u32],
        alphabetSize: u32,
        textLength: bgint_t,
    ) {
        let bitPerBytePackedChar = BitPerBytePackedChar(alphabetSize);
        let bitPerWordPackedChar = BitPerWordPackedChar(alphabetSize);
        let charPerByte = BITS_IN_BYTE / bitPerBytePackedChar;
        let charPerWord = BITS_IN_WORD / bitPerWordPackedChar;
        let bytePerIteration = charPerWord / charPerByte;
        let mask = ALL_ONE_MASK >> (BITS_IN_WORD - bitPerWordPackedChar)
            << (BITS_IN_WORD - bitPerWordPackedChar);
        let shift = BITS_IN_WORD - BITS_IN_BYTE + bitPerBytePackedChar - bitPerWordPackedChar;
        let mut byteProcessed = 0usize;
        let mut wordProcessed = 0usize;
        let mut buffer = [0u32; BITS_IN_WORD as usize];
        while ((wordProcessed + 1) as u64) * (charPerWord as u64) < textLength {
            let mut k = 0usize;
            for _ in 0..bytePerIteration {
                let mut c = (input[byteProcessed] as u32) << shift;
                for _ in 0..charPerByte {
                    buffer[k] = c & mask;
                    c <<= bitPerBytePackedChar;
                    k += 1;
                }
                byteProcessed += 1;
            }
            let mut c = 0u32;
            for (i, &b) in buffer.iter().enumerate().take(charPerWord as usize) {
                c |= b >> (bitPerWordPackedChar * i as u32);
            }
            output[wordProcessed] = c;
            wordProcessed += 1;
        }
        let mut k = 0usize;
        let remaining = textLength - wordProcessed as u64 * charPerWord as u64;
        for _ in 0..((remaining - 1) / charPerByte as u64 + 1) {
            let mut c = (input[byteProcessed] as u32) << shift;
            for _ in 0..charPerByte {
                buffer[k] = c & mask;
                c <<= bitPerBytePackedChar;
                k += 1;
            }
            byteProcessed += 1;
        }
        let mut c = 0u32;
        for (i, &b) in buffer.iter().enumerate().take(remaining as usize) {
            c |= b >> (bitPerWordPackedChar * i as u32);
        }
        output[wordProcessed] = c;
    }

    /// Original C global function `BWTCreate` from `minibwa/bwtgen.c:319`.
    pub fn BWTCreate(textLength: bgint_t, decodeTable: Option<Vec<u32>>) -> BWT {
        let mut cumulativeFreq = vec![0; ALPHABET_SIZE + 1];
        initializeVAL_bg(&mut cumulativeFreq, (ALPHABET_SIZE + 1) as u64, 0);
        let decodeTable = match decodeTable {
            Some(t) => t,
            None => {
                let mut t = vec![0; DNA_OCC_CNT_TABLE_SIZE_IN_WORD];
                GenerateDNAOccCountTable(&mut t);
                t
            }
        };
        let occMajorSizeInWord = BWTOccValueMajorSizeInWord(textLength);
        BWT {
            textLength: 0,
            inverseSa0: 0,
            cumulativeFreq,
            bwtCode: Vec::new(),
            occValue: Vec::new(),
            occValueMajor: vec![0; occMajorSizeInWord as usize],
            decodeTable,
            bwtSizeInWord: 0,
            occSizeInWord: 0,
            occMajorSizeInWord,
        }
    }

    /// Original C global function `BWTIncCreate` from `minibwa/bwtgen.c:350`.
    pub fn BWTIncCreate(
        textLength: bgint_t,
        mut initialMaxBuildSize: u32,
        mut incMaxBuildSize: u32,
    ) -> BWTInc {
        if textLength < incMaxBuildSize as u64 {
            incMaxBuildSize = textLength as u32;
        }
        if textLength < initialMaxBuildSize as u64 {
            initialMaxBuildSize = textLength as u32;
        }
        let mut packedShift = vec![0; CHAR_PER_WORD as usize];
        for (i, x) in packedShift.iter_mut().enumerate() {
            *x = BITS_IN_WORD - (i as u32 + 1) * BIT_PER_CHAR;
        }
        let n_iter = (textLength - initialMaxBuildSize as u64) / incMaxBuildSize as u64 + 1;
        let word_scale = (std::mem::size_of::<bgint_t>() / 4) as u64;
        let mut availableWord = BWTResidentSizeInWord(textLength)
            + BWTOccValueMinorSizeInWord(textLength)
            + OCC_INTERVAL / BIT_PER_CHAR as u64 * n_iter * 2 * word_scale
            + incMaxBuildSize as u64 / 5 * 3 * word_scale;
        if availableWord < MIN_AVAILABLE_WORD {
            availableWord = MIN_AVAILABLE_WORD;
        }
        eprintln!("[BWTIncCreate] textLength={textLength}, availableWord={availableWord}");
        BWTInc {
            bwt: BWTCreate(textLength, None),
            numberOfIterationDone: 0,
            cumulativeCountInCurrentBuild: vec![0; ALPHABET_SIZE + 1],
            availableWord,
            buildSize: 0,
            initialMaxBuildSize: initialMaxBuildSize as u64,
            incMaxBuildSize: incMaxBuildSize as u64,
            firstCharInLastIteration: 0,
            workingMemory: vec![0; availableWord as usize],
            packedText: Vec::new(),
            textBuffer: Vec::new(),
            packedShift,
            packedTextOffset: 0,
            textBufferOffset: 0,
        }
    }

    /// Original C static function `BWTIncPutPackedTextToRank` from `minibwa/bwtgen.c:382`.
    pub fn BWTIncPutPackedTextToRank(
        packedText: &[u32],
        rank: &mut [bgint_t],
        cumulativeCount: &mut [bgint_t],
        numChar: bgint_t,
    ) {
        let lastWord = (numChar - 1) / CHAR_PER_WORD;
        let numCharInLastWord = numChar - lastWord * CHAR_PER_WORD;
        let packedMask = ALL_ONE_MASK >> (BITS_IN_WORD - BIT_PER_CHAR);
        let mut rankIndex = numChar - 1;

        let mut t = packedText[lastWord as usize]
            >> (BITS_IN_WORD - numCharInLastWord as u32 * BIT_PER_CHAR);
        for _ in 0..numCharInLastWord {
            let c = t & packedMask;
            cumulativeCount[c as usize + 1] += 1;
            rank[rankIndex as usize] = c as bgint_t;
            rankIndex = rankIndex.wrapping_sub(1);
            t >>= BIT_PER_CHAR;
        }

        for i in (0..lastWord as usize).rev() {
            t = packedText[i];
            for _ in 0..CHAR_PER_WORD {
                let c = t & packedMask;
                cumulativeCount[c as usize + 1] += 1;
                rank[rankIndex as usize] = c as bgint_t;
                rankIndex = rankIndex.wrapping_sub(1);
                t >>= BIT_PER_CHAR;
            }
        }

        cumulativeCount[2] += cumulativeCount[1];
        cumulativeCount[3] += cumulativeCount[2];
        cumulativeCount[4] += cumulativeCount[3];
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn count_table_and_size_helpers_match_expected_values() {
            assert_eq!(TextLengthFromBytePacked(3, 2, 2), 10);
            assert_eq!(BWTOccValueMajorSizeInWord(0), 4);
            assert_eq!(BWTOccValueMajorSizeInWord(65_536), 8);
            assert_eq!(BWTOccValueMinorSizeInWord(0), 4);
            assert_eq!(BWTOccValueMinorSizeInWord(256), 4);
            assert_eq!(BWTOccValueMinorSizeInWord(257), 8);
            assert_eq!(BWTResidentSizeInWord(1), 16);
            assert_eq!(BWTResidentSizeInWord(256), 16);
            assert_eq!(BWTResidentSizeInWord(257), 32);

            let mut table = vec![0u32; DNA_OCC_CNT_TABLE_SIZE_IN_WORD];
            GenerateDNAOccCountTable(&mut table);
            assert_eq!(table[0], 8);
            assert_eq!(table[0xffff], 8 << 24);
            assert_eq!(table[0x1b1b], 0x02020202);
        }

        #[test]
        fn bit_width_helpers_follow_original_rounding() {
            assert_eq!(leadingZero(0), 32);
            assert_eq!(leadingZero(1), 31);
            assert_eq!(leadingZero(0x8000_0000), 0);
            assert_eq!(ceilLog2(1), 0);
            assert_eq!(ceilLog2(2), 1);
            assert_eq!(ceilLog2(3), 2);
            assert_eq!(BitPerBytePackedChar(4), 2);
            assert_eq!(BitPerWordPackedChar(4), 2);
        }

        #[test]
        fn byte_packed_to_word_packed_preserves_dna_order() {
            let input = [0x1b, 0x1b, 0x1b, 0x1b, 0x10];
            let mut output = [0u32; 2];
            ConvertBytePackedToWordPacked(&input, &mut output, 4, 18);
            assert_eq!(output[0], 0x1b1b1b1b);
            assert_eq!(output[1], 0x10000000);
        }

        #[test]
        fn bwt_create_and_inc_create_initialize_metadata() {
            let bwt = BWTCreate(1000, None);
            assert_eq!(bwt.textLength, 0);
            assert_eq!(bwt.cumulativeFreq, vec![0; 5]);
            assert_eq!(bwt.occMajorSizeInWord, BWTOccValueMajorSizeInWord(1000));
            assert_eq!(bwt.occValueMajor.len(), bwt.occMajorSizeInWord as usize);
            assert_eq!(bwt.decodeTable[0x1b1b], 0x02020202);

            let mut inc = BWTIncCreate(1000, 320, 160);
            assert_eq!(inc.packedShift[0], 30);
            assert_eq!(inc.packedShift[15], 0);
            assert!(inc.availableWord >= MIN_AVAILABLE_WORD);
            BWTIncSetBuildSizeAndTextAddr(&mut inc);
            assert_eq!(inc.buildSize % CHAR_PER_WORD, 0);
            assert!(inc.packedTextOffset > inc.textBufferOffset);
        }

        #[test]
        fn packed_text_to_rank_decodes_reverse_then_cumulative_counts() {
            let packed = [0x1b1b1b1b, 0x10000000];
            let mut rank = [0u64; 18];
            let mut cumulative = [0u64; 5];
            BWTIncPutPackedTextToRank(&packed, &mut rank, &mut cumulative, 18);
            assert_eq!(rank, [0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1]);
            assert_eq!(cumulative, [0, 5, 10, 14, 18]);
        }

        #[test]
        fn occurrence_counters_match_bruteforce_on_packed_words() {
            let dna = [0x1b1b1b1b, 0x10000000];
            let mut table = vec![0u32; DNA_OCC_CNT_TABLE_SIZE_IN_WORD];
            GenerateDNAOccCountTable(&mut table);

            assert_eq!(ForwardDNAOccCount(&dna, 10, 0, &table), 3);
            assert_eq!(ForwardDNAOccCount(&dna, 10, 1, &table), 3);
            assert_eq!(ForwardDNAOccCount(&dna, 10, 2, &table), 2);
            assert_eq!(ForwardDNAOccCount(&dna, 10, 3, &table), 2);

            let mut occ = [0u64; 4];
            ForwardDNAAllOccCountNoLimit(&dna, 18, &mut occ, &table);
            assert_eq!(occ, [5, 5, 4, 4]);
            assert_eq!(BackwardDNAOccCount(&dna, 4, 0, &table), 4);
            assert_eq!(BackwardDNAOccCount(&dna, 4, 1, &table), 0);
        }

        #[test]
        fn packed_bwt_builder_and_occ_value_follow_original_layout() {
            let relative = [3, 0, 1, 2, 4];
            let cumulative = [0, 1, 2, 3, 4];
            let shifts = (0..16).map(|i| 30 - i * 2).collect::<Vec<_>>();
            let mut bwt_words = [0u32; 1];
            BWTIncBuildPackedBwt(&relative, &mut bwt_words, 4, &cumulative, &shifts);
            assert_eq!(bwt_words[0], 0x81000000);

            let mut bwt = BWTCreate(64, None);
            bwt.inverseSa0 = 100;
            bwt.bwtCode = vec![0x1b1b1b1b, 0x10000000];
            bwt.occValue = vec![0; 4];
            bwt.occValue[2] = (7 << 16) | 3;
            bwt.occValueMajor[2] = 100;
            assert_eq!(BWTOccValueExplicit(&bwt, 0, 2), 107);
            assert_eq!(BWTOccValueExplicit(&bwt, 1, 2), 103);
            assert_eq!(BWTOccValue(&bwt, 18, 0), 5);
            bwt.inverseSa0 = 8;
            assert_eq!(BWTOccValue(&bwt, 18, 0), 5);
        }

        #[test]
        fn incremental_sort_and_rank_builders_follow_key_groups() {
            let mut key = [5, 1, 3, 1, 4, 3, 2, 5, 0, 4];
            let mut seq = [50, 10, 30, 11, 40, 31, 20, 51, 0, 41];
            BWTIncSortKey(&mut key, &mut seq, 10);
            assert_eq!(key, [0, 1, 1, 2, 3, 3, 4, 4, 5, 5]);
            for pair in key.windows(2) {
                assert!(pair[0] <= pair[1]);
            }

            let mut sorted_rank = [1, 2, 2, 5, 8, 9];
            let mut grouped_seq = [0, 1, 2, 3, 4, 5];
            let mut relative = [0u64; 6];
            BWTIncBuildRelativeRank(
                &mut sorted_rank,
                &mut grouped_seq,
                &mut relative,
                5,
                4,
                &[0, 2, 4, 5, 6],
            );
            assert_eq!(relative[5], 5);
            assert_eq!(relative[1], 1);
            assert_eq!(relative[2], 2);
            assert_eq!(grouped_seq[5], (-1i64) as u64);
        }

        #[test]
        fn generated_occ_values_match_bruteforce_counts() {
            let bases: Vec<u32> = (0..512).map(|i| ((i * 7 + i / 3) & 3) as u32).collect();
            let mut bwt_code =
                vec![0u32; (bases.len() + CHAR_PER_WORD as usize - 1) / CHAR_PER_WORD as usize];
            for (i, &base) in bases.iter().enumerate() {
                let word = i / CHAR_PER_WORD as usize;
                let shift = BITS_IN_WORD - ((i % CHAR_PER_WORD as usize) as u32 + 1) * BIT_PER_CHAR;
                bwt_code[word] |= base << shift;
            }
            let mut decode = vec![0u32; DNA_OCC_CNT_TABLE_SIZE_IN_WORD];
            GenerateDNAOccCountTable(&mut decode);
            let mut occ_value = vec![0u32; BWTOccValueMinorSizeInWord(bases.len() as u64) as usize];
            let mut occ_major = vec![0u64; BWTOccValueMajorSizeInWord(bases.len() as u64) as usize];
            BWTGenerateOccValueFromBwt(
                &bwt_code,
                &mut occ_value,
                &mut occ_major,
                bases.len() as u64,
                &decode,
            );

            let bwt = BWT {
                textLength: bases.len() as u64,
                inverseSa0: bases.len() as u64 + 100,
                cumulativeFreq: vec![0; ALPHABET_SIZE + 1],
                bwtCode: bwt_code,
                occValue: occ_value,
                occValueMajor: occ_major,
                decodeTable: decode,
                bwtSizeInWord: BWTResidentSizeInWord(bases.len() as u64),
                occSizeInWord: BWTOccValueMinorSizeInWord(bases.len() as u64),
                occMajorSizeInWord: BWTOccValueMajorSizeInWord(bases.len() as u64),
            };
            for &idx in &[0u64, 1, 17, 255, 256, 257, 511, 512] {
                for c in 0..4u32 {
                    let expected = bases[..idx as usize].iter().filter(|&&x| x == c).count() as u64;
                    assert_eq!(BWTOccValue(&bwt, idx, c), expected, "idx={idx} c={c}");
                }
            }
        }

        #[test]
        fn trailing_bwt_code_is_cleared_after_text_length() {
            let mut bwt = BWTCreate(300, None);
            bwt.textLength = 18;
            bwt.bwtCode = vec![0xffff_ffff; BWTResidentSizeInWord(18) as usize];
            BWTClearTrailingBwtCode(&mut bwt);
            assert_eq!(bwt.bwtCode[0], 0xffff_ffff);
            assert_eq!(bwt.bwtCode[1], 0xf000_0000);
            assert!(bwt.bwtCode[2..].iter().all(|&x| x == 0));
        }

        #[test]
        fn bwt_file_size_and_save_emit_original_binary_layout() {
            assert_eq!(BWTFileSizeInWord(0), 0);
            assert_eq!(BWTFileSizeInWord(1), 1);
            assert_eq!(BWTFileSizeInWord(16), 1);
            assert_eq!(BWTFileSizeInWord(17), 2);

            let mut bwt = BWTCreate(32, None);
            bwt.textLength = 17;
            bwt.inverseSa0 = 9;
            bwt.cumulativeFreq = vec![0, 3, 7, 11, 17];
            bwt.bwtCode = vec![0x0123_4567, 0x89ab_cdef, 0xffff_ffff];
            let path = std::env::temp_dir().join("minibwa-rs-test.bwt");
            BWTSaveBwtCodeAndOcc(&bwt, &path, None).expect("save BWT");
            let bytes = std::fs::read(&path).expect("read BWT");
            let _ = std::fs::remove_file(&path);

            let mut expected = Vec::new();
            expected.extend_from_slice(&9u64.to_le_bytes());
            for value in [3u64, 7, 11, 17] {
                expected.extend_from_slice(&value.to_le_bytes());
            }
            expected.extend_from_slice(&0x0123_4567u32.to_le_bytes());
            expected.extend_from_slice(&0x89ab_cdefu32.to_le_bytes());
            assert_eq!(bytes, expected);
        }

        #[test]
        fn bwt_inc_free_accepts_null_and_owned_state() {
            BWTIncFree(None);
            BWTIncFree(Some(BWTIncCreate(64, 16, 16)));
        }

        #[test]
        fn construct_from_packed_and_mb_bwtgen_handle_small_pac_file() {
            let bases: Vec<u8> = (0..32).map(|i| ((i * 5 + 1) & 3) as u8).collect();
            let mut packed =
                vec![0u8; (bases.len() + CHAR_PER_BYTE as usize - 1) / CHAR_PER_BYTE as usize];
            for (i, &base) in bases.iter().enumerate() {
                packed[i / CHAR_PER_BYTE as usize] |= base
                    << (BITS_IN_BYTE - ((i % CHAR_PER_BYTE as usize) as u32 + 1) * BIT_PER_CHAR);
            }
            let last_len = (bases.len() % CHAR_PER_BYTE as usize) as u8;
            if last_len == 0 {
                packed.push(0);
            }
            packed.push(last_len);

            let pac_path = std::env::temp_dir().join(format!(
                "minibwa-rs-bwtgen-small-{}.pac",
                std::process::id()
            ));
            let bwt_path = std::env::temp_dir().join(format!(
                "minibwa-rs-bwtgen-small-{}.bwt",
                std::process::id()
            ));
            std::fs::write(&pac_path, &packed).expect("write pac");

            let bwt_inc = BWTIncConstructFromPacked(&pac_path, 64, 64).expect("construct BWT");
            assert_eq!(bwt_inc.bwt.textLength, bases.len() as u64);
            let mut counts = [0u64; ALPHABET_SIZE];
            for &base in &bases {
                counts[base as usize] += 1;
            }
            assert_eq!(bwt_inc.bwt.cumulativeFreq[1], counts[0]);
            assert_eq!(bwt_inc.bwt.cumulativeFreq[2], counts[0] + counts[1]);
            assert_eq!(
                bwt_inc.bwt.cumulativeFreq[3],
                counts[0] + counts[1] + counts[2]
            );
            assert_eq!(bwt_inc.bwt.cumulativeFreq[4], bases.len() as u64);
            assert!(bwt_inc.bwt.inverseSa0 <= bases.len() as u64);

            mb_bwtgen(&pac_path, &bwt_path, 64).expect("save generated BWT");
            let saved = std::fs::read(&bwt_path).expect("read generated BWT");
            let _ = std::fs::remove_file(&pac_path);
            let _ = std::fs::remove_file(&bwt_path);
            assert_eq!(
                saved.len(),
                std::mem::size_of::<bgint_t>() * (1 + ALPHABET_SIZE)
                    + BWTFileSizeInWord(bases.len() as u64) as usize * BYTES_IN_WORD
            );
        }
    }

    /// Original C static function `ForwardDNAAllOccCountNoLimit` from `minibwa/bwtgen.c:426`.
    pub fn ForwardDNAAllOccCountNoLimit(
        dna: &[u32],
        index: bgint_t,
        occCount: &mut [bgint_t],
        dnaDecodeTable: &[u32],
    ) {
        const TRUNCATE_RIGHT_MASK: [u32; 16] = [
            0x00000000, 0xC0000000, 0xF0000000, 0xFC000000, 0xFF000000, 0xFFC00000, 0xFFF00000,
            0xFFFC0000, 0xFFFF0000, 0xFFFFC000, 0xFFFFF000, 0xFFFFFC00, 0xFFFFFF00, 0xFFFFFFC0,
            0xFFFFFFF0, 0xFFFFFFFC,
        ];

        occCount[0] = 0;
        occCount[1] = 0;
        occCount[2] = 0;
        occCount[3] = 0;

        let iteration = index / 256;
        let wordToCount = (index - iteration * 256) / 16;
        let charToCount = index - iteration * 256 - wordToCount * 16;
        let mut dna_i = 0usize;

        for _ in 0..iteration {
            let mut sum = 0u32;
            for _ in 0..16 {
                let word = dna[dna_i];
                sum = sum.wrapping_add(dnaDecodeTable[(word >> 16) as usize]);
                sum = sum.wrapping_add(dnaDecodeTable[(word & 0x0000ffff) as usize]);
                dna_i += 1;
            }
            if (sum & 0xfefefeff) != 0 {
                occCount[0] += (sum & 0x000000ff) as bgint_t;
                sum >>= 8;
                occCount[1] += (sum & 0x000000ff) as bgint_t;
                sum >>= 8;
                occCount[2] += (sum & 0x000000ff) as bgint_t;
                sum >>= 8;
                occCount[3] += sum as bgint_t;
            } else if sum == 0x00000100 {
                occCount[0] += 256;
            } else if sum == 0x00010000 {
                occCount[1] += 256;
            } else if sum == 0x01000000 {
                occCount[2] += 256;
            } else if sum == 0x00000000 {
                occCount[3] += 256;
            } else {
                eprintln!("ForwardDNAAllOccCountNoLimit(): DNA occ sum exception!");
                std::process::exit(1);
            }
        }

        let mut sum = 0u32;
        for _ in 0..wordToCount {
            let word = dna[dna_i];
            sum = sum.wrapping_add(dnaDecodeTable[(word >> 16) as usize]);
            sum = sum.wrapping_add(dnaDecodeTable[(word & 0x0000ffff) as usize]);
            dna_i += 1;
        }

        if charToCount > 0 {
            let c = dna[dna_i] & TRUNCATE_RIGHT_MASK[charToCount as usize];
            sum = sum.wrapping_add(dnaDecodeTable[(c >> 16) as usize]);
            sum = sum.wrapping_add(dnaDecodeTable[(c & 0xffff) as usize]);
            sum = sum.wrapping_add(charToCount as u32).wrapping_sub(16);
        }

        occCount[0] += (sum & 0x000000ff) as bgint_t;
        sum >>= 8;
        occCount[1] += (sum & 0x000000ff) as bgint_t;
        sum >>= 8;
        occCount[2] += (sum & 0x000000ff) as bgint_t;
        sum >>= 8;
        occCount[3] += sum as bgint_t;
    }

    /// Original C static function `BWTIncBuildPackedBwt` from `minibwa/bwtgen.c:499`.
    pub fn BWTIncBuildPackedBwt(
        relativeRank: &[bgint_t],
        bwt: &mut [u32],
        numChar: bgint_t,
        cumulativeCount: &[bgint_t],
        packedShift: &[u32],
    ) {
        let inverseSa0 = relativeRank[0];
        let mut previousRank = relativeRank[0];
        for i in 1..=numChar as usize {
            let currentRank = relativeRank[i];
            let c = ((previousRank > cumulativeCount[1]) as u32)
                + ((previousRank > cumulativeCount[2]) as u32)
                + ((previousRank > cumulativeCount[3]) as u32);
            if c > 0 {
                let mut r = currentRank;
                if r > inverseSa0 {
                    r -= 1;
                }
                let wordIndex = r / CHAR_PER_WORD;
                let charIndex = r - wordIndex * CHAR_PER_WORD;
                bwt[wordIndex as usize] |= c << packedShift[charIndex as usize];
            }
            previousRank = currentRank;
        }
    }

    /// Original C static function `BWTOccValueExplicit` from `minibwa/bwtgen.c:531`.
    pub fn BWTOccValueExplicit(bwt: &BWT, occIndexExplicit: bgint_t, character: u32) -> bgint_t {
        let occIndexMajor = occIndexExplicit * OCC_INTERVAL / OCC_INTERVAL_MAJOR;
        if occIndexExplicit % OCC_VALUE_PER_WORD == 0 {
            bwt.occValueMajor[(occIndexMajor * ALPHABET_SIZE as u64 + character as u64) as usize]
                + (bwt.occValue[(occIndexExplicit / OCC_VALUE_PER_WORD * ALPHABET_SIZE as u64
                    + character as u64) as usize]
                    >> 16) as u64
        } else {
            bwt.occValueMajor[(occIndexMajor * ALPHABET_SIZE as u64 + character as u64) as usize]
                + (bwt.occValue[(occIndexExplicit / OCC_VALUE_PER_WORD * ALPHABET_SIZE as u64
                    + character as u64) as usize]
                    & 0xffff) as u64
        }
    }

    /// Original C static function `ForwardDNAOccCount` from `minibwa/bwtgen.c:549`.
    pub fn ForwardDNAOccCount(
        dna: &[u32],
        index: u32,
        character: u32,
        dnaDecodeTable: &[u32],
    ) -> u32 {
        const TRUNCATE_RIGHT_MASK: [u32; 16] = [
            0x00000000, 0xC0000000, 0xF0000000, 0xFC000000, 0xFF000000, 0xFFC00000, 0xFFF00000,
            0xFFFC0000, 0xFFFF0000, 0xFFFFC000, 0xFFFFF000, 0xFFFFFC00, 0xFFFFFF00, 0xFFFFFFC0,
            0xFFFFFFF0, 0xFFFFFFFC,
        ];

        let wordToCount = index / 16;
        let charToCount = index - wordToCount * 16;
        let mut sum = 0u32;

        for i in 0..wordToCount as usize {
            sum = sum.wrapping_add(dnaDecodeTable[(dna[i] >> 16) as usize]);
            sum = sum.wrapping_add(dnaDecodeTable[(dna[i] & 0x0000ffff) as usize]);
        }

        if charToCount > 0 {
            let c = dna[wordToCount as usize] & TRUNCATE_RIGHT_MASK[charToCount as usize];
            sum = sum.wrapping_add(dnaDecodeTable[(c >> 16) as usize]);
            sum = sum.wrapping_add(dnaDecodeTable[(c & 0xffff) as usize]);
            sum = sum.wrapping_add(charToCount).wrapping_sub(16);
        }

        (sum >> (character * 8)) & 0x000000ff
    }

    /// Original C static function `BackwardDNAOccCount` from `minibwa/bwtgen.c:580`.
    pub fn BackwardDNAOccCount(
        dna: &[u32],
        index: u32,
        character: u32,
        dnaDecodeTable: &[u32],
    ) -> u32 {
        const TRUNCATE_LEFT_MASK: [u32; 16] = [
            0x00000000, 0x00000003, 0x0000000f, 0x0000003f, 0x000000ff, 0x000003ff, 0x00000fff,
            0x00003fff, 0x0000ffff, 0x0003ffff, 0x000fffff, 0x003fffff, 0x00ffffff, 0x03ffffff,
            0x0fffffff, 0x3fffffff,
        ];

        let wordToCount = index / 16;
        let charToCount = index - wordToCount * 16;
        let mut sum = 0u32;
        let mut pos = dna.len() - wordToCount as usize - 1;

        if charToCount > 0 {
            let c = dna[pos] & TRUNCATE_LEFT_MASK[charToCount as usize];
            sum = sum.wrapping_add(dnaDecodeTable[(c >> 16) as usize]);
            sum = sum.wrapping_add(dnaDecodeTable[(c & 0xffff) as usize]);
            sum = sum.wrapping_add(charToCount).wrapping_sub(16);
        }

        for _ in 0..wordToCount {
            pos += 1;
            sum = sum.wrapping_add(dnaDecodeTable[(dna[pos] >> 16) as usize]);
            sum = sum.wrapping_add(dnaDecodeTable[(dna[pos] & 0x0000ffff) as usize]);
        }

        (sum >> (character * 8)) & 0x000000ff
    }

    /// Original C global function `BWTOccValue` from `minibwa/bwtgen.c:614`.
    pub fn BWTOccValue(bwt: &BWT, mut index: bgint_t, character: u32) -> bgint_t {
        if index > bwt.inverseSa0 {
            index -= 1;
        }

        let occExplicitIndex = (index + OCC_INTERVAL / 2 - 1) / OCC_INTERVAL;
        let occIndex = occExplicitIndex * OCC_INTERVAL;
        let occValue = BWTOccValueExplicit(bwt, occExplicitIndex, character);

        if occIndex == index {
            return occValue;
        }

        let wordIndex = (occIndex / CHAR_PER_WORD) as usize;
        if occIndex < index {
            occValue
                + ForwardDNAOccCount(
                    &bwt.bwtCode[wordIndex..],
                    (index - occIndex) as u32,
                    character,
                    &bwt.decodeTable,
                ) as u64
        } else {
            occValue
                - BackwardDNAOccCount(
                    &bwt.bwtCode[..wordIndex],
                    (occIndex - index) as u32,
                    character,
                    &bwt.decodeTable,
                ) as u64
        }
    }

    /// Original C static function `BWTIncGetAbsoluteRank` from `minibwa/bwtgen.c:638`.
    pub fn BWTIncGetAbsoluteRank(
        bwt: &BWT,
        absoluteRank: &mut [bgint_t],
        seq: &mut [bgint_t],
        packedText: &[u32],
        numChar: bgint_t,
        cumulativeCount: &[bgint_t],
        firstCharInLastIteration: u32,
    ) -> bgint_t {
        let mut seqIndexFromStart = [0u64; ALPHABET_SIZE];
        let mut seqIndexFromEnd = [0u64; ALPHABET_SIZE];
        for i in 0..ALPHABET_SIZE {
            seqIndexFromStart[i] = cumulativeCount[i];
            seqIndexFromEnd[i] = cumulativeCount[i + 1] - 1;
        }

        let shift = BITS_IN_WORD - BIT_PER_CHAR;
        let packedMask = ALL_ONE_MASK >> shift;
        let mut saIndex = bwt.inverseSa0;
        let mut rankIndex = numChar - 1;
        let lastWord = numChar / CHAR_PER_WORD;

        for i in (0..lastWord as usize).rev() {
            let mut t = packedText[i];
            for _ in 0..CHAR_PER_WORD {
                let c = t & packedMask;
                saIndex = bwt.cumulativeFreq[c as usize] + BWTOccValue(bwt, saIndex, c) + 1;
                if saIndex > bwt.inverseSa0 {
                    let idx = seqIndexFromEnd[c as usize] as usize;
                    seq[idx] = rankIndex;
                    absoluteRank[idx] = saIndex;
                    seqIndexFromEnd[c as usize] -= 1;
                } else {
                    let idx = seqIndexFromStart[c as usize] as usize;
                    seq[idx] = rankIndex;
                    absoluteRank[idx] = saIndex;
                    seqIndexFromStart[c as usize] += 1;
                }
                rankIndex = rankIndex.wrapping_sub(1);
                t >>= BIT_PER_CHAR;
            }
        }

        let idx = seqIndexFromStart[firstCharInLastIteration as usize] as usize;
        absoluteRank[idx] = bwt.inverseSa0;
        seq[idx] = numChar;
        seqIndexFromStart[firstCharInLastIteration as usize]
    }

    /// Original C static function `BWTIncSortKey` from `minibwa/bwtgen.c:690`.
    pub fn BWTIncSortKey(key: &mut [bgint_t], seq: &mut [bgint_t], numItem: bgint_t) {
        const EQUAL_KEY_THRESHOLD: i64 = 4;
        if numItem < 2 {
            return;
        }

        let mut lowIndex = 0i64;
        let mut highIndex = numItem as i64 - 1;
        let mut lowStack = [0i64; 32];
        let mut highStack = [0i64; 32];
        let mut stackDepth = 0usize;

        loop {
            loop {
                if highIndex - lowIndex < BWTINC_INSERT_SORT_NUM_ITEM {
                    for i in lowIndex + 1..=highIndex {
                        let tempSeq = seq[i as usize];
                        let tempKey = key[i as usize];
                        let mut j = i;
                        while j > lowIndex && key[(j - 1) as usize] > tempKey {
                            seq[j as usize] = seq[(j - 1) as usize];
                            key[j as usize] = key[(j - 1) as usize];
                            j -= 1;
                        }
                        if j != i {
                            seq[j as usize] = tempSeq;
                            key[j as usize] = tempKey;
                        }
                    }
                    break;
                }

                let mut midIndex = (lowIndex & highIndex) + ((lowIndex ^ highIndex) / 2);
                if key[lowIndex as usize] > key[midIndex as usize] {
                    seq.swap(lowIndex as usize, midIndex as usize);
                    key.swap(lowIndex as usize, midIndex as usize);
                }
                if key[lowIndex as usize] > key[highIndex as usize] {
                    seq.swap(lowIndex as usize, highIndex as usize);
                    key.swap(lowIndex as usize, highIndex as usize);
                }
                if key[midIndex as usize] > key[highIndex as usize] {
                    seq.swap(midIndex as usize, highIndex as usize);
                    key.swap(midIndex as usize, highIndex as usize);
                }

                let mut numberOfEqualKey = 0i64;
                let mut lowPartitionIndex = lowIndex + 1;
                let mut highPartitionIndex = highIndex - 1;

                loop {
                    while lowPartitionIndex <= highPartitionIndex
                        && key[lowPartitionIndex as usize] <= key[midIndex as usize]
                    {
                        numberOfEqualKey +=
                            (key[lowPartitionIndex as usize] == key[midIndex as usize]) as i64;
                        lowPartitionIndex += 1;
                    }
                    while lowPartitionIndex < highPartitionIndex {
                        if key[midIndex as usize] >= key[highPartitionIndex as usize] {
                            numberOfEqualKey +=
                                (key[midIndex as usize] == key[highPartitionIndex as usize]) as i64;
                            break;
                        }
                        highPartitionIndex -= 1;
                    }
                    if lowPartitionIndex >= highPartitionIndex {
                        break;
                    }
                    seq.swap(lowPartitionIndex as usize, highPartitionIndex as usize);
                    key.swap(lowPartitionIndex as usize, highPartitionIndex as usize);
                    if highPartitionIndex == midIndex {
                        midIndex = lowPartitionIndex;
                    }
                    lowPartitionIndex += 1;
                    highPartitionIndex -= 1;
                }

                highPartitionIndex = lowPartitionIndex;
                lowPartitionIndex -= 1;

                seq.swap(midIndex as usize, lowPartitionIndex as usize);
                key.swap(midIndex as usize, lowPartitionIndex as usize);

                if highIndex - lowIndex + BWTINC_INSERT_SORT_NUM_ITEM
                    <= EQUAL_KEY_THRESHOLD * numberOfEqualKey
                {
                    midIndex = lowIndex;
                    loop {
                        while midIndex < lowPartitionIndex
                            && key[midIndex as usize] < key[lowPartitionIndex as usize]
                        {
                            midIndex += 1;
                        }
                        while midIndex < lowPartitionIndex
                            && key[lowPartitionIndex as usize]
                                == key[(lowPartitionIndex - 1) as usize]
                        {
                            lowPartitionIndex -= 1;
                        }
                        if midIndex >= lowPartitionIndex {
                            break;
                        }
                        seq.swap(midIndex as usize, (lowPartitionIndex - 1) as usize);
                        key.swap(midIndex as usize, (lowPartitionIndex - 1) as usize);
                        midIndex += 1;
                        lowPartitionIndex -= 1;
                    }
                }

                if lowPartitionIndex - lowIndex > highIndex - highPartitionIndex {
                    lowStack[stackDepth] = lowIndex;
                    highStack[stackDepth] = lowPartitionIndex - 1;
                    stackDepth += 1;
                    lowIndex = highPartitionIndex;
                } else {
                    lowStack[stackDepth] = highPartitionIndex;
                    highStack[stackDepth] = highIndex;
                    stackDepth += 1;
                    if lowPartitionIndex > lowIndex {
                        highIndex = lowPartitionIndex - 1;
                    } else {
                        break;
                    }
                }
            }

            if stackDepth > 0 {
                stackDepth -= 1;
                lowIndex = lowStack[stackDepth];
                highIndex = highStack[stackDepth];
            } else {
                return;
            }
        }
    }

    /// Original C static function `BWTIncBuildRelativeRank` from `minibwa/bwtgen.c:868`.
    pub fn BWTIncBuildRelativeRank(
        sortedRank: &mut [bgint_t],
        seq: &mut [bgint_t],
        relativeRank: &mut [bgint_t],
        numItem: bgint_t,
        mut oldInverseSa0: bgint_t,
        cumulativeCount: &[bgint_t],
    ) {
        let mut lastIndex = numItem;
        let mut lastRank = sortedRank[numItem as usize];
        if lastRank > oldInverseSa0 {
            sortedRank[numItem as usize] -= 1;
        }
        let mut s = seq[numItem as usize];
        relativeRank[s as usize] = numItem;
        if lastRank == oldInverseSa0 {
            oldInverseSa0 += 1;
            lastRank += 1;
        }

        let mut c = ALPHABET_SIZE as u64 - 1;
        let mut freq = cumulativeCount[c as usize];
        for i in (0..numItem).rev() {
            let r = sortedRank[i as usize];
            if r > oldInverseSa0 {
                sortedRank[i as usize] -= 1;
            }
            s = seq[i as usize];
            if i < freq {
                if lastIndex >= freq {
                    lastRank += 1;
                }
                c -= 1;
                freq = cumulativeCount[c as usize];
            }
            if r == lastRank {
                relativeRank[s as usize] = lastIndex;
            } else {
                if i == lastIndex - 1 {
                    if lastIndex < numItem && (seq[(lastIndex + 1) as usize] as sbgint_t) < 0 {
                        seq[lastIndex as usize] = seq[(lastIndex + 1) as usize].wrapping_sub(1);
                    } else {
                        seq[lastIndex as usize] = (-1i64) as bgint_t;
                    }
                }
                lastIndex = i;
                lastRank = r;
                relativeRank[s as usize] = i;
                if r == oldInverseSa0 {
                    oldInverseSa0 += 1;
                    lastRank += 1;
                }
            }
        }
    }

    /// Original C static function `BWTIncBuildBwt` from `minibwa/bwtgen.c:925`.
    pub fn BWTIncBuildBwt(
        insertBwt: &mut [u32],
        relativeRank: &[bgint_t],
        numChar: bgint_t,
        cumulativeCount: &[bgint_t],
    ) {
        let mut previousRank = relativeRank[0];
        for i in 1..=numChar as usize {
            let currentRank = relativeRank[i];
            let c = ((previousRank >= cumulativeCount[1]) as u32)
                + ((previousRank >= cumulativeCount[2]) as u32)
                + ((previousRank >= cumulativeCount[3]) as u32);
            insertBwt[currentRank as usize] = c;
            previousRank = currentRank;
        }
    }

    /// Original C static function `BWTIncMergeBwt` from `minibwa/bwtgen.c:943`.
    #[allow(unused_assignments)]
    pub fn BWTIncMergeBwt(
        sortedRank: &[bgint_t],
        oldBwt: &[u32],
        insertBwt: &[u32],
        mergedBwt: &mut [u32],
        numOldBwt: bgint_t,
        numInsertBwt: bgint_t,
    ) {
        let mut oIndex = 0u64;
        let mut iIndex = 0u64;
        let mut mIndex = 0u64;
        let mut mWord = 0u64;
        let mut mChar = 0u64;
        mergedBwt[0] = 0;

        while oIndex < numOldBwt {
            while iIndex <= numInsertBwt && sortedRank[iIndex as usize] <= oIndex {
                if sortedRank[iIndex as usize] != 0 {
                    mergedBwt[mWord as usize] |= insertBwt[iIndex as usize]
                        << (BITS_IN_WORD - (mChar as u32 + 1) * BIT_PER_CHAR);
                    mIndex += 1;
                    mChar += 1;
                    if mChar == CHAR_PER_WORD {
                        mChar = 0;
                        mWord += 1;
                        mergedBwt[mWord as usize] = 0;
                    }
                }
                iIndex += 1;
            }

            let o = if iIndex <= numInsertBwt {
                sortedRank[iIndex as usize]
            } else {
                numOldBwt
            };
            let numInsert = o - oIndex;
            let mut oWord = oIndex / CHAR_PER_WORD;
            let oChar = oIndex - oWord * CHAR_PER_WORD;

            if oChar > mChar {
                let leftShift = ((oChar - mChar) as u32) * BIT_PER_CHAR;
                let rightShift = ((CHAR_PER_WORD + mChar - oChar) as u32) * BIT_PER_CHAR;
                mergedBwt[mWord as usize] |= (oldBwt[oWord as usize]
                    << (oChar as u32 * BIT_PER_CHAR)
                    >> (mChar as u32 * BIT_PER_CHAR))
                    | (oldBwt[oWord as usize + 1] >> rightShift);
                oIndex += numInsert.min(CHAR_PER_WORD - mChar);
                while o > oIndex {
                    oWord += 1;
                    mWord += 1;
                    mergedBwt[mWord as usize] = (oldBwt[oWord as usize] << leftShift)
                        | (oldBwt[oWord as usize + 1] >> rightShift);
                    oIndex += CHAR_PER_WORD;
                }
            } else if oChar < mChar {
                let rightShift = ((mChar - oChar) as u32) * BIT_PER_CHAR;
                let leftShift = ((CHAR_PER_WORD + oChar - mChar) as u32) * BIT_PER_CHAR;
                mergedBwt[mWord as usize] |= oldBwt[oWord as usize]
                    << (oChar as u32 * BIT_PER_CHAR)
                    >> (mChar as u32 * BIT_PER_CHAR);
                oIndex += numInsert.min(CHAR_PER_WORD - mChar);
                while o > oIndex {
                    oWord += 1;
                    mWord += 1;
                    mergedBwt[mWord as usize] = (oldBwt[oWord as usize - 1] << leftShift)
                        | (oldBwt[oWord as usize] >> rightShift);
                    oIndex += CHAR_PER_WORD;
                }
            } else {
                mergedBwt[mWord as usize] |= oldBwt[oWord as usize]
                    << (mChar as u32 * BIT_PER_CHAR)
                    >> (mChar as u32 * BIT_PER_CHAR);
                oIndex += numInsert.min(CHAR_PER_WORD - mChar);
                while o > oIndex {
                    oWord += 1;
                    mWord += 1;
                    mergedBwt[mWord as usize] = oldBwt[oWord as usize];
                    oIndex += CHAR_PER_WORD;
                }
            }

            oIndex = o;
            mIndex += numInsert;
            mWord = mIndex / CHAR_PER_WORD;
            mChar = mIndex - mWord * CHAR_PER_WORD;
            if mChar == 0 {
                mergedBwt[mWord as usize] = 0;
            } else {
                let offset = BITS_IN_WORD - mChar as u32 * BIT_PER_CHAR;
                mergedBwt[mWord as usize] = mergedBwt[mWord as usize] >> offset << offset;
            }
        }

        while iIndex <= numInsertBwt {
            if sortedRank[iIndex as usize] != 0 {
                mergedBwt[mWord as usize] |= insertBwt[iIndex as usize]
                    << (BITS_IN_WORD - (mChar as u32 + 1) * BIT_PER_CHAR);
                mIndex += 1;
                mChar += 1;
                if mChar == CHAR_PER_WORD {
                    mChar = 0;
                    mWord += 1;
                    mergedBwt[mWord as usize] = 0;
                }
            }
            iIndex += 1;
        }
    }

    /// Original C global function `BWTClearTrailingBwtCode` from `minibwa/bwtgen.c:1053`.
    pub fn BWTClearTrailingBwtCode(bwt: &mut BWT) {
        let bwtResidentSizeInWord = BWTResidentSizeInWord(bwt.textLength);
        let wordIndex = bwt.textLength / CHAR_PER_WORD;
        let offset = (bwt.textLength - wordIndex * CHAR_PER_WORD) as u32 * BIT_PER_CHAR;
        if offset > 0 {
            let rshift = BITS_IN_WORD - offset;
            bwt.bwtCode[wordIndex as usize] = bwt.bwtCode[wordIndex as usize] >> rshift << rshift;
        } else if wordIndex < bwtResidentSizeInWord {
            bwt.bwtCode[wordIndex as usize] = 0;
        }
        for i in wordIndex + 1..bwtResidentSizeInWord {
            bwt.bwtCode[i as usize] = 0;
        }
    }

    /// Original C global function `BWTGenerateOccValueFromBwt` from `minibwa/bwtgen.c:1077`.
    pub fn BWTGenerateOccValueFromBwt(
        bwt: &[u32],
        occValue: &mut [u32],
        occValueMajor: &mut [bgint_t],
        textLength: bgint_t,
        decodeTable: &[u32],
    ) {
        let wordBetweenOccValue = OCC_INTERVAL / CHAR_PER_WORD;
        let numberOfOccValue = (textLength + OCC_INTERVAL - 1) / OCC_INTERVAL + 1;
        let numberOfOccIntervalPerMajor = OCC_INTERVAL_MAJOR / OCC_INTERVAL;
        let numberOfOccValueMajor =
            (numberOfOccValue + numberOfOccIntervalPerMajor - 1) / numberOfOccIntervalPerMajor;

        let mut tempOccValue0 = [0u64; ALPHABET_SIZE];
        let mut tempOccValue1 = [0u64; ALPHABET_SIZE];
        occValueMajor[0] = 0;
        occValueMajor[1] = 0;
        occValueMajor[2] = 0;
        occValueMajor[3] = 0;

        let mut occIndex = 0u64;
        let mut bwtIndex = 0usize;
        for occMajorIndex in 1..numberOfOccValueMajor {
            for _ in 0..numberOfOccIntervalPerMajor / 2 {
                let mut sum = 0u64;
                tempOccValue1.copy_from_slice(&tempOccValue0);
                for _ in 0..wordBetweenOccValue {
                    let c = bwt[bwtIndex];
                    sum += decodeTable[(c >> 16) as usize] as u64;
                    sum += decodeTable[(c & 0x0000ffff) as usize] as u64;
                    bwtIndex += 1;
                }
                if (sum & 0xfefefeff) != 0 {
                    tempOccValue1[0] += sum & 0x000000ff;
                    sum >>= 8;
                    tempOccValue1[1] += sum & 0x000000ff;
                    sum >>= 8;
                    tempOccValue1[2] += sum & 0x000000ff;
                    sum >>= 8;
                    tempOccValue1[3] += sum;
                } else if sum == 0x00000100 {
                    tempOccValue1[0] += 256;
                } else if sum == 0x00010000 {
                    tempOccValue1[1] += 256;
                } else if sum == 0x01000000 {
                    tempOccValue1[2] += 256;
                } else {
                    tempOccValue1[3] += 256;
                }
                for c in 0..ALPHABET_SIZE {
                    occValue[occIndex as usize * 4 + c] =
                        ((tempOccValue0[c] << 16) | tempOccValue1[c]) as u32;
                    tempOccValue0[c] = tempOccValue1[c];
                }
                occIndex += 1;

                sum = 0;
                for _ in 0..wordBetweenOccValue {
                    let c = bwt[bwtIndex];
                    sum += decodeTable[(c >> 16) as usize] as u64;
                    sum += decodeTable[(c & 0x0000ffff) as usize] as u64;
                    bwtIndex += 1;
                }
                if (sum & 0xfefefeff) != 0 {
                    tempOccValue0[0] += sum & 0x000000ff;
                    sum >>= 8;
                    tempOccValue0[1] += sum & 0x000000ff;
                    sum >>= 8;
                    tempOccValue0[2] += sum & 0x000000ff;
                    sum >>= 8;
                    tempOccValue0[3] += sum;
                } else if sum == 0x00000100 {
                    tempOccValue0[0] += 256;
                } else if sum == 0x00010000 {
                    tempOccValue0[1] += 256;
                } else if sum == 0x01000000 {
                    tempOccValue0[2] += 256;
                } else {
                    tempOccValue0[3] += 256;
                }
            }

            for c in 0..ALPHABET_SIZE {
                occValueMajor[occMajorIndex as usize * 4 + c] =
                    occValueMajor[(occMajorIndex - 1) as usize * 4 + c] + tempOccValue0[c];
                tempOccValue0[c] = 0;
            }
        }

        while occIndex < (numberOfOccValue - 1) / 2 {
            let mut sum = 0u64;
            tempOccValue1.copy_from_slice(&tempOccValue0);
            for _ in 0..wordBetweenOccValue {
                let c = bwt[bwtIndex];
                sum += decodeTable[(c >> 16) as usize] as u64;
                sum += decodeTable[(c & 0x0000ffff) as usize] as u64;
                bwtIndex += 1;
            }
            if (sum & 0xfefefeff) != 0 {
                tempOccValue1[0] += sum & 0x000000ff;
                sum >>= 8;
                tempOccValue1[1] += sum & 0x000000ff;
                sum >>= 8;
                tempOccValue1[2] += sum & 0x000000ff;
                sum >>= 8;
                tempOccValue1[3] += sum;
            } else if sum == 0x00000100 {
                tempOccValue1[0] += 256;
            } else if sum == 0x00010000 {
                tempOccValue1[1] += 256;
            } else if sum == 0x01000000 {
                tempOccValue1[2] += 256;
            } else {
                tempOccValue1[3] += 256;
            }
            for c in 0..ALPHABET_SIZE {
                occValue[occIndex as usize * 4 + c] =
                    ((tempOccValue0[c] << 16) | tempOccValue1[c]) as u32;
                tempOccValue0[c] = tempOccValue1[c];
            }
            occIndex += 1;

            sum = 0;
            for _ in 0..wordBetweenOccValue {
                let c = bwt[bwtIndex];
                sum += decodeTable[(c >> 16) as usize] as u64;
                sum += decodeTable[(c & 0x0000ffff) as usize] as u64;
                bwtIndex += 1;
            }
            if (sum & 0xfefefeff) != 0 {
                tempOccValue0[0] += sum & 0x000000ff;
                sum >>= 8;
                tempOccValue0[1] += sum & 0x000000ff;
                sum >>= 8;
                tempOccValue0[2] += sum & 0x000000ff;
                sum >>= 8;
                tempOccValue0[3] += sum;
            } else if sum == 0x00000100 {
                tempOccValue0[0] += 256;
            } else if sum == 0x00010000 {
                tempOccValue0[1] += 256;
            } else if sum == 0x01000000 {
                tempOccValue0[2] += 256;
            } else {
                tempOccValue0[3] += 256;
            }
        }

        let mut sum = 0u64;
        tempOccValue1.copy_from_slice(&tempOccValue0);
        if occIndex * 2 < numberOfOccValue - 1 {
            for _ in 0..wordBetweenOccValue {
                let c = bwt[bwtIndex];
                sum += decodeTable[(c >> 16) as usize] as u64;
                sum += decodeTable[(c & 0x0000ffff) as usize] as u64;
                bwtIndex += 1;
            }
            if (sum & 0xfefefeff) != 0 {
                tempOccValue1[0] += sum & 0x000000ff;
                sum >>= 8;
                tempOccValue1[1] += sum & 0x000000ff;
                sum >>= 8;
                tempOccValue1[2] += sum & 0x000000ff;
                sum >>= 8;
                tempOccValue1[3] += sum;
            } else if sum == 0x00000100 {
                tempOccValue1[0] += 256;
            } else if sum == 0x00010000 {
                tempOccValue1[1] += 256;
            } else if sum == 0x01000000 {
                tempOccValue1[2] += 256;
            } else {
                tempOccValue1[3] += 256;
            }
        }

        for c in 0..ALPHABET_SIZE {
            occValue[occIndex as usize * 4 + c] =
                ((tempOccValue0[c] << 16) | tempOccValue1[c]) as u32;
        }
    }

    /// Original C static function `BWTIncConstruct` from `minibwa/bwtgen.c:1289`.
    pub fn BWTIncConstruct(bwtInc: &mut BWTInc, numChar: bgint_t) {
        let mergedBwtSizeInWord = BWTResidentSizeInWord(bwtInc.bwt.textLength + numChar);
        let mergedOccSizeInWord = BWTOccValueMinorSizeInWord(bwtInc.bwt.textLength + numChar);
        initializeVAL_bg(
            &mut bwtInc.cumulativeCountInCurrentBuild,
            (ALPHABET_SIZE + 1) as u64,
            0,
        );

        let firstCharInThisIteration;
        let newInverseSa0;
        let mut mergedBwt;

        if bwtInc.bwt.textLength == 0 {
            let mut seq = vec![0u64; bwtInc.buildSize as usize + 1];
            let mut relativeRank = vec![0u64; bwtInc.buildSize as usize + 1];
            mergedBwt = vec![0u32; mergedBwtSizeInWord as usize + 1];

            BWTIncPutPackedTextToRank(
                &bwtInc.packedText,
                &mut relativeRank,
                &mut bwtInc.cumulativeCountInCurrentBuild,
                numChar,
            );

            firstCharInThisIteration = relativeRank[0] as u32;
            relativeRank[numChar as usize] = 0;

            let mut qs_v = relativeRank
                .iter()
                .take(numChar as usize + 1)
                .map(|&x| x as i64)
                .collect::<Vec<_>>();
            let mut qs_i = vec![0i64; numChar as usize + 1];
            QSufSortSuffixSort(
                &mut qs_v,
                &mut qs_i,
                numChar as i64,
                ALPHABET_SIZE as i64 - 1,
                0,
                0,
            );
            for i in 0..=numChar as usize {
                relativeRank[i] = qs_v[i] as u64;
                seq[i] = qs_i[i] as u64;
            }
            newInverseSa0 = relativeRank[0];

            initializeVAL(&mut mergedBwt, mergedBwtSizeInWord, 0);
            BWTIncBuildPackedBwt(
                &relativeRank,
                &mut mergedBwt,
                numChar,
                &bwtInc.cumulativeCountInCurrentBuild,
                &bwtInc.packedShift,
            );

            bwtInc.firstCharInLastIteration = ALPHABET_SIZE as u32;
        } else {
            let mut sortedRank = vec![0u64; bwtInc.buildSize as usize + 1];
            let mut seq = vec![0u64; bwtInc.buildSize as usize + 1];
            let mut insertBwt = vec![0u32; numChar as usize + 1];
            let mut relativeRank = vec![0u64; bwtInc.buildSize as usize + 1];

            firstCharInThisIteration = bwtInc.packedText[0] >> (BITS_IN_WORD - BIT_PER_CHAR);

            ForwardDNAAllOccCountNoLimit(
                &bwtInc.packedText,
                numChar,
                &mut bwtInc.cumulativeCountInCurrentBuild[1..],
                &bwtInc.bwt.decodeTable,
            );
            bwtInc.cumulativeCountInCurrentBuild[bwtInc.firstCharInLastIteration as usize + 1] += 1;
            bwtInc.cumulativeCountInCurrentBuild[2] += bwtInc.cumulativeCountInCurrentBuild[1];
            bwtInc.cumulativeCountInCurrentBuild[3] += bwtInc.cumulativeCountInCurrentBuild[2];
            bwtInc.cumulativeCountInCurrentBuild[4] += bwtInc.cumulativeCountInCurrentBuild[3];

            let oldInverseSa0RelativeRank = BWTIncGetAbsoluteRank(
                &bwtInc.bwt,
                &mut sortedRank,
                &mut seq,
                &bwtInc.packedText,
                numChar,
                &bwtInc.cumulativeCountInCurrentBuild,
                bwtInc.firstCharInLastIteration,
            );

            for i in 0..ALPHABET_SIZE {
                let lo = bwtInc.cumulativeCountInCurrentBuild[i];
                let hi = bwtInc.cumulativeCountInCurrentBuild[i + 1];
                if lo > oldInverseSa0RelativeRank || hi <= oldInverseSa0RelativeRank {
                    BWTIncSortKey(
                        &mut sortedRank[lo as usize..],
                        &mut seq[lo as usize..],
                        hi - lo,
                    );
                } else {
                    if lo < oldInverseSa0RelativeRank {
                        BWTIncSortKey(
                            &mut sortedRank[lo as usize..],
                            &mut seq[lo as usize..],
                            oldInverseSa0RelativeRank - lo,
                        );
                    }
                    if hi > oldInverseSa0RelativeRank + 1 {
                        BWTIncSortKey(
                            &mut sortedRank[(oldInverseSa0RelativeRank + 1) as usize..],
                            &mut seq[(oldInverseSa0RelativeRank + 1) as usize..],
                            hi - oldInverseSa0RelativeRank - 1,
                        );
                    }
                }
            }

            BWTIncBuildRelativeRank(
                &mut sortedRank,
                &mut seq,
                &mut relativeRank,
                numChar,
                bwtInc.bwt.inverseSa0,
                &bwtInc.cumulativeCountInCurrentBuild,
            );
            assert_eq!(relativeRank[numChar as usize], oldInverseSa0RelativeRank);

            let mut qs_v = relativeRank
                .iter()
                .take(numChar as usize + 1)
                .map(|&x| x as i64)
                .collect::<Vec<_>>();
            let mut qs_i = seq
                .iter()
                .take(numChar as usize + 1)
                .map(|&x| x as i64)
                .collect::<Vec<_>>();
            QSufSortSuffixSort(&mut qs_v, &mut qs_i, numChar as i64, numChar as i64, 1, 1);
            for i in 0..=numChar as usize {
                relativeRank[i] = qs_v[i] as u64;
                seq[i] = qs_i[i] as u64;
            }

            let newInverseSa0RelativeRank = relativeRank[0];
            newInverseSa0 =
                sortedRank[newInverseSa0RelativeRank as usize] + newInverseSa0RelativeRank;
            sortedRank[newInverseSa0RelativeRank as usize] = 0;

            BWTIncBuildBwt(
                &mut insertBwt,
                &relativeRank,
                numChar,
                &bwtInc.cumulativeCountInCurrentBuild,
            );

            mergedBwt = vec![0u32; mergedBwtSizeInWord as usize + 2];
            let mut oldBwt = bwtInc.bwt.bwtCode.clone();
            oldBwt.resize(oldBwt.len() + 2, 0);
            BWTIncMergeBwt(
                &sortedRank,
                &oldBwt,
                &insertBwt,
                &mut mergedBwt,
                bwtInc.bwt.textLength,
                numChar,
            );
        }

        bwtInc.bwt.textLength += numChar;
        bwtInc.bwt.bwtCode = mergedBwt;
        bwtInc.bwt.bwtSizeInWord = mergedBwtSizeInWord;
        bwtInc.bwt.occSizeInWord = mergedOccSizeInWord;
        bwtInc.bwt.occValue = vec![0; mergedOccSizeInWord as usize];

        BWTClearTrailingBwtCode(&mut bwtInc.bwt);
        BWTGenerateOccValueFromBwt(
            &bwtInc.bwt.bwtCode,
            &mut bwtInc.bwt.occValue,
            &mut bwtInc.bwt.occValueMajor,
            bwtInc.bwt.textLength,
            &bwtInc.bwt.decodeTable,
        );

        bwtInc.bwt.inverseSa0 = newInverseSa0;

        bwtInc.bwt.cumulativeFreq[1] +=
            bwtInc.cumulativeCountInCurrentBuild[1] - (bwtInc.firstCharInLastIteration <= 0) as u64;
        bwtInc.bwt.cumulativeFreq[2] +=
            bwtInc.cumulativeCountInCurrentBuild[2] - (bwtInc.firstCharInLastIteration <= 1) as u64;
        bwtInc.bwt.cumulativeFreq[3] +=
            bwtInc.cumulativeCountInCurrentBuild[3] - (bwtInc.firstCharInLastIteration <= 2) as u64;
        bwtInc.bwt.cumulativeFreq[4] +=
            bwtInc.cumulativeCountInCurrentBuild[4] - (bwtInc.firstCharInLastIteration <= 3) as u64;

        bwtInc.firstCharInLastIteration = firstCharInThisIteration;
        BWTIncSetBuildSizeAndTextAddr(bwtInc);
        bwtInc.numberOfIterationDone += 1;
    }

    /// Original C global function `BWTIncConstructFromPacked` from `minibwa/bwtgen.c:1427`.
    pub fn BWTIncConstructFromPacked(
        inputFileName: &std::path::Path,
        initialMaxBuildSize: bgint_t,
        incMaxBuildSize: bgint_t,
    ) -> std::io::Result<BWTInc> {
        let packed = match std::fs::read(inputFileName) {
            Ok(packed) => packed,
            Err(err) => {
                let reason = err
                    .raw_os_error()
                    .map(|code| unsafe {
                        std::ffi::CStr::from_ptr(libc::strerror(code))
                            .to_string_lossy()
                            .into_owned()
                    })
                    .unwrap_or_else(|| err.to_string());
                eprintln!(
                    "BWTIncConstructFromPacked() : Cannot open {} : {}",
                    inputFileName.display(),
                    reason
                );
                std::process::exit(1);
            }
        };
        if packed.is_empty() {
            let reason = unsafe {
                std::ffi::CStr::from_ptr(libc::strerror(libc::EINVAL))
                    .to_string_lossy()
                    .into_owned()
            };
            eprintln!(
                "BWTIncConstructFromPacked() : Can't seek on {} : {}",
                inputFileName.display(),
                reason
            );
            std::process::exit(1);
        }
        let lastByteLength = *packed.last().unwrap() as u32;
        let packedFileLen = packed.len() as u64 - 1;
        let totalTextLength = TextLengthFromBytePacked(packedFileLen, BIT_PER_CHAR, lastByteLength);
        let packedData = &packed[..packed.len() - 1];

        let mut bwtInc = BWTIncCreate(
            totalTextLength,
            initialMaxBuildSize as u32,
            incMaxBuildSize as u32,
        );
        BWTIncSetBuildSizeAndTextAddr(&mut bwtInc);

        let mut textToLoad = if bwtInc.buildSize > totalTextLength {
            totalTextLength
        } else {
            totalTextLength
                - ((totalTextLength - bwtInc.buildSize + CHAR_PER_WORD - 1) / CHAR_PER_WORD
                    * CHAR_PER_WORD)
        };
        let mut textSizeInByte = textToLoad / CHAR_PER_BYTE as u64;
        let mut chunkStart = packedData
            .len()
            .checked_sub(textSizeInByte as usize + 1)
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "packed input is too short",
                )
            })?;
        bwtInc.textBuffer =
            packedData[chunkStart..chunkStart + textSizeInByte as usize + 1].to_vec();
        bwtInc.packedText =
            vec![
                0;
                (textToLoad as usize + CHAR_PER_WORD as usize - 1) / CHAR_PER_WORD as usize + 1
            ];
        ConvertBytePackedToWordPacked(
            &bwtInc.textBuffer,
            &mut bwtInc.packedText,
            ALPHABET_SIZE as u32,
            textToLoad,
        );
        BWTIncConstruct(&mut bwtInc, textToLoad);

        let mut processedTextLength = textToLoad;
        while processedTextLength < totalTextLength {
            textToLoad = bwtInc.buildSize / CHAR_PER_WORD * CHAR_PER_WORD;
            if textToLoad > totalTextLength - processedTextLength {
                textToLoad = totalTextLength - processedTextLength;
            }
            textSizeInByte = textToLoad / CHAR_PER_BYTE as u64;
            chunkStart = chunkStart
                .checked_sub(textSizeInByte as usize)
                .ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        "packed input is too short",
                    )
                })?;
            bwtInc.textBuffer =
                packedData[chunkStart..chunkStart + textSizeInByte as usize].to_vec();
            bwtInc.packedText =
                vec![
                    0;
                    (textToLoad as usize + CHAR_PER_WORD as usize - 1) / CHAR_PER_WORD as usize + 1
                ];
            ConvertBytePackedToWordPacked(
                &bwtInc.textBuffer,
                &mut bwtInc.packedText,
                ALPHABET_SIZE as u32,
                textToLoad,
            );
            BWTIncConstruct(&mut bwtInc, textToLoad);
            processedTextLength += textToLoad;
            if bwtInc.numberOfIterationDone % 10 == 0 {
                eprintln!(
                    "[BWTIncConstructFromPacked] {} iterations done. {} characters processed.",
                    bwtInc.numberOfIterationDone, processedTextLength
                );
            }
        }

        Ok(bwtInc)
    }

    /// Original C global function `BWTIncFree` from `minibwa/bwtgen.c:1537`.
    pub fn BWTIncFree(bwtInc: Option<BWTInc>) {
        if bwtInc.is_none() {
            return;
        }
    }

    /// Original C static function `BWTFileSizeInWord` from `minibwa/bwtgen.c:1550`.
    pub fn BWTFileSizeInWord(numChar: bgint_t) -> bgint_t {
        (numChar + CHAR_PER_WORD - 1) / CHAR_PER_WORD
    }

    /// Original C global function `BWTSaveBwtCodeAndOcc` from `minibwa/bwtgen.c:1556`.
    pub fn BWTSaveBwtCodeAndOcc(
        bwt: &BWT,
        bwtFileName: &std::path::Path,
        occValueFileName: Option<&std::path::Path>,
    ) -> std::io::Result<()> {
        let mut bwtFile = match std::fs::File::create(bwtFileName) {
            Ok(file) => file,
            Err(err) => {
                let reason = err
                    .raw_os_error()
                    .map(|code| unsafe {
                        std::ffi::CStr::from_ptr(libc::strerror(code))
                            .to_string_lossy()
                            .into_owned()
                    })
                    .unwrap_or_else(|| err.to_string());
                eprintln!(
                    "BWTSaveBwtCodeAndOcc(): Cannot open {} for writing: {}",
                    bwtFileName.display(),
                    reason
                );
                std::process::exit(1);
            }
        };
        let bwtLength = BWTFileSizeInWord(bwt.textLength);
        use std::io::Write;
        bwtFile.write_all(&bwt.inverseSa0.to_le_bytes())?;
        for i in 1..=ALPHABET_SIZE {
            bwtFile.write_all(&bwt.cumulativeFreq[i].to_le_bytes())?;
        }
        for &word in bwt.bwtCode.iter().take(bwtLength as usize) {
            bwtFile.write_all(&word.to_le_bytes())?;
        }
        let _ = occValueFileName;
        Ok(())
    }

    /// Original C global function `mb_bwtgen` from `minibwa/bwtgen.c:1588`.
    pub fn mb_bwtgen(
        fn_pac: &std::path::Path,
        fn_bwt: &std::path::Path,
        block_size: i32,
    ) -> std::io::Result<()> {
        let bwtInc = BWTIncConstructFromPacked(fn_pac, block_size as u64, block_size as u64)?;
        eprintln!(
            "[bwt_gen] Finished constructing BWT in {} iterations.",
            bwtInc.numberOfIterationDone
        );
        BWTSaveBwtCodeAndOcc(&bwtInc.bwt, fn_bwt, None)?;
        BWTIncFree(Some(bwtInc));
        Ok(())
    }
}

pub mod cs {
    #![allow(unused_variables, dead_code, non_snake_case)]

    use crate::align::{
        MB_CIGAR_DEL, MB_CIGAR_EQ_MATCH, MB_CIGAR_INS, MB_CIGAR_MATCH, MB_CIGAR_N_SKIP,
        MB_CIGAR_X_MISMATCH,
    };
    use crate::kommon::{km_sprintf_lite, kom_sprintf_arg, kstring_t};
    use crate::pe::mb_hit_t;

    /// Original C static function `alloc_tmp` from `minibwa/cs.c:7`.
    pub fn alloc_tmp(km: (), r: &mb_hit_t) -> usize {
        let mut min_tmp_len = 31usize;
        let Some(p) = &r.p else {
            return min_tmp_len + 1;
        };
        for &cg in p.cigar().iter().take(p.n_cigar as usize) {
            let op = cg & 0xf;
            let len = (cg >> 4) as usize;
            if op == MB_CIGAR_INS || op == MB_CIGAR_DEL {
                min_tmp_len = min_tmp_len.max(len + 5);
            }
        }
        min_tmp_len + 1
    }

    /// Original C static function `write_indel_ds` from `minibwa/cs.c:18`.
    pub fn write_indel_ds(km: (), str_: &mut kstring_t, len: i64, seq: &[u8], ll: i64, lr: i64) {
        const BASES: &[u8; 5] = b"acgtn";
        if ll + lr >= len {
            km_sprintf_lite(km, str_, "[", &[]);
            for i in 0..len as usize {
                km_sprintf_lite(
                    km,
                    str_,
                    "%c",
                    &[kom_sprintf_arg::c(BASES[seq[i] as usize] as i32)],
                );
            }
            km_sprintf_lite(km, str_, "]", &[]);
        } else {
            let mut k = 0usize;
            if ll > 0 {
                km_sprintf_lite(km, str_, "[", &[]);
                for i in 0..ll as usize {
                    km_sprintf_lite(
                        km,
                        str_,
                        "%c",
                        &[kom_sprintf_arg::c(BASES[seq[k + i] as usize] as i32)],
                    );
                }
                km_sprintf_lite(km, str_, "]", &[]);
                k += ll as usize;
            }
            for i in 0..(len - lr - ll) as usize {
                km_sprintf_lite(
                    km,
                    str_,
                    "%c",
                    &[kom_sprintf_arg::c(BASES[seq[k + i] as usize] as i32)],
                );
            }
            k += (len - lr - ll) as usize;
            if lr > 0 {
                km_sprintf_lite(km, str_, "[", &[]);
                for i in 0..lr as usize {
                    km_sprintf_lite(
                        km,
                        str_,
                        "%c",
                        &[kom_sprintf_arg::c(BASES[seq[k + i] as usize] as i32)],
                    );
                }
                km_sprintf_lite(km, str_, "]", &[]);
            }
        }
    }

    /// Original C global function `mb_write_cs_ds` from `minibwa/cs.c:47`.
    pub fn mb_write_cs_ds(
        km: (),
        s: &mut kstring_t,
        tseq: &[u8],
        qseq: &[u8],
        r: &mb_hit_t,
        is_ds: i32,
    ) {
        const BASES: &[u8; 5] = b"acgtn";
        let Some(p) = &r.p else {
            return;
        };
        let mut q_len = 0usize;
        let mut t_len = 0usize;
        km_sprintf_lite(
            km,
            s,
            "%cs:Z:",
            &[kom_sprintf_arg::c(
                if is_ds != 0 { b'd' } else { b'c' } as i32
            )],
        );
        for &cg in p.cigar().iter().take(p.n_cigar as usize) {
            let op = cg & 0xf;
            let len = (cg >> 4) as usize;
            if op == MB_CIGAR_MATCH || op == MB_CIGAR_EQ_MATCH || op == MB_CIGAR_X_MISMATCH {
                q_len += len;
                t_len += len;
            } else if op == MB_CIGAR_INS {
                q_len += len;
            } else if op == MB_CIGAR_DEL || op == MB_CIGAR_N_SKIP {
                t_len += len;
            }
        }
        let _tmp = vec![0u8; alloc_tmp(km, r)];
        let mut q_off = 0usize;
        let mut t_off = 0usize;
        for &cg in p.cigar().iter().take(p.n_cigar as usize) {
            let op = cg & 0xf;
            let len = (cg >> 4) as usize;
            assert!(
                (MB_CIGAR_MATCH..=MB_CIGAR_N_SKIP).contains(&op)
                    || op == MB_CIGAR_EQ_MATCH
                    || op == MB_CIGAR_X_MISMATCH
            );
            if op == MB_CIGAR_MATCH || op == MB_CIGAR_EQ_MATCH || op == MB_CIGAR_X_MISMATCH {
                let mut l_tmp = 0i32;
                for j in 0..len {
                    if qseq[q_off + j] != tseq[t_off + j] {
                        if l_tmp > 0 {
                            km_sprintf_lite(km, s, ":%d", &[kom_sprintf_arg::d(l_tmp)]);
                            l_tmp = 0;
                        }
                        km_sprintf_lite(
                            km,
                            s,
                            "*%c%c",
                            &[
                                kom_sprintf_arg::c(BASES[tseq[t_off + j] as usize] as i32),
                                kom_sprintf_arg::c(BASES[qseq[q_off + j] as usize] as i32),
                            ],
                        );
                    } else {
                        l_tmp += 1;
                    }
                }
                if l_tmp > 0 {
                    km_sprintf_lite(km, s, ":%d", &[kom_sprintf_arg::d(l_tmp)]);
                }
                q_off += len;
                t_off += len;
            } else if op == MB_CIGAR_INS {
                if is_ds != 0 {
                    let y = q_off;
                    let mut z = 1usize;
                    while z <= len {
                        if y < z || qseq[y + len - z] != qseq[y - z] {
                            break;
                        }
                        z += 1;
                    }
                    let lr = z - 1;
                    z = 0;
                    while z < len {
                        if y + len + z >= q_len || qseq[y + len + z] != qseq[y + z] {
                            break;
                        }
                        z += 1;
                    }
                    let ll = z;
                    km_sprintf_lite(km, s, "+", &[]);
                    write_indel_ds(km, s, len as i64, &qseq[y..], ll as i64, lr as i64);
                } else {
                    let text: String = qseq[q_off..q_off + len]
                        .iter()
                        .map(|&b| BASES[b as usize] as char)
                        .collect();
                    km_sprintf_lite(km, s, "+%s", &[kom_sprintf_arg::s(&text)]);
                }
                q_off += len;
            } else if op == MB_CIGAR_DEL {
                if is_ds != 0 {
                    let x = t_off;
                    let mut z = 1usize;
                    while z <= len {
                        if x < z || tseq[x + len - z] != tseq[x - z] {
                            break;
                        }
                        z += 1;
                    }
                    let lr = z - 1;
                    z = 0;
                    while z < len {
                        if x + len + z >= t_len || tseq[x + z] != tseq[x + len + z] {
                            break;
                        }
                        z += 1;
                    }
                    let ll = z;
                    km_sprintf_lite(km, s, "-", &[]);
                    write_indel_ds(km, s, len as i64, &tseq[x..], ll as i64, lr as i64);
                } else {
                    let text: String = tseq[t_off..t_off + len]
                        .iter()
                        .map(|&b| BASES[b as usize] as char)
                        .collect();
                    km_sprintf_lite(km, s, "-%s", &[kom_sprintf_arg::s(&text)]);
                }
                t_off += len;
            } else {
                assert!(len >= 2);
                km_sprintf_lite(
                    km,
                    s,
                    "~%c%c%d%c%c",
                    &[
                        kom_sprintf_arg::c(BASES[tseq[t_off] as usize] as i32),
                        kom_sprintf_arg::c(BASES[tseq[t_off + 1] as usize] as i32),
                        kom_sprintf_arg::d(len as i32),
                        kom_sprintf_arg::c(BASES[tseq[t_off + len - 2] as usize] as i32),
                        kom_sprintf_arg::c(BASES[tseq[t_off + len - 1] as usize] as i32),
                    ],
                );
                t_off += len;
            }
        }
        assert_eq!(t_off as i64, r.te - r.ts);
        assert_eq!(q_off as i32, r.qe - r.qs);
    }

    /// Original C global function `mb_write_MD` from `minibwa/cs.c:129`.
    pub fn mb_write_MD(km: (), s: &mut kstring_t, tseq: &[u8], qseq: &[u8], r: &mb_hit_t) {
        const BASES: &[u8; 5] = b"ACGTN";
        let Some(p) = &r.p else {
            return;
        };
        km_sprintf_lite(km, s, "\tMD:Z:", &[]);
        let _tmp = vec![0u8; alloc_tmp(km, r)];
        let mut q_off = 0usize;
        let mut t_off = 0usize;
        let mut l_MD = 0i32;
        for &cg in p.cigar().iter().take(p.n_cigar as usize) {
            let op = cg & 0xf;
            let len = (cg >> 4) as usize;
            assert!(
                (MB_CIGAR_MATCH..=MB_CIGAR_N_SKIP).contains(&op)
                    || op == MB_CIGAR_EQ_MATCH
                    || op == MB_CIGAR_X_MISMATCH
            );
            if op == MB_CIGAR_MATCH || op == MB_CIGAR_EQ_MATCH || op == MB_CIGAR_X_MISMATCH {
                for j in 0..len {
                    if qseq[q_off + j] != tseq[t_off + j] {
                        km_sprintf_lite(
                            km,
                            s,
                            "%d%c",
                            &[
                                kom_sprintf_arg::d(l_MD),
                                kom_sprintf_arg::c(BASES[tseq[t_off + j] as usize] as i32),
                            ],
                        );
                        l_MD = 0;
                    } else {
                        l_MD += 1;
                    }
                }
                q_off += len;
                t_off += len;
            } else if op == MB_CIGAR_INS {
                q_off += len;
            } else if op == MB_CIGAR_DEL {
                let text: String = tseq[t_off..t_off + len]
                    .iter()
                    .map(|&b| BASES[b as usize] as char)
                    .collect();
                km_sprintf_lite(
                    km,
                    s,
                    "%d^%s",
                    &[kom_sprintf_arg::d(l_MD), kom_sprintf_arg::s(&text)],
                );
                l_MD = 0;
                t_off += len;
            } else if op == MB_CIGAR_N_SKIP {
                t_off += len;
            }
        }
        if l_MD > 0 {
            km_sprintf_lite(km, s, "%d", &[kom_sprintf_arg::d(l_MD)]);
        }
        assert_eq!(t_off as i64, r.te - r.ts);
        assert_eq!(q_off as i32, r.qe - r.qs);
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::pe::{mb_extra_t, mb_hit_t};

        #[test]
        fn cs_and_md_tags_follow_cigar_edits() {
            let hit = mb_hit_t {
                qs: 0,
                qe: 6,
                ts: 0,
                te: 5,
                p: Some(
                    mb_extra_t {
                        ..Default::default()
                    }
                    .with_cigar(&[
                        2 << 4 | MB_CIGAR_MATCH,
                        1 << 4 | MB_CIGAR_INS,
                        3 << 4 | MB_CIGAR_MATCH,
                    ]),
                ),
                ..Default::default()
            };
            let q = [0, 1, 2, 3, 0, 1];
            let t = [0, 1, 3, 0, 1];
            let mut s = kstring_t::default();
            mb_write_cs_ds((), &mut s, &t, &q, &hit, 0);
            assert_eq!(String::from_utf8_lossy(&s.s[..s.l]), "cs:Z::2+g:3");

            let mut md = kstring_t::default();
            mb_write_MD((), &mut md, &t, &q, &hit);
            assert_eq!(String::from_utf8_lossy(&md.s[..md.l]), "\tMD:Z:5");
        }

        #[test]
        fn ds_marks_repeat_context() {
            let mut s = kstring_t::default();
            write_indel_ds((), &mut s, 3, &[0, 0, 0], 1, 1);
            assert_eq!(String::from_utf8_lossy(&s.s[..s.l]), "[a]a[a]");
        }
    }
}

pub mod fastmap {
    #![allow(unused_variables, dead_code, non_snake_case, non_camel_case_types)]

    use crate::bseq::{mb_bseq_open, mb_bseq_read};
    use crate::bwt::{
        mb_bwt_sa_batch, mb_bwt_smem, mb_bwt_smem_batch_ref_with_queue, mb_sai_t, mb_sai_v,
        mb_smem_entry_ref, tiny_queue_t,
    };
    use crate::ketopt::{ketopt, ko_longopt_t, KETOPT_INIT};
    use crate::kommon::kstring_t;
    use crate::l2bit::l2b_intv2cid;
    use crate::map_algo::{mb_idx_load, mb_idx_t};
    use std::io::Write;

    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    pub struct batch_seq1_t {
        pub name: String,
        pub l_seq: i32,
        pub seq: Vec<u8>,
        pub v: mb_sai_v,
    }

    fn ks_put_bytes(out: &mut kstring_t, bytes: &[u8]) {
        let end = out.l + bytes.len();
        if end + 1 > out.s.len() {
            out.s.resize(end + 1, 0);
            out.m = out.s.len();
        }
        out.s[out.l..end].copy_from_slice(bytes);
        out.l = end;
        out.s[out.l] = 0;
    }

    fn ks_put_u64(out: &mut kstring_t, mut x: u64) {
        let mut buf = [0u8; 20];
        let mut i = buf.len();
        loop {
            i -= 1;
            buf[i] = b'0' + (x % 10) as u8;
            x /= 10;
            if x == 0 {
                break;
            }
        }
        ks_put_bytes(out, &buf[i..]);
    }

    fn ks_put_i64(out: &mut kstring_t, x: i64) {
        if x < 0 {
            ks_put_bytes(out, b"-");
            ks_put_u64(out, x.unsigned_abs());
        } else {
            ks_put_u64(out, x as u64);
        }
    }

    fn ks_put_i32(out: &mut kstring_t, x: i32) {
        ks_put_i64(out, x as i64);
    }

    fn emit_kstring(
        out: &kstring_t,
        emitted: &mut String,
        output_writer: &mut Option<&mut dyn Write>,
    ) -> bool {
        if let Some(writer) = output_writer.as_deref_mut() {
            writer.write_all(&out.s[..out.l]).is_ok()
        } else {
            emitted.push_str(std::str::from_utf8(&out.s[..out.l]).unwrap());
            true
        }
    }

    /// Original C static function `batch_smem` from `minibwa/fastmap.c:17`.
    pub fn batch_smem(idx: &mb_idx_t, n: i32, t: &mut [batch_seq1_t], min_len: i32, min_occ: i32) {
        let mut tq = tiny_queue_t::default();
        batch_smem_with_queue(idx, n, t, min_len, min_occ, &mut tq);
    }

    fn batch_smem_with_queue(
        idx: &mb_idx_t,
        n: i32,
        t: &mut [batch_seq1_t],
        min_len: i32,
        min_occ: i32,
        tq: &mut tiny_queue_t,
    ) {
        let mut v = Vec::with_capacity(n as usize);
        for ti in t.iter_mut().take(n as usize) {
            ti.v.n = 0;
            ti.v.a.clear();
            v.push(std::mem::take(&mut ti.v));
        }
        let mut s = Vec::with_capacity(n as usize);
        let v_ptr = v.as_mut_ptr();
        for (i, ti) in t.iter().take(n as usize).enumerate() {
            s.push(mb_smem_entry_ref {
                min_len,
                min_occ,
                st: 0,
                en: ti.l_seq,
                q: ti.seq.as_ptr(),
                v: unsafe { v_ptr.add(i) },
                stage: 0,
                x: 0,
                i: 0,
                kmer: 0,
                p: mb_sai_t::default(),
            });
        }
        mb_bwt_smem_batch_ref_with_queue((), &idx.bwt, n, &mut s, &mut v, tq);
        for (ti, vi) in t.iter_mut().zip(v.into_iter()).take(n as usize) {
            ti.v = vi;
        }
    }

    /// Original C static function `write_intv` from `minibwa/fastmap.c:35`.
    pub fn write_intv(
        idx: &mb_idx_t,
        p: &mb_sai_t,
        max_size_out: i32,
        sa: &mut [u64],
        out: &mut kstring_t,
    ) {
        let len = (p.info & 0xffff_ffff).wrapping_sub(p.info >> 32);
        ks_put_bytes(out, b"EM\t");
        ks_put_u64(out, p.info >> 32);
        ks_put_bytes(out, b"\t");
        ks_put_u64(out, p.info & 0xffff_ffff);
        ks_put_bytes(out, b"\t");
        ks_put_u64(out, p.size);
        if p.size <= max_size_out as u64 {
            let n_sa = p.size as usize;
            for (j, slot) in sa.iter_mut().take(n_sa).enumerate() {
                *slot = p.x[0] + j as u64;
            }
            mb_bwt_sa_batch((), &idx.bwt, p.size as i64, sa);
            for &pos in sa.iter().take(n_sa) {
                let mut rev = 0;
                let mut cst = 0i64;
                let cid = l2b_intv2cid(&idx.l2b, pos, pos + len, &mut cst, &mut rev);
                if cid < 0 {
                    ks_put_bytes(out, b"\t.");
                } else {
                    let name = &idx.l2b.ctg[cid as usize].name;
                    ks_put_bytes(out, b"\t");
                    ks_put_bytes(out, name.as_bytes());
                    ks_put_bytes(out, if rev != 0 { b":-" } else { b":+" });
                    ks_put_i64(out, cst + 1);
                }
            }
        } else {
            ks_put_bytes(out, b"\t*");
        }
        ks_put_bytes(out, b"\n");
    }

    /// Original C static function `process_batch` from `minibwa/fastmap.c:56`.
    pub fn process_batch(
        idx: &mb_idx_t,
        n: i32,
        t: &mut [batch_seq1_t],
        min_len: i32,
        min_occ: i32,
        max_size_out: i32,
        sa: &mut [u64],
        out: &mut kstring_t,
    ) -> String {
        let mut emitted = String::new();
        let mut tq = tiny_queue_t::default();
        let mut output_writer = None;
        if process_batch_append(
            idx,
            n,
            t,
            min_len,
            min_occ,
            max_size_out,
            sa,
            out,
            &mut emitted,
            &mut tq,
            &mut output_writer,
        ) {
            emitted
        } else {
            String::new()
        }
    }

    fn process_batch_append(
        idx: &mb_idx_t,
        n: i32,
        t: &mut [batch_seq1_t],
        min_len: i32,
        min_occ: i32,
        max_size_out: i32,
        sa: &mut [u64],
        out: &mut kstring_t,
        emitted: &mut String,
        tq: &mut tiny_queue_t,
        output_writer: &mut Option<&mut dyn Write>,
    ) -> bool {
        batch_smem_with_queue(idx, n, t, min_len, min_occ, tq);
        for ti in t.iter_mut().take(n as usize) {
            out.l = 0;
            ks_put_bytes(out, b"SQ\t");
            ks_put_bytes(out, ti.name.as_bytes());
            ks_put_bytes(out, b"\t");
            ks_put_i32(out, ti.l_seq);
            ks_put_bytes(out, b"\n");
            for p in ti.v.a.iter().take(ti.v.n) {
                write_intv(idx, p, max_size_out, sa, out);
            }
            ks_put_bytes(out, b"//\n");
            if !emit_kstring(out, emitted, output_writer) {
                return false;
            }
        }
        true
    }

    /// Original C static function `usage_fastmap` from `minibwa/fastmap.c:63`.
    pub fn usage_fastmap(
        to_stdout: bool,
        min_len: i32,
        min_occ: i32,
        max_size_out: i32,
        max_seq: i32,
    ) -> (i32, String) {
        let text = format!(
            "Usage: minibwa fastmap [options] <idx-prefix> <in.fq>\nOptions:\n  -l INT     min seed length [{min_len}]\n  -s INT     min interval size [{min_occ}]\n  -w INT     max interval size to output coordinates [{max_size_out}]\n  -b INT     batch size [{max_seq}]\n  --help     print this help message\n"
        );
        (if to_stdout { 0 } else { 1 }, text)
    }

    /// Original C global function `main_fastmap` from `minibwa/fastmap.c:73`.
    pub fn main_fastmap(argv: &[String]) -> (i32, String) {
        main_fastmap_inner(argv, None)
    }

    pub fn main_fastmap_write(argv: &[String], output_writer: &mut dyn Write) -> (i32, String) {
        main_fastmap_inner(argv, Some(output_writer))
    }

    fn main_fastmap_inner(
        argv: &[String],
        mut output_writer: Option<&mut dyn Write>,
    ) -> (i32, String) {
        let long_opts = [ko_longopt_t {
            name: Some("help".into()),
            has_arg: 0,
            val: 901,
        }];
        let argc = argv.len() as i32;
        let mut args = argv.to_vec();
        let mut o = KETOPT_INIT.clone();
        let mut min_len = 19;
        let mut min_occ = 1;
        let mut max_size_out = 20;
        let mut max_seq = 1;
        loop {
            let c = ketopt(&mut o, argc, &mut args, 1, "l:s:w:b:", Some(&long_opts));
            if c < 0 {
                break;
            }
            if c == 'l' as i32 {
                min_len = o.arg.as_deref().unwrap_or("0").parse().unwrap_or(0);
            } else if c == 's' as i32 {
                min_occ = o.arg.as_deref().unwrap_or("0").parse().unwrap_or(0);
            } else if c == 'w' as i32 {
                max_size_out = o.arg.as_deref().unwrap_or("0").parse().unwrap_or(0);
            } else if c == 'b' as i32 {
                max_seq = o.arg.as_deref().unwrap_or("0").parse().unwrap_or(0);
            } else if c == 901 {
                return usage_fastmap(true, min_len, min_occ, max_size_out, max_seq);
            }
        }
        if argc - o.ind < 2 {
            return usage_fastmap(false, min_len, min_occ, max_size_out, max_seq);
        }
        let Some(idx) = mb_idx_load(&args[o.ind as usize], 0) else {
            return (1, String::new());
        };
        let Some(mut fp) = mb_bseq_open(Some(&args[o.ind as usize + 1])) else {
            return (1, String::new());
        };
        fp.suppress_parse_warnings = true;
        let mut emitted = String::new();
        let mut out = kstring_t::default();
        let mut sa = vec![0u64; max_size_out.max(0) as usize];
        let mut tq = tiny_queue_t::default();
        let mut batch = if max_seq > 1 {
            vec![batch_seq1_t::default(); max_seq as usize]
        } else {
            Vec::new()
        };
        loop {
            let mut n_read = 0;
            let reads = mb_bseq_read(
                &mut fp,
                if max_seq > 1 { max_seq as i64 } else { 1 },
                0,
                0,
                0,
                1,
                if max_seq > 1 { max_seq as i64 } else { 1 },
                &mut n_read,
            );
            if n_read == 0 {
                break;
            }
            if max_seq > 1 {
                let n_read_usize = n_read as usize;
                if batch.len() < n_read_usize {
                    batch.resize_with(n_read_usize, batch_seq1_t::default);
                }
                for (slot, r) in batch.iter_mut().zip(reads.iter()).take(n_read_usize) {
                    slot.name.clear();
                    slot.name.push_str(&r.name);
                    slot.l_seq = r.l_seq as i32;
                    slot.seq.clear();
                    slot.seq.extend(r.seq.bytes().map(|c| match c {
                        b'A' | b'a' => 0,
                        b'C' | b'c' => 1,
                        b'G' | b'g' => 2,
                        b'T' | b't' => 3,
                        _ => 4,
                    }));
                }
                if !process_batch_append(
                    &idx,
                    n_read,
                    &mut batch,
                    min_len,
                    min_occ,
                    max_size_out,
                    &mut sa,
                    &mut out,
                    &mut emitted,
                    &mut tq,
                    &mut output_writer,
                ) {
                    return (-1, String::new());
                }
            } else {
                let r = &reads[0];
                let seq = r
                    .seq
                    .bytes()
                    .map(|c| match c {
                        b'A' | b'a' => 0,
                        b'C' | b'c' => 1,
                        b'G' | b'g' => 2,
                        b'T' | b't' => 3,
                        _ => 4,
                    })
                    .collect::<Vec<_>>();
                out.l = 0;
                ks_put_bytes(&mut out, b"SQ\t");
                ks_put_bytes(&mut out, r.name.as_bytes());
                ks_put_bytes(&mut out, b"\t");
                ks_put_u64(&mut out, r.l_seq);
                ks_put_bytes(&mut out, b"\n");
                let mut x = 0i64;
                let mut a = Vec::<mb_sai_t>::new();
                while x < r.l_seq as i64 {
                    let mut p = mb_sai_t::default();
                    x = mb_bwt_smem(
                        &idx.bwt,
                        r.l_seq as u32,
                        &seq,
                        x,
                        min_len as i64,
                        min_occ as i64,
                        &mut p,
                    );
                    if p.size > 0 {
                        a.push(p);
                    }
                }
                for p in &a {
                    write_intv(&idx, p, max_size_out, &mut sa, &mut out);
                }
                ks_put_bytes(&mut out, b"//\n");
                if !emit_kstring(&out, &mut emitted, &mut output_writer) {
                    return (-1, String::new());
                }
            }
        }
        if let Some(writer) = output_writer {
            if writer.flush().is_err() {
                return (-1, String::new());
            }
        }
        (0, emitted)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::bwt::mb_bwt_init;
        use crate::l2bit::{l2b_ctg_t, l2b_t};

        #[test]
        fn write_intv_emits_interval_summary_and_size_cutoff() {
            let idx = mb_idx_t {
                is_meth: 0,
                l2b: l2b_t {
                    tot_len: 100,
                    n_ctg: 1,
                    ctg: vec![l2b_ctg_t {
                        name: "ctg".to_string(),
                        len: 100,
                        off: 0,
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                bwt: mb_bwt_init(),
            };
            let p = mb_sai_t {
                x: [5, 0],
                size: 3,
                info: (2 << 32) | 8,
            };
            let mut out = kstring_t::default();
            let mut sa = vec![0; 3];
            write_intv(&idx, &p, 2, &mut sa, &mut out);
            assert_eq!(String::from_utf8_lossy(&out.s[..out.l]), "EM\t2\t8\t3\t*\n");
        }

        #[test]
        fn usage_fastmap_formats_defaults_and_status() {
            let (status, text) = usage_fastmap(false, 19, 1, 20, 1);
            assert_eq!(status, 1);
            assert!(text.contains("Usage: minibwa fastmap"));
            assert!(text.contains("-l INT     min seed length [19]"));
        }

        #[test]
        fn main_fastmap_emits_smems_on_real_chrm_fixture() {
            let mut fq = std::env::temp_dir();
            fq.push(format!(
                "minibwa_rs_fastmap_{}_{}.fq",
                std::process::id(),
                crate::kommon::kom_realtime().to_bits()
            ));
            std::fs::write(
                &fq,
                b"@r0\nGATCACAGGTCTATCACCCTATTAACCACTCACGGGAGCTCTCCATGCAT\n+\nFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF\n",
            )
            .unwrap();
            let args = vec![
                "fastmap".to_string(),
                "-l".to_string(),
                "19".to_string(),
                "-w".to_string(),
                "2".to_string(),
                "minibwa/chrM-human".to_string(),
                fq.to_string_lossy().into_owned(),
            ];
            let (ret, out) = main_fastmap(&args);
            assert_eq!(ret, 0);
            assert!(out.starts_with("SQ\tr0\t"));
            assert!(out.contains("\nEM\t"));
            assert!(out.ends_with("//\n"));
            let _ = std::fs::remove_file(fq);
        }
    }
}

pub mod format {
    #![allow(unused_variables, dead_code, non_snake_case)]

    use crate::bseq::mb_bseq1_t;
    use crate::kommon::{kom_sprintf_arg, kom_sprintf_lite, kstring_t};
    use crate::l2bit::l2b_t;
    use crate::options::{MB_F_2ND_SEQ, MB_F_COPY_COMMENT, MB_F_SAM, MB_F_SUPP_SOFT};
    use crate::pe::{mb_hit_buf_t, mb_hit_t};
    use std::sync::{Mutex, OnceLock};

    const MB_CIGAR_STR: &[u8] = b"MIDNSHP=XB";

    static MB_RG_ID: OnceLock<Mutex<String>> = OnceLock::new();

    #[inline(always)]
    fn append_bytes(s: &mut kstring_t, bytes: &[u8]) {
        let end = s.l + bytes.len();
        if end > s.s.len() {
            s.s.resize(end, 0);
            s.m = s.s.len();
        }
        s.s[s.l..end].copy_from_slice(bytes);
        s.l = end;
    }

    #[inline(always)]
    fn append_str(s: &mut kstring_t, text: &str) {
        append_bytes(s, text.as_bytes());
    }

    #[inline(always)]
    fn append_byte(s: &mut kstring_t, byte: u8) {
        let end = s.l + 1;
        if end > s.s.len() {
            s.s.resize(end, 0);
            s.m = s.s.len();
        }
        s.s[s.l] = byte;
        s.l = end;
    }

    #[inline(always)]
    fn append_u64(s: &mut kstring_t, mut value: u64) {
        let mut buf = [0u8; 20];
        let mut i = buf.len();
        loop {
            i -= 1;
            buf[i] = b'0' + (value % 10) as u8;
            value /= 10;
            if value == 0 {
                break;
            }
        }
        append_bytes(s, &buf[i..]);
    }

    #[inline(always)]
    fn append_i64(s: &mut kstring_t, value: i64) {
        if value < 0 {
            append_byte(s, b'-');
            append_u64(s, value.unsigned_abs());
        } else {
            append_u64(s, value as u64);
        }
    }

    #[inline(always)]
    fn append_i32(s: &mut kstring_t, value: i32) {
        append_i64(s, value as i64);
    }

    /// Original C static function `write_tags` from `minibwa/format.c:13`.
    pub fn write_tags(s: &mut kstring_t, p: &mb_hit_t) {
        let extra = p.p.as_ref().unwrap();
        let nm = p.blen - p.mlen + extra.n_ambi() as i32;
        append_bytes(s, b"\tNM:i:");
        append_i32(s, nm);
        append_bytes(s, b"\tAS:i:");
        append_i32(s, extra.dp_score);
        append_bytes(s, b"\tms:i:");
        append_i32(s, extra.dp_max0);
        append_bytes(s, b"\tmd:i:");
        append_i32(s, extra.dp_max - extra.dp_max2);
    }

    /// Original C global function `mb_fmt_paf` from `minibwa/format.c:19`.
    pub fn mb_fmt_paf(
        s: &mut kstring_t,
        l2b: &l2b_t,
        t: &mb_bseq1_t,
        p: Option<&mb_hit_t>,
        opt_flag: u64,
        n_seg: i32,
        seg_idx: i32,
    ) {
        append_str(s, &t.name);
        if n_seg > 1 && seg_idx >= 0 {
            append_byte(s, b'/');
            append_i32(s, seg_idx + 1);
        }
        append_byte(s, b'\t');
        append_i64(s, t.l_seq as i64);
        let Some(p) = p else {
            append_bytes(s, b"\t*\t*\t*\t*\t*\t*\t*\t0\t0\t0\n");
            return;
        };
        let ctg = &l2b.ctg[p.tid as usize];
        append_byte(s, b'\t');
        append_i32(s, p.qs);
        append_byte(s, b'\t');
        append_i32(s, p.qe);
        append_byte(s, b'\t');
        append_byte(s, if p.rev() != 0 { b'-' } else { b'+' });
        append_byte(s, b'\t');
        append_str(s, &ctg.name);
        append_byte(s, b'\t');
        append_i64(s, ctg.len as i64);
        append_byte(s, b'\t');
        append_i64(s, p.ts);
        append_byte(s, b'\t');
        append_i64(s, p.te);
        append_byte(s, b'\t');
        append_i32(s, p.mlen);
        append_byte(s, b'\t');
        append_i32(s, p.blen);
        append_byte(s, b'\t');
        append_i32(s, p.mapq);
        append_bytes(s, b"\ttp:A:");
        append_byte(s, if p.parent == p.id { b'P' } else { b'S' });
        append_bytes(s, b"\ts1:i:");
        append_i32(s, p.score);
        append_bytes(s, b"\tcm:i:");
        append_i32(s, p.cnt);
        if p.parent == p.id {
            append_bytes(s, b"\ts2:i:");
            append_i32(s, if p.subsc >= 0 { p.subsc } else { 0 });
        }
        if let Some(extra) = &p.p {
            write_tags(s, p);
            if extra.n_cigar > 0 {
                append_bytes(s, b"\tcg:Z:");
                for &c in extra.cigar().iter().take(extra.n_cigar as usize) {
                    append_i32(s, (c >> 4) as i32);
                    append_byte(s, MB_CIGAR_STR[(c & 0xf) as usize]);
                }
            }
            if extra.cs() != 0 {
                let mut bytes = Vec::new();
                'outer: for &w in extra.cigar_all().iter().skip(extra.n_cigar as usize) {
                    for i in 0..4 {
                        let b = ((w >> (i * 8)) & 0xff) as u8;
                        if b == 0 {
                            break 'outer;
                        }
                        bytes.push(b);
                    }
                }
                let tag = String::from_utf8_lossy(&bytes);
                append_byte(s, b'\t');
                append_str(s, &tag);
            }
        }
        if (opt_flag & MB_F_COPY_COMMENT) != 0 {
            if let Some(comment) = &t.comment {
                append_byte(s, b'\t');
                append_str(s, comment);
            }
        }
        append_byte(s, b'\n');
    }

    /// Original C static function `mb_escape` from `minibwa/format.c:52`.
    pub fn mb_escape(s: &str) -> String {
        let b = s.as_bytes();
        let mut out = Vec::with_capacity(b.len());
        let mut i = 0usize;
        while i < b.len() {
            if b[i] == b'\\' {
                i += 1;
                if i < b.len() {
                    match b[i] {
                        b't' => out.push(b'\t'),
                        b'n' => out.push(b'\n'),
                        b'r' => out.push(b'\r'),
                        b'\\' => out.push(b'\\'),
                        _ => {}
                    }
                }
            } else {
                out.push(b[i]);
            }
            i += 1;
        }
        String::from_utf8_lossy(&out).into_owned()
    }

    /// Original C static function `sam_write_rg_line` from `minibwa/format.c:66`.
    pub fn sam_write_rg_line(str_: &mut kstring_t, s: Option<&str>) -> i32 {
        let store = MB_RG_ID.get_or_init(|| Mutex::new(String::new()));
        store.lock().unwrap().clear();
        let Some(s) = s else {
            return 0;
        };
        if !s.starts_with("@RG") {
            eprintln!("[ERROR] the read group line is not started with @RG");
            return -1;
        }
        if s.as_bytes().contains(&b'\t') {
            eprintln!(
                "[ERROR] the read group line contained literal <tab> characters -- replace with escaped tabs: \\t"
            );
            return -1;
        }
        let rg_line = mb_escape(s);
        let Some(id_start0) = rg_line.find("\tID:") else {
            eprintln!("[ERROR] no ID within the read group line");
            return -1;
        };
        let id_start = id_start0 + 4;
        let id_end = rg_line[id_start..]
            .find(|c| c == '\t' || c == '\n')
            .map(|x| id_start + x)
            .unwrap_or(rg_line.len());
        if id_end - id_start + 1 > 256 {
            eprintln!("[ERROR] @RG:ID is longer than 255 characters");
            return -1;
        }
        *store.lock().unwrap() = rg_line[id_start..id_end].to_string();
        kom_sprintf_lite(str_, "%s\n", &[kom_sprintf_arg::s(&rg_line)]);
        0
    }

    /// Original C global function `mb_fmt_sam_hdr` from `minibwa/format.c:101`.
    pub fn mb_fmt_sam_hdr(
        str_: &mut kstring_t,
        idx: Option<&l2b_t>,
        rg: Option<&str>,
        ver: Option<&str>,
        argv: &[&str],
    ) -> i32 {
        let mut ret = 0;
        str_.l = 0;
        kom_sprintf_lite(str_, "@HD\tVN:1.6\tSO:unsorted\tGO:query\n", &[]);
        if let Some(idx) = idx {
            for ctg in &idx.ctg[..idx.n_ctg as usize] {
                kom_sprintf_lite(
                    str_,
                    "@SQ\tSN:%s\tLN:%ld\n",
                    &[
                        kom_sprintf_arg::s(&ctg.name),
                        kom_sprintf_arg::ld(ctg.len as i64),
                    ],
                );
            }
        }
        if rg.is_some() {
            ret = sam_write_rg_line(str_, rg);
        }
        kom_sprintf_lite(str_, "@PG\tID:minibwa\tPN:minibwa", &[]);
        if let Some(ver) = ver {
            kom_sprintf_lite(str_, "\tVN:%s", &[kom_sprintf_arg::s(ver)]);
        }
        if argv.len() > 1 {
            kom_sprintf_lite(str_, "\tCL:minibwa", &[]);
            for arg in &argv[1..] {
                kom_sprintf_lite(str_, " %s", &[kom_sprintf_arg::s(arg)]);
            }
        }
        kom_sprintf_lite(str_, "\n", &[]);
        ret
    }

    /// Original C static function `str_enlarge` from `minibwa/format.c:125`.
    pub fn str_enlarge(s: &mut kstring_t, l: i32) {
        let need = s.l + l as usize + 1;
        if need > s.m {
            s.m = need;
            s.m = s.m.wrapping_sub(1);
            s.m |= s.m >> 1;
            s.m |= s.m >> 2;
            s.m |= s.m >> 4;
            s.m |= s.m >> 8;
            s.m |= s.m >> 16;
            s.m |= s.m >> 32;
            s.m = s.m.wrapping_add(1);
            s.s.resize(s.m, 0);
        }
    }

    /// Original C static function `str_copy` from `minibwa/format.c:134`.
    pub fn str_copy(s: &mut kstring_t, st: &[u8], en: usize) {
        str_enlarge(s, en as i32);
        let end = s.l + en;
        s.s[s.l..end].copy_from_slice(&st[..en]);
        s.l = end;
    }

    /// Original C static function `sam_write_sq` from `minibwa/format.c:141`.
    pub fn sam_write_sq(s: &mut kstring_t, seq: &[u8], l: i32, rev: i32, comp: i32) {
        let l = l as usize;
        if rev != 0 {
            str_enlarge(s, l as i32);
            let start = s.l;
            for i in 0..l {
                let c = seq[l - 1 - i];
                s.s[start + i] = if c < 128 && comp != 0 {
                    match c {
                        b'A' => b'T',
                        b'C' => b'G',
                        b'G' => b'C',
                        b'T' => b'A',
                        b'a' => b't',
                        b'c' => b'g',
                        b'g' => b'c',
                        b't' => b'a',
                        _ => c,
                    }
                } else {
                    c
                };
            }
            s.l += l;
        } else {
            str_copy(s, seq, l);
        }
    }

    /// Original C static function `get_sam_pri` from `minibwa/format.c:154`.
    pub fn get_sam_pri(n_hit: i32, hit: &[mb_hit_t]) -> Option<&mb_hit_t> {
        for h in hit.iter().take(n_hit as usize) {
            if h.sam_pri() != 0 {
                return Some(h);
            }
        }
        assert_eq!(n_hit, 0);
        None
    }

    /// Original C static function `write_sam_cigar` from `minibwa/format.c:164`.
    pub fn write_sam_cigar(
        s: &mut kstring_t,
        sam_flag: i32,
        in_tag: i32,
        qlen: i32,
        r: &mb_hit_t,
        opt_flag: u64,
    ) {
        let Some(extra) = &r.p else {
            kom_sprintf_lite(s, "*", &[]);
            return;
        };
        let clip_len0 = if r.rev() != 0 { qlen - r.qe } else { r.qs };
        let clip_len1 = if r.rev() != 0 { r.qs } else { qlen - r.qe };
        if in_tag != 0 {
            let clip_char = if ((sam_flag & 0x800) != 0
                || ((sam_flag & 0x100) != 0 && (opt_flag & MB_F_2ND_SEQ) != 0))
                && (opt_flag & MB_F_SUPP_SOFT) == 0
            {
                5
            } else {
                4
            };
            kom_sprintf_lite(s, "\tCG:B:I", &[]);
            if clip_len0 != 0 {
                kom_sprintf_lite(
                    s,
                    ",%u",
                    &[kom_sprintf_arg::u((clip_len0 as u32) << 4 | clip_char)],
                );
            }
            for &c in extra.cigar().iter().take(extra.n_cigar as usize) {
                kom_sprintf_lite(s, ",%u", &[kom_sprintf_arg::u(c)]);
            }
            if clip_len1 != 0 {
                kom_sprintf_lite(
                    s,
                    ",%u",
                    &[kom_sprintf_arg::u((clip_len1 as u32) << 4 | clip_char)],
                );
            }
        } else {
            let clip_char = if ((sam_flag & 0x800) != 0
                || ((sam_flag & 0x100) != 0 && (opt_flag & MB_F_2ND_SEQ) != 0))
                && (opt_flag & MB_F_SUPP_SOFT) == 0
            {
                'H' as i32
            } else {
                'S' as i32
            };
            assert!(clip_len0 < qlen && clip_len1 < qlen);
            if clip_len0 != 0 {
                kom_sprintf_lite(
                    s,
                    "%d%c",
                    &[kom_sprintf_arg::d(clip_len0), kom_sprintf_arg::c(clip_char)],
                );
            }
            for &c in extra.cigar().iter().take(extra.n_cigar as usize) {
                kom_sprintf_lite(
                    s,
                    "%d%c",
                    &[
                        kom_sprintf_arg::d((c >> 4) as i32),
                        kom_sprintf_arg::c(MB_CIGAR_STR[(c & 0xf) as usize] as i32),
                    ],
                );
            }
            if clip_len1 != 0 {
                kom_sprintf_lite(
                    s,
                    "%d%c",
                    &[kom_sprintf_arg::d(clip_len1), kom_sprintf_arg::c(clip_char)],
                );
            }
        }
    }

    /// Original C global function `mb_fmt_sam` from `minibwa/format.c:192`.
    pub fn mb_fmt_sam(
        km: (),
        s: &mut kstring_t,
        l2b: &l2b_t,
        t: &mb_bseq1_t,
        n_seg: i32,
        n_hit: &[i32],
        hit: &[mb_hit_buf_t],
        hit_idx: i32,
        opt_flag: u64,
        seg_idx: i32,
        mate_qlen: i32,
    ) {
        assert!(n_seg == 1 || n_seg == 2);
        let n_h = n_hit[seg_idx as usize];
        let h = &hit[seg_idx as usize];
        let r = if n_h > 0 && hit_idx < n_h && hit_idx >= 0 {
            Some(&h[hit_idx as usize])
        } else {
            None
        };
        let r_next = if n_seg > 1 {
            let next_sid = ((seg_idx + 1) % n_seg) as usize;
            get_sam_pri(n_hit[next_sid], &hit[next_sid])
        } else {
            None
        };
        let r_prev = r_next;

        kom_sprintf_lite(s, "%s", &[kom_sprintf_arg::s(&t.name)]);
        let mut flag = if n_seg > 1 { 0x1 } else { 0x0 };
        if let Some(r) = r {
            if r.rev() != 0 {
                flag |= 0x10;
            }
            if r.parent != r.id {
                flag |= 0x100;
            } else if r.sam_pri() == 0 {
                flag |= 0x800;
            }
        } else {
            flag |= 0x4;
        }
        if n_seg > 1 {
            if r.is_some_and(|x| x.proper_pair() != 0) {
                flag |= 0x2;
            }
            if seg_idx == 0 {
                flag |= 0x40;
            } else if seg_idx == n_seg - 1 {
                flag |= 0x80;
            }
            if let Some(next) = r_next {
                if next.rev() != 0 {
                    flag |= 0x20;
                }
            } else {
                flag |= 0x8;
            }
        }
        kom_sprintf_lite(s, "\t%d", &[kom_sprintf_arg::d(flag)]);

        let mut this_tid = -1i32;
        let mut this_pos = -1i32;
        if let Some(r) = r {
            this_tid = r.tid as i32;
            this_pos = r.ts as i32;
            kom_sprintf_lite(
                s,
                "\t%s\t%d\t%d\t",
                &[
                    kom_sprintf_arg::s(&l2b.ctg[r.tid as usize].name),
                    kom_sprintf_arg::d(r.ts as i32 + 1),
                    kom_sprintf_arg::d(r.mapq),
                ],
            );
            write_sam_cigar(s, flag, 0, t.l_seq as i32, r, opt_flag);
        } else if let Some(prev) = r_prev {
            this_tid = prev.tid as i32;
            this_pos = prev.ts as i32;
            kom_sprintf_lite(
                s,
                "\t%s\t%d\t0\t*",
                &[
                    kom_sprintf_arg::s(&l2b.ctg[this_tid as usize].name),
                    kom_sprintf_arg::d(this_pos + 1),
                ],
            );
        } else {
            kom_sprintf_lite(s, "\t*\t0\t0\t*", &[]);
        }

        if n_seg > 1 {
            let mut tlen = 0i32;
            if this_tid >= 0 {
                if let Some(next) = r_next {
                    if this_tid as i64 == next.tid {
                        if let Some(r) = r {
                            let this_pos5 = if r.rev() != 0 {
                                r.te as i32 - 1
                            } else {
                                this_pos
                            };
                            let next_pos5 = if next.rev() != 0 {
                                next.te as i32 - 1
                            } else {
                                next.ts as i32
                            };
                            tlen = next_pos5 - this_pos5;
                        }
                        kom_sprintf_lite(s, "\t=\t", &[]);
                    } else {
                        kom_sprintf_lite(
                            s,
                            "\t%s\t",
                            &[kom_sprintf_arg::s(&l2b.ctg[next.tid as usize].name)],
                        );
                    }
                    kom_sprintf_lite(s, "%d\t", &[kom_sprintf_arg::d(next.ts as i32 + 1)]);
                } else {
                    kom_sprintf_lite(s, "\t=\t%d\t", &[kom_sprintf_arg::d(this_pos + 1)]);
                }
            } else if let Some(next) = r_next {
                kom_sprintf_lite(
                    s,
                    "\t%s\t%d\t",
                    &[
                        kom_sprintf_arg::s(&l2b.ctg[next.tid as usize].name),
                        kom_sprintf_arg::d(next.ts as i32 + 1),
                    ],
                );
            } else {
                kom_sprintf_lite(s, "\t*\t0\t", &[]);
            }
            if tlen > 0 {
                tlen += 1;
            } else if tlen < 0 {
                tlen -= 1;
            }
            kom_sprintf_lite(s, "%d\t", &[kom_sprintf_arg::d(tlen)]);
        } else {
            kom_sprintf_lite(s, "\t*\t0\t0\t", &[]);
        }

        let seq = t.seq.as_bytes();
        if let Some(r) = r {
            if (flag & 0x900) == 0 || (opt_flag & MB_F_SUPP_SOFT) != 0 {
                sam_write_sq(s, seq, t.l_seq as i32, r.rev() as i32, r.rev() as i32);
                kom_sprintf_lite(s, "\t", &[]);
                if let Some(qual) = &t.qual {
                    sam_write_sq(s, qual.as_bytes(), t.l_seq as i32, r.rev() as i32, 0);
                } else {
                    kom_sprintf_lite(s, "*", &[]);
                }
            } else if (flag & 0x100) != 0 && (opt_flag & MB_F_2ND_SEQ) == 0 {
                kom_sprintf_lite(s, "*\t*", &[]);
            } else {
                sam_write_sq(
                    s,
                    &seq[r.qs as usize..],
                    r.qe - r.qs,
                    r.rev() as i32,
                    r.rev() as i32,
                );
                kom_sprintf_lite(s, "\t", &[]);
                if let Some(qual) = &t.qual {
                    sam_write_sq(
                        s,
                        &qual.as_bytes()[r.qs as usize..],
                        r.qe - r.qs,
                        r.rev() as i32,
                        0,
                    );
                } else {
                    kom_sprintf_lite(s, "*", &[]);
                }
            }
        } else {
            sam_write_sq(s, seq, t.l_seq as i32, 0, 0);
            kom_sprintf_lite(s, "\t", &[]);
            if let Some(qual) = &t.qual {
                sam_write_sq(s, qual.as_bytes(), t.l_seq as i32, 0, 0);
            } else {
                kom_sprintf_lite(s, "*", &[]);
            }
        }

        let rg_id = MB_RG_ID
            .get_or_init(|| Mutex::new(String::new()))
            .lock()
            .unwrap()
            .clone();
        if !rg_id.is_empty() {
            kom_sprintf_lite(s, "\tRG:Z:%s", &[kom_sprintf_arg::s(&rg_id)]);
        }
        if n_seg > 2 {
            kom_sprintf_lite(s, "\tFI:i:%d", &[kom_sprintf_arg::d(seg_idx)]);
        }
        if let Some(r) = r {
            if r.p.is_some() {
                write_tags(s, r);
            }
            if n_seg > 1 {
                if let Some(next) = r_next {
                    if next.p.as_ref().is_some_and(|p| p.n_cigar > 0) && mate_qlen > 0 {
                        kom_sprintf_lite(s, "\tMC:Z:", &[]);
                        write_sam_cigar(s, 0, 0, mate_qlen, next, opt_flag);
                        kom_sprintf_lite(s, "\tMQ:i:%d", &[kom_sprintf_arg::d(next.mapq)]);
                    }
                }
            }
            if let Some(extra) = &r.p {
                if extra.cs() != 0 {
                    let mut bytes = Vec::new();
                    'outer: for &w in extra.cigar_all().iter().skip(extra.n_cigar as usize) {
                        for i in 0..4 {
                            let b = ((w >> (i * 8)) & 0xff) as u8;
                            if b == 0 {
                                break 'outer;
                            }
                            bytes.push(b);
                        }
                    }
                    let tag = String::from_utf8_lossy(&bytes);
                    kom_sprintf_lite(s, "\t%s", &[kom_sprintf_arg::s(&tag)]);
                }
                if r.parent == r.id && n_h > 1 {
                    let self_idx = hit_idx as usize;
                    let mut n_sa = 0;
                    for (i, q) in h.iter().take(n_h as usize).enumerate() {
                        if i != self_idx && q.parent == q.id && q.p.is_some() {
                            n_sa += 1;
                        }
                    }
                    if n_sa > 0 {
                        kom_sprintf_lite(s, "\tSA:Z:", &[]);
                        for (i, q) in h.iter().take(n_h as usize).enumerate() {
                            if i == self_idx || q.parent != q.id || q.p.is_none() {
                                continue;
                            }
                            let mut l_i = 0;
                            let mut l_d = 0;
                            let l_m = if q.qe - q.qs < (q.te - q.ts) as i32 {
                                l_d = (q.te - q.ts) as i32 - (q.qe - q.qs);
                                q.qe - q.qs
                            } else {
                                l_i = (q.qe - q.qs) - (q.te - q.ts) as i32;
                                (q.te - q.ts) as i32
                            };
                            let clip5 = if q.rev() != 0 {
                                t.l_seq as i32 - q.qe
                            } else {
                                q.qs
                            };
                            let clip3 = if q.rev() != 0 {
                                q.qs
                            } else {
                                t.l_seq as i32 - q.qe
                            };
                            kom_sprintf_lite(
                                s,
                                "%s,%d,%c,",
                                &[
                                    kom_sprintf_arg::s(&l2b.ctg[q.tid as usize].name),
                                    kom_sprintf_arg::d(q.ts as i32 + 1),
                                    kom_sprintf_arg::c(if q.rev() != 0 {
                                        '-' as i32
                                    } else {
                                        '+' as i32
                                    }),
                                ],
                            );
                            if clip5 != 0 {
                                kom_sprintf_lite(s, "%dS", &[kom_sprintf_arg::d(clip5)]);
                            }
                            if l_m != 0 {
                                kom_sprintf_lite(s, "%dM", &[kom_sprintf_arg::d(l_m)]);
                            }
                            if l_i != 0 {
                                kom_sprintf_lite(s, "%dI", &[kom_sprintf_arg::d(l_i)]);
                            }
                            if l_d != 0 {
                                kom_sprintf_lite(s, "%dD", &[kom_sprintf_arg::d(l_d)]);
                            }
                            if clip3 != 0 {
                                kom_sprintf_lite(s, "%dS", &[kom_sprintf_arg::d(clip3)]);
                            }
                            let qextra = q.p.as_ref().unwrap();
                            kom_sprintf_lite(
                                s,
                                ",%d,%d;",
                                &[
                                    kom_sprintf_arg::d(q.mapq),
                                    kom_sprintf_arg::d(q.blen - q.mlen + qextra.n_ambi() as i32),
                                ],
                            );
                        }
                    }
                }
            }
        }

        if (opt_flag & MB_F_COPY_COMMENT) != 0 {
            if let Some(comment) = &t.comment {
                kom_sprintf_lite(s, "\t%s", &[kom_sprintf_arg::s(comment)]);
            }
        }
        kom_sprintf_lite(s, "\n", &[]);
        if s.s.len() <= s.l {
            s.s.resize(s.l + 1, 0);
            s.m = s.s.len();
        }
        s.s[s.l] = 0;
    }

    /// Original C global function `mb_format` from `minibwa/format.c:322`.
    pub fn mb_format(
        km: (),
        s: &mut kstring_t,
        l2b: &l2b_t,
        t: &mb_bseq1_t,
        n_seg: i32,
        n_hit: &[i32],
        hit: &[mb_hit_buf_t],
        hit_idx: i32,
        opt_flag: u64,
        seg_idx: i32,
        mate_qlen: i32,
    ) {
        if (opt_flag & MB_F_SAM) != 0 {
            mb_fmt_sam(
                km, s, l2b, t, n_seg, n_hit, hit, hit_idx, opt_flag, seg_idx, mate_qlen,
            );
        } else {
            let p = if hit_idx >= 0 {
                Some(&hit[seg_idx as usize][hit_idx as usize])
            } else {
                None
            };
            mb_fmt_paf(s, l2b, t, p, opt_flag, n_seg, seg_idx);
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::align::MB_CIGAR_MATCH;
        use crate::l2bit::l2b_ctg_t;
        use crate::pe::mb_extra_t;

        #[test]
        fn paf_formats_unmapped_and_mapped_records() {
            let l2b = l2b_t {
                n_ctg: 1,
                ctg: vec![l2b_ctg_t {
                    name: "chrM".to_string(),
                    len: 16569,
                    ..Default::default()
                }],
                ..Default::default()
            };
            let read = mb_bseq1_t {
                l_seq: 8,
                name: "read1".into(),
                seq: "ACGTACGT".into(),
                comment: Some("rl:i:8".into()),
                ..Default::default()
            };
            let mut out = kstring_t::default();
            mb_fmt_paf(&mut out, &l2b, &read, None, MB_F_COPY_COMMENT, 1, 0);
            assert_eq!(
                String::from_utf8_lossy(&out.s[..out.l]),
                "read1\t8\t*\t*\t*\t*\t*\t*\t*\t0\t0\t0\n"
            );

            let mut tag_words = vec![8 << 4 | MB_CIGAR_MATCH];
            for chunk in b"cs:Z::8\0".chunks(4) {
                let mut w = 0u32;
                for (i, &b) in chunk.iter().enumerate() {
                    w |= (b as u32) << (i * 8);
                }
                tag_words.push(w);
            }
            let hit = mb_hit_t {
                tid: 0,
                ts: 3,
                te: 11,
                id: 7,
                parent: 7,
                qs: 0,
                qe: 8,
                mlen: 8,
                blen: 8,
                mapq: 60,
                score: 24,
                subsc: 10,
                cnt: 2,
                p: Some(
                    mb_extra_t {
                        dp_score: 24,
                        dp_max0: 24,
                        dp_max: 24,
                        dp_max2: 3,
                        n_ambi_cs: mb_extra_t::CS_FLAG,
                        ..Default::default()
                    }
                    .with_cigar(&tag_words),
                ),
                ..Default::default()
            };
            out.l = 0;
            mb_fmt_paf(&mut out, &l2b, &read, Some(&hit), MB_F_COPY_COMMENT, 1, 0);
            let got = String::from_utf8_lossy(&out.s[..out.l]);
            assert!(got.contains(
                "read1\t8\t0\t8\t+\tchrM\t16569\t3\t11\t8\t8\t60\ttp:A:P\ts1:i:24\tcm:i:2"
            ));
            assert!(got.contains("\tNM:i:0\tAS:i:24\tms:i:24\tmd:i:21\tcg:Z:8M\tcs:Z::8\trl:i:8\n"));
        }

        #[test]
        fn sam_header_escapes_rg_and_records_id() {
            let l2b = l2b_t {
                n_ctg: 1,
                ctg: vec![l2b_ctg_t {
                    name: "chr1".to_string(),
                    len: 100,
                    ..Default::default()
                }],
                ..Default::default()
            };
            let mut out = kstring_t::default();
            let ret = mb_fmt_sam_hdr(
                &mut out,
                Some(&l2b),
                Some("@RG\\tID:grp1\\tSM:sample"),
                Some("0.0-test"),
                &["minibwa", "mem", "ref.fa", "reads.fq"],
            );
            let got = String::from_utf8_lossy(&out.s[..out.l]);
            assert_eq!(ret, 0);
            assert!(got.contains("@SQ\tSN:chr1\tLN:100\n"));
            assert!(got.contains("@RG\tID:grp1\tSM:sample\n"));
            assert!(got.contains(
                "@PG\tID:minibwa\tPN:minibwa\tVN:0.0-test\tCL:minibwa mem ref.fa reads.fq\n"
            ));
        }

        #[test]
        fn sam_formats_cigar_sequence_quality_and_rg() {
            let l2b = l2b_t {
                n_ctg: 1,
                ctg: vec![l2b_ctg_t {
                    name: "chr1".to_string(),
                    len: 100,
                    ..Default::default()
                }],
                ..Default::default()
            };
            let read = mb_bseq1_t {
                l_seq: 6,
                name: "q1".into(),
                seq: "ACGTAA".into(),
                qual: Some("abcdef".into()),
                ..Default::default()
            };
            let hit = mb_hit_t {
                tid: 0,
                ts: 9,
                te: 13,
                id: 1,
                parent: 1,
                qs: 1,
                qe: 5,
                flags: mb_hit_t::flags_with(1, 0, 1, 0, 0, 0, 0, 0, 0),
                mlen: 4,
                blen: 4,
                mapq: 42,
                p: Some(
                    mb_extra_t {
                        dp_score: 12,
                        dp_max0: 12,
                        dp_max: 12,
                        dp_max2: 0,
                        ..Default::default()
                    }
                    .with_cigar(&[4 << 4 | MB_CIGAR_MATCH]),
                ),
                ..Default::default()
            };
            let mut out = kstring_t::default();
            sam_write_rg_line(&mut out, Some("@RG\\tID:grp2"));
            out.l = 0;
            mb_format(
                (),
                &mut out,
                &l2b,
                &read,
                1,
                &[1],
                &[mb_hit_buf_t::from_vec(vec![hit])],
                0,
                MB_F_SAM,
                0,
                0,
            );
            let got = String::from_utf8_lossy(&out.s[..out.l]);
            assert!(got.starts_with("q1\t16\tchr1\t10\t42\t1S4M1S\t*\t0\t0\tTTACGT\tfedcba"));
            assert!(got.contains("\tNM:i:0\tAS:i:12\tms:i:12\tmd:i:12\n"));
        }
    }
}

pub mod index {
    #![allow(unused_variables, dead_code, non_snake_case)]

    use crate::bwt::{
        mb_bwt_destroy, mb_bwt_gen_sa, mb_bwt_init_from_raw, mb_bwt_load, mb_bwt_load_raw,
        mb_bwt_save, mb_bwt_t,
    };
    use crate::bwtgen::mb_bwtgen;
    use crate::ketopt::{ketopt, ko_longopt_t, KETOPT_INIT};
    use crate::kommon::{kom_panic, kom_parse_num};
    use crate::l2bit::{
        l2b_get0, l2b_import, l2b_load, l2b_save, l2b_save_pac, l2b_save_pac_meth, l2b_t,
    };
    use rayon::prelude::*;

    fn run_with_index_pool<R: Send>(n_thread: i32, f: impl FnOnce() -> R + Send) -> R {
        if n_thread <= 1 || rayon::current_thread_index().is_some() {
            return f();
        }
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(n_thread as usize)
            .build()
            .expect("failed to build Rayon thread pool");
        pool.install(f)
    }

    fn use_parallel_index_post(n_thread: i32, len: usize) -> bool {
        n_thread > 1 && len >= (1 << 20) && rayon::current_thread_index().is_some()
    }

    /// Original C static function `l2b_c2t` from `minibwa/index.c:17`.
    pub fn l2b_c2t(b: u8) -> u8 {
        if b == 1 {
            3
        } else {
            b
        }
    }

    /// Original C static function `l2b_g2a` from `minibwa/index.c:18`.
    pub fn l2b_g2a(b: u8) -> u8 {
        if b == 2 {
            0
        } else {
            b
        }
    }

    #[cfg(target_arch = "x86")]
    #[inline]
    unsafe fn mb_unpack_2bit_16bytes_sse2(src: *const u8, dst: *mut u8) {
        use std::arch::x86::{
            __m128i, _mm_and_si128, _mm_loadu_si128, _mm_set1_epi8, _mm_srli_epi16,
            _mm_storeu_si128, _mm_unpackhi_epi16, _mm_unpackhi_epi8, _mm_unpacklo_epi16,
            _mm_unpacklo_epi8,
        };

        let x = unsafe { _mm_loadu_si128(src as *const __m128i) };
        let mask = unsafe { _mm_set1_epi8(3) };
        let b0 = unsafe { _mm_and_si128(x, mask) };
        let b1 = unsafe { _mm_and_si128(_mm_srli_epi16::<2>(x), mask) };
        let b2 = unsafe { _mm_and_si128(_mm_srli_epi16::<4>(x), mask) };
        let b3 = unsafe { _mm_and_si128(_mm_srli_epi16::<6>(x), mask) };

        let lo01 = unsafe { _mm_unpacklo_epi8(b0, b1) };
        let lo23 = unsafe { _mm_unpacklo_epi8(b2, b3) };
        let hi01 = unsafe { _mm_unpackhi_epi8(b0, b1) };
        let hi23 = unsafe { _mm_unpackhi_epi8(b2, b3) };

        unsafe { _mm_storeu_si128(dst as *mut __m128i, _mm_unpacklo_epi16(lo01, lo23)) };
        unsafe { _mm_storeu_si128(dst.add(16) as *mut __m128i, _mm_unpackhi_epi16(lo01, lo23)) };
        unsafe { _mm_storeu_si128(dst.add(32) as *mut __m128i, _mm_unpacklo_epi16(hi01, hi23)) };
        unsafe { _mm_storeu_si128(dst.add(48) as *mut __m128i, _mm_unpackhi_epi16(hi01, hi23)) };
    }

    #[cfg(target_arch = "x86_64")]
    #[inline]
    unsafe fn mb_unpack_2bit_16bytes_sse2(src: *const u8, dst: *mut u8) {
        use std::arch::x86_64::{
            __m128i, _mm_and_si128, _mm_loadu_si128, _mm_set1_epi8, _mm_srli_epi16,
            _mm_storeu_si128, _mm_unpackhi_epi16, _mm_unpackhi_epi8, _mm_unpacklo_epi16,
            _mm_unpacklo_epi8,
        };

        let x = unsafe { _mm_loadu_si128(src as *const __m128i) };
        let mask = unsafe { _mm_set1_epi8(3) };
        let b0 = unsafe { _mm_and_si128(x, mask) };
        let b1 = unsafe { _mm_and_si128(_mm_srli_epi16::<2>(x), mask) };
        let b2 = unsafe { _mm_and_si128(_mm_srli_epi16::<4>(x), mask) };
        let b3 = unsafe { _mm_and_si128(_mm_srli_epi16::<6>(x), mask) };

        let lo01 = unsafe { _mm_unpacklo_epi8(b0, b1) };
        let lo23 = unsafe { _mm_unpacklo_epi8(b2, b3) };
        let hi01 = unsafe { _mm_unpackhi_epi8(b0, b1) };
        let hi23 = unsafe { _mm_unpackhi_epi8(b2, b3) };

        unsafe { _mm_storeu_si128(dst as *mut __m128i, _mm_unpacklo_epi16(lo01, lo23)) };
        unsafe { _mm_storeu_si128(dst.add(16) as *mut __m128i, _mm_unpackhi_epi16(lo01, lo23)) };
        unsafe { _mm_storeu_si128(dst.add(32) as *mut __m128i, _mm_unpacklo_epi16(hi01, hi23)) };
        unsafe { _mm_storeu_si128(dst.add(48) as *mut __m128i, _mm_unpackhi_epi16(hi01, hi23)) };
    }

    #[inline]
    fn mb_l2b_fill_forward_2bit(seq: &mut [u8], l2b: &l2b_t) -> usize {
        let len = l2b.tot_len as usize;
        let mut i = 0usize;
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            // SIMD note: SSE2 expands 16 packed input bytes into 64 two-bit bases.
            let simd_len = len & !63usize;
            let src = l2b.pac.as_ptr() as *const u8;
            while i < simd_len {
                unsafe { mb_unpack_2bit_16bytes_sse2(src.add(i >> 2), seq.as_mut_ptr().add(i)) };
                i += 64;
            }
        }
        while i < len {
            seq[i] = l2b_get0(l2b, i as u64) as u8;
            i += 1;
        }
        len
    }

    #[inline]
    fn mb_l2b_fill_reverse_complement_2bit(seq: &mut [u8], l2b: &l2b_t) -> usize {
        let len = l2b.tot_len as usize;
        let mut src_end = len;
        let mut dst = 0usize;
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            while src_end & 63 != 0 {
                src_end -= 1;
                seq[dst] = 3 - l2b_get0(l2b, src_end as u64) as u8;
                dst += 1;
            }
            let src = l2b.pac.as_ptr() as *const u8;
            let mut tmp = [0u8; 64];
            while src_end >= 64 {
                src_end -= 64;
                unsafe { mb_unpack_2bit_16bytes_sse2(src.add(src_end >> 2), tmp.as_mut_ptr()) };
                for k in 0..64 {
                    seq[dst + k] = 3 - tmp[63 - k];
                }
                dst += 64;
            }
        }
        while src_end > 0 {
            src_end -= 1;
            seq[dst] = 3 - l2b_get0(l2b, src_end as u64) as u8;
            dst += 1;
        }
        len
    }

    #[inline]
    fn mb_l2b_fill_forward_meth_2bit(seq: &mut [u8], l2b: &l2b_t, is_g2a: bool) -> usize {
        let len = l2b.tot_len as usize;
        let mut i = 0usize;
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            let simd_len = len & !63usize;
            let src = l2b.pac.as_ptr() as *const u8;
            while i < simd_len {
                unsafe { mb_unpack_2bit_16bytes_sse2(src.add(i >> 2), seq.as_mut_ptr().add(i)) };
                for b in &mut seq[i..i + 64] {
                    if is_g2a {
                        *b = l2b_g2a(*b);
                    } else {
                        *b = l2b_c2t(*b);
                    }
                }
                i += 64;
            }
        }
        while i < len {
            let b = l2b_get0(l2b, i as u64) as u8;
            seq[i] = if is_g2a { l2b_g2a(b) } else { l2b_c2t(b) };
            i += 1;
        }
        len
    }

    #[inline]
    fn mb_l2b_fill_reverse_meth_2bit(seq: &mut [u8], l2b: &l2b_t, is_g2a: bool) -> usize {
        let len = l2b.tot_len as usize;
        let mut src_end = len;
        let mut dst = 0usize;
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            while src_end & 63 != 0 {
                src_end -= 1;
                let b = l2b_get0(l2b, src_end as u64) as u8;
                seq[dst] = 3 - if is_g2a { l2b_g2a(b) } else { l2b_c2t(b) };
                dst += 1;
            }
            let src = l2b.pac.as_ptr() as *const u8;
            let mut tmp = [0u8; 64];
            while src_end >= 64 {
                src_end -= 64;
                unsafe { mb_unpack_2bit_16bytes_sse2(src.add(src_end >> 2), tmp.as_mut_ptr()) };
                for k in 0..64 {
                    let b = tmp[63 - k];
                    seq[dst + k] = 3 - if is_g2a { l2b_g2a(b) } else { l2b_c2t(b) };
                }
                dst += 64;
            }
        }
        while src_end > 0 {
            src_end -= 1;
            let b = l2b_get0(l2b, src_end as u64) as u8;
            seq[dst] = 3 - if is_g2a { l2b_g2a(b) } else { l2b_c2t(b) };
            dst += 1;
        }
        len
    }

    unsafe fn sample_sa_i32(
        a: *const i32,
        len: usize,
        sa_shift: usize,
        step: usize,
        ssa: &mut [u64],
    ) {
        let ssa_ptr = ssa.as_mut_ptr();
        let mut i = 0usize;
        while i <= len {
            *ssa_ptr.add(i >> sa_shift) = *a.add(i) as u64;
            i += step;
        }
    }

    unsafe fn sample_sa_i64(
        a: *const i64,
        len: usize,
        sa_shift: usize,
        step: usize,
        ssa: &mut [u64],
    ) {
        let ssa_ptr = ssa.as_mut_ptr();
        let mut i = 0usize;
        while i <= len {
            *ssa_ptr.add(i >> sa_shift) = *a.add(i) as u64;
            i += step;
        }
    }

    unsafe fn sa_to_bwt_i32(a: *mut i32, seq: &mut [u8], len: usize) -> u64 {
        let seq_read = seq.as_ptr();
        let mut primary = usize::MAX;
        for i in 0..=len {
            let ai = *a.add(i);
            if ai == 0 {
                primary = i;
            } else {
                *a.add(i) = *seq_read.add(ai as usize - 1) as i32;
            }
        }
        assert_ne!(primary, usize::MAX);

        let seq_write = seq.as_mut_ptr();
        for i in 0..primary {
            *seq_write.add(i) = *a.add(i) as u8;
        }
        for i in primary..len {
            *seq_write.add(i) = *a.add(i + 1) as u8;
        }
        primary as u64
    }

    fn sa_to_bwt_i32_parallel(a: &mut [i32], seq: &mut [u8], len: usize) -> u64 {
        const CHUNK: usize = 1 << 18;
        let seq_read = &seq[..];
        let primary = a[..=len]
            .par_chunks_mut(CHUNK)
            .enumerate()
            .filter_map(|(chunk_idx, chunk)| {
                let base = chunk_idx * CHUNK;
                let mut primary = None;
                for (offset, ai) in chunk.iter_mut().enumerate() {
                    if *ai == 0 {
                        primary = Some(base + offset);
                    } else {
                        *ai = seq_read[*ai as usize - 1] as i32;
                    }
                }
                primary
            })
            .reduce_with(|a, b| a.min(b))
            .expect("suffix array primary index not found");

        let a_read = &a[..=len];
        let (left, right) = seq.split_at_mut(primary);
        left.par_iter_mut()
            .enumerate()
            .for_each(|(i, dst)| *dst = a_read[i] as u8);
        right
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, dst)| *dst = a_read[primary + i + 1] as u8);
        primary as u64
    }

    unsafe fn sa_to_bwt_i64(a: *mut i64, seq: &mut [u8], len: usize) -> u64 {
        let seq_read = seq.as_ptr();
        let mut primary = usize::MAX;
        for i in 0..=len {
            let ai = *a.add(i);
            if ai == 0 {
                primary = i;
            } else {
                *a.add(i) = *seq_read.add(ai as usize - 1) as i64;
            }
        }
        assert_ne!(primary, usize::MAX);

        let seq_write = seq.as_mut_ptr();
        for i in 0..primary {
            *seq_write.add(i) = *a.add(i) as u8;
        }
        for i in primary..len {
            *seq_write.add(i) = *a.add(i + 1) as u8;
        }
        primary as u64
    }

    fn sa_to_bwt_i64_parallel(a: &mut [i64], seq: &mut [u8], len: usize) -> u64 {
        const CHUNK: usize = 1 << 18;
        let seq_read = &seq[..];
        let primary = a[..=len]
            .par_chunks_mut(CHUNK)
            .enumerate()
            .filter_map(|(chunk_idx, chunk)| {
                let base = chunk_idx * CHUNK;
                let mut primary = None;
                for (offset, ai) in chunk.iter_mut().enumerate() {
                    if *ai == 0 {
                        primary = Some(base + offset);
                    } else {
                        *ai = seq_read[*ai as usize - 1] as i64;
                    }
                }
                primary
            })
            .reduce_with(|a, b| a.min(b))
            .expect("suffix array primary index not found");

        let a_read = &a[..=len];
        let (left, right) = seq.split_at_mut(primary);
        left.par_iter_mut()
            .enumerate()
            .for_each(|(i, dst)| *dst = a_read[i] as u8);
        right
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, dst)| *dst = a_read[primary + i + 1] as u8);
        primary as u64
    }

    /// Original C global function `mb_bwt_libsais` from `minibwa/index.c:19`.
    pub fn mb_bwt_libsais(
        l2b: &l2b_t,
        sa_bit: i32,
        both_strand: i32,
        is_meth: i32,
        n_thread: i32,
    ) -> mb_bwt_t {
        let time_stages = std::env::var_os("MINIBWA_RS_INDEX_TIMING").is_some();
        let mut stage_start = std::time::Instant::now();
        let copies = (if is_meth != 0 { 2 } else { 1 }) * (if both_strand != 0 { 2 } else { 1 });
        let len = l2b.tot_len as usize * copies as usize;
        let mut seq = Vec::<u8>::with_capacity(len);
        if is_meth != 0 {
            let copy_len = l2b.tot_len as usize;
            seq.resize(copy_len, 0);
            mb_l2b_fill_forward_meth_2bit(&mut seq, l2b, false);
            let old_len = seq.len();
            seq.resize(old_len + copy_len, 0);
            mb_l2b_fill_forward_meth_2bit(&mut seq[old_len..], l2b, true);
            if both_strand != 0 {
                let old_len = seq.len();
                seq.resize(old_len + copy_len, 0);
                mb_l2b_fill_reverse_meth_2bit(&mut seq[old_len..], l2b, true);
                let old_len = seq.len();
                seq.resize(old_len + copy_len, 0);
                mb_l2b_fill_reverse_meth_2bit(&mut seq[old_len..], l2b, false);
            }
        } else {
            seq.resize(l2b.tot_len as usize, 0);
            mb_l2b_fill_forward_2bit(&mut seq, l2b);
            if both_strand != 0 {
                let old_len = seq.len();
                seq.resize(old_len + l2b.tot_len as usize, 0);
                mb_l2b_fill_reverse_complement_2bit(&mut seq[old_len..], l2b);
            }
        }
        assert_eq!(seq.len(), len);
        if time_stages {
            eprintln!(
                "[index::mb_bwt_libsais] fill_seq {:.3}s",
                stage_start.elapsed().as_secs_f64()
            );
            stage_start = std::time::Instant::now();
        }
        let sa_shift = sa_bit as usize;
        let step = 1u64 << sa_shift;
        let step_usize = step as usize;
        let n_ssa = ((len as u64) + step) >> sa_shift;
        const FS: usize = 10000;
        let use_32bit_sa = std::env::var_os("MINIBWA_RS_INDEX_32BIT_SA").is_some()
            && len <= i32::MAX as usize - FS;
        let (primary, mut ssa) = if use_32bit_sa {
            let mut a = vec![0i32; len + FS + 1];
            let rc =
                libsais_rs::libsais_upstream_c_omp(&seq, &mut a[1..], FS as i32, None, n_thread);
            assert_eq!(rc, 0, "libsais failed with status {rc}");
            if time_stages {
                eprintln!(
                    "[index::mb_bwt_libsais] libsais32 {:.3}s",
                    stage_start.elapsed().as_secs_f64()
                );
                stage_start = std::time::Instant::now();
            }
            a[0] = len as i32;
            let mut ssa = vec![0u64; n_ssa as usize];
            let primary = unsafe {
                let a_ptr = a.as_mut_ptr();
                sample_sa_i32(a_ptr, len, sa_shift, step_usize, &mut ssa);
                if use_parallel_index_post(n_thread, len) {
                    sa_to_bwt_i32_parallel(&mut a[..=len], &mut seq, len)
                } else {
                    sa_to_bwt_i32(a_ptr, &mut seq, len)
                }
            };
            if time_stages {
                eprintln!(
                    "[index::mb_bwt_libsais] post32 {:.3}s",
                    stage_start.elapsed().as_secs_f64()
                );
                stage_start = std::time::Instant::now();
            }
            drop(a);
            (primary, ssa)
        } else {
            let mut a_storage = Vec::<std::mem::MaybeUninit<i64>>::with_capacity(len + FS + 1);
            unsafe {
                a_storage.set_len(len + FS + 1);
            }
            let rc = libsais_rs::libsais64::libsais64_upstream_c_omp_uninit(
                &seq,
                &mut a_storage[1..],
                FS as i64,
                None,
                n_thread as i64,
            );
            assert_eq!(rc, 0, "libsais failed with status {rc}");
            if time_stages {
                eprintln!(
                    "[index::mb_bwt_libsais] libsais64 {:.3}s",
                    stage_start.elapsed().as_secs_f64()
                );
                stage_start = std::time::Instant::now();
            }
            let a_ptr = a_storage.as_mut_ptr().cast::<i64>();
            unsafe {
                *a_ptr = len as i64;
            }
            let a = unsafe { std::slice::from_raw_parts_mut(a_ptr, len + 1) };
            let mut ssa = vec![0u64; n_ssa as usize];
            let primary = unsafe {
                let a_ptr = a.as_mut_ptr();
                sample_sa_i64(a_ptr, len, sa_shift, step_usize, &mut ssa);
                if use_parallel_index_post(n_thread, len) {
                    sa_to_bwt_i64_parallel(a, &mut seq, len)
                } else {
                    sa_to_bwt_i64(a_ptr, &mut seq, len)
                }
            };
            if time_stages {
                eprintln!(
                    "[index::mb_bwt_libsais] post64 {:.3}s",
                    stage_start.elapsed().as_secs_f64()
                );
                stage_start = std::time::Instant::now();
            }
            (primary, ssa)
        };
        if !ssa.is_empty() {
            ssa[0] = u64::MAX;
        }
        let mut bwt = mb_bwt_init_from_raw(1, &seq, len as u64, primary);
        if time_stages {
            eprintln!(
                "[index::mb_bwt_libsais] init_bwt {:.3}s",
                stage_start.elapsed().as_secs_f64()
            );
        }
        drop(seq);
        bwt.sa_bit = sa_bit as u32;
        bwt.n_sa = n_ssa;
        bwt.sa = ssa;
        bwt
    }

    /// Original C static function `usage_fa2bit` from `minibwa/index.c:47`.
    pub fn usage_fa2bit(to_stdout: bool, seed: u64) -> (i32, String) {
        (
            if to_stdout { 0 } else { 1 },
            format!(
                "Usage: minibwa fa2bit [options] <in.fa> <out.l2b>\nOptions:\n  -s INT    random seed [{seed}]\n  -p        output the BWA pac format\n  -2        output both strands (effective with -p)\n  --help    print this help message\n"
            ),
        )
    }

    /// Original C global function `main_fa2bit` from `minibwa/index.c:56`.
    pub fn main_fa2bit(argv: &[String]) -> (i32, String) {
        let long_opts = [
            ko_longopt_t {
                name: Some("help".into()),
                has_arg: 0,
                val: 901,
            },
            ko_longopt_t {
                name: Some("meth".into()),
                has_arg: 0,
                val: 902,
            },
        ];
        let argc = argv.len() as i32;
        let mut args = argv.to_vec();
        let mut o = KETOPT_INIT.clone();
        let mut out_pac = 0;
        let mut both_strand = 0;
        let mut seed = 11u64;
        loop {
            let c = ketopt(&mut o, argc, &mut args, 1, "s:p2", Some(&long_opts));
            if c < 0 {
                break;
            }
            if c == 's' as i32 {
                seed = o.arg.as_deref().unwrap_or("0").parse().unwrap_or(0);
            } else if c == 'p' as i32 {
                out_pac = 1;
            } else if c == '2' as i32 {
                both_strand = 1;
            } else if c == 901 {
                return usage_fa2bit(true, seed);
            }
        }
        if argc - o.ind < 2 {
            return usage_fa2bit(false, seed);
        }
        let Some(l2b) = l2b_import(&args[o.ind as usize], seed) else {
            return (1, String::new());
        };
        if out_pac != 0 {
            l2b_save_pac(&args[o.ind as usize + 1], &l2b, both_strand);
        } else {
            l2b_save(&args[o.ind as usize + 1], &l2b);
        }
        (0, String::new())
    }

    /// Original C static function `usage_genraw` from `minibwa/index.c:75`.
    pub fn usage_genraw(to_stdout: bool) -> (i32, String) {
        (
            if to_stdout { 0 } else { 1 },
            "Usage: minibwa genraw [options] <in.pac> <out.raw-bwt>\nOptions:\n  -b NUM      block size [10m]\n  --help      print this help message\n"
                .into(),
        )
    }

    /// Original C global function `main_genraw` from `minibwa/index.c:85`.
    pub fn main_genraw(argv: &[String]) -> (i32, String) {
        let long_opts = [ko_longopt_t {
            name: Some("help".into()),
            has_arg: 0,
            val: 901,
        }];
        let argc = argv.len() as i32;
        let mut args = argv.to_vec();
        let mut o = KETOPT_INIT.clone();
        let mut block_size = 10_000_000i32;
        loop {
            let c = ketopt(&mut o, argc, &mut args, 1, "b:", Some(&long_opts));
            if c < 0 {
                break;
            }
            if c == 'b' as i32 {
                block_size = kom_parse_num(o.arg.as_deref().unwrap_or("0")).0 as i32;
            } else if c == 901 {
                return usage_genraw(true);
            }
        }
        if argc - o.ind < 2 {
            return usage_genraw(false);
        }
        if mb_bwtgen(
            std::path::Path::new(&args[o.ind as usize]),
            std::path::Path::new(&args[o.ind as usize + 1]),
            block_size,
        )
        .is_err()
        {
            return (1, String::new());
        }
        (0, String::new())
    }

    /// Original C static function `usage_raw2bwt` from `minibwa/index.c:98`.
    pub fn usage_raw2bwt(to_stdout: bool) -> (i32, String) {
        (
            if to_stdout { 0 } else { 1 },
            "Usage: minibwa raw2bwt <raw.bwt> <recode.bwt>\nOptions:\n  --help    print this help message\n"
                .into(),
        )
    }

    /// Original C global function `main_raw2bwt` from `minibwa/index.c:107`.
    pub fn main_raw2bwt(argv: &[String]) -> (i32, String) {
        if argv.iter().skip(1).any(|s| s == "--help") {
            return usage_raw2bwt(true);
        }
        if argv.len() < 3 {
            return usage_raw2bwt(false);
        }
        let Some(bwt) = mb_bwt_load_raw(&argv[1]) else {
            return (1, String::new());
        };
        mb_bwt_save(&argv[2], &bwt);
        mb_bwt_destroy(Some(bwt));
        (0, String::new())
    }

    /// Original C static function `usage_genbwt` from `minibwa/index.c:120`.
    pub fn usage_genbwt(to_stdout: bool, sa_bit: i32, n_thread: i32) -> (i32, String) {
        (
            if to_stdout { 0 } else { 1 },
            format!(
                "Usage: minibwa genbwt [options] <in.l2b> <out.bwt>\nOptions:\n  -u INT      SA sample rate at 1/(1<<INT) [{sa_bit}]\n  -1          forward strand only\n  -t INT      number of threads [{n_thread}]\n  --help      print this help message\n"
            ),
        )
    }

    /// Original C global function `main_genbwt` from `minibwa/index.c:135`.
    pub fn main_genbwt(argv: &[String]) -> (i32, String) {
        let long_opts = [ko_longopt_t {
            name: Some("help".into()),
            has_arg: 0,
            val: 901,
        }];
        let argc = argv.len() as i32;
        let mut args = argv.to_vec();
        let mut o = KETOPT_INIT.clone();
        let mut n_thread = 4;
        let mut both_strand = 1;
        let mut sa_bit = 4;
        loop {
            let c = ketopt(&mut o, argc, &mut args, 1, "1u:t:", Some(&long_opts));
            if c < 0 {
                break;
            }
            if c == 't' as i32 {
                n_thread = o.arg.as_deref().unwrap_or("0").parse().unwrap_or(0);
            } else if c == '1' as i32 {
                both_strand = 0;
            } else if c == 'u' as i32 {
                sa_bit = o.arg.as_deref().unwrap_or("0").parse().unwrap_or(0);
            } else if c == 901 {
                return usage_genbwt(true, sa_bit, n_thread);
            }
        }
        if argc - o.ind < 2 {
            return usage_genbwt(false, sa_bit, n_thread);
        }
        let Some(l2b) = l2b_load(&args[o.ind as usize]) else {
            kom_panic("main_genbwt", "failed to open the input file.");
        };
        let bwt = run_with_index_pool(n_thread, || {
            mb_bwt_libsais(&l2b, sa_bit, both_strand, 0, n_thread)
        });
        mb_bwt_save(&args[o.ind as usize + 1], &bwt);
        (0, String::new())
    }

    /// Original C static function `usage_gensa` from `minibwa/index.c:150`.
    pub fn usage_gensa(to_stdout: bool, sa_bit: i32) -> (i32, String) {
        (
            if to_stdout { 0 } else { 1 },
            format!(
                "Usage: minibwa gensa [options] <in.bwt> <out.bwt>\nOptions:\n  -u INT    sample rate at 1/(1<<INT) [{sa_bit}]\n  -r        input BWT in the raw BWA format\n  --help    print this help message\n"
            ),
        )
    }

    /// Original C global function `main_gensa` from `minibwa/index.c:158`.
    pub fn main_gensa(argv: &[String]) -> (i32, String) {
        let long_opts = [ko_longopt_t {
            name: Some("help".into()),
            has_arg: 0,
            val: 901,
        }];
        let argc = argv.len() as i32;
        let mut args = argv.to_vec();
        let mut o = KETOPT_INIT.clone();
        let mut sa_bit = 4;
        let mut is_raw = 0;
        loop {
            let c = ketopt(&mut o, argc, &mut args, 1, "ru:", Some(&long_opts));
            if c < 0 {
                break;
            }
            if c == 'u' as i32 {
                sa_bit = o.arg.as_deref().unwrap_or("0").parse().unwrap_or(0);
            } else if c == 'r' as i32 {
                is_raw = 1;
            } else if c == 901 {
                return usage_gensa(true, sa_bit);
            }
        }
        if argc - o.ind < 2 {
            return usage_gensa(false, sa_bit);
        }
        let Some(mut bwt) = (if is_raw != 0 {
            mb_bwt_load_raw(&args[o.ind as usize])
        } else {
            mb_bwt_load(&args[o.ind as usize])
        }) else {
            return (1, String::new());
        };
        mb_bwt_gen_sa(&mut bwt, sa_bit as u32);
        mb_bwt_save(&args[o.ind as usize + 1], &bwt);
        (0, String::new())
    }

    /// Original C static function `usage_index` from `minibwa/index.c:173`.
    pub fn usage_index(to_stdout: bool, seed: u64, sa_bit: i32, n_thread: i32) -> (i32, String) {
        (
            if to_stdout { 0 } else { 1 },
            format!(
                "Usage: minibwa index [options] <in.fasta> [out.prefix]\nOptions:\n  -s INT    random seed for amibiguous bases [{seed}]\n  -u INT    SA sample rate at 1/(1<<INT) [{sa_bit}]\n  -l        low-memory GPL'd algorithm for BWT construction\n  -b NUM    block size (effective with -l) [10m]\n  -t INT    number of threads (effective w/o -l) [{n_thread}]\n  --meth    build FM-index for BS-seq mapping\n  --help    print this help message\n"
            ),
        )
    }

    /// Original C global function `main_index` from `minibwa/index.c:193`.
    pub fn main_index(argv: &[String]) -> (i32, String) {
        let long_opts = [
            ko_longopt_t {
                name: Some("help".into()),
                has_arg: 0,
                val: 901,
            },
            ko_longopt_t {
                name: Some("meth".into()),
                has_arg: 0,
                val: 902,
            },
        ];
        let argc = argv.len() as i32;
        let mut args = argv.to_vec();
        let mut o = KETOPT_INIT.clone();
        let mut low_mem = 0;
        let mut n_thread = 4;
        let mut sa_bit = 4;
        let mut is_meth = 0;
        let mut block_size = 10_000_000i64;
        let mut seed = 11u64;
        loop {
            let c = ketopt(&mut o, argc, &mut args, 1, "ls:u:b:t:", Some(&long_opts));
            if c < 0 {
                break;
            }
            if c == 't' as i32 {
                n_thread = o.arg.as_deref().unwrap_or("0").parse().unwrap_or(0);
            } else if c == 'l' as i32 {
                low_mem = 1;
            } else if c == 'b' as i32 {
                block_size = kom_parse_num(o.arg.as_deref().unwrap_or("0")).0;
            } else if c == 'u' as i32 {
                sa_bit = o.arg.as_deref().unwrap_or("0").parse().unwrap_or(0);
            } else if c == 's' as i32 {
                seed = o.arg.as_deref().unwrap_or("0").parse().unwrap_or(0);
            } else if c == 901 {
                return usage_index(true, seed, sa_bit, n_thread);
            } else if c == 902 {
                is_meth = 1;
            }
        }
        if argc - o.ind == 0 {
            return usage_index(false, seed, sa_bit, n_thread);
        }
        let prefix = if o.ind + 1 < argc {
            args[o.ind as usize + 1].clone()
        } else {
            args[o.ind as usize].clone()
        };
        let fn_l2b = format!("{prefix}.l2b");
        let fn_bwt = format!("{prefix}.mbw");
        let fn_meth_bwt = format!("{prefix}.meth.mbw");
        let time_index = std::env::var_os("MINIBWA_RS_INDEX_TIMING").is_some();
        let mut stage_start = std::time::Instant::now();
        let Some(mut l2b) = l2b_import(&args[o.ind as usize], seed) else {
            kom_panic("main_index", "failed to read the genome FASTA.");
        };
        if time_index {
            eprintln!(
                "[index::main_index] l2b_import {:.3}s",
                stage_start.elapsed().as_secs_f64()
            );
            stage_start = std::time::Instant::now();
        }
        if low_mem != 0 {
            l2b_save_pac(&fn_l2b, &l2b, 1);
            if mb_bwtgen(
                std::path::Path::new(&fn_l2b),
                std::path::Path::new(&fn_bwt),
                block_size as i32,
            )
            .is_err()
            {
                return (1, String::new());
            }
            l2b_save(&fn_l2b, &l2b);
            l2b.ctg = Vec::new();
            l2b.ambi = Vec::new();
            l2b.mask = Vec::new();
            l2b.cat_name = Vec::new();
            l2b.cat_comm = Vec::new();
            let Some(mut bwt) = mb_bwt_load_raw(&fn_bwt) else {
                return (1, String::new());
            };
            mb_bwt_gen_sa(&mut bwt, sa_bit as u32);
            mb_bwt_save(&fn_bwt, &bwt);
            drop(bwt);
            if is_meth != 0 {
                l2b_save_pac_meth(&fn_l2b, &l2b, 1);
                if mb_bwtgen(
                    std::path::Path::new(&fn_l2b),
                    std::path::Path::new(&fn_meth_bwt),
                    block_size as i32,
                )
                .is_err()
                {
                    return (1, String::new());
                }
                let Some(mut bwt) = mb_bwt_load_raw(&fn_meth_bwt) else {
                    return (1, String::new());
                };
                mb_bwt_gen_sa(&mut bwt, sa_bit as u32);
                mb_bwt_save(&fn_meth_bwt, &bwt);
            }
        } else {
            l2b_save(&fn_l2b, &l2b);
            if time_index {
                eprintln!(
                    "[index::main_index] l2b_save {:.3}s",
                    stage_start.elapsed().as_secs_f64()
                );
                stage_start = std::time::Instant::now();
            }
            l2b.ctg = Vec::new();
            l2b.ambi = Vec::new();
            l2b.mask = Vec::new();
            l2b.cat_name = Vec::new();
            l2b.cat_comm = Vec::new();
            let bwt =
                run_with_index_pool(n_thread, || mb_bwt_libsais(&l2b, sa_bit, 1, 0, n_thread));
            if time_index {
                eprintln!(
                    "[index::main_index] mb_bwt_libsais total {:.3}s",
                    stage_start.elapsed().as_secs_f64()
                );
                stage_start = std::time::Instant::now();
            }
            mb_bwt_save(&fn_bwt, &bwt);
            if time_index {
                eprintln!(
                    "[index::main_index] mb_bwt_save {:.3}s",
                    stage_start.elapsed().as_secs_f64()
                );
            }
            drop(bwt);
            if is_meth != 0 {
                let bwt =
                    run_with_index_pool(n_thread, || mb_bwt_libsais(&l2b, sa_bit, 1, 1, n_thread));
                mb_bwt_save(&fn_meth_bwt, &bwt);
            }
        }
        (0, String::new())
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::bwt::{mb_bwt_load, mb_bwt_sa};
        use crate::l2bit::{l2b_add_seq, l2b_format_seq, l2b_load, l2b_t};
        use std::fs;

        #[test]
        fn bwt_libsais_builds_loadable_forward_reverse_index() {
            let mut seq = b"ACGTAC".to_vec();
            let mut rng = 11u64;
            l2b_format_seq(seq.len() as u64, &mut seq, &mut rng);
            let mut l2b = l2b_t::default();
            l2b_add_seq(&mut l2b, seq.len() as u64, &seq, "ctg", None, &mut rng);
            let bwt = mb_bwt_libsais(&l2b, 2, 1, 0, 1);
            assert_eq!(bwt.seq_len, 12);
            assert_eq!(bwt.sa_bit, 2);
            assert_eq!(bwt.sa[0], u64::MAX);
            assert!(bwt.primary <= bwt.seq_len);
            assert!(mb_bwt_sa(&bwt, 4) <= bwt.seq_len);
        }

        #[test]
        fn fa2bit_and_genbwt_subcommands_roundtrip_small_fasta() {
            let mut dir = std::env::temp_dir();
            dir.push(format!(
                "minibwa_rs_index_roundtrip_{}_{}",
                std::process::id(),
                crate::kommon::kom_realtime().to_bits()
            ));
            fs::create_dir_all(&dir).unwrap();
            let fa = dir.join("in.fa");
            let l2b = dir.join("out.l2b");
            let mbw = dir.join("out.mbw");
            fs::write(&fa, b">ctg\nACGTACGTACGT\n").unwrap();
            let args = vec![
                "fa2bit".to_string(),
                fa.to_string_lossy().into_owned(),
                l2b.to_string_lossy().into_owned(),
            ];
            assert_eq!(main_fa2bit(&args).0, 0);
            let loaded = l2b_load(&l2b).expect("load generated l2b");
            assert_eq!(loaded.tot_len, 12);
            assert_eq!(loaded.ctg[0].name, "ctg");

            let args = vec![
                "genbwt".to_string(),
                "-u".to_string(),
                "2".to_string(),
                l2b.to_string_lossy().into_owned(),
                mbw.to_string_lossy().into_owned(),
            ];
            assert_eq!(main_genbwt(&args).0, 0);
            let bwt = mb_bwt_load(&mbw).expect("load generated bwt");
            assert_eq!(bwt.seq_len, 24);
            assert_eq!(bwt.sa_bit, 2);
            let _ = fs::remove_dir_all(&dir);
        }

        #[test]
        fn main_index_builds_l2b_and_mbw_prefix_files() {
            let mut dir = std::env::temp_dir();
            dir.push(format!(
                "minibwa_rs_index_main_index_{}_{}",
                std::process::id(),
                crate::kommon::kom_realtime().to_bits()
            ));
            fs::create_dir_all(&dir).unwrap();
            let fa = dir.join("in.fa");
            let prefix = dir.join("idx");
            fs::write(&fa, b">ctg\nACGTACGT\n").unwrap();
            let args = vec![
                "index".to_string(),
                "-u".to_string(),
                "2".to_string(),
                fa.to_string_lossy().into_owned(),
                prefix.to_string_lossy().into_owned(),
            ];
            assert_eq!(main_index(&args).0, 0);
            assert!(l2b_load(format!("{}.l2b", prefix.to_string_lossy())).is_some());
            assert!(mb_bwt_load(format!("{}.mbw", prefix.to_string_lossy())).is_some());
            let _ = fs::remove_dir_all(&dir);
        }
    }
}

pub mod kalloc {
    #![allow(unused_variables, dead_code, non_snake_case, non_camel_case_types)]

    use std::alloc::{alloc, alloc_zeroed, dealloc, Layout};
    use std::collections::HashMap;
    use std::ptr;

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct header_t {
        pub size: usize,
        pub ptr: *mut header_t,
    }

    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    pub struct km_stat_t {
        pub capacity: usize,
        pub available: usize,
        pub n_blocks: usize,
        pub n_cores: usize,
        pub largest: usize,
    }

    #[derive(Debug)]
    pub struct kmem_t {
        pub par_min_core_size: usize,
        pub min_core_size: usize,
        pub capacity: usize,
        pub active: usize,
        pub largest: usize,
        pub allocations: HashMap<usize, usize>,
    }

    /// Original C static function `panic` from `minibwa/kalloc.c:32`.
    pub fn panic(s: &str) -> ! {
        eprintln!("{s}");
        unsafe { libc::abort() }
    }

    /// Original C global function `km_init2` from `minibwa/kalloc.c:38`.
    pub fn km_init2(km_par: Option<&kmem_t>, min_core_size: usize) -> kmem_t {
        let parent_min = km_par.map(|km| km.min_core_size).unwrap_or(0);
        let min_core_size = if let Some(parent) = km_par {
            if min_core_size > 0 {
                min_core_size
            } else {
                parent.min_core_size.saturating_sub(2)
            }
        } else if min_core_size > 0 {
            min_core_size
        } else {
            0x80000
        };
        kmem_t {
            par_min_core_size: parent_min,
            min_core_size,
            capacity: 0,
            active: 0,
            largest: 0,
            allocations: HashMap::new(),
        }
    }

    /// Original C global function `km_init` from `minibwa/kalloc.c:48`.
    pub fn km_init() -> kmem_t {
        km_init2(None, 0)
    }

    /// Original C global function `km_destroy` from `minibwa/kalloc.c:50`.
    pub unsafe fn km_destroy(km: Option<&mut kmem_t>) {
        let Some(km) = km else {
            return;
        };
        let ptrs = km.allocations.keys().copied().collect::<Vec<_>>();
        for ptr in ptrs {
            kfree(Some(km), ptr as *mut u8);
        }
    }

    /// Original C static function `morecore` from `minibwa/kalloc.c:65`.
    pub fn morecore(km: &mut kmem_t, nu: usize) -> usize {
        let units = (nu + 1 + (km.min_core_size - 1)) / km.min_core_size * km.min_core_size;
        let bytes = units * std::mem::size_of::<usize>() * 2;
        km.capacity += bytes;
        km.largest = km.largest.max(bytes);
        units
    }

    /// Original C global function `kfree` from `minibwa/kalloc.c:80`.
    pub unsafe fn kfree(km: Option<&mut kmem_t>, ap: *mut u8) {
        if ap.is_null() {
            return;
        }
        let header = std::mem::size_of::<usize>();
        let base = ap.sub(header);
        let size = *(base as *const usize);
        let total = size + header;
        let layout = Layout::from_size_align(total.max(1), std::mem::align_of::<usize>()).unwrap();
        dealloc(base, layout);
        if let Some(km) = km {
            km.allocations.remove(&(ap as usize));
            km.active = km.active.saturating_sub(size);
        }
    }

    /// Original C global function `kmalloc` from `minibwa/kalloc.c:128`.
    pub unsafe fn kmalloc(km: Option<&mut kmem_t>, n_bytes: usize) -> *mut u8 {
        if n_bytes == 0 {
            return ptr::null_mut();
        }
        let header = std::mem::size_of::<usize>();
        let total = n_bytes + header;
        let layout = Layout::from_size_align(total, std::mem::align_of::<usize>()).unwrap();
        let base = alloc(layout);
        if base.is_null() {
            return ptr::null_mut();
        }
        *(base as *mut usize) = n_bytes;
        let ap = base.add(header);
        if let Some(km) = km {
            km.capacity += total;
            km.active += n_bytes;
            km.largest = km.largest.max(total);
            km.allocations.insert(ap as usize, n_bytes);
        }
        ap
    }

    /// Original C global function `kcalloc` from `minibwa/kalloc.c:157`.
    pub unsafe fn kcalloc(km: Option<&mut kmem_t>, count: usize, size: usize) -> *mut u8 {
        if size == 0 || count == 0 {
            return ptr::null_mut();
        }
        let n_bytes = count * size;
        let header = std::mem::size_of::<usize>();
        let total = n_bytes + header;
        let layout = Layout::from_size_align(total, std::mem::align_of::<usize>()).unwrap();
        let base = alloc_zeroed(layout);
        if base.is_null() {
            return ptr::null_mut();
        }
        *(base as *mut usize) = n_bytes;
        let ap = base.add(header);
        if let Some(km) = km {
            km.capacity += total;
            km.active += n_bytes;
            km.largest = km.largest.max(total);
            km.allocations.insert(ap as usize, n_bytes);
        }
        ap
    }

    /// Original C global function `krealloc` from `minibwa/kalloc.c:168`.
    pub unsafe fn krealloc(km: Option<&mut kmem_t>, ap: *mut u8, n_bytes: usize) -> *mut u8 {
        if n_bytes == 0 {
            kfree(km, ap);
            return ptr::null_mut();
        }
        if ap.is_null() {
            return kmalloc(km, n_bytes);
        }
        let header = std::mem::size_of::<usize>();
        let base = ap.sub(header);
        let cap = *(base as *const usize);
        if cap >= n_bytes {
            return ap;
        }
        match km {
            Some(km) => {
                let q = kmalloc(Some(km), n_bytes);
                if !q.is_null() {
                    ptr::copy_nonoverlapping(ap, q, cap);
                    kfree(Some(km), ap);
                }
                q
            }
            None => {
                let q = kmalloc(None, n_bytes);
                if !q.is_null() {
                    ptr::copy_nonoverlapping(ap, q, cap);
                    kfree(None, ap);
                }
                q
            }
        }
    }

    /// Original C global function `krelocate` from `minibwa/kalloc.c:187`.
    pub unsafe fn krelocate(km: Option<&mut kmem_t>, ap: *mut u8, n_bytes: usize) -> *mut u8 {
        if km.is_none() || ap.is_null() {
            return ap;
        }
        let km = km.unwrap();
        let p = kmalloc(Some(km), n_bytes);
        if !p.is_null() {
            ptr::copy_nonoverlapping(ap, p, n_bytes);
            kfree(Some(km), ap);
        }
        p
    }

    /// Original C global function `km_stat` from `minibwa/kalloc.c:197`.
    pub fn km_stat(km: Option<&kmem_t>, s: &mut km_stat_t) {
        *s = km_stat_t::default();
        let Some(km) = km else {
            return;
        };
        s.capacity = km.capacity;
        s.available = km.capacity.saturating_sub(km.active);
        s.n_blocks = km.allocations.len();
        s.n_cores = if km.capacity == 0 { 0 } else { 1 };
        s.largest = km.largest;
    }

    /// Original C global function `km_stat_print` from `minibwa/kalloc.c:218`.
    pub fn km_stat_print(km: Option<&kmem_t>) -> String {
        let mut st = km_stat_t::default();
        km_stat(km, &mut st);
        format!(
            "[km_stat] cap={}, avail={}, largest={}, n_core={}, n_block={}\n",
            st.capacity, st.available, st.largest, st.n_blocks, st.n_cores
        )
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn kalloc_alloc_realloc_relocate_and_stats_track_live_blocks() {
            unsafe {
                let mut km = km_init();
                let p = kmalloc(Some(&mut km), 8);
                assert!(!p.is_null());
                for i in 0..8 {
                    *p.add(i) = i as u8;
                }
                let q = krealloc(Some(&mut km), p, 32);
                assert!(!q.is_null());
                for i in 0..8 {
                    assert_eq!(*q.add(i), i as u8);
                }
                let r = krelocate(Some(&mut km), q, 16);
                assert!(!r.is_null());
                for i in 0..8 {
                    assert_eq!(*r.add(i), i as u8);
                }
                let z = kcalloc(Some(&mut km), 4, 4);
                for i in 0..16 {
                    assert_eq!(*z.add(i), 0);
                }
                let mut st = km_stat_t::default();
                km_stat(Some(&km), &mut st);
                assert_eq!(st.n_blocks, 2);
                assert!(st.capacity >= st.available);
                assert!(km_stat_print(Some(&km)).starts_with("[km_stat] cap="));
                kfree(Some(&mut km), r);
                kfree(Some(&mut km), z);
                km_stat(Some(&km), &mut st);
                assert_eq!(st.n_blocks, 0);
            }
        }

        #[test]
        fn kalloc_null_allocator_uses_header_backed_raw_allocations() {
            unsafe {
                let p = kcalloc(None, 3, 5);
                assert!(!p.is_null());
                for i in 0..15 {
                    assert_eq!(*p.add(i), 0);
                }
                let q = krealloc(None, p, 24);
                assert!(!q.is_null());
                kfree(None, q);
            }
        }
    }
}

pub mod ketopt {
    #![allow(
        unused_variables,
        dead_code,
        non_snake_case,
        non_camel_case_types,
        non_upper_case_globals
    )]

    pub const ko_no_argument: i32 = 0;
    pub const ko_required_argument: i32 = 1;
    pub const ko_optional_argument: i32 = 2;

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct ketopt_t {
        pub ind: i32,
        pub opt: i32,
        pub arg: Option<String>,
        pub longidx: i32,
        pub i: i32,
        pub pos: i32,
        pub n_args: i32,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct ko_longopt_t {
        pub name: Option<String>,
        pub has_arg: i32,
        pub val: i32,
    }

    pub static KETOPT_INIT: ketopt_t = ketopt_t {
        ind: 1,
        opt: 0,
        arg: None,
        longidx: -1,
        i: 1,
        pos: 0,
        n_args: 0,
    };

    /// Original C static function `ketopt_permute` from `minibwa/ketopt.h:27`.
    pub fn ketopt_permute(argv: &mut [String], j: i32, n: i32) {
        let mut k = 0;
        let p = argv[j as usize].clone();
        while k < n {
            let dst = (j - k) as usize;
            let src = (j - k - 1) as usize;
            argv[dst] = argv[src].clone();
            k += 1;
        }
        argv[(j - k) as usize] = p;
    }

    /// Original C static function `ketopt` from `minibwa/ketopt.h:56`.
    pub fn ketopt(
        s: &mut ketopt_t,
        argc: i32,
        argv: &mut [String],
        permute: i32,
        ostr: &str,
        longopts: Option<&[ko_longopt_t]>,
    ) -> i32 {
        let mut opt: i32;
        if permute != 0 {
            while s.i < argc {
                let a = argv[s.i as usize].as_bytes();
                if a.first() == Some(&b'-') && a.get(1) != Some(&b'\0') && a.len() > 1 {
                    break;
                }
                s.i += 1;
                s.n_args += 1;
            }
        }
        s.arg = None;
        s.longidx = -1;
        let i0 = s.i;
        if s.i >= argc {
            s.ind = s.i - s.n_args;
            return -1;
        }
        let cur = argv[s.i as usize].clone();
        let cur_b = cur.as_bytes();
        if cur_b.first() != Some(&b'-') || cur_b.get(1).is_none() {
            s.ind = s.i - s.n_args;
            return -1;
        }
        if cur_b.first() == Some(&b'-') && cur_b.get(1) == Some(&b'-') {
            if cur_b.get(2).is_none() {
                ketopt_permute(argv, s.i, s.n_args);
                s.i += 1;
                s.ind = s.i - s.n_args;
                return -1;
            }
            s.opt = 0;
            opt = '?' as i32;
            s.pos = -1;
            if let Some(longopts) = longopts {
                let mut j = 2usize;
                while j < cur_b.len() && cur_b[j] != b'=' {
                    j += 1;
                }
                let query = &cur[2..j];
                let mut n_exact = 0;
                let mut n_partial = 0;
                let mut exact_idx = 0usize;
                let mut partial_idx = 0usize;
                for (k, o) in longopts.iter().enumerate() {
                    let Some(name) = &o.name else {
                        break;
                    };
                    if name.starts_with(query) {
                        if name.len() == query.len() {
                            n_exact += 1;
                            exact_idx = k;
                        } else {
                            n_partial += 1;
                            partial_idx = k;
                        }
                    }
                }
                if n_exact > 1 || (n_exact == 0 && n_partial > 1) {
                    s.i += 1;
                    return '?' as i32;
                }
                let o_idx = if n_exact == 1 {
                    Some(exact_idx)
                } else if n_partial == 1 {
                    Some(partial_idx)
                } else {
                    None
                };
                if let Some(o_idx) = o_idx {
                    let o = &longopts[o_idx];
                    s.opt = o.val;
                    opt = o.val;
                    s.longidx = o_idx as i32;
                    if j < cur_b.len() && cur_b[j] == b'=' {
                        s.arg = Some(cur[j + 1..].to_string());
                    }
                    if o.has_arg == ko_required_argument && j == cur_b.len() {
                        if s.i < argc - 1 {
                            s.i += 1;
                            s.arg = Some(argv[s.i as usize].clone());
                        } else {
                            opt = ':' as i32;
                        }
                    }
                }
            }
        } else {
            if s.pos == 0 {
                s.pos = 1;
            }
            let pos = s.pos as usize;
            opt = cur_b[pos] as i32;
            s.opt = opt;
            s.pos += 1;
            if let Some(p) = ostr.as_bytes().iter().position(|&c| c as i32 == opt) {
                if ostr.as_bytes().get(p + 1) == Some(&b':') {
                    if s.pos as usize >= cur_b.len() {
                        if s.i < argc - 1 {
                            s.i += 1;
                            s.arg = Some(argv[s.i as usize].clone());
                        } else {
                            opt = ':' as i32;
                        }
                    } else {
                        s.arg = Some(cur[s.pos as usize..].to_string());
                    }
                    s.pos = -1;
                }
            } else {
                opt = '?' as i32;
            }
        }
        if s.pos < 0 || s.pos as usize >= argv[s.i as usize].len() {
            s.i += 1;
            s.pos = 0;
            if s.n_args > 0 {
                let mut j = i0;
                while j < s.i {
                    ketopt_permute(argv, j, s.n_args);
                    j += 1;
                }
            }
        }
        s.ind = s.i - s.n_args;
        opt
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn ketopt_parses_short_options_and_permutes_arguments() {
            let mut argv = vec![
                "prog".to_string(),
                "input.fa".to_string(),
                "-abval".to_string(),
                "tail".to_string(),
            ];
            let mut st = KETOPT_INIT.clone();
            assert_eq!(ketopt(&mut st, 4, &mut argv, 1, "ab:", None), 'a' as i32);
            assert_eq!(st.arg, None);
            assert_eq!(ketopt(&mut st, 4, &mut argv, 1, "ab:", None), 'b' as i32);
            assert_eq!(st.arg.as_deref(), Some("val"));
            assert_eq!(ketopt(&mut st, 4, &mut argv, 1, "ab:", None), -1);
            assert_eq!(st.ind, 2);
            assert_eq!(argv[1], "-abval");
            assert_eq!(argv[2], "input.fa");
        }

        #[test]
        fn ketopt_parses_long_options_and_errors() {
            let longopts = vec![
                ko_longopt_t {
                    name: Some("threads".to_string()),
                    has_arg: ko_required_argument,
                    val: 't' as i32,
                },
                ko_longopt_t {
                    name: Some("verbose".to_string()),
                    has_arg: ko_no_argument,
                    val: 'v' as i32,
                },
                ko_longopt_t {
                    name: None,
                    has_arg: 0,
                    val: 0,
                },
            ];
            let mut argv = vec![
                "prog".to_string(),
                "--threads=8".to_string(),
                "--verb".to_string(),
                "--unknown".to_string(),
            ];
            let mut st = KETOPT_INIT.clone();
            assert_eq!(
                ketopt(&mut st, 4, &mut argv, 0, "", Some(&longopts)),
                't' as i32
            );
            assert_eq!(st.longidx, 0);
            assert_eq!(st.arg.as_deref(), Some("8"));
            assert_eq!(
                ketopt(&mut st, 4, &mut argv, 0, "", Some(&longopts)),
                'v' as i32
            );
            assert_eq!(st.longidx, 1);
            assert_eq!(
                ketopt(&mut st, 4, &mut argv, 0, "", Some(&longopts)),
                '?' as i32
            );
        }
    }
}

pub mod kommon {
    #![allow(unused_variables, dead_code, non_snake_case, non_camel_case_types)]

    use std::sync::{Mutex, OnceLock};
    use std::time::{SystemTime, UNIX_EPOCH};

    pub type c_long = libc::c_long;
    pub type rusage = libc::rusage;
    pub type timeval = libc::timeval;

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct timezone {
        pub tz_minuteswest: i32,
        pub tz_dsttime: i32,
    }

    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    pub struct kstring_t {
        pub l: usize,
        pub m: usize,
        pub s: Vec<u8>,
    }

    const fn make_kom_nt4_table() -> [u8; 256] {
        let mut table = [4u8; 256];
        table[0] = 0;
        table[1] = 1;
        table[2] = 2;
        table[3] = 3;
        table[b'A' as usize] = 0;
        table[b'C' as usize] = 1;
        table[b'G' as usize] = 2;
        table[b'T' as usize] = 3;
        table[b'a' as usize] = 0;
        table[b'c' as usize] = 1;
        table[b'g' as usize] = 2;
        table[b't' as usize] = 3;
        table
    }

    pub const KOM_NT4_TABLE: [u8; 256] = make_kom_nt4_table();

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum kom_sprintf_arg<'a> {
        d(i32),
        ld(i64),
        u(u32),
        s(&'a str),
        c(i32),
    }

    /// Original C global function `kom_strdup` from `minibwa/kommon.c:9`.
    pub fn kom_strdup(src: &str) -> String {
        src.to_string()
    }

    /// Original C global function `kom_parse_num` from `minibwa/kommon.c:19`.
    pub fn kom_parse_num(str_: &str) -> (i64, usize) {
        let bytes = str_.as_bytes();
        let nul = bytes.iter().position(|&c| c == 0).unwrap_or(bytes.len());
        let mut cstr = Vec::with_capacity(nul + 1);
        cstr.extend_from_slice(&bytes[..nul]);
        cstr.push(0);
        let mut end: *mut libc::c_char = std::ptr::null_mut();
        let mut x = unsafe { libc::strtod(cstr.as_ptr() as *const libc::c_char, &mut end) };
        let mut p = if end.is_null() {
            0
        } else {
            unsafe { end.offset_from(cstr.as_ptr() as *const libc::c_char) as usize }
        };
        if p < bytes.len() {
            if bytes[p] == b'G' || bytes[p] == b'g' {
                x *= 1e9;
                p += 1;
            } else if bytes[p] == b'M' || bytes[p] == b'm' {
                x *= 1e6;
                p += 1;
            } else if bytes[p] == b'K' || bytes[p] == b'k' {
                x *= 1e3;
                p += 1;
            }
        }
        ((x + 0.499) as i64, p)
    }

    /// Original C global function `kom_panic` from `minibwa/kommon.c:31`.
    pub fn kom_panic(func: &str, msg: &str) -> ! {
        eprintln!("[E::{func}] {msg} ABORT!");
        unsafe { libc::abort() }
    }

    /// Original C static function `str_enlarge` from `minibwa/kommon.c:45`.
    pub fn str_enlarge(km: (), s: &mut kstring_t, l: i32) {
        let need = s.l + l as usize + 1;
        if need > s.m {
            s.m = need;
            s.m = s.m.wrapping_sub(1);
            s.m |= s.m >> 1;
            s.m |= s.m >> 2;
            s.m |= s.m >> 4;
            s.m |= s.m >> 8;
            s.m |= s.m >> 16;
            s.m |= s.m >> 32;
            s.m = s.m.wrapping_add(1);
            s.s.resize(s.m, 0);
        }
    }

    /// Original C static function `str_copy` from `minibwa/kommon.c:58`.
    pub fn str_copy(km: (), s: &mut kstring_t, st: &[u8], en: usize) {
        str_enlarge(km, s, en as i32);
        let end = s.l + en;
        s.s[s.l..end].copy_from_slice(&st[..en]);
        s.l = end;
    }

    /// Original C static function `kom_splitmix64` from `minibwa/kommon.h:63`.
    pub fn kom_splitmix64(x: &mut u64) -> u64 {
        *x = x.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = *x;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }

    /// Original C global function `kom_sprintf_lite_core` from `minibwa/kommon.c:65`.
    pub fn kom_sprintf_lite_core(
        km: (),
        mut s: Option<&mut kstring_t>,
        fmt: &str,
        ap: &[kom_sprintf_arg<'_>],
    ) -> i64 {
        let mut len = 0i64;
        let bytes = fmt.as_bytes();
        let mut p = 0usize;
        let mut q = 0usize;
        let mut ai = 0usize;
        while p < bytes.len() {
            if bytes[p] == b'%' {
                if p > q {
                    len += (p - q) as i64;
                    if let Some(ref mut out) = s {
                        str_copy(km, out, &bytes[q..p], p - q);
                    }
                }
                p += 1;
                let text = if p < bytes.len() && bytes[p] == b'd' {
                    let v = match ap[ai] {
                        kom_sprintf_arg::d(v) => v,
                        _ => panic!("kom_sprintf_lite_core: expected %d argument"),
                    };
                    ai += 1;
                    v.to_string()
                } else if p + 1 < bytes.len() && bytes[p] == b'l' && bytes[p + 1] == b'd' {
                    let v = match ap[ai] {
                        kom_sprintf_arg::ld(v) => v,
                        _ => panic!("kom_sprintf_lite_core: expected %ld argument"),
                    };
                    ai += 1;
                    p += 1;
                    v.to_string()
                } else if p < bytes.len() && bytes[p] == b'u' {
                    let v = match ap[ai] {
                        kom_sprintf_arg::u(v) => v,
                        _ => panic!("kom_sprintf_lite_core: expected %u argument"),
                    };
                    ai += 1;
                    v.to_string()
                } else if p < bytes.len() && bytes[p] == b's' {
                    let v = match ap[ai] {
                        kom_sprintf_arg::s(v) => v,
                        _ => panic!("kom_sprintf_lite_core: expected %s argument"),
                    };
                    ai += 1;
                    v.to_string()
                } else if p < bytes.len() && bytes[p] == b'c' {
                    let v = match ap[ai] {
                        kom_sprintf_arg::c(v) => v as u8,
                        _ => panic!("kom_sprintf_lite_core: expected %c argument"),
                    };
                    ai += 1;
                    String::from_utf8(vec![v]).unwrap()
                } else {
                    let ch = if p < bytes.len() {
                        bytes[p] as char
                    } else {
                        '\0'
                    };
                    panic!("ERROR: unrecognized type '%{ch}'");
                };
                len += text.len() as i64;
                if let Some(ref mut out) = s {
                    str_copy(km, out, text.as_bytes(), text.len());
                }
                q = p + 1;
            }
            p += 1;
        }
        if p > q {
            len += (p - q) as i64;
            if let Some(ref mut out) = s {
                str_copy(km, out, &bytes[q..p], p - q);
            }
        }
        if let Some(ref mut out) = s {
            if out.s.len() <= out.l {
                out.s.resize(out.l + 1, 0);
                out.m = out.s.len();
            }
            out.s[out.l] = 0;
        }
        len
    }

    /// Original C static function `kom_u64todbl` from `minibwa/kommon.h:71`.
    pub fn kom_u64todbl(x: u64) -> f64 {
        f64::from_bits(0x3ffu64 << 52 | x >> 12) - 1.0
    }

    /// Original C global function `kom_sprintf_lite` from `minibwa/kommon.c:140`.
    pub fn kom_sprintf_lite(s: &mut kstring_t, fmt: &str, ap: &[kom_sprintf_arg<'_>]) -> i64 {
        kom_sprintf_lite_core((), Some(s), fmt, ap)
    }

    /// Original C global function `km_sprintf_lite` from `minibwa/kommon.c:148`.
    pub fn km_sprintf_lite(
        km: (),
        s: &mut kstring_t,
        fmt: &str,
        ap: &[kom_sprintf_arg<'_>],
    ) -> i64 {
        kom_sprintf_lite_core(km, Some(s), fmt, ap)
    }

    /// Original C global function `kom_revcomp` from `minibwa/kommon.c:198`.
    pub fn kom_revcomp(len: u64, seq: &mut [u8]) {
        const KOM_COMP_TABLE: [u8; 256] = [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45,
            46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 84, 86, 71,
            72, 69, 70, 67, 68, 73, 74, 77, 76, 75, 78, 79, 80, 81, 89, 83, 65, 65, 66, 87, 88, 82,
            90, 91, 92, 93, 94, 95, 96, 116, 118, 103, 104, 101, 102, 99, 100, 105, 106, 109, 108,
            107, 110, 111, 112, 113, 121, 115, 97, 97, 98, 119, 120, 114, 122, 123, 124, 125, 126,
            127, 128, 129, 130, 131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143,
            144, 145, 146, 147, 148, 149, 150, 151, 152, 153, 154, 155, 156, 157, 158, 159, 160,
            161, 162, 163, 164, 165, 166, 167, 168, 169, 170, 171, 172, 173, 174, 175, 176, 177,
            178, 179, 180, 181, 182, 183, 184, 185, 186, 187, 188, 189, 190, 191, 192, 193, 194,
            195, 196, 197, 198, 199, 200, 201, 202, 203, 204, 205, 206, 207, 208, 209, 210, 211,
            212, 213, 214, 215, 216, 217, 218, 219, 220, 221, 222, 223, 224, 225, 226, 227, 228,
            229, 230, 231, 232, 233, 234, 235, 236, 237, 238, 239, 240, 241, 242, 243, 244, 245,
            246, 247, 248, 249, 250, 251, 252, 253, 254, 255,
        ];
        for i in 0..(len >> 1) as usize {
            let t = seq[len as usize - i - 1];
            seq[len as usize - i - 1] = KOM_COMP_TABLE[seq[i] as usize];
            seq[i] = KOM_COMP_TABLE[t as usize];
        }
        if len & 1 != 0 {
            let mid = (len >> 1) as usize;
            seq[mid] = KOM_COMP_TABLE[seq[mid] as usize];
        }
    }

    /// Original C global function `gettimeofday` from `minibwa/kommon.c:261`.
    pub fn gettimeofday() -> timeval {
        let dur = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        timeval {
            tv_sec: dur.as_secs() as libc::time_t,
            tv_usec: dur.subsec_micros() as libc::suseconds_t,
        }
    }

    /// Original C global function `kom_cputime` from `minibwa/kommon.c:279`.
    pub fn kom_cputime() -> f64 {
        let mut r: rusage = unsafe { std::mem::zeroed() };
        unsafe {
            libc::getrusage(libc::RUSAGE_SELF, &mut r);
        }
        r.ru_utime.tv_sec as f64
            + r.ru_stime.tv_sec as f64
            + 1e-6 * (r.ru_utime.tv_usec + r.ru_stime.tv_usec) as f64
    }

    /// Original C global function `kom_peakrss` from `minibwa/kommon.c:296`.
    pub fn kom_peakrss() -> c_long {
        let mut r: rusage = unsafe { std::mem::zeroed() };
        unsafe {
            libc::getrusage(libc::RUSAGE_SELF, &mut r);
        }
        if cfg!(target_os = "linux") {
            r.ru_maxrss * 1024
        } else {
            r.ru_maxrss
        }
    }

    /// Original C global function `kom_realtime` from `minibwa/kommon.c:321`.
    pub fn kom_realtime() -> f64 {
        static REALTIME0: OnceLock<Mutex<f64>> = OnceLock::new();
        let tp = gettimeofday();
        let t = tp.tv_sec as f64 + tp.tv_usec as f64 * 1e-6;
        let lock = REALTIME0.get_or_init(|| Mutex::new(-1.0));
        let mut realtime0 = lock.lock().unwrap();
        if *realtime0 < 0.0 {
            *realtime0 = t;
        }
        t - *realtime0
    }

    /// Original C global function `kom_percent_cpu` from `minibwa/kommon.c:332`.
    pub fn kom_percent_cpu() -> f64 {
        (kom_cputime() + 1e-6) / (kom_realtime() + 1e-6)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn parse_num_matches_suffix_rounding_rules() {
            assert_eq!(kom_parse_num("12.4k rest"), (12400, 5));
            assert_eq!(kom_parse_num("1.5M"), (1500000, 4));
            assert_eq!(kom_parse_num("2Gx"), (2000000000, 2));
            assert_eq!(kom_parse_num("17"), (17, 2));
            assert_eq!(kom_parse_num("  .5k"), (500, 5));
            assert_eq!(kom_parse_num("abc"), (0, 0));
        }

        #[test]
        fn splitmix_and_u64todbl_match_known_vectors() {
            let mut x = 0u64;
            assert_eq!(kom_splitmix64(&mut x), 0xe220a8397b1dcdaf);
            assert_eq!(x, 0x9e3779b97f4a7c15);
            assert_eq!(kom_u64todbl(0), 0.0);
            assert!(kom_u64todbl(u64::MAX) < 1.0);
        }

        #[test]
        fn sprintf_lite_writes_supported_conversions() {
            let mut s = kstring_t::default();
            let len = kom_sprintf_lite(
                &mut s,
                "%s:%d/%ld/%u/%c",
                &[
                    kom_sprintf_arg::s("read"),
                    kom_sprintf_arg::d(-7),
                    kom_sprintf_arg::ld(123456789),
                    kom_sprintf_arg::u(42),
                    kom_sprintf_arg::c(b'X' as i32),
                ],
            );
            assert_eq!(len as usize, "read:-7/123456789/42/X".len());
            assert_eq!(&s.s[..s.l], b"read:-7/123456789/42/X");
            assert_eq!(s.s[s.l], 0);
        }

        #[test]
        fn revcomp_matches_real_chrm_prefix() {
            let mut seq = b"GATCACAGGTCTATCACCCTATTAACCACTCACGGGAGCTCTCCATGCAT".to_vec();
            let len = seq.len() as u64;
            kom_revcomp(len, &mut seq);
            assert_eq!(&seq, b"ATGCATGGAGAGCTCCCGTGAGTGGTTAATAGGGTGATAGACCTGTGATC");
        }

        #[test]
        fn timing_functions_return_nonnegative_values() {
            let tv = gettimeofday();
            assert!(tv.tv_sec > 0);
            assert!(kom_cputime() >= 0.0);
            assert!(kom_peakrss() >= 0);
            assert!(kom_realtime() >= 0.0);
            assert!(kom_percent_cpu() >= 0.0);
        }
    }
}

pub mod stage_time {
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
}

pub mod kommon_cfg_variants {
    #![allow(unused_variables, dead_code, non_snake_case)]

    use libc::c_long;

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct FILETIME {
        pub dwLowDateTime: u32,
        pub dwHighDateTime: u32,
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct SYSTEMTIME {
        pub wHour: u16,
        pub wMinute: u16,
        pub wSecond: u16,
        pub wMilliseconds: u16,
    }

    /// Original C Windows global function `kom_cputime` from `minibwa/kommon.c:279`.
    pub fn kom_cputime() -> f64 {
        // The Windows body sums kernel and user FILETIME values after converting
        // them to SYSTEMTIME. The portable crate runtime uses `kommon::kom_cputime`;
        // this cfg variant records the alternate original body for audit.
        let stKernel = SYSTEMTIME::default();
        let stUser = SYSTEMTIME::default();
        let kernelModeTime = ((stKernel.wHour as f64 * 60.0) + stKernel.wMinute as f64 * 60.0)
            + stKernel.wSecond as f64
            + stKernel.wMilliseconds as f64 / 1000.0;
        let userModeTime = ((stUser.wHour as f64 * 60.0) + stUser.wMinute as f64 * 60.0)
            + stUser.wSecond as f64
            + stUser.wMilliseconds as f64 / 1000.0;
        kernelModeTime + userModeTime
    }

    /// Original C Windows global function `kom_peakrss` from `minibwa/kommon.c:296`.
    pub fn kom_peakrss() -> c_long {
        0
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn windows_cfg_timing_variants_follow_stubbed_original_shape() {
            assert_eq!(kom_cputime(), 0.0);
            assert_eq!(kom_peakrss(), 0);
        }
    }
}

pub mod ksw2 {
    #![allow(unused_variables, dead_code, non_snake_case, non_camel_case_types)]

    pub const KSW_NEG_INF: i32 = -0x40000000;
    pub const KSW_EZ_SCORE_ONLY: i32 = 0x01;
    pub const KSW_EZ_RIGHT: i32 = 0x02;
    pub const KSW_EZ_GENERIC_SC: i32 = 0x04;
    pub const KSW_EZ_APPROX_MAX: i32 = 0x08;
    pub const KSW_EZ_APPROX_DROP: i32 = 0x10;
    pub const KSW_EZ_EXTZ_ONLY: i32 = 0x40;
    pub const KSW_EZ_REV_CIGAR: i32 = 0x80;
    pub const KSW_EZ_SPLICE_FOR: i32 = 0x100;
    pub const KSW_EZ_SPLICE_REV: i32 = 0x200;
    pub const KSW_EZ_SPLICE_FLANK: i32 = 0x400;
    pub const KSW_EZ_SPLICE_CMPLX: i32 = 0x800;
    pub const KSW_EZ_SPLICE_SCORE: i32 = 0x1000;
    pub const KSW_LL_STOP: i32 = 0x20000;
    pub const KSW_LL_SUBO: i32 = 0x40000;
    pub const KSW_SPSC_OFFSET: i32 = 64;

    pub const KSW_CIGAR_MATCH: u32 = 0;
    pub const KSW_CIGAR_INS: u32 = 1;
    pub const KSW_CIGAR_DEL: u32 = 2;
    pub const KSW_CIGAR_N_SKIP: u32 = 3;

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct ksw_extz_t {
        pub max: u32,
        pub zdropped: u32,
        pub max_q: i32,
        pub max_t: i32,
        pub mqe: i32,
        pub mqe_t: i32,
        pub mte: i32,
        pub mte_q: i32,
        pub score: i32,
        pub m_cigar: i32,
        pub n_cigar: i32,
        pub reach_end: i32,
        pub cigar: Vec<u32>,
    }

    impl Default for ksw_extz_t {
        fn default() -> Self {
            Self {
                max: 0,
                zdropped: 0,
                max_q: -1,
                max_t: -1,
                mqe: KSW_NEG_INF,
                mqe_t: -1,
                mte: KSW_NEG_INF,
                mte_q: -1,
                score: KSW_NEG_INF,
                m_cigar: 0,
                n_cigar: 0,
                reach_end: 0,
                cigar: Vec::new(),
            }
        }
    }

    /// Original C static function `ksw_push_cigar` from `minibwa/ksw2.h:124`.
    pub fn ksw_push_cigar(
        km: (),
        n_cigar: &mut i32,
        m_cigar: &mut i32,
        cigar: &mut Vec<u32>,
        op: u32,
        len: i32,
    ) {
        if *n_cigar == 0 || op != (cigar[*n_cigar as usize - 1] & 0xf) {
            if *n_cigar == *m_cigar {
                *m_cigar = if *m_cigar != 0 { *m_cigar << 1 } else { 4 };
                cigar.reserve((*m_cigar as usize).saturating_sub(cigar.capacity()));
            }
            if *n_cigar as usize == cigar.len() {
                cigar.push((len as u32) << 4 | op);
            } else {
                cigar[*n_cigar as usize] = (len as u32) << 4 | op;
            }
            *n_cigar += 1;
        } else {
            cigar[*n_cigar as usize - 1] += (len as u32) << 4;
        }
    }

    /// Original C static function `ksw_backtrack` from `minibwa/ksw2.h:140`.
    pub fn ksw_backtrack(
        km: (),
        is_rot: i32,
        is_rev: i32,
        min_intron_len: i32,
        p: &[u8],
        off: &[i32],
        off_end: Option<&[i32]>,
        n_col: i32,
        i0: i32,
        j0: i32,
        m_cigar_: &mut i32,
        n_cigar_: &mut i32,
        cigar_: &mut Vec<u32>,
    ) {
        let mut n_cigar = 0;
        let mut m_cigar = *m_cigar_;
        let mut cigar = cigar_.clone();
        let mut i = i0;
        let mut j = j0;
        let mut state = 0i32;
        while i >= 0 && j >= 0 {
            let mut force_state = -1;
            let tmp = if is_rot != 0 {
                let r = i + j;
                if i < off[r as usize] {
                    force_state = 2;
                }
                if let Some(off_end) = off_end {
                    if i > off_end[r as usize] {
                        force_state = 1;
                    }
                }
                if force_state < 0 {
                    p[(r * n_col + i - off[r as usize]) as usize]
                } else {
                    0
                }
            } else {
                if j < off[i as usize] {
                    force_state = 2;
                }
                if let Some(off_end) = off_end {
                    if j > off_end[i as usize] {
                        force_state = 1;
                    }
                }
                if force_state < 0 {
                    p[(i * n_col + j - off[i as usize]) as usize]
                } else {
                    0
                }
            };
            if state == 0 {
                state = (tmp & 7) as i32;
            } else if ((tmp >> (state + 2)) & 1) == 0 {
                state = 0;
            }
            if state == 0 {
                state = (tmp & 7) as i32;
            }
            if force_state >= 0 {
                state = force_state;
            }
            if state == 0 {
                ksw_push_cigar(
                    km,
                    &mut n_cigar,
                    &mut m_cigar,
                    &mut cigar,
                    KSW_CIGAR_MATCH,
                    1,
                );
                i -= 1;
                j -= 1;
            } else if state == 1 || (state == 3 && min_intron_len <= 0) {
                ksw_push_cigar(km, &mut n_cigar, &mut m_cigar, &mut cigar, KSW_CIGAR_DEL, 1);
                i -= 1;
            } else if state == 3 && min_intron_len > 0 {
                ksw_push_cigar(
                    km,
                    &mut n_cigar,
                    &mut m_cigar,
                    &mut cigar,
                    KSW_CIGAR_N_SKIP,
                    1,
                );
                i -= 1;
            } else {
                ksw_push_cigar(km, &mut n_cigar, &mut m_cigar, &mut cigar, KSW_CIGAR_INS, 1);
                j -= 1;
            }
        }
        if i >= 0 {
            let op = if min_intron_len > 0 && i >= min_intron_len {
                KSW_CIGAR_N_SKIP
            } else {
                KSW_CIGAR_DEL
            };
            ksw_push_cigar(km, &mut n_cigar, &mut m_cigar, &mut cigar, op, i + 1);
        }
        if j >= 0 {
            ksw_push_cigar(
                km,
                &mut n_cigar,
                &mut m_cigar,
                &mut cigar,
                KSW_CIGAR_INS,
                j + 1,
            );
        }
        if is_rev == 0 {
            cigar[..n_cigar as usize].reverse();
        }
        *m_cigar_ = m_cigar;
        *n_cigar_ = n_cigar;
        *cigar_ = cigar;
    }

    /// Original C static function `ksw_reset_extz` from `minibwa/ksw2.h:174`.
    pub fn ksw_reset_extz(ez: &mut ksw_extz_t) {
        ez.max_q = -1;
        ez.max_t = -1;
        ez.mqe_t = -1;
        ez.mte_q = -1;
        ez.max = 0;
        ez.score = KSW_NEG_INF;
        ez.mqe = KSW_NEG_INF;
        ez.mte = KSW_NEG_INF;
        ez.n_cigar = 0;
        ez.zdropped = 0;
        ez.reach_end = 0;
    }

    /// Original C static function `ksw_apply_zdrop` from `minibwa/ksw2.h:181`.
    pub fn ksw_apply_zdrop(
        ez: &mut ksw_extz_t,
        is_rot: i32,
        h: i32,
        a: i32,
        b: i32,
        zdrop: i32,
        e: i8,
    ) -> i32 {
        let (r, t) = if is_rot != 0 { (a, b) } else { (a + b, a) };
        if h > ez.max as i32 {
            ez.max = h as u32;
            ez.max_t = t;
            ez.max_q = r - t;
        } else if t >= ez.max_t && r - t >= ez.max_q {
            let tl = t - ez.max_t;
            let ql = (r - t) - ez.max_q;
            let l = if tl > ql { tl - ql } else { ql - tl };
            if zdrop >= 0 && ez.max as i32 - h > zdrop + l * e as i32 {
                ez.zdropped = 1;
                return 1;
            }
        }
        0
    }

    /// Original C static function `ksw_gen_nt4_mat` from `minibwa/ksw2.h:199`.
    pub fn ksw_gen_nt4_mat(mat: &mut [i8; 25], a: i8, b: i8, b_ts: i8, b_ambi: i8) {
        let a = a.abs();
        let b = if b > 0 { -b } else { b };
        let b_ambi = if b_ambi > 0 { -b_ambi } else { b_ambi };
        for i in 0..4usize {
            for j in 0..4usize {
                mat[i * 5 + j] = if i == j { a } else { b };
            }
            mat[i * 5 + 4] = b_ambi;
        }
        for j in 0..5usize {
            mat[4 * 5 + j] = b_ambi;
        }
        if b_ts == 0 || b_ts == b {
            return;
        }
        let b_ts = if b_ts > 0 { -b_ts } else { b_ts };
        mat[2] = b_ts;
        mat[8] = b_ts;
        mat[10] = b_ts;
        mat[16] = b_ts;
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn cigar_push_merges_adjacent_ops() {
            let mut n = 0;
            let mut m = 0;
            let mut cigar = Vec::new();
            ksw_push_cigar((), &mut n, &mut m, &mut cigar, KSW_CIGAR_MATCH, 3);
            ksw_push_cigar((), &mut n, &mut m, &mut cigar, KSW_CIGAR_MATCH, 2);
            ksw_push_cigar((), &mut n, &mut m, &mut cigar, KSW_CIGAR_INS, 1);
            assert_eq!(n, 2);
            assert_eq!(
                cigar[..n as usize],
                [5 << 4 | KSW_CIGAR_MATCH, 1 << 4 | KSW_CIGAR_INS]
            );
        }

        #[test]
        fn reset_zdrop_and_nt4_matrix_follow_header_rules() {
            assert_eq!(
                (
                    KSW_EZ_SPLICE_FOR,
                    KSW_EZ_SPLICE_REV,
                    KSW_EZ_SPLICE_FLANK,
                    KSW_EZ_SPLICE_CMPLX,
                    KSW_EZ_SPLICE_SCORE,
                    KSW_SPSC_OFFSET,
                ),
                (0x100, 0x200, 0x400, 0x800, 0x1000, 64)
            );

            let mut ez = ksw_extz_t {
                score: 5,
                n_cigar: 2,
                zdropped: 1,
                ..Default::default()
            };
            ksw_reset_extz(&mut ez);
            assert_eq!(
                (ez.max_q, ez.score, ez.n_cigar, ez.zdropped),
                (-1, KSW_NEG_INF, 0, 0)
            );
            assert_eq!(ksw_apply_zdrop(&mut ez, 0, 10, 3, 4, 5, 2), 0);
            assert_eq!(ez.max, 10);
            assert_eq!(ksw_apply_zdrop(&mut ez, 0, 0, 8, 8, 5, 2), 1);

            let mut mat = [0i8; 25];
            ksw_gen_nt4_mat(&mut mat, 2, 4, 1, 3);
            assert_eq!(mat[0], 2);
            assert_eq!(mat[1], -4);
            assert_eq!(mat[2], -1);
            assert_eq!(mat[4], -3);
            assert_eq!(mat[24], -3);
        }

        #[test]
        fn backtrack_builds_and_reverses_cigar() {
            let p = [0u8; 4];
            let off = [0i32; 2];
            let off_end = [1i32; 2];
            let mut m = 0;
            let mut n = 0;
            let mut cigar = Vec::new();
            ksw_backtrack(
                (),
                0,
                0,
                0,
                &p,
                &off,
                Some(&off_end),
                2,
                1,
                1,
                &mut m,
                &mut n,
                &mut cigar,
            );
            assert_eq!(n, 1);
            assert_eq!(cigar[..n as usize], [2 << 4 | KSW_CIGAR_MATCH]);

            let p = [1u8, 0, 0, 0];
            ksw_backtrack(
                (),
                0,
                0,
                5,
                &p,
                &off,
                Some(&off_end),
                2,
                1,
                1,
                &mut m,
                &mut n,
                &mut cigar,
            );
            assert_eq!(
                cigar[..n as usize],
                [
                    1 << 4 | KSW_CIGAR_INS,
                    1 << 4 | KSW_CIGAR_DEL,
                    1 << 4 | KSW_CIGAR_MATCH
                ]
            );
        }
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod ksw2_c_sse {
    use crate::ksw2::ksw_extz_t;
    use crate::ksw2_ll_sse::ksw_llrst_t;
    use std::ffi::c_void;
    use std::ptr;
    use std::sync::OnceLock;

    #[repr(C)]
    #[derive(Default)]
    struct ksw_extz_raw_t {
        max_zdropped: u32,
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
        cigar: *mut u32,
    }

    #[repr(C)]
    #[derive(Default)]
    struct ksw_llrst_raw_t {
        score: i32,
        te: i32,
        qe: i32,
        score2: i32,
        te2: i32,
    }

    unsafe extern "C" {
        #[link_name = "ksw_extz2_sse"]
        fn c_ksw_extz2_sse(
            km: *mut c_void,
            qlen: i32,
            query: *const u8,
            tlen: i32,
            target: *const u8,
            m: i8,
            mat: *const i8,
            q: i8,
            e: i8,
            w: i32,
            zdrop: i32,
            end_bonus: i32,
            flag: i32,
            ez: *mut ksw_extz_raw_t,
        );

        #[link_name = "ksw_extd2_sse"]
        fn c_ksw_extd2_sse(
            km: *mut c_void,
            qlen: i32,
            query: *const u8,
            tlen: i32,
            target: *const u8,
            m: i8,
            mat: *const i8,
            q: i8,
            e: i8,
            q2: i8,
            e2: i8,
            w: i32,
            zdrop: i32,
            end_bonus: i32,
            flag: i32,
            ez: *mut ksw_extz_raw_t,
        );

        #[link_name = "ksw_ll_qinit"]
        fn c_ksw_ll_qinit(
            km: *mut c_void,
            size: i32,
            qlen: i32,
            query: *const u8,
            m: i32,
            mat: *const i8,
        ) -> *mut c_void;

        #[link_name = "ksw_ll_i16"]
        fn c_ksw_ll_i16(
            q: *mut c_void,
            tlen: i32,
            target: *const u8,
            gapo: i32,
            gape: i32,
            qe: *mut i32,
            te: *mut i32,
        ) -> i32;

        #[link_name = "ksw_ll_u8_core"]
        fn c_ksw_ll_u8_core(
            q: *mut c_void,
            tlen: i32,
            target: *const u8,
            gapo: i32,
            gape: i32,
            xtra: i32,
        ) -> ksw_llrst_raw_t;

        #[link_name = "ksw_ll_i16_core"]
        fn c_ksw_ll_i16_core(
            q: *mut c_void,
            tlen: i32,
            target: *const u8,
            gapo: i32,
            gape: i32,
            xtra: i32,
        ) -> ksw_llrst_raw_t;
    }

    #[inline(always)]
    fn use_c_ksw() -> bool {
        static USE_C_KSW: OnceLock<bool> = OnceLock::new();
        *USE_C_KSW.get_or_init(|| std::env::var_os("MINIBWA_RS_RUST_KSW").is_none())
    }

    #[inline(always)]
    unsafe fn raw_from_ez_cigar(ez: &mut ksw_extz_t) -> ksw_extz_raw_t {
        let mut cigar = std::mem::take(&mut ez.cigar);
        let cap = cigar.capacity();
        let ptr = if cap == 0 {
            ptr::null_mut()
        } else {
            cigar.as_mut_ptr()
        };
        std::mem::forget(cigar);
        ksw_extz_raw_t {
            m_cigar: cap as i32,
            cigar: ptr,
            ..Default::default()
        }
    }

    #[inline(always)]
    unsafe fn copy_raw_ez(raw: &mut ksw_extz_raw_t, ez: &mut ksw_extz_t) {
        ez.max = raw.max_zdropped & 0x7fff_ffff;
        ez.zdropped = raw.max_zdropped >> 31;
        ez.max_q = raw.max_q;
        ez.max_t = raw.max_t;
        ez.mqe = raw.mqe;
        ez.mqe_t = raw.mqe_t;
        ez.mte = raw.mte;
        ez.mte_q = raw.mte_q;
        ez.score = raw.score;
        ez.m_cigar = raw.m_cigar;
        ez.n_cigar = raw.n_cigar;
        ez.reach_end = raw.reach_end;
        ez.cigar = if raw.cigar.is_null() {
            Vec::new()
        } else {
            unsafe {
                Vec::from_raw_parts(
                    raw.cigar,
                    raw.n_cigar.max(0) as usize,
                    raw.m_cigar.max(0) as usize,
                )
            }
        };
        raw.cigar = ptr::null_mut();
        raw.m_cigar = 0;
        raw.n_cigar = 0;
    }

    #[allow(clippy::too_many_arguments)]
    #[inline(always)]
    pub fn maybe_extz2(
        qlen: i32,
        query: &[u8],
        tlen: i32,
        target: &[u8],
        m: i8,
        mat: &[i8],
        q: i8,
        e: i8,
        w: i32,
        zdrop: i32,
        end_bonus: i32,
        flag: i32,
        ez: &mut ksw_extz_t,
    ) -> bool {
        if !use_c_ksw() {
            return false;
        }
        let mut raw = unsafe { raw_from_ez_cigar(ez) };
        unsafe {
            c_ksw_extz2_sse(
                ptr::null_mut(),
                qlen,
                query.as_ptr(),
                tlen,
                target.as_ptr(),
                m,
                mat.as_ptr(),
                q,
                e,
                w,
                zdrop,
                end_bonus,
                flag,
                &mut raw,
            );
            copy_raw_ez(&mut raw, ez);
        }
        true
    }

    #[allow(clippy::too_many_arguments)]
    #[inline(always)]
    pub fn maybe_extd2(
        qlen: i32,
        query: &[u8],
        tlen: i32,
        target: &[u8],
        m: i8,
        mat: &[i8],
        q: i8,
        e: i8,
        q2: i8,
        e2: i8,
        w: i32,
        zdrop: i32,
        end_bonus: i32,
        flag: i32,
        ez: &mut ksw_extz_t,
    ) -> bool {
        if !use_c_ksw() {
            return false;
        }
        let mut raw = unsafe { raw_from_ez_cigar(ez) };
        unsafe {
            c_ksw_extd2_sse(
                ptr::null_mut(),
                qlen,
                query.as_ptr(),
                tlen,
                target.as_ptr(),
                m,
                mat.as_ptr(),
                q,
                e,
                q2,
                e2,
                w,
                zdrop,
                end_bonus,
                flag,
                &mut raw,
            );
            copy_raw_ez(&mut raw, ez);
        }
        true
    }

    #[inline(always)]
    pub fn maybe_ll_i16(
        qlen: i32,
        query: &[u8],
        mat: &[i8],
        tlen: i32,
        target: &[u8],
        gapo: i32,
        gape: i32,
        qe: &mut i32,
        te: &mut i32,
    ) -> Option<i32> {
        if !use_c_ksw() {
            return None;
        }
        unsafe {
            let q = c_ksw_ll_qinit(ptr::null_mut(), 2, qlen, query.as_ptr(), 5, mat.as_ptr());
            if q.is_null() {
                return None;
            }
            let score = c_ksw_ll_i16(q, tlen, target.as_ptr(), gapo, gape, qe, te);
            libc::free(q);
            Some(score)
        }
    }

    #[inline(always)]
    pub fn maybe_ll_core(
        size: i32,
        qlen: i32,
        query: &[u8],
        mat: &[i8],
        tlen: i32,
        target: &[u8],
        gapo: i32,
        gape: i32,
        xtra: i32,
    ) -> Option<ksw_llrst_t> {
        if !use_c_ksw() {
            return None;
        }
        unsafe {
            let q = c_ksw_ll_qinit(ptr::null_mut(), size, qlen, query.as_ptr(), 5, mat.as_ptr());
            if q.is_null() {
                return None;
            }
            let raw = if size <= 1 {
                c_ksw_ll_u8_core(q, tlen, target.as_ptr(), gapo, gape, xtra)
            } else {
                c_ksw_ll_i16_core(q, tlen, target.as_ptr(), gapo, gape, xtra)
            };
            libc::free(q);
            Some(ksw_llrst_t {
                score: raw.score,
                te: raw.te,
                qe: raw.qe,
                score2: raw.score2,
                te2: raw.te2,
            })
        }
    }
}

pub mod ksw2_extd2_sse {
    #![allow(unused_variables, dead_code, non_snake_case)]

    use crate::ksw2::{
        ksw_apply_zdrop, ksw_backtrack, ksw_extz_t, ksw_reset_extz, KSW_EZ_APPROX_DROP,
        KSW_EZ_APPROX_MAX, KSW_EZ_EXTZ_ONLY, KSW_EZ_GENERIC_SC, KSW_EZ_REV_CIGAR, KSW_EZ_RIGHT,
        KSW_EZ_SCORE_ONLY, KSW_NEG_INF,
    };
    use crate::s2n_lite;
    use std::cell::RefCell;
    use std::thread::LocalKey;

    thread_local! {
        static EXTD_U: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
        static EXTD_V: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
        static EXTD_X: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
        static EXTD_Y: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
        static EXTD_X2: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
        static EXTD_Y2: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
        static EXTD_S: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
        static EXTD_SF_QR: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
        static EXTD_H: RefCell<Vec<i32>> = const { RefCell::new(Vec::new()) };
        static EXTD_P: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
        static EXTD_OFF: RefCell<Vec<i32>> = const { RefCell::new(Vec::new()) };
        static EXTD_OFF_END: RefCell<Vec<i32>> = const { RefCell::new(Vec::new()) };
    }

    #[inline(always)]
    fn take_u8(key: &'static LocalKey<RefCell<Vec<u8>>>, len: usize, fill: u8) -> Vec<u8> {
        key.with(|cell| {
            let mut v = std::mem::take(&mut *cell.borrow_mut());
            v.clear();
            v.resize(len, fill);
            v
        })
    }

    #[inline(always)]
    fn put_u8(key: &'static LocalKey<RefCell<Vec<u8>>>, mut v: Vec<u8>) {
        v.clear();
        key.with(|cell| {
            *cell.borrow_mut() = v;
        });
    }

    #[inline(always)]
    fn take_i32(key: &'static LocalKey<RefCell<Vec<i32>>>, len: usize, fill: i32) -> Vec<i32> {
        key.with(|cell| {
            let mut v = std::mem::take(&mut *cell.borrow_mut());
            v.clear();
            v.resize(len, fill);
            v
        })
    }

    #[inline(always)]
    fn put_i32(key: &'static LocalKey<RefCell<Vec<i32>>>, mut v: Vec<i32>) {
        v.clear();
        key.with(|cell| {
            *cell.borrow_mut() = v;
        });
    }

    #[inline(always)]
    unsafe fn load16_at(slice: &[u8], offset: usize) -> [u8; 16] {
        let mut out = [0u8; 16];
        unsafe {
            std::ptr::copy_nonoverlapping(slice.as_ptr().add(offset), out.as_mut_ptr(), 16);
        }
        out
    }

    #[inline(always)]
    unsafe fn store16_at(slice: &mut [u8], offset: usize, value: [u8; 16]) {
        unsafe {
            std::ptr::copy_nonoverlapping(value.as_ptr(), slice.as_mut_ptr().add(offset), 16);
        }
    }

    /// Original C global function `ksw_extd2_sse` from `minibwa/ksw2_extd2_sse.c:16`.
    pub fn ksw_extd2_sse(
        km: (),
        qlen: i32,
        query: &[u8],
        tlen: i32,
        target: &[u8],
        m: i8,
        mat: &[i8],
        mut q: i8,
        mut e: i8,
        mut q2: i8,
        mut e2: i8,
        w: i32,
        zdrop: i32,
        end_bonus: i32,
        flag: i32,
        ez: &mut ksw_extz_t,
    ) {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if crate::ksw2_c_sse::maybe_extd2(
            qlen, query, tlen, target, m, mat, q, e, q2, e2, w, zdrop, end_bonus, flag, ez,
        ) {
            return;
        }
        // SIMD note: the byte-state DP and exact-max scans use the
        // native-backed s2n_lite shim.
        let h_qe = q as i32 + e as i32;
        if (q2 as i32) + (e2 as i32) < (q as i32) + (e as i32) {
            std::mem::swap(&mut q, &mut q2);
            std::mem::swap(&mut e, &mut e2);
        }
        ksw_reset_extz(ez);
        if m <= 1 || qlen <= 0 || tlen <= 0 {
            return;
        }
        let qlen = qlen as usize;
        let tlen = tlen as usize;
        let m = m as usize;
        let q_i32 = q as i32;
        let e_i32 = e as i32;
        let q2_i32 = q2 as i32;
        let e2_i32 = e2 as i32;
        let qe = q_i32 + e_i32;
        let qe2 = q2_i32 + e2_i32;
        let q_byte = q as u8;
        let q2_byte = q2 as u8;
        let qe_byte = qe as i8 as u8;
        let qe2_byte = qe2 as i8 as u8;
        let mut w = w;
        if w < 0 {
            w = qlen.max(tlen) as i32;
        }
        let mut max_sc = mat[0] as i32;
        let mut min_sc = mat[1] as i32;
        for &v in mat.iter().take(m * m).skip(1) {
            let v = v as i32;
            max_sc = max_sc.max(v);
            min_sc = min_sc.min(v);
        }
        if -min_sc > 2 * qe {
            return;
        }
        let mut long_thres = if e != e2 {
            (q2_i32 - q_i32) / (e_i32 - e2_i32) - 1
        } else {
            0
        };
        if q2_i32 + e2_i32 + long_thres * e2_i32 > q_i32 + e_i32 + long_thres * e_i32 {
            long_thres += 1;
        }
        let long_diff = long_thres * (e_i32 - e2_i32) - (q2_i32 - q_i32) - e2_i32;

        let tlen_ = tlen.div_ceil(16);
        let qlen_ = qlen.div_ceil(16);
        let n_vec_bytes = tlen_ * 16;
        let n_col_ = ((qlen.min(tlen).min((w + 1).max(0) as usize) + 15) / 16) + 1;
        let approx_max = (flag & KSW_EZ_APPROX_MAX) != 0;
        let with_cigar = (flag & KSW_EZ_SCORE_ONLY) == 0;
        let neg_qe = (-qe) as i8 as u8;
        let neg_qe2 = (-qe2) as i8 as u8;
        let mut u = take_u8(&EXTD_U, n_vec_bytes.max(1), neg_qe);
        let mut v = take_u8(&EXTD_V, n_vec_bytes.max(1), neg_qe);
        let mut x = take_u8(&EXTD_X, n_vec_bytes.max(1), neg_qe);
        let mut y = take_u8(&EXTD_Y, n_vec_bytes.max(1), neg_qe);
        let mut x2 = take_u8(&EXTD_X2, n_vec_bytes.max(1), neg_qe2);
        let mut y2 = take_u8(&EXTD_Y2, n_vec_bytes.max(1), neg_qe2);
        let mut s = take_u8(&EXTD_S, (n_vec_bytes + 16).max(1), 0);
        let qr_off = n_vec_bytes;
        let mut sf_qr = take_u8(&EXTD_SF_QR, n_vec_bytes + ((qlen_ + 1) * 16).max(1), 0);
        let mut h = if approx_max {
            take_i32(&EXTD_H, 0, KSW_NEG_INF)
        } else {
            take_i32(&EXTD_H, n_vec_bytes.max(1), KSW_NEG_INF)
        };
        let mut p = if with_cigar {
            take_u8(&EXTD_P, (qlen + tlen - 1) * n_col_ * 16, 0)
        } else {
            take_u8(&EXTD_P, 0, 0)
        };
        let mut off = if with_cigar {
            take_i32(&EXTD_OFF, qlen + tlen - 1, 0)
        } else {
            take_i32(&EXTD_OFF, 0, 0)
        };
        let mut off_end = if with_cigar {
            take_i32(&EXTD_OFF_END, qlen + tlen - 1, 0)
        } else {
            take_i32(&EXTD_OFF_END, 0, 0)
        };
        for t in 0..qlen {
            sf_qr[qr_off + t] = query[qlen - 1 - t];
        }
        sf_qr[..tlen].copy_from_slice(&target[..tlen]);

        let sc_mch = mat[0] as u8;
        let sc_mis = mat[1] as u8;
        let sc_n = if mat[m * m - 1] == 0 {
            (-e2_i32) as i8 as u8
        } else {
            mat[m * m - 1] as u8
        };
        let wildcard = (m - 1) as u8;
        let sc_mch_v = s2n_lite::_mm_set1_epi8(sc_mch as i32);
        let sc_mis_v = s2n_lite::_mm_set1_epi8(sc_mis as i32);
        let sc_n_v = s2n_lite::_mm_set1_epi8(sc_n as i32);
        let wildcard_v = s2n_lite::_mm_set1_epi8(wildcard as i32);
        let zero_v = s2n_lite::_mm_setzero_si128();
        let q_v = s2n_lite::_mm_set1_epi8(q_byte as i32);
        let q2_v = s2n_lite::_mm_set1_epi8(q2_byte as i32);
        let qe_v = s2n_lite::_mm_set1_epi8(qe_byte as i32);
        let qe2_v = s2n_lite::_mm_set1_epi8(qe2_byte as i32);
        let flag1_v = s2n_lite::_mm_set1_epi8(1);
        let flag2_v = s2n_lite::_mm_set1_epi8(2);
        let flag3_v = s2n_lite::_mm_set1_epi8(3);
        let flag4_v = s2n_lite::_mm_set1_epi8(4);
        let flag8_v = s2n_lite::_mm_set1_epi8(0x08);
        let flag16_v = s2n_lite::_mm_set1_epi8(0x10);
        let flag32_v = s2n_lite::_mm_set1_epi8(0x20);
        let flag64_v = s2n_lite::_mm_set1_epi8(0x40);
        let mut h0 = 0i32;
        let mut last_h0_t = 0i32;
        let mut last_st = -1i32;
        let mut last_en = -1i32;
        let wl = w;
        let wr = w;

        for r in 0..(qlen + tlen - 1) {
            let r_i32 = r as i32;
            let mut st = 0i32;
            let mut en = tlen as i32 - 1;
            if st < r_i32 - qlen as i32 + 1 {
                st = r_i32 - qlen as i32 + 1;
            }
            if en > r_i32 {
                en = r_i32;
            }
            if st < (r_i32 - wr + 1) >> 1 {
                st = (r_i32 - wr + 1) >> 1;
            }
            if en > (r_i32 + wl) >> 1 {
                en = (r_i32 + wl) >> 1;
            }
            if st > en {
                ez.zdropped = 1;
                break;
            }
            let st0 = st;
            let en0 = en;
            st = st / 16 * 16;
            en = (en + 16) / 16 * 16 - 1;

            let (x1, x21, v1) = if st > 0 {
                if st - 1 >= last_st && st - 1 <= last_en {
                    (
                        x[(st - 1) as usize],
                        x2[(st - 1) as usize],
                        v[(st - 1) as usize],
                    )
                } else {
                    (neg_qe, neg_qe2, neg_qe)
                }
            } else {
                let boundary = if r == 0 {
                    neg_qe
                } else if r_i32 < long_thres {
                    (-e_i32) as i8 as u8
                } else if r_i32 == long_thres {
                    long_diff as i8 as u8
                } else {
                    (-e2_i32) as i8 as u8
                };
                (neg_qe, neg_qe2, boundary)
            };
            if en >= r_i32 {
                y[r] = neg_qe;
                y2[r] = neg_qe2;
                u[r] = if r == 0 {
                    neg_qe
                } else if r_i32 < long_thres {
                    (-e_i32) as i8 as u8
                } else if r_i32 == long_thres {
                    long_diff as i8 as u8
                } else {
                    (-e2_i32) as i8 as u8
                };
            }

            if (flag & KSW_EZ_GENERIC_SC) == 0 {
                let mut score_t = st0;
                while score_t <= en0 {
                    let t_usize = score_t as usize;
                    let q_usize = (qr_off as i32 + qlen as i32 - 1 - r_i32 + score_t) as usize;
                    let sq = unsafe { load16_at(&sf_qr, t_usize) };
                    let stq = unsafe { load16_at(&sf_qr, q_usize) };
                    let eq = s2n_lite::_mm_cmpeq_epi8(sq, stq);
                    let sq_wild = s2n_lite::_mm_cmpeq_epi8(sq, wildcard_v);
                    let st_wild = s2n_lite::_mm_cmpeq_epi8(stq, wildcard_v);
                    let wild = s2n_lite::_mm_or_si128(sq_wild, st_wild);
                    let mut score = s2n_lite::_mm_blendv_epi8(sc_mis_v, sc_mch_v, eq);
                    score = s2n_lite::_mm_blendv_epi8(score, sc_n_v, wild);
                    unsafe { store16_at(&mut s, t_usize, score) };
                    score_t += 16;
                }
            } else {
                for t in st0..=en0 {
                    let q_base = qr_off as i32 + qlen as i32 - 1 - r_i32 + t;
                    s[t as usize] =
                        mat[sf_qr[t as usize] as usize * m + sf_qr[q_base as usize] as usize] as u8;
                }
            }

            let st_block = st / 16;
            let en_block = en / 16;
            debug_assert!(en_block - st_block + 1 <= n_col_ as i32);
            if with_cigar {
                off[r] = st;
                off_end[r] = en;
            }
            let mut x1_lane = x1;
            let mut x21_lane = x21;
            let mut v1_lane = v1;
            for block in st_block..=en_block {
                let base = block as usize * 16;
                let mut old_x = [0u8; 16];
                let mut old_y = [0u8; 16];
                let mut old_x2 = [0u8; 16];
                let mut old_y2 = [0u8; 16];
                let mut old_u = [0u8; 16];
                let mut old_v = [0u8; 16];
                old_x.copy_from_slice(&x[base..base + 16]);
                old_y.copy_from_slice(&y[base..base + 16]);
                old_x2.copy_from_slice(&x2[base..base + 16]);
                old_y2.copy_from_slice(&y2[base..base + 16]);
                old_u.copy_from_slice(&u[base..base + 16]);
                old_v.copy_from_slice(&v[base..base + 16]);
                let old_x_tail = old_x[15];
                let old_x2_tail = old_x2[15];
                let old_v_tail = old_v[15];
                {
                    let mut s_block = [0u8; 16];
                    s_block.copy_from_slice(&s[base..base + 16]);
                    let xt1 = s2n_lite::_mm_or_si128(
                        s2n_lite::_mm_slli_si128::<1>(old_x),
                        s2n_lite::_mm_cvtsi32_si128(x1_lane as i32),
                    );
                    let x2t1 = s2n_lite::_mm_or_si128(
                        s2n_lite::_mm_slli_si128::<1>(old_x2),
                        s2n_lite::_mm_cvtsi32_si128(x21_lane as i32),
                    );
                    let vt1 = s2n_lite::_mm_or_si128(
                        s2n_lite::_mm_slli_si128::<1>(old_v),
                        s2n_lite::_mm_cvtsi32_si128(v1_lane as i32),
                    );
                    let ut = old_u;
                    let mut z = s_block;
                    let mut a = s2n_lite::_mm_add_epi8(xt1, vt1);
                    let mut b = s2n_lite::_mm_add_epi8(old_y, ut);
                    let mut a2 = s2n_lite::_mm_add_epi8(x2t1, vt1);
                    let mut b2 = s2n_lite::_mm_add_epi8(old_y2, ut);
                    let mut d = zero_v;
                    if !with_cigar {
                        z = s2n_lite::_mm_max_epi8(z, a);
                        z = s2n_lite::_mm_max_epi8(z, b);
                        z = s2n_lite::_mm_max_epi8(z, a2);
                        z = s2n_lite::_mm_max_epi8(z, b2);
                    } else if (flag & KSW_EZ_RIGHT) == 0 {
                        d = s2n_lite::_mm_and_si128(s2n_lite::_mm_cmpgt_epi8(a, z), flag1_v);
                        z = s2n_lite::_mm_max_epi8(z, a);
                        let tmp = s2n_lite::_mm_cmpgt_epi8(b, z);
                        d = s2n_lite::_mm_blendv_epi8(d, flag2_v, tmp);
                        z = s2n_lite::_mm_max_epi8(z, b);
                        let tmp = s2n_lite::_mm_cmpgt_epi8(a2, z);
                        d = s2n_lite::_mm_blendv_epi8(d, flag3_v, tmp);
                        z = s2n_lite::_mm_max_epi8(z, a2);
                        let tmp = s2n_lite::_mm_cmpgt_epi8(b2, z);
                        d = s2n_lite::_mm_blendv_epi8(d, flag4_v, tmp);
                        z = s2n_lite::_mm_max_epi8(z, b2);
                    } else {
                        d = s2n_lite::_mm_andnot_si128(s2n_lite::_mm_cmpgt_epi8(z, a), flag1_v);
                        z = s2n_lite::_mm_max_epi8(z, a);
                        let tmp = s2n_lite::_mm_cmpgt_epi8(z, b);
                        d = s2n_lite::_mm_blendv_epi8(flag2_v, d, tmp);
                        z = s2n_lite::_mm_max_epi8(z, b);
                        let tmp = s2n_lite::_mm_cmpgt_epi8(z, a2);
                        d = s2n_lite::_mm_blendv_epi8(flag3_v, d, tmp);
                        z = s2n_lite::_mm_max_epi8(z, a2);
                        let tmp = s2n_lite::_mm_cmpgt_epi8(z, b2);
                        d = s2n_lite::_mm_blendv_epi8(flag4_v, d, tmp);
                        z = s2n_lite::_mm_max_epi8(z, b2);
                    }
                    z = s2n_lite::_mm_min_epi8(z, sc_mch_v);
                    u[base..base + 16].copy_from_slice(&s2n_lite::_mm_sub_epi8(z, vt1));
                    v[base..base + 16].copy_from_slice(&s2n_lite::_mm_sub_epi8(z, ut));
                    let z_gap = s2n_lite::_mm_sub_epi8(z, q_v);
                    let z_gap2 = s2n_lite::_mm_sub_epi8(z, q2_v);
                    a = s2n_lite::_mm_sub_epi8(a, z_gap);
                    b = s2n_lite::_mm_sub_epi8(b, z_gap);
                    a2 = s2n_lite::_mm_sub_epi8(a2, z_gap2);
                    b2 = s2n_lite::_mm_sub_epi8(b2, z_gap2);
                    if !with_cigar {
                        x[base..base + 16].copy_from_slice(&s2n_lite::_mm_sub_epi8(
                            s2n_lite::_mm_max_epi8(a, zero_v),
                            qe_v,
                        ));
                        y[base..base + 16].copy_from_slice(&s2n_lite::_mm_sub_epi8(
                            s2n_lite::_mm_max_epi8(b, zero_v),
                            qe_v,
                        ));
                        x2[base..base + 16].copy_from_slice(&s2n_lite::_mm_sub_epi8(
                            s2n_lite::_mm_max_epi8(a2, zero_v),
                            qe2_v,
                        ));
                        y2[base..base + 16].copy_from_slice(&s2n_lite::_mm_sub_epi8(
                            s2n_lite::_mm_max_epi8(b2, zero_v),
                            qe2_v,
                        ));
                        let _ = d;
                    } else if (flag & KSW_EZ_RIGHT) == 0 {
                        let tmp = s2n_lite::_mm_cmpgt_epi8(a, zero_v);
                        x[base..base + 16].copy_from_slice(&s2n_lite::_mm_sub_epi8(
                            s2n_lite::_mm_and_si128(tmp, a),
                            qe_v,
                        ));
                        d = s2n_lite::_mm_or_si128(d, s2n_lite::_mm_and_si128(tmp, flag8_v));
                        let tmp = s2n_lite::_mm_cmpgt_epi8(b, zero_v);
                        y[base..base + 16].copy_from_slice(&s2n_lite::_mm_sub_epi8(
                            s2n_lite::_mm_and_si128(tmp, b),
                            qe_v,
                        ));
                        d = s2n_lite::_mm_or_si128(d, s2n_lite::_mm_and_si128(tmp, flag16_v));
                        let tmp = s2n_lite::_mm_cmpgt_epi8(a2, zero_v);
                        x2[base..base + 16].copy_from_slice(&s2n_lite::_mm_sub_epi8(
                            s2n_lite::_mm_and_si128(tmp, a2),
                            qe2_v,
                        ));
                        d = s2n_lite::_mm_or_si128(d, s2n_lite::_mm_and_si128(tmp, flag32_v));
                        let tmp = s2n_lite::_mm_cmpgt_epi8(b2, zero_v);
                        y2[base..base + 16].copy_from_slice(&s2n_lite::_mm_sub_epi8(
                            s2n_lite::_mm_and_si128(tmp, b2),
                            qe2_v,
                        ));
                        d = s2n_lite::_mm_or_si128(d, s2n_lite::_mm_and_si128(tmp, flag64_v));
                        let p_idx = (r * n_col_ + block as usize - st_block as usize) * 16;
                        p[p_idx..p_idx + 16].copy_from_slice(&d);
                    } else {
                        let tmp = s2n_lite::_mm_cmpgt_epi8(zero_v, a);
                        x[base..base + 16].copy_from_slice(&s2n_lite::_mm_sub_epi8(
                            s2n_lite::_mm_andnot_si128(tmp, a),
                            qe_v,
                        ));
                        d = s2n_lite::_mm_or_si128(d, s2n_lite::_mm_andnot_si128(tmp, flag8_v));
                        let tmp = s2n_lite::_mm_cmpgt_epi8(zero_v, b);
                        y[base..base + 16].copy_from_slice(&s2n_lite::_mm_sub_epi8(
                            s2n_lite::_mm_andnot_si128(tmp, b),
                            qe_v,
                        ));
                        d = s2n_lite::_mm_or_si128(d, s2n_lite::_mm_andnot_si128(tmp, flag16_v));
                        let tmp = s2n_lite::_mm_cmpgt_epi8(zero_v, a2);
                        x2[base..base + 16].copy_from_slice(&s2n_lite::_mm_sub_epi8(
                            s2n_lite::_mm_andnot_si128(tmp, a2),
                            qe2_v,
                        ));
                        d = s2n_lite::_mm_or_si128(d, s2n_lite::_mm_andnot_si128(tmp, flag32_v));
                        let tmp = s2n_lite::_mm_cmpgt_epi8(zero_v, b2);
                        y2[base..base + 16].copy_from_slice(&s2n_lite::_mm_sub_epi8(
                            s2n_lite::_mm_andnot_si128(tmp, b2),
                            qe2_v,
                        ));
                        d = s2n_lite::_mm_or_si128(d, s2n_lite::_mm_andnot_si128(tmp, flag64_v));
                        let p_idx = (r * n_col_ + block as usize - st_block as usize) * 16;
                        p[p_idx..p_idx + 16].copy_from_slice(&d);
                    }
                    x1_lane = old_x_tail;
                    x21_lane = old_x2_tail;
                    v1_lane = old_v_tail;
                    continue;
                }
            }

            if !approx_max {
                let mut max_h_mut;
                let mut max_t_mut;
                if r > 0 {
                    if en0 > 0 {
                        h[en0 as usize] = h[(en0 - 1) as usize] + u[en0 as usize] as i8 as i32;
                    } else {
                        h[en0 as usize] += v[en0 as usize] as i8 as i32;
                    }
                    max_h_mut = h[en0 as usize];
                    max_t_mut = en0;
                    let en1 = st0 + (en0 - st0) / 4 * 4;
                    let mut max_h_v = s2n_lite::_mm_set1_epi32(max_h_mut);
                    let mut max_t_v = s2n_lite::_mm_set1_epi32(max_t_mut);
                    let mut t = st0;
                    while t < en1 {
                        let base = t as usize;
                        let mut h_v = [0u8; 16];
                        for lane in 0..4 {
                            h_v[lane * 4..lane * 4 + 4]
                                .copy_from_slice(&h[base + lane].to_le_bytes());
                        }
                        let v_v = s2n_lite::_mm_setr_epi32(
                            v[base] as i8 as i32,
                            v[base + 1] as i8 as i32,
                            v[base + 2] as i8 as i32,
                            v[base + 3] as i8 as i32,
                        );
                        h_v = s2n_lite::_mm_add_epi32(h_v, v_v);
                        for lane in 0..4 {
                            h[base + lane] = i32::from_le_bytes([
                                h_v[lane * 4],
                                h_v[lane * 4 + 1],
                                h_v[lane * 4 + 2],
                                h_v[lane * 4 + 3],
                            ]);
                        }
                        let t_v = s2n_lite::_mm_set1_epi32(t);
                        let gt = s2n_lite::_mm_cmpgt_epi32(h_v, max_h_v);
                        max_h_v = s2n_lite::_mm_blendv_epi8(max_h_v, h_v, gt);
                        max_t_v = s2n_lite::_mm_blendv_epi8(max_t_v, t_v, gt);
                        t += 4;
                    }
                    let mut hh = [0i32; 4];
                    let mut tt = [0i32; 4];
                    for lane in 0..4 {
                        hh[lane] = i32::from_le_bytes([
                            max_h_v[lane * 4],
                            max_h_v[lane * 4 + 1],
                            max_h_v[lane * 4 + 2],
                            max_h_v[lane * 4 + 3],
                        ]);
                        tt[lane] = i32::from_le_bytes([
                            max_t_v[lane * 4],
                            max_t_v[lane * 4 + 1],
                            max_t_v[lane * 4 + 2],
                            max_t_v[lane * 4 + 3],
                        ]);
                        if max_h_mut < hh[lane] {
                            max_h_mut = hh[lane];
                            max_t_mut = tt[lane] + lane as i32;
                        }
                    }
                    while t < en0 {
                        let k = t as usize;
                        h[k] += v[k] as i8 as i32;
                        if h[k] > max_h_mut {
                            max_h_mut = h[k];
                            max_t_mut = t;
                        }
                        t += 1;
                    }
                } else {
                    h[0] = v[0] as i8 as i32 - h_qe;
                    max_h_mut = h[0];
                    max_t_mut = 0;
                }
                if en0 == tlen as i32 - 1 && h[en0 as usize] > ez.mte {
                    ez.mte = h[en0 as usize];
                    ez.mte_q = r_i32 - en0;
                }
                if r_i32 - st0 == qlen as i32 - 1 && h[st0 as usize] > ez.mqe {
                    ez.mqe = h[st0 as usize];
                    ez.mqe_t = st0;
                }
                if ksw_apply_zdrop(ez, 1, max_h_mut, r_i32, max_t_mut, zdrop, e2) != 0 {
                    break;
                }
                if r == qlen + tlen - 2 && en0 == tlen as i32 - 1 {
                    ez.score = h[tlen - 1];
                }
            } else {
                if r > 0 {
                    if last_h0_t >= st0
                        && last_h0_t <= en0
                        && last_h0_t + 1 >= st0
                        && last_h0_t < en0
                    {
                        let d0 = v[last_h0_t as usize] as i8 as i32;
                        let d1 = u[(last_h0_t + 1) as usize] as i8 as i32;
                        if d0 > d1 {
                            h0 += d0;
                        } else {
                            h0 += d1;
                            last_h0_t += 1;
                        }
                    } else if last_h0_t >= st0 && last_h0_t <= en0 {
                        h0 += v[last_h0_t as usize] as i8 as i32;
                    } else {
                        last_h0_t += 1;
                        h0 += u[last_h0_t as usize] as i8 as i32;
                    }
                } else {
                    h0 = v[0] as i8 as i32 - h_qe;
                    last_h0_t = 0;
                }
                if (flag & KSW_EZ_APPROX_DROP) != 0
                    && ksw_apply_zdrop(ez, 1, h0, r_i32, last_h0_t, zdrop, e2) != 0
                {
                    break;
                }
                if r == qlen + tlen - 2 && en0 == tlen as i32 - 1 {
                    ez.score = h0;
                }
            }
            last_st = st;
            last_en = en;
        }

        if with_cigar {
            let rev_cigar = ((flag & KSW_EZ_REV_CIGAR) != 0) as i32;
            if ez.zdropped == 0 && (flag & KSW_EZ_EXTZ_ONLY) == 0 {
                ksw_backtrack(
                    km,
                    1,
                    rev_cigar,
                    0,
                    &p,
                    &off,
                    Some(&off_end),
                    (n_col_ * 16) as i32,
                    tlen as i32 - 1,
                    qlen as i32 - 1,
                    &mut ez.m_cigar,
                    &mut ez.n_cigar,
                    &mut ez.cigar,
                );
            } else if (flag & KSW_EZ_EXTZ_ONLY) != 0 && ez.mqe + end_bonus > ez.max as i32 {
                ez.reach_end = 1;
                ksw_backtrack(
                    km,
                    1,
                    rev_cigar,
                    0,
                    &p,
                    &off,
                    Some(&off_end),
                    (n_col_ * 16) as i32,
                    ez.mqe_t,
                    qlen as i32 - 1,
                    &mut ez.m_cigar,
                    &mut ez.n_cigar,
                    &mut ez.cigar,
                );
            } else if ez.max_t >= 0 && ez.max_q >= 0 {
                ksw_backtrack(
                    km,
                    1,
                    rev_cigar,
                    0,
                    &p,
                    &off,
                    Some(&off_end),
                    (n_col_ * 16) as i32,
                    ez.max_t,
                    ez.max_q,
                    &mut ez.m_cigar,
                    &mut ez.n_cigar,
                    &mut ez.cigar,
                );
            }
        }
        put_u8(&EXTD_U, u);
        put_u8(&EXTD_V, v);
        put_u8(&EXTD_X, x);
        put_u8(&EXTD_Y, y);
        put_u8(&EXTD_X2, x2);
        put_u8(&EXTD_Y2, y2);
        put_u8(&EXTD_S, s);
        put_u8(&EXTD_SF_QR, sf_qr);
        put_i32(&EXTD_H, h);
        put_u8(&EXTD_P, p);
        put_i32(&EXTD_OFF, off);
        put_i32(&EXTD_OFF_END, off_end);
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::ksw2::{
            KSW_CIGAR_DEL, KSW_EZ_APPROX_MAX, KSW_EZ_GENERIC_SC, KSW_EZ_SCORE_ONLY, KSW_NEG_INF,
        };

        #[test]
        fn extd_uses_second_affine_gap_for_long_deletion() {
            let mat = [
                2, -4, -4, -4, -1, -4, 2, -4, -4, -1, -4, -4, 2, -4, -1, -4, -4, -4, 2, -1, -1, -1,
                -1, -1, -1,
            ];
            let query = [0; 20];
            let target = [
                0, 0, 0, 0, 0, 0, 0, 0, 3, 3, 3, 3, 3, 3, 3, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ];
            let mut ez = ksw_extz_t::default();
            ksw_extd2_sse(
                (),
                query.len() as i32,
                &query,
                target.len() as i32,
                &target,
                5,
                &mat,
                1,
                4,
                7,
                1,
                50,
                100,
                -1,
                0,
                &mut ez,
            );
            assert_eq!(ez.score, 25);
            assert!(ez
                .cigar
                .iter()
                .take(ez.n_cigar as usize)
                .any(|op| (*op & 0xf) == KSW_CIGAR_DEL && (*op >> 4) >= 8));
        }

        #[test]
        fn extd_non_generic_scoring_treats_last_symbol_as_wildcard() {
            let mat = [
                2, -4, -4, -4, 0, -4, 2, -4, -4, 0, -4, -4, 2, -4, 0, -4, -4, -4, 2, 0, 0, 0, 0, 0,
                0,
            ];
            let query = [0, 4, 0];
            let target = [0, 4, 0];
            let mut ez = ksw_extz_t::default();
            ksw_extd2_sse(
                (),
                3,
                &query,
                3,
                &target,
                5,
                &mat,
                5,
                3,
                8,
                1,
                20,
                100,
                -1,
                KSW_EZ_SCORE_ONLY,
                &mut ez,
            );
            assert_eq!(ez.score, 3);

            ksw_extd2_sse(
                (),
                3,
                &query,
                3,
                &target,
                5,
                &mat,
                5,
                3,
                8,
                1,
                20,
                100,
                -1,
                KSW_EZ_SCORE_ONLY | KSW_EZ_GENERIC_SC,
                &mut ez,
            );
            assert_eq!(ez.score, 4);
        }

        #[test]
        fn extd_generic_scoring_matches_original_across_simd_blocks() {
            let mat = [
                2, -4, -4, -4, -1, -4, 2, -4, -4, -1, -4, -4, 2, -4, -1, -4, -4, -4, 2, -1, -1, -1,
                -1, -1, -1,
            ];
            let query = [
                0, 1, 2, 3, 0, 1, 2, 3, 4, 4, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 4, 2, 3, 0, 1, 2, 3, 0,
                1, 2, 4, 4, 0, 1, 2, 3, 0, 1, 2, 3,
            ];
            let target = [
                0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 4, 4, 4, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0,
                4, 1, 2, 3, 0, 1, 2, 3, 0,
            ];
            let mut ez = ksw_extz_t::default();
            ksw_extd2_sse(
                (),
                query.len() as i32,
                &query,
                target.len() as i32,
                &target,
                5,
                &mat,
                4,
                2,
                9,
                1,
                -1,
                30,
                1,
                KSW_EZ_GENERIC_SC,
                &mut ez,
            );
            assert_eq!((ez.max, ez.max_q, ez.max_t), (34, 39, 35));
            assert_eq!(
                (ez.mqe, ez.mqe_t, ez.mte, ez.mte_q, ez.score),
                (34, 35, 29, 36, 28)
            );
            assert_eq!(
                &ez.cigar[..ez.n_cigar as usize],
                &[128, 33, 64, 17, 80, 17, 304, 18]
            );
        }

        #[test]
        fn extd_zdrop_keeps_score_unset_when_bottom_right_not_reached() {
            let mat = [
                2, -4, -4, -4, -1, -4, 2, -4, -4, -1, -4, -4, 2, -4, -1, -4, -4, -4, 2, -1, -1, -1,
                -1, -1, -1,
            ];
            let query = [0, 0, 0, 0, 0, 0, 0, 0];
            let target = [0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1];
            let mut ez = ksw_extz_t::default();
            ksw_extd2_sse(
                (),
                query.len() as i32,
                &query,
                target.len() as i32,
                &target,
                5,
                &mat,
                5,
                1,
                7,
                1,
                50,
                0,
                -1,
                KSW_EZ_SCORE_ONLY,
                &mut ez,
            );
            assert_eq!(ez.zdropped, 1);
            assert_eq!(ez.score, KSW_NEG_INF);
        }

        #[test]
        fn extd_approx_max_without_approx_drop_suppresses_zdrop() {
            let mat = [
                2, -4, -4, -4, -1, -4, 2, -4, -4, -1, -4, -4, 2, -4, -1, -4, -4, -4, 2, -1, -1, -1,
                -1, -1, -1,
            ];
            let query = [0, 0, 0, 0, 0, 0, 0, 0];
            let target = [0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1];
            let mut ez = ksw_extz_t::default();
            ksw_extd2_sse(
                (),
                query.len() as i32,
                &query,
                target.len() as i32,
                &target,
                5,
                &mat,
                5,
                1,
                7,
                1,
                50,
                0,
                -1,
                KSW_EZ_SCORE_ONLY | KSW_EZ_APPROX_MAX,
                &mut ez,
            );
            assert_eq!(ez.zdropped, 0);
            assert_ne!(ez.score, KSW_NEG_INF);
            assert_eq!(ez.max, 0);
        }

        #[test]
        fn extd_marks_zdrop_when_band_has_no_cells() {
            let mat = [
                2, -4, -4, -4, -1, -4, 2, -4, -4, -1, -4, -4, 2, -4, -1, -4, -4, -4, 2, -1, -1, -1,
                -1, -1, -1,
            ];
            let query = [0, 0];
            let target = [0, 0, 0, 0];
            let mut ez = ksw_extz_t::default();
            ksw_extd2_sse(
                (),
                query.len() as i32,
                &query,
                target.len() as i32,
                &target,
                5,
                &mat,
                5,
                1,
                7,
                1,
                0,
                100,
                -1,
                KSW_EZ_SCORE_ONLY,
                &mut ez,
            );
            assert_eq!(ez.zdropped, 1);
            assert_eq!(ez.score, KSW_NEG_INF);
        }
    }
}

pub mod ksw2_extz2_sse {
    #![allow(unused_variables, dead_code, non_snake_case)]

    use crate::ksw2::{
        ksw_apply_zdrop, ksw_backtrack, ksw_extz_t, ksw_reset_extz, KSW_EZ_APPROX_DROP,
        KSW_EZ_APPROX_MAX, KSW_EZ_EXTZ_ONLY, KSW_EZ_GENERIC_SC, KSW_EZ_REV_CIGAR, KSW_EZ_RIGHT,
        KSW_EZ_SCORE_ONLY, KSW_NEG_INF,
    };
    use crate::s2n_lite;

    #[inline(always)]
    unsafe fn load16_at(slice: &[u8], offset: usize) -> [u8; 16] {
        let mut out = [0u8; 16];
        unsafe {
            std::ptr::copy_nonoverlapping(slice.as_ptr().add(offset), out.as_mut_ptr(), 16);
        }
        out
    }

    #[inline(always)]
    unsafe fn store16_at(slice: &mut [u8], offset: usize, value: [u8; 16]) {
        unsafe {
            std::ptr::copy_nonoverlapping(value.as_ptr(), slice.as_mut_ptr().add(offset), 16);
        }
    }

    /// Original C global function `ksw_extz2_sse` from `minibwa/ksw2_extz2_sse.c:16`.
    pub fn ksw_extz2_sse(
        km: (),
        qlen: i32,
        query: &[u8],
        tlen: i32,
        target: &[u8],
        m: i8,
        mat: &[i8],
        q: i8,
        e: i8,
        w: i32,
        zdrop: i32,
        end_bonus: i32,
        flag: i32,
        ez: &mut ksw_extz_t,
    ) {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if crate::ksw2_c_sse::maybe_extz2(
            qlen, query, tlen, target, m, mat, q, e, w, zdrop, end_bonus, flag, ez,
        ) {
            return;
        }
        // SIMD note: the byte-state DP and exact-max scans use the
        // native-backed s2n_lite shim.
        ksw_reset_extz(ez);
        if m <= 0 || qlen <= 0 || tlen <= 0 {
            return;
        }
        let qlen = qlen as usize;
        let tlen = tlen as usize;
        let m = m as usize;
        let q_i32 = q as i32;
        let e_i32 = e as i32;
        let qe = q_i32 + e_i32;
        let qe2 = ((q_i32 + e_i32) * 2) as i8 as u8;
        let q_byte = q as u8;
        let max_sc_byte = (mat[0] as i32 + (q_i32 + e_i32) * 2) as i8 as u8;
        let mut w = w;
        if w < 0 {
            w = qlen.max(tlen) as i32;
        }
        let mut max_sc = mat[0] as i32;
        let mut min_sc = mat[1] as i32;
        for &v in mat.iter().take(m * m).skip(1) {
            let v = v as i32;
            max_sc = max_sc.max(v);
            min_sc = min_sc.min(v);
        }
        if -min_sc > 2 * qe {
            return;
        }
        let tlen_ = tlen.div_ceil(16);
        let qlen_ = qlen.div_ceil(16);
        let n_vec_bytes = tlen_ * 16;
        let n_col_ = ((qlen.min(tlen).min((w + 1).max(0) as usize) + 15) / 16) + 1;
        let approx_max = (flag & KSW_EZ_APPROX_MAX) != 0;
        let with_cigar = (flag & KSW_EZ_SCORE_ONLY) == 0;
        let mut u = vec![0u8; n_vec_bytes.max(1)];
        let mut v = vec![0u8; n_vec_bytes.max(1)];
        let mut x = vec![0u8; n_vec_bytes.max(1)];
        let mut y = vec![0u8; n_vec_bytes.max(1)];
        let mut s = vec![0u8; (n_vec_bytes + 16).max(1)];
        let mut qr = vec![0u8; ((qlen_ + 1) * 16).max(1)];
        let mut sf = vec![0u8; (n_vec_bytes + 16).max(1)];
        let mut h = if approx_max {
            Vec::new()
        } else {
            vec![KSW_NEG_INF; n_vec_bytes.max(1)]
        };
        let mut p = if with_cigar {
            vec![0u8; (qlen + tlen - 1) * n_col_ * 16]
        } else {
            Vec::new()
        };
        let mut off = if with_cigar {
            vec![0i32; qlen + tlen - 1]
        } else {
            Vec::new()
        };
        let mut off_end = if with_cigar {
            vec![0i32; qlen + tlen - 1]
        } else {
            Vec::new()
        };
        for t in 0..qlen {
            qr[t] = query[qlen - 1 - t];
        }
        sf[..tlen].copy_from_slice(&target[..tlen]);

        let sc_mch = mat[0] as u8;
        let sc_mis = mat[1] as u8;
        let sc_n = if mat[m * m - 1] == 0 {
            (-e_i32) as i8 as u8
        } else {
            mat[m * m - 1] as u8
        };
        let wildcard = (m - 1) as u8;
        let sc_mch_v = s2n_lite::_mm_set1_epi8(sc_mch as i32);
        let sc_mis_v = s2n_lite::_mm_set1_epi8(sc_mis as i32);
        let sc_n_v = s2n_lite::_mm_set1_epi8(sc_n as i32);
        let wildcard_v = s2n_lite::_mm_set1_epi8(wildcard as i32);
        let zero_v = s2n_lite::_mm_setzero_si128();
        let q_v = s2n_lite::_mm_set1_epi8(q_byte as i32);
        let qe2_v = s2n_lite::_mm_set1_epi8(qe2 as i32);
        let max_sc_v = s2n_lite::_mm_set1_epi8(max_sc_byte as i32);
        let flag1_v = s2n_lite::_mm_set1_epi8(1);
        let flag2_v = s2n_lite::_mm_set1_epi8(2);
        let flag8_v = s2n_lite::_mm_set1_epi8(0x08);
        let flag16_v = s2n_lite::_mm_set1_epi8(0x10);
        let mut h0 = 0i32;
        let mut last_h0_t = 0i32;
        let mut last_st = -1i32;
        let mut last_en = -1i32;
        let wl = w;
        let wr = w;

        for r in 0..(qlen + tlen - 1) {
            let r_i32 = r as i32;
            let mut st = 0i32;
            let mut en = tlen as i32 - 1;
            if st < r_i32 - qlen as i32 + 1 {
                st = r_i32 - qlen as i32 + 1;
            }
            if en > r_i32 {
                en = r_i32;
            }
            if st < (r_i32 - wr + 1) >> 1 {
                st = (r_i32 - wr + 1) >> 1;
            }
            if en > (r_i32 + wl) >> 1 {
                en = (r_i32 + wl) >> 1;
            }
            if st > en {
                ez.zdropped = 1;
                break;
            }
            let st0 = st;
            let en0 = en;
            st = st / 16 * 16;
            en = (en + 16) / 16 * 16 - 1;

            let (x1, v1) = if st > 0 {
                if st - 1 >= last_st && st - 1 <= last_en {
                    (x[(st - 1) as usize], v[(st - 1) as usize])
                } else {
                    (0, 0)
                }
            } else if r != 0 {
                (0, q_byte)
            } else {
                (0, 0)
            };
            if en >= r_i32 {
                y[r] = 0;
                u[r] = if r != 0 { q_byte } else { 0 };
            }

            if (flag & KSW_EZ_GENERIC_SC) == 0 {
                let mut score_t = st0;
                while score_t <= en0 {
                    let t_usize = score_t as usize;
                    let q_usize = (qlen as i32 - 1 - r_i32 + score_t) as usize;
                    let sq = unsafe { load16_at(&sf, t_usize) };
                    let stq = unsafe { load16_at(&qr, q_usize) };
                    let eq = s2n_lite::_mm_cmpeq_epi8(sq, stq);
                    let sq_wild = s2n_lite::_mm_cmpeq_epi8(sq, wildcard_v);
                    let st_wild = s2n_lite::_mm_cmpeq_epi8(stq, wildcard_v);
                    let wild = s2n_lite::_mm_or_si128(sq_wild, st_wild);
                    let mut score = s2n_lite::_mm_blendv_epi8(sc_mis_v, sc_mch_v, eq);
                    score = s2n_lite::_mm_blendv_epi8(score, sc_n_v, wild);
                    unsafe { store16_at(&mut s, t_usize, score) };
                    score_t += 16;
                }
            } else {
                for t in st0..=en0 {
                    let q_base = qlen as i32 - 1 - r_i32 + t;
                    s[t as usize] =
                        mat[sf[t as usize] as usize * m + qr[q_base as usize] as usize] as u8;
                }
            }

            let st_block = st / 16;
            let en_block = en / 16;
            debug_assert!(en_block - st_block + 1 <= n_col_ as i32);
            if with_cigar {
                off[r] = st;
                off_end[r] = en;
            }
            let mut x1_lane = x1;
            let mut v1_lane = v1;
            for block in st_block..=en_block {
                let base = block as usize * 16;
                let old_x = unsafe { load16_at(&x, base) };
                let old_v = unsafe { load16_at(&v, base) };
                let old_u = unsafe { load16_at(&u, base) };
                let old_y = unsafe { load16_at(&y, base) };
                let old_x_tail = old_x[15];
                let old_v_tail = old_v[15];
                {
                    let s_block = unsafe { load16_at(&s, base) };
                    let xt1 = s2n_lite::_mm_or_si128(
                        s2n_lite::_mm_slli_si128::<1>(old_x),
                        s2n_lite::_mm_cvtsi32_si128(x1_lane as i32),
                    );
                    let vt1 = s2n_lite::_mm_or_si128(
                        s2n_lite::_mm_slli_si128::<1>(old_v),
                        s2n_lite::_mm_cvtsi32_si128(v1_lane as i32),
                    );
                    let ut = old_u;
                    let mut z = s2n_lite::_mm_add_epi8(s_block, qe2_v);
                    let mut a = s2n_lite::_mm_add_epi8(xt1, vt1);
                    let mut b = s2n_lite::_mm_add_epi8(old_y, ut);
                    let mut d;
                    if (flag & KSW_EZ_RIGHT) == 0 {
                        d = s2n_lite::_mm_and_si128(s2n_lite::_mm_cmpgt_epi8(a, z), flag1_v);
                        z = s2n_lite::_mm_max_epi8(z, a);
                        let tmp = s2n_lite::_mm_cmpgt_epi8(b, z);
                        d = s2n_lite::_mm_blendv_epi8(d, flag2_v, tmp);
                    } else {
                        d = s2n_lite::_mm_andnot_si128(s2n_lite::_mm_cmpgt_epi8(z, a), flag1_v);
                        z = s2n_lite::_mm_max_epi8(z, a);
                        let tmp = s2n_lite::_mm_cmpgt_epi8(z, b);
                        d = s2n_lite::_mm_blendv_epi8(flag2_v, d, tmp);
                    }
                    z = s2n_lite::_mm_max_epu8(z, b);
                    z = s2n_lite::_mm_min_epu8(z, max_sc_v);
                    unsafe { store16_at(&mut u, base, s2n_lite::_mm_sub_epi8(z, vt1)) };
                    unsafe { store16_at(&mut v, base, s2n_lite::_mm_sub_epi8(z, ut)) };
                    let z_gap = s2n_lite::_mm_sub_epi8(z, q_v);
                    a = s2n_lite::_mm_sub_epi8(a, z_gap);
                    b = s2n_lite::_mm_sub_epi8(b, z_gap);
                    if !with_cigar {
                        unsafe { store16_at(&mut x, base, s2n_lite::_mm_max_epi8(a, zero_v)) };
                        unsafe { store16_at(&mut y, base, s2n_lite::_mm_max_epi8(b, zero_v)) };
                    } else if (flag & KSW_EZ_RIGHT) == 0 {
                        let tmp = s2n_lite::_mm_cmpgt_epi8(a, zero_v);
                        unsafe { store16_at(&mut x, base, s2n_lite::_mm_and_si128(tmp, a)) };
                        d = s2n_lite::_mm_or_si128(d, s2n_lite::_mm_and_si128(tmp, flag8_v));
                        let tmp = s2n_lite::_mm_cmpgt_epi8(b, zero_v);
                        unsafe { store16_at(&mut y, base, s2n_lite::_mm_and_si128(tmp, b)) };
                        d = s2n_lite::_mm_or_si128(d, s2n_lite::_mm_and_si128(tmp, flag16_v));
                        let p_idx = (r * n_col_ + block as usize - st_block as usize) * 16;
                        unsafe { store16_at(&mut p, p_idx, d) };
                    } else {
                        let tmp = s2n_lite::_mm_cmpgt_epi8(zero_v, a);
                        unsafe { store16_at(&mut x, base, s2n_lite::_mm_andnot_si128(tmp, a)) };
                        d = s2n_lite::_mm_or_si128(d, s2n_lite::_mm_andnot_si128(tmp, flag8_v));
                        let tmp = s2n_lite::_mm_cmpgt_epi8(zero_v, b);
                        unsafe { store16_at(&mut y, base, s2n_lite::_mm_andnot_si128(tmp, b)) };
                        d = s2n_lite::_mm_or_si128(d, s2n_lite::_mm_andnot_si128(tmp, flag16_v));
                        let p_idx = (r * n_col_ + block as usize - st_block as usize) * 16;
                        unsafe { store16_at(&mut p, p_idx, d) };
                    }
                    x1_lane = old_x_tail;
                    v1_lane = old_v_tail;
                    continue;
                }
            }

            if !approx_max {
                let mut max_h_mut;
                let mut max_t_mut;
                if r > 0 {
                    if en0 > 0 {
                        h[en0 as usize] = h[(en0 - 1) as usize] + u[en0 as usize] as i32 - qe;
                    } else {
                        h[en0 as usize] += v[en0 as usize] as i32 - qe;
                    }
                    max_h_mut = h[en0 as usize];
                    max_t_mut = en0;
                    let en1 = st0 + (en0 - st0) / 4 * 4;
                    let mut max_h_v = s2n_lite::_mm_set1_epi32(max_h_mut);
                    let mut max_t_v = s2n_lite::_mm_set1_epi32(max_t_mut);
                    let qe_v = s2n_lite::_mm_set1_epi32(qe);
                    let mut t = st0;
                    while t < en1 {
                        let base = t as usize;
                        let mut h_v = [0u8; 16];
                        for lane in 0..4 {
                            h_v[lane * 4..lane * 4 + 4]
                                .copy_from_slice(&h[base + lane].to_le_bytes());
                        }
                        let v_v = s2n_lite::_mm_setr_epi32(
                            v[base] as i32,
                            v[base + 1] as i32,
                            v[base + 2] as i32,
                            v[base + 3] as i32,
                        );
                        h_v = s2n_lite::_mm_add_epi32(h_v, v_v);
                        h_v = s2n_lite::_mm_sub_epi32(h_v, qe_v);
                        for lane in 0..4 {
                            h[base + lane] = i32::from_le_bytes([
                                h_v[lane * 4],
                                h_v[lane * 4 + 1],
                                h_v[lane * 4 + 2],
                                h_v[lane * 4 + 3],
                            ]);
                        }
                        let t_v = s2n_lite::_mm_set1_epi32(t);
                        let gt = s2n_lite::_mm_cmpgt_epi32(h_v, max_h_v);
                        max_h_v = s2n_lite::_mm_blendv_epi8(max_h_v, h_v, gt);
                        max_t_v = s2n_lite::_mm_blendv_epi8(max_t_v, t_v, gt);
                        t += 4;
                    }
                    let mut hh = [0i32; 4];
                    let mut tt = [0i32; 4];
                    for lane in 0..4 {
                        hh[lane] = i32::from_le_bytes([
                            max_h_v[lane * 4],
                            max_h_v[lane * 4 + 1],
                            max_h_v[lane * 4 + 2],
                            max_h_v[lane * 4 + 3],
                        ]);
                        tt[lane] = i32::from_le_bytes([
                            max_t_v[lane * 4],
                            max_t_v[lane * 4 + 1],
                            max_t_v[lane * 4 + 2],
                            max_t_v[lane * 4 + 3],
                        ]);
                        if max_h_mut < hh[lane] {
                            max_h_mut = hh[lane];
                            max_t_mut = tt[lane] + lane as i32;
                        }
                    }
                    while t < en0 {
                        let k = t as usize;
                        h[k] += v[k] as i32 - qe;
                        if h[k] > max_h_mut {
                            max_h_mut = h[k];
                            max_t_mut = t;
                        }
                        t += 1;
                    }
                } else {
                    h[0] = v[0] as i32 - qe - qe;
                    max_h_mut = h[0];
                    max_t_mut = 0;
                }
                if en0 == tlen as i32 - 1 && h[en0 as usize] > ez.mte {
                    ez.mte = h[en0 as usize];
                    ez.mte_q = r_i32 - en0;
                }
                if r_i32 - st0 == qlen as i32 - 1 && h[st0 as usize] > ez.mqe {
                    ez.mqe = h[st0 as usize];
                    ez.mqe_t = st0;
                }
                if ksw_apply_zdrop(ez, 1, max_h_mut, r_i32, max_t_mut, zdrop, e) != 0 {
                    break;
                }
                if r == qlen + tlen - 2 && en0 == tlen as i32 - 1 {
                    ez.score = h[tlen - 1];
                }
            } else {
                if r > 0 {
                    if last_h0_t >= st0
                        && last_h0_t <= en0
                        && last_h0_t + 1 >= st0
                        && last_h0_t < en0
                    {
                        let d0 = v[last_h0_t as usize] as i32 - qe;
                        let d1 = u[(last_h0_t + 1) as usize] as i32 - qe;
                        if d0 > d1 {
                            h0 += d0;
                        } else {
                            h0 += d1;
                            last_h0_t += 1;
                        }
                    } else if last_h0_t >= st0 && last_h0_t <= en0 {
                        h0 += v[last_h0_t as usize] as i32 - qe;
                    } else {
                        last_h0_t += 1;
                        h0 += u[last_h0_t as usize] as i32 - qe;
                    }
                    if (flag & KSW_EZ_APPROX_DROP) != 0
                        && ksw_apply_zdrop(ez, 1, h0, r_i32, last_h0_t, zdrop, e) != 0
                    {
                        break;
                    }
                } else {
                    h0 = v[0] as i32 - qe - qe;
                    last_h0_t = 0;
                }
                if r == qlen + tlen - 2 && en0 == tlen as i32 - 1 {
                    ez.score = h0;
                }
            }
            last_st = st;
            last_en = en;
        }

        if with_cigar {
            let rev_cigar = ((flag & KSW_EZ_REV_CIGAR) != 0) as i32;
            if ez.zdropped == 0 && (flag & KSW_EZ_EXTZ_ONLY) == 0 {
                ksw_backtrack(
                    km,
                    1,
                    rev_cigar,
                    0,
                    &p,
                    &off,
                    Some(&off_end),
                    (n_col_ * 16) as i32,
                    tlen as i32 - 1,
                    qlen as i32 - 1,
                    &mut ez.m_cigar,
                    &mut ez.n_cigar,
                    &mut ez.cigar,
                );
            } else if (flag & KSW_EZ_EXTZ_ONLY) != 0 && ez.mqe + end_bonus > ez.max as i32 {
                ez.reach_end = 1;
                ksw_backtrack(
                    km,
                    1,
                    rev_cigar,
                    0,
                    &p,
                    &off,
                    Some(&off_end),
                    (n_col_ * 16) as i32,
                    ez.mqe_t,
                    qlen as i32 - 1,
                    &mut ez.m_cigar,
                    &mut ez.n_cigar,
                    &mut ez.cigar,
                );
            } else if ez.max_t >= 0 && ez.max_q >= 0 {
                ksw_backtrack(
                    km,
                    1,
                    rev_cigar,
                    0,
                    &p,
                    &off,
                    Some(&off_end),
                    (n_col_ * 16) as i32,
                    ez.max_t,
                    ez.max_q,
                    &mut ez.m_cigar,
                    &mut ez.n_cigar,
                    &mut ez.cigar,
                );
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::ksw2::{
            KSW_CIGAR_MATCH, KSW_EZ_APPROX_MAX, KSW_EZ_EXTZ_ONLY, KSW_EZ_GENERIC_SC,
            KSW_EZ_SCORE_ONLY, KSW_NEG_INF,
        };

        #[test]
        fn extz_aligns_exact_match_and_cigar() {
            let mat = [
                2, -4, -4, -4, -1, -4, 2, -4, -4, -1, -4, -4, 2, -4, -1, -4, -4, -4, 2, -1, -1, -1,
                -1, -1, -1,
            ];
            let mut ez = ksw_extz_t::default();
            ksw_extz2_sse(
                (),
                4,
                &[0, 1, 2, 3],
                4,
                &[0, 1, 2, 3],
                5,
                &mat,
                5,
                1,
                20,
                100,
                -1,
                0,
                &mut ez,
            );
            assert_eq!(ez.score, 8);
            assert_eq!(ez.max, 8);
            assert_eq!(ez.cigar[..ez.n_cigar as usize], [4 << 4 | KSW_CIGAR_MATCH]);
        }

        #[test]
        fn extz_only_can_stop_at_best_prefix() {
            let mat = [
                2, -4, -4, -4, -1, -4, 2, -4, -4, -1, -4, -4, 2, -4, -1, -4, -4, -4, 2, -1, -1, -1,
                -1, -1, -1,
            ];
            let mut ez = ksw_extz_t::default();
            ksw_extz2_sse(
                (),
                3,
                &[0, 1, 2],
                4,
                &[0, 1, 2, 3],
                5,
                &mat,
                5,
                1,
                20,
                100,
                1,
                KSW_EZ_EXTZ_ONLY,
                &mut ez,
            );
            assert_eq!(ez.max, 6);
            assert_eq!(ez.reach_end, 1);
            assert_eq!(ez.n_cigar, 1);
        }

        #[test]
        fn extz_non_generic_scoring_treats_last_symbol_as_wildcard() {
            let mat = [
                2, -4, -4, -4, 0, -4, 2, -4, -4, 0, -4, -4, 2, -4, 0, -4, -4, -4, 2, 0, 0, 0, 0, 0,
                0,
            ];
            let query = [0, 4, 0];
            let target = [0, 4, 0];
            let mut ez = ksw_extz_t::default();
            ksw_extz2_sse(
                (),
                3,
                &query,
                3,
                &target,
                5,
                &mat,
                5,
                1,
                20,
                100,
                -1,
                KSW_EZ_SCORE_ONLY,
                &mut ez,
            );
            assert_eq!(ez.score, 3);

            ksw_extz2_sse(
                (),
                3,
                &query,
                3,
                &target,
                5,
                &mat,
                5,
                1,
                20,
                100,
                -1,
                KSW_EZ_SCORE_ONLY | KSW_EZ_GENERIC_SC,
                &mut ez,
            );
            assert_eq!(ez.score, 4);
        }

        #[test]
        fn extz_generic_scoring_matches_original_across_simd_blocks() {
            let mat = [
                2, -4, -4, -4, -1, -4, 2, -4, -4, -1, -4, -4, 2, -4, -1, -4, -4, -4, 2, -1, -1, -1,
                -1, -1, -1,
            ];
            let query = [
                0, 1, 2, 3, 0, 1, 2, 3, 4, 4, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 4, 2, 3, 0, 1, 2, 3, 0,
                1, 2, 4, 4, 0, 1, 2, 3, 0, 1, 2, 3,
            ];
            let target = [
                0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 4, 4, 4, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0,
                4, 1, 2, 3, 0, 1, 2, 3, 0,
            ];
            let mut ez = ksw_extz_t::default();
            ksw_extz2_sse(
                (),
                query.len() as i32,
                &query,
                target.len() as i32,
                &target,
                5,
                &mat,
                4,
                2,
                -1,
                30,
                1,
                KSW_EZ_GENERIC_SC,
                &mut ez,
            );
            assert_eq!((ez.max, ez.max_q, ez.max_t), (34, 39, 35));
            assert_eq!(
                (ez.mqe, ez.mqe_t, ez.mte, ez.mte_q, ez.score),
                (34, 35, 29, 36, 28)
            );
            assert_eq!(
                &ez.cigar[..ez.n_cigar as usize],
                &[128, 33, 64, 17, 80, 17, 304, 18]
            );
        }

        #[test]
        fn extz_zdrop_keeps_score_unset_when_bottom_right_not_reached() {
            let mat = [
                2, -4, -4, -4, -1, -4, 2, -4, -4, -1, -4, -4, 2, -4, -1, -4, -4, -4, 2, -1, -1, -1,
                -1, -1, -1,
            ];
            let query = [0, 0, 0, 0, 0, 0, 0, 0];
            let target = [0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1];
            let mut ez = ksw_extz_t::default();
            ksw_extz2_sse(
                (),
                query.len() as i32,
                &query,
                target.len() as i32,
                &target,
                5,
                &mat,
                5,
                1,
                50,
                0,
                -1,
                KSW_EZ_SCORE_ONLY,
                &mut ez,
            );
            assert_eq!(ez.zdropped, 1);
            assert_eq!(ez.score, KSW_NEG_INF);
        }

        #[test]
        fn extz_approx_max_without_approx_drop_suppresses_zdrop() {
            let mat = [
                2, -4, -4, -4, -1, -4, 2, -4, -4, -1, -4, -4, 2, -4, -1, -4, -4, -4, 2, -1, -1, -1,
                -1, -1, -1,
            ];
            let query = [0, 0, 0, 0, 0, 0, 0, 0];
            let target = [0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1];
            let mut ez = ksw_extz_t::default();
            ksw_extz2_sse(
                (),
                query.len() as i32,
                &query,
                target.len() as i32,
                &target,
                5,
                &mat,
                5,
                1,
                50,
                0,
                -1,
                KSW_EZ_SCORE_ONLY | KSW_EZ_APPROX_MAX,
                &mut ez,
            );
            assert_eq!(ez.zdropped, 0);
            assert_ne!(ez.score, KSW_NEG_INF);
            assert_eq!(ez.max, 0);
        }

        #[test]
        fn extz_returns_empty_when_mismatch_penalty_breaks_byte_dp_assumption() {
            let mat = [
                2, -30, -30, -30, -1, -30, 2, -30, -30, -1, -30, -30, 2, -30, -1, -30, -30, -30, 2,
                -1, -1, -1, -1, -1, -1,
            ];
            let mut ez = ksw_extz_t::default();
            ksw_extz2_sse(
                (),
                4,
                &[0, 1, 2, 3],
                4,
                &[0, 1, 2, 3],
                5,
                &mat,
                5,
                1,
                20,
                100,
                -1,
                KSW_EZ_SCORE_ONLY,
                &mut ez,
            );
            assert_eq!((ez.score, ez.max), (KSW_NEG_INF, 0));
        }

        #[test]
        fn extz_marks_zdrop_when_band_has_no_cells() {
            let mat = [
                2, -4, -4, -4, -1, -4, 2, -4, -4, -1, -4, -4, 2, -4, -1, -4, -4, -4, 2, -1, -1, -1,
                -1, -1, -1,
            ];
            let query = [0, 0];
            let target = [0, 0, 0, 0];
            let mut ez = ksw_extz_t::default();
            ksw_extz2_sse(
                (),
                query.len() as i32,
                &query,
                target.len() as i32,
                &target,
                5,
                &mat,
                5,
                1,
                0,
                100,
                -1,
                KSW_EZ_SCORE_ONLY,
                &mut ez,
            );
            assert_eq!(ez.zdropped, 1);
            assert_eq!(ez.score, KSW_NEG_INF);
        }
    }
}

pub mod ksw2_ll_sse {
    #![allow(
        unused_variables,
        dead_code,
        non_snake_case,
        non_camel_case_types,
        unreachable_code
    )]

    use crate::ksw2::{KSW_LL_STOP, KSW_LL_SUBO};

    // The original ksw2_ll_sse.c segmented-vector core is translated through the
    // s2n_lite intrinsic shim below; x86/x86_64 uses native SSE2 for the mapped
    // vector operations, with scalar fallbacks for portability.

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct kswq_t {
        pub qlen: i32,
        pub slen: i32,
        pub shift: u8,
        pub mdiff: u8,
        pub max: u8,
        pub size: u8,
        pub query: Vec<u8>,
        pub mat: Vec<i8>,
        pub m: i32,
        pub qp: Vec<[u8; 16]>,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct ksw_llrst_t {
        pub score: i32,
        pub te: i32,
        pub qe: i32,
        pub score2: i32,
        pub te2: i32,
    }

    impl Default for ksw_llrst_t {
        fn default() -> Self {
            Self {
                score: 0,
                te: -1,
                qe: -1,
                score2: -1,
                te2: -1,
            }
        }
    }

    /// Original C global function `ksw_ll_qinit` from `minibwa/ksw2_ll_sse.c:50`.
    pub fn ksw_ll_qinit(km: (), size: i32, qlen: i32, query: &[u8], m: i32, mat: &[i8]) -> kswq_t {
        let size = if size > 1 { 2 } else { 1 };
        let p = 8 * (3 - size);
        let slen = (qlen + p - 1) / p;
        let tmp = (m * m) as usize;
        let mut shift = 127i8;
        let mut mdiff = 0i8;
        for &v in &mat[..tmp] {
            if v < shift {
                shift = v;
            }
            if v > mdiff {
                mdiff = v;
            }
        }
        let max = mdiff as u8;
        let shift_u8 = (256i32 - shift as i32) as u8;
        let mdiff_u8 = mdiff.wrapping_add(shift_u8 as i8) as u8;
        let mut qp = vec![[0u8; 16]; m as usize * slen as usize];
        if size == 1 {
            let nlen = slen * p;
            for a in 0..m {
                let ma = &mat[(a * m) as usize..];
                for i in 0..slen {
                    let mut lanes = [0u8; 16];
                    for lane in 0..16 {
                        let k = i + lane * slen;
                        let score = if k >= nlen || k >= qlen {
                            -1
                        } else {
                            ma[query[k as usize] as usize] as i32
                        };
                        lanes[lane as usize] = (score + shift_u8 as i32) as u8;
                    }
                    qp[(a * slen + i) as usize] = lanes;
                }
            }
        } else {
            let nlen = slen * p;
            for a in 0..m {
                let ma = &mat[(a * m) as usize..];
                for i in 0..slen {
                    let mut lanes = [0u8; 16];
                    for lane in 0..8 {
                        let k = i + lane * slen;
                        let score = if k >= nlen || k >= qlen {
                            -1
                        } else {
                            ma[query[k as usize] as usize] as i32
                        };
                        lanes[lane as usize * 2..lane as usize * 2 + 2]
                            .copy_from_slice(&(score as i16).to_le_bytes());
                    }
                    qp[(a * slen + i) as usize] = lanes;
                }
            }
        }
        kswq_t {
            qlen,
            slen,
            shift: shift_u8,
            mdiff: mdiff_u8,
            max,
            size: size as u8,
            query: query[..qlen as usize].to_vec(),
            mat: mat[..tmp].to_vec(),
            m,
            qp,
        }
    }

    /// Original C static function `ksw_le_u8` from `minibwa/ksw2_ll_sse.c:99`.
    pub fn ksw_le_u8(a: &[u8; 16], b: &[u8; 16]) -> i32 {
        let diff = crate::s2n_lite::_mm_subs_epu8(*a, *b);
        let eq = crate::s2n_lite::_mm_cmpeq_epi8(diff, crate::s2n_lite::_mm_setzero_si128());
        (crate::s2n_lite::_mm_movemask_epi8(eq) == 0xffff) as i32
    }

    /// Original C static function `ksw_max_u8` from `minibwa/ksw2_ll_sse.c:108`.
    pub fn ksw_max_u8(x: &[u8; 16]) -> i32 {
        let mut v = *x;
        v = crate::s2n_lite::_mm_max_epu8(v, crate::s2n_lite::_mm_srli_si128::<8>(v));
        v = crate::s2n_lite::_mm_max_epu8(v, crate::s2n_lite::_mm_srli_si128::<4>(v));
        v = crate::s2n_lite::_mm_max_epu8(v, crate::s2n_lite::_mm_srli_si128::<2>(v));
        v = crate::s2n_lite::_mm_max_epu8(v, crate::s2n_lite::_mm_srli_si128::<1>(v));
        crate::s2n_lite::_mm_extract_epi16::<0>(v) & 0x00ff
    }

    /// Original C global function `ksw_ll_u8_core` from `minibwa/ksw2_ll_sse.c:121`.
    pub fn ksw_ll_u8_core(
        q: &kswq_t,
        tlen: i32,
        target: &[u8],
        _gapo: i32,
        _gape: i32,
        xtra: i32,
    ) -> ksw_llrst_t {
        let qlen = q.qlen as usize;
        let slen = q.slen.max(0) as usize;
        let tlen = tlen.max(0) as usize;
        let gapoe = crate::s2n_lite::_mm_set1_epi8(_gapo + _gape);
        let gape = crate::s2n_lite::_mm_set1_epi8(_gape);
        let shift = crate::s2n_lite::_mm_set1_epi8(q.shift as i32);
        let minsc = if (xtra & KSW_LL_SUBO) != 0 {
            xtra & 0xffff
        } else {
            0x10000
        };
        let endsc = if (xtra & KSW_LL_STOP) != 0 {
            xtra & 0xffff
        } else {
            0x10000
        };
        let mut r = ksw_llrst_t::default();
        let mut h0 = vec![[0u8; 16]; slen];
        let mut h1 = vec![[0u8; 16]; slen];
        let mut e_vec = vec![[0u8; 16]; slen];
        let mut hmax = vec![[0u8; 16]; slen];
        let mut b = Vec::<u64>::new();
        let mut gmax = 0u8;
        let mut te = -1i32;
        for i in 0..tlen {
            let mut f = crate::s2n_lite::_mm_setzero_si128();
            let mut max = crate::s2n_lite::_mm_setzero_si128();
            let s_base = target[i] as usize * slen;
            let mut h = if slen == 0 {
                crate::s2n_lite::_mm_setzero_si128()
            } else {
                h0[slen - 1]
            };
            h = crate::s2n_lite::_mm_slli_si128::<1>(h);
            for j in 0..slen {
                h = crate::s2n_lite::_mm_adds_epu8(h, q.qp[s_base + j]);
                h = crate::s2n_lite::_mm_subs_epu8(h, shift);
                let mut e = e_vec[j];
                h = crate::s2n_lite::_mm_max_epu8(h, e);
                h = crate::s2n_lite::_mm_max_epu8(h, f);
                max = crate::s2n_lite::_mm_max_epu8(max, h);
                h1[j] = h;
                e = crate::s2n_lite::_mm_subs_epu8(e, gape);
                let mut t = crate::s2n_lite::_mm_subs_epu8(h, gapoe);
                e = crate::s2n_lite::_mm_max_epu8(e, t);
                e_vec[j] = e;
                f = crate::s2n_lite::_mm_subs_epu8(f, gape);
                t = crate::s2n_lite::_mm_subs_epu8(h, gapoe);
                f = crate::s2n_lite::_mm_max_epu8(f, t);
                h = h0[j];
            }
            'lazy_f_u8: for _k in 0..16 {
                f = crate::s2n_lite::_mm_slli_si128::<1>(f);
                for h1_j in h1.iter_mut().take(slen) {
                    h = *h1_j;
                    h = crate::s2n_lite::_mm_max_epu8(h, f);
                    *h1_j = h;
                    h = crate::s2n_lite::_mm_subs_epu8(h, gapoe);
                    f = crate::s2n_lite::_mm_subs_epu8(f, gape);
                    if ksw_le_u8(&f, &h) != 0 {
                        break 'lazy_f_u8;
                    }
                }
            }
            let row_max = ksw_max_u8(&max) as u8;
            if row_max as i32 >= minsc {
                if b.last().map_or(true, |&v| (v as i32) + 1 != i as i32) {
                    b.push(((row_max as u64) << 32) | i as u64);
                } else if ((b[b.len() - 1] >> 32) as i32) < row_max as i32 {
                    let last = b.len() - 1;
                    b[last] = ((row_max as u64) << 32) | i as u64;
                }
            }
            if row_max > gmax {
                gmax = row_max;
                te = i as i32;
                hmax.copy_from_slice(&h1);
                if gmax as i32 + q.shift as i32 >= 255 || gmax as i32 >= endsc {
                    break;
                }
            }
            std::mem::swap(&mut h0, &mut h1);
        }
        r.score = if gmax as i32 + (q.shift as i32) < 255 {
            gmax as i32
        } else {
            255
        };
        r.te = te;
        if r.score != 255 {
            let mut max = -1i32;
            for i in 0..(slen * 16) {
                let tmp = (i / 16 + i % 16 * slen) as i32;
                if tmp >= qlen as i32 {
                    continue;
                }
                let h = hmax[i / 16][i % 16] as i32;
                if h > max {
                    max = h;
                    r.qe = tmp;
                } else if h == max && tmp < r.qe {
                    r.qe = tmp;
                }
            }
            if !b.is_empty() && q.max != 0 {
                let radius = (r.score + q.max as i32 - 1) / q.max as i32;
                let low = te - radius;
                let high = te + radius;
                for v in b {
                    let epos = v as i32;
                    let score = (v >> 32) as i32;
                    if (epos < low || epos > high) && score > r.score2 {
                        r.score2 = score;
                        r.te2 = epos;
                    }
                }
            }
        }
        r
    }

    /// Original C static function `ksw_le_epi16` from `minibwa/ksw2_ll_sse.c:224`.
    pub fn ksw_le_epi16(a: [u8; 16], b: [u8; 16]) -> i32 {
        let gt = crate::s2n_lite::_mm_cmpgt_epi16(a, b);
        (crate::s2n_lite::_mm_movemask_epi8(gt) == 0) as i32
    }

    /// Original C static function `ksw_max_i16` from `minibwa/ksw2_ll_sse.c:233`.
    pub fn ksw_max_i16(x: [u8; 16]) -> i32 {
        let mut v = x;
        v = crate::s2n_lite::_mm_max_epi16(v, crate::s2n_lite::_mm_srli_si128::<8>(v));
        v = crate::s2n_lite::_mm_max_epi16(v, crate::s2n_lite::_mm_srli_si128::<4>(v));
        v = crate::s2n_lite::_mm_max_epi16(v, crate::s2n_lite::_mm_srli_si128::<2>(v));
        crate::s2n_lite::_mm_extract_epi16::<0>(v) as i16 as i32
    }

    /// Original C global function `ksw_ll_i16_core` from `minibwa/ksw2_ll_sse.c:245`.
    pub fn ksw_ll_i16_core(
        q: &kswq_t,
        tlen: i32,
        target: &[u8],
        _gapo: i32,
        _gape: i32,
        xtra: i32,
    ) -> ksw_llrst_t {
        let qlen = q.qlen.max(0) as usize;
        let slen = q.slen.max(0) as usize;
        let tlen = tlen.max(0) as usize;
        let gapoe = crate::s2n_lite::_mm_set1_epi16(_gapo + _gape);
        let gape = crate::s2n_lite::_mm_set1_epi16(_gape);
        let minsc = if (xtra & KSW_LL_SUBO) != 0 {
            xtra & 0xffff
        } else {
            0x10000
        };
        let endsc = if (xtra & KSW_LL_STOP) != 0 {
            xtra & 0xffff
        } else {
            0x10000
        };
        let mut r = ksw_llrst_t::default();
        let mut h0 = vec![[0u8; 16]; slen];
        let mut h1 = vec![[0u8; 16]; slen];
        let mut e_vec = vec![[0u8; 16]; slen];
        let mut hmax = vec![[0u8; 16]; slen];
        let mut b = Vec::<u64>::new();
        let mut gmax = 0i32;
        let mut te = -1i32;
        for i in 0..tlen {
            let mut f = crate::s2n_lite::_mm_setzero_si128();
            let mut max = crate::s2n_lite::_mm_setzero_si128();
            let s_base = target[i] as usize * slen;
            let mut h = if slen == 0 {
                crate::s2n_lite::_mm_setzero_si128()
            } else {
                h0[slen - 1]
            };
            h = crate::s2n_lite::_mm_slli_si128::<2>(h);
            for j in 0..slen {
                h = crate::s2n_lite::_mm_adds_epi16(h, q.qp[s_base + j]);
                let mut e = e_vec[j];
                h = crate::s2n_lite::_mm_max_epi16(h, e);
                h = crate::s2n_lite::_mm_max_epi16(h, f);
                max = crate::s2n_lite::_mm_max_epi16(max, h);
                h1[j] = h;
                e = crate::s2n_lite::_mm_subs_epu16(e, gape);
                let mut t = crate::s2n_lite::_mm_subs_epu16(h, gapoe);
                e = crate::s2n_lite::_mm_max_epi16(e, t);
                e_vec[j] = e;
                f = crate::s2n_lite::_mm_subs_epu16(f, gape);
                t = crate::s2n_lite::_mm_subs_epu16(h, gapoe);
                f = crate::s2n_lite::_mm_max_epi16(f, t);
                h = h0[j];
            }
            'lazy_f_i16: for _k in 0..16 {
                f = crate::s2n_lite::_mm_slli_si128::<2>(f);
                for h1_j in h1.iter_mut().take(slen) {
                    h = *h1_j;
                    h = crate::s2n_lite::_mm_max_epi16(h, f);
                    *h1_j = h;
                    h = crate::s2n_lite::_mm_subs_epu16(h, gapoe);
                    f = crate::s2n_lite::_mm_subs_epu16(f, gape);
                    if ksw_le_epi16(f, h) != 0 {
                        break 'lazy_f_i16;
                    }
                }
            }
            let row_max = ksw_max_i16(max);
            if row_max >= minsc {
                if b.last().map_or(true, |&v| (v as i32) + 1 != i as i32) {
                    b.push(((row_max as u64) << 32) | i as u64);
                } else if ((b[b.len() - 1] >> 32) as i32) < row_max {
                    let last = b.len() - 1;
                    b[last] = ((row_max as u64) << 32) | i as u64;
                }
            }
            if row_max > gmax {
                gmax = row_max;
                te = i as i32;
                hmax.copy_from_slice(&h1);
                if gmax >= endsc {
                    break;
                }
            }
            std::mem::swap(&mut h0, &mut h1);
        }
        r.score = gmax;
        r.te = te;
        let mut max = -1i32;
        for i in 0..(slen * 8) {
            let tmp = (i / 8 + i % 8 * slen) as i32;
            if tmp >= qlen as i32 {
                continue;
            }
            let lane = i % 8;
            let h = u16::from_le_bytes([hmax[i / 8][lane * 2], hmax[i / 8][lane * 2 + 1]]) as i32;
            if h > max {
                max = h;
                r.qe = tmp;
            } else if h == max && tmp < r.qe {
                r.qe = tmp;
            }
        }
        if !b.is_empty() {
            let radius = (r.score + q.max as i32 - 1) / q.max as i32;
            let low = te - radius;
            let high = te + radius;
            for v in b {
                let epos = v as i32;
                let score = (v >> 32) as i32;
                if (epos < low || epos > high) && score > r.score2 {
                    r.score2 = score;
                    r.te2 = epos;
                }
            }
        }
        r
    }

    /// Original C global function `ksw_ll_i16` from `minibwa/ksw2_ll_sse.c:335`.
    pub fn ksw_ll_i16(
        q: &kswq_t,
        tlen: i32,
        target: &[u8],
        _gapo: i32,
        _gape: i32,
        qe: &mut i32,
        te: &mut i32,
    ) -> i32 {
        let r = ksw_ll_i16_core(q, tlen, target, _gapo, _gape, 0);
        *qe = r.qe;
        *te = r.te;
        r.score
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn ll_query_init_keeps_header_fields() {
            let mat = [
                2, -3, -3, -3, -1, -3, 2, -3, -3, -1, -3, -3, 2, -3, -1, -3, -3, -3, 2, -1, -1, -1,
                -1, -1, -1,
            ];
            let q = ksw_ll_qinit((), 2, 4, &[0, 1, 2, 3], 5, &mat);
            assert_eq!((q.size, q.slen, q.shift, q.max), (2, 1, 259u16 as u8, 2));
            assert_eq!(q.mdiff, 5);
        }

        #[test]
        fn ll_i16_core_matches_simple_sw_result() {
            let mat = [
                2, -3, -3, -3, -1, -3, 2, -3, -3, -1, -3, -3, 2, -3, -1, -3, -3, -3, 2, -1, -1, -1,
                -1, -1, -1,
            ];
            let q = ksw_ll_qinit((), 2, 4, &[0, 1, 2, 3], 5, &mat);
            let mut qe = -1;
            let mut te = -1;
            let score = ksw_ll_i16(&q, 4, &[0, 1, 2, 3], 5, 1, &mut qe, &mut te);
            assert_eq!((score, qe, te), (8, 3, 3));

            let r = ksw_ll_i16_core(&q, 4, &[0, 1, 4, 3], 5, 1, 0);
            assert_eq!((r.score, r.qe, r.te), (5, 3, 3));
        }

        #[test]
        fn ll_u8_core_uses_unsigned_saturation_cutoff() {
            let mat = [
                5, -4, -4, -4, -1, -4, 5, -4, -4, -1, -4, -4, 5, -4, -1, -4, -4, -4, 5, -1, -1, -1,
                -1, -1, -1,
            ];
            let query = vec![0u8; 80];
            let target = vec![0u8; 80];
            let q = ksw_ll_qinit((), 1, query.len() as i32, &query, 5, &mat);
            let r = ksw_ll_u8_core(&q, target.len() as i32, &target, 5, 1, 0);
            assert_eq!(r.score, 255);
            assert_eq!(r.qe, -1);
            assert!(r.te >= 0);

            let q = ksw_ll_qinit((), 1, 4, &[0, 1, 2, 3], 5, &mat);
            let r = ksw_ll_u8_core(&q, 4, &[0, 1, 2, 3], 5, 1, 0);
            assert_eq!((r.score, r.qe, r.te), (20, 3, 3));
        }

        #[test]
        fn vector_predicates_match_intrinsic_intent() {
            let pack_i16 = |lanes: [i16; 8]| {
                let mut out = [0u8; 16];
                for (i, lane) in lanes.iter().enumerate() {
                    out[i * 2..i * 2 + 2].copy_from_slice(&lane.to_le_bytes());
                }
                out
            };

            assert_eq!(ksw_le_u8(&[1; 16], &[2; 16]), 1);
            assert_eq!(ksw_le_u8(&[3; 16], &[2; 16]), 0);
            assert_eq!(
                ksw_max_u8(&[1, 7, 2, 3, 4, 5, 6, 0, 1, 1, 1, 1, 1, 1, 1, 1]),
                7
            );
            assert_eq!(ksw_le_epi16(pack_i16([1; 8]), pack_i16([1; 8])), 1);
            assert_eq!(ksw_le_epi16(pack_i16([-2; 8]), pack_i16([-1; 8])), 1);
            assert_eq!(ksw_le_epi16(pack_i16([-1; 8]), pack_i16([-2; 8])), 0);
            assert_eq!(ksw_max_i16(pack_i16([1, -2, 9, 3, 4, 5, 6, 7])), 9);
            assert_eq!(ksw_max_i16(pack_i16([-12, -2, -9, -3, -4, -5, -6, -7])), -2);
        }
    }
}

pub mod kthread {
    #![allow(unused_variables, dead_code, non_snake_case, non_camel_case_types)]

    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    pub struct ktf_worker_t {
        pub i: i64,
    }

    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    pub struct kt_for_t {
        pub n_threads: i32,
        pub n: i64,
        pub w: Vec<ktf_worker_t>,
    }

    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    pub struct ktp_worker_t<T> {
        pub index: i64,
        pub step: i32,
        pub data: Option<T>,
    }

    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    pub struct ktp_t<T> {
        pub index: i64,
        pub n_workers: i32,
        pub n_steps: i32,
        pub workers: Vec<ktp_worker_t<T>>,
    }

    /// Original C static function `steal_work` from `minibwa/kthread.c:30`.
    pub fn steal_work(t: &mut kt_for_t) -> i64 {
        let mut min_i = -1i32;
        let mut min = i64::MAX;
        for i in 0..t.n_threads as usize {
            if min > t.w[i].i {
                min = t.w[i].i;
                min_i = i as i32;
            }
        }
        let k = t.w[min_i as usize].i;
        t.w[min_i as usize].i += t.n_threads as i64;
        if k >= t.n {
            -1
        } else {
            k
        }
    }

    /// Original C static function `ktf_worker` from `minibwa/kthread.c:40`.
    pub fn ktf_worker<F>(t: &mut kt_for_t, worker_id: i32, func: &mut F)
    where
        F: FnMut(i64, i32),
    {
        loop {
            let i = t.w[worker_id as usize].i;
            t.w[worker_id as usize].i += t.n_threads as i64;
            if i >= t.n {
                break;
            }
            func(i, worker_id);
        }
        loop {
            let i = steal_work(t);
            if i < 0 {
                break;
            }
            func(i, worker_id);
        }
    }

    /// Original C global function `kt_for` from `minibwa/kthread.c:54`.
    pub fn kt_for<F>(n_threads: i32, mut func: F, n: i64)
    where
        F: FnMut(i64, i32),
    {
        if n_threads > 1 {
            let mut t = kt_for_t {
                n_threads,
                n,
                w: (0..n_threads)
                    .map(|i| ktf_worker_t { i: i as i64 })
                    .collect(),
            };
            for i in 0..n_threads {
                ktf_worker(&mut t, i, &mut func);
            }
        } else {
            for j in 0..n {
                func(j, 0);
            }
        }
    }

    /// Original C static function `ktp_worker` from `minibwa/kthread.c:97`.
    pub fn ktp_worker<S, T, F>(p: &mut ktp_t<T>, worker_id: i32, shared: &mut S, func: &mut F)
    where
        F: FnMut(&mut S, i32, Option<T>) -> Option<T>,
    {
        let wid = worker_id as usize;
        while p.workers[wid].step < p.n_steps {
            let step = p.workers[wid].step;
            let input = if step != 0 {
                p.workers[wid].data.take()
            } else {
                None
            };
            p.workers[wid].data = func(shared, step, input);
            if step == p.n_steps - 1 || p.workers[wid].data.is_some() {
                p.workers[wid].step = (step + 1) % p.n_steps;
            } else {
                p.workers[wid].step = p.n_steps;
            }
            if p.workers[wid].step == 0 {
                p.workers[wid].index = p.index;
                p.index += 1;
            }
        }
    }

    /// Original C global function `kt_pipeline` from `minibwa/kthread.c:130`.
    pub fn kt_pipeline<S, T, F>(n_threads: i32, mut func: F, shared_data: &mut S, n_steps: i32)
    where
        F: FnMut(&mut S, i32, Option<T>) -> Option<T>,
    {
        let n_threads = n_threads.max(1);
        let mut aux = ktp_t {
            index: 0,
            n_workers: n_threads,
            n_steps,
            workers: Vec::new(),
        };
        for _ in 0..n_threads {
            aux.workers.push(ktp_worker_t {
                index: aux.index,
                step: 0,
                data: None,
            });
            aux.index += 1;
        }
        for i in 0..n_threads {
            ktp_worker(&mut aux, i, shared_data, &mut func);
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn steal_work_takes_lowest_pending_worker_index() {
            let mut t = kt_for_t {
                n_threads: 3,
                n: 10,
                w: vec![
                    ktf_worker_t { i: 8 },
                    ktf_worker_t { i: 4 },
                    ktf_worker_t { i: 6 },
                ],
            };
            assert_eq!(steal_work(&mut t), 4);
            assert_eq!(t.w[1].i, 7);
        }

        #[test]
        fn kt_for_visits_each_index_once() {
            let mut seen = Vec::new();
            kt_for(4, |i, tid| seen.push((i, tid)), 17);
            seen.sort_unstable_by_key(|x| x.0);
            assert_eq!(
                seen.iter().map(|x| x.0).collect::<Vec<_>>(),
                (0..17).collect::<Vec<_>>()
            );
            assert!(seen.iter().all(|x| x.1 >= 0 && x.1 < 4));
        }

        #[test]
        fn kt_pipeline_runs_items_through_all_steps_until_source_empty() {
            #[derive(Default)]
            struct Shared {
                next: i32,
                out: Vec<String>,
            }
            let mut shared = Shared::default();
            kt_pipeline(
                2,
                |s: &mut Shared, step, input: Option<i32>| -> Option<i32> {
                    if step == 0 {
                        if s.next < 4 {
                            let v = s.next;
                            s.next += 1;
                            Some(v)
                        } else {
                            None
                        }
                    } else if step == 1 {
                        input.map(|v| v * 10)
                    } else {
                        let v = input.unwrap();
                        s.out.push(format!("{v}"));
                        Some(v)
                    }
                },
                &mut shared,
                3,
            );
            assert_eq!(shared.out, ["0", "10", "20", "30"]);
        }
    }
}

pub mod l2bit {
    #![allow(unused_variables, dead_code, non_snake_case, non_camel_case_types)]

    use flate2::read::MultiGzDecoder;
    use std::fs::File;
    use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
    use std::path::Path;

    const L2B_MAGIC: &[u8; 4] = b"L2B\x01";

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub enum l2b_meth_t {
        #[default]
        L2B_METH_NONE = 0,
        L2B_METH_C2T = 1,
        L2B_METH_G2A = 2,
    }

    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    pub struct l2b_ctg_t {
        pub name: String,
        pub comm: Option<String>,
        pub len: u64,
        pub off: u64,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct l2b_intv_t {
        pub st: u64,
        pub en: u64,
    }

    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    pub struct l2b_t {
        pub tot_len: u64,
        pub n_ctg: u64,
        pub m_ctg: u64,
        pub ctg: Vec<l2b_ctg_t>,
        pub n_pac: u64,
        pub m_pac: u64,
        pub n_ambi: u64,
        pub m_ambi: u64,
        pub n_mask: u64,
        pub m_mask: u64,
        pub ambi: Vec<l2b_intv_t>,
        pub mask: Vec<l2b_intv_t>,
        pub pac: Vec<u64>,
        pub cat_name: Vec<u8>,
        pub cat_comm: Vec<u8>,
    }

    /// Original C static function `l2b_pos2cid` from `minibwa/l2bit.c:9`.
    pub fn l2b_pos2cid(l2b: &l2b_t, s: i64, len: i64, cst: &mut i64) -> i64 {
        let mut lo = 0i64;
        let mut hi = l2b.n_ctg as i64;
        let mut mid = 0i64;
        while lo < hi {
            mid = (lo + hi) / 2;
            let ctg = &l2b.ctg[mid as usize];
            if ctg.off as i64 <= s && s < (ctg.off + ctg.len) as i64 {
                break;
            } else if s < ctg.off as i64 {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        let ctg = &l2b.ctg[mid as usize];
        *cst = s - ctg.off as i64;
        if s + len <= (ctg.off + ctg.len) as i64 {
            mid
        } else {
            -1
        }
    }

    /// Original C global function `l2b_intv2cid` from `minibwa/l2bit.c:9`.
    pub fn l2b_intv2cid(l2b: &l2b_t, st: u64, en: u64, cst: &mut i64, rev: &mut i32) -> i64 {
        assert!(st < en);
        if en > l2b.tot_len * 2 {
            return -3;
        }
        if st < l2b.tot_len && l2b.tot_len < en {
            return -2;
        }
        *rev = (st >= l2b.tot_len) as i32;
        let s = if st < l2b.tot_len {
            st
        } else {
            l2b.tot_len * 2 - en
        } as i64;
        l2b_pos2cid(l2b, s, (en - st) as i64, cst)
    }

    /// Original C global function `l2b_intv2cid_meth` from `minibwa/l2bit.c:36`.
    pub fn l2b_intv2cid_meth(
        l2b: &l2b_t,
        st: u64,
        en: u64,
        mt: &mut l2b_meth_t,
        cst: &mut i64,
        rev: &mut i32,
    ) -> i64 {
        assert!(st < en);
        let len = (en - st) as i64;
        let tot_len = l2b.tot_len;
        if en > tot_len * 4 {
            return -3;
        }
        let copy = (st / tot_len) as i32;
        *mt = if copy == 0 || copy == 3 {
            l2b_meth_t::L2B_METH_C2T
        } else {
            l2b_meth_t::L2B_METH_G2A
        };
        *rev = (copy >= 2) as i32;
        let mut s = (st - tot_len * copy as u64) as i64;
        if copy >= 2 {
            s = tot_len as i64 - len - s;
        }
        if s < 0 {
            -2
        } else {
            l2b_pos2cid(l2b, s, len, cst)
        }
    }

    /// Original C global function `l2b_getseq` from `minibwa/l2bit.c:29`.
    pub fn l2b_getseq(l2b: &l2b_t, tid: i64, mut st: i64, mut en: i64, seq: &mut [u8]) -> i64 {
        if tid < 0 || tid >= l2b.n_ctg as i64 {
            return -1;
        }
        let ctg = &l2b.ctg[tid as usize];
        if st < 0 {
            st = 0;
        }
        if en > ctg.len as i64 {
            en = ctg.len as i64;
        }
        st += ctg.off as i64;
        en += ctg.off as i64;
        let len = (en - st).max(0) as usize;
        let pac = l2b.pac.as_ptr();
        let seq_ptr = seq.as_mut_ptr();
        let start = st as u64;
        for j in 0..len {
            let i = start + j as u64;
            let c = unsafe { (*pac.add((i >> 5) as usize) >> ((i & 31) << 1)) & 3 };
            unsafe {
                *seq_ptr.add(j) = c as u8;
            }
        }
        let mut n_ambi = 0i32;
        let aid = l2b_getambi(
            l2b,
            tid,
            st - ctg.off as i64,
            en - ctg.off as i64,
            &mut n_ambi,
        );
        if aid >= 0 {
            for i in 0..n_ambi as i64 {
                let iv = l2b.ambi[(aid + i) as usize];
                let mut s = iv.st as i64;
                let mut e = iv.en as i64;
                if s < st {
                    s = st;
                }
                if e > en {
                    e = en;
                }
                if s < e {
                    let off = (s - st) as usize;
                    let len = (e - s) as usize;
                    unsafe {
                        std::ptr::write_bytes(seq_ptr.add(off), 4, len);
                    }
                }
            }
        }
        en - st
    }

    /// Original C global function `l2b_getseq_meth` from `minibwa/l2bit.c:45`.
    pub fn l2b_getseq_meth(
        l2b: &l2b_t,
        tid: i64,
        st: i64,
        en: i64,
        mt: l2b_meth_t,
        seq: &mut [u8],
    ) -> i64 {
        let len = l2b_getseq(l2b, tid, st, en, seq);
        if len <= 0 || mt == l2b_meth_t::L2B_METH_NONE {
            return len;
        }
        if mt == l2b_meth_t::L2B_METH_C2T {
            for i in 0..len as usize {
                if seq[i] == 1 {
                    seq[i] = 3;
                }
            }
        } else {
            for i in 0..len as usize {
                if seq[i] == 2 {
                    seq[i] = 0;
                }
            }
        }
        len
    }

    /// Original C static function `l2b_get0` from `minibwa/l2bit.h:43`.
    #[inline(always)]
    pub fn l2b_get0(l2b: &l2b_t, i: u64) -> i32 {
        ((l2b.pac[(i >> 5) as usize] >> ((i & 31) << 1)) & 3) as i32
    }

    /// Original C static function `l2b_seq_prefetch` from `minibwa/l2bit.h:48`.
    pub fn l2b_seq_prefetch(l2b: &l2b_t, tid: i64, st: i64) {
        if tid >= 0 && tid < l2b.n_ctg as i64 && st >= 0 && st < l2b.ctg[tid as usize].len as i64 {
            let idx = (st as u64 + l2b.ctg[tid as usize].off) >> 5;
            if let Some(cell) = l2b.pac.get(idx as usize) {
                crate::s2n_lite::_mm_prefetch(cell as *const u64 as *const u8, 3);
            }
        }
    }

    /// Original C static function `l2b_meth_rev` from `minibwa/l2bit.h:54`.
    pub fn l2b_meth_rev(mt: l2b_meth_t) -> l2b_meth_t {
        if mt == l2b_meth_t::L2B_METH_NONE {
            l2b_meth_t::L2B_METH_NONE
        } else if mt == l2b_meth_t::L2B_METH_C2T {
            l2b_meth_t::L2B_METH_G2A
        } else {
            l2b_meth_t::L2B_METH_C2T
        }
    }

    /// Original C global function `l2b_getambi` from `minibwa/l2bit.c:56`.
    pub fn l2b_getambi(l2b: &l2b_t, tid: i64, mut st: i64, mut en: i64, n_ambi: &mut i32) -> i64 {
        *n_ambi = 0;
        if tid < 0 || tid >= l2b.n_ctg as i64 {
            return -1;
        }
        if st < 0 {
            st = 0;
        }
        if en > l2b.ctg[tid as usize].len as i64 {
            en = l2b.ctg[tid as usize].len as i64;
        }
        if st >= en {
            return -1;
        }
        let g_beg = l2b.ctg[tid as usize].off as i64 + st;
        let g_end = l2b.ctg[tid as usize].off as i64 + en;

        let mut lo = 0i64;
        let mut hi = l2b.n_ambi as i64;
        while lo < hi {
            let mid = (lo + hi) / 2;
            if l2b.ambi[mid as usize].en as i64 > g_beg {
                hi = mid;
            } else {
                lo = mid + 1;
            }
        }
        let i_st = lo;
        lo = i_st;
        hi = l2b.n_ambi as i64;
        while lo < hi {
            let mid = (lo + hi) / 2;
            if l2b.ambi[mid as usize].st as i64 >= g_end {
                hi = mid;
            } else {
                lo = mid + 1;
            }
        }
        let i_en = lo;
        *n_ambi = (i_en - i_st) as i32;
        if *n_ambi == 0 {
            -1
        } else {
            i_st
        }
    }

    /// Original C static function `l2b_format_seq` from `minibwa/l2bit.c:88`.
    pub fn l2b_format_seq(len: u64, seq: &mut [u8], rng: &mut u64) {
        for i in 0..len as usize {
            let b = seq[i];
            let mut c = match b {
                b'A' | b'a' => 0,
                b'C' | b'c' => 1,
                b'G' | b'g' => 2,
                b'T' | b't' | b'U' | b'u' => 3,
                _ => 4,
            };
            if c == 4 {
                *rng = rng.wrapping_add(0x9e3779b97f4a7c15);
                let mut z = *rng;
                z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
                z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
                c |= ((z ^ (z >> 31)) & 3) as u8;
            }
            if b < b'A' || b > b'Z' {
                c |= 1 << 3;
            }
            seq[i] = c;
        }
    }

    /// Original C static function `l2b_add_seq` from `minibwa/l2bit.c:101`.
    pub fn l2b_add_seq(
        l2b: &mut l2b_t,
        len: u64,
        seq: &[u8],
        name: &str,
        comm: Option<&str>,
        rng: &mut u64,
    ) {
        let off = l2b.tot_len;
        l2b.ctg.push(l2b_ctg_t {
            name: name.to_string(),
            comm: comm.map(str::to_string),
            len,
            off: l2b.tot_len,
        });
        l2b.n_ctg += 1;
        l2b.m_ctg = l2b.ctg.capacity() as u64;
        l2b.tot_len += len;

        l2b.n_pac = (l2b.tot_len + 31) / 32;
        if l2b.pac.len() < l2b.n_pac as usize {
            l2b.pac.resize(l2b.n_pac as usize, 0);
        }
        l2b.m_pac = l2b.pac.capacity() as u64;

        let mut ambi_len = 0u64;
        let mut mask_len = 0u64;
        for i in 0..len {
            let c = seq[i as usize] as u64;
            let x = off + i;
            if (c & (1 << 3)) != 0 {
                mask_len += 1;
            } else if mask_len > 0 {
                l2b.mask.push(l2b_intv_t {
                    st: x - mask_len,
                    en: x,
                });
                l2b.n_mask += 1;
                mask_len = 0;
            }
            if (c & (1 << 2)) != 0 {
                ambi_len += 1;
            } else if ambi_len > 0 {
                l2b.ambi.push(l2b_intv_t {
                    st: x - ambi_len,
                    en: x,
                });
                l2b.n_ambi += 1;
                ambi_len = 0;
            }
            l2b.pac[(x >> 5) as usize] |= (c & 3) << ((x & 0x1f) * 2);
        }
        l2b.m_mask = l2b.mask.capacity() as u64;
        l2b.m_ambi = l2b.ambi.capacity() as u64;
    }

    /// Original C static function `l2b_collate_str` from `minibwa/l2bit.c:145`.
    pub fn l2b_collate_str(l2b: &mut l2b_t) {
        if !l2b.cat_name.is_empty() || !l2b.cat_comm.is_empty() {
            return;
        }
        for ctg in &l2b.ctg {
            l2b.cat_name.extend_from_slice(ctg.name.as_bytes());
            l2b.cat_name.push(0);
            if let Some(comm) = &ctg.comm {
                l2b.cat_comm.extend_from_slice(comm.as_bytes());
            }
            l2b.cat_comm.push(0);
        }
    }

    /// Original C global function `l2b_import` from `minibwa/l2bit.c:175`.
    pub fn l2b_import<P: AsRef<Path>>(fn_: P, seed: u64) -> Option<l2b_t> {
        let path = fn_.as_ref().to_string_lossy();
        let raw: Box<dyn Read> = if path == "-" {
            Box::new(io::stdin())
        } else {
            Box::new(File::open(fn_.as_ref()).ok()?)
        };
        let mut buffered = BufReader::with_capacity(1 << 20, raw);
        let is_gzip = buffered.fill_buf().ok()?.starts_with(&[0x1f, 0x8b]);
        let mut reader: Box<dyn BufRead> = if is_gzip {
            Box::new(BufReader::with_capacity(
                1 << 20,
                MultiGzDecoder::new(buffered),
            ))
        } else {
            Box::new(buffered)
        };
        let mut l2b = l2b_t::default();
        let mut rng = seed;
        let mut name = String::new();
        let mut comm: Option<String> = None;
        let mut seq = Vec::new();
        let mut line = String::new();
        loop {
            line.clear();
            if reader.read_line(&mut line).ok()? == 0 {
                break;
            }
            let line = line.trim_end_matches(['\n', '\r']);
            if let Some(header) = line.strip_prefix('>') {
                if !name.is_empty() {
                    l2b_format_seq(seq.len() as u64, &mut seq, &mut rng);
                    l2b_add_seq(
                        &mut l2b,
                        seq.len() as u64,
                        &seq,
                        &name,
                        comm.as_deref(),
                        &mut rng,
                    );
                    seq.clear();
                }
                let mut parts = header.splitn(2, char::is_whitespace);
                name = parts.next().unwrap_or("").to_string();
                comm = parts.next().filter(|s| !s.is_empty()).map(str::to_string);
            } else {
                seq.extend_from_slice(line.trim().as_bytes());
            }
        }
        if !name.is_empty() {
            l2b_format_seq(seq.len() as u64, &mut seq, &mut rng);
            l2b_add_seq(
                &mut l2b,
                seq.len() as u64,
                &seq,
                &name,
                comm.as_deref(),
                &mut rng,
            );
        }
        l2b_collate_str(&mut l2b);
        Some(l2b)
    }

    /// Original C global function `l2b_destroy` from `minibwa/l2bit.c:196`.
    pub fn l2b_destroy(l2b: Option<l2b_t>) {
        drop(l2b);
    }

    /// Original C global function `l2b_save` from `minibwa/l2bit.c:202`.
    pub fn l2b_save<P: AsRef<Path>>(fn_: P, l2b: &l2b_t) -> i32 {
        let path = fn_.as_ref().to_string_lossy();
        let mut fp: Box<dyn Write> = if path == "-" {
            Box::new(BufWriter::with_capacity(1 << 20, io::stdout()))
        } else {
            match File::create(fn_.as_ref()) {
                Ok(fp) => Box::new(BufWriter::with_capacity(1 << 20, fp)),
                Err(_) => return -1,
            }
        };
        let len_name: u64 = l2b.ctg.iter().map(|c| c.name.len() as u64 + 1).sum();
        let len_comm: u64 = l2b
            .ctg
            .iter()
            .map(|c| c.comm.as_ref().map(|s| s.len() as u64 + 1).unwrap_or(1))
            .sum();
        if fp.write_all(L2B_MAGIC).is_err() || fp.write_all(&0u32.to_le_bytes()).is_err() {
            return -1;
        }
        for x in [
            l2b.n_ctg,
            l2b.tot_len,
            l2b.n_ambi,
            l2b.n_mask,
            len_name,
            len_comm,
            l2b.n_pac,
        ] {
            if fp.write_all(&x.to_le_bytes()).is_err() {
                return -1;
            }
        }
        for ctg in &l2b.ctg {
            if fp.write_all(&ctg.len.to_le_bytes()).is_err() {
                return -1;
            }
        }
        for iv in &l2b.ambi {
            if fp.write_all(&iv.st.to_le_bytes()).is_err()
                || fp.write_all(&iv.en.to_le_bytes()).is_err()
            {
                return -1;
            }
        }
        for iv in &l2b.mask {
            if fp.write_all(&iv.st.to_le_bytes()).is_err()
                || fp.write_all(&iv.en.to_le_bytes()).is_err()
            {
                return -1;
            }
        }
        if write_u64_slice_le(&mut fp, &l2b.pac).is_err() {
            return -1;
        }
        for ctg in &l2b.ctg {
            if fp.write_all(ctg.name.as_bytes()).is_err() || fp.write_all(&[0]).is_err() {
                return -1;
            }
        }
        for ctg in &l2b.ctg {
            if let Some(comm) = &ctg.comm {
                if fp.write_all(comm.as_bytes()).is_err() {
                    return -1;
                }
            }
            if fp.write_all(&[0]).is_err() {
                return -1;
            }
        }
        if fp.flush().is_err() {
            -1
        } else {
            0
        }
    }

    fn write_u64_slice_le<W: Write + ?Sized>(writer: &mut W, words: &[u64]) -> std::io::Result<()> {
        #[cfg(target_endian = "little")]
        {
            let bytes = unsafe {
                std::slice::from_raw_parts(
                    words.as_ptr().cast::<u8>(),
                    std::mem::size_of_val(words),
                )
            };
            writer.write_all(bytes)
        }
        #[cfg(not(target_endian = "little"))]
        {
            for &word in words {
                writer.write_all(&word.to_le_bytes())?;
            }
            Ok(())
        }
    }

    fn read_intv_vec_le<R: Read + ?Sized>(reader: &mut R, n: u64) -> Option<Vec<l2b_intv_t>> {
        let n = n as usize;
        let mut v = Vec::<l2b_intv_t>::with_capacity(n);
        let bytes = unsafe {
            std::slice::from_raw_parts_mut(
                v.as_mut_ptr() as *mut u8,
                n.checked_mul(std::mem::size_of::<l2b_intv_t>())?,
            )
        };
        reader.read_exact(bytes).ok()?;
        unsafe {
            v.set_len(n);
        }
        #[cfg(target_endian = "big")]
        {
            for x in &mut v {
                x.st = u64::from_le(x.st);
                x.en = u64::from_le(x.en);
            }
        }
        Some(v)
    }

    /// Original C global function `l2b_load` from `minibwa/l2bit.c:234`.
    pub fn l2b_load<P: AsRef<Path>>(fn_: P) -> Option<l2b_t> {
        let path = fn_.as_ref().to_string_lossy();
        let mut fp: Box<dyn Read> = if path == "-" {
            Box::new(io::stdin())
        } else {
            Box::new(File::open(fn_.as_ref()).ok()?)
        };
        let mut magic = [0u8; 4];
        fp.read_exact(&mut magic).ok()?;
        if &magic != L2B_MAGIC {
            return None;
        }
        let mut dummy = [0u8; 4];
        fp.read_exact(&mut dummy).ok()?;
        let mut hdr = [0u8; 56];
        fp.read_exact(&mut hdr).ok()?;
        let mut fields = [0u64; 7];
        for i in 0..7usize {
            fields[i] = u64::from_le_bytes(hdr[i * 8..i * 8 + 8].try_into().unwrap());
        }
        let (n_ctg, tot_len, n_ambi, n_mask, len_name, len_comm, n_pac) = (
            fields[0], fields[1], fields[2], fields[3], fields[4], fields[5], fields[6],
        );
        let mut l2b = l2b_t {
            tot_len,
            n_ctg,
            m_ctg: n_ctg,
            n_pac,
            m_pac: n_pac,
            n_ambi,
            m_ambi: n_ambi,
            n_mask,
            m_mask: n_mask,
            ..Default::default()
        };
        let mut off = 0u64;
        for _ in 0..n_ctg {
            let mut len_buf = [0u8; 8];
            fp.read_exact(&mut len_buf).ok()?;
            let len = u64::from_le_bytes(len_buf);
            l2b.ctg.push(l2b_ctg_t {
                name: String::new(),
                comm: None,
                len,
                off,
            });
            off += len;
        }
        if off != tot_len {
            return None;
        }
        l2b.ambi = read_intv_vec_le(&mut *fp, n_ambi)?;
        l2b.mask = read_intv_vec_le(&mut *fp, n_mask)?;
        l2b.pac = Vec::with_capacity(n_pac as usize);
        let pac_bytes = unsafe {
            std::slice::from_raw_parts_mut(l2b.pac.as_mut_ptr() as *mut u8, n_pac as usize * 8)
        };
        fp.read_exact(pac_bytes).ok()?;
        unsafe {
            l2b.pac.set_len(n_pac as usize);
        }
        #[cfg(target_endian = "big")]
        {
            for x in &mut l2b.pac {
                *x = u64::from_le(*x);
            }
        }
        l2b.cat_name = vec![0; len_name as usize];
        l2b.cat_comm = vec![0; len_comm as usize];
        fp.read_exact(&mut l2b.cat_name).ok()?;
        fp.read_exact(&mut l2b.cat_comm).ok()?;
        let mut p_name = 0usize;
        let mut p_comm = 0usize;
        for i in 0..n_ctg as usize {
            let name_end = p_name + l2b.cat_name[p_name..].iter().position(|&b| b == 0)?;
            l2b.ctg[i].name = String::from_utf8_lossy(&l2b.cat_name[p_name..name_end]).into_owned();
            p_name = name_end + 1;
            if l2b.cat_comm[p_comm] != 0 {
                let comm_end = p_comm + l2b.cat_comm[p_comm..].iter().position(|&b| b == 0)?;
                l2b.ctg[i].comm =
                    Some(String::from_utf8_lossy(&l2b.cat_comm[p_comm..comm_end]).into_owned());
                p_comm = comm_end + 1;
            } else {
                p_comm += 1;
            }
        }
        if p_name != len_name as usize || p_comm != len_comm as usize {
            return None;
        }
        Some(l2b)
    }

    /// Original C global function `l2b_save_pac` from `minibwa/l2bit.c:286`.
    pub fn l2b_save_pac<P: AsRef<Path>>(fn_: P, l2b: &l2b_t, both_strand: i32) -> i32 {
        let path = fn_.as_ref().to_string_lossy();
        let mut fp: Box<dyn Write> = if path == "-" {
            Box::new(io::stdout())
        } else {
            match File::create(fn_.as_ref()) {
                Ok(fp) => Box::new(fp),
                Err(_) => return -1,
            }
        };
        let n_pac = ((if both_strand != 0 {
            l2b.tot_len * 2
        } else {
            l2b.tot_len
        }) + 3)
            / 4;
        let mut pac = vec![0u8; n_pac as usize];
        let mut x = 0u64;
        for i in 0..l2b.tot_len {
            pac[(x >> 2) as usize] |= (l2b_get0(l2b, i) as u8) << ((!x & 3) * 2);
            x += 1;
        }
        if both_strand != 0 {
            for i in (0..l2b.tot_len).rev() {
                pac[(x >> 2) as usize] |= (3 - l2b_get0(l2b, i) as u8) << ((!x & 3) * 2);
                x += 1;
            }
        }
        let n_write = (x >> 2) + if (x & 3) == 0 { 0 } else { 1 };
        if fp.write_all(&pac[..n_write as usize]).is_err() {
            return -1;
        }
        if x % 4 == 0 && fp.write_all(&[0]).is_err() {
            return -1;
        }
        if fp.write_all(&[(x % 4) as u8]).is_err() {
            return -1;
        }
        0
    }

    /// Original C static function `l2b_c2t` from `minibwa/l2bit.c:321`.
    pub fn l2b_c2t(b: u8) -> u8 {
        if b == 1 {
            3
        } else {
            b
        }
    }

    /// Original C static function `l2b_g2a` from `minibwa/l2bit.c:322`.
    pub fn l2b_g2a(b: u8) -> u8 {
        if b == 2 {
            0
        } else {
            b
        }
    }

    /// Original C global function `l2b_save_pac_meth` from `minibwa/l2bit.c:324`.
    pub fn l2b_save_pac_meth<P: AsRef<Path>>(fn_: P, l2b: &l2b_t, both_strand: i32) -> i32 {
        let path = fn_.as_ref().to_string_lossy();
        let mut fp: Box<dyn Write> = if path == "-" {
            Box::new(io::stdout())
        } else {
            match File::create(fn_.as_ref()) {
                Ok(fp) => Box::new(fp),
                Err(_) => return -1,
            }
        };
        let mut len = l2b.tot_len * 2;
        if both_strand != 0 {
            len *= 2;
        }
        let n_pac = (len + 3) / 4;
        let mut pac = vec![0u8; n_pac as usize];
        let mut x = 0u64;
        for i in 0..l2b.tot_len {
            let b = l2b_c2t(l2b_get0(l2b, i) as u8);
            pac[(x >> 2) as usize] |= b << ((!x & 3) * 2);
            x += 1;
        }
        for i in 0..l2b.tot_len {
            let b = l2b_g2a(l2b_get0(l2b, i) as u8);
            pac[(x >> 2) as usize] |= b << ((!x & 3) * 2);
            x += 1;
        }
        if both_strand != 0 {
            for i in (0..l2b.tot_len).rev() {
                let b = 3 - l2b_g2a(l2b_get0(l2b, i) as u8);
                pac[(x >> 2) as usize] |= b << ((!x & 3) * 2);
                x += 1;
            }
            for i in (0..l2b.tot_len).rev() {
                let b = 3 - l2b_c2t(l2b_get0(l2b, i) as u8);
                pac[(x >> 2) as usize] |= b << ((!x & 3) * 2);
                x += 1;
            }
        }
        let n_write = (x >> 2) + if (x & 3) == 0 { 0 } else { 1 };
        if fp.write_all(&pac[..n_write as usize]).is_err() {
            return -1;
        }
        if x % 4 == 0 && fp.write_all(&[0]).is_err() {
            return -1;
        }
        if fp.write_all(&[(x % 4) as u8]).is_err() {
            return -1;
        }
        0
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn load_real_chrM_l2b_and_get_reference_prefix() {
            let l2b = l2b_load("minibwa/chrM-human.l2b").expect("load chrM-human.l2b");
            assert_eq!(l2b.n_ctg, 1);
            assert_eq!(l2b.ctg[0].name, "chrM");
            assert_eq!(l2b.tot_len, l2b.ctg[0].len);

            let mut seq = vec![0u8; 32];
            assert_eq!(l2b_getseq(&l2b, 0, 0, 32, &mut seq), 32);
            let expected = b"GATCACAGGTCTATCACCCTATTAACCACTCA"
                .iter()
                .map(|&b| match b {
                    b'A' => 0,
                    b'C' => 1,
                    b'G' => 2,
                    b'T' => 3,
                    _ => 4,
                })
                .collect::<Vec<_>>();
            assert_eq!(seq, expected);
        }

        #[test]
        fn real_chrM_l2b_roundtrips_through_save_load() {
            let l2b = l2b_load("minibwa/chrM-human.l2b").expect("load chrM-human.l2b");
            let path = std::env::temp_dir().join("minibwa-rs-roundtrip.l2b");
            assert_eq!(l2b_save(&path, &l2b), 0);
            let loaded = l2b_load(&path).expect("reload l2b");
            let _ = std::fs::remove_file(&path);
            assert_eq!(loaded, l2b);
        }

        #[test]
        fn intv2cid_handles_forward_reverse_and_invalid_real_chrM_ranges() {
            let l2b = l2b_load("minibwa/chrM-human.l2b").expect("load chrM-human.l2b");
            let mut cst = -1;
            let mut rev = -1;
            assert_eq!(l2b_intv2cid(&l2b, 10, 20, &mut cst, &mut rev), 0);
            assert_eq!(cst, 10);
            assert_eq!(rev, 0);

            let st = l2b.tot_len * 2 - 20;
            let en = l2b.tot_len * 2 - 10;
            assert_eq!(l2b_intv2cid(&l2b, st, en, &mut cst, &mut rev), 0);
            assert_eq!(cst, 10);
            assert_eq!(rev, 1);

            assert_eq!(
                l2b_intv2cid(&l2b, l2b.tot_len - 1, l2b.tot_len + 1, &mut cst, &mut rev),
                -2
            );
            assert_eq!(
                l2b_intv2cid(&l2b, 0, l2b.tot_len * 2 + 1, &mut cst, &mut rev),
                -3
            );
        }

        #[test]
        fn save_pac_emits_expected_small_packed_sequence() {
            let mut l2b = l2b_t::default();
            let mut seq = b"ACGTAC".to_vec();
            let mut rng = 11;
            l2b_format_seq(seq.len() as u64, &mut seq, &mut rng);
            l2b_add_seq(&mut l2b, seq.len() as u64, &seq, "s", None, &mut rng);
            l2b_collate_str(&mut l2b);
            let path = std::env::temp_dir().join("minibwa-rs-small.pac");
            assert_eq!(l2b_save_pac(&path, &l2b, 0), 0);
            let bytes = std::fs::read(&path).expect("read pac");
            let _ = std::fs::remove_file(&path);
            assert_eq!(bytes, vec![0x1b, 0x10, 2]);
        }

        #[test]
        fn import_reads_real_gzipped_chrM_fasta() {
            let imported =
                l2b_import("minibwa/test/chrM-human.fa.gz", 11).expect("import gzipped FASTA");
            let loaded = l2b_load("minibwa/chrM-human.l2b").expect("load chrM-human.l2b");
            assert_eq!(imported.n_ctg, loaded.n_ctg);
            assert_eq!(imported.ctg[0].name, loaded.ctg[0].name);
            assert_eq!(imported.ctg[0].len, loaded.ctg[0].len);
            assert_eq!(imported.tot_len, loaded.tot_len);
            let mut imported_seq = vec![0; 64];
            let mut loaded_seq = vec![0; 64];
            l2b_getseq(&imported, 0, 0, 64, &mut imported_seq);
            l2b_getseq(&loaded, 0, 0, 64, &mut loaded_seq);
            assert_eq!(imported_seq, loaded_seq);
        }

        #[test]
        fn methylation_interval_and_sequence_conversion_match_rules() {
            let l2b = l2b_load("minibwa/chrM-human.l2b").expect("load chrM-human.l2b");
            let mut cst = -1;
            let mut rev = -1;
            let mut mt = l2b_meth_t::L2B_METH_NONE;
            assert_eq!(
                l2b_intv2cid_meth(&l2b, 0, 10, &mut mt, &mut cst, &mut rev),
                0
            );
            assert_eq!(mt, l2b_meth_t::L2B_METH_C2T);
            assert_eq!(cst, 0);
            assert_eq!(rev, 0);
            assert_eq!(
                l2b_intv2cid_meth(
                    &l2b,
                    l2b.tot_len,
                    l2b.tot_len + 10,
                    &mut mt,
                    &mut cst,
                    &mut rev
                ),
                0
            );
            assert_eq!(mt, l2b_meth_t::L2B_METH_G2A);
            assert_eq!(rev, 0);
            assert_eq!(
                l2b_intv2cid_meth(
                    &l2b,
                    l2b.tot_len * 3,
                    l2b.tot_len * 3 + 10,
                    &mut mt,
                    &mut cst,
                    &mut rev
                ),
                0
            );
            assert_eq!(mt, l2b_meth_t::L2B_METH_C2T);
            assert_eq!(rev, 1);
            assert_eq!(
                l2b_meth_rev(l2b_meth_t::L2B_METH_C2T),
                l2b_meth_t::L2B_METH_G2A
            );

            let mut seq = vec![0; 16];
            l2b_getseq_meth(&l2b, 0, 0, 16, l2b_meth_t::L2B_METH_C2T, &mut seq);
            assert!(!seq.contains(&1));
            l2b_getseq_meth(&l2b, 0, 0, 16, l2b_meth_t::L2B_METH_G2A, &mut seq);
            assert!(!seq.contains(&2));
        }

        #[test]
        fn save_pac_meth_emits_expected_small_packed_sequence() {
            let mut l2b = l2b_t::default();
            let mut seq = b"ACGT".to_vec();
            let mut rng = 3;
            l2b_format_seq(seq.len() as u64, &mut seq, &mut rng);
            l2b_add_seq(&mut l2b, seq.len() as u64, &seq, "s", None, &mut rng);
            let path = std::env::temp_dir().join("minibwa-rs-small-meth.pac");
            assert_eq!(l2b_save_pac_meth(&path, &l2b, 1), 0);
            let bytes = std::fs::read(&path).expect("read meth pac");
            let _ = std::fs::remove_file(&path);
            assert_eq!(bytes, vec![0x3b, 0x13, 0x3b, 0x13, 0, 0]);
        }
    }
}

pub mod lchain {
    #![allow(unused_variables, dead_code, non_snake_case, non_camel_case_types)]

    use crate::l2bit::l2b_t;

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct mb_anchor_t {
        pub sid: i32,
        pub len: i32,
        pub qpos: i32,
        pub flag: u32,
        pub flt: u32,
        pub tpos: i64,
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct mb128_t {
        pub x: u64,
        pub y: u64,
    }

    fn radix_insert_sort_mb128x(a: &mut [mb128_t]) {
        for i in 1..a.len() {
            if a[i].x < a[i - 1].x {
                let tmp = a[i];
                let mut j = i;
                while j > 0 && tmp.x < a[j - 1].x {
                    a[j] = a[j - 1];
                    j -= 1;
                }
                a[j] = tmp;
            }
        }
    }

    fn radix_sort_mb128x_rec(a: &mut [mb128_t], n_bits: u32, s: u32) {
        const RS_MIN_SIZE: usize = 64;
        let size = 1usize << n_bits;
        let mask = size as u64 - 1;
        let mut counts = [0usize; 256];
        for x in a.iter() {
            counts[((x.x >> s) & mask) as usize] += 1;
        }
        let mut starts = [0usize; 256];
        let mut ends = [0usize; 256];
        let mut sum = 0usize;
        for k in 0..size {
            starts[k] = sum;
            sum += counts[k];
            ends[k] = sum;
        }
        let mut b = starts;
        let mut k = 0usize;
        while k < size {
            if b[k] != ends[k] {
                let mut l = ((a[b[k]].x >> s) & mask) as usize;
                if l != k {
                    let mut tmp = a[b[k]];
                    loop {
                        std::mem::swap(&mut tmp, &mut a[b[l]]);
                        b[l] += 1;
                        l = ((tmp.x >> s) & mask) as usize;
                        if l == k {
                            break;
                        }
                    }
                    a[b[k]] = tmp;
                    b[k] += 1;
                } else {
                    b[k] += 1;
                }
            } else {
                k += 1;
            }
        }
        if s != 0 {
            let next_s = s.saturating_sub(n_bits);
            for k in 0..size {
                let st = starts[k];
                let en = ends[k];
                if en - st > RS_MIN_SIZE {
                    radix_sort_mb128x_rec(&mut a[st..en], n_bits, next_s);
                } else if en - st > 1 {
                    radix_insert_sort_mb128x(&mut a[st..en]);
                }
            }
        }
    }

    fn radix_sort_mb128x(a: &mut [mb128_t]) {
        const RS_MIN_SIZE: usize = 64;
        if a.len() <= RS_MIN_SIZE {
            radix_insert_sort_mb128x(a);
        } else {
            radix_sort_mb128x_rec(a, 8, 56);
        }
    }

    pub fn mb_log2(x: f32) -> f32 {
        let mut i = x.to_bits();
        let mut log_2 = ((i >> 23) & 255) as f32 - 128.0;
        i &= !(255 << 23);
        i += 127 << 23;
        let f = f32::from_bits(i);
        log_2 += (-0.34484843f32 * f + 2.02466578f32) * f - 0.67487759f32;
        log_2
    }

    /// Original C static function `mb_chain_bk_end` from `minibwa/lchain.c:25`.
    pub fn mb_chain_bk_end(
        max_drop: i32,
        z: &[mb128_t],
        f: &[i32],
        p: &[i64],
        t: &mut [i32],
        k: i64,
    ) -> i64 {
        let mut i = z[k as usize].y as i64;
        let mut end_i: i64;
        let mut max_i = i;
        let mut max_s = 0i32;
        if i < 0 || t[i as usize] != 0 {
            return i;
        }
        loop {
            t[i as usize] = 2;
            i = p[i as usize];
            end_i = i;
            let s = if i < 0 {
                z[k as usize].x as i32
            } else {
                z[k as usize].x as i32 - f[i as usize]
            };
            if s > max_s {
                max_s = s;
                max_i = i;
            } else if max_s - s > max_drop {
                break;
            }
            if !(i >= 0 && t[i as usize] == 0) {
                break;
            }
        }
        i = z[k as usize].y as i64;
        while i >= 0 && i != end_i {
            t[i as usize] = 0;
            i = p[i as usize];
        }
        max_i
    }

    /// Original C static function `mb_chain_backtrack` from `minibwa/lchain.c:43`.
    pub fn mb_chain_backtrack(
        km: (),
        n: i64,
        f: &[i32],
        p: &[i64],
        v: &mut [i32],
        t: &mut [i32],
        min_sc: i32,
        max_drop: i32,
        n_u_: &mut i32,
        n_v_: &mut i32,
    ) -> Vec<u64> {
        *n_u_ = 0;
        *n_v_ = 0;
        let n_z = (0..n as usize).filter(|&i| f[i] >= min_sc).count();
        if n_z == 0 {
            return Vec::new();
        }
        let mut z = Vec::with_capacity(n_z);
        for i in 0..n as usize {
            if f[i] >= min_sc {
                z.push(mb128_t {
                    x: f[i] as u64,
                    y: i as u64,
                });
            }
        }
        radix_sort_mb128x(&mut z);

        t[..n as usize].fill(0);
        let mut n_v = 0i64;
        let mut n_u = 0i32;
        for k in (0..n_z as i64).rev() {
            if t[z[k as usize].y as usize] == 0 {
                let n_v0 = n_v;
                let end_i = mb_chain_bk_end(max_drop, &z, f, p, t, k);
                let mut i = z[k as usize].y as i64;
                while i != end_i {
                    n_v += 1;
                    t[i as usize] = 1;
                    i = p[i as usize];
                }
                let sc = if i < 0 {
                    z[k as usize].x as i32
                } else {
                    z[k as usize].x as i32 - f[i as usize]
                };
                if sc >= min_sc && n_v > n_v0 {
                    n_u += 1;
                } else {
                    n_v = n_v0;
                }
            }
        }

        let mut u = vec![0u64; n_u as usize];
        t[..n as usize].fill(0);
        n_v = 0;
        n_u = 0;
        for k in (0..n_z as i64).rev() {
            if t[z[k as usize].y as usize] == 0 {
                let n_v0 = n_v;
                let end_i = mb_chain_bk_end(max_drop, &z, f, p, t, k);
                let mut i = z[k as usize].y as i64;
                while i != end_i {
                    v[n_v as usize] = i as i32;
                    n_v += 1;
                    t[i as usize] = 1;
                    i = p[i as usize];
                }
                let sc = if i < 0 {
                    z[k as usize].x as i32
                } else {
                    z[k as usize].x as i32 - f[i as usize]
                };
                if sc >= min_sc && n_v > n_v0 {
                    u[n_u as usize] = (sc as u64) << 32 | (n_v - n_v0) as u64;
                    n_u += 1;
                } else {
                    n_v = n_v0;
                }
            }
        }
        assert!(n_v < i32::MAX as i64);
        *n_u_ = n_u;
        *n_v_ = n_v as i32;
        u
    }

    /// Original C static function `compact_a` from `minibwa/lchain.c:94`.
    pub fn compact_a(
        km: (),
        l2b: &l2b_t,
        n_u: i32,
        u: &mut [u64],
        n_v: i32,
        v: Vec<i32>,
        mut a: Vec<mb_anchor_t>,
    ) -> Vec<mb_anchor_t> {
        let mut b = vec![mb_anchor_t::default(); n_v as usize];
        let mut k = 0usize;
        for i in 0..n_u as usize {
            let k0 = k;
            let ni = u[i] as i32;
            for j in 0..ni as usize {
                b[k] = a[v[k0 + (ni as usize - j - 1)] as usize];
                k += 1;
            }
        }

        let mut w = vec![mb128_t::default(); n_u as usize];
        k = 0;
        for i in 0..n_u as usize {
            let ctg = &l2b.ctg[(b[k].sid >> 1) as usize];
            w[i].x = b[k].tpos as u64 + ctg.off * 2 + ctg.len * (b[k].sid as u64 & 1);
            w[i].y = (k as u64) << 32 | i as u64;
            k += u[i] as u32 as usize;
        }
        radix_sort_mb128x(&mut w);
        let mut u2 = vec![0u64; n_u as usize];
        k = 0;
        for i in 0..n_u as usize {
            let j = w[i].y as u32 as usize;
            let n = u[j] as u32 as usize;
            u2[i] = u[j];
            let src = (w[i].y >> 32) as usize;
            a[k..k + n].copy_from_slice(&b[src..src + n]);
            k += n;
        }
        u[..n_u as usize].copy_from_slice(&u2);
        b[..k].copy_from_slice(&a[..k]);
        b
    }

    /// Original C static function `comput_sc` from `minibwa/lchain.c:131`.
    pub fn comput_sc(
        ai: &mb_anchor_t,
        aj: &mb_anchor_t,
        max_dist_x: i32,
        max_dist_y: i32,
        bw: i32,
        chn_pen_gap: f32,
    ) -> i32 {
        let dq = (ai.qpos - aj.qpos) as i64;
        if dq <= 0 || dq > (max_dist_y + ai.len) as i64 {
            return i32::MIN;
        }
        if ai.sid != aj.sid {
            return i32::MIN;
        }
        let dr = ai.tpos - aj.tpos;
        if dr <= 0 || dq > (max_dist_x + ai.len) as i64 {
            return i32::MIN;
        }
        let dd = if dr > dq { dr - dq } else { dq - dr };
        if dd > bw as i64 {
            return i32::MIN;
        }
        let dg = if dr < dq { dr } else { dq };
        let mut sc = if (ai.len as i64) < dg {
            ai.len as i64
        } else {
            dg
        };
        if dd != 0 {
            let lin_pen = chn_pen_gap * dd as f32;
            let log_pen = if dd >= 1 {
                mb_log2((dd + 1) as f32)
            } else {
                0.0
            };
            sc -= (lin_pen + 0.5 * log_pen) as i64;
        }
        sc as i32
    }

    /// Original C global function `mb_lchain_dp` from `minibwa/lchain.c:163`.
    pub fn mb_lchain_dp(
        km: (),
        l2b: &l2b_t,
        mut max_dist_x: i32,
        mut max_dist_y: i32,
        bw: i32,
        max_skip: i32,
        max_iter: i32,
        min_sc: i32,
        chn_pen_gap: f32,
        n: i64,
        a: Vec<mb_anchor_t>,
        n_u_: &mut i32,
        _u: &mut Vec<u64>,
    ) -> Vec<mb_anchor_t> {
        *_u = Vec::new();
        *n_u_ = 0;
        if n == 0 || a.is_empty() {
            return Vec::new();
        }
        if max_dist_x < bw {
            max_dist_x = bw;
        }
        if max_dist_y < bw {
            max_dist_y = bw;
        }
        let n_usize = n as usize;
        let mut v = vec![0i32; n_usize];
        let mut p = vec![0i64; n_usize];
        let mut f = vec![0i32; n_usize];
        let mut t = vec![0i32; n_usize];

        let mut mmax_f = 0i32;
        let max_drop = bw;
        let mut max_ii = -1i64;
        for i in 0..n {
            let mut max_j = -1i64;
            let mut max_f = a[i as usize].len;
            let mut n_skip = 0i32;
            let mut j = i - 1;
            while j >= 0 && j >= i - max_iter as i64 {
                if a[i as usize].tpos - a[j as usize].tpos
                    >= (max_dist_x + a[i as usize].len) as i64
                {
                    break;
                }
                let mut sc = comput_sc(
                    &a[i as usize],
                    &a[j as usize],
                    max_dist_x,
                    max_dist_y,
                    bw,
                    chn_pen_gap,
                );
                if sc != i32::MIN {
                    sc += f[j as usize];
                    if sc > max_f {
                        max_f = sc;
                        max_j = j;
                        if n_skip > 0 {
                            n_skip -= 1;
                        }
                    } else if t[j as usize] == i as i32 {
                        n_skip += 1;
                        if n_skip > max_skip {
                            break;
                        }
                    }
                    if p[j as usize] >= 0 {
                        t[p[j as usize] as usize] = i as i32;
                    }
                }
                j -= 1;
            }
            let end_j = j;
            if max_ii < 0
                || a[i as usize].tpos - a[max_ii as usize].tpos
                    > (max_dist_x + a[i as usize].len) as i64
            {
                let mut max = i32::MIN;
                max_ii = -1;
                j = i - 1;
                while j >= end_j && j >= 0 {
                    if max < f[j as usize] {
                        max = f[j as usize];
                        max_ii = j;
                    }
                    j -= 1;
                }
            }
            if max_ii >= 0 && max_ii < end_j {
                let tmp = comput_sc(
                    &a[i as usize],
                    &a[max_ii as usize],
                    max_dist_x,
                    max_dist_y,
                    bw,
                    chn_pen_gap,
                );
                if tmp != i32::MIN && max_f < tmp + f[max_ii as usize] {
                    max_f = tmp + f[max_ii as usize];
                    max_j = max_ii;
                }
            }
            f[i as usize] = max_f;
            p[i as usize] = max_j;
            v[i as usize] = if max_j >= 0 && v[max_j as usize] > max_f {
                v[max_j as usize]
            } else {
                max_f
            };
            if max_ii < 0
                || (a[i as usize].tpos - a[max_ii as usize].tpos
                    <= (max_dist_x + a[i as usize].len) as i64
                    && f[max_ii as usize] < f[i as usize])
            {
                max_ii = i;
            }
            if mmax_f < max_f {
                mmax_f = max_f;
            }
        }

        let mut n_u = 0i32;
        let mut n_v = 0i32;
        let mut u = mb_chain_backtrack(
            km, n, &f, &p, &mut v, &mut t, min_sc, max_drop, &mut n_u, &mut n_v,
        );
        *n_u_ = n_u;
        *_u = u.clone();
        if n_u == 0 {
            return Vec::new();
        }
        let b = compact_a(km, l2b, n_u, &mut u, n_v, v, a);
        *_u = u;
        b
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::l2bit::{l2b_ctg_t, l2b_t};

        #[test]
        fn comput_sc_accepts_colinear_same_target_anchors() {
            let a0 = mb_anchor_t {
                sid: 0,
                len: 10,
                qpos: 19,
                tpos: 119,
                ..Default::default()
            };
            let a1 = mb_anchor_t {
                sid: 0,
                len: 10,
                qpos: 39,
                tpos: 139,
                ..Default::default()
            };
            assert_eq!(comput_sc(&a1, &a0, 100, 100, 20, 0.01), 10);

            let wrong_sid = mb_anchor_t { sid: 1, ..a1 };
            assert_eq!(comput_sc(&wrong_sid, &a0, 100, 100, 20, 0.01), i32::MIN);

            let far_band = mb_anchor_t { tpos: 180, ..a1 };
            assert_eq!(comput_sc(&far_band, &a0, 100, 100, 5, 0.01), i32::MIN);
        }

        #[test]
        fn lchain_dp_chains_simple_colinear_anchors() {
            let l2b = l2b_t {
                tot_len: 1000,
                n_ctg: 1,
                m_ctg: 1,
                ctg: vec![l2b_ctg_t {
                    name: "ctg".to_string(),
                    comm: None,
                    len: 1000,
                    off: 0,
                }],
                ..Default::default()
            };
            let anchors = vec![
                mb_anchor_t {
                    sid: 0,
                    len: 10,
                    qpos: 9,
                    tpos: 109,
                    ..Default::default()
                },
                mb_anchor_t {
                    sid: 0,
                    len: 10,
                    qpos: 29,
                    tpos: 129,
                    ..Default::default()
                },
                mb_anchor_t {
                    sid: 0,
                    len: 10,
                    qpos: 49,
                    tpos: 149,
                    ..Default::default()
                },
            ];
            let mut n_u = 0;
            let mut u = Vec::new();
            let out = mb_lchain_dp(
                (),
                &l2b,
                100,
                100,
                20,
                25,
                5000,
                1,
                0.01,
                anchors.len() as i64,
                anchors,
                &mut n_u,
                &mut u,
            );
            assert_eq!(n_u, 1);
            assert_eq!(u.len(), 1);
            assert_eq!(u[0] as u32, 3);
            assert_eq!((u[0] >> 32) as i32, 30);
            assert_eq!(out.len(), 3);
            assert_eq!(
                out.iter().map(|a| a.qpos).collect::<Vec<_>>(),
                vec![9, 29, 49]
            );
        }

        #[test]
        fn lchain_dp_splits_different_targets() {
            let mut l2b = l2b_t {
                tot_len: 1000,
                n_ctg: 1,
                m_ctg: 1,
                ctg: vec![l2b_ctg_t {
                    name: "ctg".to_string(),
                    comm: None,
                    len: 1000,
                    off: 0,
                }],
                ..Default::default()
            };
            l2b.n_ctg = 2;
            l2b.tot_len = 2000;
            l2b.ctg.push(l2b_ctg_t {
                name: "ctg2".to_string(),
                comm: None,
                len: 1000,
                off: 1000,
            });
            let anchors = vec![
                mb_anchor_t {
                    sid: 0,
                    len: 10,
                    qpos: 9,
                    tpos: 109,
                    ..Default::default()
                },
                mb_anchor_t {
                    sid: 0,
                    len: 10,
                    qpos: 29,
                    tpos: 129,
                    ..Default::default()
                },
                mb_anchor_t {
                    sid: 2,
                    len: 10,
                    qpos: 9,
                    tpos: 209,
                    ..Default::default()
                },
                mb_anchor_t {
                    sid: 2,
                    len: 10,
                    qpos: 29,
                    tpos: 229,
                    ..Default::default()
                },
            ];
            let mut n_u = 0;
            let mut u = Vec::new();
            let out = mb_lchain_dp(
                (),
                &l2b,
                100,
                100,
                20,
                25,
                5000,
                1,
                0.01,
                anchors.len() as i64,
                anchors,
                &mut n_u,
                &mut u,
            );
            assert_eq!(n_u, 2);
            assert_eq!(u.iter().map(|x| *x as u32).collect::<Vec<_>>(), vec![2, 2]);
            assert_eq!(out.len(), 4);
        }
    }
}

pub mod api_test_ex_one {
    #![allow(unused_variables, dead_code, non_snake_case)]

    use crate::align::MB_CIGAR_STR;
    use crate::bseq::{mb_bseq_close, mb_bseq_open, mb_bseq_read};
    use crate::map_algo::{mb_idx_ctg_len, mb_idx_ctg_name, mb_idx_destroy, mb_idx_load, mb_map};
    use crate::options::{mb_opt_init, mb_opt_t};

    /// Original C global function `main` from `minibwa/api-test/ex-one.c:9`.
    pub fn main(argv: &[String]) -> (i32, String) {
        let mut opt = mb_opt_t::default();
        mb_opt_init(&mut opt);
        if argv.len() < 3 {
            return (1, "Usage: mbmap-sgl <idxPrefix> <query.fa>\n".to_string());
        }
        let mut fp = match mb_bseq_open(Some(&argv[2])) {
            Some(fp) => fp,
            None => return (1, String::new()),
        };
        let idx = match mb_idx_load(&argv[1], 0) {
            Some(idx) => idx,
            None => {
                mb_bseq_close(Some(fp));
                return (1, String::new());
            }
        };
        let mut out = String::new();
        loop {
            let mut n_seq = 0;
            let seqs = mb_bseq_read(&mut fp, 1, 0, 0, 0, 1, 1, &mut n_seq);
            if n_seq <= 0 {
                break;
            }
            for s in seqs {
                let mut n_hit = 0;
                let hit = mb_map(
                    &opt,
                    &idx,
                    s.l_seq as i32,
                    &s.seq,
                    0,
                    &mut n_hit,
                    None,
                    Some(&s.name),
                );
                for h in hit.iter().take(n_hit as usize) {
                    let strand = if h.rev() != 0 { '-' } else { '+' };
                    out.push_str(&format!(
                        "{}\t{}\t{}\t{}\t{}\t",
                        s.name, s.l_seq, h.qs, h.qe, strand
                    ));
                    out.push_str(&format!(
                        "{}\t{}\t{}\t{}\t{}\t{}\t{}\tcg:Z:",
                        mb_idx_ctg_name(&idx, h.tid as i32).unwrap_or("*"),
                        mb_idx_ctg_len(&idx, h.tid as i32),
                        h.ts,
                        h.te,
                        h.mlen,
                        h.blen,
                        h.mapq
                    ));
                    if let Some(extra) = &h.p {
                        for &c in extra.cigar().iter().take(extra.n_cigar as usize) {
                            let op = MB_CIGAR_STR.as_bytes()[(c & 0xf) as usize] as char;
                            out.push_str(&format!("{}{}", c >> 4, op));
                        }
                    }
                    out.push('\n');
                }
            }
        }
        mb_idx_destroy(Some(idx));
        mb_bseq_close(Some(fp));
        (0, out)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn api_ex_one_maps_real_chrm_fixture() {
            let argv = vec![
                "mbmap-sgl".to_string(),
                "minibwa/chrM-human".to_string(),
                "minibwa/test/chrM-read_1.fa.gz".to_string(),
            ];
            let (status, out) = main(&argv);
            assert_eq!(status, 0);
            assert!(out.contains("\tcg:Z:"));
        }
    }
}

pub mod api_test_ex_batch {
    #![allow(unused_variables, dead_code, non_snake_case)]

    use crate::align::MB_CIGAR_STR;
    use crate::bseq::{mb_bseq_close, mb_bseq_open, mb_bseq_read};
    use crate::map_algo::{
        mb_idx_ctg_len, mb_idx_ctg_name, mb_idx_destroy, mb_idx_load, mb_map_batch, mb_tbuf_t,
    };
    use crate::options::{mb_opt_init, mb_opt_t};

    /// Original C static function `process_batch` from `minibwa/api-test/ex-batch.c:9`.
    pub fn process_batch(
        opt: &mb_opt_t,
        idx: &crate::map_algo::mb_idx_t,
        tbuf: Option<&mut mb_tbuf_t>,
        n_seq: i32,
        qlen: &[i32],
        seq: &[Box<str>],
        name: &[Box<str>],
    ) -> String {
        let seq_refs = seq.iter().map(|s| &**s).collect::<Vec<_>>();
        let name_refs = name.iter().map(|s| &**s).collect::<Vec<_>>();
        let mut n_hit = vec![0i32; n_seq.max(0) as usize];
        let hit = mb_map_batch(
            opt,
            idx,
            n_seq,
            qlen,
            &seq_refs,
            &mut n_hit,
            tbuf,
            Some(&name_refs),
        );
        let mut out = String::new();
        for k in 0..n_seq as usize {
            for h in hit[k].iter().take(n_hit[k] as usize) {
                let strand = if h.rev() != 0 { '-' } else { '+' };
                out.push_str(&format!(
                    "{}\t{}\t{}\t{}\t{}\t",
                    name[k], qlen[k], h.qs, h.qe, strand
                ));
                out.push_str(&format!(
                    "{}\t{}\t{}\t{}\t{}\t{}\t{}\tcg:Z:",
                    mb_idx_ctg_name(idx, h.tid as i32).unwrap_or("*"),
                    mb_idx_ctg_len(idx, h.tid as i32),
                    h.ts,
                    h.te,
                    h.mlen,
                    h.blen,
                    h.mapq
                ));
                if let Some(extra) = &h.p {
                    for &c in extra.cigar().iter().take(extra.n_cigar as usize) {
                        let op = MB_CIGAR_STR.as_bytes()[(c & 0xf) as usize] as char;
                        out.push_str(&format!("{}{}", c >> 4, op));
                    }
                }
                out.push('\n');
            }
        }
        out
    }

    /// Original C global function `main` from `minibwa/api-test/ex-batch.c:35`.
    pub fn main(argv: &[String]) -> (i32, String) {
        let mut opt = mb_opt_t::default();
        mb_opt_init(&mut opt);
        if argv.len() < 3 {
            return (1, "Usage: mbmap-batch <idxPrefix> <query.fa>\n".to_string());
        }
        let mut fp = match mb_bseq_open(Some(&argv[2])) {
            Some(fp) => fp,
            None => return (1, String::new()),
        };
        let idx = match mb_idx_load(&argv[1], 0) {
            Some(idx) => idx,
            None => {
                mb_bseq_close(Some(fp));
                return (1, String::new());
            }
        };
        let mut out = String::new();
        let mut qlen = Vec::new();
        let mut seq = Vec::new();
        let mut name = Vec::new();
        loop {
            let mut n_read = 0;
            let reads = mb_bseq_read(&mut fp, 1, 0, 0, 0, 1, 1, &mut n_read);
            if n_read <= 0 {
                break;
            }
            for s in reads {
                qlen.push(s.l_seq as i32);
                seq.push(s.seq);
                name.push(s.name);
                if seq.len() >= opt.sb_seq as usize {
                    out.push_str(&process_batch(
                        &opt,
                        &idx,
                        None,
                        seq.len() as i32,
                        &qlen,
                        &seq,
                        &name,
                    ));
                    qlen.clear();
                    seq.clear();
                    name.clear();
                }
            }
        }
        if !seq.is_empty() {
            out.push_str(&process_batch(
                &opt,
                &idx,
                None,
                seq.len() as i32,
                &qlen,
                &seq,
                &name,
            ));
        }
        mb_idx_destroy(Some(idx));
        mb_bseq_close(Some(fp));
        (0, out)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn api_ex_batch_matches_single_api_on_real_chrm_fixture() {
            let argv = vec![
                "mbmap-batch".to_string(),
                "minibwa/chrM-human".to_string(),
                "minibwa/test/chrM-read_1.fa.gz".to_string(),
            ];
            let (status, out) = main(&argv);
            assert_eq!(status, 0);
            assert!(out.contains("\tcg:Z:"));
            let single_argv = vec![
                "mbmap-sgl".to_string(),
                "minibwa/chrM-human".to_string(),
                "minibwa/test/chrM-read_1.fa.gz".to_string(),
            ];
            let (_, single_out) = crate::api_test_ex_one::main(&single_argv);
            assert_eq!(out, single_out);
        }
    }
}

pub mod main {
    #![allow(unused_variables, dead_code, non_snake_case, non_camel_case_types)]

    use crate::bwt::{mb_bwt_destroy, mb_bwt_load, mb_bwt_rank2a, mb_bwt_sa, mb_bwt_sa_batch};
    use crate::fastmap::main_fastmap;
    use crate::index::{
        main_fa2bit, main_genbwt, main_genraw, main_gensa, main_index, main_raw2bwt,
    };
    use crate::ketopt::{ketopt, ko_longopt_t, KETOPT_INIT};
    use crate::kommon::{
        kom_cputime, kom_panic, kom_parse_num, kom_peakrss, kom_realtime, kom_splitmix64,
    };
    use crate::map_main::main_map;

    pub const MB_VERSION: &str = "0.0-r310-dirty";

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(i32)]
    pub enum mb_bench_type_t {
        MB_BENCH_2A = 0,
        MB_BENCH_SA = 1,
        MB_BENCH_MSA = 2,
    }

    /// Original C static function `usage` from `minibwa/main.c:23`.
    pub fn usage(to_stdout: bool, is_long: i32) -> (i32, String) {
        let mut out = String::new();
        out.push_str("Usage: minibwt <command> <arguments>\n");
        out.push_str("Commands:\n");
        if is_long != 0 {
            out.push_str("  General:\n");
            out.push_str("    index      index reference FASTA\n");
            out.push_str("    map        read alignment\n");
            out.push_str("    version    print the version number\n");
            out.push_str("  Separate indexing routines:\n");
            out.push_str("    fa2bit     convert FASTA to the long-2bit format\n");
            out.push_str("    genraw     generate BWT from pac with the BWT-SW algorithm\n");
            out.push_str("    raw2bwt    recode bwtgen raw BWT\n");
            out.push_str("    gensa      generate sampled SA from BWT\n");
            out.push_str("    genbwt     generate BWT+SSA from long-2bit with libsais\n");
            out.push_str("  Debugging:\n");
            out.push_str("    bench      performance evaluation\n");
            out.push_str("    fastmap    test seeding strategies\n");
            out.push_str("  Help:\n");
            out.push_str("    --help     print this help message\n");
        } else {
            out.push_str("  index      index reference FASTA\n");
            out.push_str("  map        read alignment\n");
            out.push_str("  version    print the version number\n");
        }
        (if to_stdout { 0 } else { 1 }, out)
    }

    /// Original C global function `main` from `minibwa/main.c:49`.
    pub fn main(argv: &[String]) -> i32 {
        let argc = argv.len();
        let _ = kom_realtime();
        if argc == 1 {
            return usage(true, 0).0;
        }
        let ret = if argv[1] == "index" {
            main_index(&argv[1..]).0
        } else if argv[1] == "map" || argv[1] == "mem" {
            main_map(&argv[1..]).0
        } else if argv[1] == "fa2bit" {
            main_fa2bit(&argv[1..]).0
        } else if argv[1] == "genraw" {
            main_genraw(&argv[1..]).0
        } else if argv[1] == "raw2bwt" {
            main_raw2bwt(&argv[1..]).0
        } else if argv[1] == "genbwt" {
            main_genbwt(&argv[1..]).0
        } else if argv[1] == "gensa" {
            main_gensa(&argv[1..]).0
        } else if argv[1] == "bench" {
            main_bench(&argv[1..]).0
        } else if argv[1] == "fastmap" {
            main_fastmap(&argv[1..]).0
        } else if argv[1] == "--help" {
            return usage(true, 1).0;
        } else if argv[1] == "version" {
            return 0;
        } else {
            return 1;
        };
        let _ = (ret, kom_cputime(), kom_peakrss());
        0
    }

    /// Original C static function `usage_bench` from `minibwa/main.c:78`.
    pub fn usage_bench(to_stdout: bool, intv: i32) -> (i32, String) {
        let mut out = String::new();
        out.push_str("Usage: minibwa bench [options] <in.mbw>\n");
        out.push_str("Options:\n");
        out.push_str("  -b STR         type: 2a, sa or msa [2a]\n");
        out.push_str("  -n NUM         number of data points [1m]\n");
        out.push_str(&format!(
            "  -v INT         interval size for msa [{}]\n",
            intv
        ));
        out.push_str("  -p             print results for each data point\n");
        out.push_str("  -1             use unbatched sa for msa\n");
        out.push_str("  --help         print this help message\n");
        (if to_stdout { 0 } else { 1 }, out)
    }

    /// Original C global function `main_bench` from `minibwa/main.c:85`.
    pub fn main_bench(argv: &[String]) -> (i32, String, String) {
        let long_opts = [ko_longopt_t {
            name: Some("help".into()),
            has_arg: 0,
            val: 901,
        }];
        let argc = argv.len() as i32;
        let mut args = argv.to_vec();
        let mut o = KETOPT_INIT.clone();
        let mut type_ = mb_bench_type_t::MB_BENCH_2A;
        let mut x = 11u64;
        let mut cs = 1u64;
        let mut n = 1_000_000i64;
        let mut print_val = 0;
        let mut use_single = 0;
        let mut intv = 20i32;
        loop {
            let c = ketopt(&mut o, argc, &mut args, 1, "pn:b:v:1", Some(&long_opts));
            if c < 0 {
                break;
            }
            if c == 'n' as i32 {
                n = kom_parse_num(o.arg.as_deref().unwrap_or("0")).0;
            } else if c == 'p' as i32 {
                print_val = 1;
            } else if c == '1' as i32 {
                use_single = 1;
            } else if c == 'v' as i32 {
                intv = o.arg.as_deref().unwrap_or("0").parse().unwrap_or(0);
            } else if c == 'b' as i32 {
                type_ = match o.arg.as_deref().unwrap_or("2a") {
                    "2a" => mb_bench_type_t::MB_BENCH_2A,
                    "sa" => mb_bench_type_t::MB_BENCH_SA,
                    "msa" => mb_bench_type_t::MB_BENCH_MSA,
                    _ => kom_panic("main_bench", "unknown type"),
                };
            } else if c == 901 {
                let (ret, out) = usage_bench(true, intv);
                return (ret, out, String::new());
            }
        }
        if argc - o.ind < 1 {
            let (ret, err) = usage_bench(false, intv);
            return (ret, String::new(), err);
        }
        let Some(bwt) = mb_bwt_load(&args[o.ind as usize]) else {
            return (1, String::new(), String::new());
        };
        let t = kom_cputime();
        let mut out = String::new();
        if type_ == mb_bench_type_t::MB_BENCH_2A {
            for _ in 0..n {
                let k = kom_splitmix64(&mut x) % bwt.seq_len;
                let l = kom_splitmix64(&mut x) % bwt.seq_len;
                let mut cntk = [0u64; 4];
                let mut cntl = [0u64; 4];
                mb_bwt_rank2a(&bwt, k, l, &mut cntk, &mut cntl);
                cs = cs.wrapping_mul(cntk[1]).wrapping_add(cntl[0]);
                if print_val != 0 {
                    out.push_str(&format!("{}\n", cntk[1]));
                }
            }
        } else if type_ == mb_bench_type_t::MB_BENCH_SA {
            for _ in 0..n {
                let k = kom_splitmix64(&mut x) % bwt.seq_len;
                let s = mb_bwt_sa(&bwt, k);
                cs = cs.wrapping_mul(0xbf58476d1ce4e5b9) ^ s;
                if print_val != 0 {
                    out.push_str(&format!("{s}\n"));
                }
            }
        } else {
            for _ in 0..n {
                let k = kom_splitmix64(&mut x) % bwt.seq_len;
                let l = if k + (intv as u64) < bwt.seq_len {
                    k + intv as u64
                } else {
                    bwt.seq_len
                };
                let mut xor = 0u64;
                if use_single != 0 {
                    for j in k..l {
                        xor ^= mb_bwt_sa(&bwt, j);
                    }
                } else {
                    let n_sa = (l - k) as usize;
                    let mut sa = (0..n_sa).map(|j| k + j as u64).collect::<Vec<_>>();
                    mb_bwt_sa_batch((), &bwt, (l - k) as i64, &mut sa);
                    for s in sa {
                        xor ^= s;
                    }
                }
                cs = cs.wrapping_mul(0xbf58476d1ce4e5b9) ^ xor;
                if print_val != 0 {
                    out.push_str(&format!("{xor}\n"));
                }
            }
        }
        mb_bwt_destroy(Some(bwt));
        let err = format!("checksum = {cs:x}\nt = {:.3}\n", kom_cputime() - t);
        (0, out, err)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn usage_formats_short_and_long_command_lists() {
            let (ret, short) = usage(true, 0);
            assert_eq!(ret, 0);
            assert!(short.contains("Usage: minibwt <command> <arguments>\n"));
            assert!(short.contains("  map        read alignment\n"));
            assert!(!short.contains("Separate indexing routines"));

            let (ret, long) = usage(false, 1);
            assert_eq!(ret, 1);
            assert!(long.contains("    genbwt     generate BWT+SSA from long-2bit with libsais\n"));
            assert!(long.contains("    fastmap    test seeding strategies\n"));
        }

        #[test]
        fn usage_bench_formats_options() {
            assert_eq!(
                (
                    mb_bench_type_t::MB_BENCH_2A as i32,
                    mb_bench_type_t::MB_BENCH_SA as i32,
                    mb_bench_type_t::MB_BENCH_MSA as i32,
                ),
                (0, 1, 2)
            );
            let (ret, out) = usage_bench(false, 20);
            assert_eq!(ret, 1);
            assert!(out.contains("Usage: minibwa bench [options] <in.mbw>\n"));
            assert!(out.contains("  -v INT         interval size for msa [20]\n"));
            assert!(out.contains("  --help         print this help message\n"));
        }
    }
}

pub mod map_algo {
    #![allow(unused_variables, dead_code, non_snake_case, non_camel_case_types)]

    use crate::align::mb_align_skeleton_with_scratch;
    use crate::bwt::mb_sai_v;
    use crate::bwt::{mb_bwt_cache, mb_bwt_load, mb_bwt_t, mb_sai_t};
    use crate::l2bit::{l2b_load, l2b_meth_t, l2b_t};
    use crate::lchain::{mb_anchor_t, mb_lchain_dp};
    use crate::mbpriv::{
        mb_hash64, mb_hash_str, mb_is_sr_mode, KOM_DBG_FLAG, MB_DBG_ANCHOR, MB_DBG_QNAME,
        MB_DBG_SEED,
    };
    use crate::options::{mb_opt_adap, mb_opt_t, MB_F_METH, MB_F_NO_ALN, MB_F_PE, MB_F_PRIMARY5};
    use crate::pe::mb_hit_t;
    use crate::seed::{
        mb_anchor_sort, mb_anchor_v, mb_anchor_with_scratch, mb_seed_intv, mb_seed_intv_batch,
    };
    use std::sync::atomic::Ordering;

    pub const MB_PARENT_UNSET: i32 = -1;
    pub const MB_PARENT_TMP_PRI: i32 = -2;

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct mb_idx_s {
        pub is_meth: i32,
        pub l2b: l2b_t,
        pub bwt: mb_bwt_t,
    }
    pub type mb_idx_t = mb_idx_s;

    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    pub struct mb_tbuf_s {
        pub km: bool,
        pub se_len: Vec<i32>,
        pub se_buf: Vec<u8>,
        pub se_seq_ptrs: Vec<usize>,
        pub se_sai: Vec<mb_sai_v>,
        pub anchor_v: mb_anchor_v,
        pub anchor_aux: Vec<(i64, i64)>,
        pub anchor_sa: Vec<u64>,
        pub anchor_sa_batch: Vec<(u64, u64)>,
        pub anchor_batch: Vec<(i64, i64)>,
        pub chain_w: Vec<u64>,
        pub align_tseq: Vec<u8>,
        pub align_qseq0: Vec<u8>,
    }
    pub type mb_tbuf_t = mb_tbuf_s;

    /// Original C global function `mb_idx_load` from `minibwa/map-algo.c:19`.
    pub fn mb_idx_load(prefix: &str, is_meth: i32) -> Option<mb_idx_t> {
        let t0 = std::env::var_os("MINIBWA_RS_LOAD_TIMING").map(|_| std::time::Instant::now());
        let l2b = l2b_load(format!("{prefix}.l2b"))?;
        let t_l2b = t0.map(|t0| {
            let t_l2b = std::time::Instant::now();
            eprintln!(
                "[minibwa-rs load] l2b {:.3} ms",
                (t_l2b - t0).as_secs_f64() * 1000.0
            );
            t_l2b
        });
        let bwt_name = if is_meth != 0 {
            format!("{prefix}.meth.mbw")
        } else {
            format!("{prefix}.mbw")
        };
        let mut bwt = mb_bwt_load(bwt_name)?;
        let t_bwt = t_l2b.map(|t_l2b| {
            let t_bwt = std::time::Instant::now();
            eprintln!(
                "[minibwa-rs load] bwt {:.3} ms",
                (t_bwt - t_l2b).as_secs_f64() * 1000.0
            );
            t_bwt
        });
        mb_bwt_cache(&mut bwt, 10);
        if let (Some(t0), Some(t_bwt)) = (t0, t_bwt) {
            let t_cache = std::time::Instant::now();
            eprintln!(
                "[minibwa-rs load] cache {:.3} ms",
                (t_cache - t_bwt).as_secs_f64() * 1000.0
            );
            eprintln!(
                "[minibwa-rs load] total {:.3} ms",
                (t_cache - t0).as_secs_f64() * 1000.0
            );
        }
        Some(mb_idx_t {
            is_meth: (is_meth != 0) as i32,
            l2b,
            bwt,
        })
    }

    /// Original C global function `mb_idx_destroy` from `minibwa/map-algo.c:43`.
    pub fn mb_idx_destroy(idx: Option<mb_idx_t>) {
        drop(idx);
    }

    /// Original C global function `mb_idx_ctg_name` from `minibwa/map-algo.c:52`.
    pub fn mb_idx_ctg_name(idx: &mb_idx_t, tid: i32) -> Option<&str> {
        if tid >= 0 && tid < idx.l2b.n_ctg as i32 {
            Some(&idx.l2b.ctg[tid as usize].name)
        } else {
            None
        }
    }

    /// Original C global function `mb_idx_ctg_len` from `minibwa/map-algo.c:57`.
    pub fn mb_idx_ctg_len(idx: &mb_idx_t, tid: i32) -> i64 {
        if tid >= 0 && tid < idx.l2b.n_ctg as i32 {
            idx.l2b.ctg[tid as usize].len as i64
        } else {
            -1
        }
    }

    /// Original C global function `mb_tbuf_init` from `minibwa/map-algo.c:59`.
    pub fn mb_tbuf_init(no_kalloc: i32) -> mb_tbuf_t {
        mb_tbuf_t {
            km: no_kalloc == 0,
            se_len: Vec::new(),
            se_buf: Vec::new(),
            se_seq_ptrs: Vec::new(),
            se_sai: Vec::new(),
            anchor_v: mb_anchor_v::default(),
            anchor_aux: Vec::new(),
            anchor_sa: Vec::new(),
            anchor_sa_batch: Vec::new(),
            anchor_batch: Vec::new(),
            chain_w: Vec::new(),
            align_tseq: Vec::new(),
            align_qseq0: Vec::new(),
        }
    }

    /// Original C global function `mb_tbuf_km` from `minibwa/map-algo.c:67`.
    pub fn mb_tbuf_km(b: &mut mb_tbuf_t) -> bool {
        b.km
    }

    /// Original C global function `mb_tbuf_destroy` from `minibwa/map-algo.c:72`.
    pub fn mb_tbuf_destroy(b: Option<mb_tbuf_t>) {
        drop(b);
    }

    /// Original C global function `mb_tbuf_reset` from `minibwa/map-algo.c:78`.
    pub fn mb_tbuf_reset(b: &mut mb_tbuf_t, max_blk_sz: i64) -> i32 {
        if !b.km {
            return 0;
        }
        0
    }

    /// Original C global function `mb_cal_mblen` from `minibwa/map-algo.c:97`.
    pub fn mb_cal_mblen(n: i32, a: &[mb_anchor_t], blen_: &mut i32) -> i32 {
        *blen_ = 0;
        if n <= 0 {
            return 0;
        }
        let mut mlen = a[0].len as i64;
        let mut blen = a[0].len as i64;
        for i in 1..n as usize {
            let span = a[i].len;
            let tl = a[i].tpos as i32 - a[i - 1].tpos as i32;
            let ql = a[i].qpos - a[i - 1].qpos;
            blen += tl.max(ql) as i64;
            mlen += if tl > span && ql > span {
                span
            } else {
                tl.min(ql)
            } as i64;
        }
        *blen_ = blen as i32;
        mlen as i32
    }

    /// Original C static function `mb_cal_fuzzy_len` from `minibwa/map-algo.c:115`.
    pub fn mb_cal_fuzzy_len(r: &mut mb_hit_t, a: &[mb_anchor_t]) {
        r.mlen = mb_cal_mblen(r.cnt, &a[r.as_ as usize..], &mut r.blen);
    }

    /// Original C static function `mb_hit_set_coor` from `minibwa/map-algo.c:120`.
    pub fn mb_hit_set_coor(r: &mut mb_hit_t, qlen: i32, l2b: &l2b_t, a: &[mb_anchor_t]) {
        let k = r.as_ as usize;
        let ak0 = &a[k];
        let ak1 = &a[k + r.cnt as usize - 1];
        r.tid = (ak0.sid >> 1) as i64;
        r.set_rev((ak0.sid & 1) as u8);
        r.ts = ak0.tpos + 1 - ak0.len as i64;
        r.te = ak1.tpos + 1;
        if r.rev() == 0 {
            r.qs = ak0.qpos + 1 - ak0.len;
            r.qe = ak1.qpos + 1;
        } else {
            r.qs = qlen - (ak1.qpos + 1);
            r.qe = qlen - (ak0.qpos + 1 - ak0.len);
        }
        mb_cal_fuzzy_len(r, a);
    }

    /// Original C global function `mb_cal_high_cov` from `minibwa/map-algo.c:139`.
    pub fn mb_cal_high_cov(km: (), n: i32, sai: &[mb_sai_t], max_occ: i32) -> i32 {
        let mut b = Vec::new();
        for x in sai.iter().take(n as usize) {
            if x.size > max_occ as u64 {
                b.push(x.info);
            }
        }
        if b.is_empty() {
            return 0;
        }
        b.sort_unstable();
        let mut hi_st = (b[0] >> 32) as i32;
        let mut hi_en = b[0] as u32 as i32;
        let mut hi_cov = 0;
        for &x in b.iter().skip(1) {
            let st = (x >> 32) as i32;
            let en = x as u32 as i32;
            if st > hi_en {
                hi_cov += hi_en - hi_st;
                hi_st = st;
                hi_en = en;
            } else {
                hi_en = hi_en.max(en);
            }
        }
        hi_cov + hi_en - hi_st
    }

    /// Original C global function `mb_sync_high_cov` from `minibwa/map-algo.c:165`.
    pub fn mb_sync_high_cov(n: i32, h: &mut [mb_hit_t]) {
        let mut max_frac = 0u8;
        for x in h.iter().take(n as usize) {
            max_frac = max_frac.max(x.frac_high());
        }
        for x in h.iter_mut().take(n as usize) {
            x.set_frac_high(max_frac);
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::l2bit::{l2b_ctg_t, l2b_t};
        use crate::options::{mb_opt_init, mb_opt_t, MB_F_NO_ALN};
        use crate::pe::mb_extra_t;

        #[test]
        fn idx_load_reads_real_chrm_index() {
            let idx = mb_idx_load("minibwa/chrM-human", 0).expect("load index");
            assert_eq!(idx.is_meth, 0);
            assert_eq!(mb_idx_ctg_name(&idx, 0), Some("chrM"));
            assert_eq!(mb_idx_ctg_len(&idx, 0), 16569);
            assert_eq!(mb_idx_ctg_name(&idx, 1), None);
            assert_eq!(mb_idx_ctg_len(&idx, -1), -1);
            assert_eq!(idx.bwt.pre_len, 10);
        }

        #[test]
        fn map_no_aln_finds_real_chrm_hits() {
            let idx = mb_idx_load("minibwa/chrM-human", 0).expect("load index");
            let mut opt = mb_opt_t::default();
            mb_opt_init(&mut opt);
            opt.flag |= MB_F_NO_ALN;
            let seq = "GATCACAGGTCTATCACCCTATTAACCACTCACGGGAGCTCTCCATGCAT";
            let mut n_hit = 0;
            let hits = mb_map(
                &opt,
                &idx,
                seq.len() as i32,
                seq,
                0,
                &mut n_hit,
                None,
                Some("chrM_prefix"),
            );
            assert_eq!(n_hit as usize, hits.len());
            assert!(n_hit > 0);
            assert!(hits.iter().all(|h| h.tid == 0));
            assert!(hits.iter().any(|h| h.qs == 0 && h.qe >= 19));
        }

        #[test]
        fn map_batch_no_aln_matches_single_read_counts() {
            let idx = mb_idx_load("minibwa/chrM-human", 0).expect("load index");
            let mut opt = mb_opt_t::default();
            mb_opt_init(&mut opt);
            opt.flag |= MB_F_NO_ALN;
            opt.sb_seq = 2;
            opt.sb_len = 1000;
            let seqs = [
                "GATCACAGGTCTATCACCCTATTAACCACTCACGGGAGCTCTCCATGCAT",
                "ATCACAGGTCTATCACCCTATTAACCACTCACGGGAGCTCTCCATGCAT",
            ];
            let qlen = seqs.iter().map(|s| s.len() as i32).collect::<Vec<_>>();
            let mut n_hit = vec![0i32; seqs.len()];
            let names = ["r0", "r1"];
            let hits = mb_map_batch(
                &opt,
                &idx,
                seqs.len() as i32,
                &qlen,
                &seqs,
                &mut n_hit,
                None,
                Some(&names),
            );
            assert_eq!(hits.len(), seqs.len());
            for i in 0..seqs.len() {
                let mut n_single = 0;
                let single = mb_map(
                    &opt,
                    &idx,
                    qlen[i],
                    seqs[i],
                    0,
                    &mut n_single,
                    None,
                    Some(names[i]),
                );
                assert_eq!(n_hit[i], n_single);
                assert_eq!(hits[i].len(), single.len());
            }
        }

        #[test]
        fn thread_buffer_flags_follow_no_kalloc() {
            let mut with_km = mb_tbuf_init(0);
            let mut without_km = mb_tbuf_init(1);
            assert!(mb_tbuf_km(&mut with_km));
            assert!(!mb_tbuf_km(&mut without_km));
            assert_eq!(mb_tbuf_reset(&mut with_km, 1024), 0);
            mb_tbuf_destroy(Some(without_km));
        }

        #[test]
        fn mblen_and_hit_coordinates_match_anchor_chain() {
            let anchors = vec![
                mb_anchor_t {
                    sid: 0,
                    len: 10,
                    qpos: 19,
                    tpos: 109,
                    ..Default::default()
                },
                mb_anchor_t {
                    sid: 0,
                    len: 8,
                    qpos: 39,
                    tpos: 129,
                    ..Default::default()
                },
                mb_anchor_t {
                    sid: 0,
                    len: 12,
                    qpos: 59,
                    tpos: 149,
                    ..Default::default()
                },
            ];
            let mut blen = 0;
            assert_eq!(mb_cal_mblen(anchors.len() as i32, &anchors, &mut blen), 30);
            assert_eq!(blen, 50);

            let l2b = l2b_t {
                tot_len: 1000,
                n_ctg: 1,
                m_ctg: 1,
                ctg: vec![l2b_ctg_t {
                    name: "ctg".into(),
                    comm: None,
                    len: 1000,
                    off: 0,
                }],
                ..Default::default()
            };
            let mut hit = mb_hit_t {
                cnt: anchors.len() as i32,
                as_: 0,
                ..Default::default()
            };
            mb_hit_set_coor(&mut hit, 100, &l2b, &anchors);
            assert_eq!((hit.tid, hit.rev(), hit.ts, hit.te), (0, 0, 100, 150));
            assert_eq!((hit.qs, hit.qe, hit.mlen, hit.blen), (10, 60, 30, 50));

            let mut rev = hit.clone();
            let mut rev_anchors = anchors.clone();
            for a in &mut rev_anchors {
                a.sid = 1;
            }
            mb_hit_set_coor(&mut rev, 100, &l2b, &rev_anchors);
            assert_eq!((rev.rev(), rev.qs, rev.qe), (1, 40, 90));
        }

        #[test]
        fn high_cov_intervals_are_merged_and_synced() {
            let sai = vec![
                mb_sai_t {
                    size: 20,
                    info: (10u64 << 32) | 30,
                    ..Default::default()
                },
                mb_sai_t {
                    size: 5,
                    info: (100u64 << 32) | 110,
                    ..Default::default()
                },
                mb_sai_t {
                    size: 30,
                    info: (25u64 << 32) | 40,
                    ..Default::default()
                },
                mb_sai_t {
                    size: 40,
                    info: (50u64 << 32) | 60,
                    ..Default::default()
                },
            ];
            assert_eq!(mb_cal_high_cov((), sai.len() as i32, &sai, 10), 40);

            let mut hits = vec![
                mb_hit_t {
                    flags: mb_hit_t::flags_with(0, 0, 0, 0, 0, 0, 0, 0, 3),
                    ..Default::default()
                },
                mb_hit_t {
                    flags: mb_hit_t::flags_with(0, 0, 0, 0, 0, 0, 0, 0, 9),
                    ..Default::default()
                },
                mb_hit_t {
                    flags: mb_hit_t::flags_with(0, 0, 0, 0, 0, 0, 0, 0, 1),
                    ..Default::default()
                },
            ];
            mb_sync_high_cov(hits.len() as i32, &mut hits);
            assert_eq!(
                hits.iter().map(|h| h.frac_high()).collect::<Vec<_>>(),
                vec![9, 9, 9]
            );
        }

        #[test]
        fn generated_hits_are_sorted_and_coordinate_filled() {
            let l2b = l2b_t {
                tot_len: 1000,
                n_ctg: 1,
                m_ctg: 1,
                ctg: vec![l2b_ctg_t {
                    name: "ctg".into(),
                    comm: None,
                    len: 1000,
                    off: 0,
                }],
                ..Default::default()
            };
            let anchors = vec![
                mb_anchor_t {
                    sid: 0,
                    len: 10,
                    qpos: 19,
                    tpos: 109,
                    ..Default::default()
                },
                mb_anchor_t {
                    sid: 0,
                    len: 8,
                    qpos: 39,
                    tpos: 129,
                    ..Default::default()
                },
                mb_anchor_t {
                    sid: 0,
                    len: 12,
                    qpos: 79,
                    tpos: 179,
                    ..Default::default()
                },
            ];
            let u = vec![(50u64 << 32) | 2, (80u64 << 32) | 1];
            let hits = mb_gen_hit((), 17, 100, &l2b, u.len() as i32, &u, &anchors);
            assert_eq!(hits.len(), 2);
            assert!(hits[0].score >= hits[1].score);
            assert_eq!((hits[0].id, hits[0].parent), (0, MB_PARENT_UNSET));
            assert_eq!((hits[0].score, hits[0].score0, hits[0].cnt), (80, 80, 1));
            assert_eq!(
                (hits[0].as_, hits[0].qs, hits[0].qe, hits[0].ts, hits[0].te),
                (2, 68, 80, 168, 180)
            );
            assert_eq!((hits[1].score, hits[1].cnt, hits[1].as_), (50, 2, 0));
        }

        #[test]
        fn split_hit_recalculates_both_halves() {
            let l2b = l2b_t {
                tot_len: 1000,
                n_ctg: 1,
                m_ctg: 1,
                ctg: vec![l2b_ctg_t {
                    name: "ctg".into(),
                    comm: None,
                    len: 1000,
                    off: 0,
                }],
                ..Default::default()
            };
            let anchors = vec![
                mb_anchor_t {
                    sid: 0,
                    len: 10,
                    qpos: 19,
                    tpos: 109,
                    ..Default::default()
                },
                mb_anchor_t {
                    sid: 0,
                    len: 8,
                    qpos: 39,
                    tpos: 129,
                    ..Default::default()
                },
                mb_anchor_t {
                    sid: 0,
                    len: 12,
                    qpos: 59,
                    tpos: 149,
                    ..Default::default()
                },
            ];
            let mut left = mb_hit_t {
                id: 7,
                parent: 7,
                cnt: 3,
                score: 99,
                as_: 0,
                flags: mb_hit_t::flags_with(0, 0, 1, 0, 0, 0, 0, 0, 0),
                p: Some(mb_extra_t::default().boxed()),
                ..Default::default()
            };
            mb_hit_set_coor(&mut left, 100, &l2b, &anchors);
            let mut right = mb_hit_t::default();
            mb_split_hit(&mut left, &mut right, 1, 100, &anchors, &l2b);
            assert_eq!(
                (left.cnt, left.score, left.qs, left.qe, left.split()),
                (1, 33, 10, 20, 1)
            );
            assert_eq!(
                (right.id, right.parent, right.cnt, right.score, right.as_),
                (-1, MB_PARENT_TMP_PRI, 2, 66, 1)
            );
            assert_eq!(
                (
                    right.sam_pri(),
                    right.p.is_none(),
                    right.qs,
                    right.qe,
                    right.split()
                ),
                (0, true, 32, 60, 2)
            );
        }

        #[test]
        fn parent_and_sync_logic_matches_primary_selection() {
            let mut hits = vec![
                mb_hit_t {
                    id: 5,
                    parent: 5,
                    qs: 0,
                    qe: 100,
                    score: 100,
                    ..Default::default()
                },
                mb_hit_t {
                    id: 8,
                    parent: 5,
                    qs: 10,
                    qe: 90,
                    score: 80,
                    ..Default::default()
                },
                mb_hit_t {
                    id: 9,
                    parent: MB_PARENT_TMP_PRI,
                    qs: 120,
                    qe: 160,
                    score: 40,
                    ..Default::default()
                },
            ];
            mb_sync_hits((), hits.len() as i32, &mut hits);
            assert_eq!(
                hits.iter().map(|h| (h.id, h.parent)).collect::<Vec<_>>(),
                vec![(0, 0), (1, 0), (2, 2)]
            );
            assert_eq!(
                hits.iter().map(|h| h.sam_pri()).collect::<Vec<_>>(),
                vec![1, 0, 0]
            );

            mb_set_sam_pri(hits.len() as i32, &mut hits, 1);
            assert_eq!(
                hits.iter().map(|h| h.sam_pri()).collect::<Vec<_>>(),
                vec![1, 0, 0]
            );

            let mut parent_hits = vec![
                mb_hit_t {
                    qs: 0,
                    qe: 100,
                    score: 100,
                    parent: MB_PARENT_UNSET,
                    ..Default::default()
                },
                mb_hit_t {
                    qs: 20,
                    qe: 80,
                    score: 60,
                    parent: MB_PARENT_UNSET,
                    ..Default::default()
                },
                mb_hit_t {
                    qs: 130,
                    qe: 170,
                    score: 30,
                    parent: MB_PARENT_UNSET,
                    ..Default::default()
                },
            ];
            mb_set_parent(
                (),
                0.5,
                10,
                parent_hits.len() as i32,
                &mut parent_hits,
                0,
                0,
            );
            assert_eq!(
                parent_hits
                    .iter()
                    .map(|h| (h.id, h.parent))
                    .collect::<Vec<_>>(),
                vec![(0, 0), (1, 0), (2, 2)]
            );
        }

        #[test]
        fn select_sub_sort_and_filter_compact_hits() {
            let mut hits = vec![
                mb_hit_t {
                    id: 0,
                    parent: 0,
                    qs: 0,
                    qe: 100,
                    tid: 0,
                    ts: 0,
                    te: 100,
                    score: 100,
                    hash: 3,
                    ..Default::default()
                },
                mb_hit_t {
                    id: 1,
                    parent: 0,
                    qs: 10,
                    qe: 90,
                    tid: 0,
                    ts: 10,
                    te: 90,
                    score: 80,
                    hash: 1,
                    p: Some(
                        mb_extra_t {
                            dp_max: 90,
                            ..Default::default()
                        }
                        .boxed(),
                    ),
                    ..Default::default()
                },
                mb_hit_t {
                    id: 2,
                    parent: 0,
                    qs: 0,
                    qe: 100,
                    tid: 0,
                    ts: 0,
                    te: 100,
                    score: 70,
                    hash: 9,
                    ..Default::default()
                },
            ];
            let mut n = hits.len() as i32;
            mb_select_sub((), 0.5, 0, 1, &mut n, &mut hits);
            assert_eq!(n, 2);
            assert_eq!(
                hits.iter().map(|h| (h.id, h.parent)).collect::<Vec<_>>(),
                vec![(0, 0), (1, 0)]
            );

            hits[0].score = 30;
            hits[1].score = 20;
            hits[1].p = Some(
                mb_extra_t {
                    dp_max: 120,
                    ..Default::default()
                }
                .boxed(),
            );
            mb_hit_sort((), &mut n, &mut hits);
            assert_eq!(hits.iter().map(|h| h.hash).collect::<Vec<_>>(), vec![1, 3]);

            let mut opt = mb_opt_t::default();
            mb_opt_init(&mut opt);
            opt.min_chain_score = 40;
            opt.min_dp_max = 50;
            opt.a = 2;
            hits[0].mlen = 100;
            hits[1].mlen = 10;
            hits[1].p = Some(
                mb_extra_t {
                    dp_max: 200,
                    ..Default::default()
                }
                .boxed(),
            );
            mb_filter_hits(&opt, 100, &mut n, &mut hits);
            assert_eq!(n, 1);
            assert_eq!(hits[0].hash, 1);
        }

        #[test]
        fn squeeze_anchors_updates_hit_offsets_in_offset_order() {
            let mut anchors = vec![
                mb_anchor_t {
                    qpos: 0,
                    ..Default::default()
                },
                mb_anchor_t {
                    qpos: 1,
                    ..Default::default()
                },
                mb_anchor_t {
                    qpos: 20,
                    ..Default::default()
                },
                mb_anchor_t {
                    qpos: 21,
                    ..Default::default()
                },
                mb_anchor_t {
                    qpos: 40,
                    ..Default::default()
                },
            ];
            let mut hits = vec![
                mb_hit_t {
                    as_: 4,
                    cnt: 1,
                    ..Default::default()
                },
                mb_hit_t {
                    as_: 2,
                    cnt: 2,
                    ..Default::default()
                },
            ];
            let n = mb_squeeze_a((), hits.len() as i32, &mut hits, &mut anchors);
            assert_eq!(n, 3);
            assert_eq!(hits.iter().map(|h| h.as_).collect::<Vec<_>>(), vec![2, 0]);
            assert_eq!(
                anchors.iter().take(3).map(|a| a.qpos).collect::<Vec<_>>(),
                vec![20, 21, 40]
            );
        }

        #[test]
        fn mapq_sets_primary_secondary_and_inversion_neighbor_quality() {
            let mut hits = vec![
                mb_hit_t {
                    id: 0,
                    parent: 0,
                    score: 100,
                    score0: 100,
                    subsc: 40,
                    mlen: 90,
                    blen: 100,
                    p: Some(
                        mb_extra_t {
                            dp_max: 120,
                            dp_max2: 60,
                            ..Default::default()
                        }
                        .boxed(),
                    ),
                    tid: 0,
                    ts: 10,
                    ..Default::default()
                },
                mb_hit_t {
                    id: 1,
                    parent: 0,
                    score: 70,
                    score0: 70,
                    tid: 0,
                    ts: 30,
                    ..Default::default()
                },
                mb_hit_t {
                    id: 2,
                    parent: 2,
                    flags: mb_hit_t::flags_with(0, 0, 0, 0, 1, 0, 0, 0, 0),
                    tid: 0,
                    ts: 20,
                    ..Default::default()
                },
                mb_hit_t {
                    id: 3,
                    parent: 3,
                    score: 90,
                    score0: 90,
                    subsc: 40,
                    mlen: 85,
                    blen: 100,
                    p: Some(
                        mb_extra_t {
                            dp_max: 100,
                            dp_max2: 40,
                            ..Default::default()
                        }
                        .boxed(),
                    ),
                    tid: 0,
                    ts: 40,
                    ..Default::default()
                },
            ];
            mb_set_mapq((), 150, hits.len() as i32, &mut hits, 40, 2, 1, 325);
            assert!(hits[0].mapq > 0);
            assert_eq!(hits[1].mapq, 0);
            assert_eq!(hits[2].mapq, hits[0].mapq.min(hits[3].mapq));
        }
    }

    /// Original C global function `mb_gen_hit` from `minibwa/map-algo.c:174`.
    pub fn mb_gen_hit(
        km: (),
        hash: u32,
        qlen: i32,
        l2b: &l2b_t,
        n_u: i32,
        u: &[u64],
        a: &[mb_anchor_t],
    ) -> Vec<mb_hit_t> {
        if n_u <= 0 {
            return Vec::new();
        }
        let mut z = Vec::with_capacity(n_u as usize);
        let mut k = 0usize;
        for &ui in u.iter().take(n_u as usize) {
            let h = (mb_hash64(
                (mb_hash64(a[k].tpos as u64).wrapping_add(mb_hash64(a[k].qpos as u64)))
                    ^ hash as u64,
            ) & 0xffff_ffff) as u32;
            let x = ui ^ h as u64;
            let y = ((k as u64) << 32) | (ui as u32 as u64);
            z.push((x, y));
            k += ui as u32 as usize;
        }
        z.sort_by_key(|&(x, _)| x);
        z.reverse();

        let mut r = Vec::with_capacity(n_u as usize);
        for (i, &(x, y)) in z.iter().enumerate() {
            let mut ri = mb_hit_t {
                id: i as i32,
                parent: MB_PARENT_UNSET,
                score: (x >> 32) as i32,
                score0: (x >> 32) as i32,
                hash: x as u32,
                cnt: y as u32 as i32,
                as_: (y >> 32) as i32,
                ..Default::default()
            };
            mb_hit_set_coor(&mut ri, qlen, l2b, a);
            r.push(ri);
        }
        r
    }

    /// Original C global function `mb_split_hit` from `minibwa/map-algo.c:211`.
    pub fn mb_split_hit(
        r: &mut mb_hit_t,
        r2: &mut mb_hit_t,
        n: i32,
        qlen: i32,
        a: &[mb_anchor_t],
        l2b: &l2b_t,
    ) {
        if n <= 0 || n >= r.cnt {
            return;
        }
        *r2 = r.clone();
        r2.id = -1;
        r2.set_sam_pri(0);
        r2.p = None;
        r2.set_split_inv(0);
        r2.cnt = r.cnt - n;
        r2.score = ((r.score as f32 * (r2.cnt as f32 / r.cnt as f32)) + 0.499) as i32;
        r2.as_ = r.as_ + n;
        if r.parent == r.id {
            r2.parent = MB_PARENT_TMP_PRI;
        }
        mb_hit_set_coor(r2, qlen, l2b, a);
        r.cnt -= r2.cnt;
        r.score -= r2.score;
        mb_hit_set_coor(r, qlen, l2b, a);
        r.set_split(r.split() | 1);
        r2.set_split(r2.split() | 2);
    }

    /// Original C global function `mb_sync_hits` from `minibwa/map-algo.c:230`.
    pub fn mb_sync_hits(km: (), n_regs: i32, regs: &mut [mb_hit_t]) {
        if n_regs <= 0 {
            return;
        }
        let n = n_regs as usize;
        let mut max_id = -1;
        for r in regs.iter().take(n) {
            max_id = max_id.max(r.id);
        }
        let n_tmp = max_id + 1;
        let mut tmp = vec![-1i32; n_tmp.max(0) as usize];
        for (i, r) in regs.iter().take(n).enumerate() {
            if r.id >= 0 {
                tmp[r.id as usize] = i as i32;
            }
        }
        for i in 0..n {
            let old_parent = regs[i].parent;
            regs[i].id = i as i32;
            if old_parent == MB_PARENT_TMP_PRI {
                regs[i].parent = i as i32;
            } else if old_parent >= 0
                && (old_parent as usize) < tmp.len()
                && tmp[old_parent as usize] >= 0
            {
                regs[i].parent = tmp[old_parent as usize];
            } else {
                regs[i].parent = MB_PARENT_UNSET;
            }
        }
        mb_set_sam_pri(n_regs, regs, 0);
    }

    /// Original C static function `update_sub` from `minibwa/map-algo.c:258`.
    pub fn update_sub(
        ri: &mut mb_hit_t,
        rp: &mut mb_hit_t,
        mask_level: f32,
        mask_len: i32,
        sub_diff: i32,
        uncov_len: i32,
    ) -> i32 {
        let si = ri.qs;
        let ei = ri.qe;
        let sj = rp.qs;
        let ej = rp.qe;
        if ej <= si || sj >= ei {
            return 0;
        }
        let min = (ej - sj).min(ei - si);
        let max = (ej - sj).max(ei - si);
        let ol = ei.min(ej) - si.max(sj);
        if ol as f64 / min as f64 - uncov_len as f64 / max as f64 > mask_level as f64
            && uncov_len <= mask_len
        {
            let mut cnt_sub = 0;
            let mut sci = ri.score;
            ri.parent = rp.parent;
            rp.subsc = rp.subsc.max(sci);
            if rp.p.is_some()
                && ri.p.is_some()
                && (rp.tid != ri.tid || rp.ts != ri.ts || rp.te != ri.te || ol != min)
            {
                let ri_dp_max = ri.p.as_ref().unwrap().dp_max;
                let rp_extra = rp.p.as_mut().unwrap();
                sci = ri_dp_max;
                rp_extra.dp_max2 = rp_extra.dp_max2.max(sci);
                if rp_extra.dp_max - ri_dp_max <= sub_diff {
                    cnt_sub = 1;
                }
            }
            if cnt_sub != 0 {
                rp.n_sub += 1;
            }
            1
        } else {
            0
        }
    }

    /// Original C global function `mb_set_parent` from `minibwa/map-algo.c:279`.
    pub fn mb_set_parent(
        km: (),
        mask_level: f32,
        mask_len: i32,
        n: i32,
        r: &mut [mb_hit_t],
        sub_diff: i32,
        hard_mask_level: i32,
    ) {
        if n <= 0 {
            return;
        }
        let n = n as usize;
        for (i, ri) in r.iter_mut().take(n).enumerate() {
            ri.id = i as i32;
        }
        let mut w = vec![0i32; n];
        let mut k = 1usize;
        r[0].parent = 0;
        for i in 1..n {
            let si = r[i].qs;
            let ei = r[i].qe;
            let mut cov = Vec::new();
            let mut uncov_len = 0i32;
            if hard_mask_level == 0 {
                for &wj in w.iter().take(k) {
                    let rp = &r[wj as usize];
                    let mut sj = rp.qs;
                    let mut ej = rp.qe;
                    if ej <= si || sj >= ei {
                        continue;
                    }
                    if sj < si {
                        sj = si;
                    }
                    if ej > ei {
                        ej = ei;
                    }
                    cov.push((sj as u64) << 32 | ej as u32 as u64);
                }
                if cov.is_empty() {
                    w[k] = i as i32;
                    k += 1;
                    r[i].parent = i as i32;
                    r[i].n_sub = 0;
                    continue;
                }
                cov.sort_unstable();
                let mut x = si;
                for &cv in &cov {
                    let st = (cv >> 32) as i32;
                    let en = cv as u32 as i32;
                    if st > x {
                        uncov_len += st - x;
                    }
                    x = x.max(en);
                }
                if ei > x {
                    uncov_len += ei - x;
                }
            }

            let mut max_ol = 0;
            let mut max_j = -1i32;
            for (j, &wj) in w.iter().take(k).enumerate() {
                let rp = &r[wj as usize];
                let sj = rp.qs;
                let ej = rp.qe;
                let ol = if ej <= si || sj >= ei {
                    0
                } else {
                    ei.min(ej) - si.max(sj)
                };
                if max_ol < ol {
                    max_ol = ol;
                    max_j = j as i32;
                }
            }
            let mut n_par = 0;
            if max_j >= 0 {
                let wi = w[max_j as usize] as usize;
                if wi < i {
                    let (left, right) = r.split_at_mut(i);
                    n_par += update_sub(
                        &mut right[0],
                        &mut left[wi],
                        mask_level,
                        mask_len,
                        sub_diff,
                        uncov_len,
                    );
                } else {
                    let (left, right) = r.split_at_mut(wi);
                    n_par += update_sub(
                        &mut left[i],
                        &mut right[0],
                        mask_level,
                        mask_len,
                        sub_diff,
                        uncov_len,
                    );
                }
                let mut j = 0usize;
                while j < k && n_par == 0 {
                    let wi = w[j] as usize;
                    if wi < i {
                        let (left, right) = r.split_at_mut(i);
                        n_par += update_sub(
                            &mut right[0],
                            &mut left[wi],
                            mask_level,
                            mask_len,
                            sub_diff,
                            uncov_len,
                        );
                    } else {
                        let (left, right) = r.split_at_mut(wi);
                        n_par += update_sub(
                            &mut left[i],
                            &mut right[0],
                            mask_level,
                            mask_len,
                            sub_diff,
                            uncov_len,
                        );
                    }
                    j += 1;
                }
            }
            if n_par == 0 {
                w[k] = i as i32;
                k += 1;
                r[i].parent = i as i32;
                r[i].n_sub = 0;
            }
        }
    }

    /// Original C global function `mb_set_sam_pri` from `minibwa/map-algo.c:330`.
    pub fn mb_set_sam_pri(n: i32, r: &mut [mb_hit_t], is_primary5: i32) {
        if n <= 0 {
            return;
        }
        let mut n_pri = 0;
        let mut min_i = -1;
        let mut min_qs = -1;
        let mut first_i = -1;
        for (i, ri) in r.iter_mut().take(n as usize).enumerate() {
            ri.set_sam_pri(0);
            if ri.id != ri.parent {
                continue;
            }
            n_pri += 1;
            if n_pri == 1 {
                first_i = i as i32;
            }
            if min_qs < 0 || ri.qs < min_qs {
                min_i = i as i32;
                min_qs = ri.qs;
            }
        }
        assert!(n_pri > 0);
        if is_primary5 != 0 {
            r[min_i as usize].set_sam_pri(1);
        } else {
            r[first_i as usize].set_sam_pri(1);
        }
    }

    /// Original C global function `mb_select_sub` from `minibwa/map-algo.c:346`.
    pub fn mb_select_sub(
        km: (),
        pri_ratio: f32,
        min_diff: i32,
        best_n: i32,
        n_: &mut i32,
        r: &mut Vec<mb_hit_t>,
    ) {
        if pri_ratio > 0.0 && *n_ > 0 {
            let n = *n_ as usize;
            let mut keep = vec![0u8; n];
            let mut n_2nd = 0;
            for i in 0..n {
                let p = r[i].parent;
                if p == i as i32 || r[i].inv() != 0 {
                    keep[i] = 1;
                } else if p >= 0
                    && (p as usize) < r.len()
                    && ((r[i].score as f32) >= r[p as usize].score as f32 * pri_ratio
                        || r[i].score + min_diff >= r[p as usize].score)
                    && n_2nd < best_n
                    && !(r[i].qs == r[p as usize].qs
                        && r[i].qe == r[p as usize].qe
                        && r[i].tid == r[p as usize].tid
                        && r[i].ts == r[p as usize].ts
                        && r[i].te == r[p as usize].te)
                {
                    keep[i] = 1;
                    n_2nd += 1;
                }
            }
            let mut k = 0usize;
            for i in 0..n {
                if keep[i] != 0 {
                    if k < i {
                        r[k] = std::mem::take(&mut r[i]);
                    }
                    k += 1;
                } else if r[i].p.is_some() {
                    r[i].p = None;
                }
            }
            if k != n {
                r.truncate(k);
                mb_sync_hits(km, k as i32, r);
            }
            *n_ = k as i32;
        }
    }

    /// Original C global function `mb_hit_sort` from `minibwa/map-algo.c:366`.
    pub fn mb_hit_sort(km: (), n_regs: &mut i32, r: &mut Vec<mb_hit_t>) {
        let n = *n_regs as usize;
        if n <= 1 {
            return;
        }
        let mut aux = Vec::new();
        for i in 0..n {
            if r[i].inv() != 0 || r[i].cnt >= 0 {
                let score = r[i].p.as_ref().map(|p| p.dp_max).unwrap_or(r[i].score);
                aux.push((((score as u64) << 32) | r[i].hash as u64, i));
            } else if r[i].p.is_some() {
                r[i].p = None;
            }
        }
        aux.sort_by_key(|&(x, _)| x);
        let mut old = std::mem::take(r);
        let mut t = Vec::with_capacity(aux.len());
        for &(_, idx) in aux.iter().rev() {
            t.push(std::mem::take(&mut old[idx]));
        }
        *n_regs = t.len() as i32;
        *r = t;
    }

    /// Original C global function `mb_filter_hits` from `minibwa/map-algo.c:394`.
    pub fn mb_filter_hits(opt: &mb_opt_t, qlen: i32, n_regs: &mut i32, regs: &mut Vec<mb_hit_t>) {
        let mut k = 0usize;
        for i in 0..*n_regs as usize {
            let mut flt = regs[i].flt() != 0;
            if regs[i].p.is_some() {
                let dp_max = regs[i].p.as_ref().unwrap().dp_max;
                if regs[i].mlen < opt.min_chain_score {
                    flt = true;
                } else if dp_max < opt.min_dp_max * opt.a {
                    flt = true;
                }
                if flt {
                    regs[i].p = None;
                }
            }
            if !flt {
                if k < i {
                    regs[k] = std::mem::take(&mut regs[i]);
                }
                k += 1;
            }
        }
        regs.truncate(k);
        *n_regs = k as i32;
    }

    /// Original C global function `mb_squeeze_a` from `minibwa/map-algo.c:413`.
    pub fn mb_squeeze_a(km: (), n_regs: i32, regs: &mut [mb_hit_t], a: &mut [mb_anchor_t]) -> i32 {
        let mut aux = Vec::with_capacity(n_regs as usize);
        for (i, r) in regs.iter().take(n_regs as usize).enumerate() {
            aux.push(((r.as_ as u64) << 32) | i as u32 as u64);
        }
        aux.sort_unstable();
        let mut as_ = 0usize;
        for &x in &aux {
            let i = x as u32 as usize;
            let r = &mut regs[i];
            if r.as_ as usize != as_ {
                let src = r.as_ as usize;
                let cnt = r.cnt as usize;
                a.copy_within(src..src + cnt, as_);
                r.as_ = as_ as i32;
            }
            as_ += r.cnt as usize;
        }
        as_ as i32
    }

    /// Original C static function `mb_set_inv_mapq` from `minibwa/map-algo.c:437`.
    pub fn mb_set_inv_mapq(km: (), n_regs: i32, regs: &mut [mb_hit_t]) {
        let n = n_regs as usize;
        if n_regs < 3 {
            return;
        }
        if !regs.iter().take(n).any(|r| r.inv() != 0) {
            return;
        }
        let mut aux = Vec::new();
        for (i, r) in regs.iter().take(n).enumerate() {
            if r.parent == i as i32 || r.parent < 0 {
                aux.push((((r.tid as u64) << 32) | r.ts as u64, i));
            }
        }
        aux.sort_by_key(|&(x, _)| x);
        for i in 1..aux.len().saturating_sub(1) {
            let inv_i = aux[i].1;
            if regs[inv_i].inv() != 0 {
                let l_mapq = regs[aux[i - 1].1].mapq;
                let r_mapq = regs[aux[i + 1].1].mapq;
                regs[inv_i].mapq = l_mapq.min(r_mapq);
            }
        }
    }

    /// Original C global function `mb_set_mapq` from `minibwa/map-algo.c:463`.
    pub fn mb_set_mapq(
        km: (),
        qlen: i32,
        n_regs: i32,
        regs: &mut [mb_hit_t],
        min_chain_sc: i32,
        match_sc: i32,
        is_sr: i32,
        max_sr_len: i32,
    ) {
        const MAPQ_COEF_LEN: i32 = 50;
        const MAPQ_COEF_FAC: f64 = 3.0;
        const Q_COEF: f64 = 40.0;
        if n_regs == 0 {
            return;
        }
        for r in regs.iter_mut().take(n_regs as usize) {
            if r.inv() != 0 {
                r.mapq = 0;
            } else if r.parent == r.id {
                let subsc = r.subsc.max(min_chain_sc);
                let pen_chn = if (r.score as f64) > qlen as f64 * 0.1 {
                    1.0
                } else {
                    10.0 * r.score as f64 / qlen as f64
                };
                let mut mapq;
                if let Some(p) = &r.p {
                    if p.dp_max2 > 0 && p.dp_max > 0 {
                        let identity = r.mlen as f64 / r.blen as f64;
                        let mut x = if r.blen < MAPQ_COEF_LEN {
                            1.0
                        } else {
                            MAPQ_COEF_FAC / (r.blen as f64).ln()
                        };
                        x *= identity * identity;
                        let mapq_sr = (6.02 * x * x * (p.dp_max - p.dp_max2) as f64
                            / match_sc as f64
                            + 0.499) as i32;
                        x = p.dp_max2 as f64 / p.dp_max as f64;
                        if subsc > r.score0 {
                            x *= subsc as f64 / r.score0 as f64;
                        }
                        let mapq_lr = (pen_chn
                            * identity
                            * Q_COEF
                            * (1.0 - x * x)
                            * (p.dp_max as f64 / match_sc as f64).ln())
                            as i32;
                        if is_sr != 0 {
                            mapq = mapq_sr;
                        } else if max_sr_len < 0 {
                            mapq = mapq_lr;
                        } else {
                            mapq = if qlen < max_sr_len {
                                mapq_sr
                            } else {
                                (mapq_lr as f64
                                    - (mapq_lr - mapq_sr) as f64
                                        * 2.0f64.powf(1.0 - qlen as f64 / max_sr_len as f64)
                                    + 0.499) as i32
                            };
                        }
                    } else {
                        let x = subsc as f64 / r.score0 as f64;
                        let identity = r.mlen as f64 / r.blen as f64;
                        mapq = (pen_chn
                            * identity
                            * Q_COEF
                            * (1.0 - x)
                            * (p.dp_max as f64 / match_sc as f64).ln())
                            as i32;
                    }
                } else {
                    let x = subsc as f64 / r.score0 as f64;
                    mapq = (pen_chn * Q_COEF * (1.0 - x) * (r.score as f64).ln()) as i32;
                }
                mapq -= (4.343f64 * ((r.n_sub + 1) as f64).ln() + 0.499) as i32;
                mapq = mapq.max(0);
                r.mapq = mapq.min(60);
                if let Some(p) = &r.p {
                    if p.dp_max > p.dp_max2 && r.mapq == 0 {
                        r.mapq = 1;
                    }
                }
            } else {
                r.mapq = 0;
            }
        }
        mb_set_inv_mapq(km, n_regs, regs);
    }

    /// Original C static function `mb_dbg_seed` from `minibwa/map-algo.c:513`.
    pub fn mb_dbg_seed(n: i64, u: &[mb_sai_t], qname: Option<&str>) {
        let name = qname.unwrap_or("*");
        for p in u.iter().take(n as usize) {
            eprintln!(
                "SD\t{}\t{}\t{}\t{}",
                name,
                (p.info >> 32) as i32,
                p.info as u32 as i32,
                p.size as i64
            );
        }
    }

    /// Original C static function `mb_dbg_anchor` from `minibwa/map-algo.c:522`.
    pub fn mb_dbg_anchor(
        idx: &mb_idx_t,
        qlen: i32,
        n: i64,
        a: &[mb_anchor_t],
        qname: Option<&str>,
    ) {
        let name = qname.unwrap_or("*");
        for ai in a.iter().take(n as usize) {
            let rid = (ai.sid >> 1) as usize;
            let rev = ai.sid & 1;
            let qs = if rev != 0 {
                qlen - 1 - ai.qpos
            } else {
                ai.qpos + 1 - ai.len
            };
            let ts = ai.tpos + 1 - ai.len as i64;
            let strand = if rev != 0 { '-' } else { '+' };
            eprintln!(
                "AC\t{}\t{}\t{}\t{}\t{}\t{}",
                name, qs, strand, idx.l2b.ctg[rid].name, ts, ai.len
            );
        }
    }

    /// Original C global function `mb_map_sai` from `minibwa/map-algo.c:535`.
    pub fn mb_map_sai(
        opt: &mb_opt_t,
        idx: &mb_idx_t,
        qlen: i64,
        seq: &[u8],
        mt: l2b_meth_t,
        u: &mut mb_sai_v,
        n_hit_: &mut i32,
        b: &mut mb_tbuf_t,
        qname: Option<&str>,
    ) -> Vec<mb_hit_t> {
        const MIN_RECHAIN_LEN: i32 = 1000;
        const MIN_RECHAIN_RATIO: f64 = 0.1;
        *n_hit_ = 0;
        if u.n == 0 {
            u.a.clear();
            return Vec::new();
        }
        let mut hash = qname.map(mb_hash_str).unwrap_or(0);
        hash ^= (mb_hash64(qlen as u64).wrapping_add(mb_hash64(opt.seed as u64))) as u32;
        hash = mb_hash64(hash as u64) as u32;
        let hi_cov = mb_cal_high_cov((), u.n as i32, &u.a, opt.max_occ);
        let is_sr = mb_is_sr_mode(opt, qlen as i32);
        let sub_diff = (opt.a + opt.b).max(opt.q + opt.e);
        let chn_pen_gap = opt.chain_gap_scale * 0.01 * opt.min_len as f32;

        let mut v = std::mem::take(&mut b.anchor_v);
        if (KOM_DBG_FLAG.load(Ordering::Relaxed) & MB_DBG_QNAME) != 0 {
            eprintln!("QN\t{}", qname.unwrap_or(""));
        }
        if (KOM_DBG_FLAG.load(Ordering::Relaxed) & MB_DBG_SEED) != 0 {
            mb_dbg_seed(u.n as i64, &u.a, qname);
        }
        crate::stage_time::measure(crate::stage_time::Bucket::Anchor, || {
            mb_anchor_with_scratch(
                (),
                idx,
                u,
                qlen as i32,
                mt,
                opt.max_occ,
                &mut v,
                &mut b.anchor_aux,
                &mut b.anchor_sa,
                &mut b.anchor_sa_batch,
                &mut b.anchor_batch,
            );
        });
        u.n = 0;
        u.a.clear();

        let mut n_hit = 0;
        let mut w = std::mem::take(&mut b.chain_w);
        w.clear();
        if (KOM_DBG_FLAG.load(Ordering::Relaxed) & MB_DBG_ANCHOR) != 0 {
            mb_dbg_anchor(idx, qlen as i32, v.n, &v.a, qname);
        }
        let anchors = std::mem::take(&mut v.a);
        let mut a = crate::stage_time::measure(crate::stage_time::Bucket::Chain, || {
            mb_lchain_dp(
                (),
                &idx.l2b,
                opt.max_gap,
                opt.max_gap,
                opt.bw,
                opt.max_chain_skip,
                opt.max_chain_iter,
                opt.min_chain_score,
                chn_pen_gap,
                v.n,
                anchors,
                &mut n_hit,
                &mut w,
            )
        });

        if opt.bw_long > opt.bw * 2 && is_sr == 0 && n_hit > 0 {
            let mut best = 0usize;
            for i in 1..n_hit as usize {
                if (w[i] >> 32) > (w[best] >> 32) {
                    best = i;
                }
            }
            let mut as_ = 0usize;
            for x in w.iter().take(best) {
                as_ += *x as u32 as usize;
            }
            let cnt = w[best] as u32 as usize;
            let st = a[as_].qpos + 1 - a[as_].len;
            let en = a[as_ + cnt - 1].qpos + 1;
            if qlen as i32 - (en - st) > MIN_RECHAIN_LEN
                && (en - st) as f64 > qlen as f64 * MIN_RECHAIN_RATIO
            {
                let n_a = w
                    .iter()
                    .take(n_hit as usize)
                    .map(|x| *x as u32 as i64)
                    .sum::<i64>();
                mb_anchor_sort(&idx.l2b, n_a, &mut a);
                w.clear();
                a = crate::stage_time::measure(crate::stage_time::Bucket::Chain, || {
                    mb_lchain_dp(
                        (),
                        &idx.l2b,
                        opt.max_gap,
                        opt.max_gap,
                        opt.bw_long,
                        opt.max_chain_skip,
                        opt.max_chain_iter,
                        opt.min_chain_score,
                        chn_pen_gap,
                        n_a,
                        a,
                        &mut n_hit,
                        &mut w,
                    )
                });
            }
        }

        let mut hit = mb_gen_hit((), hash, qlen as i32, &idx.l2b, n_hit, &w, &a);
        mb_set_parent(
            (),
            opt.mask_level,
            opt.mask_len,
            n_hit,
            &mut hit,
            sub_diff,
            0,
        );
        mb_select_sub(
            (),
            opt.pri_ratio,
            opt.min_len * 2,
            opt.best_n,
            &mut n_hit,
            &mut hit,
        );

        if (opt.flag & MB_F_NO_ALN) == 0 {
            crate::stage_time::measure(crate::stage_time::Bucket::Align, || {
                mb_align_skeleton_with_scratch(
                    (),
                    opt,
                    idx,
                    qlen as i32,
                    seq,
                    mt,
                    &mut n_hit,
                    &mut hit,
                    &mut a,
                    &mut b.align_tseq,
                    &mut b.align_qseq0,
                );
            });
            mb_set_parent(
                (),
                opt.mask_level,
                opt.mask_len,
                n_hit,
                &mut hit,
                sub_diff,
                0,
            );
            mb_select_sub(
                (),
                opt.pri_ratio,
                opt.min_len * 2,
                opt.best_n,
                &mut n_hit,
                &mut hit,
            );
            mb_set_sam_pri(n_hit, &mut hit, ((opt.flag & MB_F_PRIMARY5) != 0) as i32);
        }
        crate::stage_time::measure(crate::stage_time::Bucket::MapqPost, || {
            for h in hit.iter_mut().take(n_hit as usize) {
                h.set_frac_high((255.0 * hi_cov as f64 / qlen as f64) as u8);
            }
            mb_set_mapq(
                (),
                qlen as i32,
                n_hit,
                &mut hit,
                opt.min_chain_score,
                opt.a,
                is_sr,
                opt.max_sr_len,
            );
        });
        *n_hit_ = n_hit;
        v.n = 0;
        v.m = a.capacity() as i64;
        v.a = a;
        b.anchor_v = v;
        b.chain_w = w;
        hit
    }

    /// Original C global function `mb_map` from `minibwa/map-algo.c:615`.
    pub fn mb_map(
        opt: &mb_opt_t,
        idx: &mb_idx_t,
        qlen: i32,
        seq0: &str,
        mt0: i32,
        n_hit_: &mut i32,
        b0: Option<&mut mb_tbuf_t>,
        qname: Option<&str>,
    ) -> Vec<mb_hit_t> {
        let mt = if mt0 == 0 {
            l2b_meth_t::L2B_METH_NONE
        } else if mt0 == 1 {
            l2b_meth_t::L2B_METH_C2T
        } else {
            l2b_meth_t::L2B_METH_G2A
        };
        let mut owned;
        let b = if let Some(b) = b0 {
            b
        } else {
            owned = mb_tbuf_init(1);
            &mut owned
        };
        let mut opt_adap = mb_opt_t::default();
        mb_opt_adap(opt, qlen, &mut opt_adap);
        let seq = seq0
            .bytes()
            .take(qlen as usize)
            .map(|c| match c {
                b'A' | b'a' => 0,
                b'C' | b'c' => 1,
                b'G' | b'g' => 2,
                b'T' | b't' => 3,
                _ => 4,
            })
            .collect::<Vec<_>>();
        let mut u = mb_sai_v::default();
        mb_seed_intv(
            (),
            &idx.bwt,
            qlen,
            &seq,
            opt.min_len,
            opt.max_sub_occ,
            &mut u,
        );
        mb_map_sai(
            &opt_adap,
            idx,
            qlen as i64,
            &seq,
            mt,
            &mut u,
            n_hit_,
            b,
            qname,
        )
    }

    /// Original C global function `mb_map_batch` from `minibwa/map-algo.c:656`.
    pub fn mb_map_batch(
        opt: &mb_opt_t,
        idx: &mb_idx_t,
        n_seq: i32,
        qlen: &[i32],
        seq: &[&str],
        n_hit: &mut [i32],
        b0: Option<&mut mb_tbuf_t>,
        qname: Option<&[&str]>,
    ) -> Vec<Vec<mb_hit_t>> {
        if n_seq <= 0 {
            return Vec::new();
        }
        let mut owned;
        let b = if let Some(b) = b0 {
            b
        } else {
            owned = mb_tbuf_init(0);
            &mut owned
        };
        let is_pe = ((opt.flag & MB_F_PE) != 0) as i32;
        let mut hit = vec![Vec::new(); n_seq as usize];
        let sb_max = opt.sb_seq.min(n_seq).max(0) as usize;
        let mut sai = vec![mb_sai_v::default(); sb_max.max(1)];
        let mut seq4 = vec![Vec::<u8>::new(); sb_max.max(1)];
        let mut i = 0i32;
        let mut sb_st = 0i32;
        let mut sb_len = 0i32;
        while i <= n_seq {
            if i == n_seq || sb_len >= opt.sb_len || i - sb_st >= opt.sb_seq {
                let sb_n = i - sb_st;
                if sb_n == 0 {
                    sb_st = i;
                    sb_len = 0;
                    i += 1;
                    continue;
                }
                for k in 0..sb_n as usize {
                    let idx_k = sb_st as usize + k;
                    seq4[k].clear();
                    seq4[k].extend(seq[idx_k].bytes().take(qlen[idx_k] as usize).map(
                        |c| match c {
                            b'A' | b'a' => 0,
                            b'C' | b'c' => 1,
                            b'G' | b'g' => 2,
                            b'T' | b't' => 3,
                            _ => 4,
                        },
                    ));
                    sai[k] = mb_sai_v::default();
                }
                let seq4_refs = seq4[..sb_n as usize]
                    .iter()
                    .map(|s| s.as_ptr())
                    .collect::<Vec<_>>();
                mb_seed_intv_batch(
                    (),
                    &idx.bwt,
                    sb_n,
                    &qlen[sb_st as usize..],
                    &seq4_refs,
                    opt.min_len,
                    opt.max_sub_occ,
                    &mut sai[..sb_n as usize],
                );
                for k in 0..sb_n as usize {
                    let idx_k = sb_st as usize + k;
                    let mut opt_adap = mb_opt_t::default();
                    mb_opt_adap(opt, qlen[idx_k], &mut opt_adap);
                    let mut mt = l2b_meth_t::L2B_METH_NONE;
                    if (opt.flag & MB_F_METH) != 0 {
                        mt = if is_pe != 0 {
                            if (idx_k & 1) == 0 {
                                l2b_meth_t::L2B_METH_C2T
                            } else {
                                l2b_meth_t::L2B_METH_G2A
                            }
                        } else {
                            l2b_meth_t::L2B_METH_C2T
                        };
                    }
                    let name = qname.and_then(|names| names.get(idx_k).copied());
                    hit[idx_k] = mb_map_sai(
                        &opt_adap,
                        idx,
                        qlen[idx_k] as i64,
                        &seq4[k],
                        mt,
                        &mut sai[k],
                        &mut n_hit[idx_k],
                        b,
                        name,
                    );
                }
                sb_st = i;
                sb_len = 0;
            }
            if i < n_seq {
                sb_len += qlen[i as usize];
            }
            i += 1;
        }
        hit
    }
}

pub mod map_main {
    #![allow(unused_variables, dead_code, non_snake_case, non_camel_case_types)]

    use crate::bseq::{
        mb_bseq1_t, mb_bseq_close, mb_bseq_file_t, mb_bseq_open, mb_bseq_read, mb_bseq_read_frag,
        mb_qname_same,
    };
    use crate::bwt::mb_sai_v;
    use crate::format::{mb_fmt_sam_hdr, mb_format};
    use crate::ketopt::{ketopt, ko_longopt_t, KETOPT_INIT};
    use crate::kommon::{kom_panic, kom_parse_num, kstring_t, KOM_NT4_TABLE};
    use crate::l2bit::l2b_meth_t;
    use crate::main::MB_VERSION;
    use crate::map_algo::{
        mb_idx_load, mb_idx_t, mb_map_sai, mb_tbuf_destroy, mb_tbuf_init, mb_tbuf_reset, mb_tbuf_t,
    };
    use crate::mbpriv::{
        KOM_DBG_FLAG, MB_DBG_ALN_PE, MB_DBG_ALN_SEQ, MB_DBG_ANCHOR, MB_DBG_AN_POS, MB_DBG_QNAME,
        MB_DBG_SEED,
    };
    use crate::options::{
        mb_opt_adap, mb_opt_init, mb_opt_preset, mb_opt_t, MB_F_ADAP, MB_F_COPY_COMMENT, MB_F_EQX,
        MB_F_LONG, MB_F_METH, MB_F_NO_ALN, MB_F_NO_KALLOC, MB_F_NO_PAIRING, MB_F_NO_UNMAP, MB_F_PE,
        MB_F_PE_PREDEF, MB_F_PRIMARY5, MB_F_SAM, MB_F_SUPP_SOFT, MB_F_WRITE_CS, MB_F_WRITE_DS,
        MB_F_WRITE_MD,
    };
    use crate::pe::{mb_hit_buf_t, mb_hit_t, mb_pair, mb_pestat, mb_pestat_t};
    use crate::seed::mb_seed_intv_batch;
    use rayon::prelude::*;
    use rayon::ThreadPool;
    use std::cell::RefCell;
    use std::ffi::CStr;
    use std::sync::atomic::Ordering;
    use std::sync::mpsc;
    use std::{fs, io::BufWriter, io::Write};

    thread_local! {
        static MAIN_MAP_OUTPUT_WRITER: RefCell<Option<*mut dyn Write>> = RefCell::new(None);
    }

    pub struct pipeline_t<'a, 'w, 'p> {
        pub n_fp: i32,
        pub n_threads: i32,
        pub n_base: i64,
        pub n_seq: i64,
        pub mb_size: i64,
        pub opt: &'a mb_opt_t,
        pub fp: Vec<mb_bseq_file_t>,
        pub idx: &'a mb_idx_t,
        pub worker_pool: Option<&'p ThreadPool>,
        pub output: String,
        pub output_writer: Option<&'w mut dyn Write>,
        pub output_error: bool,
    }

    #[derive(Debug)]
    pub struct step_t<'a> {
        pub opt: &'a mb_opt_t,
        pub idx: &'a mb_idx_t,
        pub n_seq: i32,
        pub n_frag: i32,
        pub n_sb: i32,
        pub n_pe: i32,
        pub n_hit: Vec<i32>,
        pub seg_off: Vec<i32>,
        pub seg_cnt: Vec<i32>,
        pub sb_off: Vec<i32>,
        pub sb_cnt: Vec<i32>,
        pub pes: [mb_pestat_t; 4],
        pub seq: Vec<mb_bseq1_t>,
        pub hit: Vec<mb_hit_buf_t>,
        pub tbuf: Vec<mb_tbuf_t>,
    }

    #[derive(Clone, Copy)]
    struct SeBatchView<'a> {
        opt: &'a mb_opt_t,
        idx: &'a mb_idx_t,
        seq: &'a [mb_bseq1_t],
        seg_off: &'a [i32],
        seg_cnt: &'a [i32],
        sb_off: &'a [i32],
        sb_cnt: &'a [i32],
    }

    #[derive(Clone, Copy)]
    struct SeBatchOutputs {
        n_hit: *mut i32,
        hit: *mut mb_hit_buf_t,
    }

    unsafe impl Send for SeBatchOutputs {}
    unsafe impl Sync for SeBatchOutputs {}

    impl SeBatchOutputs {
        unsafe fn write(self, idx: usize, n_hit: i32, hit: Vec<mb_hit_t>) {
            // SAFETY: callers partition work by non-overlapping sub-batches,
            // so each read slot is written by at most one worker.
            unsafe {
                *self.n_hit.add(idx) = n_hit;
                *self.hit.add(idx) = mb_hit_buf_t::from_vec(hit);
            }
        }
    }

    #[derive(Clone, Copy)]
    struct SeBatchScratch {
        tbuf: *mut mb_tbuf_t,
        len: usize,
    }

    unsafe impl Send for SeBatchScratch {}
    unsafe impl Sync for SeBatchScratch {}

    impl SeBatchScratch {
        unsafe fn get(self, tid: usize) -> &'static mut mb_tbuf_t {
            debug_assert!(tid < self.len);
            // SAFETY: Rayon runs at most one task on a worker thread at a time.
            // Each worker indexes its own scratch buffer by current_thread_index().
            unsafe { &mut *self.tbuf.add(tid.min(self.len - 1)) }
        }
    }

    #[derive(Clone, Copy)]
    struct PePairView<'a> {
        opt: &'a mb_opt_t,
        idx: &'a mb_idx_t,
        seq: &'a [mb_bseq1_t],
        seg_off: &'a [i32],
        seg_cnt: &'a [i32],
        pes: &'a [mb_pestat_t; 4],
    }

    #[derive(Clone, Copy)]
    struct PePairOutputs {
        n_hit: *mut i32,
        hit: *mut mb_hit_buf_t,
    }

    unsafe impl Send for PePairOutputs {}
    unsafe impl Sync for PePairOutputs {}

    impl PePairOutputs {
        unsafe fn pair_in_place(self, view: PePairView<'_>, frag: usize, tid: i32) {
            if view.seg_cnt[frag] != 2 {
                return;
            }
            let off = view.seg_off[frag] as usize;
            if (KOM_DBG_FLAG.load(Ordering::Relaxed) & MB_DBG_QNAME) != 0 {
                eprintln!("QP\t{}\t{}", view.seq[off].name, tid);
            }
            let len = [view.seq[off].l_seq as i32, view.seq[off + 1].l_seq as i32];
            let seq = [&*view.seq[off].seq, &*view.seq[off + 1].seq];
            // SAFETY: paired-end work is partitioned by fragment. Each
            // two-read hit slot is owned by exactly one fragment.
            unsafe {
                let mut n_pair = [*self.n_hit.add(off), *self.n_hit.add(off + 1)];
                let mut hit_pair = [
                    std::mem::take(&mut *self.hit.add(off)).into_vec(),
                    std::mem::take(&mut *self.hit.add(off + 1)).into_vec(),
                ];
                mb_pair(
                    (),
                    view.opt,
                    &view.idx.l2b,
                    &mut n_pair,
                    &mut hit_pair,
                    view.pes,
                    len,
                    seq,
                );
                *self.n_hit.add(off) = n_pair[0];
                *self.n_hit.add(off + 1) = n_pair[1];
                *self.hit.add(off) = mb_hit_buf_t::from_vec(std::mem::take(&mut hit_pair[0]));
                *self.hit.add(off + 1) = mb_hit_buf_t::from_vec(std::mem::take(&mut hit_pair[1]));
            }
        }
    }

    fn release_step_tbuf(s: &mut step_t<'_>) {
        for b in std::mem::take(&mut s.tbuf) {
            mb_tbuf_destroy(Some(b));
        }
    }

    /// Original C static function `worker_for_se_batch` from `minibwa/map-main.c:33`.
    fn worker_for_se_batch_collect_view(
        view: SeBatchView<'_>,
        i: i64,
        tid: i32,
        b: &mut mb_tbuf_t,
    ) -> Vec<(usize, i32, Vec<mb_hit_t>)> {
        let opt = view.opt;
        let idx = view.idx;
        let sb_i = i.max(0) as usize;
        let mut n = 0usize;
        let mut tot = 0usize;
        let mut out = Vec::with_capacity(n);
        for k in 0..view.sb_cnt[sb_i] as usize {
            let frag = view.sb_off[sb_i] as usize + k;
            let off = view.seg_off[frag] as usize;
            let cnt = view.seg_cnt[frag] as usize;
            n += cnt;
            for j in 0..cnt {
                tot += view.seq[off + j].l_seq as usize;
            }
        }
        let mut len = std::mem::take(&mut b.se_len);
        len.clear();
        len.resize(n, 0);
        let mut buf = std::mem::take(&mut b.se_buf);
        buf.clear();
        buf.reserve(tot.saturating_sub(buf.capacity()));
        let mut sai = std::mem::take(&mut b.se_sai);
        sai.clear();
        sai.resize(n, mb_sai_v::default());
        let mut p = 0usize;
        crate::stage_time::measure(crate::stage_time::Bucket::Encode, || {
            for k in 0..view.sb_cnt[sb_i] as usize {
                let frag = view.sb_off[sb_i] as usize + k;
                let off = view.seg_off[frag] as usize;
                let cnt = view.seg_cnt[frag] as usize;
                for j in 0..cnt {
                    let t = &view.seq[off + j];
                    len[p] = t.l_seq as i32;
                    let range_st = buf.len();
                    buf.extend(
                        t.seq
                            .bytes()
                            .take(t.l_seq as usize)
                            .map(|c| KOM_NT4_TABLE[c as usize]),
                    );
                    if idx.is_meth != 0 {
                        if (j & 1) == 0 {
                            for x in &mut buf[range_st..] {
                                if *x == 1 {
                                    *x = 3;
                                }
                            }
                        } else {
                            for x in &mut buf[range_st..] {
                                if *x == 2 {
                                    *x = 0;
                                }
                            }
                        }
                    }
                    p += 1;
                }
            }
        });
        assert_eq!(p, n);
        let mut seq = std::mem::take(&mut b.se_seq_ptrs);
        seq.clear();
        seq.reserve(n.saturating_sub(seq.capacity()));
        let mut range_st = 0usize;
        for &l in &len {
            seq.push(unsafe { buf.as_ptr().add(range_st) } as usize);
            range_st += l as usize;
        }
        crate::stage_time::measure(crate::stage_time::Bucket::Seed, || {
            mb_seed_intv_batch(
                (),
                &idx.bwt,
                n as i32,
                &len,
                unsafe { std::slice::from_raw_parts(seq.as_ptr() as *const *const u8, seq.len()) },
                opt.min_len,
                opt.max_sub_occ,
                &mut sai,
            );
        });
        p = 0;
        for k in 0..view.sb_cnt[sb_i] as usize {
            let frag = view.sb_off[sb_i] as usize + k;
            let off = view.seg_off[frag] as usize;
            let cnt = view.seg_cnt[frag] as usize;
            for j in 0..cnt {
                let t = &view.seq[off + j];
                let mt = if idx.is_meth == 0 {
                    l2b_meth_t::L2B_METH_NONE
                } else if (j & 1) == 0 {
                    l2b_meth_t::L2B_METH_C2T
                } else {
                    l2b_meth_t::L2B_METH_G2A
                };
                let mut opt_adap = mb_opt_t::default();
                mb_opt_adap(opt, len[p], &mut opt_adap);
                let mut n_hit = 0;
                let hit = mb_map_sai(
                    &opt_adap,
                    idx,
                    len[p] as i64,
                    unsafe { std::slice::from_raw_parts(seq[p] as *const u8, len[p] as usize) },
                    mt,
                    &mut sai[p],
                    &mut n_hit,
                    b,
                    Some(&t.name),
                );
                out.push((off + j, n_hit, hit));
                p += 1;
            }
        }
        let _ = tot;
        let _ = tid;
        mb_tbuf_reset(b, opt.cap_kalloc);
        b.se_len = len;
        b.se_buf = buf;
        b.se_seq_ptrs = seq;
        b.se_sai = sai;
        crate::stage_time::flush_local();
        out
    }

    fn worker_for_se_batch_collect(
        s: &mut step_t<'_>,
        i: i64,
        tid: i32,
    ) -> Vec<(usize, i32, Vec<mb_hit_t>)> {
        let tid_idx = tid.max(0) as usize;
        let mut b = mb_tbuf_init(((s.opt.flag & MB_F_NO_KALLOC) != 0) as i32);
        let b = if tid_idx < s.tbuf.len() {
            &mut s.tbuf[tid_idx]
        } else {
            &mut b
        };
        worker_for_se_batch_collect_view(
            SeBatchView {
                opt: s.opt,
                idx: s.idx,
                seq: &s.seq,
                seg_off: &s.seg_off,
                seg_cnt: &s.seg_cnt,
                sb_off: &s.sb_off,
                sb_cnt: &s.sb_cnt,
            },
            i,
            tid,
            b,
        )
    }

    /// Original C static function `worker_for_se_batch` from `minibwa/map-main.c:33`.
    pub fn worker_for_se_batch(s: &mut step_t<'_>, i: i64, tid: i32) {
        for (idx, n_hit, hit) in worker_for_se_batch_collect(s, i, tid) {
            s.n_hit[idx] = n_hit;
            s.hit[idx] = mb_hit_buf_t::from_vec(hit);
        }
    }

    fn worker_for_pe_collect(
        s: &step_t<'_>,
        i: i64,
        tid: i32,
    ) -> Option<(usize, [i32; 2], [Vec<mb_hit_t>; 2])> {
        let frag = i.max(0) as usize;
        if s.seg_cnt[frag] != 2 {
            return None;
        }
        let off = s.seg_off[frag] as usize;
        if (KOM_DBG_FLAG.load(Ordering::Relaxed) & MB_DBG_QNAME) != 0 {
            eprintln!("QP\t{}\t{}", s.seq[off].name, tid);
        }
        let len = [s.seq[off].l_seq as i32, s.seq[off + 1].l_seq as i32];
        let seq = [&*s.seq[off].seq, &*s.seq[off + 1].seq];
        let mut n_pair = [s.n_hit[off], s.n_hit[off + 1]];
        let mut hit_pair = [
            s.hit[off].as_slice().to_vec(),
            s.hit[off + 1].as_slice().to_vec(),
        ];
        mb_pair(
            (),
            s.opt,
            &s.idx.l2b,
            &mut n_pair,
            &mut hit_pair,
            &s.pes,
            len,
            seq,
        );
        Some((off, n_pair, hit_pair))
    }

    /// Original C static function `worker_for_pe` from `minibwa/map-main.c:82`.
    pub fn worker_for_pe(s: &mut step_t<'_>, i: i64, tid: i32) {
        let frag = i.max(0) as usize;
        if s.seg_cnt[frag] != 2 {
            return;
        }
        let off = s.seg_off[frag] as usize;
        if (KOM_DBG_FLAG.load(Ordering::Relaxed) & MB_DBG_QNAME) != 0 {
            eprintln!("QP\t{}\t{}", s.seq[off].name, tid);
        }
        let len = [s.seq[off].l_seq as i32, s.seq[off + 1].l_seq as i32];
        let seq = [&*s.seq[off].seq, &*s.seq[off + 1].seq];
        let mut n_pair = [s.n_hit[off], s.n_hit[off + 1]];
        let mut hit_pair = [
            std::mem::take(&mut s.hit[off]).into_vec(),
            std::mem::take(&mut s.hit[off + 1]).into_vec(),
        ];
        mb_pair(
            (),
            s.opt,
            &s.idx.l2b,
            &mut n_pair,
            &mut hit_pair,
            &s.pes,
            len,
            seq,
        );
        s.n_hit[off] = n_pair[0];
        s.n_hit[off + 1] = n_pair[1];
        s.hit[off] = mb_hit_buf_t::from_vec(std::mem::take(&mut hit_pair[0]));
        s.hit[off + 1] = mb_hit_buf_t::from_vec(std::mem::take(&mut hit_pair[1]));
        let _ = tid;
    }

    /// Original C static function `worker_pipeline` from `minibwa/map-main.c:98`.
    pub fn worker_pipeline<'a, 'w, 'p>(
        p: &mut pipeline_t<'a, 'w, 'p>,
        step: i32,
        input: Option<step_t<'a>>,
    ) -> Option<step_t<'a>> {
        const MIN_READ_CNT: i32 = 40000;
        let opt = p.opt;
        if step == 0 {
            let with_qual = ((opt.flag & MB_F_SAM) != 0) as i32;
            let with_comment = ((opt.flag & MB_F_COPY_COMMENT) != 0) as i32;
            let frag_mode = (p.n_fp > 1 || (opt.flag & MB_F_PE) != 0) as i32;
            let mut n_seq = 0;
            let mut seq = crate::stage_time::measure(crate::stage_time::Bucket::ReadIo, || {
                if p.n_fp > 1 {
                    mb_bseq_read_frag(
                        p.n_fp,
                        &mut p.fp,
                        p.mb_size,
                        with_qual,
                        with_comment,
                        &mut n_seq,
                    )
                } else {
                    mb_bseq_read(
                        &mut p.fp[0],
                        p.mb_size,
                        with_qual,
                        with_comment,
                        frag_mode,
                        MIN_READ_CNT,
                        opt.max_mb_size,
                        &mut n_seq,
                    )
                }
            });
            if seq.is_empty() {
                return None;
            }
            let mut s = step_t {
                opt,
                idx: p.idx,
                n_seq,
                n_frag: 0,
                n_sb: 0,
                n_pe: 0,
                n_hit: vec![0; n_seq as usize],
                seg_off: vec![0; n_seq as usize],
                seg_cnt: vec![0; n_seq as usize],
                sb_off: vec![0; n_seq as usize + 1],
                sb_cnt: vec![0; n_seq as usize + 1],
                pes: [mb_pestat_t {
                    failed: 1,
                    ..Default::default()
                }; 4],
                hit: vec![mb_hit_buf_t::default(); n_seq as usize],
                tbuf: Vec::with_capacity(opt.n_thread.max(1) as usize),
                seq: Vec::new(),
            };
            for t in &mut seq {
                t.id = p.n_seq as u64;
                p.n_seq += 1;
            }
            for _ in 0..opt.n_thread.max(1) {
                s.tbuf
                    .push(mb_tbuf_init(((opt.flag & MB_F_NO_KALLOC) != 0) as i32));
            }
            s.seq = seq;
            let mut j = 0usize;
            for i in 1..=s.n_seq as usize {
                if i == s.n_seq as usize
                    || frag_mode == 0
                    || mb_qname_same(&s.seq[i - 1].name, &s.seq[i].name) == 0
                {
                    assert!(i - j <= 2);
                    s.seg_cnt[s.n_frag as usize] = (i - j) as i32;
                    s.seg_off[s.n_frag as usize] = j as i32;
                    s.n_frag += 1;
                    if i - j == 2 {
                        s.n_pe += 1;
                    }
                    j = i;
                }
            }
            if s.n_pe > 0 {
                for i in 0..s.n_frag as usize {
                    if s.seg_cnt[i] != 2 {
                        continue;
                    }
                    let j0 = s.seg_off[i] as usize;
                    let j1 = j0 + 1;
                    let l0 = s.seq[j0].name.len();
                    let l1 = s.seq[j1].name.len();
                    if l0 >= 3
                        && l0 == l1
                        && s.seq[j0].name.as_bytes()[l0 - 1] != s.seq[j1].name.as_bytes()[l1 - 1]
                        && s.seq[j0].name.as_bytes()[l0 - 2] == b'/'
                    {
                        s.seq[j0].name = s.seq[j0].name[..l0 - 2].into();
                        s.seq[j1].name = s.seq[j1].name[..l1 - 2].into();
                    }
                }
            }
            let mut sb_len = 0i32;
            let mut sb_off = 0usize;
            for i in 0..s.n_frag as usize {
                if sb_len >= opt.sb_len || i - sb_off >= opt.sb_seq as usize {
                    s.sb_off[s.n_sb as usize] = sb_off as i32;
                    s.sb_cnt[s.n_sb as usize] = (i - sb_off) as i32;
                    s.n_sb += 1;
                    sb_len = 0;
                    sb_off = i;
                }
                for j in 0..s.seg_cnt[i] as usize {
                    sb_len += s.seq[s.seg_off[i] as usize + j].l_seq as i32;
                }
            }
            s.sb_off[s.n_sb as usize] = sb_off as i32;
            s.sb_cnt[s.n_sb as usize] = (s.n_frag as usize - sb_off) as i32;
            s.n_sb += 1;
            Some(s)
        } else if step == 1 {
            let mut s = input?;
            if let Some(pool) = p.worker_pool {
                let view = SeBatchView {
                    opt: s.opt,
                    idx: s.idx,
                    seq: &s.seq,
                    seg_off: &s.seg_off,
                    seg_cnt: &s.seg_cnt,
                    sb_off: &s.sb_off,
                    sb_cnt: &s.sb_cnt,
                };
                let outputs = SeBatchOutputs {
                    n_hit: s.n_hit.as_mut_ptr(),
                    hit: s.hit.as_mut_ptr(),
                };
                let scratch = SeBatchScratch {
                    tbuf: s.tbuf.as_mut_ptr(),
                    len: s.tbuf.len(),
                };
                pool.install(|| {
                    (0..s.n_sb as usize).into_par_iter().for_each(|i| {
                        let tid = rayon::current_thread_index()
                            .expect("map workers must run inside the configured Rayon pool");
                        let b = unsafe { scratch.get(tid) };
                        for (idx, n_hit, hit) in
                            worker_for_se_batch_collect_view(view, i as i64, tid as i32, b)
                        {
                            unsafe { outputs.write(idx, n_hit, hit) };
                        }
                    })
                });
            } else {
                for i in 0..s.n_sb {
                    worker_for_se_batch(&mut s, i as i64, 0);
                }
            }
            if (opt.flag & MB_F_PE) != 0 && s.n_frag < s.n_seq && (opt.flag & MB_F_NO_PAIRING) == 0
            {
                if (opt.flag & MB_F_PE_PREDEF) != 0 || s.n_pe < 20 {
                    s.pes[1].failed = 0;
                    s.pes[1].avg = opt.pe_avg as f64;
                    s.pes[1].std = opt.pe_std as f64;
                    s.pes[1].lo = opt.pe_lo;
                    s.pes[1].hi = opt.pe_hi;
                } else {
                    mb_pestat(
                        (),
                        opt,
                        s.n_frag,
                        &s.seg_off,
                        &s.seg_cnt,
                        &s.n_hit,
                        &s.hit,
                        &mut s.pes,
                    );
                }
                let _pair_t0 = if crate::stage_time::enabled() {
                    Some(std::time::Instant::now())
                } else {
                    None
                };
                if let Some(pool) = p.worker_pool {
                    let view = PePairView {
                        opt: s.opt,
                        idx: s.idx,
                        seq: &s.seq,
                        seg_off: &s.seg_off,
                        seg_cnt: &s.seg_cnt,
                        pes: &s.pes,
                    };
                    let outputs = PePairOutputs {
                        n_hit: s.n_hit.as_mut_ptr(),
                        hit: s.hit.as_mut_ptr(),
                    };
                    pool.install(|| {
                        (0..s.n_frag as usize).into_par_iter().for_each(|i| {
                            let tid = rayon::current_thread_index().unwrap_or(0) as i32;
                            unsafe { outputs.pair_in_place(view, i, tid) };
                        })
                    });
                } else {
                    for i in 0..s.n_frag {
                        worker_for_pe(&mut s, i as i64, 0);
                    }
                }
                if let Some(t0) = _pair_t0 {
                    crate::stage_time::accumulate_global(
                        crate::stage_time::Bucket::Pair,
                        t0.elapsed().as_nanos() as u64,
                    );
                }
            }
            Some(s)
        } else if step == 2 {
            let mut s = input?;
            let mut out = kstring_t::default();
            const OUTPUT_FLUSH_BYTES: usize = 1 << 20;
            let mut tot_len = 0i64;
            release_step_tbuf(&mut s);
            let _stage2 = if crate::stage_time::enabled() {
                Some(std::time::Instant::now())
            } else {
                None
            };
            for k in 0..s.n_frag as usize {
                let seg_st = s.seg_off[k] as usize;
                let seg_en = seg_st + s.seg_cnt[k] as usize;
                if p.output_writer.is_none() {
                    out.l = 0;
                }
                for i in seg_st..seg_en {
                    let mate_qlen = if seg_en - seg_st > 1 {
                        let mate_idx = if i != seg_en - 1 { i + 1 } else { seg_st };
                        s.seq[mate_idx].l_seq as i32
                    } else {
                        0
                    };
                    tot_len += s.seq[i].l_seq as i64;
                    if s.n_hit[i] > 0 {
                        let mut n_sec = 0;
                        for j in 0..s.n_hit[i] as usize {
                            let h = &s.hit[i][j];
                            if h.parent == h.id || n_sec < opt.out_n {
                                mb_format(
                                    (),
                                    &mut out,
                                    &p.idx.l2b,
                                    &s.seq[i],
                                    (seg_en - seg_st) as i32,
                                    &s.n_hit[seg_st..seg_en],
                                    &s.hit[seg_st..seg_en],
                                    j as i32,
                                    opt.flag,
                                    (i - seg_st) as i32,
                                    mate_qlen,
                                );
                            }
                            if h.parent != h.id {
                                n_sec += 1;
                            }
                        }
                    } else if (opt.flag & MB_F_NO_UNMAP) == 0 {
                        mb_format(
                            (),
                            &mut out,
                            &p.idx.l2b,
                            &s.seq[i],
                            (seg_en - seg_st) as i32,
                            &s.n_hit[seg_st..seg_en],
                            &s.hit[seg_st..seg_en],
                            -1,
                            opt.flag,
                            (i - seg_st) as i32,
                            mate_qlen,
                        );
                    }
                }
                if p.output_writer.is_some() {
                    if out.l >= OUTPUT_FLUSH_BYTES {
                        if let Some(writer) = p.output_writer.as_mut() {
                            if writer.write_all(&out.s[..out.l]).is_err() {
                                p.output_error = true;
                            }
                        }
                        out.l = 0;
                    }
                } else {
                    p.output
                        .push_str(std::str::from_utf8(&out.s[..out.l]).unwrap());
                }
            }
            if out.l > 0 && p.output_writer.is_some() {
                if let Some(writer) = p.output_writer.as_mut() {
                    if writer.write_all(&out.s[..out.l]).is_err() {
                        p.output_error = true;
                    }
                }
            }
            if let Some(t0) = _stage2 {
                crate::stage_time::accumulate_global(
                    crate::stage_time::Bucket::Output,
                    t0.elapsed().as_nanos() as u64,
                );
            }
            p.n_base += tot_len;
            None
        } else {
            None
        }
    }

    /// Original C static function `mb_open_bseqs` from `minibwa/map-main.c:230`.
    pub fn mb_open_bseqs(n: i32, fn_: &[&str]) -> Option<Vec<mb_bseq_file_t>> {
        let mut fp = Vec::with_capacity(n.max(0) as usize);
        for i in 0..n.max(0) as usize {
            let Some(f) = mb_bseq_open(Some(fn_[i])) else {
                let reason = std::fs::File::open(fn_[i])
                    .err()
                    .and_then(|e| e.raw_os_error())
                    .map(|code| unsafe {
                        CStr::from_ptr(libc::strerror(code))
                            .to_string_lossy()
                            .into_owned()
                    })
                    .unwrap_or_else(|| "Unknown error".to_string());
                eprintln!("ERROR: failed to open file '{}': {}", fn_[i], reason);
                return None;
            };
            fp.push(f);
        }
        Some(fp)
    }

    /// Original C global function `mb_map_file` from `minibwa/map-main.c:248`.
    pub fn mb_map_file(
        opt: &mb_opt_t,
        idx: &mb_idx_t,
        n: i32,
        fn_: &[&str],
        fn_out: Option<&str>,
    ) -> (i32, String) {
        let worker_pool = if opt.n_thread > 1 {
            Some(
                rayon::ThreadPoolBuilder::new()
                    .num_threads(opt.n_thread as usize)
                    .build()
                    .expect("failed to build Rayon thread pool"),
            )
        } else {
            None
        };
        mb_map_file_with_pool(opt, idx, n, fn_, fn_out, worker_pool.as_ref())
    }

    pub fn mb_map_file_with_pool(
        opt: &mb_opt_t,
        idx: &mb_idx_t,
        n: i32,
        fn_: &[&str],
        fn_out: Option<&str>,
        worker_pool: Option<&ThreadPool>,
    ) -> (i32, String) {
        if n < 1 {
            return (-1, String::new());
        }
        let mut output_file = if let Some(name) = fn_out {
            if name != "-" {
                match fs::File::create(name) {
                    Ok(fp) => Some(BufWriter::with_capacity(1 << 20, fp)),
                    Err(_) => return (-1, String::new()),
                }
            } else {
                None
            }
        } else {
            None
        };
        let Some(fp) = mb_open_bseqs(n, fn_) else {
            return (-1, String::new());
        };
        let output_writer: Option<&mut dyn Write> = if fn_out.is_none() {
            MAIN_MAP_OUTPUT_WRITER.with(|writer| {
                // SAFETY: main_map_write installs this pointer only for the
                // synchronous dynamic extent of its main_map call.
                writer.borrow().map(|ptr| unsafe { &mut *ptr })
            })
        } else {
            None
        };
        let run =
            |fp: Vec<mb_bseq_file_t>, output_writer: Option<&mut dyn Write>| -> (i32, String) {
                if worker_pool.is_some() && opt.mb_size <= 20_000_000 {
                    return std::thread::scope(|scope| {
                        let (read_tx, read_rx) = mpsc::sync_channel(0);
                        let (map_tx, map_rx) = mpsc::sync_channel(0);
                        scope.spawn(move || {
                            let mut read_pl = pipeline_t {
                                n_fp: n,
                                n_threads: if opt.n_thread <= 2 { opt.n_thread } else { 3 },
                                n_base: 0,
                                n_seq: 0,
                                mb_size: opt.mb_size,
                                opt,
                                fp,
                                idx,
                                worker_pool,
                                output: String::new(),
                                output_writer: None,
                                output_error: false,
                            };
                            while let Some(s0) = worker_pipeline(&mut read_pl, 0, None) {
                                if read_tx.send(s0).is_err() {
                                    break;
                                }
                            }
                            let fp = std::mem::take(&mut read_pl.fp);
                            for f in fp {
                                mb_bseq_close(Some(f));
                            }
                        });
                        scope.spawn(move || {
                            let mut map_pl = pipeline_t {
                                n_fp: n,
                                n_threads: if opt.n_thread <= 2 { opt.n_thread } else { 3 },
                                n_base: 0,
                                n_seq: 0,
                                mb_size: opt.mb_size,
                                opt,
                                fp: Vec::new(),
                                idx,
                                worker_pool,
                                output: String::new(),
                                output_writer: None,
                                output_error: false,
                            };
                            for s0 in read_rx {
                                if let Some(s1) = worker_pipeline(&mut map_pl, 1, Some(s0)) {
                                    let mut s1 = s1;
                                    release_step_tbuf(&mut s1);
                                    if map_tx.send(s1).is_err() {
                                        break;
                                    }
                                }
                            }
                        });
                        let mut out_pl = pipeline_t {
                            n_fp: n,
                            n_threads: if opt.n_thread <= 2 { opt.n_thread } else { 3 },
                            n_base: 0,
                            n_seq: 0,
                            mb_size: opt.mb_size,
                            opt,
                            fp: Vec::new(),
                            idx,
                            worker_pool,
                            output: String::new(),
                            output_writer,
                            output_error: false,
                        };
                        for s1 in map_rx {
                            worker_pipeline(&mut out_pl, 2, Some(s1));
                            if out_pl.output_error {
                                break;
                            }
                        }
                        if out_pl.output_error {
                            (-1, String::new())
                        } else if let Some(writer) = out_pl.output_writer.as_mut() {
                            if writer.flush().is_err() {
                                (-1, String::new())
                            } else {
                                (0, out_pl.output)
                            }
                        } else {
                            (0, out_pl.output)
                        }
                    });
                }
                let mut pl = pipeline_t {
                    n_fp: n,
                    n_threads: if opt.n_thread <= 2 { opt.n_thread } else { 3 },
                    n_base: 0,
                    n_seq: 0,
                    mb_size: opt.mb_size,
                    opt,
                    fp,
                    idx,
                    worker_pool,
                    output: String::new(),
                    output_writer,
                    output_error: false,
                };
                while let Some(s0) = worker_pipeline(&mut pl, 0, None) {
                    let Some(s1) = worker_pipeline(&mut pl, 1, Some(s0)) else {
                        break;
                    };
                    worker_pipeline(&mut pl, 2, Some(s1));
                    if pl.output_error {
                        break;
                    }
                }
                let fp = std::mem::take(&mut pl.fp);
                for f in fp {
                    mb_bseq_close(Some(f));
                }
                if pl.output_error {
                    (-1, String::new())
                } else if let Some(writer) = pl.output_writer.as_mut() {
                    if writer.flush().is_err() {
                        (-1, String::new())
                    } else {
                        (0, pl.output)
                    }
                } else {
                    (0, pl.output)
                }
            };
        if let Some(writer) = output_writer {
            return run(fp, Some(writer));
        }
        if let Some(fp_out) = output_file.as_mut() {
            let (ret, _) = run(fp, Some(fp_out));
            if ret != 0 {
                return (ret, String::new());
            }
            if fp_out.flush().is_err() {
                return (-1, String::new());
            }
            return (0, String::new());
        }
        run(fp, None)
    }

    /// Original C static function `usage` from `minibwa/map-main.c:296`.
    pub fn usage(to_stdout: bool, opt: &mb_opt_t) -> (i32, String) {
        let mut out = String::new();
        out.push_str("Usage: minibwa map [options] <in.idx> <in.fastq>\n");
        out.push_str("Options:\n");
        out.push_str("  Common:\n");
        out.push_str("    -a               output SAM (PAF by default)\n");
        out.push_str(&format!(
            "    -t INT           number of worker threads [{}]\n",
            opt.n_thread
        ));
        out.push_str(&format!(
            "    -l NUM           treat reads <NUM as short reads in the default adaptive mode [{}]\n",
            opt.max_sr_len
        ));
        out.push_str(
            "    -R STR           SAM read group line in a format like '@RG\\tID:foo\\tSM:bar' []\n",
        );
        out.push_str("    -b STR           output a base alignment tag: cs, ds or MD []\n");
        out.push_str("    --hic            map Hi-C reads; equivalent to option -5P\n");
        out.push_str("    --meth           map *directional* bisulfite sequencing reads\n");
        out.push_str("  Mapping:\n");
        out.push_str(&format!(
            "    -k INT           min seed length [{}]\n",
            opt.min_len
        ));
        out.push_str(&format!(
            "    -c NUM           max seed occurrences [{}]\n",
            opt.max_occ
        ));
        out.push_str(&format!(
            "    -g NUM           max gap size, controlling extension and chain breaking [{}]\n",
            opt.max_gap
        ));
        out.push_str(&format!("    -w NUM           bandwidth [{}]\n", opt.bw));
        out.push_str(&format!(
            "    -W NUM           long bandwidth (for long reads or the adaptive mode) [{}]\n",
            opt.bw_long
        ));
        out.push_str(&format!(
            "    -m INT           min chaining score [{}]\n",
            opt.min_chain_score
        ));
        out.push_str(&format!(
            "    -p FLOAT         min secondary-to-primary score ratio [{}]\n",
            opt.pri_ratio
        ));
        out.push_str(&format!(
            "    -N INT           retain at most INT secondary alignments [{}]\n",
            opt.best_n
        ));
        out.push_str("    --chain-only     perform chaining only without base alignment\n");
        out.push_str(
            "    -x STR           preset (sr, lr or adap for mixed short/long reads) [adap]\n",
        );
        out.push_str("  Alignment:\n");
        out.push_str(&format!(
            "    -A INT           matching score [{}]\n",
            opt.a
        ));
        out.push_str(&format!(
            "    -B INT           mismatching openalty [{}]\n",
            opt.b
        ));
        out.push_str(&format!(
            "    -O INT1[,INT2]   gap open penalty [{},{}]\n",
            opt.q, opt.q2
        ));
        out.push_str(&format!(
            "    -E INT1[,INT2]   gap extension penalty [{},{}]\n",
            opt.e, opt.e2
        ));
        out.push_str(&format!(
            "    -s INT           suppress alignment with DP score lower than INT*{{-A}} [{}]\n",
            opt.min_dp_max
        ));
        out.push_str("  Paired-end:\n");
        out.push_str("    -P               skip pairing and mate resuce\n");
        out.push_str(&format!(
            "    --rescue=INT     mate rescue for up to INT candidates; 0 to skip rescue [{}]\n",
            opt.max_rescue
        ));
        out.push_str("  Input/Output:\n");
        out.push_str("    -o FILE          output file name [stdout]\n");
        out.push_str("    -u               don't output unmapped reads\n");
        out.push_str("    --outn=INT       output up to INT secondary alignments [0]\n");
        out.push_str("    -y               copy FASTA/Q comments to output\n");
        out.push_str("    -Y               use soft clipping for supplementary alignments\n");
        out.push_str(
            "    -5               take the alignment with the smallest query position as primary\n",
        );
        out.push_str(
            "    -K NUM1[,NUM2]   process NUM1-NUM2 bp of query sequences in a batch [100m,1g]\n",
        );
        out.push_str("    --version        print version number\n");
        out.push_str("    --help           print this help message\n");
        (if to_stdout { 0 } else { 1 }, out)
    }

    /// Original C static function `yes_or_no` from `minibwa/map-main.c:338`.
    pub fn yes_or_no(
        opt: &mut mb_opt_t,
        flag: u64,
        option_name: &str,
        arg: &str,
        yes_to_set: i32,
    ) -> Option<String> {
        if yes_to_set != 0 {
            if arg == "yes" || arg == "y" {
                opt.flag |= flag;
                None
            } else if arg == "no" || arg == "n" {
                opt.flag &= !flag;
                None
            } else {
                Some(format!(
                    "[WARNING]\u{1b}[1;31m option '--{}' only accepts 'yes' or 'no'.\u{1b}[0m\n",
                    option_name
                ))
            }
        } else if arg == "yes" || arg == "y" {
            opt.flag &= !flag;
            None
        } else if arg == "no" || arg == "n" {
            opt.flag |= flag;
            None
        } else {
            Some(format!(
                "[WARNING]\u{1b}[1;31m option '--{}' only accepts 'yes' or 'no'.\u{1b}[0m\n",
                option_name
            ))
        }
    }

    /// Original C global function `main_map` from `minibwa/map-main.c:351`.
    pub fn main_map(argv: &[String]) -> (i32, String) {
        let has_output_writer = MAIN_MAP_OUTPUT_WRITER.with(|writer| writer.borrow().is_some());
        let opt_str = "x:o:k:c:m:p:A:B:b:O:E:t:K:N:PyYR:aul:w:W:g:5s:";
        let long_options = [
            ko_longopt_t {
                name: Some("kalloc".into()),
                has_arg: 1,
                val: 301,
            },
            ko_longopt_t {
                name: Some("outn".into()),
                has_arg: 1,
                val: 302,
            },
            ko_longopt_t {
                name: Some("pe-predef".into()),
                has_arg: 2,
                val: 303,
            },
            ko_longopt_t {
                name: Some("rescue".into()),
                has_arg: 1,
                val: 304,
            },
            ko_longopt_t {
                name: Some("eqx".into()),
                has_arg: 0,
                val: 305,
            },
            ko_longopt_t {
                name: Some("pe".into()),
                has_arg: 1,
                val: 306,
            },
            ko_longopt_t {
                name: Some("long".into()),
                has_arg: 2,
                val: 307,
            },
            ko_longopt_t {
                name: Some("adap".into()),
                has_arg: 1,
                val: 308,
            },
            ko_longopt_t {
                name: Some("chain-only".into()),
                has_arg: 0,
                val: 309,
            },
            ko_longopt_t {
                name: Some("meth".into()),
                has_arg: 0,
                val: 310,
            },
            ko_longopt_t {
                name: Some("hic".into()),
                has_arg: 0,
                val: 311,
            },
            ko_longopt_t {
                name: Some("dbg-aln-seq".into()),
                has_arg: 0,
                val: 601,
            },
            ko_longopt_t {
                name: Some("dbg-anchor".into()),
                has_arg: 0,
                val: 602,
            },
            ko_longopt_t {
                name: Some("dbg-seed".into()),
                has_arg: 0,
                val: 603,
            },
            ko_longopt_t {
                name: Some("dbg-qname".into()),
                has_arg: 0,
                val: 604,
            },
            ko_longopt_t {
                name: Some("dbg-aln-pe".into()),
                has_arg: 0,
                val: 605,
            },
            ko_longopt_t {
                name: Some("dbg-an-pos".into()),
                has_arg: 0,
                val: 606,
            },
            ko_longopt_t {
                name: Some("version".into()),
                has_arg: 0,
                val: 901,
            },
            ko_longopt_t {
                name: Some("help".into()),
                has_arg: 0,
                val: 902,
            },
        ];
        let argc = argv.len() as i32;
        let mut args = argv.to_vec();
        let mut mo = mb_opt_t::default();
        mb_opt_init(&mut mo);
        let mut o = KETOPT_INIT.clone();
        loop {
            let c = ketopt(&mut o, argc, &mut args, 1, opt_str, Some(&long_options));
            if c < 0 {
                break;
            }
            if c == 'x' as i32 {
                if mb_opt_preset(&mut mo, o.arg.as_deref().unwrap_or("")) < 0 {
                    return (
                        1,
                        format!(
                            "[ERROR] unknown preset '{}'\n",
                            o.arg.as_deref().unwrap_or("")
                        ),
                    );
                }
            } else if c == ':' as i32 {
                return (1, "[ERROR] missing option argument\n".to_string());
            } else if c == '?' as i32 {
                let i = (o.i - 1).max(0) as usize;
                return (
                    1,
                    format!(
                        "[ERROR] unknown option in \"{}\"\n",
                        args.get(i).map(String::as_str).unwrap_or("")
                    ),
                );
            }
        }
        let mut fn_out: Option<String> = None;
        let mut rg_line: Option<String> = None;
        o = KETOPT_INIT.clone();
        loop {
            let c = ketopt(&mut o, argc, &mut args, 1, opt_str, Some(&long_options));
            if c < 0 {
                break;
            }
            let arg = o.arg.as_deref().unwrap_or("");
            if c == 'k' as i32 {
                mo.min_len = arg.parse().unwrap_or(mo.min_len);
            } else if c == 'c' as i32 {
                mo.max_occ = kom_parse_num(arg).0 as i32;
            } else if c == 'p' as i32 {
                mo.pri_ratio = arg.parse().unwrap_or(mo.pri_ratio);
            } else if c == 'm' as i32 {
                mo.min_chain_score = arg.parse().unwrap_or(mo.min_chain_score);
            } else if c == 'N' as i32 {
                mo.best_n = arg.parse().unwrap_or(mo.best_n);
            } else if c == 'A' as i32 {
                mo.a = arg.parse().unwrap_or(mo.a);
            } else if c == 'B' as i32 {
                mo.b = arg.parse().unwrap_or(mo.b);
            } else if c == 'l' as i32 {
                mo.max_sr_len = kom_parse_num(arg).0 as i32;
            } else if c == 'g' as i32 {
                mo.max_gap = kom_parse_num(arg).0 as i32;
            } else if c == 'w' as i32 {
                mo.bw = kom_parse_num(arg).0 as i32;
            } else if c == 'W' as i32 {
                mo.bw_long = kom_parse_num(arg).0 as i32;
            } else if c == 'a' as i32 {
                mo.flag |= MB_F_SAM;
            } else if c == 'u' as i32 {
                mo.flag |= MB_F_NO_UNMAP;
            } else if c == 'y' as i32 {
                mo.flag |= MB_F_COPY_COMMENT;
            } else if c == 'Y' as i32 {
                mo.flag |= MB_F_SUPP_SOFT;
            } else if c == '5' as i32 {
                mo.flag |= MB_F_PRIMARY5;
            } else if c == 'P' as i32 {
                mo.flag |= MB_F_NO_PAIRING;
            } else if c == 's' as i32 {
                mo.min_dp_max = arg.parse().unwrap_or(mo.min_dp_max);
            } else if c == 'o' as i32 {
                fn_out = Some(arg.to_string());
            } else if c == 't' as i32 {
                mo.n_thread = arg.parse().unwrap_or(mo.n_thread);
            } else if c == 'R' as i32 {
                rg_line = Some(arg.to_string());
            } else if c == 'K' as i32 {
                let (x, used) = kom_parse_num(arg);
                mo.mb_size = x;
                mo.max_mb_size = x;
                if arg.as_bytes().get(used) == Some(&b',') {
                    mo.max_mb_size = kom_parse_num(&arg[used + 1..]).0;
                }
                if mo.max_mb_size < mo.mb_size {
                    mo.max_mb_size = mo.mb_size;
                }
            } else if c == 'O' as i32 {
                let mut it = arg.split(',');
                mo.q = it.next().unwrap_or("").parse().unwrap_or(mo.q);
                mo.q2 = it.next().unwrap_or("").parse().unwrap_or(mo.q);
            } else if c == 'E' as i32 {
                let mut it = arg.split(',');
                mo.e = it.next().unwrap_or("").parse().unwrap_or(mo.e);
                mo.e2 = it.next().unwrap_or("").parse().unwrap_or(mo.e);
            } else if c == 'b' as i32 {
                mo.flag &= !(MB_F_WRITE_CS | MB_F_WRITE_DS | MB_F_WRITE_MD);
                if arg == "cs" {
                    mo.flag |= MB_F_WRITE_CS;
                } else if arg == "ds" {
                    mo.flag |= MB_F_WRITE_DS;
                } else if arg == "MD" || arg == "md" {
                    mo.flag |= MB_F_WRITE_MD;
                } else {
                    mo.flag |= MB_F_WRITE_CS;
                    eprintln!(
                        "[WARNING]\u{1b}[1;31m -b only takes 'cs', 'ds' or 'MD'. Invalid values are assumed to be 'cs'.\u{1b}[0m"
                    );
                }
            } else if c == 301 {
                if let Some(warning) = yes_or_no(&mut mo, MB_F_NO_KALLOC, "kalloc", arg, 0) {
                    eprint!("{warning}");
                }
            } else if c == 302 {
                mo.out_n = arg.parse().unwrap_or(mo.out_n);
            } else if c == 303 {
                mo.flag |= MB_F_PE_PREDEF;
            } else if c == 304 {
                mo.max_rescue = arg.parse().unwrap_or(mo.max_rescue);
            } else if c == 305 {
                mo.flag |= MB_F_EQX;
            } else if c == 306 {
                if let Some(warning) = yes_or_no(&mut mo, MB_F_PE, "pe", arg, 1) {
                    eprint!("{warning}");
                }
            } else if c == 307 {
                if o.arg.is_none() {
                    mo.flag |= MB_F_LONG;
                } else if let Some(warning) = yes_or_no(&mut mo, MB_F_LONG, "long", arg, 1) {
                    eprint!("{warning}");
                }
            } else if c == 308 {
                if let Some(warning) = yes_or_no(&mut mo, MB_F_ADAP, "adap", arg, 1) {
                    eprint!("{warning}");
                }
            } else if c == 309 {
                mo.flag |= MB_F_NO_ALN;
            } else if c == 310 {
                mo.flag |= MB_F_METH;
            } else if c == 311 {
                mo.flag |= MB_F_PRIMARY5 | MB_F_NO_PAIRING;
            } else if c == 601 {
                KOM_DBG_FLAG.fetch_or(MB_DBG_ALN_SEQ, Ordering::Relaxed);
            } else if c == 602 {
                KOM_DBG_FLAG.fetch_or(MB_DBG_ANCHOR, Ordering::Relaxed);
            } else if c == 603 {
                KOM_DBG_FLAG.fetch_or(MB_DBG_SEED, Ordering::Relaxed);
            } else if c == 604 {
                KOM_DBG_FLAG.fetch_or(MB_DBG_QNAME, Ordering::Relaxed);
            } else if c == 605 {
                KOM_DBG_FLAG.fetch_or(MB_DBG_ALN_PE, Ordering::Relaxed);
            } else if c == 606 {
                KOM_DBG_FLAG.fetch_or(MB_DBG_AN_POS, Ordering::Relaxed);
            } else if c == 901 {
                return (0, MB_VERSION.to_string() + "\n");
            } else if c == 902 {
                let (ret, text) = usage(true, &mo);
                return (ret, text);
            } else if c == ':' as i32 || c == '?' as i32 {
                return (1, String::new());
            }
        }
        if argc - o.ind < 2 {
            let (ret, text) = usage(false, &mo);
            return (ret, text);
        }
        let Some(idx) = mb_idx_load(&args[o.ind as usize], ((mo.flag & MB_F_METH) != 0) as i32)
        else {
            kom_panic("main_map", "failed to load the index.");
        };
        let inputs = args[o.ind as usize + 1..]
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        let mut out = String::new();
        if (mo.flag & MB_F_SAM) != 0 {
            let mut hdr = kstring_t::default();
            let argv_refs = args.iter().map(String::as_str).collect::<Vec<_>>();
            if mb_fmt_sam_hdr(
                &mut hdr,
                Some(&idx.l2b),
                rg_line.as_deref(),
                Some(MB_VERSION),
                &argv_refs,
            ) < 0
            {
                return (1, String::new());
            }
            if has_output_writer {
                let write_ok = MAIN_MAP_OUTPUT_WRITER.with(|writer| {
                    // SAFETY: main_map_write installs this pointer only for the
                    // synchronous dynamic extent of this main_map call.
                    writer
                        .borrow()
                        .map(|ptr| unsafe { (&mut *ptr).write_all(&hdr.s[..hdr.l]).is_ok() })
                        .unwrap_or(false)
                });
                if !write_ok {
                    return (-1, String::new());
                }
            } else {
                out.push_str(std::str::from_utf8(&hdr.s[..hdr.l]).unwrap());
            }
        }
        let (ret, body) = mb_map_file(&mo, &idx, inputs.len() as i32, &inputs, fn_out.as_deref());
        if fn_out.is_none() && !has_output_writer {
            out.push_str(&body);
        }
        (ret, out)
    }

    pub fn main_map_write(argv: &[String], output_writer: &mut dyn Write) -> (i32, String) {
        MAIN_MAP_OUTPUT_WRITER.with(|writer| {
            // SAFETY: the pointer is restored before main_map_write returns,
            // and main_map only borrows it synchronously through the
            // thread-local slot.
            let output_writer = unsafe {
                std::mem::transmute::<&mut dyn Write, *mut (dyn Write + 'static)>(output_writer)
            };
            let old = writer.replace(Some(output_writer));
            let ret = main_map(argv);
            writer.replace(old);
            ret
        })
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::map_algo::mb_idx_load;
        use crate::options::{mb_opt_init, MB_F_NO_ALN};

        #[test]
        fn usage_formats_map_defaults() {
            let mut opt = mb_opt_t::default();
            mb_opt_init(&mut opt);
            let (ret, out) = usage(true, &opt);
            assert_eq!(ret, 0);
            assert!(out.contains("Usage: minibwa map [options] <in.idx> <in.fastq>\n"));
            assert!(out.contains(&format!(
                "    -k INT           min seed length [{}]\n",
                opt.min_len
            )));
            assert!(out.contains("    --help           print this help message\n"));
        }

        #[test]
        fn yes_or_no_sets_clears_and_reports_bad_values() {
            let mut opt = mb_opt_t::default();
            assert_eq!(
                yes_or_no(&mut opt, MB_F_NO_ALN, "chain-only", "yes", 1),
                None
            );
            assert_ne!(opt.flag & MB_F_NO_ALN, 0);
            assert_eq!(
                yes_or_no(&mut opt, MB_F_NO_ALN, "chain-only", "no", 1),
                None
            );
            assert_eq!(opt.flag & MB_F_NO_ALN, 0);
            assert_eq!(
                yes_or_no(&mut opt, MB_F_NO_ALN, "chain-only", "no", 0),
                None
            );
            assert_ne!(opt.flag & MB_F_NO_ALN, 0);
            assert!(yes_or_no(&mut opt, MB_F_NO_ALN, "chain-only", "maybe", 1)
                .unwrap()
                .contains("only accepts 'yes' or 'no'"));
        }

        #[test]
        fn map_file_maps_real_chrm_fastq_to_paf_text() {
            let idx = mb_idx_load("minibwa/chrM-human", 0).expect("load index");
            let mut opt = mb_opt_t::default();
            mb_opt_init(&mut opt);
            opt.flag |= MB_F_NO_ALN;
            opt.mb_size = 1000;
            opt.max_mb_size = 1000;
            let mut fq = std::env::temp_dir();
            fq.push(format!(
                "minibwa_rs_map_file_{}_{}.fq",
                std::process::id(),
                crate::kommon::kom_realtime().to_bits()
            ));
            std::fs::write(
                &fq,
                b"@r0\nGATCACAGGTCTATCACCCTATTAACCACTCACGGGAGCTCTCCATGCAT\n+\nFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF\n",
            )
            .unwrap();
            let path = fq.to_string_lossy().into_owned();
            let (ret, out) = mb_map_file(&opt, &idx, 1, &[path.as_str()], None);
            assert_eq!(ret, 0);
            assert!(out.contains("r0\t"));
            assert!(out.contains("\tchrM\t"));
            let _ = std::fs::remove_file(fq);
        }

        #[test]
        fn map_file_processes_multiple_read_batches() {
            let idx = mb_idx_load("minibwa/chrM-human", 0).expect("load index");
            let mut opt = mb_opt_t::default();
            mb_opt_init(&mut opt);
            opt.flag |= MB_F_NO_ALN;
            opt.mb_size = 20;
            opt.max_mb_size = 20;
            let mut fq = std::env::temp_dir();
            fq.push(format!(
                "minibwa_rs_map_file_batches_{}_{}.fq",
                std::process::id(),
                crate::kommon::kom_realtime().to_bits()
            ));
            std::fs::write(
                &fq,
                b"@r0\nGATCACAGGTCTATCACCCTATTAACCACTCACGGGAGCTCTCCATGCAT\n+\nFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF\n@r1\nATCACAGGTCTATCACCCTATTAACCACTCACGGGAGCTCTCCATGCATG\n+\nFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF\n",
            )
            .unwrap();
            let path = fq.to_string_lossy().into_owned();
            let (ret, out) = mb_map_file(&opt, &idx, 1, &[path.as_str()], None);
            assert_eq!(ret, 0);
            assert!(out.contains("r0\t"));
            assert!(out.contains("r1\t"));
            let _ = std::fs::remove_file(fq);
        }

        #[test]
        fn main_map_parses_options_and_maps_real_chrm_fastq() {
            let mut fq = std::env::temp_dir();
            fq.push(format!(
                "minibwa_rs_main_map_{}_{}.fq",
                std::process::id(),
                crate::kommon::kom_realtime().to_bits()
            ));
            std::fs::write(
                &fq,
                b"@r0\nGATCACAGGTCTATCACCCTATTAACCACTCACGGGAGCTCTCCATGCAT\n+\nFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF\n",
            )
            .unwrap();
            let args = vec![
                "map".to_string(),
                "--chain-only".to_string(),
                "-K".to_string(),
                "1k,1k".to_string(),
                "minibwa/chrM-human".to_string(),
                fq.to_string_lossy().into_owned(),
            ];
            let (ret, out) = main_map(&args);
            assert_eq!(ret, 0);
            assert!(out.contains("r0\t"));
            assert!(out.contains("\tchrM\t"));
            let _ = std::fs::remove_file(fq);
        }
    }
}

pub mod mbpriv {
    #![allow(unused_variables, dead_code, non_snake_case)]

    use crate::options::{mb_opt_t, MB_F_ADAP, MB_F_LONG};
    use std::sync::atomic::AtomicI32;

    pub const MB_DBG_ALN_SEQ: i32 = 0x1;
    pub const MB_DBG_ANCHOR: i32 = 0x2;
    pub const MB_DBG_SEED: i32 = 0x4;
    pub const MB_DBG_QNAME: i32 = 0x8;
    pub const MB_DBG_ALN_PE: i32 = 0x10;
    pub const MB_DBG_AN_POS: i32 = 0x20;

    /// Original C global variable `kom_dbg_flag` from `minibwa/kommon.c:7`.
    pub static KOM_DBG_FLAG: AtomicI32 = AtomicI32::new(0);

    /// Original C static function `mb_log2` from `minibwa/mbpriv.h:100`.
    pub fn mb_log2(x: f32) -> f32 {
        let mut i = x.to_bits();
        let mut log_2 = ((i >> 23) & 255) as f32 - 128.0;
        i &= !(255 << 23);
        i += 127 << 23;
        let f = f32::from_bits(i);
        log_2 += (-0.34484843f32 * f + 2.02466578f32) * f - 0.67487759f32;
        log_2
    }

    /// Original C static function `mb_hash64` from `minibwa/mbpriv.h:110`.
    pub fn mb_hash64(mut x: u64) -> u64 {
        x ^= x >> 30;
        x = x.wrapping_mul(0xbf58476d1ce4e5b9);
        x ^= x >> 27;
        x = x.wrapping_mul(0x94d049bb133111eb);
        x ^= x >> 31;
        x
    }

    /// Original C static function `mb_hash_str` from `minibwa/mbpriv.h:120`.
    pub fn mb_hash_str(s: &str) -> u32 {
        let mut h = 2166136261u32;
        for &c in s.as_bytes() {
            h ^= c as u32;
            h = h.wrapping_mul(16777619);
        }
        h
    }

    /// Original C static function `mb_seq_rev` from `minibwa/mbpriv.h:129`.
    pub fn mb_seq_rev(len: u32, seq: &mut [u8]) {
        seq[..len as usize].reverse();
    }

    /// Original C static function `mb_is_sr_mode` from `minibwa/mbpriv.h:134`.
    pub fn mb_is_sr_mode(opt: &mb_opt_t, qlen: i32) -> i32 {
        if (opt.flag & MB_F_LONG) != 0 || ((opt.flag & MB_F_ADAP) != 0 && qlen > opt.max_sr_len) {
            0
        } else {
            1
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::options::{mb_opt_init, mb_opt_preset};

        #[test]
        fn hash_helpers_match_known_vectors() {
            assert_eq!(mb_hash64(0), 0);
            assert_eq!(mb_hash64(1), 0x5692161d100b05e5);
            assert_eq!(mb_hash_str("chrM"), 0xc96c8abb);
        }

        #[test]
        fn seq_rev_reverses_requested_prefix() {
            let mut s = *b"ACGTNN";
            mb_seq_rev(4, &mut s);
            assert_eq!(&s, b"TGCANN");
        }

        #[test]
        fn sr_mode_follows_flags_and_read_length() {
            let mut opt = mb_opt_t::default();
            mb_opt_init(&mut opt);
            assert_eq!(mb_is_sr_mode(&opt, 100), 1);
            assert_eq!(mb_is_sr_mode(&opt, 1000), 0);
            mb_opt_preset(&mut opt, "lr");
            assert_eq!(mb_is_sr_mode(&opt, 100), 0);
        }
    }
}

pub mod options {
    #![allow(unused_variables, dead_code, non_snake_case, non_camel_case_types)]

    pub const MB_F_SAM: u64 = 0x1;
    pub const MB_F_NO_UNMAP: u64 = 0x2;
    pub const MB_F_COPY_COMMENT: u64 = 0x4;
    pub const MB_F_PE: u64 = 0x8;
    pub const MB_F_LONG: u64 = 0x10;
    pub const MB_F_EQX: u64 = 0x20;
    pub const MB_F_NO_KALLOC: u64 = 0x40;
    pub const MB_F_NO_ALN: u64 = 0x80;
    pub const MB_F_PE_PREDEF: u64 = 0x100;
    pub const MB_F_WRITE_DS: u64 = 0x200;
    pub const MB_F_WRITE_CS: u64 = 0x400;
    pub const MB_F_WRITE_MD: u64 = 0x800;
    pub const MB_F_2ND_SEQ: u64 = 0x1000;
    pub const MB_F_SUPP_SOFT: u64 = 0x2000;
    pub const MB_F_ADAP: u64 = 0x4000;
    pub const MB_F_PRIMARY5: u64 = 0x8000;
    pub const MB_F_NO_PAIRING: u64 = 0x10000;
    pub const MB_F_METH: u64 = 0x20000;

    #[derive(Clone, Debug, Default, PartialEq)]
    pub struct mb_opt_t {
        pub flag: u64,
        pub min_len: i32,
        pub max_sub_occ: i32,
        pub max_occ: i32,
        pub bw: i32,
        pub bw_long: i32,
        pub max_gap: i32,
        pub max_sr_len: i32,
        pub max_chain_skip: i32,
        pub max_chain_iter: i32,
        pub min_chain_score: i32,
        pub chain_gap_scale: f32,
        pub mask_level: f32,
        pub mask_len: i32,
        pub pri_ratio: f32,
        pub best_n: i32,
        pub a: i32,
        pub b: i32,
        pub b_ts: i32,
        pub b_ambi: i32,
        pub q: i32,
        pub q2: i32,
        pub e: i32,
        pub e2: i32,
        pub end_bonus: i32,
        pub min_dp_max: i32,
        pub zdrop: i32,
        pub zdrop_inv: i32,
        pub min_ksw_len: i32,
        pub max_pe_ins: i32,
        pub max_rescue: i32,
        pub pen_unpair: i32,
        pub pe_avg: i32,
        pub pe_std: i32,
        pub pe_lo: i32,
        pub pe_hi: i32,
        pub sb_len: i32,
        pub sb_seq: i32,
        pub n_thread: i32,
        pub out_n: i32,
        pub seed: i32,
        pub mb_size: i64,
        pub max_mb_size: i64,
        pub max_sw_mat: i64,
        pub cap_kalloc: i64,
    }

    /// Original C static function `mb_opt_reset` from `minibwa/options.c:5`.
    pub fn mb_opt_reset(opt: &mut mb_opt_t) {
        *opt = mb_opt_t::default();
        opt.min_len = 19;
        opt.max_sr_len = 325;
        opt.max_sub_occ = 10;
        opt.max_occ = 250;
        opt.max_chain_skip = 25;
        opt.max_chain_iter = 5000;
        opt.chain_gap_scale = 0.8;
        opt.bw_long = 20000;
        opt.pri_ratio = 0.5;
        opt.mask_level = 0.5;
        opt.mask_len = 0x7fffffff;
        opt.a = 2;
        opt.b = 8;
        opt.q = 12;
        opt.q2 = 23;
        opt.e = 2;
        opt.e2 = 1;
        opt.b_ambi = 1;
        opt.max_pe_ins = 10000;
        opt.max_rescue = 10;
        opt.pen_unpair = 17;
        opt.pe_avg = 400;
        opt.pe_std = 100;
        opt.pe_lo = 50;
        opt.pe_hi = 800;
        opt.sb_len = 1000000;
        opt.sb_seq = 24;
        opt.n_thread = 1;
        opt.seed = 11;
        opt.max_sw_mat = 100000000;
        opt.cap_kalloc = 1i64 << 28;
        opt.max_mb_size = 1000000000;
        opt.flag |= MB_F_NO_KALLOC;
    }

    /// Original C global function `mb_opt_init` from `minibwa/options.c:46`.
    pub fn mb_opt_init(opt: &mut mb_opt_t) {
        mb_opt_reset(opt);
        mb_opt_preset(opt, "adap");
    }

    /// Original C global function `mb_opt_preset` from `minibwa/options.c:52`.
    pub fn mb_opt_preset(opt: &mut mb_opt_t, preset: &str) -> i32 {
        mb_opt_reset(opt);
        if preset == "sr" || preset == "adap" {
            opt.flag |= MB_F_PE;
            if preset == "adap" {
                opt.flag |= MB_F_ADAP;
            }
            opt.min_dp_max = 30;
            opt.flag |= MB_F_ADAP;
            opt.bw = 100;
            opt.max_gap = 100;
            opt.zdrop = 80;
            opt.zdrop_inv = 80;
            opt.best_n = 50;
            opt.end_bonus = 10;
            opt.min_chain_score = 25;
            opt.min_ksw_len = 20;
            opt.mb_size = 100000000;
        } else if preset == "lr" {
            opt.flag |= MB_F_LONG;
            opt.flag &= !MB_F_PE;
            opt.min_dp_max = 50;
            opt.bw = 500;
            opt.max_gap = 5000;
            opt.zdrop = 400;
            opt.zdrop_inv = 240;
            opt.best_n = 5;
            opt.end_bonus = -1;
            opt.min_chain_score = 40;
            opt.min_ksw_len = 200;
            opt.mb_size = 500000000;
        } else {
            return -1;
        }
        0
    }

    /// Original C global function `mb_opt_adap` from `minibwa/options.c:88`.
    pub fn mb_opt_adap(opt0: &mb_opt_t, len: i32, opt: &mut mb_opt_t) {
        let min_len = 100;
        let mid_len = 2000;
        *opt = opt0.clone();
        if (opt0.flag & MB_F_ADAP) == 0 {
            return;
        }
        let a = -0.5f64.ln() / (mid_len - min_len) as f64;
        let b = (-a * ((if len > min_len { len } else { min_len }) - min_len) as f64).exp();
        if opt0.max_gap < 5000 {
            opt.max_gap = (5000.0 - (5000 - opt0.max_gap) as f64 * b + 0.499) as i32;
            if opt.max_gap > len {
                opt.max_gap = len;
            }
        }
        if opt0.bw < 500 {
            opt.bw = (500.0 - (500 - opt0.bw) as f64 * b + 0.499) as i32;
        }
        if opt.bw_long > len * 5 {
            opt.bw_long = len * 5;
        }
        if opt.bw_long < opt.bw {
            opt.bw_long = opt.bw;
        }
        if opt0.zdrop < 400 {
            opt.zdrop = (400.0 - (400 - opt0.zdrop) as f64 * b + 0.499) as i32;
        }
        if opt0.zdrop_inv < 240 {
            opt.zdrop_inv = (240.0 - (240 - opt0.zdrop_inv) as f64 * b + 0.499) as i32;
        }
        if opt0.best_n > 5 {
            opt.best_n = ((opt0.best_n - 5) as f64 * b + 5.0 + 0.499) as i32;
        }
        if opt0.min_dp_max < 50 {
            opt.min_dp_max = (50.0 - (50 - opt0.min_dp_max) as f64 * b + 0.499) as i32;
        }
        if opt0.min_chain_score < 40 {
            opt.min_chain_score = (40.0 - (40 - opt0.min_chain_score) as f64 * b + 0.499) as i32;
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn adap_preset_matches_original_defaults() {
            let mut opt = mb_opt_t::default();
            mb_opt_init(&mut opt);
            assert_eq!(
                opt.flag & (MB_F_PE | MB_F_ADAP | MB_F_NO_KALLOC),
                MB_F_PE | MB_F_ADAP | MB_F_NO_KALLOC
            );
            assert_eq!(opt.min_len, 19);
            assert_eq!(opt.bw, 100);
            assert_eq!(opt.max_gap, 100);
            assert_eq!(opt.zdrop, 80);
            assert_eq!(opt.zdrop_inv, 80);
            assert_eq!(opt.best_n, 50);
            assert_eq!(opt.mb_size, 100000000);
        }

        #[test]
        fn lr_preset_matches_original_defaults() {
            let mut opt = mb_opt_t::default();
            assert_eq!(mb_opt_preset(&mut opt, "lr"), 0);
            assert_ne!(opt.flag & MB_F_LONG, 0);
            assert_eq!(opt.flag & MB_F_PE, 0);
            assert_eq!(opt.bw, 500);
            assert_eq!(opt.max_gap, 5000);
            assert_eq!(opt.zdrop, 400);
            assert_eq!(opt.zdrop_inv, 240);
            assert_eq!(opt.best_n, 5);
            assert_eq!(opt.end_bonus, -1);
            assert_eq!(opt.mb_size, 500000000);
        }

        #[test]
        fn adaptive_options_change_with_length() {
            let mut base = mb_opt_t::default();
            mb_opt_init(&mut base);
            let mut opt = mb_opt_t::default();
            mb_opt_adap(&base, 2000, &mut opt);
            assert_eq!(opt.max_gap, 2000);
            assert_eq!(opt.bw, 300);
            assert_eq!(opt.zdrop, 240);
            assert_eq!(opt.zdrop_inv, 160);
            assert_eq!(opt.best_n, 27);
            assert_eq!(opt.min_dp_max, 40);
            assert_eq!(opt.min_chain_score, 32);
        }
    }
}

pub mod pe {
    #![allow(unused_variables, dead_code, non_snake_case, non_camel_case_types)]

    use crate::align::{mb_append_cigar, mb_update_extra};
    use crate::ksw2::{
        ksw_extz_t, ksw_gen_nt4_mat, KSW_EZ_EXTZ_ONLY, KSW_EZ_REV_CIGAR, KSW_EZ_RIGHT, KSW_LL_SUBO,
    };
    use crate::ksw2_extz2_sse::ksw_extz2_sse;
    use crate::ksw2_ll_sse::{ksw_ll_i16_core, ksw_ll_qinit, ksw_ll_u8_core};
    use crate::l2bit::{l2b_getseq_meth, l2b_meth_rev, l2b_meth_t, l2b_t};
    use crate::map_algo::{
        mb_hit_sort, mb_set_mapq, mb_set_parent, mb_set_sam_pri, mb_sync_high_cov, MB_PARENT_UNSET,
    };
    use crate::mbpriv::{mb_is_sr_mode, mb_seq_rev, KOM_DBG_FLAG, MB_DBG_ALN_PE};
    use crate::options::{mb_opt_t, MB_F_METH, MB_F_PRIMARY5};
    use std::ptr::NonNull;
    use std::sync::atomic::Ordering;

    #[repr(C)]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct mb_extra_t {
        pub cap: u32,
        pub dp_score: i32,
        pub dp_max0: i32,
        pub dp_max: i32,
        pub dp_max2: i32,
        pub n_ambi_cs: u32,
        pub n_cigar: i32,
    }

    impl mb_extra_t {
        pub(crate) const CS_FLAG: u32 = 1 << 31;
        const N_AMBI_MASK: u32 = !Self::CS_FLAG;

        pub fn boxed(self) -> mb_extra_ptr_t {
            mb_extra_ptr_t::from_header_and_cigar(self, &[])
        }

        pub fn with_cigar(self, cigar: &[u32]) -> mb_extra_ptr_t {
            mb_extra_ptr_t::from_header_and_cigar(self, cigar)
        }

        #[inline(always)]
        pub fn n_ambi(&self) -> u32 {
            self.n_ambi_cs & Self::N_AMBI_MASK
        }

        #[inline(always)]
        pub fn set_n_ambi(&mut self, value: u32) {
            self.n_ambi_cs = (self.n_ambi_cs & Self::CS_FLAG) | (value & Self::N_AMBI_MASK);
        }

        #[inline(always)]
        pub fn add_n_ambi(&mut self, value: u32) {
            self.set_n_ambi(self.n_ambi().saturating_add(value));
        }

        #[inline(always)]
        pub fn cs(&self) -> u32 {
            (self.n_ambi_cs >> 31) & 1
        }

        #[inline(always)]
        pub fn set_cs(&mut self, value: u32) {
            if value != 0 {
                self.n_ambi_cs |= Self::CS_FLAG;
            } else {
                self.n_ambi_cs &= !Self::CS_FLAG;
            }
        }
    }

    pub struct mb_extra_ptr_t {
        ptr: NonNull<mb_extra_t>,
    }

    unsafe impl Send for mb_extra_ptr_t {}
    unsafe impl Sync for mb_extra_ptr_t {}

    impl mb_extra_ptr_t {
        pub fn new(cap: u32) -> Self {
            let cap = cap.max(1);
            let size =
                std::mem::size_of::<mb_extra_t>() + cap as usize * std::mem::size_of::<u32>();
            let align = std::mem::align_of::<mb_extra_t>();
            let layout = std::alloc::Layout::from_size_align(size, align).unwrap();
            let raw = unsafe { std::alloc::alloc_zeroed(layout) as *mut mb_extra_t };
            if raw.is_null() {
                std::alloc::handle_alloc_error(layout);
            }
            unsafe {
                (*raw).cap = cap;
            }
            Self {
                ptr: unsafe { NonNull::new_unchecked(raw) },
            }
        }

        pub fn from_header_and_cigar(mut header: mb_extra_t, cigar: &[u32]) -> Self {
            let cap = header.cap.max(cigar.len() as u32).max(1);
            header.cap = cap;
            header.n_cigar = cigar.len() as i32;
            Self::from_header_and_words(header, cigar)
        }

        pub fn from_header_and_words(mut header: mb_extra_t, words: &[u32]) -> Self {
            let cap = header.cap.max(words.len() as u32).max(1);
            header.cap = cap;
            header.n_cigar = header.n_cigar.min(words.len() as i32).max(0);
            let mut out = Self::new(cap);
            unsafe {
                *out.ptr.as_mut() = header;
            }
            if !words.is_empty() {
                unsafe {
                    std::ptr::copy_nonoverlapping(words.as_ptr(), out.cigar_mut_ptr(), words.len());
                }
            }
            out
        }

        pub fn cigar(&self) -> &[u32] {
            let len = self.n_cigar.max(0) as usize;
            if len == 0 {
                &[]
            } else {
                unsafe { std::slice::from_raw_parts(self.cigar_ptr(), len) }
            }
        }

        pub fn cigar_mut(&mut self) -> &mut [u32] {
            let len = self.n_cigar.max(0) as usize;
            if len == 0 {
                &mut []
            } else {
                unsafe { std::slice::from_raw_parts_mut(self.cigar_mut_ptr(), len) }
            }
        }

        pub fn cigar_all(&self) -> &[u32] {
            let mut len = self.n_cigar.max(0) as usize;
            if self.cs() != 0 {
                let cap = self.cap as usize;
                while len < cap {
                    let w = unsafe { *self.cigar_ptr().add(len) };
                    len += 1;
                    if w.to_le_bytes().contains(&0) {
                        break;
                    }
                }
            }
            if len == 0 {
                &[]
            } else {
                unsafe { std::slice::from_raw_parts(self.cigar_ptr(), len.min(self.cap as usize)) }
            }
        }

        pub fn set_cigar_from_vec(&mut self, values: Vec<u32>) {
            self.ensure_capacity(values.len() as u32);
            self.n_cigar = values.len() as i32;
            self.set_cs(0);
            if !values.is_empty() {
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        values.as_ptr(),
                        self.cigar_mut_ptr(),
                        values.len(),
                    );
                }
            }
        }

        pub fn truncate_cigar(&mut self, len: usize) {
            self.n_cigar = self.n_cigar.min(len as i32);
            self.set_cs(0);
        }

        pub fn push_cigar(&mut self, value: u32) {
            let len = self.n_cigar.max(0) as usize;
            self.ensure_capacity((len + 1) as u32);
            unsafe {
                *self.cigar_mut_ptr().add(len) = value;
            }
            self.n_cigar = len as i32 + 1;
            self.set_cs(0);
        }

        pub fn set_tag_words_from_slice(&mut self, values: &[u32]) {
            let len = self.n_cigar.max(0) as usize;
            self.ensure_capacity((len + values.len()) as u32);
            if !values.is_empty() {
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        values.as_ptr(),
                        self.cigar_mut_ptr().add(len),
                        values.len(),
                    );
                }
            }
            self.set_cs(!values.is_empty() as u32);
        }

        pub fn push_word(&mut self, value: u32) {
            let len = self.cigar_all().len();
            self.ensure_capacity((len + 1) as u32);
            unsafe {
                *self.cigar_mut_ptr().add(len) = value;
            }
            self.set_cs(1);
        }

        pub fn extend_cigar_from_slice(&mut self, values: &[u32]) {
            if values.is_empty() {
                return;
            }
            let len = self.n_cigar.max(0) as usize;
            self.ensure_capacity((len + values.len()) as u32);
            unsafe {
                std::ptr::copy_nonoverlapping(
                    values.as_ptr(),
                    self.cigar_mut_ptr().add(len),
                    values.len(),
                );
            }
            self.n_cigar = (len + values.len()) as i32;
            self.set_cs(0);
        }

        pub fn remove_cigar(&mut self, index: usize) -> u32 {
            let len = self.n_cigar.max(0) as usize;
            assert!(index < len);
            let ptr = self.cigar_mut_ptr();
            let value = unsafe { *ptr.add(index) };
            if index + 1 < len {
                unsafe {
                    std::ptr::copy(ptr.add(index + 1), ptr.add(index), len - index - 1);
                }
            }
            self.n_cigar -= 1;
            self.set_cs(0);
            value
        }

        pub fn ensure_capacity(&mut self, needed: u32) {
            if needed <= self.cap {
                return;
            }
            let new_cap = needed.max(1);
            let old_cap = self.cap.max(1);
            let old_size =
                std::mem::size_of::<mb_extra_t>() + old_cap as usize * std::mem::size_of::<u32>();
            let new_size =
                std::mem::size_of::<mb_extra_t>() + new_cap as usize * std::mem::size_of::<u32>();
            let align = std::mem::align_of::<mb_extra_t>();
            let layout = std::alloc::Layout::from_size_align(old_size, align).unwrap();
            let raw = unsafe {
                std::alloc::realloc(self.ptr.as_ptr().cast(), layout, new_size) as *mut mb_extra_t
            };
            if raw.is_null() {
                std::alloc::handle_alloc_error(
                    std::alloc::Layout::from_size_align(new_size, align).unwrap(),
                );
            }
            self.ptr = unsafe { NonNull::new_unchecked(raw) };
            self.cap = new_cap;
        }

        fn cigar_ptr(&self) -> *const u32 {
            unsafe {
                (self.ptr.as_ptr() as *const u8).add(std::mem::size_of::<mb_extra_t>())
                    as *const u32
            }
        }

        fn cigar_mut_ptr(&mut self) -> *mut u32 {
            unsafe {
                (self.ptr.as_ptr() as *mut u8).add(std::mem::size_of::<mb_extra_t>()) as *mut u32
            }
        }
    }

    impl Clone for mb_extra_ptr_t {
        fn clone(&self) -> Self {
            Self::from_header_and_words(**self, self.cigar_all())
        }
    }

    impl Drop for mb_extra_ptr_t {
        fn drop(&mut self) {
            let cap = self.cap.max(1);
            let size =
                std::mem::size_of::<mb_extra_t>() + cap as usize * std::mem::size_of::<u32>();
            let align = std::mem::align_of::<mb_extra_t>();
            let layout = std::alloc::Layout::from_size_align(size, align).unwrap();
            unsafe { std::alloc::dealloc(self.ptr.as_ptr().cast(), layout) };
        }
    }

    impl std::fmt::Debug for mb_extra_ptr_t {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("mb_extra_ptr_t")
                .field("header", &**self)
                .field("cigar", &self.cigar())
                .finish()
        }
    }

    impl PartialEq for mb_extra_ptr_t {
        fn eq(&self, other: &Self) -> bool {
            **self == **other && self.cigar_all() == other.cigar_all()
        }
    }

    impl Eq for mb_extra_ptr_t {}

    impl Default for mb_extra_ptr_t {
        fn default() -> Self {
            Self::new(1)
        }
    }

    impl std::ops::Deref for mb_extra_ptr_t {
        type Target = mb_extra_t;

        fn deref(&self) -> &Self::Target {
            unsafe { self.ptr.as_ref() }
        }
    }

    impl std::ops::DerefMut for mb_extra_ptr_t {
        fn deref_mut(&mut self) -> &mut Self::Target {
            unsafe { self.ptr.as_mut() }
        }
    }

    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    pub struct mb_hit_t {
        pub tid: i64,
        pub ts: i64,
        pub te: i64,
        pub id: i32,
        pub cnt: i32,
        pub score: i32,
        pub score0: i32,
        pub as_: i32,
        pub qs: i32,
        pub qe: i32,
        pub parent: i32,
        pub n_sub: i32,
        pub subsc: i32,
        pub mlen: i32,
        pub blen: i32,
        pub mapq: i32,
        pub hash: u32,
        pub flags: u32,
        pub p: Option<mb_extra_ptr_t>,
    }

    impl mb_hit_t {
        const REV: u32 = 1 << 0;
        const PROPER_PAIR: u32 = 1 << 1;
        const SAM_PRI: u32 = 1 << 2;
        const FLT: u32 = 1 << 3;
        const INV: u32 = 1 << 4;
        const SPLIT_SHIFT: u32 = 5;
        const SPLIT_MASK: u32 = 0b11 << Self::SPLIT_SHIFT;
        const SPLIT_INV: u32 = 1 << 7;
        const RESCUED: u32 = 1 << 8;
        const FRAC_HIGH_SHIFT: u32 = 9;
        const FRAC_HIGH_MASK: u32 = 0xff << Self::FRAC_HIGH_SHIFT;

        #[inline(always)]
        fn bit(&self, mask: u32) -> u8 {
            ((self.flags & mask) != 0) as u8
        }

        #[inline(always)]
        fn set_bit(&mut self, mask: u32, value: u8) {
            if value != 0 {
                self.flags |= mask;
            } else {
                self.flags &= !mask;
            }
        }

        #[inline(always)]
        pub fn rev(&self) -> u8 {
            self.bit(Self::REV)
        }

        #[inline(always)]
        pub fn set_rev(&mut self, value: u8) {
            self.set_bit(Self::REV, value);
        }

        #[inline(always)]
        pub fn proper_pair(&self) -> u8 {
            self.bit(Self::PROPER_PAIR)
        }

        #[inline(always)]
        pub fn set_proper_pair(&mut self, value: u8) {
            self.set_bit(Self::PROPER_PAIR, value);
        }

        #[inline(always)]
        pub fn sam_pri(&self) -> u8 {
            self.bit(Self::SAM_PRI)
        }

        #[inline(always)]
        pub fn set_sam_pri(&mut self, value: u8) {
            self.set_bit(Self::SAM_PRI, value);
        }

        #[inline(always)]
        pub fn flt(&self) -> u8 {
            self.bit(Self::FLT)
        }

        #[inline(always)]
        pub fn set_flt(&mut self, value: u8) {
            self.set_bit(Self::FLT, value);
        }

        #[inline(always)]
        pub fn inv(&self) -> u8 {
            self.bit(Self::INV)
        }

        #[inline(always)]
        pub fn set_inv(&mut self, value: u8) {
            self.set_bit(Self::INV, value);
        }

        #[inline(always)]
        pub fn split(&self) -> u8 {
            ((self.flags & Self::SPLIT_MASK) >> Self::SPLIT_SHIFT) as u8
        }

        #[inline(always)]
        pub fn set_split(&mut self, value: u8) {
            self.flags = (self.flags & !Self::SPLIT_MASK)
                | (((value as u32) << Self::SPLIT_SHIFT) & Self::SPLIT_MASK);
        }

        #[inline(always)]
        pub fn split_inv(&self) -> u8 {
            self.bit(Self::SPLIT_INV)
        }

        #[inline(always)]
        pub fn set_split_inv(&mut self, value: u8) {
            self.set_bit(Self::SPLIT_INV, value);
        }

        #[inline(always)]
        pub fn rescued(&self) -> u8 {
            self.bit(Self::RESCUED)
        }

        #[inline(always)]
        pub fn set_rescued(&mut self, value: u8) {
            self.set_bit(Self::RESCUED, value);
        }

        #[inline(always)]
        pub fn frac_high(&self) -> u8 {
            ((self.flags & Self::FRAC_HIGH_MASK) >> Self::FRAC_HIGH_SHIFT) as u8
        }

        #[inline(always)]
        pub fn set_frac_high(&mut self, value: u8) {
            self.flags =
                (self.flags & !Self::FRAC_HIGH_MASK) | ((value as u32) << Self::FRAC_HIGH_SHIFT);
        }

        pub const fn flags_with(
            rev: u8,
            proper_pair: u8,
            sam_pri: u8,
            flt: u8,
            inv: u8,
            split: u8,
            split_inv: u8,
            rescued: u8,
            frac_high: u8,
        ) -> u32 {
            (rev as u32 & 1)
                | ((proper_pair as u32 & 1) << 1)
                | ((sam_pri as u32 & 1) << 2)
                | ((flt as u32 & 1) << 3)
                | ((inv as u32 & 1) << 4)
                | ((split as u32 & 3) << Self::SPLIT_SHIFT)
                | ((split_inv as u32 & 1) << 7)
                | ((rescued as u32 & 1) << 8)
                | ((frac_high as u32) << Self::FRAC_HIGH_SHIFT)
        }
    }

    pub struct mb_hit_buf_t {
        ptr: *mut mb_hit_t,
        len: u32,
        cap: u32,
    }

    unsafe impl Send for mb_hit_buf_t {}
    unsafe impl Sync for mb_hit_buf_t {}

    impl mb_hit_buf_t {
        #[inline]
        pub fn from_vec(mut v: Vec<mb_hit_t>) -> Self {
            let len = v.len();
            let cap = v.capacity();
            assert!(u32::try_from(len).is_ok() && u32::try_from(cap).is_ok());
            if cap == 0 {
                return Self::default();
            }
            let ptr = v.as_mut_ptr();
            std::mem::forget(v);
            Self {
                ptr,
                len: len as u32,
                cap: cap as u32,
            }
        }

        #[inline]
        pub fn into_vec(mut self) -> Vec<mb_hit_t> {
            let v = if self.cap == 0 {
                Vec::new()
            } else {
                unsafe { Vec::from_raw_parts(self.ptr, self.len as usize, self.cap as usize) }
            };
            self.ptr = std::ptr::null_mut();
            self.len = 0;
            self.cap = 0;
            v
        }

        #[inline]
        pub fn as_slice(&self) -> &[mb_hit_t] {
            if self.len == 0 {
                &[]
            } else {
                unsafe { std::slice::from_raw_parts(self.ptr, self.len as usize) }
            }
        }
    }

    impl Default for mb_hit_buf_t {
        fn default() -> Self {
            Self {
                ptr: std::ptr::null_mut(),
                len: 0,
                cap: 0,
            }
        }
    }

    impl Drop for mb_hit_buf_t {
        fn drop(&mut self) {
            if self.cap != 0 {
                unsafe {
                    drop(Vec::from_raw_parts(
                        self.ptr,
                        self.len as usize,
                        self.cap as usize,
                    ));
                }
            }
        }
    }

    impl Clone for mb_hit_buf_t {
        fn clone(&self) -> Self {
            Self::from_vec(self.as_slice().to_vec())
        }
    }

    impl std::fmt::Debug for mb_hit_buf_t {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_list().entries(self.as_slice()).finish()
        }
    }

    impl AsRef<[mb_hit_t]> for mb_hit_buf_t {
        #[inline]
        fn as_ref(&self) -> &[mb_hit_t] {
            self.as_slice()
        }
    }

    impl std::ops::Deref for mb_hit_buf_t {
        type Target = [mb_hit_t];

        #[inline]
        fn deref(&self) -> &Self::Target {
            self.as_slice()
        }
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq)]
    pub struct mb_pestat_t {
        pub lo: i32,
        pub hi: i32,
        pub failed: i32,
        pub avg: f64,
        pub std: f64,
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct mb_pairaux_t {
        pub score: i32,
        pub sub_sc: i32,
        pub n_sub: i32,
        pub n_pp: i32,
        pub i: [i32; 2],
    }

    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    pub struct mb_hit_v {
        pub n: i32,
        pub m: i32,
        pub a: Vec<mb_hit_t>,
    }

    /// Original C static function `mb_insert_dir` from `minibwa/pe.c:10`.
    pub fn mb_insert_dir(h0: &mb_hit_t, h1: &mb_hit_t, dist: &mut i64) -> i32 {
        let p0 = if h0.rev() != 0 { h0.te } else { h0.ts };
        let p1 = if h1.rev() != 0 { h1.te } else { h1.ts };
        *dist = if p0 > p1 { p0 - p1 } else { p1 - p0 };
        (((h0.rev() as i32) << 1 | h1.rev() as i32) ^ if p0 < p1 { 0 } else { 3 }) as i32
    }

    /// Original C static function `mb_pair_score` from `minibwa/pe.c:19`.
    pub fn mb_pair_score(
        h0: &mb_hit_t,
        h1: &mb_hit_t,
        pes: &[mb_pestat_t; 4],
        match_sc: i32,
    ) -> f64 {
        const MB_SQRT1_2: f64 = 0.707106781186547524401;
        let mut dist = 0i64;
        let dir = mb_insert_dir(h0, h1, &mut dist) as usize;
        if pes[dir].failed == 0 && dist >= pes[dir].lo as i64 && dist <= pes[dir].hi as i64 {
            let ns = (dist as f64 - pes[dir].avg) / pes[dir].std;
            let dp0 = h0.p.as_ref().map(|p| p.dp_max).unwrap_or(0);
            let dp1 = h1.p.as_ref().map(|p| p.dp_max).unwrap_or(0);
            return dp0 as f64
                + dp1 as f64
                + 0.721
                    * libm::erfc(ns.abs() * MB_SQRT1_2).mul_add(2.0, 0.0).ln()
                    * match_sc as f64;
        }
        -1.0
    }

    /// Original C static function `mb_select_unique_se` from `minibwa/pe.c:32`.
    pub fn mb_select_unique_se(n_hit: i32, hit: &[mb_hit_t]) -> Option<&mb_hit_t> {
        let mut n_pri = 0;
        let mut mapq = 0;
        let mut k = None;
        for j in 0..n_hit as usize {
            if hit[j].id == hit[j].parent {
                n_pri += 1;
                mapq = hit[j].mapq;
                k = Some(j);
            }
        }
        if n_pri == 1 && mapq >= 10 {
            k.map(|i| &hit[i])
        } else {
            None
        }
    }

    /// Original C static function `mb_hit_sum_score` from `minibwa/pe.c:41`.
    pub fn mb_hit_sum_score(km: (), n_hit: i32, hit: &[mb_hit_t]) -> i32 {
        let mut n_pri = 0;
        let mut sc = 0;
        for h in hit.iter().take(n_hit as usize) {
            if h.id == h.parent && h.p.is_some() {
                n_pri += 1;
                sc = h.p.as_ref().unwrap().dp_max;
            }
        }
        if n_pri == 0 {
            return 0;
        }
        if n_pri == 1 {
            return sc;
        }
        let mut a = Vec::with_capacity(n_pri as usize);
        for (i, h) in hit.iter().take(n_hit as usize).enumerate() {
            if h.id == h.parent && h.p.is_some() {
                a.push(((h.qs as u64) << 32) | i as u64);
            }
        }
        a.sort_unstable();
        sc = 0;
        let mut qe = 0;
        for x in a {
            let h = &hit[x as u32 as usize];
            if h.qe <= qe {
                continue;
            }
            let dp_max = h.p.as_ref().unwrap().dp_max;
            if h.qs < qe {
                sc += ((h.qe - qe) as f64 / (h.qe - h.qs) as f64 * dp_max as f64 + 0.499) as i32;
            } else {
                sc += dp_max;
            }
            qe = h.qe;
        }
        sc
    }

    /// Original C global function `mb_pestat` from `minibwa/pe.c:68`.
    pub fn mb_pestat<H: AsRef<[mb_hit_t]>>(
        km: (),
        opt: &mb_opt_t,
        n_frag: i32,
        seg_off: &[i32],
        seg_cnt: &[i32],
        n_hit: &[i32],
        hit: &[H],
        pes: &mut [mb_pestat_t; 4],
    ) {
        const MIN_DIR_CNT: i32 = 20;
        const MIN_DIR_RATIO: f64 = 0.05;
        const OUTLIER_BOUND: f64 = 2.0;
        const MAPPING_BOUND: f64 = 3.0;
        const MAX_STDDEV: f64 = 4.0;
        let mut is = [Vec::<u64>::new(), Vec::new(), Vec::new(), Vec::new()];
        *pes = [mb_pestat_t::default(); 4];
        for i in 0..n_frag as usize {
            if seg_cnt[i] != 2 {
                continue;
            }
            let off = seg_off[i] as usize;
            let r0 = mb_select_unique_se(n_hit[off], hit[off].as_ref());
            let r1 = mb_select_unique_se(n_hit[off + 1], hit[off + 1].as_ref());
            let (Some(r0), Some(r1)) = (r0, r1) else {
                continue;
            };
            if r0.tid != r1.tid {
                continue;
            }
            let mut dist = 0i64;
            let dir = mb_insert_dir(r0, r1, &mut dist) as usize;
            if dist < opt.max_pe_ins as i64 {
                is[dir].push(dist as u64);
            }
        }
        let max = is.iter().map(|v| v.len() as i32).max().unwrap_or(0);
        for d in 0..4usize {
            let q = &mut is[d];
            let r = &mut pes[d];
            if (q.len() as i32) < MIN_DIR_CNT || (q.len() as f64) < (max as f64) * MIN_DIR_RATIO {
                r.failed = 1;
                continue;
            }
            q.sort_unstable();
            let n = q.len() as f64;
            let p25 = q[(0.25 * n + 0.499) as usize] as i32;
            let p50 = q[(0.50 * n + 0.499) as usize] as i32;
            let p75 = q[(0.75 * n + 0.499) as usize] as i32;
            r.lo = (p25 as f64 - OUTLIER_BOUND * (p75 - p25) as f64 + 0.499) as i32;
            if r.lo < 1 {
                r.lo = 1;
            }
            r.hi = (p75 as f64 + OUTLIER_BOUND * (p75 - p25) as f64 + 0.499) as i32;
            let mut x = 0i32;
            r.avg = 0.0;
            for &v in q.iter() {
                if v >= r.lo as u64 && v <= r.hi as u64 {
                    r.avg += v as f64;
                    x += 1;
                }
            }
            if x == 0 {
                r.failed = 1;
                continue;
            }
            r.avg /= x as f64;
            r.std = 0.0;
            for &v in q.iter() {
                if v >= r.lo as u64 && v <= r.hi as u64 {
                    let z = v as f64 - r.avg;
                    r.std += z * z;
                }
            }
            r.std = (r.std / x as f64).sqrt();
            let _ = p50;
            r.lo = (p25 as f64 - MAPPING_BOUND * (p75 - p25) as f64 + 0.499) as i32;
            r.hi = (p75 as f64 + MAPPING_BOUND * (p75 - p25) as f64 + 0.499) as i32;
            let lo_std = (r.avg - MAX_STDDEV * r.std + 0.499) as i32;
            let hi_std = (r.avg + MAX_STDDEV * r.std + 0.499) as i32;
            if r.lo > lo_std {
                r.lo = lo_std;
            }
            if r.hi < hi_std {
                r.hi = hi_std;
            }
            if r.lo < 1 {
                r.lo = 1;
            }
        }
    }

    /// Original C static function `mb_pair_hits` from `minibwa/pe.c:140`.
    pub fn mb_pair_hits(
        km: (),
        opt: &mb_opt_t,
        l2b: &l2b_t,
        n_hit: [i32; 2],
        hit: &mut [Vec<mb_hit_t>; 2],
        pes: &[mb_pestat_t; 4],
        ret: &mut mb_pairaux_t,
    ) {
        *ret = mb_pairaux_t {
            score: -1,
            sub_sc: -1,
            i: [-1, -1],
            ..Default::default()
        };
        if n_hit[0] == 0 || n_hit[1] == 0 {
            return;
        }
        let mut pa = Vec::<(u64, u64)>::with_capacity((n_hit[0] + n_hit[1]) as usize);
        for r in 0..2usize {
            for i in 0..n_hit[r] as usize {
                let h = &mut hit[r][i];
                h.set_proper_pair(0);
                let p = l2b.ctg[h.tid as usize].off + if h.rev() != 0 { h.te } else { h.ts } as u64;
                pa.push((p, ((i as u64) << 2) | ((h.rev() as u64) << 1) | r as u64));
            }
        }
        pa.sort_unstable_by_key(|&(x, y)| (x, y));
        let mut y = [-1i32; 4];
        let mut pp = Vec::<(u64, u64)>::new();
        for i in 0..pa.len() {
            let (pix, piy) = pa[i];
            let pi_read = (piy & 1) as usize;
            let pi_idx = (piy >> 2) as usize;
            let hi_tid = hit[pi_read][pi_idx].tid;
            for r in 0..2usize {
                let dir = (r << 1) | ((piy >> 1) as usize & 1);
                if pes[dir].failed != 0 {
                    continue;
                }
                let which = (r << 1) | ((piy as usize & 1) ^ 1);
                if y[which] < 0 {
                    continue;
                }
                let mut k = y[which] as isize;
                while k >= 0 {
                    let (pkx, pky) = pa[k as usize];
                    if (pky & 3) as usize != which {
                        k -= 1;
                        continue;
                    }
                    let pk_read = (pky & 1) as usize;
                    let pk_idx = (pky >> 2) as usize;
                    let hk_tid = hit[pk_read][pk_idx].tid;
                    if hi_tid != hk_tid {
                        break;
                    }
                    let dist = pix as i64 - pkx as i64;
                    if dist > pes[dir].hi as i64 {
                        break;
                    }
                    if dist >= pes[dir].lo as i64 {
                        hit[pk_read][pk_idx].set_proper_pair(1);
                        hit[pi_read][pi_idx].set_proper_pair(1);
                        let mut s = if pk_read == pi_read {
                            -1.0
                        } else {
                            let (left, right) = hit.split_at(1);
                            if pk_read == 0 {
                                mb_pair_score(&left[0][pk_idx], &right[0][pi_idx], pes, opt.a)
                            } else {
                                mb_pair_score(&left[0][pi_idx], &right[0][pk_idx], pes, opt.a)
                            }
                        };
                        if s < 0.0 {
                            s = 0.0;
                        }
                        let yv = if (pky & 1) == 0 {
                            ((pk_idx as u64) << 32) | pi_idx as u64
                        } else {
                            ((pi_idx as u64) << 32) | pk_idx as u64
                        };
                        let hash = hit[pk_read][pk_idx].hash ^ hit[pi_read][pi_idx].hash;
                        pp.push((((s + 0.499) as u64) << 32 | hash as u64, yv));
                    }
                    k -= 1;
                }
            }
            y[(piy & 3) as usize] = i as i32;
        }
        ret.n_pp = pp.len() as i32;
        if !pp.is_empty() {
            let mut max = 0u64;
            let mut max2 = 0u64;
            let tmp = (opt.a + opt.b).max(opt.q + opt.e);
            for &(x, yv) in &pp {
                if x > max {
                    max2 = max;
                    max = x;
                    ret.i[0] = (yv >> 32) as i32;
                    ret.i[1] = yv as u32 as i32;
                } else if x > max2 {
                    max2 = x;
                }
            }
            ret.score = (max >> 32) as i32;
            ret.sub_sc = (max2 >> 32) as i32;
            for &(x, _) in &pp {
                if (x >> 32) as i32 >= ret.score - tmp {
                    ret.n_sub += 1;
                }
            }
        }
    }

    /// Original C static function `mb_ungap` from `minibwa/pe.c:220`.
    pub fn mb_ungap(
        km: (),
        qlen: i32,
        qseq: &[u8],
        tlen: i32,
        tseq: &[u8],
        kmer: i32,
        max_i: &mut i32,
        n_good: &mut i32,
        n_kmer: &mut i32,
    ) -> i32 {
        *max_i = -1;
        *n_good = 0;
        *n_kmer = 0;
        if qlen >= u16::MAX as i32 || kmer <= 0 {
            return 0;
        }
        let cap = 1usize << (kmer * 2);
        let mask = cap - 1;
        let mut a = vec![0i32; tlen.max(0) as usize];
        let mut h = vec![0u16; cap];
        let mut l = 0i32;
        let mut x = 0usize;
        for i in 0..qlen as usize {
            if qseq[i] < 4 {
                x = ((x << 2) | qseq[i] as usize) & mask;
                l += 1;
                if l >= kmer {
                    if h[x] == 0 {
                        *n_kmer += 1;
                    }
                    h[x] = i as u16;
                }
            } else {
                x = 0;
                l = 0;
            }
        }
        l = 0;
        x = 0;
        for i in 0..tlen as usize {
            if tseq[i] < 4 {
                x = ((x << 2) | tseq[i] as usize) & mask;
                l += 1;
                if l >= kmer && h[x] > 0 && i >= h[x] as usize {
                    a[i - h[x] as usize] += 1;
                }
            } else {
                x = 0;
                l = 0;
            }
        }
        let mut max = 0;
        for (i, &v) in a.iter().enumerate() {
            if max < v {
                max = v;
                *max_i = i as i32;
            }
        }
        for &v in &a {
            if v > max >> 1 {
                *n_good += 1;
            }
        }
        max
    }

    /// Original C static function `mb_matesw_align` from `minibwa/pe.c:245`.
    pub fn mb_matesw_align(
        km: (),
        opt: &mb_opt_t,
        qlen: i32,
        qseq: &mut [u8],
        tlen: i32,
        tseq: &mut [u8],
        h: &mut mb_hit_t,
        min_sc: i32,
        ez: &mut ksw_extz_t,
    ) {
        *h = mb_hit_t::default();
        let max_sc = qlen.min(tlen);
        let b_mm = (opt.b + opt.a - 1) / opt.a;
        let b_ts = (opt.b_ts + opt.a - 1) / opt.a;
        let b_ambi = (opt.b_ambi + opt.a - 1) / opt.a;
        let gapo = (opt.q + opt.a - 1) / opt.a;
        let gape = (opt.e + opt.a - 1) / opt.a;
        if max_sc >= 32767 || qlen <= 0 || tlen <= 0 {
            return;
        }
        let mut mat = [0i8; 25];
        ksw_gen_nt4_mat(&mut mat, 1, b_mm as i8, b_ts as i8, b_ambi as i8);
        let sz = if max_sc < 255 - b_mm { 1 } else { 2 };
        let xtra = KSW_LL_SUBO | opt.min_len;
        let rst = if let Some(rst) =
            crate::ksw2_c_sse::maybe_ll_core(sz, qlen, qseq, &mat, tlen, tseq, gapo, gape, xtra)
        {
            rst
        } else {
            let qp = ksw_ll_qinit(km, sz, qlen, qseq, 5, &mat);
            if sz == 1 {
                ksw_ll_u8_core(&qp, tlen, tseq, gapo, gape, xtra)
            } else {
                ksw_ll_i16_core(&qp, tlen, tseq, gapo, gape, xtra)
            }
        };
        if (KOM_DBG_FLAG.load(Ordering::Relaxed) & MB_DBG_ALN_PE) != 0 {
            eprintln!(
                "===> qlen={}; tlen={}; score={}; qe={}; te={} <===",
                qlen,
                tlen,
                rst.score,
                rst.qe + 1,
                rst.te + 1
            );
            let alphabet = b"ACGTN";
            eprintln!(
                "{}",
                qseq.iter()
                    .take(qlen.max(0) as usize)
                    .map(|&c| alphabet[c.min(4) as usize] as char)
                    .collect::<String>()
            );
            eprintln!(
                "{}",
                tseq.iter()
                    .take(tlen.max(0) as usize)
                    .map(|&c| alphabet[c.min(4) as usize] as char)
                    .collect::<String>()
            );
        }
        if rst.score >= opt.min_dp_max && rst.score >= min_sc {
            let te = rst.te + 1;
            let qe = rst.qe + 1;
            if te <= 0 || qe <= 0 {
                return;
            }
            mb_seq_rev(qe as u32, qseq);
            mb_seq_rev(te as u32, tseq);
            ksw_gen_nt4_mat(
                &mut mat,
                opt.a as i8,
                opt.b as i8,
                opt.b_ts as i8,
                opt.b_ambi as i8,
            );
            ksw_extz2_sse(
                km,
                qe,
                qseq,
                te,
                tseq,
                5,
                &mat,
                opt.q as i8,
                opt.e as i8,
                opt.bw,
                opt.zdrop,
                opt.end_bonus,
                KSW_EZ_EXTZ_ONLY | KSW_EZ_RIGHT | KSW_EZ_REV_CIGAR,
                ez,
            );
            mb_seq_rev(qe as u32, qseq);
            mb_seq_rev(te as u32, tseq);
            if ez.n_cigar > 0 && ez.max as i32 >= opt.min_dp_max * opt.a {
                h.p = Some(mb_extra_t::default().boxed());
                let cigar = ez.cigar.clone();
                mb_append_cigar(h, ez.n_cigar as u32, &cigar);
                h.set_rescued(1);
                h.qe = qe;
                h.te = te as i64;
                h.ts = (te
                    - if ez.reach_end != 0 {
                        ez.mqe_t + 1
                    } else {
                        ez.max_t + 1
                    }) as i64;
                h.qs = qe - if ez.reach_end != 0 { qe } else { ez.max_q + 1 };
                if let Some(p) = &mut h.p {
                    p.dp_max = ez.max as i32;
                    p.dp_score = ez.max as i32;
                    p.dp_max2 =
                        ((ez.max as f64 / rst.score as f64) * rst.score2 as f64 + 0.499) as i32;
                    if p.dp_max2 < 0 {
                        p.dp_max2 = 0;
                    }
                }
                h.score = rst.score;
                h.score0 = rst.score;
                h.subsc = rst.score2;
                h.cnt = 0;
                h.as_ = -1;
                h.parent = MB_PARENT_UNSET;
                if (KOM_DBG_FLAG.load(Ordering::Relaxed) & MB_DBG_ALN_PE) != 0 {
                    let cigar = ez
                        .cigar
                        .iter()
                        .take(ez.n_cigar as usize)
                        .map(|&c| {
                            format!(
                                "{}{}",
                                c >> 4,
                                crate::align::MB_CIGAR_STR.as_bytes()[(c & 0xf) as usize] as char
                            )
                        })
                        .collect::<String>();
                    eprintln!(
                        "max={}; ts={}; qs={}; reach_end={}; cigar={}",
                        ez.max, h.ts, h.qs, ez.reach_end, cigar
                    );
                }
                mb_update_extra(
                    km,
                    h,
                    &qseq[h.qs.max(0) as usize..qe as usize],
                    &tseq[h.ts.max(0) as usize..te as usize],
                    &mat,
                    opt.q as i8,
                    opt.e as i8,
                    opt.flag,
                    0,
                );
            }
        }
    }

    /// Original C static function `mb_matesw_core` from `minibwa/pe.c:314`.
    pub fn mb_matesw_core(
        km: (),
        opt: &mb_opt_t,
        l2b: &l2b_t,
        pes: &[mb_pestat_t; 4],
        h0: &mb_hit_t,
        r0: i32,
        len: i32,
        seq: &mut [Vec<u8>; 2],
        mt0: l2b_meth_t,
        h1: &mut mb_hit_v,
        min_sc: i32,
        ez: &mut ksw_extz_t,
    ) -> Option<f64> {
        let mut skip = [false; 4];
        for dir in 0..4usize {
            skip[dir] = pes[dir].failed != 0;
        }
        if skip.iter().all(|&x| x) {
            return None;
        }
        let pos5 = if h0.rev() != 0 { h0.te } else { h0.ts };
        let mut ret = None;
        for dir in 0..4usize {
            if skip[dir] {
                continue;
            }
            let is_rev = (((dir >> 1) != (dir & 1)) as u8 ^ h0.rev()) != 0;
            let is_larger = if (dir >> 1) != (dir & 1) {
                (((dir >> 1) as i32) ^ is_rev as i32) != 0
            } else {
                (((dir >> 1) as i32) ^ r0 ^ h0.rev() as i32) != 0
            };
            let mut ts = (if is_larger {
                pos5 + pes[dir].lo as i64
            } else {
                pos5 - pes[dir].hi as i64
            }) - if !is_rev { 0 } else { len as i64 };
            let mut te = (if is_larger {
                pos5 + pes[dir].hi as i64
            } else {
                pos5 - pes[dir].lo as i64
            }) + if !is_rev { len as i64 } else { 0 };
            if ts < 0 {
                ts = 0;
            }
            let ctg_len = l2b.ctg[h0.tid as usize].len as i64;
            if te > ctg_len {
                te = ctg_len;
            }
            if te - ts <= len as i64 {
                continue;
            }
            let mut ts2 = ts;
            let mut te2 = te;
            let mut refseq = vec![0u8; (te - ts) as usize];
            let mt = if is_rev { l2b_meth_rev(mt0) } else { mt0 };
            l2b_getseq_meth(l2b, h0.tid, ts, te, mt, &mut refseq);
            let seq_idx = is_rev as usize;
            let mut max_i = 0;
            let mut n_good = 0;
            let mut n_kmer = 0;
            let max_ug = mb_ungap(
                km,
                len,
                &seq[seq_idx],
                (te - ts) as i32,
                &refseq,
                7,
                &mut max_i,
                &mut n_good,
                &mut n_kmer,
            );
            if max_ug >= 10 && max_ug >= len >> 1 && n_good == 1 {
                ts2 = ts + max_i as i64 - (len / 2) as i64;
                te2 = ts2 + (len * 2) as i64;
                if ts2 < ts {
                    ts2 = ts;
                }
                if te2 > te {
                    te2 = te;
                }
            }
            if max_ug >= 10 || (max_ug as f64) >= n_kmer as f64 * 0.33 {
                let mut ht = mb_hit_t {
                    p: None,
                    ..Default::default()
                };
                let offset = (ts2 - ts) as usize;
                mb_matesw_align(
                    km,
                    opt,
                    len,
                    &mut seq[seq_idx],
                    (te2 - ts2) as i32,
                    &mut refseq[offset..offset + (te2 - ts2) as usize],
                    &mut ht,
                    min_sc,
                    ez,
                );
                if ht.p.is_some() {
                    ht.tid = h0.tid;
                    ht.ts += ts2;
                    ht.te += ts2;
                    ht.set_rev(is_rev as u8);
                    if is_rev {
                        let qt = ht.qs;
                        ht.qs = len - ht.qe;
                        ht.qe = len - qt;
                    }
                    let score = mb_pair_score(h0, &ht, pes, opt.a);
                    h1.a.push(ht);
                    h1.n = h1.a.len() as i32;
                    h1.m = h1.n;
                    ret = Some(score);
                }
            }
        }
        ret
    }

    /// Original C static function `mb_matesw` from `minibwa/pe.c:375`.
    pub fn mb_matesw(
        km: (),
        opt: &mb_opt_t,
        l2b: &l2b_t,
        n_hit: &mut [i32; 2],
        hit: &mut [Vec<mb_hit_t>; 2],
        pes: &[mb_pestat_t; 4],
        paux0: &mb_pairaux_t,
        qlen: [i32; 2],
        qseq: [&str; 2],
        is_meth: i32,
    ) -> i32 {
        if opt.max_rescue == 0 {
            return 0;
        }
        let mut n_res = 0i32;
        for r in 0..2usize {
            let mut m = 0;
            for i in 0..n_hit[r] as usize {
                if m >= opt.max_rescue {
                    break;
                }
                let dp = hit[r][i].p.as_ref().map(|p| p.dp_max).unwrap_or(0);
                let best = hit[r][0].p.as_ref().map(|p| p.dp_max).unwrap_or(0);
                if hit[r][i].proper_pair() == 0 && dp >= best - opt.pen_unpair * opt.a {
                    m += 1;
                    n_res += 1;
                }
            }
        }
        if n_res == 0 {
            return 0;
        }
        let mut a = Vec::<(u64, u64)>::with_capacity(n_res as usize);
        for r in 0..2usize {
            let mut m = 0;
            for i in 0..n_hit[r] as usize {
                if m >= opt.max_rescue {
                    break;
                }
                let dp = hit[r][i].p.as_ref().map(|p| p.dp_max).unwrap_or(0);
                let best = hit[r][0].p.as_ref().map(|p| p.dp_max).unwrap_or(0);
                if hit[r][i].proper_pair() == 0 && dp >= best - opt.pen_unpair * opt.a {
                    a.push((
                        ((dp as u64) << 32) | hit[r][i].hash as u64,
                        ((i as u64) << 1) | r as u64,
                    ));
                    m += 1;
                }
            }
        }
        a.sort_unstable_by_key(|&(x, y)| (x, y));
        let mut qs = [
            [
                Vec::<u8>::with_capacity(qlen[0] as usize),
                Vec::<u8>::with_capacity(qlen[0] as usize),
            ],
            [
                Vec::<u8>::with_capacity(qlen[1] as usize),
                Vec::<u8>::with_capacity(qlen[1] as usize),
            ],
        ];
        for r in 0..2usize {
            let bases = qseq[r].as_bytes();
            qs[r][0] = bases
                .iter()
                .take(qlen[r] as usize)
                .map(|&c| match c {
                    b'A' | b'a' => 0,
                    b'C' | b'c' => 1,
                    b'G' | b'g' => 2,
                    b'T' | b't' => 3,
                    _ => 4,
                })
                .collect();
            qs[r][1] = qs[r][0]
                .iter()
                .rev()
                .map(|&c| if c < 4 { 3 - c } else { 4 })
                .collect();
        }
        let min_sc = [
            mb_hit_sum_score(km, n_hit[0], &hit[0]) / opt.a - opt.pen_unpair,
            mb_hit_sum_score(km, n_hit[1], &hit[1]) / opt.a - opt.pen_unpair,
        ];
        let mut ha = [
            mb_hit_v {
                n: n_hit[0],
                m: n_hit[0],
                a: std::mem::take(&mut hit[0]),
            },
            mb_hit_v {
                n: n_hit[1],
                m: n_hit[1],
                a: std::mem::take(&mut hit[1]),
            },
        ];
        let mut max = [paux0.score, paux0.score];
        let mut max2 = [paux0.sub_sc, paux0.sub_sc];
        let mut skip = [false, false];
        let mut ez = ksw_extz_t {
            m_cigar: 16,
            cigar: Vec::with_capacity(16),
            ..Default::default()
        };
        for &(_, yv) in a.iter().rev() {
            let r = (yv & 1) as usize;
            let j = (yv >> 1) as usize;
            if skip[r] {
                continue;
            }
            let mt = if is_meth == 0 {
                l2b_meth_t::L2B_METH_NONE
            } else if r == 0 {
                l2b_meth_t::L2B_METH_G2A
            } else {
                l2b_meth_t::L2B_METH_C2T
            };
            let h0 = ha[r].a[j].clone();
            if let Some(sc) = mb_matesw_core(
                km,
                opt,
                l2b,
                pes,
                &h0,
                r as i32,
                qlen[1 - r],
                &mut qs[1 - r],
                mt,
                &mut ha[1 - r],
                min_sc[1 - r],
                &mut ez,
            ) {
                let sc = sc as i32;
                if sc > max[r] {
                    max2[r] = max[r];
                    max[r] = sc;
                } else if sc > max2[r] {
                    max2[r] = sc;
                }
                if max[r] == max2[r] {
                    skip[r] = true;
                }
            }
        }
        let n_add = (ha[0].n - n_hit[0]) + (ha[1].n - n_hit[1]);
        for r in 0..2usize {
            n_hit[r] = ha[r].n;
            hit[r] = std::mem::take(&mut ha[r].a);
        }
        n_add
    }

    /// Original C global function `mb_pair` from `minibwa/pe.c:456`.
    pub fn mb_pair(
        km: (),
        opt: &mb_opt_t,
        l2b: &l2b_t,
        n_hit: &mut [i32; 2],
        hit: &mut [Vec<mb_hit_t>; 2],
        pes: &[mb_pestat_t; 4],
        qlen: [i32; 2],
        qseq: [&str; 2],
    ) {
        let is_meth = ((opt.flag & MB_F_METH) != 0) as i32;
        let mut paux = mb_pairaux_t::default();
        mb_pair_hits(km, opt, l2b, *n_hit, hit, pes, &mut paux);
        let do_matesw = !(paux.n_pp > 0 && paux.score == paux.sub_sc);
        if do_matesw && opt.max_rescue > 0 {
            let sub_diff = (opt.a + opt.b).max(opt.q + opt.e);
            if mb_matesw(km, opt, l2b, n_hit, hit, pes, &paux, qlen, qseq, is_meth) > 0 {
                for r in 0..2usize {
                    for h in hit[r].iter_mut().take(n_hit[r] as usize) {
                        if h.rescued() == 0 {
                            h.n_sub = 0;
                            h.subsc = 0;
                            if let Some(p) = &mut h.p {
                                p.dp_max2 = 0;
                            }
                        }
                    }
                    mb_hit_sort(km, &mut n_hit[r], &mut hit[r]);
                    mb_set_parent(
                        km,
                        opt.mask_level,
                        opt.mask_len,
                        n_hit[r],
                        &mut hit[r],
                        sub_diff,
                        0,
                    );
                    mb_set_mapq(
                        km,
                        qlen[r],
                        n_hit[r],
                        &mut hit[r],
                        opt.min_chain_score,
                        opt.a,
                        mb_is_sr_mode(opt, qlen[r]),
                        opt.max_sr_len,
                    );
                }
                mb_pair_hits(km, opt, l2b, *n_hit, hit, pes, &mut paux);
            }
        }
        if paux.n_pp != 0 {
            let i0 = paux.i[0] as usize;
            let i1 = paux.i[1] as usize;
            let score_se = hit[0][i0].p.as_ref().map(|p| p.dp_max).unwrap_or(0)
                + hit[1][i1].p.as_ref().map(|p| p.dp_max).unwrap_or(0);
            if paux.score >= score_se - opt.pen_unpair * opt.a {
                let mut score2 = paux.sub_sc;
                mb_sync_high_cov(n_hit[0], &mut hit[0]);
                mb_sync_high_cov(n_hit[1], &mut hit[1]);
                let identity = (hit[0][i0].mlen + hit[1][i1].mlen) as f64
                    / (hit[0][i0].blen + hit[1][i1].blen).max(1) as f64;
                if (hit[0][i0].id != hit[0][i0].parent || hit[1][i1].id != hit[1][i1].parent)
                    && score2 < score_se - opt.pen_unpair * opt.a
                {
                    score2 = score_se - opt.pen_unpair * opt.a;
                }
                let frac_high =
                    hit[0][i0].frac_high() as f64 / 255.0 + hit[1][i1].frac_high() as f64 / 255.0;
                let mut mapq_pe = (6.02 * identity * identity * (paux.score - score2) as f64
                    / opt.a as f64
                    - 4.343 * ((paux.n_sub + 1) as f64).ln()
                    + 0.499) as i32;
                mapq_pe = (mapq_pe as f64 * (1.0 - 0.5 * frac_high) + 0.499) as i32;
                if mapq_pe > 60 {
                    mapq_pe = 60;
                }
                if mapq_pe == 0 && paux.score > score2 {
                    mapq_pe = 1;
                }
                for (r, &idx) in [i0, i1].iter().enumerate() {
                    if hit[r][idx].mapq < mapq_pe {
                        hit[r][idx].mapq =
                            (0.2 * hit[r][idx].mapq as f64 + 0.8 * mapq_pe as f64 + 0.499) as i32;
                    }
                    if hit[r][idx].id != hit[r][idx].parent {
                        let old_parent = hit[r][idx].parent;
                        let new_parent = hit[r][idx].id;
                        for p in hit[r].iter_mut().take(n_hit[r] as usize) {
                            if p.parent == old_parent {
                                p.parent = new_parent;
                            }
                        }
                        if old_parent >= 0 && (old_parent as usize) < hit[r].len() {
                            hit[r][old_parent as usize].mapq = 0;
                        }
                    }
                    let q = hit[r][idx].clone();
                    for i in 0..n_hit[r] as usize {
                        if i == idx || hit[r][i].id != hit[r][i].parent {
                            continue;
                        }
                        let ol = if hit[r][i].qe <= q.qs || hit[r][i].qs >= q.qe {
                            0
                        } else {
                            hit[r][i].qe.min(q.qe) - hit[r][i].qs.max(q.qs)
                        };
                        if ol as f32 > opt.mask_level * (hit[r][i].qe - hit[r][i].qs) as f32 {
                            let old_id = hit[r][i].id;
                            for j in 0..n_hit[r] as usize {
                                if hit[r][j].parent == old_id {
                                    hit[r][j].parent = q.id;
                                }
                            }
                            hit[r][i].mapq = 0;
                        }
                    }
                }
            }
        }
        mb_set_sam_pri(
            n_hit[0],
            &mut hit[0],
            ((opt.flag & MB_F_PRIMARY5) != 0) as i32,
        );
        mb_set_sam_pri(
            n_hit[1],
            &mut hit[1],
            ((opt.flag & MB_F_PRIMARY5) != 0) as i32,
        );
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn insert_dir_matches_orientation_and_distance() {
            let h0 = mb_hit_t {
                ts: 100,
                te: 150,
                ..Default::default()
            };
            let h1 = mb_hit_t {
                ts: 300,
                te: 350,
                flags: mb_hit_t::flags_with(1, 0, 0, 0, 0, 0, 0, 0, 0),
                ..Default::default()
            };
            let mut dist = 0;
            assert_eq!(mb_insert_dir(&h0, &h1, &mut dist), 1);
            assert_eq!(dist, 250);
            assert_eq!(mb_insert_dir(&h1, &h0, &mut dist), 1);
            assert_eq!(dist, 250);
        }

        #[test]
        fn select_unique_se_requires_one_primary_with_mapq() {
            let hits = vec![
                mb_hit_t {
                    id: 0,
                    parent: 0,
                    mapq: 9,
                    ..Default::default()
                },
                mb_hit_t {
                    id: 1,
                    parent: 0,
                    mapq: 60,
                    ..Default::default()
                },
            ];
            assert!(mb_select_unique_se(hits.len() as i32, &hits).is_none());
            let hits = vec![mb_hit_t {
                id: 0,
                parent: 0,
                mapq: 10,
                ..Default::default()
            }];
            assert_eq!(mb_select_unique_se(1, &hits).unwrap().id, 0);
        }

        #[test]
        fn pair_score_uses_proper_orientation_distribution() {
            let h0 = mb_hit_t {
                ts: 100,
                te: 150,
                p: Some(
                    mb_extra_t {
                        dp_max: 80,
                        ..Default::default()
                    }
                    .boxed(),
                ),
                ..Default::default()
            };
            let h1 = mb_hit_t {
                ts: 300,
                te: 350,
                flags: mb_hit_t::flags_with(1, 0, 0, 0, 0, 0, 0, 0, 0),
                p: Some(
                    mb_extra_t {
                        dp_max: 70,
                        ..Default::default()
                    }
                    .boxed(),
                ),
                ..Default::default()
            };
            let mut pes = [mb_pestat_t {
                failed: 1,
                ..Default::default()
            }; 4];
            pes[1] = mb_pestat_t {
                lo: 200,
                hi: 300,
                failed: 0,
                avg: 250.0,
                std: 25.0,
            };
            let s = mb_pair_score(&h0, &h1, &pes, 2);
            assert!(s > 140.0 && s < 151.0);
            pes[1].failed = 1;
            assert_eq!(mb_pair_score(&h0, &h1, &pes, 2), -1.0);
        }

        #[test]
        fn hit_sum_score_merges_overlapping_primary_query_ranges() {
            let hits = vec![
                mb_hit_t {
                    id: 0,
                    parent: 0,
                    qs: 0,
                    qe: 100,
                    p: Some(
                        mb_extra_t {
                            dp_max: 100,
                            ..Default::default()
                        }
                        .boxed(),
                    ),
                    ..Default::default()
                },
                mb_hit_t {
                    id: 1,
                    parent: 1,
                    qs: 50,
                    qe: 150,
                    p: Some(
                        mb_extra_t {
                            dp_max: 80,
                            ..Default::default()
                        }
                        .boxed(),
                    ),
                    ..Default::default()
                },
                mb_hit_t {
                    id: 2,
                    parent: 0,
                    qs: 0,
                    qe: 150,
                    p: Some(
                        mb_extra_t {
                            dp_max: 1000,
                            ..Default::default()
                        }
                        .boxed(),
                    ),
                    ..Default::default()
                },
            ];
            assert_eq!(mb_hit_sum_score((), hits.len() as i32, &hits), 140);
        }

        #[test]
        fn pestat_estimates_fr_distribution_from_unique_pairs() {
            let mut opt = mb_opt_t::default();
            crate::options::mb_opt_init(&mut opt);
            opt.max_pe_ins = 1000;
            let mut seg_off = Vec::new();
            let mut seg_cnt = Vec::new();
            let mut n_hit = Vec::new();
            let mut hit = Vec::new();
            for i in 0..24usize {
                seg_off.push((i * 2) as i32);
                seg_cnt.push(2);
                n_hit.push(1);
                hit.push(vec![mb_hit_t {
                    tid: 0,
                    ts: (1000 + i as i64 * 10),
                    te: (1050 + i as i64 * 10),
                    id: 0,
                    parent: 0,
                    mapq: 60,
                    ..Default::default()
                }]);
                n_hit.push(1);
                hit.push(vec![mb_hit_t {
                    tid: 0,
                    ts: (1200 + i as i64 * 10),
                    te: (1250 + i as i64 * 10),
                    id: 0,
                    parent: 0,
                    mapq: 60,
                    flags: mb_hit_t::flags_with(1, 0, 0, 0, 0, 0, 0, 0, 0),
                    ..Default::default()
                }]);
            }
            let mut pes = [mb_pestat_t::default(); 4];
            mb_pestat((), &opt, 24, &seg_off, &seg_cnt, &n_hit, &hit, &mut pes);
            assert_eq!(pes[1].failed, 0);
            assert_eq!(pes[1].avg, 250.0);
            assert!(pes[1].lo <= 250 && pes[1].hi >= 250);
            assert_eq!(pes[0].failed, 1);
        }

        #[test]
        fn pair_hits_finds_best_proper_pair_and_marks_hits() {
            let l2b = l2b_t {
                n_ctg: 1,
                ctg: vec![crate::l2bit::l2b_ctg_t {
                    name: "ctg".into(),
                    len: 2000,
                    off: 0,
                    comm: None,
                }],
                ..Default::default()
            };
            let mut opt = mb_opt_t::default();
            crate::options::mb_opt_init(&mut opt);
            let mut hit = [
                vec![mb_hit_t {
                    tid: 0,
                    ts: 100,
                    te: 150,
                    id: 0,
                    parent: 0,
                    hash: 1,
                    p: Some(
                        mb_extra_t {
                            dp_max: 80,
                            ..Default::default()
                        }
                        .boxed(),
                    ),
                    ..Default::default()
                }],
                vec![mb_hit_t {
                    tid: 0,
                    ts: 300,
                    te: 350,
                    flags: mb_hit_t::flags_with(1, 0, 0, 0, 0, 0, 0, 0, 0),
                    id: 0,
                    parent: 0,
                    hash: 2,
                    p: Some(
                        mb_extra_t {
                            dp_max: 70,
                            ..Default::default()
                        }
                        .boxed(),
                    ),
                    ..Default::default()
                }],
            ];
            let mut pes = [mb_pestat_t {
                failed: 1,
                ..Default::default()
            }; 4];
            pes[1] = mb_pestat_t {
                lo: 200,
                hi: 300,
                failed: 0,
                avg: 250.0,
                std: 25.0,
            };
            let mut ret = mb_pairaux_t::default();
            mb_pair_hits((), &opt, &l2b, [1, 1], &mut hit, &pes, &mut ret);
            assert_eq!(ret.n_pp, 1);
            assert_eq!(ret.i, [0, 0]);
            assert!(ret.score > 140);
            assert_eq!((hit[0][0].proper_pair(), hit[1][0].proper_pair()), (1, 1));
        }

        #[test]
        fn ungap_detects_single_diagonal_kmer_support() {
            let qseq = [0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3];
            let tseq = [4, 4, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 4];
            let mut max_i = 0;
            let mut n_good = 0;
            let mut n_kmer = 0;
            let max = mb_ungap(
                (),
                qseq.len() as i32,
                &qseq,
                tseq.len() as i32,
                &tseq,
                3,
                &mut max_i,
                &mut n_good,
                &mut n_kmer,
            );
            assert!(max > 0);
            assert_eq!(max_i, 2);
            assert!(n_good >= 1);
            assert!(n_kmer > 0);
        }
    }
}

pub mod s2n_lite {
    #![allow(
        unused_variables,
        dead_code,
        non_snake_case,
        non_camel_case_types,
        unreachable_code
    )]

    // SIMD note: the KSW byte-state DP and exact-max scans above use this
    // native-backed shim for parity with the original packed kernels.
    pub type __m128i = [u8; 16];

    /// Original C static function `_mm_load_si128` from `minibwa/s2n-lite.h:8`.
    #[inline(always)]
    pub fn _mm_load_si128(ptr: &__m128i) -> __m128i {
        *ptr
    }

    /// Original C static function `_mm_loadu_si128` from `minibwa/s2n-lite.h:9`.
    #[inline(always)]
    pub fn _mm_loadu_si128(ptr: &__m128i) -> __m128i {
        *ptr
    }

    /// Original C static function `_mm_store_si128` from `minibwa/s2n-lite.h:10`.
    #[inline(always)]
    pub fn _mm_store_si128(ptr: &mut __m128i, a: __m128i) {
        *ptr = a;
    }

    /// Original C static function `_mm_storeu_si128` from `minibwa/s2n-lite.h:11`.
    #[inline(always)]
    pub fn _mm_storeu_si128(ptr: &mut __m128i, a: __m128i) {
        *ptr = a;
    }

    /// Original C static function `_mm_setzero_si128` from `minibwa/s2n-lite.h:12`.
    #[inline(always)]
    pub fn _mm_setzero_si128() -> __m128i {
        #[cfg(target_arch = "x86")]
        unsafe {
            return std::mem::transmute(std::arch::x86::_mm_setzero_si128());
        }
        #[cfg(target_arch = "x86_64")]
        unsafe {
            return std::mem::transmute(std::arch::x86_64::_mm_setzero_si128());
        }
        [0; 16]
    }

    /// Original C static function `_mm_or_si128` from `minibwa/s2n-lite.h:13`.
    #[inline(always)]
    pub fn _mm_or_si128(a: __m128i, b: __m128i) -> __m128i {
        #[cfg(target_arch = "x86")]
        unsafe {
            return std::mem::transmute(std::arch::x86::_mm_or_si128(
                std::mem::transmute(a),
                std::mem::transmute(b),
            ));
        }
        #[cfg(target_arch = "x86_64")]
        unsafe {
            return std::mem::transmute(std::arch::x86_64::_mm_or_si128(
                std::mem::transmute(a),
                std::mem::transmute(b),
            ));
        }
        let mut r = [0; 16];
        for i in 0..16 {
            r[i] = a[i] | b[i];
        }
        r
    }

    /// Original C static function `_mm_and_si128` from `minibwa/s2n-lite.h:14`.
    #[inline(always)]
    pub fn _mm_and_si128(a: __m128i, b: __m128i) -> __m128i {
        #[cfg(target_arch = "x86")]
        unsafe {
            return std::mem::transmute(std::arch::x86::_mm_and_si128(
                std::mem::transmute(a),
                std::mem::transmute(b),
            ));
        }
        #[cfg(target_arch = "x86_64")]
        unsafe {
            return std::mem::transmute(std::arch::x86_64::_mm_and_si128(
                std::mem::transmute(a),
                std::mem::transmute(b),
            ));
        }
        let mut r = [0; 16];
        for i in 0..16 {
            r[i] = a[i] & b[i];
        }
        r
    }

    /// Original C static function `_mm_andnot_si128` from `minibwa/s2n-lite.h:15`.
    #[inline(always)]
    pub fn _mm_andnot_si128(a: __m128i, b: __m128i) -> __m128i {
        #[cfg(target_arch = "x86")]
        unsafe {
            return std::mem::transmute(std::arch::x86::_mm_andnot_si128(
                std::mem::transmute(a),
                std::mem::transmute(b),
            ));
        }
        #[cfg(target_arch = "x86_64")]
        unsafe {
            return std::mem::transmute(std::arch::x86_64::_mm_andnot_si128(
                std::mem::transmute(a),
                std::mem::transmute(b),
            ));
        }
        let mut r = [0; 16];
        for i in 0..16 {
            r[i] = (!a[i]) & b[i];
        }
        r
    }

    /// Original C static function `_mm_blendv_epi8` from `minibwa/s2n-lite.h:20`.
    #[inline(always)]
    pub fn _mm_blendv_epi8(a: __m128i, b: __m128i, mask: __m128i) -> __m128i {
        #[cfg(all(target_arch = "x86", target_feature = "sse4.1"))]
        unsafe {
            return std::mem::transmute(std::arch::x86::_mm_blendv_epi8(
                std::mem::transmute(a),
                std::mem::transmute(b),
                std::mem::transmute(mask),
            ));
        }
        #[cfg(all(target_arch = "x86_64", target_feature = "sse4.1"))]
        unsafe {
            return std::mem::transmute(std::arch::x86_64::_mm_blendv_epi8(
                std::mem::transmute(a),
                std::mem::transmute(b),
                std::mem::transmute(mask),
            ));
        }
        #[cfg(target_arch = "x86")]
        unsafe {
            let a_v = std::mem::transmute(a);
            let b_v = std::mem::transmute(b);
            let mask_v = std::mem::transmute(mask);
            let sign = std::arch::x86::_mm_cmpgt_epi8(std::arch::x86::_mm_setzero_si128(), mask_v);
            return std::mem::transmute(std::arch::x86::_mm_or_si128(
                std::arch::x86::_mm_and_si128(sign, b_v),
                std::arch::x86::_mm_andnot_si128(sign, a_v),
            ));
        }
        #[cfg(target_arch = "x86_64")]
        unsafe {
            let a_v = std::mem::transmute(a);
            let b_v = std::mem::transmute(b);
            let mask_v = std::mem::transmute(mask);
            let sign =
                std::arch::x86_64::_mm_cmpgt_epi8(std::arch::x86_64::_mm_setzero_si128(), mask_v);
            return std::mem::transmute(std::arch::x86_64::_mm_or_si128(
                std::arch::x86_64::_mm_and_si128(sign, b_v),
                std::arch::x86_64::_mm_andnot_si128(sign, a_v),
            ));
        }
        let mut r = [0; 16];
        for i in 0..16 {
            r[i] = if (mask[i] & 0x80) != 0 { b[i] } else { a[i] };
        }
        r
    }

    /// Original C macro `_mm_slli_si128` from `minibwa/s2n-lite.h:17`.
    #[inline(always)]
    pub fn _mm_slli_si128<const IMM8: i32>(a: __m128i) -> __m128i {
        #[cfg(target_arch = "x86")]
        unsafe {
            return std::mem::transmute(std::arch::x86::_mm_slli_si128::<IMM8>(
                std::mem::transmute(a),
            ));
        }
        #[cfg(target_arch = "x86_64")]
        unsafe {
            return std::mem::transmute(std::arch::x86_64::_mm_slli_si128::<IMM8>(
                std::mem::transmute(a),
            ));
        }
        let mut r = [0; 16];
        let shift = IMM8.clamp(0, 16) as usize;
        if shift < 16 {
            r[shift..].copy_from_slice(&a[..16 - shift]);
        }
        r
    }

    /// Original C macro `_mm_srli_si128` from `minibwa/s2n-lite.h:18`.
    #[inline(always)]
    pub fn _mm_srli_si128<const IMM8: i32>(a: __m128i) -> __m128i {
        #[cfg(target_arch = "x86")]
        unsafe {
            return std::mem::transmute(std::arch::x86::_mm_srli_si128::<IMM8>(
                std::mem::transmute(a),
            ));
        }
        #[cfg(target_arch = "x86_64")]
        unsafe {
            return std::mem::transmute(std::arch::x86_64::_mm_srli_si128::<IMM8>(
                std::mem::transmute(a),
            ));
        }
        let mut r = [0; 16];
        let shift = IMM8.clamp(0, 16) as usize;
        if shift < 16 {
            r[..16 - shift].copy_from_slice(&a[shift..]);
        }
        r
    }

    /// Original C static function `_mm_set1_epi8` from `minibwa/s2n-lite.h:22`.
    #[inline(always)]
    pub fn _mm_set1_epi8(a: i32) -> __m128i {
        #[cfg(target_arch = "x86")]
        unsafe {
            return std::mem::transmute(std::arch::x86::_mm_set1_epi8(a as i8));
        }
        #[cfg(target_arch = "x86_64")]
        unsafe {
            return std::mem::transmute(std::arch::x86_64::_mm_set1_epi8(a as i8));
        }
        [a as u8; 16]
    }

    /// Original C static function `_mm_add_epi8` from `minibwa/s2n-lite.h:23`.
    #[inline(always)]
    pub fn _mm_add_epi8(a: __m128i, b: __m128i) -> __m128i {
        #[cfg(target_arch = "x86")]
        unsafe {
            return std::mem::transmute(std::arch::x86::_mm_add_epi8(
                std::mem::transmute(a),
                std::mem::transmute(b),
            ));
        }
        #[cfg(target_arch = "x86_64")]
        unsafe {
            return std::mem::transmute(std::arch::x86_64::_mm_add_epi8(
                std::mem::transmute(a),
                std::mem::transmute(b),
            ));
        }
        let mut r = [0; 16];
        for i in 0..16 {
            r[i] = a[i].wrapping_add(b[i]);
        }
        r
    }

    /// Original C static function `_mm_adds_epu8` from `minibwa/s2n-lite.h:24`.
    #[inline(always)]
    pub fn _mm_adds_epu8(a: __m128i, b: __m128i) -> __m128i {
        #[cfg(target_arch = "x86")]
        unsafe {
            return std::mem::transmute(std::arch::x86::_mm_adds_epu8(
                std::mem::transmute(a),
                std::mem::transmute(b),
            ));
        }
        #[cfg(target_arch = "x86_64")]
        unsafe {
            return std::mem::transmute(std::arch::x86_64::_mm_adds_epu8(
                std::mem::transmute(a),
                std::mem::transmute(b),
            ));
        }
        let mut r = [0; 16];
        for i in 0..16 {
            r[i] = a[i].saturating_add(b[i]);
        }
        r
    }

    /// Original C static function `_mm_sub_epi8` from `minibwa/s2n-lite.h:25`.
    #[inline(always)]
    pub fn _mm_sub_epi8(a: __m128i, b: __m128i) -> __m128i {
        #[cfg(target_arch = "x86")]
        unsafe {
            return std::mem::transmute(std::arch::x86::_mm_sub_epi8(
                std::mem::transmute(a),
                std::mem::transmute(b),
            ));
        }
        #[cfg(target_arch = "x86_64")]
        unsafe {
            return std::mem::transmute(std::arch::x86_64::_mm_sub_epi8(
                std::mem::transmute(a),
                std::mem::transmute(b),
            ));
        }
        let mut r = [0; 16];
        for i in 0..16 {
            r[i] = a[i].wrapping_sub(b[i]);
        }
        r
    }

    /// Original C static function `_mm_subs_epu8` from `minibwa/s2n-lite.h:26`.
    #[inline(always)]
    pub fn _mm_subs_epu8(a: __m128i, b: __m128i) -> __m128i {
        #[cfg(target_arch = "x86")]
        unsafe {
            return std::mem::transmute(std::arch::x86::_mm_subs_epu8(
                std::mem::transmute(a),
                std::mem::transmute(b),
            ));
        }
        #[cfg(target_arch = "x86_64")]
        unsafe {
            return std::mem::transmute(std::arch::x86_64::_mm_subs_epu8(
                std::mem::transmute(a),
                std::mem::transmute(b),
            ));
        }
        let mut r = [0; 16];
        for i in 0..16 {
            r[i] = a[i].saturating_sub(b[i]);
        }
        r
    }

    /// Original C static function `_mm_cmpeq_epi8` from `minibwa/s2n-lite.h:27`.
    #[inline(always)]
    pub fn _mm_cmpeq_epi8(a: __m128i, b: __m128i) -> __m128i {
        #[cfg(target_arch = "x86")]
        unsafe {
            return std::mem::transmute(std::arch::x86::_mm_cmpeq_epi8(
                std::mem::transmute(a),
                std::mem::transmute(b),
            ));
        }
        #[cfg(target_arch = "x86_64")]
        unsafe {
            return std::mem::transmute(std::arch::x86_64::_mm_cmpeq_epi8(
                std::mem::transmute(a),
                std::mem::transmute(b),
            ));
        }
        let mut r = [0; 16];
        for i in 0..16 {
            r[i] = if a[i] == b[i] { 0xff } else { 0 };
        }
        r
    }

    /// Original C static function `_mm_cmpgt_epi8` from `minibwa/s2n-lite.h:28`.
    #[inline(always)]
    pub fn _mm_cmpgt_epi8(a: __m128i, b: __m128i) -> __m128i {
        #[cfg(target_arch = "x86")]
        unsafe {
            return std::mem::transmute(std::arch::x86::_mm_cmpgt_epi8(
                std::mem::transmute(a),
                std::mem::transmute(b),
            ));
        }
        #[cfg(target_arch = "x86_64")]
        unsafe {
            return std::mem::transmute(std::arch::x86_64::_mm_cmpgt_epi8(
                std::mem::transmute(a),
                std::mem::transmute(b),
            ));
        }
        let mut r = [0; 16];
        for i in 0..16 {
            r[i] = if (a[i] as i8) > (b[i] as i8) { 0xff } else { 0 };
        }
        r
    }

    /// Original SSE intrinsic `_mm_cmplt_epi8` used in `minibwa/ksw2_extd2_sse.c`.
    #[inline(always)]
    pub fn _mm_cmplt_epi8(a: __m128i, b: __m128i) -> __m128i {
        #[cfg(target_arch = "x86")]
        unsafe {
            return std::mem::transmute(std::arch::x86::_mm_cmpgt_epi8(
                std::mem::transmute(b),
                std::mem::transmute(a),
            ));
        }
        #[cfg(target_arch = "x86_64")]
        unsafe {
            return std::mem::transmute(std::arch::x86_64::_mm_cmpgt_epi8(
                std::mem::transmute(b),
                std::mem::transmute(a),
            ));
        }
        let mut r = [0; 16];
        for i in 0..16 {
            r[i] = if (a[i] as i8) < (b[i] as i8) { 0xff } else { 0 };
        }
        r
    }

    /// Original C static function `_mm_max_epi8` from `minibwa/s2n-lite.h:29`.
    #[inline(always)]
    pub fn _mm_max_epi8(a: __m128i, b: __m128i) -> __m128i {
        #[cfg(all(target_arch = "x86", target_feature = "sse4.1"))]
        unsafe {
            return std::mem::transmute(std::arch::x86::_mm_max_epi8(
                std::mem::transmute(a),
                std::mem::transmute(b),
            ));
        }
        #[cfg(all(target_arch = "x86_64", target_feature = "sse4.1"))]
        unsafe {
            return std::mem::transmute(std::arch::x86_64::_mm_max_epi8(
                std::mem::transmute(a),
                std::mem::transmute(b),
            ));
        }
        #[cfg(target_arch = "x86")]
        unsafe {
            let av = std::mem::transmute(a);
            let bv = std::mem::transmute(b);
            let mask = std::arch::x86::_mm_cmpgt_epi8(av, bv);
            return std::mem::transmute(std::arch::x86::_mm_or_si128(
                std::arch::x86::_mm_and_si128(mask, av),
                std::arch::x86::_mm_andnot_si128(mask, bv),
            ));
        }
        #[cfg(target_arch = "x86_64")]
        unsafe {
            let av = std::mem::transmute(a);
            let bv = std::mem::transmute(b);
            let mask = std::arch::x86_64::_mm_cmpgt_epi8(av, bv);
            return std::mem::transmute(std::arch::x86_64::_mm_or_si128(
                std::arch::x86_64::_mm_and_si128(mask, av),
                std::arch::x86_64::_mm_andnot_si128(mask, bv),
            ));
        }
        let mut r = [0; 16];
        for i in 0..16 {
            r[i] = (a[i] as i8).max(b[i] as i8) as u8;
        }
        r
    }

    /// Original C static function `_mm_min_epi8` from `minibwa/s2n-lite.h:30`.
    #[inline(always)]
    pub fn _mm_min_epi8(a: __m128i, b: __m128i) -> __m128i {
        #[cfg(all(target_arch = "x86", target_feature = "sse4.1"))]
        unsafe {
            return std::mem::transmute(std::arch::x86::_mm_min_epi8(
                std::mem::transmute(a),
                std::mem::transmute(b),
            ));
        }
        #[cfg(all(target_arch = "x86_64", target_feature = "sse4.1"))]
        unsafe {
            return std::mem::transmute(std::arch::x86_64::_mm_min_epi8(
                std::mem::transmute(a),
                std::mem::transmute(b),
            ));
        }
        #[cfg(target_arch = "x86")]
        unsafe {
            let av = std::mem::transmute(a);
            let bv = std::mem::transmute(b);
            let mask = std::arch::x86::_mm_cmpgt_epi8(av, bv);
            return std::mem::transmute(std::arch::x86::_mm_or_si128(
                std::arch::x86::_mm_and_si128(mask, bv),
                std::arch::x86::_mm_andnot_si128(mask, av),
            ));
        }
        #[cfg(target_arch = "x86_64")]
        unsafe {
            let av = std::mem::transmute(a);
            let bv = std::mem::transmute(b);
            let mask = std::arch::x86_64::_mm_cmpgt_epi8(av, bv);
            return std::mem::transmute(std::arch::x86_64::_mm_or_si128(
                std::arch::x86_64::_mm_and_si128(mask, bv),
                std::arch::x86_64::_mm_andnot_si128(mask, av),
            ));
        }
        let mut r = [0; 16];
        for i in 0..16 {
            r[i] = (a[i] as i8).min(b[i] as i8) as u8;
        }
        r
    }

    /// Original C static function `_mm_max_epu8` from `minibwa/s2n-lite.h:31`.
    #[inline(always)]
    pub fn _mm_max_epu8(a: __m128i, b: __m128i) -> __m128i {
        #[cfg(target_arch = "x86")]
        unsafe {
            return std::mem::transmute(std::arch::x86::_mm_max_epu8(
                std::mem::transmute(a),
                std::mem::transmute(b),
            ));
        }
        #[cfg(target_arch = "x86_64")]
        unsafe {
            return std::mem::transmute(std::arch::x86_64::_mm_max_epu8(
                std::mem::transmute(a),
                std::mem::transmute(b),
            ));
        }
        let mut r = [0; 16];
        for i in 0..16 {
            r[i] = a[i].max(b[i]);
        }
        r
    }

    /// Original C static function `_mm_min_epu8` from `minibwa/s2n-lite.h:32`.
    #[inline(always)]
    pub fn _mm_min_epu8(a: __m128i, b: __m128i) -> __m128i {
        #[cfg(target_arch = "x86")]
        unsafe {
            return std::mem::transmute(std::arch::x86::_mm_min_epu8(
                std::mem::transmute(a),
                std::mem::transmute(b),
            ));
        }
        #[cfg(target_arch = "x86_64")]
        unsafe {
            return std::mem::transmute(std::arch::x86_64::_mm_min_epu8(
                std::mem::transmute(a),
                std::mem::transmute(b),
            ));
        }
        let mut r = [0; 16];
        for i in 0..16 {
            r[i] = a[i].min(b[i]);
        }
        r
    }

    /// Original C static function `_mm_set1_epi16` from `minibwa/s2n-lite.h:34`.
    #[inline(always)]
    pub fn _mm_set1_epi16(a: i32) -> __m128i {
        #[cfg(target_arch = "x86")]
        unsafe {
            return std::mem::transmute(std::arch::x86::_mm_set1_epi16(a as i16));
        }
        #[cfg(target_arch = "x86_64")]
        unsafe {
            return std::mem::transmute(std::arch::x86_64::_mm_set1_epi16(a as i16));
        }
        let mut r = [0; 16];
        for lane in 0..8 {
            r[lane * 2..lane * 2 + 2].copy_from_slice(&(a as i16).to_le_bytes());
        }
        r
    }

    /// Original C static function `_mm_cmpgt_epi16` from `minibwa/s2n-lite.h:35`.
    #[inline(always)]
    pub fn _mm_cmpgt_epi16(a: __m128i, b: __m128i) -> __m128i {
        #[cfg(target_arch = "x86")]
        unsafe {
            return std::mem::transmute(std::arch::x86::_mm_cmpgt_epi16(
                std::mem::transmute(a),
                std::mem::transmute(b),
            ));
        }
        #[cfg(target_arch = "x86_64")]
        unsafe {
            return std::mem::transmute(std::arch::x86_64::_mm_cmpgt_epi16(
                std::mem::transmute(a),
                std::mem::transmute(b),
            ));
        }
        let mut r = [0; 16];
        for lane in 0..8 {
            let i = lane * 2;
            let av = i16::from_le_bytes([a[i], a[i + 1]]);
            let bv = i16::from_le_bytes([b[i], b[i + 1]]);
            let bytes = (if av > bv { u16::MAX } else { 0 }).to_le_bytes();
            r[i..i + 2].copy_from_slice(&bytes);
        }
        r
    }

    /// Original C static function `_mm_max_epi16` from `minibwa/s2n-lite.h:36`.
    #[inline(always)]
    pub fn _mm_max_epi16(a: __m128i, b: __m128i) -> __m128i {
        #[cfg(target_arch = "x86")]
        unsafe {
            return std::mem::transmute(std::arch::x86::_mm_max_epi16(
                std::mem::transmute(a),
                std::mem::transmute(b),
            ));
        }
        #[cfg(target_arch = "x86_64")]
        unsafe {
            return std::mem::transmute(std::arch::x86_64::_mm_max_epi16(
                std::mem::transmute(a),
                std::mem::transmute(b),
            ));
        }
        let mut r = [0; 16];
        for lane in 0..8 {
            let i = lane * 2;
            let av = i16::from_le_bytes([a[i], a[i + 1]]);
            let bv = i16::from_le_bytes([b[i], b[i + 1]]);
            r[i..i + 2].copy_from_slice(&av.max(bv).to_le_bytes());
        }
        r
    }

    /// Original C static function `_mm_adds_epi16` from `minibwa/s2n-lite.h:37`.
    #[inline(always)]
    pub fn _mm_adds_epi16(a: __m128i, b: __m128i) -> __m128i {
        #[cfg(target_arch = "x86")]
        unsafe {
            return std::mem::transmute(std::arch::x86::_mm_adds_epi16(
                std::mem::transmute(a),
                std::mem::transmute(b),
            ));
        }
        #[cfg(target_arch = "x86_64")]
        unsafe {
            return std::mem::transmute(std::arch::x86_64::_mm_adds_epi16(
                std::mem::transmute(a),
                std::mem::transmute(b),
            ));
        }
        let mut r = [0; 16];
        for lane in 0..8 {
            let i = lane * 2;
            let av = i16::from_le_bytes([a[i], a[i + 1]]);
            let bv = i16::from_le_bytes([b[i], b[i + 1]]);
            r[i..i + 2].copy_from_slice(&av.saturating_add(bv).to_le_bytes());
        }
        r
    }

    /// Original C static function `_mm_subs_epi16` from `minibwa/s2n-lite.h:38`.
    #[inline(always)]
    pub fn _mm_subs_epi16(a: __m128i, b: __m128i) -> __m128i {
        #[cfg(target_arch = "x86")]
        unsafe {
            return std::mem::transmute(std::arch::x86::_mm_subs_epi16(
                std::mem::transmute(a),
                std::mem::transmute(b),
            ));
        }
        #[cfg(target_arch = "x86_64")]
        unsafe {
            return std::mem::transmute(std::arch::x86_64::_mm_subs_epi16(
                std::mem::transmute(a),
                std::mem::transmute(b),
            ));
        }
        let mut r = [0; 16];
        for lane in 0..8 {
            let i = lane * 2;
            let av = i16::from_le_bytes([a[i], a[i + 1]]);
            let bv = i16::from_le_bytes([b[i], b[i + 1]]);
            r[i..i + 2].copy_from_slice(&av.saturating_sub(bv).to_le_bytes());
        }
        r
    }

    /// Original C static function `_mm_subs_epu16` from `minibwa/s2n-lite.h:39`.
    #[inline(always)]
    pub fn _mm_subs_epu16(a: __m128i, b: __m128i) -> __m128i {
        #[cfg(target_arch = "x86")]
        unsafe {
            return std::mem::transmute(std::arch::x86::_mm_subs_epu16(
                std::mem::transmute(a),
                std::mem::transmute(b),
            ));
        }
        #[cfg(target_arch = "x86_64")]
        unsafe {
            return std::mem::transmute(std::arch::x86_64::_mm_subs_epu16(
                std::mem::transmute(a),
                std::mem::transmute(b),
            ));
        }
        let mut r = [0; 16];
        for lane in 0..8 {
            let i = lane * 2;
            let av = u16::from_le_bytes([a[i], a[i + 1]]);
            let bv = u16::from_le_bytes([b[i], b[i + 1]]);
            r[i..i + 2].copy_from_slice(&av.saturating_sub(bv).to_le_bytes());
        }
        r
    }

    /// Original C macro `_mm_extract_epi16` from `minibwa/s2n-lite.h:41`.
    #[inline(always)]
    pub fn _mm_extract_epi16<const IMM8: i32>(a: __m128i) -> i32 {
        #[cfg(target_arch = "x86")]
        unsafe {
            return std::arch::x86::_mm_extract_epi16::<IMM8>(std::mem::transmute(a));
        }
        #[cfg(target_arch = "x86_64")]
        unsafe {
            return std::arch::x86_64::_mm_extract_epi16::<IMM8>(std::mem::transmute(a));
        }
        let lane = (IMM8 & 7) as usize;
        i16::from_le_bytes([a[lane * 2], a[lane * 2 + 1]]) as i32
    }

    /// Original SSE intrinsic `_mm_movemask_epi8` used in `minibwa/ksw2_ll_sse.c`.
    #[inline(always)]
    pub fn _mm_movemask_epi8(a: __m128i) -> i32 {
        #[cfg(target_arch = "x86")]
        unsafe {
            return std::arch::x86::_mm_movemask_epi8(std::mem::transmute(a));
        }
        #[cfg(target_arch = "x86_64")]
        unsafe {
            return std::arch::x86_64::_mm_movemask_epi8(std::mem::transmute(a));
        }
        let mut r = 0;
        for i in 0..16 {
            r |= (((a[i] >> 7) & 1) as i32) << i;
        }
        r
    }

    /// Original C macro `_mm_insert_epi16` from `minibwa/s2n-lite.h:42`.
    #[inline(always)]
    pub fn _mm_insert_epi16<const IMM8: i32>(a: __m128i, b: i32) -> __m128i {
        #[cfg(target_arch = "x86")]
        unsafe {
            return std::mem::transmute(std::arch::x86::_mm_insert_epi16::<IMM8>(
                std::mem::transmute(a),
                b,
            ));
        }
        #[cfg(target_arch = "x86_64")]
        unsafe {
            return std::mem::transmute(std::arch::x86_64::_mm_insert_epi16::<IMM8>(
                std::mem::transmute(a),
                b,
            ));
        }
        let mut r = a;
        let lane = (IMM8 & 7) as usize;
        r[lane * 2..lane * 2 + 2].copy_from_slice(&(b as i16).to_le_bytes());
        r
    }

    /// Original C static function `_mm_set1_epi32` from `minibwa/s2n-lite.h:44`.
    #[inline(always)]
    pub fn _mm_set1_epi32(a: i32) -> __m128i {
        #[cfg(target_arch = "x86")]
        unsafe {
            return std::mem::transmute(std::arch::x86::_mm_set1_epi32(a));
        }
        #[cfg(target_arch = "x86_64")]
        unsafe {
            return std::mem::transmute(std::arch::x86_64::_mm_set1_epi32(a));
        }
        let mut r = [0; 16];
        for lane in 0..4 {
            r[lane * 4..lane * 4 + 4].copy_from_slice(&a.to_le_bytes());
        }
        r
    }

    /// Original C static function `_mm_cvtsi32_si128` from `minibwa/s2n-lite.h:45`.
    #[inline(always)]
    pub fn _mm_cvtsi32_si128(a: i32) -> __m128i {
        #[cfg(target_arch = "x86")]
        unsafe {
            return std::mem::transmute(std::arch::x86::_mm_cvtsi32_si128(a));
        }
        #[cfg(target_arch = "x86_64")]
        unsafe {
            return std::mem::transmute(std::arch::x86_64::_mm_cvtsi32_si128(a));
        }
        let mut r = [0; 16];
        r[..4].copy_from_slice(&a.to_le_bytes());
        r
    }

    /// Original C static function `_mm_setr_epi32` from `minibwa/s2n-lite.h:46`.
    #[inline(always)]
    pub fn _mm_setr_epi32(a: i32, b: i32, c: i32, d: i32) -> __m128i {
        #[cfg(target_arch = "x86")]
        unsafe {
            return std::mem::transmute(std::arch::x86::_mm_setr_epi32(a, b, c, d));
        }
        #[cfg(target_arch = "x86_64")]
        unsafe {
            return std::mem::transmute(std::arch::x86_64::_mm_setr_epi32(a, b, c, d));
        }
        let mut r = [0; 16];
        r[0..4].copy_from_slice(&a.to_le_bytes());
        r[4..8].copy_from_slice(&b.to_le_bytes());
        r[8..12].copy_from_slice(&c.to_le_bytes());
        r[12..16].copy_from_slice(&d.to_le_bytes());
        r
    }

    /// Original C static function `_mm_cmpgt_epi32` from `minibwa/s2n-lite.h:50`.
    #[inline(always)]
    pub fn _mm_cmpgt_epi32(a: __m128i, b: __m128i) -> __m128i {
        #[cfg(all(target_arch = "x86", target_feature = "sse4.1"))]
        unsafe {
            return std::mem::transmute(std::arch::x86::_mm_cmpgt_epi32(
                std::mem::transmute(a),
                std::mem::transmute(b),
            ));
        }
        #[cfg(all(target_arch = "x86_64", target_feature = "sse4.1"))]
        unsafe {
            return std::mem::transmute(std::arch::x86_64::_mm_cmpgt_epi32(
                std::mem::transmute(a),
                std::mem::transmute(b),
            ));
        }
        let mut r = [0; 16];
        for lane in 0..4 {
            let i = lane * 4;
            let av = i32::from_le_bytes([a[i], a[i + 1], a[i + 2], a[i + 3]]);
            let bv = i32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]);
            let bytes = (if av > bv { u32::MAX } else { 0 }).to_le_bytes();
            r[i..i + 4].copy_from_slice(&bytes);
        }
        r
    }

    /// Original C static function `_mm_max_epi32` from `minibwa/s2n-lite.h:51`.
    #[inline(always)]
    pub fn _mm_max_epi32(a: __m128i, b: __m128i) -> __m128i {
        #[cfg(all(target_arch = "x86", target_feature = "sse4.1"))]
        unsafe {
            return std::mem::transmute(std::arch::x86::_mm_max_epi32(
                std::mem::transmute(a),
                std::mem::transmute(b),
            ));
        }
        #[cfg(all(target_arch = "x86_64", target_feature = "sse4.1"))]
        unsafe {
            return std::mem::transmute(std::arch::x86_64::_mm_max_epi32(
                std::mem::transmute(a),
                std::mem::transmute(b),
            ));
        }
        let mut r = [0; 16];
        for lane in 0..4 {
            let i = lane * 4;
            let av = i32::from_le_bytes([a[i], a[i + 1], a[i + 2], a[i + 3]]);
            let bv = i32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]);
            r[i..i + 4].copy_from_slice(&av.max(bv).to_le_bytes());
        }
        r
    }

    /// Original C static function `_mm_add_epi32` from `minibwa/s2n-lite.h:52`.
    #[inline(always)]
    pub fn _mm_add_epi32(a: __m128i, b: __m128i) -> __m128i {
        #[cfg(target_arch = "x86")]
        unsafe {
            return std::mem::transmute(std::arch::x86::_mm_add_epi32(
                std::mem::transmute(a),
                std::mem::transmute(b),
            ));
        }
        #[cfg(target_arch = "x86_64")]
        unsafe {
            return std::mem::transmute(std::arch::x86_64::_mm_add_epi32(
                std::mem::transmute(a),
                std::mem::transmute(b),
            ));
        }
        let mut r = [0; 16];
        for lane in 0..4 {
            let i = lane * 4;
            let av = u32::from_le_bytes([a[i], a[i + 1], a[i + 2], a[i + 3]]);
            let bv = u32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]);
            r[i..i + 4].copy_from_slice(&av.wrapping_add(bv).to_le_bytes());
        }
        r
    }

    /// Original C static function `_mm_sub_epi32` from `minibwa/s2n-lite.h:53`.
    #[inline(always)]
    pub fn _mm_sub_epi32(a: __m128i, b: __m128i) -> __m128i {
        #[cfg(target_arch = "x86")]
        unsafe {
            return std::mem::transmute(std::arch::x86::_mm_sub_epi32(
                std::mem::transmute(a),
                std::mem::transmute(b),
            ));
        }
        #[cfg(target_arch = "x86_64")]
        unsafe {
            return std::mem::transmute(std::arch::x86_64::_mm_sub_epi32(
                std::mem::transmute(a),
                std::mem::transmute(b),
            ));
        }
        let mut r = [0; 16];
        for lane in 0..4 {
            let i = lane * 4;
            let av = u32::from_le_bytes([a[i], a[i + 1], a[i + 2], a[i + 3]]);
            let bv = u32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]);
            r[i..i + 4].copy_from_slice(&av.wrapping_sub(bv).to_le_bytes());
        }
        r
    }

    /// Original C macro `_mm_insert_epi32` from `minibwa/s2n-lite.h:55`.
    #[inline(always)]
    pub fn _mm_insert_epi32<const IMM8: i32>(a: __m128i, b: i32) -> __m128i {
        #[cfg(all(target_arch = "x86", target_feature = "sse4.1"))]
        unsafe {
            return std::mem::transmute(std::arch::x86::_mm_insert_epi32::<IMM8>(
                std::mem::transmute(a),
                b,
            ));
        }
        #[cfg(all(target_arch = "x86_64", target_feature = "sse4.1"))]
        unsafe {
            return std::mem::transmute(std::arch::x86_64::_mm_insert_epi32::<IMM8>(
                std::mem::transmute(a),
                b,
            ));
        }
        let mut r = a;
        let lane = (IMM8 & 3) as usize;
        r[lane * 4..lane * 4 + 4].copy_from_slice(&b.to_le_bytes());
        r
    }

    /// Original C macro `_mm_prefetch` from `minibwa/s2n-lite.h:57`.
    #[inline(always)]
    pub fn _mm_prefetch(ptr: *const u8, hint: i32) {
        let _ = hint;
        #[cfg(target_arch = "x86")]
        unsafe {
            std::arch::x86::_mm_prefetch::<{ std::arch::x86::_MM_HINT_T0 }>(ptr as *const i8);
        }
        #[cfg(target_arch = "x86_64")]
        unsafe {
            std::arch::x86_64::_mm_prefetch::<{ std::arch::x86_64::_MM_HINT_T0 }>(ptr as *const i8);
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            let _ = ptr;
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn byte_lanes_match_wrapping_saturating_and_masks() {
            let a = [250, 1, 128, 127, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
            let b = [10, 2, 127, 128, 9, 5, 7, 0, 1, 20, 10, 13, 13, 15, 14, 16];
            assert_eq!(_mm_add_epi8(a, b)[0], 4);
            assert_eq!(_mm_adds_epu8(a, b)[0], 255);
            assert_eq!(_mm_sub_epi8(b, a)[1], 1);
            assert_eq!(_mm_subs_epu8(b, a)[0], 0);
            assert_eq!(_mm_cmpeq_epi8(a, b)[6], 0xff);
            assert_eq!(_mm_cmpgt_epi8(a, b)[2], 0);
            assert_eq!(_mm_cmpgt_epi8(a, b)[3], 0xff);
            assert_eq!(_mm_cmplt_epi8(a, b)[1], 0xff);
            assert_eq!(_mm_movemask_epi8(_mm_cmpgt_epi8(a, b)) & (1 << 3), 1 << 3);
            assert_eq!(_mm_max_epi8(a, b)[2], 127);
            assert_eq!(_mm_max_epi8(a, b)[3], 127);
            assert_eq!(_mm_max_epu8(a, b)[0], 250);
            assert_eq!(_mm_min_epi8(a, b)[2], 128);
            assert_eq!(_mm_blendv_epi8(a, b, _mm_cmpgt_epi8(a, b))[3], 128);
        }

        #[test]
        fn word_and_dword_lanes_use_little_endian_vector_layout() {
            let shifted = _mm_slli_si128::<3>(_mm_setr_epi32(0x04030201, 0, 0, 0));
            assert_eq!(&shifted[..5], &[0, 0, 0, 1, 2]);
            assert_eq!(
                &_mm_srli_si128::<2>(_mm_setr_epi32(0x04030201, 0, 0, 0))[..3],
                &[3, 4, 0]
            );
            let a16 = _mm_set1_epi16(32_760);
            let b16 = _mm_set1_epi16(100);
            assert_eq!(
                i16::from_le_bytes([_mm_adds_epi16(a16, b16)[0], _mm_adds_epi16(a16, b16)[1]]),
                i16::MAX
            );
            assert_eq!(
                u16::from_le_bytes([
                    _mm_subs_epu16(_mm_set1_epi16(3), _mm_set1_epi16(10))[0],
                    _mm_subs_epu16(_mm_set1_epi16(3), _mm_set1_epi16(10))[1]
                ]),
                0
            );
            assert_eq!(
                i16::from_le_bytes([
                    _mm_max_epi16(_mm_set1_epi16(-5), _mm_set1_epi16(7))[0],
                    _mm_max_epi16(_mm_set1_epi16(-5), _mm_set1_epi16(7))[1]
                ]),
                7
            );
            let inserted16 = _mm_insert_epi16::<3>(_mm_setzero_si128(), -123);
            assert_eq!(_mm_extract_epi16::<3>(inserted16) as i16, -123);
            let a32 = _mm_setr_epi32(1, -5, i32::MAX, 10);
            let b32 = _mm_setr_epi32(2, -7, 1, 20);
            assert_eq!(_mm_cmpgt_epi32(a32, b32)[4..8], [0xff; 4]);
            assert_eq!(
                i32::from_le_bytes([
                    _mm_max_epi32(a32, b32)[8],
                    _mm_max_epi32(a32, b32)[9],
                    _mm_max_epi32(a32, b32)[10],
                    _mm_max_epi32(a32, b32)[11]
                ]),
                i32::MAX
            );
            assert_eq!(
                u32::from_le_bytes([
                    _mm_add_epi32(_mm_cvtsi32_si128(-1), _mm_cvtsi32_si128(2))[0],
                    _mm_add_epi32(_mm_cvtsi32_si128(-1), _mm_cvtsi32_si128(2))[1],
                    _mm_add_epi32(_mm_cvtsi32_si128(-1), _mm_cvtsi32_si128(2))[2],
                    _mm_add_epi32(_mm_cvtsi32_si128(-1), _mm_cvtsi32_si128(2))[3]
                ]),
                1
            );
            let inserted32 = _mm_insert_epi32::<2>(_mm_setzero_si128(), -77);
            assert_eq!(
                i32::from_le_bytes([inserted32[8], inserted32[9], inserted32[10], inserted32[11]]),
                -77
            );
            _mm_prefetch(inserted32.as_ptr(), 0);
        }
    }
}

pub mod seed {
    #![allow(unused_variables, dead_code, non_snake_case, non_camel_case_types)]

    use crate::bwt::{
        mb_bwt_sa_batch_with_scratch, mb_bwt_smem, mb_bwt_smem_batch_ref_with_queue, mb_bwt_t,
        mb_sai_t, mb_sai_v, mb_smem_entry_ref, tiny_queue_t,
    };
    use crate::l2bit::{l2b_intv2cid, l2b_intv2cid_meth, l2b_meth_t, l2b_t};
    use crate::lchain::mb_anchor_t;
    use crate::map_algo::mb_idx_t;

    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    pub struct mb_anchor_v {
        pub n: i64,
        pub m: i64,
        pub a: Vec<mb_anchor_t>,
    }

    /// Original C global function `mb_seed_intv` from `minibwa/seed.c:24`.
    pub fn mb_seed_intv(
        km: (),
        bwt: &mb_bwt_t,
        len: i32,
        seq: &[u8],
        min_len: i32,
        max_sub_occ: i32,
        v: &mut mb_sai_v,
    ) {
        let mut x = 0i64;
        let mut p = mb_sai_t::default();
        v.n = 0;
        v.a.clear();
        loop {
            x = mb_bwt_smem(bwt, len as u32, seq, x, min_len as i64, 1, &mut p);
            if p.size > 0 {
                v.a.push(p);
                v.n += 1;
            }
            if x >= len as i64 {
                break;
            }
        }

        let n_a0 = v.n;
        for i in 0..n_a0 {
            let st = (v.a[i].info >> 32) as u32;
            let en = v.a[i].info as u32;
            if en - st < (min_len * 2) as u32 || v.a[i].size > max_sub_occ as u64 {
                continue;
            }
            x = st as i64;
            let sub_min_len = (((en - st) / 2) as i32).max(min_len);
            loop {
                x = mb_bwt_smem(
                    bwt,
                    en,
                    seq,
                    x,
                    sub_min_len as i64,
                    v.a[i].size as i64 + 1,
                    &mut p,
                );
                if p.size > v.a[i].size {
                    v.a.push(p);
                    v.n += 1;
                }
                if x >= en as i64 {
                    break;
                }
            }
        }
        v.m = v.a.capacity();
    }

    /// Original C global function `mb_seed_intv_batch` from `minibwa/seed.c:56`.
    pub fn mb_seed_intv_batch(
        km: (),
        bwt: &mb_bwt_t,
        n_seq: i32,
        len: &[i32],
        seq: &[*const u8],
        min_len: i32,
        max_sub_occ: i32,
        v: &mut [mb_sai_v],
    ) {
        const MAX_BATCH_SIZE: usize = 50;
        for vi in v.iter_mut().take(n_seq as usize) {
            vi.n = 0;
            vi.a.clear();
        }
        let mut tq = tiny_queue_t::default();
        let mut s = Vec::with_capacity(MAX_BATCH_SIZE);
        let mut i = 0usize;
        while i < n_seq as usize {
            let en = (i + MAX_BATCH_SIZE).min(n_seq as usize);
            s.clear();
            let v_ptr = v.as_mut_ptr();
            for j in i..en {
                s.push(mb_smem_entry_ref {
                    min_len,
                    min_occ: 1,
                    st: 0,
                    en: len[j],
                    q: seq[j],
                    v: unsafe { v_ptr.add(j) },
                    stage: 0,
                    x: 0,
                    i: 0,
                    kmer: 0,
                    p: mb_sai_t::default(),
                });
            }
            mb_bwt_smem_batch_ref_with_queue(km, bwt, (en - i) as i32, &mut s, v, &mut tq);
            i = en;
        }

        let nv: Vec<usize> = v.iter().take(n_seq as usize).map(|x| x.n).collect();
        s.clear();
        let v_ptr = v.as_mut_ptr();
        for i in 0..n_seq as usize {
            for j in 0..nv[i] {
                let st = (v[i].a[j].info >> 32) as u32;
                let en = v[i].a[j].info as u32;
                if en - st < (min_len * 2) as u32 || v[i].a[j].size > max_sub_occ as u64 {
                    continue;
                }
                s.push(mb_smem_entry_ref {
                    min_len: (((en - st) / 2) as i32).max(min_len),
                    min_occ: v[i].a[j].size as i32 + 1,
                    st: st as i32,
                    en: en as i32,
                    q: seq[i],
                    v: unsafe { v_ptr.add(i) },
                    stage: 0,
                    x: 0,
                    i: 0,
                    kmer: 0,
                    p: mb_sai_t::default(),
                });
                if s.len() == MAX_BATCH_SIZE {
                    mb_bwt_smem_batch_ref_with_queue(km, bwt, s.len() as i32, &mut s, v, &mut tq);
                    s.clear();
                }
            }
        }
        if !s.is_empty() {
            mb_bwt_smem_batch_ref_with_queue(km, bwt, s.len() as i32, &mut s, v, &mut tq);
        }
        for vi in v.iter_mut().take(n_seq as usize) {
            vi.m = vi.a.capacity();
        }
    }

    /// Original C static function `mb_seed_sort_dedup` from `minibwa/seed.c:109`.
    pub fn mb_seed_sort_dedup(u: &mut mb_sai_v) {
        if u.n <= 1 {
            return;
        }
        radix_sort_mb_sai_by_key(&mut u.a[..u.n], |x| x.x[0]);
        let mut i0 = 0usize;
        let mut i = 1usize;
        while i <= u.n {
            if i == u.n || u.a[i].x[0] != u.a[i0].x[0] {
                if i - i0 > 1 {
                    radix_sort_mb_sai_by_key(&mut u.a[i0..i], |x| x.size);
                    u.a[..u.n].reverse();
                    let mut k0 = i0;
                    let mut k = i0 + 1;
                    while k <= i {
                        if k == i || u.a[k0].size != u.a[k].size {
                            if k - k0 > 1 {
                                radix_sort_mb_sai_by_key(&mut u.a[k0..k], |x| x.info);
                            }
                            k0 = k;
                        }
                        k += 1;
                    }
                }
                i0 = i;
            }
            i += 1;
        }
        let mut j = 0usize;
        for i in 1..u.n {
            if !(u.a[i].x[0] == u.a[j].x[0]
                && u.a[i].size == u.a[j].size
                && u.a[i].info == u.a[j].info)
            {
                j += 1;
                u.a[j] = u.a[i];
            }
        }
        u.n = j + 1;
        u.a.truncate(u.n);
        u.m = u.a.capacity();
    }

    fn radix_sort_mb_sai_by_key(a: &mut [mb_sai_t], key: fn(&mb_sai_t) -> u64) {
        const RS_MIN_SIZE: usize = 64;
        const RS_MAX_BITS: u32 = 8;

        fn insertion_sort(a: &mut [mb_sai_t], key: fn(&mb_sai_t) -> u64) {
            for i in 1..a.len() {
                if key(&a[i]) < key(&a[i - 1]) {
                    let tmp = a[i];
                    let mut j = i;
                    while j > 0 && key(&tmp) < key(&a[j - 1]) {
                        a[j] = a[j - 1];
                        j -= 1;
                    }
                    a[j] = tmp;
                }
            }
        }

        fn sort_rec(a: &mut [mb_sai_t], key: fn(&mb_sai_t) -> u64, n_bits: u32, s: u32) {
            let size = 1usize << n_bits;
            let mask = size - 1;
            let mut b = vec![0usize; size];
            let mut e = vec![0usize; size];
            for x in a.iter() {
                e[((key(x) >> s) as usize) & mask] += 1;
            }
            let mut sum = 0usize;
            for k in 0..size {
                let count = e[k];
                b[k] = sum;
                sum += count;
                e[k] = sum;
            }
            let bucket_end = e.clone();

            let mut k = 0usize;
            while k < size {
                if b[k] != e[k] {
                    let mut l = ((key(&a[b[k]]) >> s) as usize) & mask;
                    if l != k {
                        let mut tmp = a[b[k]];
                        loop {
                            std::mem::swap(&mut tmp, &mut a[b[l]]);
                            b[l] += 1;
                            l = ((key(&tmp) >> s) as usize) & mask;
                            if l == k {
                                break;
                            }
                        }
                        a[b[k]] = tmp;
                        b[k] += 1;
                    } else {
                        b[k] += 1;
                    }
                } else {
                    k += 1;
                }
            }

            let mut bucket_start = vec![0usize; size];
            bucket_start[1..size].copy_from_slice(&bucket_end[..size - 1]);
            if s != 0 {
                let next_s = s.saturating_sub(n_bits);
                for k in 0..size {
                    let start = bucket_start[k];
                    let end = bucket_end[k];
                    let len = end - start;
                    if len > RS_MIN_SIZE {
                        sort_rec(&mut a[start..end], key, n_bits, next_s);
                    } else if len > 1 {
                        insertion_sort(&mut a[start..end], key);
                    }
                }
            }
        }

        if a.len() <= RS_MIN_SIZE {
            insertion_sort(a, key);
        } else {
            sort_rec(a, key, RS_MAX_BITS, 7 * RS_MAX_BITS);
        }
    }

    /// Original C static function `mb_anchor_dedup` from `minibwa/seed.c:142`.
    pub fn mb_anchor_dedup(v: &mut mb_anchor_v) {
        const MAX_BACK: usize = 100;
        for i in 1..v.n as usize {
            let ai = v.a[i];
            let tsi = ai.tpos + 1 - ai.len as i64;
            let qsi = ai.qpos + 1 - ai.len;
            let mut k = 0usize;
            let mut j = i;
            while j > 0 && k < MAX_BACK {
                j -= 1;
                let aj = v.a[j];
                if aj.sid != ai.sid || aj.tpos < tsi {
                    break;
                }
                let tsj = aj.tpos + 1 - aj.len as i64;
                let qsj = aj.qpos + 1 - aj.len;
                if tsj >= tsi {
                    if tsj - tsi == (qsj - qsi) as i64 {
                        v.a[j].flt = 1;
                    }
                } else if ai.tpos == aj.tpos && ai.qpos == aj.qpos {
                    v.a[i].flt = 1;
                }
                k += 1;
            }
        }
        let mut j = 0usize;
        for i in 0..v.n as usize {
            if v.a[i].flt == 0 {
                if j < i {
                    v.a[j] = v.a[i];
                }
                j += 1;
            }
        }
        v.n = j as i64;
        v.a.truncate(j);
    }

    fn radix_sort_mb_anchor_by_tpos(a: &mut [mb_anchor_t]) {
        const RS_MIN_SIZE: usize = 64;
        const RS_MAX_BITS: u32 = 8;

        fn key(x: &mb_anchor_t) -> u64 {
            x.tpos as u64
        }

        fn insertion_sort(a: &mut [mb_anchor_t]) {
            for i in 1..a.len() {
                if key(&a[i]) < key(&a[i - 1]) {
                    let tmp = a[i];
                    let mut j = i;
                    while j > 0 && key(&tmp) < key(&a[j - 1]) {
                        a[j] = a[j - 1];
                        j -= 1;
                    }
                    a[j] = tmp;
                }
            }
        }

        fn sort_rec(a: &mut [mb_anchor_t], n_bits: u32, s: u32) {
            let size = 1usize << n_bits;
            let mask = size - 1;
            let mut b = vec![0usize; size];
            let mut e = vec![0usize; size];
            for x in a.iter() {
                e[((key(x) >> s) as usize) & mask] += 1;
            }
            let mut sum = 0usize;
            for k in 0..size {
                let count = e[k];
                b[k] = sum;
                sum += count;
                e[k] = sum;
            }
            let bucket_end = e.clone();

            let mut k = 0usize;
            while k < size {
                if b[k] != e[k] {
                    let mut l = ((key(&a[b[k]]) >> s) as usize) & mask;
                    if l != k {
                        let mut tmp = a[b[k]];
                        loop {
                            std::mem::swap(&mut tmp, &mut a[b[l]]);
                            b[l] += 1;
                            l = ((key(&tmp) >> s) as usize) & mask;
                            if l == k {
                                break;
                            }
                        }
                        a[b[k]] = tmp;
                        b[k] += 1;
                    } else {
                        b[k] += 1;
                    }
                } else {
                    k += 1;
                }
            }

            let mut bucket_start = vec![0usize; size];
            bucket_start[1..size].copy_from_slice(&bucket_end[..size - 1]);
            if s != 0 {
                let next_s = s.saturating_sub(n_bits);
                for k in 0..size {
                    let start = bucket_start[k];
                    let end = bucket_end[k];
                    let len = end - start;
                    if len > RS_MIN_SIZE {
                        sort_rec(&mut a[start..end], n_bits, next_s);
                    } else if len > 1 {
                        insertion_sort(&mut a[start..end]);
                    }
                }
            }
        }

        if a.len() <= RS_MIN_SIZE {
            insertion_sort(a);
        } else {
            sort_rec(a, RS_MAX_BITS, 7 * RS_MAX_BITS);
        }
    }

    /// Original C static function `process_batch` from `minibwa/seed.c:175`.
    pub fn process_batch(
        km: (),
        idx: &mb_idx_t,
        aux: &[(i64, i64)],
        m: i32,
        b: &[(i64, i64)],
        a: &mut [u64],
        sa_batch: &mut Vec<(u64, u64)>,
        qlen: i32,
        mt: l2b_meth_t,
        u: &mb_sai_v,
        v: &mut mb_anchor_v,
    ) {
        for k in 0..m as usize {
            a[k] = b[k].0 as u64;
        }
        mb_bwt_sa_batch_with_scratch(km, &idx.bwt, m as i64, a, sa_batch);
        for k in 0..m as usize {
            let p = aux[b[k].1 as usize];
            for j in p.0..p.1 {
                let qs = (u.a[j as usize].info >> 32) as i32;
                let qe = u.a[j as usize].info as u32 as i32;
                let len = qe - qs;
                let mut rev = 0;
                let mut cst = 0i64;
                let tid = if mt != l2b_meth_t::L2B_METH_NONE {
                    let mut mt_anchor = l2b_meth_t::L2B_METH_NONE;
                    let tid = l2b_intv2cid_meth(
                        &idx.l2b,
                        a[k],
                        a[k] + len as u64,
                        &mut mt_anchor,
                        &mut cst,
                        &mut rev,
                    );
                    if tid < 0 {
                        continue;
                    }
                    if (mt_anchor == mt) != (rev == 0) {
                        continue;
                    }
                    tid
                } else {
                    let tid = l2b_intv2cid(&idx.l2b, a[k], a[k] + len as u64, &mut cst, &mut rev);
                    if tid < 0 {
                        continue;
                    }
                    tid
                };
                rev = (rev != 0) as i32;
                let ctg = &idx.l2b.ctg[tid as usize];
                v.a.push(mb_anchor_t {
                    sid: ((tid as i32) << 1) | rev,
                    len,
                    qpos: if rev != 0 {
                        qlen - 1 - qs
                    } else {
                        qs + len - 1
                    },
                    flag: 0,
                    flt: 0,
                    tpos: (ctg.off * 2 + ctg.len * rev as u64) as i64 + cst + len as i64 - 1,
                });
                v.n += 1;
            }
        }
        v.m = v.a.capacity() as i64;
    }

    /// Original C global function `mb_anchor` from `minibwa/seed.c:206`.
    pub fn mb_anchor(
        km: (),
        idx: &mb_idx_t,
        u: &mut mb_sai_v,
        qlen: i32,
        mt: l2b_meth_t,
        max_occ: i32,
        v: &mut mb_anchor_v,
    ) {
        let mut aux = Vec::new();
        let mut a = Vec::new();
        let mut sa_batch = Vec::new();
        let mut b = Vec::new();
        mb_anchor_with_scratch(
            km,
            idx,
            u,
            qlen,
            mt,
            max_occ,
            v,
            &mut aux,
            &mut a,
            &mut sa_batch,
            &mut b,
        );
    }

    pub fn mb_anchor_with_scratch(
        km: (),
        idx: &mb_idx_t,
        u: &mut mb_sai_v,
        qlen: i32,
        mt: l2b_meth_t,
        max_occ: i32,
        v: &mut mb_anchor_v,
        aux: &mut Vec<(i64, i64)>,
        a: &mut Vec<u64>,
        sa_batch: &mut Vec<(u64, u64)>,
        b: &mut Vec<(i64, i64)>,
    ) {
        const BATCH_SIZE: i32 = 20;
        v.n = 0;
        v.a.clear();
        if u.n == 0 {
            return;
        }
        mb_seed_sort_dedup(u);
        aux.clear();
        let mut i0 = 0usize;
        let mut i = 1usize;
        while i <= u.n {
            if i == u.n || u.a[i].x[0] != u.a[i0].x[0] || u.a[i].size != u.a[i0].size {
                aux.push((i0 as i64, i as i64));
                i0 = i;
            }
            i += 1;
        }
        let m_a = max_occ.max(BATCH_SIZE) as usize;
        a.clear();
        a.resize(m_a.max(1), 0);
        b.clear();
        b.reserve(m_a.max(1).saturating_sub(b.capacity()));
        for (i, p) in aux.iter().enumerate() {
            let q = &u.a[p.0 as usize];
            if q.size as usize + b.len() > BATCH_SIZE as usize {
                process_batch(km, idx, aux, b.len() as i32, b, a, sa_batch, qlen, mt, u, v);
                b.clear();
            }
            if q.size <= max_occ as u64 {
                for j in 0..q.size {
                    b.push((q.x[0].wrapping_add(j) as i64, i as i64));
                }
            } else {
                let mut n = 0i32;
                let mut j = 0u64;
                while j < q.size && n < max_occ {
                    let mut step = (q.size - j) / (max_occ - n) as u64;
                    if step < 1 {
                        step = 1;
                    }
                    b.push((q.x[0].wrapping_add(j) as i64, i as i64));
                    j += step;
                    n += 1;
                }
            }
            assert!(b.len() <= m_a);
        }
        process_batch(km, idx, aux, b.len() as i32, b, a, sa_batch, qlen, mt, u, v);

        radix_sort_mb_anchor_by_tpos(&mut v.a);
        for q in &mut v.a {
            let ctg = &idx.l2b.ctg[(q.sid >> 1) as usize];
            q.tpos -= (ctg.off * 2 + ctg.len * (q.sid & 1) as u64) as i64;
        }
        v.n = v.a.len() as i64;
        mb_anchor_dedup(v);
    }

    /// Original C global function `mb_anchor_sort` from `minibwa/seed.c:269`.
    pub fn mb_anchor_sort(l2b: &l2b_t, n_a: i64, a: &mut [mb_anchor_t]) {
        if n_a <= 1 {
            return;
        }
        for x in a.iter_mut().take(n_a as usize) {
            let ctg = &l2b.ctg[(x.sid >> 1) as usize];
            x.tpos += (ctg.off * 2 + ctg.len * (x.sid & 1) as u64) as i64;
        }
        radix_sort_mb_anchor_by_tpos(&mut a[..n_a as usize]);
        for x in a.iter_mut().take(n_a as usize) {
            let ctg = &l2b.ctg[(x.sid >> 1) as usize];
            x.tpos -= (ctg.off * 2 + ctg.len * (x.sid & 1) as u64) as i64;
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::bwt::mb_bwt_load;
        use crate::map_algo::mb_idx_load;

        #[test]
        fn seed_intervals_are_found_on_real_chrm_bwt() {
            let bwt = mb_bwt_load("minibwa/chrM-human.mbw").expect("load bwt");
            let seq = b"GATCACAGGTCTATCACCCTATTAACCACTCACGGGAGCTCTCCATGCAT"
                .iter()
                .map(|&c| match c {
                    b'A' | b'a' => 0,
                    b'C' | b'c' => 1,
                    b'G' | b'g' => 2,
                    b'T' | b't' => 3,
                    _ => 4,
                })
                .collect::<Vec<_>>();
            let mut v = mb_sai_v::default();
            mb_seed_intv((), &bwt, seq.len() as i32, &seq, 19, 10, &mut v);
            assert!(v.n > 0);
            assert!(v.a.iter().take(v.n).all(|x| x.size > 0));
            assert!(v
                .a
                .iter()
                .take(v.n)
                .any(|x| (x.info as u32) - (x.info >> 32) as u32 >= 19));
        }

        #[test]
        fn seed_batch_matches_single_seed_sets() {
            let bwt = mb_bwt_load("minibwa/chrM-human.mbw").expect("load bwt");
            let seqs = [
                "GATCACAGGTCTATCACCCTATTAACCACTCACGGGAGCTCTCCATGCAT",
                "ATCACAGGTCTATCACCCTATTAACCACTCACGGGAGCTCTCCATGCAT",
            ]
            .iter()
            .map(|s| {
                s.bytes()
                    .map(|c| match c {
                        b'A' | b'a' => 0,
                        b'C' | b'c' => 1,
                        b'G' | b'g' => 2,
                        b'T' | b't' => 3,
                        _ => 4,
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
            let lens = seqs.iter().map(|s| s.len() as i32).collect::<Vec<_>>();
            let seq_refs = seqs.iter().map(|s| s.as_ptr()).collect::<Vec<_>>();
            let mut batch = vec![mb_sai_v::default(); seqs.len()];
            mb_seed_intv_batch(
                (),
                &bwt,
                seqs.len() as i32,
                &lens,
                &seq_refs,
                19,
                10,
                &mut batch,
            );
            for i in 0..seqs.len() {
                let mut single = mb_sai_v::default();
                mb_seed_intv((), &bwt, lens[i], &seqs[i], 19, 10, &mut single);
                let mut a = single.a[..single.n].to_vec();
                let mut b = batch[i].a[..batch[i].n].to_vec();
                a.sort_by_key(|x| (x.x[0], x.size, x.info));
                b.sort_by_key(|x| (x.x[0], x.size, x.info));
                assert_eq!(b, a);
            }
        }

        #[test]
        #[ignore = "requires .tmp/large-real/yeast fixtures prepared from the real yeast conformance data"]
        fn seed_batch_matches_single_on_yeast_poly_t_read() {
            let bwt = mb_bwt_load(".tmp/large-real/yeast/ref.orig.mbw").expect("load yeast bwt");
            let seq = b"GGTTCCGATCTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTC"
                .iter()
                .map(|&c| match c {
                    b'A' | b'a' => 0,
                    b'C' | b'c' => 1,
                    b'G' | b'g' => 2,
                    b'T' | b't' => 3,
                    _ => 4,
                })
                .collect::<Vec<_>>();
            let seqs = [seq.clone()];
            let lens = [seq.len() as i32];
            let mut single = mb_sai_v::default();
            mb_seed_intv((), &bwt, lens[0], &seq, 19, 10, &mut single);
            let seq_refs = seqs.iter().map(|s| s.as_ptr()).collect::<Vec<_>>();
            let mut batch = vec![mb_sai_v::default()];
            mb_seed_intv_batch((), &bwt, 1, &lens, &seq_refs, 19, 10, &mut batch);

            let mut a = single.a[..single.n].to_vec();
            let mut b = batch[0].a[..batch[0].n].to_vec();
            a.sort_by_key(|x| (x.x[0], x.size, x.info));
            b.sort_by_key(|x| (x.x[0], x.size, x.info));
            assert_eq!(b, a);
        }

        #[test]
        fn anchors_are_generated_on_real_chrm_index() {
            let idx = mb_idx_load("minibwa/chrM-human", 0).expect("load idx");
            let seq = b"GATCACAGGTCTATCACCCTATTAACCACTCACGGGAGCTCTCCATGCAT"
                .iter()
                .map(|&c| match c {
                    b'A' | b'a' => 0,
                    b'C' | b'c' => 1,
                    b'G' | b'g' => 2,
                    b'T' | b't' => 3,
                    _ => 4,
                })
                .collect::<Vec<_>>();
            let mut u = mb_sai_v::default();
            mb_seed_intv((), &idx.bwt, seq.len() as i32, &seq, 19, 10, &mut u);
            let mut v = mb_anchor_v::default();
            mb_anchor(
                (),
                &idx,
                &mut u,
                seq.len() as i32,
                l2b_meth_t::L2B_METH_NONE,
                250,
                &mut v,
            );
            assert!(v.n > 0);
            assert!(v.a.iter().all(|a| a.sid >> 1 == 0));
            assert!(v.a.iter().all(|a| a.len >= 19));
        }

        #[test]
        fn anchor_sort_uses_absolute_contig_order_then_restores_local_tpos() {
            let mut l2b = l2b_t::default();
            l2b.n_ctg = 2;
            l2b.ctg = vec![
                crate::l2bit::l2b_ctg_t {
                    name: "a".into(),
                    len: 100,
                    off: 0,
                    comm: None,
                },
                crate::l2bit::l2b_ctg_t {
                    name: "b".into(),
                    len: 100,
                    off: 100,
                    comm: None,
                },
            ];
            let mut a = vec![
                mb_anchor_t {
                    sid: 2,
                    tpos: 1,
                    ..Default::default()
                },
                mb_anchor_t {
                    sid: 0,
                    tpos: 90,
                    ..Default::default()
                },
            ];
            mb_anchor_sort(&l2b, a.len() as i64, &mut a);
            assert_eq!(
                a.iter().map(|x| (x.sid, x.tpos)).collect::<Vec<_>>(),
                vec![(0, 90), (2, 1)]
            );
        }
    }
}
