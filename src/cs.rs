
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
