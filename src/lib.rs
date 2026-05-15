//! Rust translation of the original minibwa C surface.
//!
//! Source inventory was extracted with `ccc-rs analyze` from the upstream `minibwa` tree.
//! Vendored `libsais` is provided by the local `libsais-rs` dependency. KSW is
//! translated in Rust; C conformance helpers reference the upstream `minibwa` tree.
#![allow(
    clippy::absurd_extreme_comparisons,
    clippy::almost_swapped,
    clippy::approx_constant,
    clippy::collapsible_if,
    clippy::drop_non_drop,
    clippy::excessive_precision,
    clippy::field_reassign_with_default,
    clippy::if_same_then_else,
    clippy::int_plus_one,
    clippy::items_after_test_module,
    clippy::manual_div_ceil,
    clippy::manual_is_multiple_of,
    clippy::manual_pattern_char_comparison,
    clippy::manual_range_contains,
    clippy::missing_safety_doc,
    clippy::missing_transmute_annotations,
    clippy::needless_range_loop,
    clippy::needless_return,
    clippy::ptr_arg,
    clippy::too_many_arguments,
    clippy::unnecessary_map_or,
    clippy::unnecessary_cast
)]

struct NoDropVec<T> {
    inner: std::mem::ManuallyDrop<Vec<T>>,
}

impl<T> NoDropVec<T> {
    #[inline(always)]
    fn new(inner: Vec<T>) -> Self {
        Self {
            inner: std::mem::ManuallyDrop::new(inner),
        }
    }

    #[inline(always)]
    fn into_inner(self) -> Vec<T> {
        std::mem::ManuallyDrop::into_inner(self.inner)
    }
}

impl<T> std::ops::Deref for NoDropVec<T> {
    type Target = Vec<T>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> std::ops::DerefMut for NoDropVec<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[path = "qsufsort.rs"]
pub mod QSufSort;
pub mod align;
pub mod api_test_ex_batch;
pub mod api_test_ex_one;
pub mod bseq;
pub mod bwt;
pub mod bwtgen;
#[cfg(feature = "cli")]
pub mod cli;
pub mod cs;
pub mod fastmap;
pub mod format;
pub mod index;
pub mod kalloc;
pub mod ketopt;
pub mod kommon;
pub mod kommon_cfg_variants;
pub mod ksw2;
pub mod ksw2_extd2_sse;
pub mod ksw2_extz2_sse;
pub mod ksw2_ll_sse;
#[cfg(test)]
mod ksw_helpers_original_c_conformance;
#[cfg(test)]
mod ksw_ll_original_c_conformance;
#[cfg(test)]
mod ksw_original_c_conformance;
pub mod kthread;
pub mod l2bit;
pub mod lchain;
#[path = "main_mod.rs"]
pub mod main;
pub mod map_algo;
pub mod map_main;
pub mod mbpriv;
pub mod options;
pub mod pe;
pub mod s2n_lite;
pub mod seed;
pub mod stage_time;
