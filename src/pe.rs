#![allow(unused_variables, dead_code, non_snake_case, non_camel_case_types)]

use crate::align::{mb_append_cigar, mb_update_extra};
use crate::kommon::KOM_NT4_TABLE;
use crate::ksw2::{
    ksw_extz_t, ksw_gen_nt4_mat, KSW_EZ_EXTZ_ONLY, KSW_EZ_GENERIC_SC, KSW_EZ_REV_CIGAR,
    KSW_EZ_RIGHT, KSW_LL_SUBO,
};
use crate::ksw2_extz2_sse::ksw_extz2_sse;
use crate::ksw2_ll_sse::{ksw_ll_i16_core, ksw_ll_qinit, ksw_ll_u8_core};
use crate::l2bit::{l2b_getseq, l2b_meth_rev, l2b_meth_t, l2b_t};
use crate::lchain::{mb128_t, radix_sort_mb128x};
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
        let size = std::mem::size_of::<mb_extra_t>() + cap as usize * std::mem::size_of::<u32>();
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
                std::ptr::copy_nonoverlapping(values.as_ptr(), self.cigar_mut_ptr(), values.len());
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
            (self.ptr.as_ptr() as *const u8).add(std::mem::size_of::<mb_extra_t>()) as *const u32
        }
    }

    fn cigar_mut_ptr(&mut self) -> *mut u32 {
        unsafe { (self.ptr.as_ptr() as *mut u8).add(std::mem::size_of::<mb_extra_t>()) as *mut u32 }
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
        let size = std::mem::size_of::<mb_extra_t>() + cap as usize * std::mem::size_of::<u32>();
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
    const SEED_RATIO_SHIFT: u32 = 17;
    const SEED_RATIO_MASK: u32 = 0xff << Self::SEED_RATIO_SHIFT;

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

    #[inline(always)]
    pub fn seed_ratio(&self) -> u8 {
        ((self.flags & Self::SEED_RATIO_MASK) >> Self::SEED_RATIO_SHIFT) as u8
    }

    #[inline(always)]
    pub fn set_seed_ratio(&mut self, value: u8) {
        self.flags =
            (self.flags & !Self::SEED_RATIO_MASK) | ((value as u32) << Self::SEED_RATIO_SHIFT);
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
pub fn mb_pair_score(h0: &mb_hit_t, h1: &mb_hit_t, pes: &[mb_pestat_t; 4], match_sc: i32) -> f64 {
    const MB_SQRT1_2: f64 = 0.707106781186547524401;
    let mut dist = 0i64;
    let dir = mb_insert_dir(h0, h1, &mut dist) as usize;
    if pes[dir].failed == 0 && dist >= pes[dir].lo as i64 && dist <= pes[dir].hi as i64 {
        let p0 = h0.p.as_ref().expect("mb_pair_score requires h0->p");
        let p1 = h1.p.as_ref().expect("mb_pair_score requires h1->p");
        let ns = (dist as f64 - pes[dir].avg) / pes[dir].std;
        return p0.dp_max as f64
            + p1.dp_max as f64
            + 0.721 * libm::erfc(ns.abs() * MB_SQRT1_2).mul_add(2.0, 0.0).ln() * match_sc as f64;
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
    let mut pa = Vec::<mb128_t>::with_capacity((n_hit[0] + n_hit[1]) as usize);
    for r in 0..2usize {
        for i in 0..n_hit[r] as usize {
            let h = &mut hit[r][i];
            h.set_proper_pair(0);
            let p = l2b.ctg[h.tid as usize].off + if h.rev() != 0 { h.te } else { h.ts } as u64;
            pa.push(mb128_t {
                x: p,
                y: ((i as u64) << 2) | ((h.rev() as u64) << 1) | r as u64,
            });
        }
    }
    radix_sort_mb128x(&mut pa);
    let mut y = [-1i32; 4];
    let mut pp = Vec::<(u64, u64)>::new();
    for i in 0..pa.len() {
        let pi = pa[i];
        let pi_read = (pi.y & 1) as usize;
        let pi_idx = (pi.y >> 2) as usize;
        let hi_tid = hit[pi_read][pi_idx].tid;
        for r in 0..2usize {
            let dir = (r << 1) | ((pi.y >> 1) as usize & 1);
            if pes[dir].failed != 0 {
                continue;
            }
            let which = (r << 1) | ((pi.y as usize & 1) ^ 1);
            if y[which] < 0 {
                continue;
            }
            let mut k = y[which] as isize;
            while k >= 0 {
                let pk = pa[k as usize];
                if (pk.y & 3) as usize != which {
                    k -= 1;
                    continue;
                }
                let pk_read = (pk.y & 1) as usize;
                let pk_idx = (pk.y >> 2) as usize;
                let hk_tid = hit[pk_read][pk_idx].tid;
                if hi_tid != hk_tid {
                    break;
                }
                let dist = pi.x as i64 - pk.x as i64;
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
                    let yv = if (pk.y & 1) == 0 {
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
        y[(pi.y & 3) as usize] = i as i32;
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

/// A linear algorithm to find ungapped alignment.
///
/// Original C static function `mb_ungap` from `minibwa/pe.c:220`.
pub fn mb_ungap(
    km: (),
    qlen: i32,
    qseq: &[u8],
    tlen: i32,
    tseq: &[u8],
    kmer: i32,
    mt: l2b_meth_t,
    max_i: &mut i32,
    n_good: &mut i32,
    n_kmer: &mut i32,
) -> i32 {
    const C2T: [u8; 4] = [0, 3, 2, 3];
    const G2A: [u8; 4] = [0, 1, 0, 3];
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
            let c = if mt == l2b_meth_t::L2B_METH_C2T {
                C2T[qseq[i] as usize]
            } else if mt == l2b_meth_t::L2B_METH_G2A {
                G2A[qseq[i] as usize]
            } else {
                qseq[i]
            };
            x = ((x << 2) | c as usize) & mask;
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
            let c = if mt == l2b_meth_t::L2B_METH_C2T {
                C2T[tseq[i] as usize]
            } else if mt == l2b_meth_t::L2B_METH_G2A {
                G2A[tseq[i] as usize]
            } else {
                tseq[i]
            };
            x = ((x << 2) | c as usize) & mask;
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
    mt: l2b_meth_t,
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
    ksw_gen_nt4_mat(&mut mat, 1, b_mm as i8, b_ts as i8, b_ambi as i8, mt as i32);
    let sz = if max_sc < 255 - b_mm { 1 } else { 2 };
    let xtra = KSW_LL_SUBO | opt.min_len;
    let qp = ksw_ll_qinit(km, sz, qlen, qseq, 5, &mat);
    let rst = if sz == 1 {
        ksw_ll_u8_core(&qp, tlen, tseq, gapo, gape, xtra)
    } else {
        ksw_ll_i16_core(&qp, tlen, tseq, gapo, gape, xtra)
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
            mt as i32,
        );
        let mut ksw_flag = KSW_EZ_EXTZ_ONLY | KSW_EZ_RIGHT | KSW_EZ_REV_CIGAR;
        if mt != l2b_meth_t::L2B_METH_NONE {
            ksw_flag |= KSW_EZ_GENERIC_SC;
        }
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
            ksw_flag,
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
                p.dp_max2 = ((ez.max as f64 / rst.score as f64) * rst.score2 as f64 + 0.499) as i32;
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
        l2b_getseq(l2b, h0.tid, ts, te, &mut refseq);
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
            mt,
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
                mt,
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
        if n_hit[r] == 0 {
            continue;
        }
        let best = hit[r][0]
            .p
            .as_ref()
            .expect("mb_matesw requires h0[0]->p")
            .dp_max;
        for i in 0..n_hit[r] as usize {
            if m >= opt.max_rescue {
                break;
            }
            let dp = hit[r][i]
                .p
                .as_ref()
                .expect("mb_matesw requires h0[i]->p")
                .dp_max;
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
        if n_hit[r] == 0 {
            continue;
        }
        let best = hit[r][0]
            .p
            .as_ref()
            .expect("mb_matesw requires h0[0]->p")
            .dp_max;
        for i in 0..n_hit[r] as usize {
            if m >= opt.max_rescue {
                break;
            }
            let dp = hit[r][i]
                .p
                .as_ref()
                .expect("mb_matesw requires h0[i]->p")
                .dp_max;
            if hit[r][i].proper_pair() == 0 && dp >= best - opt.pen_unpair * opt.a {
                a.push((
                    ((dp as u64) << 32) | hit[r][i].hash as u64,
                    ((i as u64) << 1) | r as u64,
                ));
                m += 1;
            }
        }
    }
    a.sort_by_key(|&(x, _)| x);
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
        let len = qlen[r].max(0) as usize;
        qs[r][0].resize(len, 0);
        qs[r][1].resize(len, 0);
        let fwd = qs[r][0].as_mut_ptr();
        let rev = qs[r][1].as_mut_ptr();
        for (i, &base) in bases.iter().take(len).enumerate() {
            let c = KOM_NT4_TABLE[base as usize];
            unsafe {
                *fwd.add(i) = c;
                *rev.add(len - 1 - i) = if c < 4 { 3 - c } else { 4 };
            }
        }
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
        let sc = if r == 0 {
            let (left, right) = ha.split_at_mut(1);
            mb_matesw_core(
                km,
                opt,
                l2b,
                pes,
                &left[0].a[j],
                r as i32,
                qlen[1 - r],
                &mut qs[1 - r],
                mt,
                &mut right[0],
                min_sc[1 - r],
                &mut ez,
            )
        } else {
            let (left, right) = ha.split_at_mut(1);
            mb_matesw_core(
                km,
                opt,
                l2b,
                pes,
                &right[0].a[j],
                r as i32,
                qlen[1 - r],
                &mut qs[1 - r],
                mt,
                &mut left[0],
                min_sc[1 - r],
                &mut ez,
            )
        };
        if let Some(sc) = sc {
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
    if n_hit[0] == 0 && n_hit[1] == 0 {
        return;
    }
    let seed_ratio = [
        if n_hit[0] > 0 {
            hit[0][0].seed_ratio()
        } else {
            255
        },
        if n_hit[1] > 0 {
            hit[1][0].seed_ratio()
        } else {
            255
        },
    ];
    let min_seed_ratio = seed_ratio[0].min(seed_ratio[1]);
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
        let mut dp_max_se = [0i32; 2];
        for r in 0..2usize {
            for h in hit[r].iter().take(n_hit[r] as usize) {
                let dp = h.p.as_ref().expect("mb_pair requires hit[r][i]->p").dp_max;
                if dp_max_se[r] < dp {
                    dp_max_se[r] = dp;
                }
            }
        }
        let score_se = dp_max_se[0] + dp_max_se[1];
        if paux.score >= score_se - opt.pen_unpair * opt.a {
            let mut score2 = paux.sub_sc;
            mb_sync_high_cov(n_hit[0], &mut hit[0]);
            mb_sync_high_cov(n_hit[1], &mut hit[1]);
            let identity = (hit[0][i0].mlen + hit[1][i1].mlen) as f64
                / (hit[0][i0].blen + hit[1][i1].blen) as f64;
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
            if min_seed_ratio < 50 {
                let sr = min_seed_ratio as f64;
                mapq_pe = (mapq_pe as f64 * sr * sr / 2500.0) as i32;
            }
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
                    let parent_id = hit[r][old_parent as usize].id;
                    for child in hit[r].iter_mut().take(n_hit[r] as usize) {
                        if child.parent == parent_id {
                            child.parent = new_parent;
                        }
                    }
                    hit[r][old_parent as usize].mapq = 0;
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
    #[should_panic(expected = "mb_pair_score requires h0->p")]
    fn pair_score_requires_alignment_extra_like_c_deref() {
        let h0 = mb_hit_t {
            ts: 100,
            te: 150,
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
        let _ = mb_pair_score(&h0, &h1, &pes, 2);
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
    #[should_panic(expected = "mb_pair requires hit[r][i]->p")]
    fn pair_final_single_end_score_requires_alignment_extra_like_c_deref() {
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
        opt.max_rescue = 0;
        let mut n_hit = [2, 1];
        let mut hit = [
            vec![
                mb_hit_t {
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
                },
                mb_hit_t {
                    tid: 0,
                    ts: 1000,
                    te: 1050,
                    id: 1,
                    parent: 1,
                    hash: 3,
                    p: None,
                    ..Default::default()
                },
            ],
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

        mb_pair(
            (),
            &opt,
            &l2b,
            &mut n_hit,
            &mut hit,
            &pes,
            [12, 12],
            ["ACGTACGTACGT", "TGCATGCATGCA"],
        );
    }

    #[test]
    #[should_panic(expected = "n_pri > 0")]
    fn pair_lift_uses_parent_hit_id_even_when_that_leaves_no_primary_like_c_assert() {
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
        opt.max_rescue = 0;
        let mut n_hit = [3, 1];
        let mut hit = [
            vec![
                mb_hit_t {
                    tid: 0,
                    ts: 800,
                    te: 850,
                    id: 7,
                    parent: 7,
                    mapq: 50,
                    hash: 7,
                    p: Some(
                        mb_extra_t {
                            dp_max: 40,
                            ..Default::default()
                        }
                        .boxed(),
                    ),
                    ..Default::default()
                },
                mb_hit_t {
                    tid: 0,
                    ts: 100,
                    te: 150,
                    id: 42,
                    parent: 0,
                    mlen: 10,
                    blen: 10,
                    hash: 11,
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
                    tid: 0,
                    ts: 900,
                    te: 950,
                    id: 8,
                    parent: 7,
                    hash: 8,
                    p: Some(
                        mb_extra_t {
                            dp_max: 30,
                            ..Default::default()
                        }
                        .boxed(),
                    ),
                    ..Default::default()
                },
            ],
            vec![mb_hit_t {
                tid: 0,
                ts: 300,
                te: 350,
                flags: mb_hit_t::flags_with(1, 0, 0, 0, 0, 0, 0, 0, 0),
                id: 0,
                parent: 0,
                mlen: 10,
                blen: 10,
                hash: 12,
                p: Some(
                    mb_extra_t {
                        dp_max: 100,
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

        mb_pair(
            (),
            &opt,
            &l2b,
            &mut n_hit,
            &mut hit,
            &pes,
            [12, 12],
            ["ACGTACGTACGT", "TGCATGCATGCA"],
        );

        let _ = hit;
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
            l2b_meth_t::L2B_METH_NONE,
            &mut max_i,
            &mut n_good,
            &mut n_kmer,
        );
        assert!(max > 0);
        assert_eq!(max_i, 2);
        assert!(n_good >= 1);
        assert!(n_kmer > 0);
    }

    #[test]
    #[should_panic(expected = "mb_matesw requires h0[0]->p")]
    fn matesw_candidate_selection_requires_alignment_extra_like_c_deref() {
        let l2b = l2b_t {
            n_ctg: 1,
            ctg: vec![crate::l2bit::l2b_ctg_t {
                name: "ctg".into(),
                len: 1000,
                off: 0,
                comm: None,
            }],
            ..Default::default()
        };
        let mut opt = mb_opt_t::default();
        crate::options::mb_opt_init(&mut opt);
        opt.max_rescue = 10;
        opt.pen_unpair = 5;
        let mut n_hit = [1, 1];
        let mut hit = [
            vec![mb_hit_t {
                tid: 0,
                ts: 100,
                te: 150,
                id: 0,
                parent: 0,
                p: None,
                ..Default::default()
            }],
            vec![mb_hit_t {
                tid: 0,
                ts: 300,
                te: 350,
                id: 0,
                parent: 0,
                p: Some(
                    mb_extra_t {
                        dp_max: 80,
                        ..Default::default()
                    }
                    .boxed(),
                ),
                ..Default::default()
            }],
        ];
        let pes = [mb_pestat_t {
            failed: 1,
            ..Default::default()
        }; 4];
        let paux = mb_pairaux_t {
            score: -1,
            sub_sc: -1,
            ..Default::default()
        };

        let n_add = mb_matesw(
            (),
            &opt,
            &l2b,
            &mut n_hit,
            &mut hit,
            &pes,
            &paux,
            [12, 12],
            ["ACGTACGTACGT", "TGCATGCATGCA"],
            0,
        );
    }
}
