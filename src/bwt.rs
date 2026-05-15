#![allow(unused_variables, dead_code, non_snake_case, non_camel_case_types)]

use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::Path;

const MB_MAGIC: &[u8; 4] = b"MBW\x02";
const BWT_CNT_SHIFT: u32 = 56;
const BWT_CNT_MASK: u64 = (1u64 << BWT_CNT_SHIFT) - 1;

#[repr(C)]
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

#[repr(C)]
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
    pub kmer: u32,
    pub p: mb_sai_t,
}

#[repr(C)]
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
    pub kmer: u32,
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
pub fn mb_bwt_pre_prefetch(bwt: &mb_bwt_t, kmer: u32) {
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
                        s.kmer = s.kmer << 2 | s.q[s.i as usize] as u32;
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
                            s.kmer = s.kmer << 2 | s.q[s.i as usize] as u32;
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
                        s.kmer = s.kmer << 2 | unsafe { smem_q_ref(s, s.i) } as u32;
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
                            s.kmer = s.kmer << 2 | unsafe { smem_q_ref(s, s.i) } as u32;
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

fn mb_bwt_count_kmer_uninit(bwt: &mb_bwt_t, depth: i32, s: &mut [std::mem::MaybeUninit<mb_sai_t>]) {
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
            std::slice::from_raw_parts(words.as_ptr().cast::<u8>(), std::mem::size_of_val(words))
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
            let mut val = u32::from_le_bytes(packed[word * 4..word * 4 + 4].try_into().unwrap());
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
