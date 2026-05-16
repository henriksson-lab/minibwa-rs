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
        if !(u.a[i].x[0] == u.a[j].x[0] && u.a[i].size == u.a[j].size && u.a[i].info == u.a[j].info)
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

/// Remove duplicated anchors. The two-round seeding algorithm may lead an
/// anchor precisely contained in a longer anchor. This routine filters out the
/// shorter anchor. This wouldn't happen to minimap2.
///
/// NB: assuming sorted by `tpos`.
///
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

/// Converting seed intervals to anchors. This function batches small SA
/// intervals and calls `mb_bwt_sa_batch()` in `process_batch()`. With prefetch,
/// the strategy noticeably improves the performance.
///
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
