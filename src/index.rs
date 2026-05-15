#![allow(unused_variables, dead_code, non_snake_case)]

use crate::bwt::{
    mb_bwt_destroy, mb_bwt_gen_sa, mb_bwt_init_from_raw, mb_bwt_load, mb_bwt_load_raw, mb_bwt_save,
    mb_bwt_t,
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
        __m128i, _mm_and_si128, _mm_loadu_si128, _mm_set1_epi8, _mm_srli_epi16, _mm_storeu_si128,
        _mm_unpackhi_epi16, _mm_unpackhi_epi8, _mm_unpacklo_epi16, _mm_unpacklo_epi8,
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
        __m128i, _mm_and_si128, _mm_loadu_si128, _mm_set1_epi8, _mm_srli_epi16, _mm_storeu_si128,
        _mm_unpackhi_epi16, _mm_unpackhi_epi8, _mm_unpacklo_epi16, _mm_unpacklo_epi8,
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

unsafe fn sample_sa_i32(a: *const i32, len: usize, sa_shift: usize, step: usize, ssa: &mut [u64]) {
    let ssa_ptr = ssa.as_mut_ptr();
    let mut i = 0usize;
    while i <= len {
        *ssa_ptr.add(i >> sa_shift) = *a.add(i) as u64;
        i += step;
    }
}

unsafe fn sample_sa_i64(a: *const i64, len: usize, sa_shift: usize, step: usize, ssa: &mut [u64]) {
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
    let use_32bit_sa =
        std::env::var_os("MINIBWA_RS_INDEX_32BIT_SA").is_some() && len <= i32::MAX as usize - FS;
    let (primary, mut ssa) = if use_32bit_sa {
        let mut a = vec![0i32; len + FS + 1];
        let rc = libsais_rs::libsais_upstream_c_omp(&seq, &mut a[1..], FS as i32, None, n_thread);
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
        let bwt = run_with_index_pool(n_thread, || mb_bwt_libsais(&l2b, sa_bit, 1, 0, n_thread));
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
