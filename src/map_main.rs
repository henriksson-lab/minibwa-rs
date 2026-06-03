#![allow(unused_variables, dead_code, non_snake_case, non_camel_case_types)]

use crate::bseq::{
    mb_bseq1_t, mb_bseq_close, mb_bseq_file_t, mb_bseq_open, mb_bseq_read, mb_bseq_read_frag,
    mb_qname_same,
};
use crate::bwt::mb_sai_v;
use crate::format::{mb_fmt_sam_hdr, mb_format};
use crate::ketopt::{ketopt, ko_longopt_t, KETOPT_INIT};
use crate::kommon::{
    kom_atof, kom_panic, kom_parse_num, kom_percent_cpu, kom_realtime, kom_strtol, kstring_t,
    KOM_NT4_TABLE,
};
use crate::kthread::kt_for_parallel;
use crate::l2bit::l2b_meth_t;
use crate::main::MB_VERSION;
use crate::map_algo::{
    mb_idx_load, mb_idx_t, mb_map_sai, mb_tbuf_destroy, mb_tbuf_init, mb_tbuf_reset, mb_tbuf_t,
};
use crate::mbpriv::{
    mb_cstr_prefix, KOM_DBG_FLAG, MB_DBG_ALN_PE, MB_DBG_ALN_SEQ, MB_DBG_ANCHOR, MB_DBG_AN_POS,
    MB_DBG_QNAME, MB_DBG_SEED,
};
use crate::options::{
    mb_opt_adap, mb_opt_init, mb_opt_preset, mb_opt_t, MB_F_ADAP, MB_F_COPY_COMMENT, MB_F_EQX,
    MB_F_LONG, MB_F_METH, MB_F_NO_ALN, MB_F_NO_KALLOC, MB_F_NO_PAIRING, MB_F_NO_UNMAP, MB_F_PE,
    MB_F_PE_PREDEF, MB_F_PRIMARY5, MB_F_SAM, MB_F_SUPP_SOFT, MB_F_WRITE_CS, MB_F_WRITE_DS,
    MB_F_WRITE_MD,
};
use crate::pe::{mb_hit_buf_t, mb_hit_t, mb_pair, mb_pestat, mb_pestat_t};
use crate::seed::mb_seed_intv_batch;
use rayon::ThreadPool;
use std::cell::RefCell;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::{fs, io::BufWriter, io::Write};

thread_local! {
    static MAIN_MAP_OUTPUT_WRITER: RefCell<Option<*mut dyn Write>> = RefCell::new(None);
}

fn c_strerror(errno: i32) -> String {
    std::io::Error::from_raw_os_error(errno)
        .to_string()
        .trim_end_matches(&format!(" (os error {errno})"))
        .to_string()
}

fn c_str(s: &str) -> &str {
    let end = s.as_bytes().iter().position(|&c| c == 0).unwrap_or(s.len());
    &s[..end]
}

fn c_str_eq(s: &str, expected: &str) -> bool {
    c_str(s).as_bytes() == expected.as_bytes()
}

pub struct pipeline_t<'a, 'w, 'p> {
    pub n_fp: i32,
    pub n_threads: i32,
    pub n_base: i64,
    pub n_seq: i64,
    pub mb_size: i64,
    pub opt: &'a mb_opt_t,
    pub fp: Vec<mb_bseq_file_t>,
    pub idx: &'a mb_idx_t,
    pub worker_pool: Option<&'p ThreadPool>,
    pub output: String,
    pub output_writer: Option<&'w mut dyn Write>,
    pub output_error: bool,
}

#[derive(Debug)]
pub struct step_t<'a> {
    pub opt: &'a mb_opt_t,
    pub idx: &'a mb_idx_t,
    pub n_seq: i32,
    pub n_frag: i32,
    pub n_sb: i32,
    pub n_pe: i32,
    pub n_hit: Vec<i32>,
    pub seg_off: Vec<i32>,
    pub seg_cnt: Vec<i32>,
    pub sb_off: Vec<i32>,
    pub sb_cnt: Vec<i32>,
    pub pes: [mb_pestat_t; 4],
    pub seq: Vec<mb_bseq1_t>,
    pub hit: Vec<mb_hit_buf_t>,
    pub tbuf: Vec<mb_tbuf_t>,
}

#[derive(Clone, Copy)]
struct SeBatchView<'a> {
    opt: &'a mb_opt_t,
    idx: &'a mb_idx_t,
    seq: &'a [mb_bseq1_t],
    seg_off: &'a [i32],
    seg_cnt: &'a [i32],
    sb_off: &'a [i32],
    sb_cnt: &'a [i32],
}

#[derive(Clone, Copy)]
struct SeBatchOutputs {
    n_hit: *mut i32,
    hit: *mut mb_hit_buf_t,
}

unsafe impl Send for SeBatchOutputs {}
unsafe impl Sync for SeBatchOutputs {}

impl SeBatchOutputs {
    unsafe fn write(self, idx: usize, n_hit: i32, hit: Vec<mb_hit_t>) {
        // SAFETY: callers partition work by non-overlapping sub-batches,
        // so each read slot is written by at most one worker.
        unsafe {
            *self.n_hit.add(idx) = n_hit;
            *self.hit.add(idx) = mb_hit_buf_t::from_vec(hit);
        }
    }
}

#[derive(Clone, Copy)]
struct SeBatchScratch {
    tbuf: *mut mb_tbuf_t,
    len: usize,
}

unsafe impl Send for SeBatchScratch {}
unsafe impl Sync for SeBatchScratch {}

impl SeBatchScratch {
    unsafe fn get(self, tid: usize) -> &'static mut mb_tbuf_t {
        debug_assert!(tid < self.len);
        // SAFETY: Rayon runs at most one task on a worker thread at a time.
        // Each worker indexes its own scratch buffer by current_thread_index().
        unsafe { &mut *self.tbuf.add(tid.min(self.len - 1)) }
    }
}

#[derive(Clone, Copy)]
struct PePairView<'a> {
    opt: &'a mb_opt_t,
    idx: &'a mb_idx_t,
    seq: &'a [mb_bseq1_t],
    seg_off: &'a [i32],
    seg_cnt: &'a [i32],
    pes: &'a [mb_pestat_t; 4],
}

#[derive(Clone, Copy)]
struct PePairOutputs {
    n_hit: *mut i32,
    hit: *mut mb_hit_buf_t,
}

unsafe impl Send for PePairOutputs {}
unsafe impl Sync for PePairOutputs {}

impl PePairOutputs {
    unsafe fn pair_in_place(self, view: PePairView<'_>, frag: usize, tid: i32) {
        if view.seg_cnt[frag] != 2 {
            return;
        }
        let off = view.seg_off[frag] as usize;
        if (KOM_DBG_FLAG.load(Ordering::Relaxed) & MB_DBG_QNAME) != 0 {
            eprintln!("QP\t{}\t{}", mb_cstr_prefix(&view.seq[off].name), tid);
        }
        let len = [view.seq[off].l_seq as i32, view.seq[off + 1].l_seq as i32];
        let seq = [&*view.seq[off].seq, &*view.seq[off + 1].seq];
        // SAFETY: paired-end work is partitioned by fragment. Each
        // two-read hit slot is owned by exactly one fragment.
        unsafe {
            let mut n_pair = [*self.n_hit.add(off), *self.n_hit.add(off + 1)];
            let mut hit_pair = [
                std::mem::take(&mut *self.hit.add(off)).into_vec(),
                std::mem::take(&mut *self.hit.add(off + 1)).into_vec(),
            ];
            mb_pair(
                (),
                view.opt,
                &view.idx.l2b,
                &mut n_pair,
                &mut hit_pair,
                view.pes,
                len,
                seq,
            );
            *self.n_hit.add(off) = n_pair[0];
            *self.n_hit.add(off + 1) = n_pair[1];
            *self.hit.add(off) = mb_hit_buf_t::from_vec(std::mem::take(&mut hit_pair[0]));
            *self.hit.add(off + 1) = mb_hit_buf_t::from_vec(std::mem::take(&mut hit_pair[1]));
        }
    }
}

fn release_step_tbuf(s: &mut step_t<'_>) {
    for b in std::mem::take(&mut s.tbuf) {
        mb_tbuf_destroy(Some(b));
    }
}

/// Original C static function `worker_for_se_batch` from `minibwa/map-main.c:33`.
fn worker_for_se_batch_collect_view(
    view: SeBatchView<'_>,
    i: i64,
    tid: i32,
    b: &mut mb_tbuf_t,
    outputs: Option<SeBatchOutputs>,
) -> Vec<(usize, i32, Vec<mb_hit_t>)> {
    let opt = view.opt;
    let idx = view.idx;
    let sb_i = i.max(0) as usize;
    let mut n = 0usize;
    let mut tot = 0usize;
    for k in 0..view.sb_cnt[sb_i] as usize {
        let frag = view.sb_off[sb_i] as usize + k;
        let off = view.seg_off[frag] as usize;
        let cnt = view.seg_cnt[frag] as usize;
        n += cnt;
        for j in 0..cnt {
            tot += view.seq[off + j].l_seq as usize;
        }
    }
    let mut out = if outputs.is_some() {
        Vec::new()
    } else {
        Vec::with_capacity(n)
    };
    let mut len = std::mem::take(&mut b.se_len);
    len.clear();
    len.resize(n, 0);
    let mut buf = std::mem::take(&mut b.se_buf);
    buf.clear();
    buf.reserve(tot.saturating_sub(buf.capacity()));
    let mut sai = std::mem::take(&mut b.se_sai);
    sai.clear();
    sai.resize(n, mb_sai_v::default());
    let mut p = 0usize;
    crate::stage_time::measure(crate::stage_time::Bucket::Encode, || {
        for k in 0..view.sb_cnt[sb_i] as usize {
            let frag = view.sb_off[sb_i] as usize + k;
            let off = view.seg_off[frag] as usize;
            let cnt = view.seg_cnt[frag] as usize;
            for j in 0..cnt {
                let t = &view.seq[off + j];
                len[p] = t.l_seq as i32;
                let range_st = buf.len();
                buf.extend(
                    t.seq
                        .bytes()
                        .take(t.l_seq as usize)
                        .map(|c| KOM_NT4_TABLE[c as usize]),
                );
                if idx.is_meth != 0 {
                    if (j & 1) == 0 {
                        for x in &mut buf[range_st..] {
                            if *x == 1 {
                                *x = 3;
                            }
                        }
                    } else {
                        for x in &mut buf[range_st..] {
                            if *x == 2 {
                                *x = 0;
                            }
                        }
                    }
                }
                p += 1;
            }
        }
    });
    debug_assert_eq!(p, n);
    let mut seq = std::mem::take(&mut b.se_seq_ptrs);
    seq.clear();
    seq.reserve(n.saturating_sub(seq.capacity()));
    let mut range_st = 0usize;
    for &l in &len {
        seq.push(unsafe { buf.as_ptr().add(range_st) } as usize);
        range_st += l as usize;
    }
    crate::stage_time::measure(crate::stage_time::Bucket::Seed, || {
        mb_seed_intv_batch(
            (),
            &idx.bwt,
            n as i32,
            &len,
            unsafe { std::slice::from_raw_parts(seq.as_ptr() as *const *const u8, seq.len()) },
            opt.min_len,
            opt.max_sub_occ,
            &mut sai,
        );
    });
    p = 0;
    for k in 0..view.sb_cnt[sb_i] as usize {
        let frag = view.sb_off[sb_i] as usize + k;
        let off = view.seg_off[frag] as usize;
        let cnt = view.seg_cnt[frag] as usize;
        for j in 0..cnt {
            let t = &view.seq[off + j];
            let mt = if idx.is_meth == 0 {
                l2b_meth_t::L2B_METH_NONE
            } else if (j & 1) == 0 {
                l2b_meth_t::L2B_METH_C2T
            } else {
                l2b_meth_t::L2B_METH_G2A
            };
            let mut opt_adap = mb_opt_t::default();
            mb_opt_adap(opt, t.l_seq as i32, &mut opt_adap);
            let mut n_hit = 0;
            let hit = mb_map_sai(
                &opt_adap,
                idx,
                t.l_seq as i64,
                &t.seq,
                mt,
                &mut sai[p],
                &mut n_hit,
                b,
                Some(&t.name),
            );
            if let Some(outputs) = outputs {
                unsafe { outputs.write(off + j, n_hit, hit) };
            } else {
                out.push((off + j, n_hit, hit));
            }
            p += 1;
        }
    }
    let _ = tot;
    let _ = tid;
    b.se_len = len;
    b.se_buf = buf;
    b.se_seq_ptrs = seq;
    b.se_sai = sai;
    mb_tbuf_reset(b, opt.cap_kalloc);
    crate::stage_time::flush_local();
    out
}

fn worker_for_se_batch_collect(
    s: &mut step_t<'_>,
    i: i64,
    tid: i32,
) -> Vec<(usize, i32, Vec<mb_hit_t>)> {
    let tid_idx = tid.max(0) as usize;
    let mut b = mb_tbuf_init(((s.opt.flag & MB_F_NO_KALLOC) != 0) as i32);
    let b = if tid_idx < s.tbuf.len() {
        &mut s.tbuf[tid_idx]
    } else {
        &mut b
    };
    worker_for_se_batch_collect_view(
        SeBatchView {
            opt: s.opt,
            idx: s.idx,
            seq: &s.seq,
            seg_off: &s.seg_off,
            seg_cnt: &s.seg_cnt,
            sb_off: &s.sb_off,
            sb_cnt: &s.sb_cnt,
        },
        i,
        tid,
        b,
        None,
    )
}

/// Original C static function `worker_for_se_batch` from `minibwa/map-main.c:33`.
pub fn worker_for_se_batch(s: &mut step_t<'_>, i: i64, tid: i32) {
    for (idx, n_hit, hit) in worker_for_se_batch_collect(s, i, tid) {
        s.n_hit[idx] = n_hit;
        s.hit[idx] = mb_hit_buf_t::from_vec(hit);
    }
}

fn worker_for_pe_collect(
    s: &step_t<'_>,
    i: i64,
    tid: i32,
) -> Option<(usize, [i32; 2], [Vec<mb_hit_t>; 2])> {
    let frag = i.max(0) as usize;
    if s.seg_cnt[frag] != 2 {
        return None;
    }
    let off = s.seg_off[frag] as usize;
    if (KOM_DBG_FLAG.load(Ordering::Relaxed) & MB_DBG_QNAME) != 0 {
        eprintln!("QP\t{}\t{}", mb_cstr_prefix(&s.seq[off].name), tid);
    }
    let len = [s.seq[off].l_seq as i32, s.seq[off + 1].l_seq as i32];
    let seq = [&*s.seq[off].seq, &*s.seq[off + 1].seq];
    let mut n_pair = [s.n_hit[off], s.n_hit[off + 1]];
    let mut hit_pair = [
        s.hit[off].as_slice().to_vec(),
        s.hit[off + 1].as_slice().to_vec(),
    ];
    mb_pair(
        (),
        s.opt,
        &s.idx.l2b,
        &mut n_pair,
        &mut hit_pair,
        &s.pes,
        len,
        seq,
    );
    Some((off, n_pair, hit_pair))
}

/// Original C static function `worker_for_pe` from `minibwa/map-main.c:82`.
pub fn worker_for_pe(s: &mut step_t<'_>, i: i64, tid: i32) {
    let frag = i.max(0) as usize;
    if s.seg_cnt[frag] != 2 {
        return;
    }
    let off = s.seg_off[frag] as usize;
    if (KOM_DBG_FLAG.load(Ordering::Relaxed) & MB_DBG_QNAME) != 0 {
        eprintln!("QP\t{}\t{}", mb_cstr_prefix(&s.seq[off].name), tid);
    }
    let len = [s.seq[off].l_seq as i32, s.seq[off + 1].l_seq as i32];
    let seq = [&*s.seq[off].seq, &*s.seq[off + 1].seq];
    let mut n_pair = [s.n_hit[off], s.n_hit[off + 1]];
    let mut hit_pair = [
        std::mem::take(&mut s.hit[off]).into_vec(),
        std::mem::take(&mut s.hit[off + 1]).into_vec(),
    ];
    mb_pair(
        (),
        s.opt,
        &s.idx.l2b,
        &mut n_pair,
        &mut hit_pair,
        &s.pes,
        len,
        seq,
    );
    s.n_hit[off] = n_pair[0];
    s.n_hit[off + 1] = n_pair[1];
    s.hit[off] = mb_hit_buf_t::from_vec(std::mem::take(&mut hit_pair[0]));
    s.hit[off + 1] = mb_hit_buf_t::from_vec(std::mem::take(&mut hit_pair[1]));
    let _ = tid;
}

/// Original C static function `worker_pipeline` from `minibwa/map-main.c:98`.
pub fn worker_pipeline<'a, 'w, 'p>(
    p: &mut pipeline_t<'a, 'w, 'p>,
    step: i32,
    input: Option<step_t<'a>>,
) -> Option<step_t<'a>> {
    const MIN_READ_CNT: i32 = 40000;
    let opt = p.opt;
    if step == 0 {
        let with_qual = ((opt.flag & MB_F_SAM) != 0) as i32;
        let with_comment = ((opt.flag & MB_F_COPY_COMMENT) != 0) as i32;
        let frag_mode = (p.n_fp > 1 || (opt.flag & MB_F_PE) != 0) as i32;
        let mut n_seq = 0;
        let mut seq = crate::stage_time::measure(crate::stage_time::Bucket::ReadIo, || {
            if p.n_fp > 1 {
                mb_bseq_read_frag(
                    p.n_fp,
                    &mut p.fp,
                    p.mb_size,
                    with_qual,
                    with_comment,
                    &mut n_seq,
                )
            } else {
                mb_bseq_read(
                    &mut p.fp[0],
                    p.mb_size,
                    with_qual,
                    with_comment,
                    frag_mode,
                    MIN_READ_CNT,
                    opt.max_mb_size,
                    &mut n_seq,
                )
            }
        });
        if seq.is_empty() {
            return None;
        }
        let mut s = step_t {
            opt,
            idx: p.idx,
            n_seq,
            n_frag: 0,
            n_sb: 0,
            n_pe: 0,
            n_hit: vec![0; n_seq as usize],
            seg_off: vec![0; n_seq as usize],
            seg_cnt: vec![0; n_seq as usize],
            sb_off: vec![0; n_seq as usize + 1],
            sb_cnt: vec![0; n_seq as usize + 1],
            pes: [mb_pestat_t {
                failed: 1,
                ..Default::default()
            }; 4],
            hit: vec![mb_hit_buf_t::default(); n_seq as usize],
            tbuf: Vec::with_capacity(opt.n_thread.max(1) as usize),
            seq: Vec::new(),
        };
        for t in &mut seq {
            t.id = p.n_seq as u64;
            p.n_seq += 1;
        }
        for _ in 0..opt.n_thread.max(1) {
            s.tbuf
                .push(mb_tbuf_init(((opt.flag & MB_F_NO_KALLOC) != 0) as i32));
        }
        s.seq = seq;
        let mut j = 0usize;
        for i in 1..=s.n_seq as usize {
            if i == s.n_seq as usize
                || frag_mode == 0
                || mb_qname_same(&s.seq[i - 1].name, &s.seq[i].name) == 0
            {
                debug_assert!(i - j <= 2);
                s.seg_cnt[s.n_frag as usize] = (i - j) as i32;
                s.seg_off[s.n_frag as usize] = j as i32;
                s.n_frag += 1;
                if i - j == 2 {
                    s.n_pe += 1;
                }
                j = i;
            }
        }
        if s.n_pe > 0 {
            for i in 0..s.n_frag as usize {
                if s.seg_cnt[i] != 2 {
                    continue;
                }
                let j0 = s.seg_off[i] as usize;
                let j1 = j0 + 1;
                let l0 = s.seq[j0].name.len();
                let l1 = s.seq[j1].name.len();
                if l0 >= 3
                    && l0 == l1
                    && s.seq[j0].name.as_bytes()[l0 - 1] != s.seq[j1].name.as_bytes()[l1 - 1]
                    && s.seq[j0].name.as_bytes()[l0 - 2] == b'/'
                {
                    s.seq[j0].name = s.seq[j0].name[..l0 - 2].into();
                    s.seq[j1].name = s.seq[j1].name[..l1 - 2].into();
                }
            }
        }
        let mut sb_len = 0i32;
        let mut sb_off = 0usize;
        for i in 0..s.n_frag as usize {
            if sb_len >= opt.sb_len || i - sb_off >= opt.sb_seq as usize {
                s.sb_off[s.n_sb as usize] = sb_off as i32;
                s.sb_cnt[s.n_sb as usize] = (i - sb_off) as i32;
                s.n_sb += 1;
                sb_len = 0;
                sb_off = i;
            }
            for j in 0..s.seg_cnt[i] as usize {
                sb_len += s.seq[s.seg_off[i] as usize + j].l_seq as i32;
            }
        }
        s.sb_off[s.n_sb as usize] = sb_off as i32;
        s.sb_cnt[s.n_sb as usize] = (s.n_frag as usize - sb_off) as i32;
        s.n_sb += 1;
        Some(s)
    } else if step == 1 {
        let mut s = input?;
        if let Some(pool) = p.worker_pool {
            let view = SeBatchView {
                opt: s.opt,
                idx: s.idx,
                seq: &s.seq,
                seg_off: &s.seg_off,
                seg_cnt: &s.seg_cnt,
                sb_off: &s.sb_off,
                sb_cnt: &s.sb_cnt,
            };
            let outputs = SeBatchOutputs {
                n_hit: s.n_hit.as_mut_ptr(),
                hit: s.hit.as_mut_ptr(),
            };
            let scratch = SeBatchScratch {
                tbuf: s.tbuf.as_mut_ptr(),
                len: s.tbuf.len(),
            };
            pool.install(|| {
                kt_for_parallel(
                    opt.n_thread,
                    |i, tid| {
                        let b = unsafe { scratch.get(tid as usize) };
                        let _ = worker_for_se_batch_collect_view(view, i, tid, b, Some(outputs));
                    },
                    s.n_sb as i64,
                );
            });
        } else {
            for i in 0..s.n_sb {
                worker_for_se_batch(&mut s, i as i64, 0);
            }
        }
        if (opt.flag & MB_F_PE) != 0 && s.n_frag < s.n_seq && (opt.flag & MB_F_NO_PAIRING) == 0 {
            if (opt.flag & MB_F_PE_PREDEF) != 0 || s.n_pe < 20 {
                s.pes[1].failed = 0;
                s.pes[1].avg = opt.pe_avg as f64;
                s.pes[1].std = opt.pe_std as f64;
                s.pes[1].lo = opt.pe_lo;
                s.pes[1].hi = opt.pe_hi;
            } else {
                mb_pestat(
                    (),
                    opt,
                    s.n_frag,
                    &s.seg_off,
                    &s.seg_cnt,
                    &s.n_hit,
                    &s.hit,
                    &mut s.pes,
                );
            }
            let _pair_t0 = if crate::stage_time::enabled() {
                Some(std::time::Instant::now())
            } else {
                None
            };
            if let Some(pool) = p.worker_pool {
                let view = PePairView {
                    opt: s.opt,
                    idx: s.idx,
                    seq: &s.seq,
                    seg_off: &s.seg_off,
                    seg_cnt: &s.seg_cnt,
                    pes: &s.pes,
                };
                let outputs = PePairOutputs {
                    n_hit: s.n_hit.as_mut_ptr(),
                    hit: s.hit.as_mut_ptr(),
                };
                pool.install(|| {
                    kt_for_parallel(
                        opt.n_thread,
                        |i, tid| {
                            unsafe { outputs.pair_in_place(view, i as usize, tid) };
                        },
                        s.n_frag as i64,
                    );
                });
            } else {
                for i in 0..s.n_frag {
                    worker_for_pe(&mut s, i as i64, 0);
                }
            }
            if let Some(t0) = _pair_t0 {
                crate::stage_time::accumulate_global(
                    crate::stage_time::Bucket::Pair,
                    t0.elapsed().as_nanos() as u64,
                );
            }
        }
        Some(s)
    } else if step == 2 {
        let mut s = input?;
        let mut out = kstring_t::default();
        const OUTPUT_FLUSH_BYTES: usize = 1 << 20;
        const OUTPUT_RETAIN_BYTES: usize = 16 << 20;
        let mut tot_len = 0i64;
        release_step_tbuf(&mut s);
        let _stage2 = if crate::stage_time::enabled() {
            Some(std::time::Instant::now())
        } else {
            None
        };
        for k in 0..s.n_frag as usize {
            let seg_st = s.seg_off[k] as usize;
            let seg_en = seg_st + s.seg_cnt[k] as usize;
            if p.output_writer.is_none() {
                out.l = 0;
            }
            for i in seg_st..seg_en {
                let mate_qlen = if seg_en - seg_st > 1 {
                    let mate_idx = if i != seg_en - 1 { i + 1 } else { seg_st };
                    s.seq[mate_idx].l_seq as i32
                } else {
                    0
                };
                tot_len += s.seq[i].l_seq as i64;
                if s.n_hit[i] > 0 {
                    let mut n_sec = 0;
                    for j in 0..s.n_hit[i] as usize {
                        let h = &s.hit[i][j];
                        if h.parent == h.id || n_sec < opt.out_n {
                            mb_format(
                                (),
                                &mut out,
                                &p.idx.l2b,
                                &s.seq[i],
                                (seg_en - seg_st) as i32,
                                &s.n_hit[seg_st..seg_en],
                                &s.hit[seg_st..seg_en],
                                j as i32,
                                opt.flag,
                                (i - seg_st) as i32,
                                mate_qlen,
                            );
                        }
                        if h.parent != h.id {
                            n_sec += 1;
                        }
                    }
                } else if (opt.flag & MB_F_NO_UNMAP) == 0 {
                    mb_format(
                        (),
                        &mut out,
                        &p.idx.l2b,
                        &s.seq[i],
                        (seg_en - seg_st) as i32,
                        &s.n_hit[seg_st..seg_en],
                        &s.hit[seg_st..seg_en],
                        -1,
                        opt.flag,
                        (i - seg_st) as i32,
                        mate_qlen,
                    );
                }
            }
            for i in seg_st..seg_en {
                s.hit[i] = mb_hit_buf_t::default();
                s.seq[i] = mb_bseq1_t::default();
            }
            if p.output_writer.is_some() {
                if out.l >= OUTPUT_FLUSH_BYTES {
                    if let Some(writer) = p.output_writer.as_mut() {
                        if writer.write_all(&out.s[..out.l]).is_err() {
                            p.output_error = true;
                        }
                    }
                    out.l = 0;
                    if out.s.capacity() > OUTPUT_RETAIN_BYTES {
                        out = kstring_t::default();
                    }
                }
            } else {
                p.output.push_str(&String::from_utf8_lossy(&out.s[..out.l]));
                out.l = 0;
            }
        }
        if out.l > 0 && p.output_writer.is_some() {
            if let Some(writer) = p.output_writer.as_mut() {
                if writer.write_all(&out.s[..out.l]).is_err() {
                    p.output_error = true;
                }
            }
        }
        if let Some(t0) = _stage2 {
            crate::stage_time::accumulate_global(
                crate::stage_time::Bucket::Output,
                t0.elapsed().as_nanos() as u64,
            );
        }
        p.n_base += tot_len;
        None
    } else {
        None
    }
}

/// Original C static function `mb_open_bseqs` from `minibwa/map-main.c:230`.
pub fn mb_open_bseqs(n: i32, fn_: &[&str]) -> Option<Vec<mb_bseq_file_t>> {
    let mut fp = Vec::with_capacity(n.max(0) as usize);
    for i in 0..n.max(0) as usize {
        let name = c_str(fn_[i]);
        let Some(f) = mb_bseq_open(Some(name)) else {
            let reason = std::fs::File::open(name)
                .err()
                .and_then(|e| e.raw_os_error())
                .map(c_strerror)
                .unwrap_or_else(|| "Unknown error".to_string());
            eprintln!("ERROR: failed to open file '{}': {}", name, reason);
            return None;
        };
        fp.push(f);
    }
    Some(fp)
}

fn invalid_thread_count_fatal_like_c() -> ! {
    #[cfg(unix)]
    unsafe {
        libc::strlen(std::ptr::null());
        std::hint::unreachable_unchecked();
    }
    #[cfg(not(unix))]
    panic!("invalid thread count reaches kt_pipeline like C");
}

fn missing_kalloc_arg_fatal_like_c() -> ! {
    #[cfg(unix)]
    unsafe {
        libc::strcmp(std::ptr::null(), b"yes\0".as_ptr().cast());
        std::hint::unreachable_unchecked();
    }
    #[cfg(not(unix))]
    panic!("bare --kalloc passes NULL to strcmp like C");
}

/// Original C global function `mb_map_file` from `minibwa/map-main.c:248`.
pub fn mb_map_file(
    opt: &mb_opt_t,
    idx: &mb_idx_t,
    n: i32,
    fn_: &[&str],
    fn_out: Option<&str>,
) -> (i32, String) {
    let worker_pool = if opt.n_thread > 1 {
        Some(
            rayon::ThreadPoolBuilder::new()
                .num_threads(opt.n_thread as usize)
                .build()
                .expect("failed to build Rayon thread pool"),
        )
    } else {
        None
    };
    mb_map_file_with_pool(opt, idx, n, fn_, fn_out, worker_pool.as_ref())
}

pub fn mb_map_file_with_pool(
    opt: &mb_opt_t,
    idx: &mb_idx_t,
    n: i32,
    fn_: &[&str],
    fn_out: Option<&str>,
    worker_pool: Option<&ThreadPool>,
) -> (i32, String) {
    if n < 1 {
        return (-1, String::new());
    }
    let mut output_file = if let Some(name) = fn_out {
        let name = c_str(name);
        if name != "-" {
            match fs::File::create(name) {
                Ok(fp) => Some(BufWriter::with_capacity(1 << 20, fp)),
                Err(_) => return (-1, String::new()),
            }
        } else {
            None
        }
    } else {
        None
    };
    let Some(fp) = mb_open_bseqs(n, fn_) else {
        return (-1, String::new());
    };
    if opt.n_thread <= 0 {
        invalid_thread_count_fatal_like_c();
    }
    let writes_to_stdout = fn_out.is_none() || fn_out.is_some_and(|name| c_str_eq(name, "-"));
    let output_writer: Option<&mut dyn Write> = if writes_to_stdout {
        MAIN_MAP_OUTPUT_WRITER.with(|writer| {
            // SAFETY: main_map_write installs this pointer only for the
            // synchronous dynamic extent of its main_map call.
            writer.borrow().map(|ptr| unsafe { &mut *ptr })
        })
    } else {
        None
    };
    let run = |fp: Vec<mb_bseq_file_t>, output_writer: Option<&mut dyn Write>| -> (i32, String) {
        if worker_pool.is_some() {
            return std::thread::scope(|scope| {
                let (read_tx, read_rx) = mpsc::sync_channel(2);
                let (map_tx, map_rx) = mpsc::sync_channel(2);
                scope.spawn(move || {
                    let mut read_pl = pipeline_t {
                        n_fp: n,
                        n_threads: if opt.n_thread <= 2 { opt.n_thread } else { 3 },
                        n_base: 0,
                        n_seq: 0,
                        mb_size: opt.mb_size,
                        opt,
                        fp,
                        idx,
                        worker_pool,
                        output: String::new(),
                        output_writer: None,
                        output_error: false,
                    };
                    while let Some(s0) = worker_pipeline(&mut read_pl, 0, None) {
                        let send_result = crate::stage_time::measure(
                            crate::stage_time::Bucket::PipeReadSend,
                            || read_tx.send(s0),
                        );
                        if send_result.is_err() {
                            break;
                        }
                    }
                    let fp = std::mem::take(&mut read_pl.fp);
                    for f in fp {
                        mb_bseq_close(Some(f));
                    }
                    crate::stage_time::flush_local();
                });
                scope.spawn(move || {
                    let mut map_pl = pipeline_t {
                        n_fp: n,
                        n_threads: if opt.n_thread <= 2 { opt.n_thread } else { 3 },
                        n_base: 0,
                        n_seq: 0,
                        mb_size: opt.mb_size,
                        opt,
                        fp: Vec::new(),
                        idx,
                        worker_pool,
                        output: String::new(),
                        output_writer: None,
                        output_error: false,
                    };
                    loop {
                        let recv_result = crate::stage_time::measure(
                            crate::stage_time::Bucket::PipeMapRecv,
                            || read_rx.recv(),
                        );
                        let Ok(s0) = recv_result else {
                            break;
                        };
                        if let Some(s1) = worker_pipeline(&mut map_pl, 1, Some(s0)) {
                            let mut s1 = s1;
                            release_step_tbuf(&mut s1);
                            let send_result = crate::stage_time::measure(
                                crate::stage_time::Bucket::PipeMapSend,
                                || map_tx.send(s1),
                            );
                            if send_result.is_err() {
                                break;
                            }
                        }
                    }
                    crate::stage_time::flush_local();
                });
                let mut out_pl = pipeline_t {
                    n_fp: n,
                    n_threads: if opt.n_thread <= 2 { opt.n_thread } else { 3 },
                    n_base: 0,
                    n_seq: 0,
                    mb_size: opt.mb_size,
                    opt,
                    fp: Vec::new(),
                    idx,
                    worker_pool,
                    output: String::new(),
                    output_writer,
                    output_error: false,
                };
                loop {
                    let recv_result =
                        crate::stage_time::measure(crate::stage_time::Bucket::PipeOutRecv, || {
                            map_rx.recv()
                        });
                    let Ok(s1) = recv_result else {
                        break;
                    };
                    worker_pipeline(&mut out_pl, 2, Some(s1));
                    if out_pl.output_error {
                        break;
                    }
                }
                crate::stage_time::flush_local();
                if out_pl.output_error {
                    (-1, String::new())
                } else if let Some(writer) = out_pl.output_writer.as_mut() {
                    if writer.flush().is_err() {
                        (-1, String::new())
                    } else {
                        (0, out_pl.output)
                    }
                } else {
                    (0, out_pl.output)
                }
            });
        }
        let mut pl = pipeline_t {
            n_fp: n,
            n_threads: if opt.n_thread <= 2 { opt.n_thread } else { 3 },
            n_base: 0,
            n_seq: 0,
            mb_size: opt.mb_size,
            opt,
            fp,
            idx,
            worker_pool,
            output: String::new(),
            output_writer,
            output_error: false,
        };
        while let Some(s0) = worker_pipeline(&mut pl, 0, None) {
            let Some(s1) = worker_pipeline(&mut pl, 1, Some(s0)) else {
                break;
            };
            worker_pipeline(&mut pl, 2, Some(s1));
            if pl.output_error {
                break;
            }
        }
        let fp = std::mem::take(&mut pl.fp);
        for f in fp {
            mb_bseq_close(Some(f));
        }
        if pl.output_error {
            (-1, String::new())
        } else if let Some(writer) = pl.output_writer.as_mut() {
            if writer.flush().is_err() {
                (-1, String::new())
            } else {
                (0, pl.output)
            }
        } else {
            (0, pl.output)
        }
    };
    if let Some(writer) = output_writer {
        return run(fp, Some(writer));
    }
    if let Some(fp_out) = output_file.as_mut() {
        let (ret, _) = run(fp, Some(fp_out));
        if ret != 0 {
            return (ret, String::new());
        }
        if fp_out.flush().is_err() {
            return (-1, String::new());
        }
        return (0, String::new());
    }
    run(fp, None)
}

/// Original C static function `usage` from `minibwa/map-main.c:296`.
pub fn usage(to_stdout: bool, opt: &mb_opt_t) -> (i32, String) {
    let mut out = String::new();
    out.push_str("Usage: minibwa map [options] <in.idx> <in.fastq>\n");
    out.push_str("Options:\n");
    out.push_str("  Common:\n");
    out.push_str("    -a               output SAM (PAF by default)\n");
    out.push_str(&format!(
        "    -t INT           number of worker threads [{}]\n",
        opt.n_thread
    ));
    out.push_str(&format!(
        "    -l NUM           treat reads <NUM as short reads in the default adaptive mode [{}]\n",
        opt.max_sr_len
    ));
    out.push_str(
        "    -R STR           SAM read group line in a format like '@RG\\tID:foo\\tSM:bar' []\n",
    );
    out.push_str("    -b STR           output a base alignment tag: cs, ds or MD []\n");
    out.push_str("    --hic            map Hi-C reads; equivalent to option -5P\n");
    out.push_str("    --meth           map *directional* bisulfite sequencing reads\n");
    out.push_str("  Mapping:\n");
    out.push_str(&format!(
        "    -k INT           min seed length [{}]\n",
        opt.min_len
    ));
    out.push_str(&format!(
        "    -c NUM           max seed occurrences [{}]\n",
        opt.max_occ
    ));
    out.push_str(&format!(
        "    -g NUM           max gap size, controlling extension and chain breaking [{}]\n",
        opt.max_gap
    ));
    out.push_str(&format!("    -w NUM           bandwidth [{}]\n", opt.bw));
    out.push_str(&format!(
        "    -W NUM           long bandwidth (for long reads or the adaptive mode) [{}]\n",
        opt.bw_long
    ));
    out.push_str(&format!(
        "    -m INT           min chaining score [{}]\n",
        opt.min_chain_score
    ));
    out.push_str(&format!(
        "    -p FLOAT         min secondary-to-primary score ratio [{}]\n",
        opt.pri_ratio
    ));
    out.push_str(&format!(
        "    -N INT           retain at most INT secondary alignments [{}]\n",
        opt.best_n
    ));
    out.push_str("    --chain-only     perform chaining only without base alignment\n");
    out.push_str(
        "    -x STR           preset (sr, lr or adap for mixed short/long reads) [adap]\n",
    );
    out.push_str("  Alignment:\n");
    out.push_str(&format!(
        "    -A INT           matching score [{}]\n",
        opt.a
    ));
    out.push_str(&format!(
        "    -B INT           mismatching openalty [{}]\n",
        opt.b
    ));
    out.push_str(&format!(
        "    -O INT1[,INT2]   gap open penalty [{},{}]\n",
        opt.q, opt.q2
    ));
    out.push_str(&format!(
        "    -E INT1[,INT2]   gap extension penalty [{},{}]\n",
        opt.e, opt.e2
    ));
    out.push_str(&format!(
        "    -s INT           suppress alignment with DP score lower than INT*{{-A}} [{}]\n",
        opt.min_dp_max
    ));
    out.push_str("  Paired-end:\n");
    out.push_str("    -P               skip pairing and mate resuce\n");
    out.push_str(&format!(
        "    --rescue=INT     mate rescue for up to INT candidates; 0 to skip rescue [{}]\n",
        opt.max_rescue
    ));
    out.push_str("  Input/Output:\n");
    out.push_str("    -o FILE          output file name [stdout]\n");
    out.push_str("    -u               don't output unmapped reads\n");
    out.push_str("    --outn=INT       output up to INT secondary alignments [0]\n");
    out.push_str("    -y               copy FASTA/Q comments to output\n");
    out.push_str("    -Y               use soft clipping for supplementary alignments\n");
    out.push_str(
        "    -5               take the alignment with the smallest query position as primary\n",
    );
    out.push_str(
        "    -K NUM1[,NUM2]   process NUM1-NUM2 bp of query sequences in a batch [100m,1g]\n",
    );
    out.push_str("    --version        print version number\n");
    out.push_str("    --help           print this help message\n");
    (if to_stdout { 0 } else { 1 }, out)
}

/// Original C static function `yes_or_no` from `minibwa/map-main.c:338`.
pub fn yes_or_no(
    opt: &mut mb_opt_t,
    flag: u64,
    option_name: &str,
    arg: &str,
    yes_to_set: i32,
) -> Option<String> {
    if yes_to_set != 0 {
        if c_str_eq(arg, "yes") || c_str_eq(arg, "y") {
            opt.flag |= flag;
            None
        } else if c_str_eq(arg, "no") || c_str_eq(arg, "n") {
            opt.flag &= !flag;
            None
        } else {
            Some(format!(
                "[WARNING]\u{1b}[1;31m option '--{}' only accepts 'yes' or 'no'.\u{1b}[0m\n",
                option_name
            ))
        }
    } else if c_str_eq(arg, "yes") || c_str_eq(arg, "y") {
        opt.flag &= !flag;
        None
    } else if c_str_eq(arg, "no") || c_str_eq(arg, "n") {
        opt.flag |= flag;
        None
    } else {
        Some(format!(
            "[WARNING]\u{1b}[1;31m option '--{}' only accepts 'yes' or 'no'.\u{1b}[0m\n",
            option_name
        ))
    }
}

fn c_strtol_i32(s: &str) -> (i32, usize) {
    let (value, used) = kom_strtol(s);
    (value as i32, used)
}

fn c_atoi_i32(s: &str) -> i32 {
    c_strtol_i32(s).0
}

fn c_atof_f32(s: &str) -> f32 {
    kom_atof(s) as f32
}

/// Original C global function `main_map` from `minibwa/map-main.c:351`.
pub fn main_map(argv: &[String]) -> (i32, String) {
    let has_output_writer = MAIN_MAP_OUTPUT_WRITER.with(|writer| writer.borrow().is_some());
    let opt_str = "x:o:k:c:m:p:A:B:U:b:O:E:t:K:N:PyYR:aul:w:W:g:5s:";
    let long_options = [
        ko_longopt_t {
            name: Some("kalloc".into()),
            has_arg: 0,
            val: 301,
        },
        ko_longopt_t {
            name: Some("outn".into()),
            has_arg: 1,
            val: 302,
        },
        ko_longopt_t {
            name: Some("pe-predef".into()),
            has_arg: 2,
            val: 303,
        },
        ko_longopt_t {
            name: Some("rescue".into()),
            has_arg: 1,
            val: 304,
        },
        ko_longopt_t {
            name: Some("eqx".into()),
            has_arg: 0,
            val: 305,
        },
        ko_longopt_t {
            name: Some("pe".into()),
            has_arg: 1,
            val: 306,
        },
        ko_longopt_t {
            name: Some("long".into()),
            has_arg: 2,
            val: 307,
        },
        ko_longopt_t {
            name: Some("adap".into()),
            has_arg: 1,
            val: 308,
        },
        ko_longopt_t {
            name: Some("chain-only".into()),
            has_arg: 0,
            val: 309,
        },
        ko_longopt_t {
            name: Some("meth".into()),
            has_arg: 0,
            val: 310,
        },
        ko_longopt_t {
            name: Some("hic".into()),
            has_arg: 0,
            val: 311,
        },
        ko_longopt_t {
            name: Some("dbg-aln-seq".into()),
            has_arg: 0,
            val: 601,
        },
        ko_longopt_t {
            name: Some("dbg-anchor".into()),
            has_arg: 0,
            val: 602,
        },
        ko_longopt_t {
            name: Some("dbg-seed".into()),
            has_arg: 0,
            val: 603,
        },
        ko_longopt_t {
            name: Some("dbg-qname".into()),
            has_arg: 0,
            val: 604,
        },
        ko_longopt_t {
            name: Some("dbg-aln-pe".into()),
            has_arg: 0,
            val: 605,
        },
        ko_longopt_t {
            name: Some("dbg-an-pos".into()),
            has_arg: 0,
            val: 606,
        },
        ko_longopt_t {
            name: Some("version".into()),
            has_arg: 0,
            val: 901,
        },
        ko_longopt_t {
            name: Some("help".into()),
            has_arg: 0,
            val: 902,
        },
    ];
    let argc = argv.len() as i32;
    let mut args = argv.to_vec();
    let mut mo = mb_opt_t::default();
    mb_opt_init(&mut mo);
    let mut o = KETOPT_INIT.clone();
    loop {
        let c = ketopt(&mut o, argc, &mut args, 1, opt_str, Some(&long_options));
        if c < 0 {
            break;
        }
        if c == 'x' as i32 {
            if mb_opt_preset(&mut mo, o.arg.as_deref().unwrap_or("")) < 0 {
                return (
                    1,
                    format!(
                        "[ERROR] unknown preset '{}'\n",
                        c_str(o.arg.as_deref().unwrap_or(""))
                    ),
                );
            }
        } else if c == ':' as i32 {
            return (1, "[ERROR] missing option argument\n".to_string());
        } else if c == '?' as i32 {
            let i = (o.i - 1).max(0) as usize;
            return (
                1,
                format!(
                    "[ERROR] unknown option in \"{}\"\n",
                    args.get(i).map(|s| c_str(s)).unwrap_or("")
                ),
            );
        }
    }
    let mut fn_out: Option<String> = None;
    let mut rg_line: Option<String> = None;
    o = KETOPT_INIT.clone();
    loop {
        let c = ketopt(&mut o, argc, &mut args, 1, opt_str, Some(&long_options));
        if c < 0 {
            break;
        }
        let arg = o.arg.as_deref().unwrap_or("");
        if c == 'k' as i32 {
            mo.min_len = c_atoi_i32(arg);
        } else if c == 'c' as i32 {
            mo.max_occ = kom_parse_num(arg).0 as i32;
        } else if c == 'p' as i32 {
            mo.pri_ratio = c_atof_f32(arg);
        } else if c == 'm' as i32 {
            mo.min_chain_score = c_atoi_i32(arg);
        } else if c == 'N' as i32 {
            mo.best_n = c_atoi_i32(arg);
        } else if c == 'A' as i32 {
            mo.a = c_atoi_i32(arg);
        } else if c == 'B' as i32 {
            mo.b = c_atoi_i32(arg);
        } else if c == 'U' as i32 {
            mo.pen_unpair = c_atoi_i32(arg);
        } else if c == 'l' as i32 {
            mo.max_sr_len = kom_parse_num(arg).0 as i32;
        } else if c == 'g' as i32 {
            mo.max_gap = kom_parse_num(arg).0 as i32;
        } else if c == 'w' as i32 {
            mo.bw = kom_parse_num(arg).0 as i32;
        } else if c == 'W' as i32 {
            mo.bw_long = kom_parse_num(arg).0 as i32;
        } else if c == 'a' as i32 {
            mo.flag |= MB_F_SAM;
        } else if c == 'u' as i32 {
            mo.flag |= MB_F_NO_UNMAP;
        } else if c == 'y' as i32 {
            mo.flag |= MB_F_COPY_COMMENT;
        } else if c == 'Y' as i32 {
            mo.flag |= MB_F_SUPP_SOFT;
        } else if c == '5' as i32 {
            mo.flag |= MB_F_PRIMARY5;
        } else if c == 'P' as i32 {
            mo.flag |= MB_F_NO_PAIRING;
        } else if c == 's' as i32 {
            mo.min_dp_max = c_atoi_i32(arg);
        } else if c == 'o' as i32 {
            fn_out = Some(arg.to_string());
        } else if c == 't' as i32 {
            mo.n_thread = c_atoi_i32(arg);
        } else if c == 'R' as i32 {
            rg_line = Some(arg.to_string());
        } else if c == 'K' as i32 {
            let (x, used) = kom_parse_num(arg);
            mo.mb_size = x;
            mo.max_mb_size = x;
            if arg.as_bytes().get(used) == Some(&b',') {
                mo.max_mb_size = kom_parse_num(&arg[used + 1..]).0;
            }
            if mo.max_mb_size < mo.mb_size {
                mo.max_mb_size = mo.mb_size;
            }
        } else if c == 'O' as i32 {
            let (q, used) = c_strtol_i32(arg);
            mo.q = q;
            mo.q2 = q;
            if arg.as_bytes().get(used) == Some(&b',') {
                mo.q2 = c_strtol_i32(&arg[used + 1..]).0;
            }
        } else if c == 'E' as i32 {
            let (e, used) = c_strtol_i32(arg);
            mo.e = e;
            mo.e2 = e;
            if arg.as_bytes().get(used) == Some(&b',') {
                mo.e2 = c_strtol_i32(&arg[used + 1..]).0;
            }
        } else if c == 'b' as i32 {
            mo.flag &= !(MB_F_WRITE_CS | MB_F_WRITE_DS | MB_F_WRITE_MD);
            if c_str_eq(arg, "cs") {
                mo.flag |= MB_F_WRITE_CS;
            } else if c_str_eq(arg, "ds") {
                mo.flag |= MB_F_WRITE_DS;
            } else if c_str_eq(arg, "MD") || c_str_eq(arg, "md") {
                mo.flag |= MB_F_WRITE_MD;
            } else {
                mo.flag |= MB_F_WRITE_CS;
                eprintln!(
                        "[WARNING]\u{1b}[1;31m -b only takes 'cs', 'ds' or 'MD'. Invalid values are assumed to be 'cs'.\u{1b}[0m"
                    );
            }
        } else if c == 301 {
            if o.arg.is_none() {
                missing_kalloc_arg_fatal_like_c();
            }
            if let Some(warning) = yes_or_no(&mut mo, MB_F_NO_KALLOC, "kalloc", arg, 0) {
                eprint!("{warning}");
            }
        } else if c == 302 {
            mo.out_n = c_atoi_i32(arg);
        } else if c == 303 {
            mo.flag |= MB_F_PE_PREDEF;
        } else if c == 304 {
            mo.max_rescue = c_atoi_i32(arg);
        } else if c == 305 {
            mo.flag |= MB_F_EQX;
        } else if c == 306 {
            if let Some(warning) = yes_or_no(&mut mo, MB_F_PE, "pe", arg, 1) {
                eprint!("{warning}");
            }
        } else if c == 307 {
            if o.arg.is_none() {
                mo.flag |= MB_F_LONG;
            } else if let Some(warning) = yes_or_no(&mut mo, MB_F_LONG, "long", arg, 1) {
                eprint!("{warning}");
            }
        } else if c == 308 {
            if let Some(warning) = yes_or_no(&mut mo, MB_F_ADAP, "adap", arg, 1) {
                eprint!("{warning}");
            }
        } else if c == 309 {
            mo.flag |= MB_F_NO_ALN;
        } else if c == 310 {
            mo.flag |= MB_F_METH;
        } else if c == 311 {
            mo.flag |= MB_F_PRIMARY5 | MB_F_NO_PAIRING;
        } else if c == 601 {
            KOM_DBG_FLAG.fetch_or(MB_DBG_ALN_SEQ, Ordering::Relaxed);
        } else if c == 602 {
            KOM_DBG_FLAG.fetch_or(MB_DBG_ANCHOR, Ordering::Relaxed);
        } else if c == 603 {
            KOM_DBG_FLAG.fetch_or(MB_DBG_SEED, Ordering::Relaxed);
        } else if c == 604 {
            KOM_DBG_FLAG.fetch_or(MB_DBG_QNAME, Ordering::Relaxed);
        } else if c == 605 {
            KOM_DBG_FLAG.fetch_or(MB_DBG_ALN_PE, Ordering::Relaxed);
        } else if c == 606 {
            KOM_DBG_FLAG.fetch_or(MB_DBG_AN_POS, Ordering::Relaxed);
        } else if c == 901 {
            return (0, MB_VERSION.to_string() + "\n");
        } else if c == 902 {
            let (ret, text) = usage(true, &mo);
            return (ret, text);
        } else if c == ':' as i32 || c == '?' as i32 {
            return (1, String::new());
        }
    }
    if argc - o.ind < 2 {
        let (ret, text) = usage(false, &mo);
        return (ret, text);
    }
    let Some(idx) = mb_idx_load(&args[o.ind as usize], ((mo.flag & MB_F_METH) != 0) as i32) else {
        kom_panic("main_map", "failed to load the index.");
    };
    eprintln!(
        "[M::main_map::{:.3}*{:.2}] index loaded",
        kom_realtime(),
        kom_percent_cpu()
    );
    let inputs = args[o.ind as usize + 1..]
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let mut out = String::new();
    if (mo.flag & MB_F_SAM) != 0 {
        let mut hdr = kstring_t::default();
        let argv_refs = args.iter().map(String::as_str).collect::<Vec<_>>();
        if mb_fmt_sam_hdr(
            &mut hdr,
            Some(&idx.l2b),
            rg_line.as_deref(),
            Some(MB_VERSION),
            &argv_refs,
        ) < 0
        {
            return (1, String::new());
        }
        if has_output_writer {
            let write_ok = MAIN_MAP_OUTPUT_WRITER.with(|writer| {
                // SAFETY: main_map_write installs this pointer only for the
                // synchronous dynamic extent of this main_map call.
                writer
                    .borrow()
                    .map(|ptr| unsafe { (&mut *ptr).write_all(&hdr.s[..hdr.l]).is_ok() })
                    .unwrap_or(false)
            });
            if !write_ok {
                return (-1, String::new());
            }
        } else {
            out.push_str(&String::from_utf8_lossy(&hdr.s[..hdr.l]));
        }
    }
    let writes_to_stdout = fn_out.as_deref().is_none_or(|name| c_str_eq(name, "-"));
    let (_ret, body) = mb_map_file(&mo, &idx, inputs.len() as i32, &inputs, fn_out.as_deref());
    if writes_to_stdout && !has_output_writer {
        out.push_str(&body);
    }
    (0, out)
}

pub fn main_map_write(argv: &[String], output_writer: &mut dyn Write) -> (i32, String) {
    MAIN_MAP_OUTPUT_WRITER.with(|writer| {
        // SAFETY: the pointer is restored before main_map_write returns,
        // and main_map only borrows it synchronously through the
        // thread-local slot.
        let output_writer = unsafe {
            std::mem::transmute::<&mut dyn Write, *mut (dyn Write + 'static)>(output_writer)
        };
        let old = writer.replace(Some(output_writer));
        let ret = main_map(argv);
        writer.replace(old);
        ret
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map_algo::mb_idx_load;
    use crate::options::{mb_opt_init, MB_F_NO_ALN};

    #[test]
    fn usage_formats_map_defaults() {
        let mut opt = mb_opt_t::default();
        mb_opt_init(&mut opt);
        let (ret, out) = usage(true, &opt);
        assert_eq!(ret, 0);
        assert!(out.contains("Usage: minibwa map [options] <in.idx> <in.fastq>\n"));
        assert!(out.contains(&format!(
            "    -k INT           min seed length [{}]\n",
            opt.min_len
        )));
        assert!(out.contains("    --help           print this help message\n"));
    }

    #[test]
    fn mb_map_file_string_path_handles_raw_comment_bytes() {
        let dir = std::env::temp_dir().join(format!(
            "minibwa_rs_map_string_raw_comment_{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let reads = dir.join("reads.fa");
        std::fs::write(
            &reads,
            b">raw cc:Z:\xff\nGATCACAGGTCTATCACCCTATTAACCACTCACGGGAGCTCTCCATGCAT\n",
        )
        .unwrap();

        let idx = mb_idx_load("minibwa/chrM-human", 0).expect("load index");
        let mut opt = mb_opt_t::default();
        mb_opt_init(&mut opt);
        opt.flag |= MB_F_COPY_COMMENT;
        let reads_s = reads.to_string_lossy().into_owned();

        let result =
            std::panic::catch_unwind(|| mb_map_file(&opt, &idx, 1, &[reads_s.as_str()], None));
        assert!(result.is_ok());
        let (ret, out) = result.unwrap();
        assert_eq!(ret, 0);
        assert!(out.contains("cc:Z:"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn yes_or_no_sets_clears_and_reports_bad_values() {
        let mut opt = mb_opt_t::default();
        assert_eq!(
            yes_or_no(&mut opt, MB_F_NO_ALN, "chain-only", "yes", 1),
            None
        );
        assert_ne!(opt.flag & MB_F_NO_ALN, 0);
        assert_eq!(
            yes_or_no(&mut opt, MB_F_NO_ALN, "chain-only", "no", 1),
            None
        );
        assert_eq!(opt.flag & MB_F_NO_ALN, 0);
        assert_eq!(
            yes_or_no(&mut opt, MB_F_NO_ALN, "chain-only", "no", 0),
            None
        );
        assert_ne!(opt.flag & MB_F_NO_ALN, 0);
        assert!(yes_or_no(&mut opt, MB_F_NO_ALN, "chain-only", "maybe", 1)
            .unwrap()
            .contains("only accepts 'yes' or 'no'"));
    }

    #[test]
    fn option_string_values_stop_at_nul_like_strcmp() {
        let mut opt = mb_opt_t::default();
        assert_eq!(
            yes_or_no(&mut opt, MB_F_NO_ALN, "chain-only", "yes\0hidden", 1),
            None
        );
        assert_ne!(opt.flag & MB_F_NO_ALN, 0);
        assert_eq!(
            yes_or_no(&mut opt, MB_F_NO_ALN, "chain-only", "no\0hidden", 1),
            None
        );
        assert_eq!(opt.flag & MB_F_NO_ALN, 0);

        let (ret, out) = main_map(&["map".into(), "-x".into(), "bad\0hidden".into()]);
        assert_eq!(ret, 1);
        assert_eq!(out, "[ERROR] unknown preset 'bad'\n");

        let (ret, out) = main_map(&["map".into(), "--definitely-not\0hidden".into()]);
        assert_eq!(ret, 1);
        assert_eq!(out, "[ERROR] unknown option in \"--definitely-not\"\n");
    }

    #[test]
    fn numeric_helpers_follow_libc_boundaries() {
        assert_eq!(c_strtol_i32("  -42xyz"), (-42, 5));
        assert_eq!(c_strtol_i32("0x10"), (0, 1));
        assert_eq!(c_strtol_i32("12\0hidden"), (12, 2));
        assert_eq!(c_atof_f32("0x1.8p2tail"), 6.0);
        assert!(c_atof_f32("inf").is_infinite());
        assert!(c_atof_f32("nan").is_nan());
        assert_eq!(c_atof_f32("0.25\0hidden"), 0.25);
    }

    #[test]
    fn map_file_maps_real_chrm_fastq_to_paf_text() {
        let idx = mb_idx_load("minibwa/chrM-human", 0).expect("load index");
        let mut opt = mb_opt_t::default();
        mb_opt_init(&mut opt);
        opt.flag |= MB_F_NO_ALN;
        opt.mb_size = 1000;
        opt.max_mb_size = 1000;
        let mut fq = std::env::temp_dir();
        fq.push(format!(
            "minibwa_rs_map_file_{}_{}.fq",
            std::process::id(),
            crate::kommon::kom_realtime().to_bits()
        ));
        std::fs::write(
                &fq,
                b"@r0\nGATCACAGGTCTATCACCCTATTAACCACTCACGGGAGCTCTCCATGCAT\n+\nFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF\n",
            )
            .unwrap();
        let path = fq.to_string_lossy().into_owned();
        let (ret, out) = mb_map_file(&opt, &idx, 1, &[path.as_str()], None);
        assert_eq!(ret, 0);
        assert!(out.contains("r0\t"));
        assert!(out.contains("\tchrM\t"));
        let _ = std::fs::remove_file(fq);
    }

    #[test]
    fn map_file_input_path_stops_at_embedded_nul_like_c() {
        let idx = mb_idx_load("minibwa/chrM-human", 0).expect("load index");
        let mut opt = mb_opt_t::default();
        mb_opt_init(&mut opt);
        opt.flag |= MB_F_NO_ALN;
        opt.mb_size = 1000;
        opt.max_mb_size = 1000;
        let mut fq = std::env::temp_dir();
        fq.push(format!(
            "minibwa_rs_map_file_nul_path_{}_{}.fq",
            std::process::id(),
            crate::kommon::kom_realtime().to_bits()
        ));
        std::fs::write(
                &fq,
                b"@r0\nGATCACAGGTCTATCACCCTATTAACCACTCACGGGAGCTCTCCATGCAT\n+\nFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF\n",
            )
            .unwrap();
        let path = fq.to_string_lossy().into_owned() + "\0hidden";
        let (ret, out) = mb_map_file(&opt, &idx, 1, &[path.as_str()], None);
        assert_eq!(ret, 0);
        assert!(out.contains("r0\t"));
        assert!(out.contains("\tchrM\t"));
        let _ = std::fs::remove_file(fq);
    }

    #[test]
    fn main_map_dash_o_dash_returns_stdout_like_c() {
        let mut fq = std::env::temp_dir();
        fq.push(format!(
            "minibwa_rs_main_map_dash_stdout_{}_{}.fq",
            std::process::id(),
            crate::kommon::kom_realtime().to_bits()
        ));
        std::fs::write(
                &fq,
                b"@r0\nGATCACAGGTCTATCACCCTATTAACCACTCACGGGAGCTCTCCATGCAT\n+\nFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF\n",
            )
            .unwrap();
        let path = fq.to_string_lossy().into_owned();
        let base_args = vec![
            "map".to_string(),
            "--chain-only".to_string(),
            "minibwa/chrM-human".to_string(),
            path.clone(),
        ];
        let dash_args = vec![
            "map".to_string(),
            "--chain-only".to_string(),
            "-o".to_string(),
            "-".to_string(),
            "minibwa/chrM-human".to_string(),
            path,
        ];
        let (base_ret, base_out) = main_map(&base_args);
        let (dash_ret, dash_out) = main_map(&dash_args);
        assert_eq!(base_ret, 0);
        assert_eq!(dash_ret, 0);
        assert_eq!(dash_out, base_out);
        assert!(dash_out.contains("r0\t"));
        let _ = std::fs::remove_file(fq);
    }

    #[test]
    fn main_map_dash_o_nul_suffix_returns_stdout_like_c() {
        let mut fq = std::env::temp_dir();
        fq.push(format!(
            "minibwa_rs_main_map_dash_nul_stdout_{}_{}.fq",
            std::process::id(),
            crate::kommon::kom_realtime().to_bits()
        ));
        std::fs::write(
                &fq,
                b"@r0\nGATCACAGGTCTATCACCCTATTAACCACTCACGGGAGCTCTCCATGCAT\n+\nFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF\n",
            )
            .unwrap();
        let path = fq.to_string_lossy().into_owned();
        let base_args = vec![
            "map".to_string(),
            "--chain-only".to_string(),
            "minibwa/chrM-human".to_string(),
            path.clone(),
        ];
        let dash_args = vec![
            "map".to_string(),
            "--chain-only".to_string(),
            "-o".to_string(),
            "-\0hidden".to_string(),
            "minibwa/chrM-human".to_string(),
            path,
        ];
        let (base_ret, base_out) = main_map(&base_args);
        let (dash_ret, dash_out) = main_map(&dash_args);
        assert_eq!(base_ret, 0);
        assert_eq!(dash_ret, 0);
        assert_eq!(dash_out, base_out);
        assert!(dash_out.contains("r0\t"));
        let _ = std::fs::remove_file(fq);
    }

    #[test]
    fn main_map_write_dash_o_dash_uses_writer_like_stdout() {
        let mut fq = std::env::temp_dir();
        fq.push(format!(
            "minibwa_rs_main_map_write_dash_stdout_{}_{}.fq",
            std::process::id(),
            crate::kommon::kom_realtime().to_bits()
        ));
        std::fs::write(
                &fq,
                b"@r0\nGATCACAGGTCTATCACCCTATTAACCACTCACGGGAGCTCTCCATGCAT\n+\nFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF\n",
            )
            .unwrap();
        let args = vec![
            "map".to_string(),
            "--chain-only".to_string(),
            "-o".to_string(),
            "-".to_string(),
            "minibwa/chrM-human".to_string(),
            fq.to_string_lossy().into_owned(),
        ];
        let mut written = Vec::new();
        let (ret, returned) = main_map_write(&args, &mut written);
        assert_eq!(ret, 0);
        assert!(returned.is_empty());
        assert!(String::from_utf8_lossy(&written).contains("r0\t"));
        let _ = std::fs::remove_file(fq);
    }

    #[test]
    fn map_file_processes_multiple_read_batches() {
        let idx = mb_idx_load("minibwa/chrM-human", 0).expect("load index");
        let mut opt = mb_opt_t::default();
        mb_opt_init(&mut opt);
        opt.flag |= MB_F_NO_ALN;
        opt.mb_size = 20;
        opt.max_mb_size = 20;
        let mut fq = std::env::temp_dir();
        fq.push(format!(
            "minibwa_rs_map_file_batches_{}_{}.fq",
            std::process::id(),
            crate::kommon::kom_realtime().to_bits()
        ));
        std::fs::write(
                &fq,
                b"@r0\nGATCACAGGTCTATCACCCTATTAACCACTCACGGGAGCTCTCCATGCAT\n+\nFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF\n@r1\nATCACAGGTCTATCACCCTATTAACCACTCACGGGAGCTCTCCATGCATG\n+\nFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF\n",
            )
            .unwrap();
        let path = fq.to_string_lossy().into_owned();
        let (ret, out) = mb_map_file(&opt, &idx, 1, &[path.as_str()], None);
        assert_eq!(ret, 0);
        assert!(out.contains("r0\t"));
        assert!(out.contains("r1\t"));
        let _ = std::fs::remove_file(fq);
    }

    #[test]
    fn main_map_parses_options_and_maps_real_chrm_fastq() {
        let mut fq = std::env::temp_dir();
        fq.push(format!(
            "minibwa_rs_main_map_{}_{}.fq",
            std::process::id(),
            crate::kommon::kom_realtime().to_bits()
        ));
        std::fs::write(
                &fq,
                b"@r0\nGATCACAGGTCTATCACCCTATTAACCACTCACGGGAGCTCTCCATGCAT\n+\nFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF\n",
            )
            .unwrap();
        let args = vec![
            "map".to_string(),
            "--chain-only".to_string(),
            "-K".to_string(),
            "1k,1k".to_string(),
            "minibwa/chrM-human".to_string(),
            fq.to_string_lossy().into_owned(),
        ];
        let (ret, out) = main_map(&args);
        assert_eq!(ret, 0);
        assert!(out.contains("r0\t"));
        assert!(out.contains("\tchrM\t"));
        let _ = std::fs::remove_file(fq);
    }
}
