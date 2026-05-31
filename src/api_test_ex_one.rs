#![allow(unused_variables, dead_code, non_snake_case)]

use crate::align::MB_CIGAR_STR;
use crate::fastmap::RawKseqReader;
use crate::map_algo::{mb_idx_ctg_len, mb_idx_ctg_name, mb_idx_destroy, mb_idx_load, mb_map};
use crate::options::{mb_opt_init, mb_opt_t};
use std::io::Write;

/// Original C global function `main` from `minibwa/api-test/ex-one.c:9`.
pub fn main(argv: &[String]) -> (i32, String) {
    let (status, out) = main_bytes(argv);
    (status, String::from_utf8_lossy(&out).into_owned())
}

pub fn main_bytes(argv: &[String]) -> (i32, Vec<u8>) {
    let mut opt = mb_opt_t::default();
    mb_opt_init(&mut opt);
    if argv.len() < 3 {
        return (1, b"Usage: mbmap-sgl <idxPrefix> <query.fa>\n".to_vec());
    }
    let mut fp = match RawKseqReader::open(&argv[2]) {
        Some(fp) => fp,
        None => return (1, Vec::new()),
    };
    let idx = match mb_idx_load(&argv[1], 0) {
        Some(idx) => idx,
        None => return (1, Vec::new()),
    };
    let mut out = Vec::new();
    while let Some(s) = fp.read() {
        let seq = String::from_utf8_lossy(AsRef::<[u8]>::as_ref(&s.seq));
        let name = String::from_utf8_lossy(AsRef::<[u8]>::as_ref(&s.name));
        let mut n_hit = 0;
        let hit = mb_map(
            &opt,
            &idx,
            s.seq.len() as i32,
            &seq,
            0,
            &mut n_hit,
            None,
            Some(&name),
        );
        for h in hit.iter().take(n_hit as usize) {
            let strand = if h.rev() != 0 { '-' } else { '+' };
            write_c_bytes(&mut out, &s.name);
            let _ = write!(out, "\t{}\t{}\t{}\t{}\t", s.seq.len(), h.qs, h.qe, strand);
            write_c_str(&mut out, mb_idx_ctg_name(&idx, h.tid as i32).unwrap_or("*"));
            let _ = write!(
                out,
                "\t{}\t{}\t{}\t{}\t{}\t{}\tcg:Z:",
                mb_idx_ctg_len(&idx, h.tid as i32),
                h.ts,
                h.te,
                h.mlen,
                h.blen,
                h.mapq
            );
            if let Some(extra) = &h.p {
                for &c in extra.cigar().iter().take(extra.n_cigar as usize) {
                    let op = MB_CIGAR_STR.as_bytes()[(c & 0xf) as usize] as char;
                    let _ = write!(out, "{}{}", c >> 4, op);
                }
            }
            out.push(b'\n');
        }
    }
    mb_idx_destroy(Some(idx));
    (0, out)
}

fn write_c_bytes(out: &mut Vec<u8>, bytes: &[u8]) {
    let end = bytes.iter().position(|&c| c == 0).unwrap_or(bytes.len());
    out.extend_from_slice(&bytes[..end]);
}

fn write_c_str(out: &mut Vec<u8>, text: &str) {
    write_c_bytes(out, text.as_bytes());
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

    #[test]
    fn api_ex_one_bytes_preserves_raw_read_name() {
        let dir = std::env::temp_dir().join(format!(
            "minibwa_rs_api_one_raw_name_{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let reads = dir.join("reads.fa");
        std::fs::write(
            &reads,
            b">raw\xff-name\nGATCACAGGTCTATCACCCTATTAACCACTCACGGGAGCTCTCCATGCAT\n",
        )
        .unwrap();
        let argv = vec![
            "mbmap-sgl".to_string(),
            "minibwa/chrM-human".to_string(),
            reads.to_string_lossy().into_owned(),
        ];
        let (status, out) = main_bytes(&argv);
        assert_eq!(status, 0);
        assert!(out
            .windows(b"raw\xff-name\t".len())
            .any(|w| w == b"raw\xff-name\t"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn api_ex_one_target_name_output_stops_at_nul_like_printf() {
        let mut out = Vec::new();
        write_c_str(&mut out, "chr\0hidden");
        assert_eq!(out, b"chr");
    }
}
