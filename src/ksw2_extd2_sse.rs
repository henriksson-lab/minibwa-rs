#![allow(unused_variables, dead_code, non_snake_case, unreachable_code)]

use crate::ksw2::{
    ksw_apply_zdrop, ksw_backtrack, ksw_extz_t, ksw_reset_extz, KSW_EZ_APPROX_DROP,
    KSW_EZ_APPROX_MAX, KSW_EZ_EXTZ_ONLY, KSW_EZ_GENERIC_SC, KSW_EZ_REV_CIGAR, KSW_EZ_RIGHT,
    KSW_EZ_SCORE_ONLY, KSW_NEG_INF,
};
#[cfg(not(target_arch = "x86_64"))]
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

const KSW_TLS_RETAIN_MAX_BYTES: usize = 16 * 1024 * 1024;

#[inline(always)]
fn retain_capacity<T>(v: &Vec<T>) -> bool {
    v.capacity().saturating_mul(std::mem::size_of::<T>()) <= KSW_TLS_RETAIN_MAX_BYTES
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
#[allow(clippy::uninit_vec)]
fn take_u8_uninit(key: &'static LocalKey<RefCell<Vec<u8>>>, len: usize) -> Vec<u8> {
    key.with(|cell| {
        let mut v = std::mem::take(&mut *cell.borrow_mut());
        v.clear();
        v.reserve(len);
        unsafe { v.set_len(len) };
        v
    })
}

#[inline(always)]
fn put_u8(key: &'static LocalKey<RefCell<Vec<u8>>>, mut v: Vec<u8>) {
    v.clear();
    if !retain_capacity(&v) {
        v = Vec::new();
    }
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
#[allow(clippy::uninit_vec)]
fn take_i32_uninit(key: &'static LocalKey<RefCell<Vec<i32>>>, len: usize) -> Vec<i32> {
    key.with(|cell| {
        let mut v = std::mem::take(&mut *cell.borrow_mut());
        v.clear();
        v.reserve(len);
        unsafe { v.set_len(len) };
        v
    })
}

#[inline(always)]
fn put_i32(key: &'static LocalKey<RefCell<Vec<i32>>>, mut v: Vec<i32>) {
    v.clear();
    if !retain_capacity(&v) {
        v = Vec::new();
    }
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

#[inline(always)]
unsafe fn load_i32x4_at(slice: &[i32], offset: usize) -> [u8; 16] {
    let mut out = [0u8; 16];
    unsafe {
        std::ptr::copy_nonoverlapping(
            slice.as_ptr().add(offset).cast::<u8>(),
            out.as_mut_ptr(),
            16,
        );
    }
    out
}

#[inline(always)]
unsafe fn store_i32x4_at(slice: &mut [i32], offset: usize, value: [u8; 16]) {
    unsafe {
        std::ptr::copy_nonoverlapping(
            value.as_ptr(),
            slice.as_mut_ptr().add(offset).cast::<u8>(),
            16,
        );
    }
}

#[inline(always)]
unsafe fn i32x4_to_array(value: [u8; 16]) -> [i32; 4] {
    let mut out = [0i32; 4];
    unsafe {
        std::ptr::copy_nonoverlapping(value.as_ptr(), out.as_mut_ptr().cast::<u8>(), 16);
    }
    out
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn loadu128_u8(slice: &[u8], offset: usize) -> std::arch::x86_64::__m128i {
    unsafe { std::arch::x86_64::_mm_loadu_si128(slice.as_ptr().add(offset).cast()) }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn loadu128_u8_ptr(ptr: *const u8, offset: usize) -> std::arch::x86_64::__m128i {
    unsafe { std::arch::x86_64::_mm_loadu_si128(ptr.add(offset).cast()) }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn loada128_u8_ptr(ptr: *const u8, offset: usize) -> std::arch::x86_64::__m128i {
    unsafe { std::arch::x86_64::_mm_load_si128(ptr.add(offset).cast()) }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn storeu128_u8(slice: &mut [u8], offset: usize, value: std::arch::x86_64::__m128i) {
    unsafe { std::arch::x86_64::_mm_storeu_si128(slice.as_mut_ptr().add(offset).cast(), value) }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn storeu128_u8_ptr(ptr: *mut u8, offset: usize, value: std::arch::x86_64::__m128i) {
    unsafe { std::arch::x86_64::_mm_storeu_si128(ptr.add(offset).cast(), value) }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn storea128_u8_ptr(ptr: *mut u8, offset: usize, value: std::arch::x86_64::__m128i) {
    unsafe { std::arch::x86_64::_mm_store_si128(ptr.add(offset).cast(), value) }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn loadu128_i32_ptr(ptr: *const i32, offset: usize) -> std::arch::x86_64::__m128i {
    unsafe { std::arch::x86_64::_mm_loadu_si128(ptr.add(offset).cast()) }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn storeu128_i32_ptr(ptr: *mut i32, offset: usize, value: std::arch::x86_64::__m128i) {
    unsafe { std::arch::x86_64::_mm_storeu_si128(ptr.add(offset).cast(), value) }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn i8x4_to_i32x4_ptr(ptr: *const u8, offset: usize) -> std::arch::x86_64::__m128i {
    let p = unsafe { ptr.add(offset) };
    #[cfg(target_feature = "sse4.1")]
    unsafe {
        std::arch::x86_64::_mm_cvtepi8_epi32(std::arch::x86_64::_mm_cvtsi32_si128(
            std::ptr::read_unaligned(p.cast::<i32>()),
        ))
    }
    #[cfg(not(target_feature = "sse4.1"))]
    unsafe {
        std::arch::x86_64::_mm_setr_epi32(
            *p.add(0) as i8 as i32,
            *p.add(1) as i8 as i32,
            *p.add(2) as i8 as i32,
            *p.add(3) as i8 as i32,
        )
    }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn m128i_lane15_u8(value: std::arch::x86_64::__m128i) -> u8 {
    unsafe {
        let shifted = std::arch::x86_64::_mm_srli_si128::<15>(value);
        std::arch::x86_64::_mm_cvtsi128_si32(shifted) as u8
    }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn blendv_epi8_native(
    a: std::arch::x86_64::__m128i,
    b: std::arch::x86_64::__m128i,
    mask: std::arch::x86_64::__m128i,
) -> std::arch::x86_64::__m128i {
    #[cfg(target_feature = "sse4.1")]
    unsafe {
        std::arch::x86_64::_mm_blendv_epi8(a, b, mask)
    }
    #[cfg(not(target_feature = "sse4.1"))]
    unsafe {
        let sign = std::arch::x86_64::_mm_cmpgt_epi8(std::arch::x86_64::_mm_setzero_si128(), mask);
        std::arch::x86_64::_mm_or_si128(
            std::arch::x86_64::_mm_and_si128(sign, b),
            std::arch::x86_64::_mm_andnot_si128(sign, a),
        )
    }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn max_epi8_native(
    a: std::arch::x86_64::__m128i,
    b: std::arch::x86_64::__m128i,
) -> std::arch::x86_64::__m128i {
    #[cfg(target_feature = "sse4.1")]
    unsafe {
        std::arch::x86_64::_mm_max_epi8(a, b)
    }
    #[cfg(not(target_feature = "sse4.1"))]
    unsafe {
        let mask = std::arch::x86_64::_mm_cmpgt_epi8(a, b);
        std::arch::x86_64::_mm_or_si128(
            std::arch::x86_64::_mm_and_si128(mask, a),
            std::arch::x86_64::_mm_andnot_si128(mask, b),
        )
    }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn min_epi8_native(
    a: std::arch::x86_64::__m128i,
    b: std::arch::x86_64::__m128i,
) -> std::arch::x86_64::__m128i {
    #[cfg(target_feature = "sse4.1")]
    unsafe {
        std::arch::x86_64::_mm_min_epi8(a, b)
    }
    #[cfg(not(target_feature = "sse4.1"))]
    unsafe {
        let mask = std::arch::x86_64::_mm_cmpgt_epi8(a, b);
        std::arch::x86_64::_mm_or_si128(
            std::arch::x86_64::_mm_and_si128(mask, b),
            std::arch::x86_64::_mm_andnot_si128(mask, a),
        )
    }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn max_epi32_native(
    a: std::arch::x86_64::__m128i,
    b: std::arch::x86_64::__m128i,
) -> std::arch::x86_64::__m128i {
    #[cfg(target_feature = "sse4.1")]
    unsafe {
        std::arch::x86_64::_mm_max_epi32(a, b)
    }
    #[cfg(not(target_feature = "sse4.1"))]
    unsafe {
        let mask = std::arch::x86_64::_mm_cmpgt_epi32(a, b);
        std::arch::x86_64::_mm_or_si128(
            std::arch::x86_64::_mm_and_si128(mask, a),
            std::arch::x86_64::_mm_andnot_si128(mask, b),
        )
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
    let mut u = crate::NoDropVec::new(take_u8(&EXTD_U, n_vec_bytes.max(1), neg_qe));
    let mut v = crate::NoDropVec::new(take_u8(&EXTD_V, n_vec_bytes.max(1), neg_qe));
    let mut x = crate::NoDropVec::new(take_u8(&EXTD_X, n_vec_bytes.max(1), neg_qe));
    let mut y = crate::NoDropVec::new(take_u8(&EXTD_Y, n_vec_bytes.max(1), neg_qe));
    let mut x2 = crate::NoDropVec::new(take_u8(&EXTD_X2, n_vec_bytes.max(1), neg_qe2));
    let mut y2 = crate::NoDropVec::new(take_u8(&EXTD_Y2, n_vec_bytes.max(1), neg_qe2));
    let mut s = crate::NoDropVec::new(take_u8(&EXTD_S, (n_vec_bytes + 16).max(1), 0));
    let qr_off = n_vec_bytes;
    let mut sf_qr = crate::NoDropVec::new(take_u8(
        &EXTD_SF_QR,
        n_vec_bytes + ((qlen_ + 1) * 16).max(1),
        0,
    ));
    let mut h = if approx_max {
        crate::NoDropVec::new(take_i32(&EXTD_H, 0, KSW_NEG_INF))
    } else {
        crate::NoDropVec::new(take_i32(&EXTD_H, n_vec_bytes.max(1), KSW_NEG_INF))
    };
    let mut p = if with_cigar {
        crate::NoDropVec::new(take_u8_uninit(&EXTD_P, (qlen + tlen - 1) * n_col_ * 16))
    } else {
        crate::NoDropVec::new(take_u8(&EXTD_P, 0, 0))
    };
    let mut off = if with_cigar {
        crate::NoDropVec::new(take_i32_uninit(&EXTD_OFF, qlen + tlen - 1))
    } else {
        crate::NoDropVec::new(take_i32(&EXTD_OFF, 0, 0))
    };
    let mut off_end = if with_cigar {
        crate::NoDropVec::new(take_i32_uninit(&EXTD_OFF_END, qlen + tlen - 1))
    } else {
        crate::NoDropVec::new(take_i32(&EXTD_OFF_END, 0, 0))
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
    #[cfg(not(target_arch = "x86_64"))]
    let sc_mch_v = s2n_lite::_mm_set1_epi8(sc_mch as i32);
    #[cfg(not(target_arch = "x86_64"))]
    let sc_mis_v = s2n_lite::_mm_set1_epi8(sc_mis as i32);
    #[cfg(not(target_arch = "x86_64"))]
    let sc_n_v = s2n_lite::_mm_set1_epi8(sc_n as i32);
    #[cfg(not(target_arch = "x86_64"))]
    let wildcard_v = s2n_lite::_mm_set1_epi8(wildcard as i32);
    #[cfg(not(target_arch = "x86_64"))]
    let zero_v = s2n_lite::_mm_setzero_si128();
    #[cfg(not(target_arch = "x86_64"))]
    let q_v = s2n_lite::_mm_set1_epi8(q_byte as i32);
    #[cfg(not(target_arch = "x86_64"))]
    let q2_v = s2n_lite::_mm_set1_epi8(q2_byte as i32);
    #[cfg(not(target_arch = "x86_64"))]
    let qe_v = s2n_lite::_mm_set1_epi8(qe_byte as i32);
    #[cfg(not(target_arch = "x86_64"))]
    let qe2_v = s2n_lite::_mm_set1_epi8(qe2_byte as i32);
    #[cfg(not(target_arch = "x86_64"))]
    let flag1_v = s2n_lite::_mm_set1_epi8(1);
    #[cfg(not(target_arch = "x86_64"))]
    let flag2_v = s2n_lite::_mm_set1_epi8(2);
    #[cfg(not(target_arch = "x86_64"))]
    let flag3_v = s2n_lite::_mm_set1_epi8(3);
    #[cfg(not(target_arch = "x86_64"))]
    let flag4_v = s2n_lite::_mm_set1_epi8(4);
    #[cfg(not(target_arch = "x86_64"))]
    let flag8_v = s2n_lite::_mm_set1_epi8(0x08);
    #[cfg(not(target_arch = "x86_64"))]
    let flag16_v = s2n_lite::_mm_set1_epi8(0x10);
    #[cfg(not(target_arch = "x86_64"))]
    let flag32_v = s2n_lite::_mm_set1_epi8(0x20);
    #[cfg(not(target_arch = "x86_64"))]
    let flag64_v = s2n_lite::_mm_set1_epi8(0x40);
    #[cfg(target_arch = "x86_64")]
    let (
        sc_mch_v_n,
        sc_mis_v_n,
        sc_n_v_n,
        wildcard_v_n,
        zero_v_n,
        q_v_n,
        q2_v_n,
        qe_v_n,
        qe2_v_n,
        flag1_v_n,
        flag2_v_n,
        flag3_v_n,
        flag4_v_n,
        flag8_v_n,
        flag16_v_n,
        flag32_v_n,
        flag64_v_n,
    ) = unsafe {
        (
            std::arch::x86_64::_mm_set1_epi8(sc_mch as i8),
            std::arch::x86_64::_mm_set1_epi8(sc_mis as i8),
            std::arch::x86_64::_mm_set1_epi8(sc_n as i8),
            std::arch::x86_64::_mm_set1_epi8(wildcard as i8),
            std::arch::x86_64::_mm_setzero_si128(),
            std::arch::x86_64::_mm_set1_epi8(q_byte as i8),
            std::arch::x86_64::_mm_set1_epi8(q2_byte as i8),
            std::arch::x86_64::_mm_set1_epi8(qe_byte as i8),
            std::arch::x86_64::_mm_set1_epi8(qe2_byte as i8),
            std::arch::x86_64::_mm_set1_epi8(1),
            std::arch::x86_64::_mm_set1_epi8(2),
            std::arch::x86_64::_mm_set1_epi8(3),
            std::arch::x86_64::_mm_set1_epi8(4),
            std::arch::x86_64::_mm_set1_epi8(0x08),
            std::arch::x86_64::_mm_set1_epi8(0x10),
            std::arch::x86_64::_mm_set1_epi8(0x20),
            std::arch::x86_64::_mm_set1_epi8(0x40),
        )
    };
    let mut h0 = 0i32;
    let mut last_h0_t = 0i32;
    let mut last_max_h = 0i32;
    let mut last_st = -1i32;
    let mut last_en = -1i32;
    let wl = w;
    let wr = w;
    #[cfg(target_arch = "x86_64")]
    let u_ptr = u.as_mut_ptr();
    #[cfg(target_arch = "x86_64")]
    let v_ptr = v.as_mut_ptr();
    #[cfg(target_arch = "x86_64")]
    let x_ptr = x.as_mut_ptr();
    #[cfg(target_arch = "x86_64")]
    let y_ptr = y.as_mut_ptr();
    #[cfg(target_arch = "x86_64")]
    let x2_ptr = x2.as_mut_ptr();
    #[cfg(target_arch = "x86_64")]
    let y2_ptr = y2.as_mut_ptr();
    #[cfg(target_arch = "x86_64")]
    let s_ptr = s.as_mut_ptr();
    #[cfg(target_arch = "x86_64")]
    let sf_qr_ptr = sf_qr.as_ptr();
    #[cfg(target_arch = "x86_64")]
    let p_ptr = p.as_mut_ptr();

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
                #[cfg(target_arch = "x86_64")]
                unsafe {
                    (
                        *x_ptr.add((st - 1) as usize),
                        *x2_ptr.add((st - 1) as usize),
                        *v_ptr.add((st - 1) as usize),
                    )
                }
                #[cfg(not(target_arch = "x86_64"))]
                {
                    (
                        x[(st - 1) as usize],
                        x2[(st - 1) as usize],
                        v[(st - 1) as usize],
                    )
                }
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
            let boundary = if r == 0 {
                neg_qe
            } else if r_i32 < long_thres {
                (-e_i32) as i8 as u8
            } else if r_i32 == long_thres {
                long_diff as i8 as u8
            } else {
                (-e2_i32) as i8 as u8
            };
            #[cfg(target_arch = "x86_64")]
            unsafe {
                *y_ptr.add(r) = neg_qe;
                *y2_ptr.add(r) = neg_qe2;
                *u_ptr.add(r) = boundary;
            }
            #[cfg(not(target_arch = "x86_64"))]
            {
                y[r] = neg_qe;
                y2[r] = neg_qe2;
                u[r] = boundary;
            }
        }

        if (flag & KSW_EZ_GENERIC_SC) == 0 {
            let mut score_t = st0;
            while score_t <= en0 {
                let t_usize = score_t as usize;
                let q_usize = (qr_off as i32 + qlen as i32 - 1 - r_i32 + score_t) as usize;
                #[cfg(target_arch = "x86_64")]
                unsafe {
                    let sq = loadu128_u8_ptr(sf_qr_ptr, t_usize);
                    let stq = loadu128_u8_ptr(sf_qr_ptr, q_usize);
                    let eq = std::arch::x86_64::_mm_cmpeq_epi8(sq, stq);
                    let sq_wild = std::arch::x86_64::_mm_cmpeq_epi8(sq, wildcard_v_n);
                    let st_wild = std::arch::x86_64::_mm_cmpeq_epi8(stq, wildcard_v_n);
                    let wild = std::arch::x86_64::_mm_or_si128(sq_wild, st_wild);
                    let mut score = blendv_epi8_native(sc_mis_v_n, sc_mch_v_n, eq);
                    score = blendv_epi8_native(score, sc_n_v_n, wild);
                    storeu128_u8_ptr(s_ptr, t_usize, score);
                }
                #[cfg(not(target_arch = "x86_64"))]
                {
                    let sq = unsafe { load16_at(&sf_qr, t_usize) };
                    let stq = unsafe { load16_at(&sf_qr, q_usize) };
                    let eq = s2n_lite::_mm_cmpeq_epi8(sq, stq);
                    let sq_wild = s2n_lite::_mm_cmpeq_epi8(sq, wildcard_v);
                    let st_wild = s2n_lite::_mm_cmpeq_epi8(stq, wildcard_v);
                    let wild = s2n_lite::_mm_or_si128(sq_wild, st_wild);
                    let mut score = s2n_lite::_mm_blendv_epi8(sc_mis_v, sc_mch_v, eq);
                    score = s2n_lite::_mm_blendv_epi8(score, sc_n_v, wild);
                    unsafe { store16_at(&mut s, t_usize, score) };
                }
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
        #[cfg(target_arch = "x86_64")]
        let mut p_row_idx = r * n_col_ * 16;
        let mut x1_lane = x1;
        let mut x21_lane = x21;
        let mut v1_lane = v1;
        for block in st_block..=en_block {
            let base = block as usize * 16;
            #[cfg(target_arch = "x86_64")]
            unsafe {
                let old_x = loada128_u8_ptr(x_ptr, base);
                let old_y = loada128_u8_ptr(y_ptr, base);
                let old_x2 = loada128_u8_ptr(x2_ptr, base);
                let old_y2 = loada128_u8_ptr(y2_ptr, base);
                let old_u = loada128_u8_ptr(u_ptr, base);
                let old_v = loada128_u8_ptr(v_ptr, base);
                let old_x_tail = m128i_lane15_u8(old_x);
                let old_x2_tail = m128i_lane15_u8(old_x2);
                let old_v_tail = m128i_lane15_u8(old_v);
                let s_block = loada128_u8_ptr(s_ptr, base);
                let xt1 = std::arch::x86_64::_mm_or_si128(
                    std::arch::x86_64::_mm_slli_si128::<1>(old_x),
                    std::arch::x86_64::_mm_cvtsi32_si128(x1_lane as i32),
                );
                let x2t1 = std::arch::x86_64::_mm_or_si128(
                    std::arch::x86_64::_mm_slli_si128::<1>(old_x2),
                    std::arch::x86_64::_mm_cvtsi32_si128(x21_lane as i32),
                );
                let vt1 = std::arch::x86_64::_mm_or_si128(
                    std::arch::x86_64::_mm_slli_si128::<1>(old_v),
                    std::arch::x86_64::_mm_cvtsi32_si128(v1_lane as i32),
                );
                let ut = old_u;
                let mut z = s_block;
                let mut a = std::arch::x86_64::_mm_add_epi8(xt1, vt1);
                let mut b = std::arch::x86_64::_mm_add_epi8(old_y, ut);
                let mut a2 = std::arch::x86_64::_mm_add_epi8(x2t1, vt1);
                let mut b2 = std::arch::x86_64::_mm_add_epi8(old_y2, ut);
                let mut d = zero_v_n;
                if !with_cigar {
                    z = max_epi8_native(z, a);
                    z = max_epi8_native(z, b);
                    z = max_epi8_native(z, a2);
                    z = max_epi8_native(z, b2);
                } else if (flag & KSW_EZ_RIGHT) == 0 {
                    d = std::arch::x86_64::_mm_and_si128(
                        std::arch::x86_64::_mm_cmpgt_epi8(a, z),
                        flag1_v_n,
                    );
                    z = max_epi8_native(z, a);
                    let tmp = std::arch::x86_64::_mm_cmpgt_epi8(b, z);
                    d = blendv_epi8_native(d, flag2_v_n, tmp);
                    z = max_epi8_native(z, b);
                    let tmp = std::arch::x86_64::_mm_cmpgt_epi8(a2, z);
                    d = blendv_epi8_native(d, flag3_v_n, tmp);
                    z = max_epi8_native(z, a2);
                    let tmp = std::arch::x86_64::_mm_cmpgt_epi8(b2, z);
                    d = blendv_epi8_native(d, flag4_v_n, tmp);
                    z = max_epi8_native(z, b2);
                } else {
                    d = std::arch::x86_64::_mm_andnot_si128(
                        std::arch::x86_64::_mm_cmpgt_epi8(z, a),
                        flag1_v_n,
                    );
                    z = max_epi8_native(z, a);
                    let tmp = std::arch::x86_64::_mm_cmpgt_epi8(z, b);
                    d = blendv_epi8_native(flag2_v_n, d, tmp);
                    z = max_epi8_native(z, b);
                    let tmp = std::arch::x86_64::_mm_cmpgt_epi8(z, a2);
                    d = blendv_epi8_native(flag3_v_n, d, tmp);
                    z = max_epi8_native(z, a2);
                    let tmp = std::arch::x86_64::_mm_cmpgt_epi8(z, b2);
                    d = blendv_epi8_native(flag4_v_n, d, tmp);
                    z = max_epi8_native(z, b2);
                }
                z = min_epi8_native(z, sc_mch_v_n);
                storea128_u8_ptr(u_ptr, base, std::arch::x86_64::_mm_sub_epi8(z, vt1));
                storea128_u8_ptr(v_ptr, base, std::arch::x86_64::_mm_sub_epi8(z, ut));
                let z_gap = std::arch::x86_64::_mm_sub_epi8(z, q_v_n);
                let z_gap2 = std::arch::x86_64::_mm_sub_epi8(z, q2_v_n);
                a = std::arch::x86_64::_mm_sub_epi8(a, z_gap);
                b = std::arch::x86_64::_mm_sub_epi8(b, z_gap);
                a2 = std::arch::x86_64::_mm_sub_epi8(a2, z_gap2);
                b2 = std::arch::x86_64::_mm_sub_epi8(b2, z_gap2);
                if !with_cigar {
                    storea128_u8_ptr(
                        x_ptr,
                        base,
                        std::arch::x86_64::_mm_sub_epi8(max_epi8_native(a, zero_v_n), qe_v_n),
                    );
                    storea128_u8_ptr(
                        y_ptr,
                        base,
                        std::arch::x86_64::_mm_sub_epi8(max_epi8_native(b, zero_v_n), qe_v_n),
                    );
                    storea128_u8_ptr(
                        x2_ptr,
                        base,
                        std::arch::x86_64::_mm_sub_epi8(max_epi8_native(a2, zero_v_n), qe2_v_n),
                    );
                    storea128_u8_ptr(
                        y2_ptr,
                        base,
                        std::arch::x86_64::_mm_sub_epi8(max_epi8_native(b2, zero_v_n), qe2_v_n),
                    );
                } else if (flag & KSW_EZ_RIGHT) == 0 {
                    let tmp = std::arch::x86_64::_mm_cmpgt_epi8(a, zero_v_n);
                    storea128_u8_ptr(
                        x_ptr,
                        base,
                        std::arch::x86_64::_mm_sub_epi8(
                            std::arch::x86_64::_mm_and_si128(tmp, a),
                            qe_v_n,
                        ),
                    );
                    d = std::arch::x86_64::_mm_or_si128(
                        d,
                        std::arch::x86_64::_mm_and_si128(tmp, flag8_v_n),
                    );
                    let tmp = std::arch::x86_64::_mm_cmpgt_epi8(b, zero_v_n);
                    storea128_u8_ptr(
                        y_ptr,
                        base,
                        std::arch::x86_64::_mm_sub_epi8(
                            std::arch::x86_64::_mm_and_si128(tmp, b),
                            qe_v_n,
                        ),
                    );
                    d = std::arch::x86_64::_mm_or_si128(
                        d,
                        std::arch::x86_64::_mm_and_si128(tmp, flag16_v_n),
                    );
                    let tmp = std::arch::x86_64::_mm_cmpgt_epi8(a2, zero_v_n);
                    storea128_u8_ptr(
                        x2_ptr,
                        base,
                        std::arch::x86_64::_mm_sub_epi8(
                            std::arch::x86_64::_mm_and_si128(tmp, a2),
                            qe2_v_n,
                        ),
                    );
                    d = std::arch::x86_64::_mm_or_si128(
                        d,
                        std::arch::x86_64::_mm_and_si128(tmp, flag32_v_n),
                    );
                    let tmp = std::arch::x86_64::_mm_cmpgt_epi8(b2, zero_v_n);
                    storea128_u8_ptr(
                        y2_ptr,
                        base,
                        std::arch::x86_64::_mm_sub_epi8(
                            std::arch::x86_64::_mm_and_si128(tmp, b2),
                            qe2_v_n,
                        ),
                    );
                    d = std::arch::x86_64::_mm_or_si128(
                        d,
                        std::arch::x86_64::_mm_and_si128(tmp, flag64_v_n),
                    );
                    storea128_u8_ptr(p_ptr, p_row_idx, d);
                } else {
                    let tmp = std::arch::x86_64::_mm_cmpgt_epi8(zero_v_n, a);
                    storea128_u8_ptr(
                        x_ptr,
                        base,
                        std::arch::x86_64::_mm_sub_epi8(
                            std::arch::x86_64::_mm_andnot_si128(tmp, a),
                            qe_v_n,
                        ),
                    );
                    d = std::arch::x86_64::_mm_or_si128(
                        d,
                        std::arch::x86_64::_mm_andnot_si128(tmp, flag8_v_n),
                    );
                    let tmp = std::arch::x86_64::_mm_cmpgt_epi8(zero_v_n, b);
                    storea128_u8_ptr(
                        y_ptr,
                        base,
                        std::arch::x86_64::_mm_sub_epi8(
                            std::arch::x86_64::_mm_andnot_si128(tmp, b),
                            qe_v_n,
                        ),
                    );
                    d = std::arch::x86_64::_mm_or_si128(
                        d,
                        std::arch::x86_64::_mm_andnot_si128(tmp, flag16_v_n),
                    );
                    let tmp = std::arch::x86_64::_mm_cmpgt_epi8(zero_v_n, a2);
                    storea128_u8_ptr(
                        x2_ptr,
                        base,
                        std::arch::x86_64::_mm_sub_epi8(
                            std::arch::x86_64::_mm_andnot_si128(tmp, a2),
                            qe2_v_n,
                        ),
                    );
                    d = std::arch::x86_64::_mm_or_si128(
                        d,
                        std::arch::x86_64::_mm_andnot_si128(tmp, flag32_v_n),
                    );
                    let tmp = std::arch::x86_64::_mm_cmpgt_epi8(zero_v_n, b2);
                    storea128_u8_ptr(
                        y2_ptr,
                        base,
                        std::arch::x86_64::_mm_sub_epi8(
                            std::arch::x86_64::_mm_andnot_si128(tmp, b2),
                            qe2_v_n,
                        ),
                    );
                    d = std::arch::x86_64::_mm_or_si128(
                        d,
                        std::arch::x86_64::_mm_andnot_si128(tmp, flag64_v_n),
                    );
                    storea128_u8_ptr(p_ptr, p_row_idx, d);
                }
                x1_lane = old_x_tail;
                x21_lane = old_x2_tail;
                v1_lane = old_v_tail;
                p_row_idx += 16;
                continue;
            }
            #[cfg(not(target_arch = "x86_64"))]
            {
                let old_x = unsafe { load16_at(&x, base) };
                let old_y = unsafe { load16_at(&y, base) };
                let old_x2 = unsafe { load16_at(&x2, base) };
                let old_y2 = unsafe { load16_at(&y2, base) };
                let old_u = unsafe { load16_at(&u, base) };
                let old_v = unsafe { load16_at(&v, base) };
                let old_x_tail = old_x[15];
                let old_x2_tail = old_x2[15];
                let old_v_tail = old_v[15];
                let s_block = unsafe { load16_at(&s, base) };
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
                unsafe { store16_at(&mut u, base, s2n_lite::_mm_sub_epi8(z, vt1)) };
                unsafe { store16_at(&mut v, base, s2n_lite::_mm_sub_epi8(z, ut)) };
                let z_gap = s2n_lite::_mm_sub_epi8(z, q_v);
                let z_gap2 = s2n_lite::_mm_sub_epi8(z, q2_v);
                a = s2n_lite::_mm_sub_epi8(a, z_gap);
                b = s2n_lite::_mm_sub_epi8(b, z_gap);
                a2 = s2n_lite::_mm_sub_epi8(a2, z_gap2);
                b2 = s2n_lite::_mm_sub_epi8(b2, z_gap2);
                if !with_cigar {
                    unsafe {
                        store16_at(
                            &mut x,
                            base,
                            s2n_lite::_mm_sub_epi8(s2n_lite::_mm_max_epi8(a, zero_v), qe_v),
                        );
                        store16_at(
                            &mut y,
                            base,
                            s2n_lite::_mm_sub_epi8(s2n_lite::_mm_max_epi8(b, zero_v), qe_v),
                        );
                        store16_at(
                            &mut x2,
                            base,
                            s2n_lite::_mm_sub_epi8(s2n_lite::_mm_max_epi8(a2, zero_v), qe2_v),
                        );
                        store16_at(
                            &mut y2,
                            base,
                            s2n_lite::_mm_sub_epi8(s2n_lite::_mm_max_epi8(b2, zero_v), qe2_v),
                        );
                    }
                    let _ = d;
                } else if (flag & KSW_EZ_RIGHT) == 0 {
                    let tmp = s2n_lite::_mm_cmpgt_epi8(a, zero_v);
                    unsafe {
                        store16_at(
                            &mut x,
                            base,
                            s2n_lite::_mm_sub_epi8(s2n_lite::_mm_and_si128(tmp, a), qe_v),
                        )
                    };
                    d = s2n_lite::_mm_or_si128(d, s2n_lite::_mm_and_si128(tmp, flag8_v));
                    let tmp = s2n_lite::_mm_cmpgt_epi8(b, zero_v);
                    unsafe {
                        store16_at(
                            &mut y,
                            base,
                            s2n_lite::_mm_sub_epi8(s2n_lite::_mm_and_si128(tmp, b), qe_v),
                        )
                    };
                    d = s2n_lite::_mm_or_si128(d, s2n_lite::_mm_and_si128(tmp, flag16_v));
                    let tmp = s2n_lite::_mm_cmpgt_epi8(a2, zero_v);
                    unsafe {
                        store16_at(
                            &mut x2,
                            base,
                            s2n_lite::_mm_sub_epi8(s2n_lite::_mm_and_si128(tmp, a2), qe2_v),
                        )
                    };
                    d = s2n_lite::_mm_or_si128(d, s2n_lite::_mm_and_si128(tmp, flag32_v));
                    let tmp = s2n_lite::_mm_cmpgt_epi8(b2, zero_v);
                    unsafe {
                        store16_at(
                            &mut y2,
                            base,
                            s2n_lite::_mm_sub_epi8(s2n_lite::_mm_and_si128(tmp, b2), qe2_v),
                        )
                    };
                    d = s2n_lite::_mm_or_si128(d, s2n_lite::_mm_and_si128(tmp, flag64_v));
                    let p_idx = (r * n_col_ + block as usize - st_block as usize) * 16;
                    unsafe { store16_at(&mut p, p_idx, d) };
                } else {
                    let tmp = s2n_lite::_mm_cmpgt_epi8(zero_v, a);
                    unsafe {
                        store16_at(
                            &mut x,
                            base,
                            s2n_lite::_mm_sub_epi8(s2n_lite::_mm_andnot_si128(tmp, a), qe_v),
                        )
                    };
                    d = s2n_lite::_mm_or_si128(d, s2n_lite::_mm_andnot_si128(tmp, flag8_v));
                    let tmp = s2n_lite::_mm_cmpgt_epi8(zero_v, b);
                    unsafe {
                        store16_at(
                            &mut y,
                            base,
                            s2n_lite::_mm_sub_epi8(s2n_lite::_mm_andnot_si128(tmp, b), qe_v),
                        )
                    };
                    d = s2n_lite::_mm_or_si128(d, s2n_lite::_mm_andnot_si128(tmp, flag16_v));
                    let tmp = s2n_lite::_mm_cmpgt_epi8(zero_v, a2);
                    unsafe {
                        store16_at(
                            &mut x2,
                            base,
                            s2n_lite::_mm_sub_epi8(s2n_lite::_mm_andnot_si128(tmp, a2), qe2_v),
                        )
                    };
                    d = s2n_lite::_mm_or_si128(d, s2n_lite::_mm_andnot_si128(tmp, flag32_v));
                    let tmp = s2n_lite::_mm_cmpgt_epi8(zero_v, b2);
                    unsafe {
                        store16_at(
                            &mut y2,
                            base,
                            s2n_lite::_mm_sub_epi8(s2n_lite::_mm_andnot_si128(tmp, b2), qe2_v),
                        )
                    };
                    d = s2n_lite::_mm_or_si128(d, s2n_lite::_mm_andnot_si128(tmp, flag64_v));
                    let p_idx = (r * n_col_ + block as usize - st_block as usize) * 16;
                    unsafe { store16_at(&mut p, p_idx, d) };
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
            let h_ptr = h.as_mut_ptr();
            let u_ptr = u.as_ptr();
            let v_ptr = v.as_ptr();
            if r > 0 {
                if en0 > 0 {
                    unsafe {
                        *h_ptr.add(en0 as usize) =
                            *h_ptr.add((en0 - 1) as usize) + *u_ptr.add(en0 as usize) as i8 as i32;
                    }
                } else {
                    unsafe {
                        *h_ptr.add(en0 as usize) += *v_ptr.add(en0 as usize) as i8 as i32;
                    }
                }
                max_h_mut = unsafe { *h_ptr.add(en0 as usize) };
                max_t_mut = en0;
                let en1 = st0 + (en0 - st0) / 4 * 4;
                #[cfg(target_arch = "x86_64")]
                let mut max_h_v = unsafe { std::arch::x86_64::_mm_set1_epi32(max_h_mut) };
                #[cfg(target_arch = "x86_64")]
                let mut max_t_v = unsafe { std::arch::x86_64::_mm_set1_epi32(max_t_mut) };
                #[cfg(not(target_arch = "x86_64"))]
                let mut max_h_v = s2n_lite::_mm_set1_epi32(max_h_mut);
                #[cfg(not(target_arch = "x86_64"))]
                let mut max_t_v = s2n_lite::_mm_set1_epi32(max_t_mut);
                let mut t = st0;
                while t < en1 {
                    let base = t as usize;
                    #[cfg(target_arch = "x86_64")]
                    unsafe {
                        let mut h_v = loadu128_i32_ptr(h_ptr, base);
                        let v_v = i8x4_to_i32x4_ptr(v_ptr, base);
                        h_v = std::arch::x86_64::_mm_add_epi32(h_v, v_v);
                        storeu128_i32_ptr(h_ptr, base, h_v);
                        let t_v = std::arch::x86_64::_mm_set1_epi32(t);
                        let gt = std::arch::x86_64::_mm_cmpgt_epi32(h_v, max_h_v);
                        max_h_v = max_epi32_native(max_h_v, h_v);
                        max_t_v = blendv_epi8_native(max_t_v, t_v, gt);
                    }
                    #[cfg(not(target_arch = "x86_64"))]
                    {
                        let mut h_v = unsafe { load_i32x4_at(&h, base) };
                        let v_v = s2n_lite::_mm_setr_epi32(
                            unsafe { *v_ptr.add(base) } as i8 as i32,
                            unsafe { *v_ptr.add(base + 1) } as i8 as i32,
                            unsafe { *v_ptr.add(base + 2) } as i8 as i32,
                            unsafe { *v_ptr.add(base + 3) } as i8 as i32,
                        );
                        h_v = s2n_lite::_mm_add_epi32(h_v, v_v);
                        unsafe { store_i32x4_at(&mut h, base, h_v) };
                        let t_v = s2n_lite::_mm_set1_epi32(t);
                        let gt = s2n_lite::_mm_cmpgt_epi32(h_v, max_h_v);
                        max_h_v = s2n_lite::_mm_blendv_epi8(max_h_v, h_v, gt);
                        max_t_v = s2n_lite::_mm_blendv_epi8(max_t_v, t_v, gt);
                    }
                    t += 4;
                }
                #[cfg(target_arch = "x86_64")]
                let (hh, tt) = unsafe {
                    let mut hh = [0i32; 4];
                    let mut tt = [0i32; 4];
                    std::arch::x86_64::_mm_storeu_si128(hh.as_mut_ptr().cast(), max_h_v);
                    std::arch::x86_64::_mm_storeu_si128(tt.as_mut_ptr().cast(), max_t_v);
                    (hh, tt)
                };
                #[cfg(not(target_arch = "x86_64"))]
                let hh = unsafe { i32x4_to_array(max_h_v) };
                #[cfg(not(target_arch = "x86_64"))]
                let tt = unsafe { i32x4_to_array(max_t_v) };
                for lane in 0..4 {
                    if max_h_mut < hh[lane] {
                        max_h_mut = hh[lane];
                        max_t_mut = tt[lane] + lane as i32;
                    }
                }
                while t < en0 {
                    let k = t as usize;
                    let hk = unsafe {
                        let hp = h_ptr.add(k);
                        *hp += *v_ptr.add(k) as i8 as i32;
                        *hp
                    };
                    if hk > max_h_mut {
                        max_h_mut = hk;
                        max_t_mut = t;
                    }
                    t += 1;
                }
            } else {
                unsafe {
                    *h_ptr = *v_ptr as i8 as i32 - h_qe;
                    max_h_mut = *h_ptr;
                }
                max_t_mut = 0;
            }
            let h_en0 = unsafe { *h_ptr.add(en0 as usize) };
            if en0 == tlen as i32 - 1 && h_en0 > ez.mte {
                ez.mte = h_en0;
                ez.mte_q = r_i32 - en0;
            }
            let h_st0 = unsafe { *h_ptr.add(st0 as usize) };
            if r_i32 - st0 == qlen as i32 - 1 && h_st0 > ez.mqe {
                ez.mqe = h_st0;
                ez.mqe_t = st0;
            }
            if ksw_apply_zdrop(ez, 1, max_h_mut, r_i32, max_t_mut, zdrop, e2) != 0 {
                break;
            }
            if r == qlen + tlen - 2 && en0 == tlen as i32 - 1 {
                ez.score = unsafe { *h_ptr.add(tlen - 1) };
            }
            if (flag & KSW_EZ_EXTZ_ONLY) != 0 {
                let r_h = last_max_h.max(max_h_mut);
                let rq = qlen as i32 - (r_i32 - st0);
                let rt = tlen as i32 - en0;
                let rm = if rq >= tlen as i32 - st0 {
                    tlen as i32 - st0
                } else if rt >= qlen as i32 - (r_i32 - en0) {
                    qlen as i32 - (r_i32 - en0)
                } else {
                    tlen as i32 + qlen as i32 - 1 - r_i32
                };
                if r_h + rm * max_sc + end_bonus < ez.max as i32 {
                    break;
                }
            }
            last_max_h = max_h_mut;
        } else {
            if r > 0 {
                if last_h0_t >= st0 && last_h0_t <= en0 && last_h0_t + 1 >= st0 && last_h0_t < en0 {
                    #[cfg(target_arch = "x86_64")]
                    let (d0, d1) = unsafe {
                        (
                            *v_ptr.add(last_h0_t as usize) as i8 as i32,
                            *u_ptr.add((last_h0_t + 1) as usize) as i8 as i32,
                        )
                    };
                    #[cfg(not(target_arch = "x86_64"))]
                    let (d0, d1) = (
                        v[last_h0_t as usize] as i8 as i32,
                        u[(last_h0_t + 1) as usize] as i8 as i32,
                    );
                    if d0 > d1 {
                        h0 += d0;
                    } else {
                        h0 += d1;
                        last_h0_t += 1;
                    }
                } else if last_h0_t >= st0 && last_h0_t <= en0 {
                    #[cfg(target_arch = "x86_64")]
                    unsafe {
                        h0 += *v_ptr.add(last_h0_t as usize) as i8 as i32;
                    }
                    #[cfg(not(target_arch = "x86_64"))]
                    {
                        h0 += v[last_h0_t as usize] as i8 as i32;
                    }
                } else {
                    last_h0_t += 1;
                    #[cfg(target_arch = "x86_64")]
                    unsafe {
                        h0 += *u_ptr.add(last_h0_t as usize) as i8 as i32;
                    }
                    #[cfg(not(target_arch = "x86_64"))]
                    {
                        h0 += u[last_h0_t as usize] as i8 as i32;
                    }
                }
            } else {
                #[cfg(target_arch = "x86_64")]
                unsafe {
                    h0 = *v_ptr as i8 as i32 - h_qe;
                }
                #[cfg(not(target_arch = "x86_64"))]
                {
                    h0 = v[0] as i8 as i32 - h_qe;
                }
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
    put_u8(&EXTD_U, u.into_inner());
    put_u8(&EXTD_V, v.into_inner());
    put_u8(&EXTD_X, x.into_inner());
    put_u8(&EXTD_Y, y.into_inner());
    put_u8(&EXTD_X2, x2.into_inner());
    put_u8(&EXTD_Y2, y2.into_inner());
    put_u8(&EXTD_S, s.into_inner());
    put_u8(&EXTD_SF_QR, sf_qr.into_inner());
    put_i32(&EXTD_H, h.into_inner());
    put_u8(&EXTD_P, p.into_inner());
    put_i32(&EXTD_OFF, off.into_inner());
    put_i32(&EXTD_OFF_END, off_end.into_inner());
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
            2, -4, -4, -4, -1, -4, 2, -4, -4, -1, -4, -4, 2, -4, -1, -4, -4, -4, 2, -1, -1, -1, -1,
            -1, -1,
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
            2, -4, -4, -4, 0, -4, 2, -4, -4, 0, -4, -4, 2, -4, 0, -4, -4, -4, 2, 0, 0, 0, 0, 0, 0,
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
            2, -4, -4, -4, -1, -4, 2, -4, -4, -1, -4, -4, 2, -4, -1, -4, -4, -4, 2, -1, -1, -1, -1,
            -1, -1,
        ];
        let query = [
            0, 1, 2, 3, 0, 1, 2, 3, 4, 4, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 4, 2, 3, 0, 1, 2, 3, 0, 1,
            2, 4, 4, 0, 1, 2, 3, 0, 1, 2, 3,
        ];
        let target = [
            0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 4, 4, 4, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 4,
            1, 2, 3, 0, 1, 2, 3, 0,
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
            2, -4, -4, -4, -1, -4, 2, -4, -4, -1, -4, -4, 2, -4, -1, -4, -4, -4, 2, -1, -1, -1, -1,
            -1, -1,
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
            2, -4, -4, -4, -1, -4, 2, -4, -4, -1, -4, -4, 2, -4, -1, -4, -4, -4, 2, -1, -1, -1, -1,
            -1, -1,
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
            2, -4, -4, -4, -1, -4, 2, -4, -4, -1, -4, -4, 2, -4, -1, -4, -4, -4, 2, -1, -1, -1, -1,
            -1, -1,
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
