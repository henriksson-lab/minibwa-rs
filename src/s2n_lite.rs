
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
        return std::mem::transmute(std::arch::x86::_mm_slli_si128::<IMM8>(std::mem::transmute(
            a,
        )));
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
        return std::mem::transmute(std::arch::x86::_mm_srli_si128::<IMM8>(std::mem::transmute(
            a,
        )));
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
