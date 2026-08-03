#![allow(unused_variables, dead_code, non_snake_case, non_camel_case_types)]

pub const MB_F_PAF: u64 = 0x1;
pub const MB_F_NO_UNMAP: u64 = 0x2;
pub const MB_F_COPY_COMMENT: u64 = 0x4;
pub const MB_F_PE: u64 = 0x8;
pub const MB_F_LONG: u64 = 0x10;
pub const MB_F_EQX: u64 = 0x20;
pub const MB_F_NO_KALLOC: u64 = 0x40;
pub const MB_F_NO_ALN: u64 = 0x80;
pub const MB_F_PE_PREDEF: u64 = 0x100;
pub const MB_F_WRITE_DS: u64 = 0x200;
pub const MB_F_WRITE_CS: u64 = 0x400;
pub const MB_F_WRITE_MD: u64 = 0x800;
pub const MB_F_2ND_SEQ: u64 = 0x1000;
pub const MB_F_SUPP_SOFT: u64 = 0x2000;
pub const MB_F_ADAP: u64 = 0x4000;
pub const MB_F_PRIMARY5: u64 = 0x8000;
pub const MB_F_NO_PAIRING: u64 = 0x10000;
pub const MB_F_METH: u64 = 0x20000;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct mb_opt_t {
    pub flag: u64,
    pub min_len: i32,
    pub max_sub_occ: i32,
    pub max_occ: i32,
    pub bw: i32,
    pub bw_long: i32,
    pub max_gap: i32,
    pub max_sr_len: i32,
    pub max_chain_skip: i32,
    pub max_chain_iter: i32,
    pub min_chain_score: i32,
    pub chain_gap_scale: f32,
    pub mask_level: f32,
    pub mask_len: i32,
    pub pri_ratio: f32,
    pub best_n: i32,
    pub a: i32,
    pub b: i32,
    pub b_ts: i32,
    pub b_ambi: i32,
    pub q: i32,
    pub q2: i32,
    pub e: i32,
    pub e2: i32,
    pub end_bonus: i32,
    pub min_dp_max: i32,
    pub zdrop: i32,
    pub zdrop_inv: i32,
    pub min_ksw_len: i32,
    pub max_pe_ins: i32,
    pub max_rescue: i32,
    pub pen_unpair: i32,
    pub pe_avg: i32,
    pub pe_std: i32,
    pub pe_lo: i32,
    pub pe_hi: i32,
    pub sb_len: i32,
    pub sb_seq: i32,
    pub n_thread: i32,
    pub out_n: i32,
    pub out_s: f32,
    pub seed: i32,
    pub xa_max: i32,
    pub mb_size: i64,
    pub max_mb_size: i64,
    pub max_sw_mat: i64,
    pub cap_kalloc: i64,
}

/// Original C static function `mb_opt_reset` from `minibwa/options.c:5`.
pub fn mb_opt_reset(opt: &mut mb_opt_t) {
    *opt = mb_opt_t::default();
    opt.min_len = 19;
    opt.max_sr_len = 325;
    opt.max_sub_occ = 10;
    opt.max_occ = 250;
    opt.max_chain_skip = 25;
    opt.max_chain_iter = 5000;
    opt.chain_gap_scale = 0.8;
    opt.bw_long = 20000;
    opt.pri_ratio = 0.5;
    opt.mask_level = 0.5;
    opt.mask_len = 0x7fffffff;
    opt.a = 2;
    opt.b = 8;
    opt.q = 12;
    opt.q2 = 23;
    opt.e = 2;
    opt.e2 = 1;
    opt.b_ambi = 1;
    opt.max_pe_ins = 10000;
    opt.max_rescue = 10;
    opt.pen_unpair = 17;
    opt.pe_avg = 400;
    opt.pe_std = 100;
    opt.pe_lo = 50;
    opt.pe_hi = 800;
    opt.sb_len = 1000000;
    opt.sb_seq = 24;
    opt.n_thread = 1;
    opt.seed = 11;
    opt.out_s = 0.8;
    opt.xa_max = 5;
    opt.max_sw_mat = 100000000;
    opt.cap_kalloc = 1i64 << 28;
    opt.max_mb_size = 1000000000;
    opt.flag |= MB_F_NO_KALLOC;
}

/// Original C global function `mb_opt_init` from `minibwa/options.c:46`.
pub fn mb_opt_init(opt: &mut mb_opt_t) {
    mb_opt_reset(opt);
    mb_opt_preset(opt, "adap");
}

/// Original C global function `mb_opt_preset` from `minibwa/options.c:52`.
pub fn mb_opt_preset(opt: &mut mb_opt_t, preset: &str) -> i32 {
    mb_opt_reset(opt);
    let preset = preset
        .as_bytes()
        .iter()
        .position(|&c| c == 0)
        .map(|end| &preset[..end])
        .unwrap_or(preset);
    if preset == "sr" || preset == "adap" {
        opt.flag |= MB_F_PE;
        if preset == "adap" {
            opt.flag |= MB_F_ADAP;
        }
        opt.min_dp_max = 30;
        opt.flag |= MB_F_ADAP;
        opt.bw = 100;
        opt.max_gap = 100;
        opt.zdrop = 80;
        opt.zdrop_inv = 80;
        opt.best_n = 50;
        opt.end_bonus = 10;
        opt.min_chain_score = 25;
        opt.min_ksw_len = 20;
        opt.mb_size = 100000000;
    } else if preset == "lr" {
        opt.flag |= MB_F_LONG;
        opt.flag &= !MB_F_PE;
        opt.min_dp_max = 50;
        opt.bw = 500;
        opt.max_gap = 5000;
        opt.zdrop = 400;
        opt.zdrop_inv = 240;
        opt.best_n = 5;
        opt.end_bonus = -1;
        opt.min_chain_score = 40;
        opt.min_ksw_len = 200;
        opt.mb_size = 500000000;
    } else {
        return -1;
    }
    0
}

/// Original C global function `mb_opt_adap` from `minibwa/options.c:88`.
pub fn mb_opt_adap(opt0: &mb_opt_t, len: i32, opt: &mut mb_opt_t) {
    let min_len = 100;
    let mid_len = 2000;
    *opt = opt0.clone();
    if (opt0.flag & MB_F_ADAP) == 0 {
        return;
    }
    let a = -0.5f64.ln() / (mid_len - min_len) as f64;
    let b = (-a * ((if len > min_len { len } else { min_len }) - min_len) as f64).exp();
    if opt0.max_gap < 5000 {
        opt.max_gap = (5000.0 - (5000 - opt0.max_gap) as f64 * b + 0.499) as i32;
        if opt.max_gap > len {
            opt.max_gap = len;
        }
    }
    if opt0.bw < 500 {
        opt.bw = (500.0 - (500 - opt0.bw) as f64 * b + 0.499) as i32;
    }
    if opt.bw_long > len * 5 {
        opt.bw_long = len * 5;
    }
    if opt.bw_long < opt.bw {
        opt.bw_long = opt.bw;
    }
    if opt0.zdrop < 400 {
        opt.zdrop = (400.0 - (400 - opt0.zdrop) as f64 * b + 0.499) as i32;
    }
    if opt0.zdrop_inv < 240 {
        opt.zdrop_inv = (240.0 - (240 - opt0.zdrop_inv) as f64 * b + 0.499) as i32;
    }
    if opt0.best_n > 5 {
        opt.best_n = ((opt0.best_n - 5) as f64 * b + 5.0 + 0.499) as i32;
    }
    if opt0.min_dp_max < 50 {
        opt.min_dp_max = (50.0 - (50 - opt0.min_dp_max) as f64 * b + 0.499) as i32;
    }
    if opt0.min_chain_score < 40 {
        opt.min_chain_score = (40.0 - (40 - opt0.min_chain_score) as f64 * b + 0.499) as i32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adap_preset_matches_original_defaults() {
        let mut opt = mb_opt_t::default();
        mb_opt_init(&mut opt);
        assert_eq!(
            opt.flag & (MB_F_PE | MB_F_ADAP | MB_F_NO_KALLOC),
            MB_F_PE | MB_F_ADAP | MB_F_NO_KALLOC
        );
        assert_eq!(opt.min_len, 19);
        assert_eq!(opt.bw, 100);
        assert_eq!(opt.max_gap, 100);
        assert_eq!(opt.zdrop, 80);
        assert_eq!(opt.zdrop_inv, 80);
        assert_eq!(opt.best_n, 50);
        assert_eq!(opt.mb_size, 100000000);
    }

    #[test]
    fn lr_preset_matches_original_defaults() {
        let mut opt = mb_opt_t::default();
        assert_eq!(mb_opt_preset(&mut opt, "lr"), 0);
        assert_ne!(opt.flag & MB_F_LONG, 0);
        assert_eq!(opt.flag & MB_F_PE, 0);
        assert_eq!(opt.bw, 500);
        assert_eq!(opt.max_gap, 5000);
        assert_eq!(opt.zdrop, 400);
        assert_eq!(opt.zdrop_inv, 240);
        assert_eq!(opt.best_n, 5);
        assert_eq!(opt.end_bonus, -1);
        assert_eq!(opt.mb_size, 500000000);
    }

    #[test]
    fn preset_names_stop_at_embedded_nul_like_strcmp() {
        let mut opt = mb_opt_t::default();
        assert_eq!(mb_opt_preset(&mut opt, "lr\0hidden"), 0);
        assert_ne!(opt.flag & MB_F_LONG, 0);
        assert_eq!(mb_opt_preset(&mut opt, "adap\0hidden"), 0);
        assert_ne!(opt.flag & MB_F_ADAP, 0);
        assert_eq!(mb_opt_preset(&mut opt, "bad\0lr"), -1);
    }

    #[test]
    fn adaptive_options_change_with_length() {
        let mut base = mb_opt_t::default();
        mb_opt_init(&mut base);
        let mut opt = mb_opt_t::default();
        mb_opt_adap(&base, 2000, &mut opt);
        assert_eq!(opt.max_gap, 2000);
        assert_eq!(opt.bw, 300);
        assert_eq!(opt.zdrop, 240);
        assert_eq!(opt.zdrop_inv, 160);
        assert_eq!(opt.best_n, 27);
        assert_eq!(opt.min_dp_max, 40);
        assert_eq!(opt.min_chain_score, 32);
    }
}
