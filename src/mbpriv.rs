#![allow(unused_variables, dead_code, non_snake_case)]

use crate::options::{mb_opt_t, MB_F_ADAP, MB_F_LONG};
use std::sync::atomic::AtomicI32;

pub const MB_DBG_ALN_SEQ: i32 = 0x1;
pub const MB_DBG_ANCHOR: i32 = 0x2;
pub const MB_DBG_SEED: i32 = 0x4;
pub const MB_DBG_QNAME: i32 = 0x8;
pub const MB_DBG_ALN_PE: i32 = 0x10;
pub const MB_DBG_AN_POS: i32 = 0x20;

/// Original C global variable `kom_dbg_flag` from `minibwa/kommon.c:7`.
pub static KOM_DBG_FLAG: AtomicI32 = AtomicI32::new(0);

/// Original C static function `mb_log2` from `minibwa/mbpriv.h:100`.
pub fn mb_log2(x: f32) -> f32 {
    let mut i = x.to_bits();
    let mut log_2 = ((i >> 23) & 255) as f32 - 128.0;
    i &= !(255 << 23);
    i += 127 << 23;
    let f = f32::from_bits(i);
    log_2 += (-0.34484843f32 * f + 2.02466578f32) * f - 0.67487759f32;
    log_2
}

/// Original C static function `mb_hash64` from `minibwa/mbpriv.h:110`.
pub fn mb_hash64(mut x: u64) -> u64 {
    x ^= x >> 30;
    x = x.wrapping_mul(0xbf58476d1ce4e5b9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94d049bb133111eb);
    x ^= x >> 31;
    x
}

/// Original C static function `mb_hash_str` from `minibwa/mbpriv.h:120`.
pub fn mb_hash_str(s: &str) -> u32 {
    let mut h = 2166136261u32;
    for &c in s.as_bytes() {
        h ^= c as u32;
        h = h.wrapping_mul(16777619);
    }
    h
}

/// Original C static function `mb_seq_rev` from `minibwa/mbpriv.h:129`.
pub fn mb_seq_rev(len: u32, seq: &mut [u8]) {
    seq[..len as usize].reverse();
}

/// Original C static function `mb_is_sr_mode` from `minibwa/mbpriv.h:134`.
pub fn mb_is_sr_mode(opt: &mb_opt_t, qlen: i32) -> i32 {
    if (opt.flag & MB_F_LONG) != 0 || ((opt.flag & MB_F_ADAP) != 0 && qlen > opt.max_sr_len) {
        0
    } else {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::{mb_opt_init, mb_opt_preset};

    #[test]
    fn hash_helpers_match_known_vectors() {
        assert_eq!(mb_hash64(0), 0);
        assert_eq!(mb_hash64(1), 0x5692161d100b05e5);
        assert_eq!(mb_hash_str("chrM"), 0xc96c8abb);
    }

    #[test]
    fn seq_rev_reverses_requested_prefix() {
        let mut s = *b"ACGTNN";
        mb_seq_rev(4, &mut s);
        assert_eq!(&s, b"TGCANN");
    }

    #[test]
    fn sr_mode_follows_flags_and_read_length() {
        let mut opt = mb_opt_t::default();
        mb_opt_init(&mut opt);
        assert_eq!(mb_is_sr_mode(&opt, 100), 1);
        assert_eq!(mb_is_sr_mode(&opt, 1000), 0);
        mb_opt_preset(&mut opt, "lr");
        assert_eq!(mb_is_sr_mode(&opt, 100), 0);
    }
}
