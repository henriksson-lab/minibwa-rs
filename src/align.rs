#![allow(unused_variables, dead_code, non_snake_case)]

use crate::cs::{mb_write_MD, mb_write_cs_ds};
use crate::ksw2::{
    ksw_extz_t, ksw_push_cigar, ksw_reset_extz, KSW_EZ_APPROX_MAX, KSW_EZ_EXTZ_ONLY,
    KSW_EZ_GENERIC_SC, KSW_EZ_REV_CIGAR, KSW_EZ_RIGHT,
};
use crate::ksw2_extd2_sse::ksw_extd2_sse;
use crate::ksw2_extz2_sse::ksw_extz2_sse;
use crate::ksw2_ll_sse::{ksw_ll_i16, ksw_ll_qinit};
use crate::l2bit::{l2b_getseq, l2b_meth_rev, l2b_meth_t};
use crate::lchain::mb_anchor_t;
use crate::map_algo::{
    mb_cal_mblen, mb_filter_hits, mb_hit_sort, mb_idx_t, mb_split_hit, mb_squeeze_a,
    MB_PARENT_TMP_PRI, MB_PARENT_UNSET,
};
use crate::mbpriv::{
    mb_cstr_prefix, mb_is_sr_mode, mb_log2, mb_seq_rev, KOM_DBG_FLAG, MB_DBG_ALN_SEQ, MB_DBG_AN_POS,
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
    debug_assert_eq!(qoff, r.qe - r.qs);
    debug_assert_eq!(toff, (r.te - r.ts) as i32);
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
                    } else if unsafe { *mat_ptr.add(ct * 5 + cq) } < 0 {
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
        p.add_n_ambi(total_n_ambi);
        p.dp_max0 = (max + 0.499) as i32;
        p.dp_max = p.dp_max0;
    }
    debug_assert_eq!(qoff as i32, r.qe - r.qs);
    debug_assert_eq!(toff as i32, (r.te - r.ts) as i32);
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
    if (opt.b_ts != 0 && opt.b != opt.b_ts) || (opt.flag & crate::options::MB_F_METH) != 0 {
        ksw_flag |= KSW_EZ_GENERIC_SC;
    }
    if (ksw_flag & KSW_EZ_EXTZ_ONLY) != 0 && tlen >= qlen {
        ksw_reset_extz(ez);
        ez.score = 0;
        ez.max = 0;
        for j in 0..qlen as usize {
            let sc = mat[tseq[j] as usize * 5 + qseq[j] as usize] as i32;
            ez.score += sc;
            n_mm += (tseq[j] > 3 || qseq[j] > 3 || sc < 0) as i32;
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
            let sc = mat[tseq[j] as usize * 5 + qseq[j] as usize] as i32;
            ez.score += sc;
            n_mm += (tseq[j] > 3 || qseq[j] > 3 || sc < 0) as i32;
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
    let mut mat_local = [0i8; 25];
    crate::ksw2::ksw_gen_nt4_mat(
        &mut mat_local,
        opt.a as i8,
        opt.b as i8,
        opt.b_ts as i8,
        opt.b_ambi as i8,
        mt as i32,
    );
    let mat = &mat_local;
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
    let mut ts = first.tpos + 1 - first.len as i64 + mb_min_int32(first.len >> 1, max_back) as i64;
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
                r.as_,
                mb_cstr_prefix(&mi.l2b.ctg[tid as usize].name),
                cur.tpos,
                cur.qpos,
                gap,
                cur.len
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
    let (ts1, qs1) = if qs > 0 && ts > 0 {
        l2b_getseq(&mi.l2b, tid, ts0, ts, &mut tseq[..(ts - ts0) as usize]);
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
        let te = a[as1 as usize].tpos + 1 - mb_min_int32(a[as1 as usize].len >> 1, max_back) as i64;
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
            l2b_getseq(&mi.l2b, tid, ts, te, &mut tseq[..(te - ts) as usize]);
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
                if r.p.is_none() {
                    r.p = Some(
                        crate::pe::mb_extra_t {
                            cap: 1,
                            ..Default::default()
                        }
                        .boxed(),
                    );
                }
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
                r.p.as_mut().unwrap().dp_score += ez.max as i32;
                te1 = ts + ez.max_t as i64 + 1;
                qe1 = qs + ez.max_q + 1;
                let mut blen = 0;
                let mlen = mb_cal_mblen(cnt1 - (j + 1), &a[(as1 + j + 1) as usize..], &mut blen);
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
        l2b_getseq(&mi.l2b, tid, te1, te0, &mut tseq[..(te0 - te1) as usize]);
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
        l2b_getseq(&mi.l2b, tid, ts1, te1, &mut tseq[..(te1 - ts1) as usize]);
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
    if ql < opt.min_chain_score || ql > opt.max_gap || tl < opt.min_chain_score || tl > opt.max_gap
    {
        return 0;
    }
    tseq.clear();
    tseq.resize(tl as usize, 0);
    if r1.rev() == 0 {
        mt = l2b_meth_rev(mt);
    }
    let mut mat_local = [0i8; 25];
    crate::ksw2::ksw_gen_nt4_mat(
        &mut mat_local,
        opt.a as i8,
        opt.b as i8,
        opt.b_ts as i8,
        opt.b_ambi as i8,
        mt as i32,
    );
    let mat = &mat_local;
    l2b_getseq(&mi.l2b, r1.tid, r1.te, r2.ts, tseq);
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
    let qp = ksw_ll_qinit(km, 2, ql, qseq, 5, mat);
    let score = ksw_ll_i16(&qp, tl, tseq, opt.q, opt.e, &mut q_off, &mut t_off);
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
pub fn mb_update_dp_max(qlen: i32, n_regs: i32, regs: &mut [mb_hit_t], frac: f64, a: i32, b: i32) {
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
    if ((regs[max_i as usize].qe - regs[max_i as usize].qs) as f64) < qlen as f64 * frac {
        return;
    }
    if ((regs[max2_i as usize].qe - regs[max2_i as usize].qs) as f64)
        < ((regs[max_i as usize].qe - regs[max_i as usize].qs) as f64) * frac.sqrt()
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
        mt as i32,
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
            2, -4, -4, -4, -1, -4, 2, -4, -4, -1, -4, -4, 2, -4, -1, -4, -4, -4, 2, -1, -1, -1, -1,
            -1, -1,
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
            hit.p.as_ref().unwrap().cigar()[..hit.p.as_ref().unwrap().n_cigar as usize].to_vec(),
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
            hit.p.as_ref().unwrap().cigar()[..hit.p.as_ref().unwrap().n_cigar as usize].to_vec(),
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
            2, -4, -4, -4, -1, -4, 2, -4, -4, -1, -4, -4, 2, -4, -1, -4, -4, -4, 2, -1, -1, -1, -1,
            -1, -1,
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
            0,
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
            0,
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

        let target = "AAAAATAGCTGGGCATG"
            .bytes()
            .map(|c| match c {
                b'A' => 0,
                b'C' => 1,
                b'G' => 2,
                b'T' => 3,
                _ => 4,
            })
            .collect::<Vec<_>>();
        let query = "GCTGGGCAT"
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
            61,
            opt.end_bonus,
            opt.zdrop,
            KSW_EZ_EXTZ_ONLY,
            &mut ez,
        );
        assert_eq!(ez.n_cigar, 0);
        assert_eq!(ez.max, 0);
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
