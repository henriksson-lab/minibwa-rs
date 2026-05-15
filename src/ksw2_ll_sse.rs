
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

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn ksw_le_u8_native(a: std::arch::x86_64::__m128i, b: std::arch::x86_64::__m128i) -> bool {
    unsafe {
        let diff = std::arch::x86_64::_mm_subs_epu8(a, b);
        let eq = std::arch::x86_64::_mm_cmpeq_epi8(diff, std::arch::x86_64::_mm_setzero_si128());
        std::arch::x86_64::_mm_movemask_epi8(eq) == 0xffff
    }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn ksw_max_u8_native(mut v: std::arch::x86_64::__m128i) -> i32 {
    unsafe {
        v = std::arch::x86_64::_mm_max_epu8(v, std::arch::x86_64::_mm_srli_si128::<8>(v));
        v = std::arch::x86_64::_mm_max_epu8(v, std::arch::x86_64::_mm_srli_si128::<4>(v));
        v = std::arch::x86_64::_mm_max_epu8(v, std::arch::x86_64::_mm_srli_si128::<2>(v));
        v = std::arch::x86_64::_mm_max_epu8(v, std::arch::x86_64::_mm_srli_si128::<1>(v));
        std::arch::x86_64::_mm_extract_epi16::<0>(v) & 0x00ff
    }
}

#[cfg(target_arch = "x86_64")]
fn ksw_ll_u8_core_native(
    q: &kswq_t,
    tlen: i32,
    target: &[u8],
    _gapo: i32,
    _gape: i32,
    xtra: i32,
) -> ksw_llrst_t {
    unsafe {
        let qlen = q.qlen as usize;
        let slen = q.slen.max(0) as usize;
        let tlen = tlen.max(0) as usize;
        let gapoe = std::arch::x86_64::_mm_set1_epi8((_gapo + _gape) as i8);
        let gape = std::arch::x86_64::_mm_set1_epi8(_gape as i8);
        let shift = std::arch::x86_64::_mm_set1_epi8(q.shift as i8);
        let zero = std::arch::x86_64::_mm_setzero_si128();
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
        macro_rules! run_core {
                ($h0_buf:expr, $h1_buf:expr, $e_buf:expr, $hmax_buf:expr) => {{
                    let mut h0 = &mut $h0_buf[..slen];
                    let mut h1 = &mut $h1_buf[..slen];
                    let e_vec = &mut $e_buf[..slen];
                    let hmax = &mut $hmax_buf[..slen];
                    let mut r = ksw_llrst_t::default();
                    let mut b_stack = [0u64; 64];
                    let mut b_len = 0usize;
                    let mut b_overflow = Vec::<u64>::new();
                    let mut gmax = 0u8;
                    let mut te = -1i32;
                    for i in 0..tlen {
                        let mut f = zero;
                        let mut max = zero;
                        let s_base = target[i] as usize * slen;
                        let mut h = if slen == 0 { zero } else { h0[slen - 1] };
                        h = std::arch::x86_64::_mm_slli_si128::<1>(h);
                        for j in 0..slen {
                            let qp = std::arch::x86_64::_mm_loadu_si128(
                                q.qp.as_ptr().add(s_base + j).cast(),
                            );
                            h = std::arch::x86_64::_mm_adds_epu8(h, qp);
                            h = std::arch::x86_64::_mm_subs_epu8(h, shift);
                            let mut e = e_vec[j];
                            h = std::arch::x86_64::_mm_max_epu8(h, e);
                            h = std::arch::x86_64::_mm_max_epu8(h, f);
                            max = std::arch::x86_64::_mm_max_epu8(max, h);
                            h1[j] = h;
                            e = std::arch::x86_64::_mm_subs_epu8(e, gape);
                            let mut t = std::arch::x86_64::_mm_subs_epu8(h, gapoe);
                            e = std::arch::x86_64::_mm_max_epu8(e, t);
                            e_vec[j] = e;
                            f = std::arch::x86_64::_mm_subs_epu8(f, gape);
                            t = std::arch::x86_64::_mm_subs_epu8(h, gapoe);
                            f = std::arch::x86_64::_mm_max_epu8(f, t);
                            h = h0[j];
                        }
                        'lazy_f_u8: for _k in 0..16 {
                            f = std::arch::x86_64::_mm_slli_si128::<1>(f);
                            for h1_j in h1.iter_mut().take(slen) {
                                h = *h1_j;
                                h = std::arch::x86_64::_mm_max_epu8(h, f);
                                *h1_j = h;
                                h = std::arch::x86_64::_mm_subs_epu8(h, gapoe);
                                f = std::arch::x86_64::_mm_subs_epu8(f, gape);
                                if ksw_le_u8_native(f, h) {
                                    break 'lazy_f_u8;
                                }
                            }
                        }
                        let row_max = ksw_max_u8_native(max) as u8;
                        if row_max as i32 >= minsc {
                            let val = ((row_max as u64) << 32) | i as u64;
                            let last = if b_overflow.is_empty() {
                                b_len.checked_sub(1).map(|idx| b_stack[idx])
                            } else {
                                b_overflow.last().copied()
                            };
                            if last.map_or(true, |v| (v as i32) + 1 != i as i32) {
                                if b_overflow.is_empty() && b_len < b_stack.len() {
                                    b_stack[b_len] = val;
                                    b_len += 1;
                                } else {
                                    if b_overflow.is_empty() {
                                        b_overflow.extend_from_slice(&b_stack[..b_len]);
                                    }
                                    b_overflow.push(val);
                                }
                            } else if ((last.unwrap() >> 32) as i32) < row_max as i32 {
                                if b_overflow.is_empty() {
                                    b_stack[b_len - 1] = val;
                                } else {
                                    let last_idx = b_overflow.len() - 1;
                                    b_overflow[last_idx] = val;
                                }
                            }
                        }
                        if row_max > gmax {
                            gmax = row_max;
                            te = i as i32;
                            hmax.copy_from_slice(h1);
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
                            let lanes: [u8; 16] = std::mem::transmute(hmax[i / 16]);
                            let h = lanes[i % 16] as i32;
                            if h > max {
                                max = h;
                                r.qe = tmp;
                            } else if h == max && tmp < r.qe {
                                r.qe = tmp;
                            }
                        }
                        if (b_len != 0 || !b_overflow.is_empty()) && q.max != 0 {
                            let radius = (r.score + q.max as i32 - 1) / q.max as i32;
                            let low = te - radius;
                            let high = te + radius;
                            let b_values: &[u64] = if b_overflow.is_empty() {
                                &b_stack[..b_len]
                            } else {
                                &b_overflow
                            };
                            for &v in b_values {
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
                }};
            }
        if slen <= 32 {
            let mut h0 = [zero; 32];
            let mut h1 = [zero; 32];
            let mut e_vec = [zero; 32];
            let mut hmax = [zero; 32];
            run_core!(h0, h1, e_vec, hmax)
        } else {
            let mut h0 = vec![zero; slen];
            let mut h1 = vec![zero; slen];
            let mut e_vec = vec![zero; slen];
            let mut hmax = vec![zero; slen];
            run_core!(h0, h1, e_vec, hmax)
        }
    }
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
    #[cfg(target_arch = "x86_64")]
    {
        return ksw_ll_u8_core_native(q, tlen, target, _gapo, _gape, xtra);
    }
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
            2, -3, -3, -3, -1, -3, 2, -3, -3, -1, -3, -3, 2, -3, -1, -3, -3, -3, 2, -1, -1, -1, -1,
            -1, -1,
        ];
        let q = ksw_ll_qinit((), 2, 4, &[0, 1, 2, 3], 5, &mat);
        assert_eq!((q.size, q.slen, q.shift, q.max), (2, 1, 259u16 as u8, 2));
        assert_eq!(q.mdiff, 5);
    }

    #[test]
    fn ll_i16_core_matches_simple_sw_result() {
        let mat = [
            2, -3, -3, -3, -1, -3, 2, -3, -3, -1, -3, -3, 2, -3, -1, -3, -3, -3, 2, -1, -1, -1, -1,
            -1, -1,
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
            5, -4, -4, -4, -1, -4, 5, -4, -4, -1, -4, -4, 5, -4, -1, -4, -4, -4, 5, -1, -1, -1, -1,
            -1, -1,
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
