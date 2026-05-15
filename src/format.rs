
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

        let cigar = [8 << 4 | MB_CIGAR_MATCH];
        let mut tag_words = Vec::new();
        for chunk in b"cs:Z::8\0".chunks(4) {
            let mut w = 0u32;
            for (i, &b) in chunk.iter().enumerate() {
                w |= (b as u32) << (i * 8);
            }
            tag_words.push(w);
        }
        let mut extra = mb_extra_t {
            dp_score: 24,
            dp_max0: 24,
            dp_max: 24,
            dp_max2: 3,
            n_ambi_cs: mb_extra_t::CS_FLAG,
            ..Default::default()
        }
        .with_cigar(&cigar);
        extra.set_tag_words_from_slice(&tag_words);
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
            p: Some(extra),
            ..Default::default()
        };
        out.l = 0;
        mb_fmt_paf(&mut out, &l2b, &read, Some(&hit), MB_F_COPY_COMMENT, 1, 0);
        let got = String::from_utf8_lossy(&out.s[..out.l]);
        assert!(got
            .contains("read1\t8\t0\t8\t+\tchrM\t16569\t3\t11\t8\t8\t60\ttp:A:P\ts1:i:24\tcm:i:2"));
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
