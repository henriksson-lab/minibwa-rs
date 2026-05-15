
#![allow(unused_variables, dead_code, non_snake_case)]

use crate::align::MB_CIGAR_STR;
use crate::bseq::{mb_bseq_close, mb_bseq_open, mb_bseq_read};
use crate::map_algo::{mb_idx_ctg_len, mb_idx_ctg_name, mb_idx_destroy, mb_idx_load, mb_map};
use crate::options::{mb_opt_init, mb_opt_t};

/// Original C global function `main` from `minibwa/api-test/ex-one.c:9`.
pub fn main(argv: &[String]) -> (i32, String) {
    let mut opt = mb_opt_t::default();
    mb_opt_init(&mut opt);
    if argv.len() < 3 {
        return (1, "Usage: mbmap-sgl <idxPrefix> <query.fa>\n".to_string());
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
    loop {
        let mut n_seq = 0;
        let seqs = mb_bseq_read(&mut fp, 1, 0, 0, 0, 1, 1, &mut n_seq);
        if n_seq <= 0 {
            break;
        }
        for s in seqs {
            let mut n_hit = 0;
            let hit = mb_map(
                &opt,
                &idx,
                s.l_seq as i32,
                &s.seq,
                0,
                &mut n_hit,
                None,
                Some(&s.name),
            );
            for h in hit.iter().take(n_hit as usize) {
                let strand = if h.rev() != 0 { '-' } else { '+' };
                out.push_str(&format!(
                    "{}\t{}\t{}\t{}\t{}\t",
                    s.name, s.l_seq, h.qs, h.qe, strand
                ));
                out.push_str(&format!(
                    "{}\t{}\t{}\t{}\t{}\t{}\t{}\tcg:Z:",
                    mb_idx_ctg_name(&idx, h.tid as i32).unwrap_or("*"),
                    mb_idx_ctg_len(&idx, h.tid as i32),
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
    }
    mb_idx_destroy(Some(idx));
    mb_bseq_close(Some(fp));
    (0, out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_ex_one_maps_real_chrm_fixture() {
        let argv = vec![
            "mbmap-sgl".to_string(),
            "minibwa/chrM-human".to_string(),
            "minibwa/test/chrM-read_1.fa.gz".to_string(),
        ];
        let (status, out) = main(&argv);
        assert_eq!(status, 0);
        assert!(out.contains("\tcg:Z:"));
    }
}
