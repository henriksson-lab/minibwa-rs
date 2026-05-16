#![allow(unused_variables, dead_code, non_snake_case, non_camel_case_types)]

use crate::align::mb_align_skeleton_with_scratch;
use crate::bwt::mb_sai_v;
use crate::bwt::{mb_bwt_cache, mb_bwt_load, mb_bwt_t, mb_sai_t};
use crate::l2bit::{l2b_load, l2b_meth_t, l2b_t};
use crate::lchain::{mb_anchor_t, mb_lchain_dp};
use crate::mbpriv::{
    mb_hash64, mb_hash_str, mb_is_sr_mode, KOM_DBG_FLAG, MB_DBG_ANCHOR, MB_DBG_QNAME, MB_DBG_SEED,
};
use crate::options::{mb_opt_adap, mb_opt_t, MB_F_METH, MB_F_NO_ALN, MB_F_PE, MB_F_PRIMARY5};
use crate::pe::mb_hit_t;
use crate::seed::{
    mb_anchor_sort, mb_anchor_v, mb_anchor_with_scratch, mb_seed_intv, mb_seed_intv_batch,
};
use std::sync::atomic::Ordering;

pub const MB_PARENT_UNSET: i32 = -1;
pub const MB_PARENT_TMP_PRI: i32 = -2;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct mb_idx_s {
    pub is_meth: i32,
    pub l2b: l2b_t,
    pub bwt: mb_bwt_t,
}
pub type mb_idx_t = mb_idx_s;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct mb_tbuf_s {
    pub km: bool,
    pub se_len: Vec<i32>,
    pub se_buf: Vec<u8>,
    pub se_seq_ptrs: Vec<usize>,
    pub se_sai: Vec<mb_sai_v>,
    pub anchor_v: mb_anchor_v,
    pub anchor_aux: Vec<(i64, i64)>,
    pub anchor_sa: Vec<u64>,
    pub anchor_sa_batch: Vec<(u64, u64)>,
    pub anchor_batch: Vec<(i64, i64)>,
    pub chain_w: Vec<u64>,
    pub align_tseq: Vec<u8>,
    pub align_qseq0: Vec<u8>,
}
pub type mb_tbuf_t = mb_tbuf_s;

/// Original C global function `mb_idx_load` from `minibwa/map-algo.c:19`.
pub fn mb_idx_load(prefix: &str, is_meth: i32) -> Option<mb_idx_t> {
    let t0 = std::env::var_os("MINIBWA_RS_LOAD_TIMING").map(|_| std::time::Instant::now());
    let l2b = l2b_load(format!("{prefix}.l2b"))?;
    let t_l2b = t0.map(|t0| {
        let t_l2b = std::time::Instant::now();
        eprintln!(
            "[minibwa-rs load] l2b {:.3} ms",
            (t_l2b - t0).as_secs_f64() * 1000.0
        );
        t_l2b
    });
    let bwt_name = if is_meth != 0 {
        format!("{prefix}.meth.mbw")
    } else {
        format!("{prefix}.mbw")
    };
    let mut bwt = mb_bwt_load(bwt_name)?;
    let t_bwt = t_l2b.map(|t_l2b| {
        let t_bwt = std::time::Instant::now();
        eprintln!(
            "[minibwa-rs load] bwt {:.3} ms",
            (t_bwt - t_l2b).as_secs_f64() * 1000.0
        );
        t_bwt
    });
    mb_bwt_cache(&mut bwt, 10);
    if let (Some(t0), Some(t_bwt)) = (t0, t_bwt) {
        let t_cache = std::time::Instant::now();
        eprintln!(
            "[minibwa-rs load] cache {:.3} ms",
            (t_cache - t_bwt).as_secs_f64() * 1000.0
        );
        eprintln!(
            "[minibwa-rs load] total {:.3} ms",
            (t_cache - t0).as_secs_f64() * 1000.0
        );
    }
    Some(mb_idx_t {
        is_meth: (is_meth != 0) as i32,
        l2b,
        bwt,
    })
}

/// Original C global function `mb_idx_destroy` from `minibwa/map-algo.c:43`.
pub fn mb_idx_destroy(idx: Option<mb_idx_t>) {
    drop(idx);
}

/// Original C global function `mb_idx_ctg_name` from `minibwa/map-algo.c:52`.
pub fn mb_idx_ctg_name(idx: &mb_idx_t, tid: i32) -> Option<&str> {
    if tid >= 0 && tid < idx.l2b.n_ctg as i32 {
        Some(&idx.l2b.ctg[tid as usize].name)
    } else {
        None
    }
}

/// Original C global function `mb_idx_ctg_len` from `minibwa/map-algo.c:57`.
pub fn mb_idx_ctg_len(idx: &mb_idx_t, tid: i32) -> i64 {
    if tid >= 0 && tid < idx.l2b.n_ctg as i32 {
        idx.l2b.ctg[tid as usize].len as i64
    } else {
        -1
    }
}

/// Original C global function `mb_tbuf_init` from `minibwa/map-algo.c:59`.
pub fn mb_tbuf_init(no_kalloc: i32) -> mb_tbuf_t {
    mb_tbuf_t {
        km: no_kalloc == 0,
        se_len: Vec::new(),
        se_buf: Vec::new(),
        se_seq_ptrs: Vec::new(),
        se_sai: Vec::new(),
        anchor_v: mb_anchor_v::default(),
        anchor_aux: Vec::new(),
        anchor_sa: Vec::new(),
        anchor_sa_batch: Vec::new(),
        anchor_batch: Vec::new(),
        chain_w: Vec::new(),
        align_tseq: Vec::new(),
        align_qseq0: Vec::new(),
    }
}

/// Original C global function `mb_tbuf_km` from `minibwa/map-algo.c:67`.
pub fn mb_tbuf_km(b: &mut mb_tbuf_t) -> bool {
    b.km
}

/// Original C global function `mb_tbuf_destroy` from `minibwa/map-algo.c:72`.
pub fn mb_tbuf_destroy(b: Option<mb_tbuf_t>) {
    drop(b);
}

/// Original C global function `mb_tbuf_reset` from `minibwa/map-algo.c:78`.
pub fn mb_tbuf_reset(b: &mut mb_tbuf_t, max_blk_sz: i64) -> i32 {
    if !b.km {
        return 0;
    }
    0
}

/// Original C global function `mb_cal_mblen` from `minibwa/map-algo.c:97`.
pub fn mb_cal_mblen(n: i32, a: &[mb_anchor_t], blen_: &mut i32) -> i32 {
    *blen_ = 0;
    if n <= 0 {
        return 0;
    }
    let mut mlen = a[0].len as i64;
    let mut blen = a[0].len as i64;
    for i in 1..n as usize {
        let span = a[i].len;
        let tl = a[i].tpos as i32 - a[i - 1].tpos as i32;
        let ql = a[i].qpos - a[i - 1].qpos;
        blen += tl.max(ql) as i64;
        mlen += if tl > span && ql > span {
            span
        } else {
            tl.min(ql)
        } as i64;
    }
    *blen_ = blen as i32;
    mlen as i32
}

/// Original C static function `mb_cal_fuzzy_len` from `minibwa/map-algo.c:115`.
pub fn mb_cal_fuzzy_len(r: &mut mb_hit_t, a: &[mb_anchor_t]) {
    r.mlen = mb_cal_mblen(r.cnt, &a[r.as_ as usize..], &mut r.blen);
}

/// Original C static function `mb_hit_set_coor` from `minibwa/map-algo.c:120`.
pub fn mb_hit_set_coor(r: &mut mb_hit_t, qlen: i32, l2b: &l2b_t, a: &[mb_anchor_t]) {
    let k = r.as_ as usize;
    let ak0 = &a[k];
    let ak1 = &a[k + r.cnt as usize - 1];
    r.tid = (ak0.sid >> 1) as i64;
    r.set_rev((ak0.sid & 1) as u8);
    r.ts = ak0.tpos + 1 - ak0.len as i64;
    r.te = ak1.tpos + 1;
    if r.rev() == 0 {
        r.qs = ak0.qpos + 1 - ak0.len;
        r.qe = ak1.qpos + 1;
    } else {
        r.qs = qlen - (ak1.qpos + 1);
        r.qe = qlen - (ak0.qpos + 1 - ak0.len);
    }
    mb_cal_fuzzy_len(r, a);
}

/// Original C global function `mb_cal_high_cov` from `minibwa/map-algo.c:139`.
pub fn mb_cal_high_cov(km: (), n: i32, sai: &[mb_sai_t], max_occ: i32) -> i32 {
    let mut b = Vec::new();
    for x in sai.iter().take(n as usize) {
        if x.size > max_occ as u64 {
            b.push(x.info);
        }
    }
    if b.is_empty() {
        return 0;
    }
    b.sort_unstable();
    let mut hi_st = (b[0] >> 32) as i32;
    let mut hi_en = b[0] as u32 as i32;
    let mut hi_cov = 0;
    for &x in b.iter().skip(1) {
        let st = (x >> 32) as i32;
        let en = x as u32 as i32;
        if st > hi_en {
            hi_cov += hi_en - hi_st;
            hi_st = st;
            hi_en = en;
        } else {
            hi_en = hi_en.max(en);
        }
    }
    hi_cov + hi_en - hi_st
}

/// Original C global function `mb_sync_high_cov` from `minibwa/map-algo.c:165`.
pub fn mb_sync_high_cov(n: i32, h: &mut [mb_hit_t]) {
    let mut max_frac = 0u8;
    for x in h.iter().take(n as usize) {
        max_frac = max_frac.max(x.frac_high());
    }
    for x in h.iter_mut().take(n as usize) {
        x.set_frac_high(max_frac);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l2bit::{l2b_ctg_t, l2b_t};
    use crate::options::{mb_opt_init, mb_opt_t, MB_F_NO_ALN};
    use crate::pe::mb_extra_t;

    #[test]
    fn idx_load_reads_real_chrm_index() {
        let idx = mb_idx_load("minibwa/chrM-human", 0).expect("load index");
        assert_eq!(idx.is_meth, 0);
        assert_eq!(mb_idx_ctg_name(&idx, 0), Some("chrM"));
        assert_eq!(mb_idx_ctg_len(&idx, 0), 16569);
        assert_eq!(mb_idx_ctg_name(&idx, 1), None);
        assert_eq!(mb_idx_ctg_len(&idx, -1), -1);
        assert_eq!(idx.bwt.pre_len, 10);
    }

    #[test]
    fn map_no_aln_finds_real_chrm_hits() {
        let idx = mb_idx_load("minibwa/chrM-human", 0).expect("load index");
        let mut opt = mb_opt_t::default();
        mb_opt_init(&mut opt);
        opt.flag |= MB_F_NO_ALN;
        let seq = "GATCACAGGTCTATCACCCTATTAACCACTCACGGGAGCTCTCCATGCAT";
        let mut n_hit = 0;
        let hits = mb_map(
            &opt,
            &idx,
            seq.len() as i32,
            seq,
            0,
            &mut n_hit,
            None,
            Some("chrM_prefix"),
        );
        assert_eq!(n_hit as usize, hits.len());
        assert!(n_hit > 0);
        assert!(hits.iter().all(|h| h.tid == 0));
        assert!(hits.iter().any(|h| h.qs == 0 && h.qe >= 19));
    }

    #[test]
    fn map_batch_no_aln_matches_single_read_counts() {
        let idx = mb_idx_load("minibwa/chrM-human", 0).expect("load index");
        let mut opt = mb_opt_t::default();
        mb_opt_init(&mut opt);
        opt.flag |= MB_F_NO_ALN;
        opt.sb_seq = 2;
        opt.sb_len = 1000;
        let seqs = [
            "GATCACAGGTCTATCACCCTATTAACCACTCACGGGAGCTCTCCATGCAT",
            "ATCACAGGTCTATCACCCTATTAACCACTCACGGGAGCTCTCCATGCAT",
        ];
        let qlen = seqs.iter().map(|s| s.len() as i32).collect::<Vec<_>>();
        let mut n_hit = vec![0i32; seqs.len()];
        let names = ["r0", "r1"];
        let hits = mb_map_batch(
            &opt,
            &idx,
            seqs.len() as i32,
            &qlen,
            &seqs,
            &mut n_hit,
            None,
            Some(&names),
        );
        assert_eq!(hits.len(), seqs.len());
        for i in 0..seqs.len() {
            let mut n_single = 0;
            let single = mb_map(
                &opt,
                &idx,
                qlen[i],
                seqs[i],
                0,
                &mut n_single,
                None,
                Some(names[i]),
            );
            assert_eq!(n_hit[i], n_single);
            assert_eq!(hits[i].len(), single.len());
        }
    }

    #[test]
    fn thread_buffer_flags_follow_no_kalloc() {
        let mut with_km = mb_tbuf_init(0);
        let mut without_km = mb_tbuf_init(1);
        assert!(mb_tbuf_km(&mut with_km));
        assert!(!mb_tbuf_km(&mut without_km));
        assert_eq!(mb_tbuf_reset(&mut with_km, 1024), 0);
        mb_tbuf_destroy(Some(without_km));
    }

    #[test]
    fn mblen_and_hit_coordinates_match_anchor_chain() {
        let anchors = vec![
            mb_anchor_t {
                sid: 0,
                len: 10,
                qpos: 19,
                tpos: 109,
                ..Default::default()
            },
            mb_anchor_t {
                sid: 0,
                len: 8,
                qpos: 39,
                tpos: 129,
                ..Default::default()
            },
            mb_anchor_t {
                sid: 0,
                len: 12,
                qpos: 59,
                tpos: 149,
                ..Default::default()
            },
        ];
        let mut blen = 0;
        assert_eq!(mb_cal_mblen(anchors.len() as i32, &anchors, &mut blen), 30);
        assert_eq!(blen, 50);

        let l2b = l2b_t {
            tot_len: 1000,
            n_ctg: 1,
            m_ctg: 1,
            ctg: vec![l2b_ctg_t {
                name: "ctg".into(),
                comm: None,
                len: 1000,
                off: 0,
            }],
            ..Default::default()
        };
        let mut hit = mb_hit_t {
            cnt: anchors.len() as i32,
            as_: 0,
            ..Default::default()
        };
        mb_hit_set_coor(&mut hit, 100, &l2b, &anchors);
        assert_eq!((hit.tid, hit.rev(), hit.ts, hit.te), (0, 0, 100, 150));
        assert_eq!((hit.qs, hit.qe, hit.mlen, hit.blen), (10, 60, 30, 50));

        let mut rev = hit.clone();
        let mut rev_anchors = anchors.clone();
        for a in &mut rev_anchors {
            a.sid = 1;
        }
        mb_hit_set_coor(&mut rev, 100, &l2b, &rev_anchors);
        assert_eq!((rev.rev(), rev.qs, rev.qe), (1, 40, 90));
    }

    #[test]
    fn high_cov_intervals_are_merged_and_synced() {
        let sai = vec![
            mb_sai_t {
                size: 20,
                info: (10u64 << 32) | 30,
                ..Default::default()
            },
            mb_sai_t {
                size: 5,
                info: (100u64 << 32) | 110,
                ..Default::default()
            },
            mb_sai_t {
                size: 30,
                info: (25u64 << 32) | 40,
                ..Default::default()
            },
            mb_sai_t {
                size: 40,
                info: (50u64 << 32) | 60,
                ..Default::default()
            },
        ];
        assert_eq!(mb_cal_high_cov((), sai.len() as i32, &sai, 10), 40);

        let mut hits = vec![
            mb_hit_t {
                flags: mb_hit_t::flags_with(0, 0, 0, 0, 0, 0, 0, 0, 3),
                ..Default::default()
            },
            mb_hit_t {
                flags: mb_hit_t::flags_with(0, 0, 0, 0, 0, 0, 0, 0, 9),
                ..Default::default()
            },
            mb_hit_t {
                flags: mb_hit_t::flags_with(0, 0, 0, 0, 0, 0, 0, 0, 1),
                ..Default::default()
            },
        ];
        mb_sync_high_cov(hits.len() as i32, &mut hits);
        assert_eq!(
            hits.iter().map(|h| h.frac_high()).collect::<Vec<_>>(),
            vec![9, 9, 9]
        );
    }

    #[test]
    fn generated_hits_are_sorted_and_coordinate_filled() {
        let l2b = l2b_t {
            tot_len: 1000,
            n_ctg: 1,
            m_ctg: 1,
            ctg: vec![l2b_ctg_t {
                name: "ctg".into(),
                comm: None,
                len: 1000,
                off: 0,
            }],
            ..Default::default()
        };
        let anchors = vec![
            mb_anchor_t {
                sid: 0,
                len: 10,
                qpos: 19,
                tpos: 109,
                ..Default::default()
            },
            mb_anchor_t {
                sid: 0,
                len: 8,
                qpos: 39,
                tpos: 129,
                ..Default::default()
            },
            mb_anchor_t {
                sid: 0,
                len: 12,
                qpos: 79,
                tpos: 179,
                ..Default::default()
            },
        ];
        let u = vec![(50u64 << 32) | 2, (80u64 << 32) | 1];
        let hits = mb_gen_hit((), 17, 100, &l2b, u.len() as i32, &u, &anchors);
        assert_eq!(hits.len(), 2);
        assert!(hits[0].score >= hits[1].score);
        assert_eq!((hits[0].id, hits[0].parent), (0, MB_PARENT_UNSET));
        assert_eq!((hits[0].score, hits[0].score0, hits[0].cnt), (80, 80, 1));
        assert_eq!(
            (hits[0].as_, hits[0].qs, hits[0].qe, hits[0].ts, hits[0].te),
            (2, 68, 80, 168, 180)
        );
        assert_eq!((hits[1].score, hits[1].cnt, hits[1].as_), (50, 2, 0));
    }

    #[test]
    fn split_hit_recalculates_both_halves() {
        let l2b = l2b_t {
            tot_len: 1000,
            n_ctg: 1,
            m_ctg: 1,
            ctg: vec![l2b_ctg_t {
                name: "ctg".into(),
                comm: None,
                len: 1000,
                off: 0,
            }],
            ..Default::default()
        };
        let anchors = vec![
            mb_anchor_t {
                sid: 0,
                len: 10,
                qpos: 19,
                tpos: 109,
                ..Default::default()
            },
            mb_anchor_t {
                sid: 0,
                len: 8,
                qpos: 39,
                tpos: 129,
                ..Default::default()
            },
            mb_anchor_t {
                sid: 0,
                len: 12,
                qpos: 59,
                tpos: 149,
                ..Default::default()
            },
        ];
        let mut left = mb_hit_t {
            id: 7,
            parent: 7,
            cnt: 3,
            score: 99,
            as_: 0,
            flags: mb_hit_t::flags_with(0, 0, 1, 0, 0, 0, 0, 0, 0),
            p: Some(mb_extra_t::default().boxed()),
            ..Default::default()
        };
        mb_hit_set_coor(&mut left, 100, &l2b, &anchors);
        let mut right = mb_hit_t::default();
        mb_split_hit(&mut left, &mut right, 1, 100, &anchors, &l2b);
        assert_eq!(
            (left.cnt, left.score, left.qs, left.qe, left.split()),
            (1, 33, 10, 20, 1)
        );
        assert_eq!(
            (right.id, right.parent, right.cnt, right.score, right.as_),
            (-1, MB_PARENT_TMP_PRI, 2, 66, 1)
        );
        assert_eq!(
            (
                right.sam_pri(),
                right.p.is_none(),
                right.qs,
                right.qe,
                right.split()
            ),
            (0, true, 32, 60, 2)
        );
    }

    #[test]
    fn parent_and_sync_logic_matches_primary_selection() {
        let mut hits = vec![
            mb_hit_t {
                id: 5,
                parent: 5,
                qs: 0,
                qe: 100,
                score: 100,
                ..Default::default()
            },
            mb_hit_t {
                id: 8,
                parent: 5,
                qs: 10,
                qe: 90,
                score: 80,
                ..Default::default()
            },
            mb_hit_t {
                id: 9,
                parent: MB_PARENT_TMP_PRI,
                qs: 120,
                qe: 160,
                score: 40,
                ..Default::default()
            },
        ];
        mb_sync_hits((), hits.len() as i32, &mut hits);
        assert_eq!(
            hits.iter().map(|h| (h.id, h.parent)).collect::<Vec<_>>(),
            vec![(0, 0), (1, 0), (2, 2)]
        );
        assert_eq!(
            hits.iter().map(|h| h.sam_pri()).collect::<Vec<_>>(),
            vec![1, 0, 0]
        );

        mb_set_sam_pri(hits.len() as i32, &mut hits, 1);
        assert_eq!(
            hits.iter().map(|h| h.sam_pri()).collect::<Vec<_>>(),
            vec![1, 0, 0]
        );

        let mut parent_hits = vec![
            mb_hit_t {
                qs: 0,
                qe: 100,
                score: 100,
                parent: MB_PARENT_UNSET,
                ..Default::default()
            },
            mb_hit_t {
                qs: 20,
                qe: 80,
                score: 60,
                parent: MB_PARENT_UNSET,
                ..Default::default()
            },
            mb_hit_t {
                qs: 130,
                qe: 170,
                score: 30,
                parent: MB_PARENT_UNSET,
                ..Default::default()
            },
        ];
        mb_set_parent(
            (),
            0.5,
            10,
            parent_hits.len() as i32,
            &mut parent_hits,
            0,
            0,
        );
        assert_eq!(
            parent_hits
                .iter()
                .map(|h| (h.id, h.parent))
                .collect::<Vec<_>>(),
            vec![(0, 0), (1, 0), (2, 2)]
        );
    }

    #[test]
    fn select_sub_sort_and_filter_compact_hits() {
        let mut hits = vec![
            mb_hit_t {
                id: 0,
                parent: 0,
                qs: 0,
                qe: 100,
                tid: 0,
                ts: 0,
                te: 100,
                score: 100,
                hash: 3,
                ..Default::default()
            },
            mb_hit_t {
                id: 1,
                parent: 0,
                qs: 10,
                qe: 90,
                tid: 0,
                ts: 10,
                te: 90,
                score: 80,
                hash: 1,
                p: Some(
                    mb_extra_t {
                        dp_max: 90,
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
                qe: 100,
                tid: 0,
                ts: 0,
                te: 100,
                score: 70,
                hash: 9,
                ..Default::default()
            },
        ];
        let mut n = hits.len() as i32;
        mb_select_sub((), 0.5, 0, 1, &mut n, &mut hits);
        assert_eq!(n, 2);
        assert_eq!(
            hits.iter().map(|h| (h.id, h.parent)).collect::<Vec<_>>(),
            vec![(0, 0), (1, 0)]
        );

        hits[0].score = 30;
        hits[1].score = 20;
        hits[1].p = Some(
            mb_extra_t {
                dp_max: 120,
                ..Default::default()
            }
            .boxed(),
        );
        mb_hit_sort((), &mut n, &mut hits);
        assert_eq!(hits.iter().map(|h| h.hash).collect::<Vec<_>>(), vec![1, 3]);

        let mut opt = mb_opt_t::default();
        mb_opt_init(&mut opt);
        opt.min_chain_score = 40;
        opt.min_dp_max = 50;
        opt.a = 2;
        hits[0].mlen = 100;
        hits[1].mlen = 10;
        hits[1].p = Some(
            mb_extra_t {
                dp_max: 200,
                ..Default::default()
            }
            .boxed(),
        );
        mb_filter_hits(&opt, 100, &mut n, &mut hits);
        assert_eq!(n, 1);
        assert_eq!(hits[0].hash, 1);
    }

    #[test]
    fn squeeze_anchors_updates_hit_offsets_in_offset_order() {
        let mut anchors = vec![
            mb_anchor_t {
                qpos: 0,
                ..Default::default()
            },
            mb_anchor_t {
                qpos: 1,
                ..Default::default()
            },
            mb_anchor_t {
                qpos: 20,
                ..Default::default()
            },
            mb_anchor_t {
                qpos: 21,
                ..Default::default()
            },
            mb_anchor_t {
                qpos: 40,
                ..Default::default()
            },
        ];
        let mut hits = vec![
            mb_hit_t {
                as_: 4,
                cnt: 1,
                ..Default::default()
            },
            mb_hit_t {
                as_: 2,
                cnt: 2,
                ..Default::default()
            },
        ];
        let n = mb_squeeze_a((), hits.len() as i32, &mut hits, &mut anchors);
        assert_eq!(n, 3);
        assert_eq!(hits.iter().map(|h| h.as_).collect::<Vec<_>>(), vec![2, 0]);
        assert_eq!(
            anchors.iter().take(3).map(|a| a.qpos).collect::<Vec<_>>(),
            vec![20, 21, 40]
        );
    }

    #[test]
    fn mapq_sets_primary_secondary_and_inversion_neighbor_quality() {
        let mut hits = vec![
            mb_hit_t {
                id: 0,
                parent: 0,
                score: 100,
                score0: 100,
                subsc: 40,
                mlen: 90,
                blen: 100,
                p: Some(
                    mb_extra_t {
                        dp_max: 120,
                        dp_max2: 60,
                        ..Default::default()
                    }
                    .boxed(),
                ),
                tid: 0,
                ts: 10,
                ..Default::default()
            },
            mb_hit_t {
                id: 1,
                parent: 0,
                score: 70,
                score0: 70,
                tid: 0,
                ts: 30,
                ..Default::default()
            },
            mb_hit_t {
                id: 2,
                parent: 2,
                flags: mb_hit_t::flags_with(0, 0, 0, 0, 1, 0, 0, 0, 0),
                tid: 0,
                ts: 20,
                ..Default::default()
            },
            mb_hit_t {
                id: 3,
                parent: 3,
                score: 90,
                score0: 90,
                subsc: 40,
                mlen: 85,
                blen: 100,
                p: Some(
                    mb_extra_t {
                        dp_max: 100,
                        dp_max2: 40,
                        ..Default::default()
                    }
                    .boxed(),
                ),
                tid: 0,
                ts: 40,
                ..Default::default()
            },
        ];
        mb_set_mapq((), 150, hits.len() as i32, &mut hits, 40, 2, 1, 325);
        assert!(hits[0].mapq > 0);
        assert_eq!(hits[1].mapq, 0);
        assert_eq!(hits[2].mapq, hits[0].mapq.min(hits[3].mapq));
    }
}

/// Original C global function `mb_gen_hit` from `minibwa/map-algo.c:174`.
pub fn mb_gen_hit(
    km: (),
    hash: u32,
    qlen: i32,
    l2b: &l2b_t,
    n_u: i32,
    u: &[u64],
    a: &[mb_anchor_t],
) -> Vec<mb_hit_t> {
    if n_u <= 0 {
        return Vec::new();
    }
    let mut z = Vec::with_capacity(n_u as usize);
    let mut k = 0usize;
    for &ui in u.iter().take(n_u as usize) {
        let h = (mb_hash64(
            (mb_hash64(a[k].tpos as u64).wrapping_add(mb_hash64(a[k].qpos as u64))) ^ hash as u64,
        ) & 0xffff_ffff) as u32;
        let x = ui ^ h as u64;
        let y = ((k as u64) << 32) | (ui as u32 as u64);
        z.push((x, y));
        k += ui as u32 as usize;
    }
    z.sort_by_key(|&(x, _)| x);
    z.reverse();

    let mut r = Vec::with_capacity(n_u as usize);
    for (i, &(x, y)) in z.iter().enumerate() {
        let mut ri = mb_hit_t {
            id: i as i32,
            parent: MB_PARENT_UNSET,
            score: (x >> 32) as i32,
            score0: (x >> 32) as i32,
            hash: x as u32,
            cnt: y as u32 as i32,
            as_: (y >> 32) as i32,
            ..Default::default()
        };
        mb_hit_set_coor(&mut ri, qlen, l2b, a);
        r.push(ri);
    }
    r
}

/// Original C global function `mb_split_hit` from `minibwa/map-algo.c:211`.
pub fn mb_split_hit(
    r: &mut mb_hit_t,
    r2: &mut mb_hit_t,
    n: i32,
    qlen: i32,
    a: &[mb_anchor_t],
    l2b: &l2b_t,
) {
    if n <= 0 || n >= r.cnt {
        return;
    }
    *r2 = r.clone();
    r2.id = -1;
    r2.set_sam_pri(0);
    r2.p = None;
    r2.set_split_inv(0);
    r2.cnt = r.cnt - n;
    r2.score = ((r.score as f32 * (r2.cnt as f32 / r.cnt as f32)) + 0.499) as i32;
    r2.as_ = r.as_ + n;
    if r.parent == r.id {
        r2.parent = MB_PARENT_TMP_PRI;
    }
    mb_hit_set_coor(r2, qlen, l2b, a);
    r.cnt -= r2.cnt;
    r.score -= r2.score;
    mb_hit_set_coor(r, qlen, l2b, a);
    r.set_split(r.split() | 1);
    r2.set_split(r2.split() | 2);
}

/// Original C global function `mb_sync_hits` from `minibwa/map-algo.c:230`.
pub fn mb_sync_hits(km: (), n_regs: i32, regs: &mut [mb_hit_t]) {
    if n_regs <= 0 {
        return;
    }
    let n = n_regs as usize;
    let mut max_id = -1;
    for r in regs.iter().take(n) {
        max_id = max_id.max(r.id);
    }
    let n_tmp = max_id + 1;
    let mut tmp = vec![-1i32; n_tmp.max(0) as usize];
    for (i, r) in regs.iter().take(n).enumerate() {
        if r.id >= 0 {
            tmp[r.id as usize] = i as i32;
        }
    }
    for i in 0..n {
        let old_parent = regs[i].parent;
        regs[i].id = i as i32;
        if old_parent == MB_PARENT_TMP_PRI {
            regs[i].parent = i as i32;
        } else if old_parent >= 0
            && (old_parent as usize) < tmp.len()
            && tmp[old_parent as usize] >= 0
        {
            regs[i].parent = tmp[old_parent as usize];
        } else {
            regs[i].parent = MB_PARENT_UNSET;
        }
    }
    mb_set_sam_pri(n_regs, regs, 0);
}

/// Original C static function `update_sub` from `minibwa/map-algo.c:258`.
pub fn update_sub(
    ri: &mut mb_hit_t,
    rp: &mut mb_hit_t,
    mask_level: f32,
    mask_len: i32,
    sub_diff: i32,
    uncov_len: i32,
) -> i32 {
    let si = ri.qs;
    let ei = ri.qe;
    let sj = rp.qs;
    let ej = rp.qe;
    if ej <= si || sj >= ei {
        return 0;
    }
    let min = (ej - sj).min(ei - si);
    let max = (ej - sj).max(ei - si);
    let ol = ei.min(ej) - si.max(sj);
    if ol as f64 / min as f64 - uncov_len as f64 / max as f64 > mask_level as f64
        && uncov_len <= mask_len
    {
        let mut cnt_sub = 0;
        let mut sci = ri.score;
        ri.parent = rp.parent;
        rp.subsc = rp.subsc.max(sci);
        if rp.p.is_some()
            && ri.p.is_some()
            && (rp.tid != ri.tid || rp.ts != ri.ts || rp.te != ri.te || ol != min)
        {
            let ri_dp_max = ri.p.as_ref().unwrap().dp_max;
            let rp_extra = rp.p.as_mut().unwrap();
            sci = ri_dp_max;
            rp_extra.dp_max2 = rp_extra.dp_max2.max(sci);
            if rp_extra.dp_max - ri_dp_max <= sub_diff {
                cnt_sub = 1;
            }
        }
        if cnt_sub != 0 {
            rp.n_sub += 1;
        }
        1
    } else {
        0
    }
}

/// Original C global function `mb_set_parent` from `minibwa/map-algo.c:279`.
pub fn mb_set_parent(
    km: (),
    mask_level: f32,
    mask_len: i32,
    n: i32,
    r: &mut [mb_hit_t],
    sub_diff: i32,
    hard_mask_level: i32,
) {
    if n <= 0 {
        return;
    }
    let n = n as usize;
    for (i, ri) in r.iter_mut().take(n).enumerate() {
        ri.id = i as i32;
    }
    let mut w = vec![0i32; n];
    let mut k = 1usize;
    r[0].parent = 0;
    for i in 1..n {
        let si = r[i].qs;
        let ei = r[i].qe;
        let mut cov = Vec::new();
        let mut uncov_len = 0i32;
        if hard_mask_level == 0 {
            for &wj in w.iter().take(k) {
                let rp = &r[wj as usize];
                let mut sj = rp.qs;
                let mut ej = rp.qe;
                if ej <= si || sj >= ei {
                    continue;
                }
                if sj < si {
                    sj = si;
                }
                if ej > ei {
                    ej = ei;
                }
                cov.push((sj as u64) << 32 | ej as u32 as u64);
            }
            if cov.is_empty() {
                w[k] = i as i32;
                k += 1;
                r[i].parent = i as i32;
                r[i].n_sub = 0;
                continue;
            }
            cov.sort_unstable();
            let mut x = si;
            for &cv in &cov {
                let st = (cv >> 32) as i32;
                let en = cv as u32 as i32;
                if st > x {
                    uncov_len += st - x;
                }
                x = x.max(en);
            }
            if ei > x {
                uncov_len += ei - x;
            }
        }

        let mut max_ol = 0;
        let mut max_j = -1i32;
        for (j, &wj) in w.iter().take(k).enumerate() {
            let rp = &r[wj as usize];
            let sj = rp.qs;
            let ej = rp.qe;
            let ol = if ej <= si || sj >= ei {
                0
            } else {
                ei.min(ej) - si.max(sj)
            };
            if max_ol < ol {
                max_ol = ol;
                max_j = j as i32;
            }
        }
        let mut n_par = 0;
        if max_j >= 0 {
            let wi = w[max_j as usize] as usize;
            if wi < i {
                let (left, right) = r.split_at_mut(i);
                n_par += update_sub(
                    &mut right[0],
                    &mut left[wi],
                    mask_level,
                    mask_len,
                    sub_diff,
                    uncov_len,
                );
            } else {
                let (left, right) = r.split_at_mut(wi);
                n_par += update_sub(
                    &mut left[i],
                    &mut right[0],
                    mask_level,
                    mask_len,
                    sub_diff,
                    uncov_len,
                );
            }
            let mut j = 0usize;
            while j < k && n_par == 0 {
                let wi = w[j] as usize;
                if wi < i {
                    let (left, right) = r.split_at_mut(i);
                    n_par += update_sub(
                        &mut right[0],
                        &mut left[wi],
                        mask_level,
                        mask_len,
                        sub_diff,
                        uncov_len,
                    );
                } else {
                    let (left, right) = r.split_at_mut(wi);
                    n_par += update_sub(
                        &mut left[i],
                        &mut right[0],
                        mask_level,
                        mask_len,
                        sub_diff,
                        uncov_len,
                    );
                }
                j += 1;
            }
        }
        if n_par == 0 {
            w[k] = i as i32;
            k += 1;
            r[i].parent = i as i32;
            r[i].n_sub = 0;
        }
    }
}

/// Original C global function `mb_set_sam_pri` from `minibwa/map-algo.c:330`.
pub fn mb_set_sam_pri(n: i32, r: &mut [mb_hit_t], is_primary5: i32) {
    if n <= 0 {
        return;
    }
    let mut n_pri = 0;
    let mut min_i = -1;
    let mut min_qs = -1;
    let mut first_i = -1;
    for (i, ri) in r.iter_mut().take(n as usize).enumerate() {
        ri.set_sam_pri(0);
        if ri.id != ri.parent {
            continue;
        }
        n_pri += 1;
        if n_pri == 1 {
            first_i = i as i32;
        }
        if min_qs < 0 || ri.qs < min_qs {
            min_i = i as i32;
            min_qs = ri.qs;
        }
    }
    debug_assert!(n_pri > 0);
    if is_primary5 != 0 {
        r[min_i as usize].set_sam_pri(1);
    } else {
        r[first_i as usize].set_sam_pri(1);
    }
}

/// Original C global function `mb_select_sub` from `minibwa/map-algo.c:346`.
pub fn mb_select_sub(
    km: (),
    pri_ratio: f32,
    min_diff: i32,
    best_n: i32,
    n_: &mut i32,
    r: &mut Vec<mb_hit_t>,
) {
    if pri_ratio > 0.0 && *n_ > 0 {
        let n = *n_ as usize;
        let mut keep = vec![0u8; n];
        let mut n_2nd = 0;
        for i in 0..n {
            let p = r[i].parent;
            if p == i as i32 || r[i].inv() != 0 {
                keep[i] = 1;
            } else if p >= 0
                && (p as usize) < r.len()
                && ((r[i].score as f32) >= r[p as usize].score as f32 * pri_ratio
                    || r[i].score + min_diff >= r[p as usize].score)
                && n_2nd < best_n
                && !(r[i].qs == r[p as usize].qs
                    && r[i].qe == r[p as usize].qe
                    && r[i].tid == r[p as usize].tid
                    && r[i].ts == r[p as usize].ts
                    && r[i].te == r[p as usize].te)
            {
                keep[i] = 1;
                n_2nd += 1;
            }
        }
        let mut k = 0usize;
        for i in 0..n {
            if keep[i] != 0 {
                if k < i {
                    r[k] = std::mem::take(&mut r[i]);
                }
                k += 1;
            } else if r[i].p.is_some() {
                r[i].p = None;
            }
        }
        if k != n {
            r.truncate(k);
            mb_sync_hits(km, k as i32, r);
        }
        *n_ = k as i32;
    }
}

/// Original C global function `mb_hit_sort` from `minibwa/map-algo.c:366`.
pub fn mb_hit_sort(km: (), n_regs: &mut i32, r: &mut Vec<mb_hit_t>) {
    let n = *n_regs as usize;
    if n <= 1 {
        return;
    }
    let mut aux = Vec::new();
    for i in 0..n {
        if r[i].inv() != 0 || r[i].cnt >= 0 {
            let score = r[i].p.as_ref().map(|p| p.dp_max).unwrap_or(r[i].score);
            aux.push((((score as u64) << 32) | r[i].hash as u64, i));
        } else if r[i].p.is_some() {
            r[i].p = None;
        }
    }
    aux.sort_by_key(|&(x, _)| x);
    let mut old = std::mem::take(r);
    let mut t = Vec::with_capacity(aux.len());
    for &(_, idx) in aux.iter().rev() {
        t.push(std::mem::take(&mut old[idx]));
    }
    *n_regs = t.len() as i32;
    *r = t;
}

/// Original C global function `mb_filter_hits` from `minibwa/map-algo.c:394`.
pub fn mb_filter_hits(opt: &mb_opt_t, qlen: i32, n_regs: &mut i32, regs: &mut Vec<mb_hit_t>) {
    let mut k = 0usize;
    for i in 0..*n_regs as usize {
        let mut flt = regs[i].flt() != 0;
        if regs[i].p.is_some() {
            let dp_max = regs[i].p.as_ref().unwrap().dp_max;
            if regs[i].mlen < opt.min_chain_score {
                flt = true;
            } else if dp_max < opt.min_dp_max * opt.a {
                flt = true;
            }
            if flt {
                regs[i].p = None;
            }
        }
        if !flt {
            if k < i {
                regs[k] = std::mem::take(&mut regs[i]);
            }
            k += 1;
        }
    }
    regs.truncate(k);
    *n_regs = k as i32;
}

/// Original C global function `mb_squeeze_a` from `minibwa/map-algo.c:413`.
pub fn mb_squeeze_a(km: (), n_regs: i32, regs: &mut [mb_hit_t], a: &mut [mb_anchor_t]) -> i32 {
    let mut aux = Vec::with_capacity(n_regs as usize);
    for (i, r) in regs.iter().take(n_regs as usize).enumerate() {
        aux.push(((r.as_ as u64) << 32) | i as u32 as u64);
    }
    aux.sort_unstable();
    let mut as_ = 0usize;
    for &x in &aux {
        let i = x as u32 as usize;
        let r = &mut regs[i];
        if r.as_ as usize != as_ {
            let src = r.as_ as usize;
            let cnt = r.cnt as usize;
            a.copy_within(src..src + cnt, as_);
            r.as_ = as_ as i32;
        }
        as_ += r.cnt as usize;
    }
    as_ as i32
}

/// Original C static function `mb_set_inv_mapq` from `minibwa/map-algo.c:437`.
pub fn mb_set_inv_mapq(km: (), n_regs: i32, regs: &mut [mb_hit_t]) {
    let n = n_regs as usize;
    if n_regs < 3 {
        return;
    }
    if !regs.iter().take(n).any(|r| r.inv() != 0) {
        return;
    }
    let mut aux = Vec::new();
    for (i, r) in regs.iter().take(n).enumerate() {
        if r.parent == i as i32 || r.parent < 0 {
            aux.push((((r.tid as u64) << 32) | r.ts as u64, i));
        }
    }
    aux.sort_by_key(|&(x, _)| x);
    for i in 1..aux.len().saturating_sub(1) {
        let inv_i = aux[i].1;
        if regs[inv_i].inv() != 0 {
            let l_mapq = regs[aux[i - 1].1].mapq;
            let r_mapq = regs[aux[i + 1].1].mapq;
            regs[inv_i].mapq = l_mapq.min(r_mapq);
        }
    }
}

/// Original C global function `mb_set_mapq` from `minibwa/map-algo.c:463`.
pub fn mb_set_mapq(
    km: (),
    qlen: i32,
    n_regs: i32,
    regs: &mut [mb_hit_t],
    min_chain_sc: i32,
    match_sc: i32,
    is_sr: i32,
    max_sr_len: i32,
) {
    const MAPQ_COEF_LEN: i32 = 50;
    const MAPQ_COEF_FAC: f64 = 3.0;
    const Q_COEF: f64 = 40.0;
    if n_regs == 0 {
        return;
    }
    for r in regs.iter_mut().take(n_regs as usize) {
        if r.inv() != 0 {
            r.mapq = 0;
        } else if r.parent == r.id {
            let subsc = r.subsc.max(min_chain_sc);
            let pen_chn = if (r.score as f64) > qlen as f64 * 0.1 {
                1.0
            } else {
                10.0 * r.score as f64 / qlen as f64
            };
            let mut mapq;
            if let Some(p) = &r.p {
                if p.dp_max2 > 0 && p.dp_max > 0 {
                    let identity = r.mlen as f64 / r.blen as f64;
                    let mut x = if r.blen < MAPQ_COEF_LEN {
                        1.0
                    } else {
                        MAPQ_COEF_FAC / (r.blen as f64).ln()
                    };
                    x *= identity * identity;
                    let mapq_sr = (6.02 * x * x * (p.dp_max - p.dp_max2) as f64 / match_sc as f64
                        + 0.499) as i32;
                    x = p.dp_max2 as f64 / p.dp_max as f64;
                    if subsc > r.score0 {
                        x *= subsc as f64 / r.score0 as f64;
                    }
                    let mapq_lr = (pen_chn
                        * identity
                        * Q_COEF
                        * (1.0 - x * x)
                        * (p.dp_max as f64 / match_sc as f64).ln())
                        as i32;
                    if is_sr != 0 {
                        mapq = mapq_sr;
                    } else if max_sr_len < 0 {
                        mapq = mapq_lr;
                    } else {
                        mapq = if qlen < max_sr_len {
                            mapq_sr
                        } else {
                            (mapq_lr as f64
                                - (mapq_lr - mapq_sr) as f64
                                    * 2.0f64.powf(1.0 - qlen as f64 / max_sr_len as f64)
                                + 0.499) as i32
                        };
                    }
                } else {
                    let x = subsc as f64 / r.score0 as f64;
                    let identity = r.mlen as f64 / r.blen as f64;
                    mapq = (pen_chn
                        * identity
                        * Q_COEF
                        * (1.0 - x)
                        * (p.dp_max as f64 / match_sc as f64).ln())
                        as i32;
                }
            } else {
                let x = subsc as f64 / r.score0 as f64;
                mapq = (pen_chn * Q_COEF * (1.0 - x) * (r.score as f64).ln()) as i32;
            }
            mapq -= (4.343f64 * ((r.n_sub + 1) as f64).ln() + 0.499) as i32;
            mapq = mapq.max(0);
            r.mapq = mapq.min(60);
            if let Some(p) = &r.p {
                if p.dp_max > p.dp_max2 && r.mapq == 0 {
                    r.mapq = 1;
                }
            }
        } else {
            r.mapq = 0;
        }
    }
    mb_set_inv_mapq(km, n_regs, regs);
}

/// Original C static function `mb_dbg_seed` from `minibwa/map-algo.c:513`.
pub fn mb_dbg_seed(n: i64, u: &[mb_sai_t], qname: Option<&str>) {
    let name = qname.unwrap_or("*");
    for p in u.iter().take(n as usize) {
        eprintln!(
            "SD\t{}\t{}\t{}\t{}",
            name,
            (p.info >> 32) as i32,
            p.info as u32 as i32,
            p.size as i64
        );
    }
}

/// Original C static function `mb_dbg_anchor` from `minibwa/map-algo.c:522`.
pub fn mb_dbg_anchor(idx: &mb_idx_t, qlen: i32, n: i64, a: &[mb_anchor_t], qname: Option<&str>) {
    let name = qname.unwrap_or("*");
    for ai in a.iter().take(n as usize) {
        let rid = (ai.sid >> 1) as usize;
        let rev = ai.sid & 1;
        let qs = if rev != 0 {
            qlen - 1 - ai.qpos
        } else {
            ai.qpos + 1 - ai.len
        };
        let ts = ai.tpos + 1 - ai.len as i64;
        let strand = if rev != 0 { '-' } else { '+' };
        eprintln!(
            "AC\t{}\t{}\t{}\t{}\t{}\t{}",
            name, qs, strand, idx.l2b.ctg[rid].name, ts, ai.len
        );
    }
}

/// Original C global function `mb_map_sai` from `minibwa/map-algo.c:535`.
pub fn mb_map_sai(
    opt: &mb_opt_t,
    idx: &mb_idx_t,
    qlen: i64,
    seq0: &str,
    mt: l2b_meth_t,
    u: &mut mb_sai_v,
    n_hit_: &mut i32,
    b: &mut mb_tbuf_t,
    qname: Option<&str>,
) -> Vec<mb_hit_t> {
    const MIN_RECHAIN_LEN: i32 = 1000;
    const MIN_RECHAIN_RATIO: f64 = 0.1;
    *n_hit_ = 0;
    if u.n == 0 {
        u.a.clear();
        return Vec::new();
    }
    let mut hash = qname.map(mb_hash_str).unwrap_or(0);
    hash ^= (mb_hash64(qlen as u64).wrapping_add(mb_hash64(opt.seed as u64))) as u32;
    hash = mb_hash64(hash as u64) as u32;
    let mut seq: Vec<u8> = seq0
        .bytes()
        .take(qlen as usize)
        .map(|c| crate::kommon::KOM_NT4_TABLE[c as usize])
        .collect();
    let hi_cov = mb_cal_high_cov((), u.n as i32, &u.a, opt.max_occ);
    let is_sr = mb_is_sr_mode(opt, qlen as i32);
    let sub_diff = (opt.a + opt.b).max(opt.q + opt.e);
    let chn_pen_gap = opt.chain_gap_scale * 0.01 * opt.min_len as f32;

    let mut v = std::mem::take(&mut b.anchor_v);
    if (KOM_DBG_FLAG.load(Ordering::Relaxed) & MB_DBG_QNAME) != 0 {
        eprintln!("QN\t{}", qname.unwrap_or(""));
    }
    if (KOM_DBG_FLAG.load(Ordering::Relaxed) & MB_DBG_SEED) != 0 {
        mb_dbg_seed(u.n as i64, &u.a, qname);
    }
    crate::stage_time::measure(crate::stage_time::Bucket::Anchor, || {
        mb_anchor_with_scratch(
            (),
            idx,
            u,
            qlen as i32,
            mt,
            opt.max_occ,
            &mut v,
            &mut b.anchor_aux,
            &mut b.anchor_sa,
            &mut b.anchor_sa_batch,
            &mut b.anchor_batch,
        );
    });
    u.n = 0;
    u.a.clear();

    let mut n_hit = 0;
    let mut w = std::mem::take(&mut b.chain_w);
    w.clear();
    if (KOM_DBG_FLAG.load(Ordering::Relaxed) & MB_DBG_ANCHOR) != 0 {
        mb_dbg_anchor(idx, qlen as i32, v.n, &v.a, qname);
    }
    let anchors = std::mem::take(&mut v.a);
    let mut a = crate::stage_time::measure(crate::stage_time::Bucket::Chain, || {
        mb_lchain_dp(
            (),
            &idx.l2b,
            opt.max_gap,
            opt.max_gap,
            opt.bw,
            opt.max_chain_skip,
            opt.max_chain_iter,
            opt.min_chain_score,
            chn_pen_gap,
            v.n,
            anchors,
            &mut n_hit,
            &mut w,
        )
    });

    if opt.bw_long > opt.bw * 2 && is_sr == 0 && n_hit > 0 {
        let mut best = 0usize;
        for i in 1..n_hit as usize {
            if (w[i] >> 32) > (w[best] >> 32) {
                best = i;
            }
        }
        let mut as_ = 0usize;
        for x in w.iter().take(best) {
            as_ += *x as u32 as usize;
        }
        let cnt = w[best] as u32 as usize;
        let st = a[as_].qpos + 1 - a[as_].len;
        let en = a[as_ + cnt - 1].qpos + 1;
        if qlen as i32 - (en - st) > MIN_RECHAIN_LEN
            && (en - st) as f64 > qlen as f64 * MIN_RECHAIN_RATIO
        {
            let n_a = w
                .iter()
                .take(n_hit as usize)
                .map(|x| *x as u32 as i64)
                .sum::<i64>();
            mb_anchor_sort(&idx.l2b, n_a, &mut a);
            w.clear();
            a = crate::stage_time::measure(crate::stage_time::Bucket::Chain, || {
                mb_lchain_dp(
                    (),
                    &idx.l2b,
                    opt.max_gap,
                    opt.max_gap,
                    opt.bw_long,
                    opt.max_chain_skip,
                    opt.max_chain_iter,
                    opt.min_chain_score,
                    chn_pen_gap,
                    n_a,
                    a,
                    &mut n_hit,
                    &mut w,
                )
            });
        }
    }

    let mut hit = mb_gen_hit((), hash, qlen as i32, &idx.l2b, n_hit, &w, &a);
    mb_set_parent(
        (),
        opt.mask_level,
        opt.mask_len,
        n_hit,
        &mut hit,
        sub_diff,
        0,
    );
    mb_select_sub(
        (),
        opt.pri_ratio,
        opt.min_len * 2,
        opt.best_n,
        &mut n_hit,
        &mut hit,
    );

    if mt != l2b_meth_t::L2B_METH_NONE {
        crate::l2bit::l2b_meth_convert(mt, qlen, &mut seq);
    }
    if (opt.flag & MB_F_NO_ALN) == 0 {
        crate::stage_time::measure(crate::stage_time::Bucket::Align, || {
            mb_align_skeleton_with_scratch(
                (),
                opt,
                idx,
                qlen as i32,
                &seq,
                mt,
                &mut n_hit,
                &mut hit,
                &mut a,
                &mut b.align_tseq,
                &mut b.align_qseq0,
            );
        });
        mb_set_parent(
            (),
            opt.mask_level,
            opt.mask_len,
            n_hit,
            &mut hit,
            sub_diff,
            0,
        );
        mb_select_sub(
            (),
            opt.pri_ratio,
            opt.min_len * 2,
            opt.best_n,
            &mut n_hit,
            &mut hit,
        );
        mb_set_sam_pri(n_hit, &mut hit, ((opt.flag & MB_F_PRIMARY5) != 0) as i32);
    }
    crate::stage_time::measure(crate::stage_time::Bucket::MapqPost, || {
        for h in hit.iter_mut().take(n_hit as usize) {
            h.set_frac_high((255.0 * hi_cov as f64 / qlen as f64) as u8);
        }
        mb_set_mapq(
            (),
            qlen as i32,
            n_hit,
            &mut hit,
            opt.min_chain_score,
            opt.a,
            is_sr,
            opt.max_sr_len,
        );
    });
    *n_hit_ = n_hit;
    v.n = 0;
    v.m = a.capacity() as i64;
    v.a = a;
    b.anchor_v = v;
    b.chain_w = w;
    hit
}

/// Align one sequence.
///
/// # Arguments
/// - `opt`: options, typically initialized by `mb_opt_init()`
/// - `idx`: index
/// - `qlen`: query length
/// - `seq`: query sequence, ASCII or 0/1/2/3 encoded
/// - `mt`: methylation type: 0 for unmethylated, 1 for read1 (C-to-T) and 2 for read2 (G-to-A)
/// - `n_hit`: (out) number of hits
/// - `b`: thread buffer; can be `NULL`
/// - `qname`: query name
///
/// # Returns
/// hit array
///
/// Original C global function `mb_map` from `minibwa/map-algo.c:615`.
pub fn mb_map(
    opt: &mb_opt_t,
    idx: &mb_idx_t,
    qlen: i32,
    seq0: &str,
    mt0: i32,
    n_hit_: &mut i32,
    b0: Option<&mut mb_tbuf_t>,
    qname: Option<&str>,
) -> Vec<mb_hit_t> {
    let mt = if mt0 == 0 {
        l2b_meth_t::L2B_METH_NONE
    } else if mt0 == 1 {
        l2b_meth_t::L2B_METH_C2T
    } else {
        l2b_meth_t::L2B_METH_G2A
    };
    let mut owned;
    let b = if let Some(b) = b0 {
        b
    } else {
        owned = mb_tbuf_init(1);
        &mut owned
    };
    let mut opt_adap = mb_opt_t::default();
    mb_opt_adap(opt, qlen, &mut opt_adap);
    let seq = seq0
        .bytes()
        .take(qlen as usize)
        .map(|c| crate::kommon::KOM_NT4_TABLE[c as usize])
        .collect::<Vec<_>>();
    let mut u = mb_sai_v::default();
    mb_seed_intv(
        (),
        &idx.bwt,
        qlen,
        &seq,
        opt.min_len,
        opt.max_sub_occ,
        &mut u,
    );
    drop(seq);
    mb_map_sai(
        &opt_adap,
        idx,
        qlen as i64,
        seq0,
        mt,
        &mut u,
        n_hit_,
        b,
        qname,
    )
}

/// Align a set of sequences in batch.
///
/// # Arguments
/// - `opt`: options, typically initialized by `mb_opt_init()`
/// - `idx`: index
/// - `n_seq`: number of sequences
/// - `qlen`: query lengths, of size `n_seq`
/// - `seq`: query sequences, ASCII or 0/1/2/3 encoded, of size `n_seq`
/// - `n_hit`: (out) number of hits, of size `n_seq`
/// - `b`: thread buffer; can be `NULL`
/// - `qname`: query name, of size `n_seq`
///
/// # Returns
/// hits, of size `n_seq`
///
/// Original C global function `mb_map_batch` from `minibwa/map-algo.c:656`.
pub fn mb_map_batch(
    opt: &mb_opt_t,
    idx: &mb_idx_t,
    n_seq: i32,
    qlen: &[i32],
    seq: &[&str],
    n_hit: &mut [i32],
    b0: Option<&mut mb_tbuf_t>,
    qname: Option<&[&str]>,
) -> Vec<Vec<mb_hit_t>> {
    if n_seq <= 0 {
        return Vec::new();
    }
    let mut owned;
    let b = if let Some(b) = b0 {
        b
    } else {
        owned = mb_tbuf_init(0);
        &mut owned
    };
    let is_pe = ((opt.flag & MB_F_PE) != 0) as i32;
    let mut hit = vec![Vec::new(); n_seq as usize];
    let sb_max = opt.sb_seq.min(n_seq).max(0) as usize;
    let mut sai = vec![mb_sai_v::default(); sb_max.max(1)];
    let mut seq4 = vec![Vec::<u8>::new(); sb_max.max(1)];
    let mut i = 0i32;
    let mut sb_st = 0i32;
    let mut sb_len = 0i32;
    while i <= n_seq {
        if i == n_seq || sb_len >= opt.sb_len || i - sb_st >= opt.sb_seq {
            let sb_n = i - sb_st;
            if sb_n == 0 {
                sb_st = i;
                sb_len = 0;
                i += 1;
                continue;
            }
            for k in 0..sb_n as usize {
                let idx_k = sb_st as usize + k;
                seq4[k].clear();
                seq4[k].extend(
                    seq[idx_k]
                        .bytes()
                        .take(qlen[idx_k] as usize)
                        .map(|c| match c {
                            b'A' | b'a' => 0,
                            b'C' | b'c' => 1,
                            b'G' | b'g' => 2,
                            b'T' | b't' => 3,
                            _ => 4,
                        }),
                );
                sai[k] = mb_sai_v::default();
            }
            let seq4_refs = seq4[..sb_n as usize]
                .iter()
                .map(|s| s.as_ptr())
                .collect::<Vec<_>>();
            mb_seed_intv_batch(
                (),
                &idx.bwt,
                sb_n,
                &qlen[sb_st as usize..],
                &seq4_refs,
                opt.min_len,
                opt.max_sub_occ,
                &mut sai[..sb_n as usize],
            );
            for k in 0..sb_n as usize {
                seq4[k] = Vec::new();
            }
            for k in 0..sb_n as usize {
                let idx_k = sb_st as usize + k;
                let mut opt_adap = mb_opt_t::default();
                mb_opt_adap(opt, qlen[idx_k], &mut opt_adap);
                let mut mt = l2b_meth_t::L2B_METH_NONE;
                if (opt.flag & MB_F_METH) != 0 {
                    mt = if is_pe != 0 {
                        if (idx_k & 1) == 0 {
                            l2b_meth_t::L2B_METH_C2T
                        } else {
                            l2b_meth_t::L2B_METH_G2A
                        }
                    } else {
                        l2b_meth_t::L2B_METH_C2T
                    };
                }
                let name = qname.and_then(|names| names.get(idx_k).copied());
                hit[idx_k] = mb_map_sai(
                    &opt_adap,
                    idx,
                    qlen[idx_k] as i64,
                    seq[idx_k],
                    mt,
                    &mut sai[k],
                    &mut n_hit[idx_k],
                    b,
                    name,
                );
            }
            sb_st = i;
            sb_len = 0;
        }
        if i < n_seq {
            sb_len += qlen[i as usize];
        }
        i += 1;
    }
    hit
}
