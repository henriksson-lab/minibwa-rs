
#![allow(unused_variables, dead_code, non_snake_case, non_camel_case_types)]

use std::alloc::{alloc, alloc_zeroed, dealloc, Layout};
use std::collections::HashMap;
use std::ptr;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct header_t {
    pub size: usize,
    pub ptr: *mut header_t,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct km_stat_t {
    pub capacity: usize,
    pub available: usize,
    pub n_blocks: usize,
    pub n_cores: usize,
    pub largest: usize,
}

#[derive(Debug)]
pub struct kmem_t {
    pub par_min_core_size: usize,
    pub min_core_size: usize,
    pub capacity: usize,
    pub active: usize,
    pub largest: usize,
    pub allocations: HashMap<usize, usize>,
}

/// Original C static function `panic` from `minibwa/kalloc.c:32`.
pub fn panic(s: &str) -> ! {
    eprintln!("{s}");
    unsafe { libc::abort() }
}

/// Original C global function `km_init2` from `minibwa/kalloc.c:38`.
pub fn km_init2(km_par: Option<&kmem_t>, min_core_size: usize) -> kmem_t {
    let parent_min = km_par.map(|km| km.min_core_size).unwrap_or(0);
    let min_core_size = if let Some(parent) = km_par {
        if min_core_size > 0 {
            min_core_size
        } else {
            parent.min_core_size.saturating_sub(2)
        }
    } else if min_core_size > 0 {
        min_core_size
    } else {
        0x80000
    };
    kmem_t {
        par_min_core_size: parent_min,
        min_core_size,
        capacity: 0,
        active: 0,
        largest: 0,
        allocations: HashMap::new(),
    }
}

/// Original C global function `km_init` from `minibwa/kalloc.c:48`.
pub fn km_init() -> kmem_t {
    km_init2(None, 0)
}

/// Original C global function `km_destroy` from `minibwa/kalloc.c:50`.
pub unsafe fn km_destroy(km: Option<&mut kmem_t>) {
    let Some(km) = km else {
        return;
    };
    let ptrs = km.allocations.keys().copied().collect::<Vec<_>>();
    for ptr in ptrs {
        kfree(Some(km), ptr as *mut u8);
    }
}

/// Original C static function `morecore` from `minibwa/kalloc.c:65`.
pub fn morecore(km: &mut kmem_t, nu: usize) -> usize {
    let units = (nu + 1 + (km.min_core_size - 1)) / km.min_core_size * km.min_core_size;
    let bytes = units * std::mem::size_of::<usize>() * 2;
    km.capacity += bytes;
    km.largest = km.largest.max(bytes);
    units
}

/// Original C global function `kfree` from `minibwa/kalloc.c:80`.
pub unsafe fn kfree(km: Option<&mut kmem_t>, ap: *mut u8) {
    if ap.is_null() {
        return;
    }
    let header = std::mem::size_of::<usize>();
    let base = ap.sub(header);
    let size = *(base as *const usize);
    let total = size + header;
    let layout = Layout::from_size_align(total.max(1), std::mem::align_of::<usize>()).unwrap();
    dealloc(base, layout);
    if let Some(km) = km {
        km.allocations.remove(&(ap as usize));
        km.active = km.active.saturating_sub(size);
    }
}

/// Original C global function `kmalloc` from `minibwa/kalloc.c:128`.
pub unsafe fn kmalloc(km: Option<&mut kmem_t>, n_bytes: usize) -> *mut u8 {
    if n_bytes == 0 {
        return ptr::null_mut();
    }
    let header = std::mem::size_of::<usize>();
    let total = n_bytes + header;
    let layout = Layout::from_size_align(total, std::mem::align_of::<usize>()).unwrap();
    let base = alloc(layout);
    if base.is_null() {
        return ptr::null_mut();
    }
    *(base as *mut usize) = n_bytes;
    let ap = base.add(header);
    if let Some(km) = km {
        km.capacity += total;
        km.active += n_bytes;
        km.largest = km.largest.max(total);
        km.allocations.insert(ap as usize, n_bytes);
    }
    ap
}

/// Original C global function `kcalloc` from `minibwa/kalloc.c:157`.
pub unsafe fn kcalloc(km: Option<&mut kmem_t>, count: usize, size: usize) -> *mut u8 {
    if size == 0 || count == 0 {
        return ptr::null_mut();
    }
    let n_bytes = count * size;
    let header = std::mem::size_of::<usize>();
    let total = n_bytes + header;
    let layout = Layout::from_size_align(total, std::mem::align_of::<usize>()).unwrap();
    let base = alloc_zeroed(layout);
    if base.is_null() {
        return ptr::null_mut();
    }
    *(base as *mut usize) = n_bytes;
    let ap = base.add(header);
    if let Some(km) = km {
        km.capacity += total;
        km.active += n_bytes;
        km.largest = km.largest.max(total);
        km.allocations.insert(ap as usize, n_bytes);
    }
    ap
}

/// Original C global function `krealloc` from `minibwa/kalloc.c:168`.
pub unsafe fn krealloc(km: Option<&mut kmem_t>, ap: *mut u8, n_bytes: usize) -> *mut u8 {
    if n_bytes == 0 {
        kfree(km, ap);
        return ptr::null_mut();
    }
    if ap.is_null() {
        return kmalloc(km, n_bytes);
    }
    let header = std::mem::size_of::<usize>();
    let base = ap.sub(header);
    let cap = *(base as *const usize);
    if cap >= n_bytes {
        return ap;
    }
    match km {
        Some(km) => {
            let q = kmalloc(Some(km), n_bytes);
            if !q.is_null() {
                ptr::copy_nonoverlapping(ap, q, cap);
                kfree(Some(km), ap);
            }
            q
        }
        None => {
            let q = kmalloc(None, n_bytes);
            if !q.is_null() {
                ptr::copy_nonoverlapping(ap, q, cap);
                kfree(None, ap);
            }
            q
        }
    }
}

/// Original C global function `krelocate` from `minibwa/kalloc.c:187`.
pub unsafe fn krelocate(km: Option<&mut kmem_t>, ap: *mut u8, n_bytes: usize) -> *mut u8 {
    if km.is_none() || ap.is_null() {
        return ap;
    }
    let km = km.unwrap();
    let p = kmalloc(Some(km), n_bytes);
    if !p.is_null() {
        ptr::copy_nonoverlapping(ap, p, n_bytes);
        kfree(Some(km), ap);
    }
    p
}

/// Original C global function `km_stat` from `minibwa/kalloc.c:197`.
pub fn km_stat(km: Option<&kmem_t>, s: &mut km_stat_t) {
    *s = km_stat_t::default();
    let Some(km) = km else {
        return;
    };
    s.capacity = km.capacity;
    s.available = km.capacity.saturating_sub(km.active);
    s.n_blocks = km.allocations.len();
    s.n_cores = if km.capacity == 0 { 0 } else { 1 };
    s.largest = km.largest;
}

/// Original C global function `km_stat_print` from `minibwa/kalloc.c:218`.
pub fn km_stat_print(km: Option<&kmem_t>) -> String {
    let mut st = km_stat_t::default();
    km_stat(km, &mut st);
    format!(
        "[km_stat] cap={}, avail={}, largest={}, n_core={}, n_block={}\n",
        st.capacity, st.available, st.largest, st.n_blocks, st.n_cores
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kalloc_alloc_realloc_relocate_and_stats_track_live_blocks() {
        unsafe {
            let mut km = km_init();
            let p = kmalloc(Some(&mut km), 8);
            assert!(!p.is_null());
            for i in 0..8 {
                *p.add(i) = i as u8;
            }
            let q = krealloc(Some(&mut km), p, 32);
            assert!(!q.is_null());
            for i in 0..8 {
                assert_eq!(*q.add(i), i as u8);
            }
            let r = krelocate(Some(&mut km), q, 16);
            assert!(!r.is_null());
            for i in 0..8 {
                assert_eq!(*r.add(i), i as u8);
            }
            let z = kcalloc(Some(&mut km), 4, 4);
            for i in 0..16 {
                assert_eq!(*z.add(i), 0);
            }
            let mut st = km_stat_t::default();
            km_stat(Some(&km), &mut st);
            assert_eq!(st.n_blocks, 2);
            assert!(st.capacity >= st.available);
            assert!(km_stat_print(Some(&km)).starts_with("[km_stat] cap="));
            kfree(Some(&mut km), r);
            kfree(Some(&mut km), z);
            km_stat(Some(&km), &mut st);
            assert_eq!(st.n_blocks, 0);
        }
    }

    #[test]
    fn kalloc_null_allocator_uses_header_backed_raw_allocations() {
        unsafe {
            let p = kcalloc(None, 3, 5);
            assert!(!p.is_null());
            for i in 0..15 {
                assert_eq!(*p.add(i), 0);
            }
            let q = krealloc(None, p, 24);
            assert!(!q.is_null());
            kfree(None, q);
        }
    }
}
