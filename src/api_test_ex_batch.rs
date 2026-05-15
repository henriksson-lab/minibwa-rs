#![allow(unused_variables, dead_code, non_snake_case)]

use crate::align::MB_CIGAR_STR;
use crate::bseq::{mb_bseq_close, mb_bseq_open, mb_bseq_read};
use crate::map_algo::{
    mb_idx_ctg_len, mb_idx_ctg_name, mb_idx_destroy, mb_idx_load, mb_map_batch, mb_tbuf_t,
};
use crate::options::{mb_opt_init, mb_opt_t};

/// Original C static function `process_batch` from `minibwa/api-test/ex-batch.c:9`.
pub fn process_batch(
    opt: &mb_opt_t,
    idx: &crate::map_algo::mb_idx_t,
    tbuf: Option<&mut mb_tbuf_t>,
    n_seq: i32,
    qlen: &[i32],
    seq: &[Box<str>],
    name: &[Box<str>],
) -> String {
    let seq_refs = seq.iter().map(|s| &**s).collect::<Vec<_>>();
    let name_refs = name.iter().map(|s| &**s).collect::<Vec<_>>();
    let mut n_hit = vec![0i32; n_seq.max(0) as usize];
    let hit = mb_map_batch(
        opt,
        idx,
        n_seq,
        qlen,
        &seq_refs,
        &mut n_hit,
        tbuf,
        Some(&name_refs),
    );
    let mut out = String::new();
    for k in 0..n_seq as usize {
        for h in hit[k].iter().take(n_hit[k] as usize) {
            let strand = if h.rev() != 0 { '-' } else { '+' };
            out.push_str(&format!(
                "{}\t{}\t{}\t{}\t{}\t",
                name[k], qlen[k], h.qs, h.qe, strand
            ));
            out.push_str(&format!(
                "{}\t{}\t{}\t{}\t{}\t{}\t{}\tcg:Z:",
                mb_idx_ctg_name(idx, h.tid as i32).unwrap_or("*"),
                mb_idx_ctg_len(idx, h.tid as i32),
                h.ts,
                h.te,
                h.mlen,
                h.blen,
                h.mapq
            ));
            if let Some(extra) = &h.p {
                for &c in extra.cigar().iter().take(extra.n_cigar as usize) {
                    let op = MB_CIGAR_STR.as_bytes()[(c & 0xf) as usize] as char;
                    out.push_str(&format!("{}{}", c >> 4, op));
                }
            }
            out.push('\n');
        }
    }
    out
}

/// Original C global function `main` from `minibwa/api-test/ex-batch.c:35`.
pub fn main(argv: &[String]) -> (i32, String) {
    let mut opt = mb_opt_t::default();
    mb_opt_init(&mut opt);
    if argv.len() < 3 {
        return (1, "Usage: mbmap-batch <idxPrefix> <query.fa>\n".to_string());
    }
    let mut fp = match mb_bseq_open(Some(&argv[2])) {
        Some(fp) => fp,
        None => return (1, String::new()),
    };
    let idx = match mb_idx_load(&argv[1], 0) {
        Some(idx) => idx,
        None => {
            mb_bseq_close(Some(fp));
            return (1, String::new());
        }
    };
    let mut out = String::new();
    let mut qlen = Vec::new();
    let mut seq = Vec::new();
    let mut name = Vec::new();
    loop {
        let mut n_read = 0;
        let reads = mb_bseq_read(&mut fp, 1, 0, 0, 0, 1, 1, &mut n_read);
        if n_read <= 0 {
            break;
        }
        for s in reads {
            qlen.push(s.l_seq as i32);
            seq.push(s.seq);
            name.push(s.name);
            if seq.len() >= opt.sb_seq as usize {
                out.push_str(&process_batch(
                    &opt,
                    &idx,
                    None,
                    seq.len() as i32,
                    &qlen,
                    &seq,
                    &name,
                ));
                qlen.clear();
                seq.clear();
                name.clear();
            }
        }
    }
    if !seq.is_empty() {
        out.push_str(&process_batch(
            &opt,
            &idx,
            None,
            seq.len() as i32,
            &qlen,
            &seq,
            &name,
        ));
    }
    mb_idx_destroy(Some(idx));
    mb_bseq_close(Some(fp));
    (0, out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_ex_batch_matches_single_api_on_real_chrm_fixture() {
        let argv = vec![
            "mbmap-batch".to_string(),
            "minibwa/chrM-human".to_string(),
            "minibwa/test/chrM-read_1.fa.gz".to_string(),
        ];
        let (status, out) = main(&argv);
        assert_eq!(status, 0);
        assert!(out.contains("\tcg:Z:"));
        let single_argv = vec![
            "mbmap-sgl".to_string(),
            "minibwa/chrM-human".to_string(),
            "minibwa/test/chrM-read_1.fa.gz".to_string(),
        ];
        let (_, single_out) = crate::api_test_ex_one::main(&single_argv);
        assert_eq!(out, single_out);
    }
}
