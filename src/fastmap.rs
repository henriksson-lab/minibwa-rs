#![allow(unused_variables, dead_code, non_snake_case, non_camel_case_types)]

use crate::bwt::{
    mb_bwt_sa_batch, mb_bwt_smem, mb_bwt_smem_batch_ref_with_queue, mb_sai_t, mb_sai_v,
    mb_sai_v_clear, mb_smem_entry_ref, tiny_queue_t,
};
use crate::ketopt::{ketopt, ko_longopt_t, KETOPT_INIT};
use crate::kommon::{kom_atoi, kstring_t, KOM_NT4_TABLE};
use crate::l2bit::l2b_intv2cid;
use crate::map_algo::{mb_idx_load, mb_idx_t};
use flate2::bufread::MultiGzDecoder;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct batch_seq1_t {
    pub name: Vec<u8>,
    pub l_seq: i32,
    pub seq: Vec<u8>,
    pub v: mb_sai_v,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct RawKseqRecord {
    pub name: Vec<u8>,
    pub seq: Vec<u8>,
}

pub(crate) struct RawKseqReader {
    reader: Box<dyn BufRead + Send>,
    pending_header: Option<Vec<u8>>,
    eof: bool,
}

impl RawKseqReader {
    pub(crate) fn open(path: &str) -> Option<Self> {
        let path = cstr_path(path);
        let raw: Box<dyn Read + Send> = if path == "-" {
            Box::new(io::stdin())
        } else {
            Box::new(File::open(path).ok()?)
        };
        let mut buffered = BufReader::new(raw);
        let is_gzip = buffered.fill_buf().ok()?.starts_with(&[0x1f, 0x8b]);
        let reader: Box<dyn BufRead + Send> = if is_gzip {
            Box::new(BufReader::new(MultiGzDecoder::new(buffered)))
        } else {
            Box::new(buffered)
        };
        Some(Self {
            reader,
            pending_header: None,
            eof: false,
        })
    }

    pub(crate) fn read(&mut self) -> Option<RawKseqRecord> {
        if self.eof {
            return None;
        }
        let header = if let Some(header) = self.pending_header.take() {
            header
        } else {
            let mut line = Vec::new();
            loop {
                line.clear();
                let n = self.reader.read_until(b'\n', &mut line).ok()?;
                if n == 0 {
                    self.eof = true;
                    return None;
                }
                if let Some(pos) = line.iter().position(|&c| c == b'>' || c == b'@') {
                    break strip_line_ending(&line[pos + 1..]).to_vec();
                }
            }
        };
        let name_end = header
            .iter()
            .position(|&c| c.is_ascii_whitespace())
            .unwrap_or(header.len());
        let name = header[..name_end].to_vec();
        let mut seq = Vec::new();
        let mut line = Vec::new();
        loop {
            line.clear();
            let n = self.reader.read_until(b'\n', &mut line).ok()?;
            if n == 0 {
                self.eof = true;
                break;
            }
            if line.first().is_some_and(|&c| c == b'>' || c == b'@') {
                self.pending_header = Some(strip_line_ending(&line[1..]).to_vec());
                break;
            }
            if line.first() == Some(&b'+') {
                let mut qual_len = 0usize;
                while qual_len < seq.len() {
                    line.clear();
                    let n = self.reader.read_until(b'\n', &mut line).ok()?;
                    if n == 0 {
                        self.eof = true;
                        return None;
                    }
                    qual_len += strip_line_ending(&line).len();
                }
                if qual_len != seq.len() {
                    self.eof = true;
                    return None;
                }
                break;
            }
            seq.extend_from_slice(strip_line_ending(&line));
        }
        Some(RawKseqRecord { name, seq })
    }
}

fn cstr_path(path: &str) -> &str {
    let bytes = path.as_bytes();
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    &path[..end]
}

fn strip_line_ending(mut bytes: &[u8]) -> &[u8] {
    if bytes.last() == Some(&b'\n') {
        bytes = &bytes[..bytes.len() - 1];
    }
    if bytes.len() > 1 && bytes.last() == Some(&b'\r') {
        bytes = &bytes[..bytes.len() - 1];
    }
    bytes
}

fn encode_nt4(seq: &[u8]) -> Vec<u8> {
    seq.iter().map(|&c| KOM_NT4_TABLE[c as usize]).collect()
}

fn raise_sigsegv() -> ! {
    #[cfg(unix)]
    unsafe {
        libc::signal(libc::SIGSEGV, libc::SIG_DFL);
        libc::raise(libc::SIGSEGV);
    }
    std::process::abort();
}

fn ks_put_c_bytes(out: &mut kstring_t, bytes: &[u8]) {
    let end = bytes.iter().position(|&c| c == 0).unwrap_or(bytes.len());
    ks_put_bytes(out, &bytes[..end]);
}

fn ks_put_bytes(out: &mut kstring_t, bytes: &[u8]) {
    let end = out.l + bytes.len();
    if end + 1 > out.s.len() {
        out.s.resize(end + 1, 0);
        out.m = out.s.len();
    }
    out.s[out.l..end].copy_from_slice(bytes);
    out.l = end;
    out.s[out.l] = 0;
}

fn ks_put_u64(out: &mut kstring_t, mut x: u64) {
    let mut buf = [0u8; 20];
    let mut i = buf.len();
    loop {
        i -= 1;
        buf[i] = b'0' + (x % 10) as u8;
        x /= 10;
        if x == 0 {
            break;
        }
    }
    ks_put_bytes(out, &buf[i..]);
}

fn ks_put_i64(out: &mut kstring_t, x: i64) {
    if x < 0 {
        ks_put_bytes(out, b"-");
        ks_put_u64(out, x.unsigned_abs());
    } else {
        ks_put_u64(out, x as u64);
    }
}

fn ks_put_i32(out: &mut kstring_t, x: i32) {
    ks_put_i64(out, x as i64);
}

fn emit_kstring(
    out: &kstring_t,
    emitted: &mut String,
    output_writer: &mut Option<&mut dyn Write>,
) -> bool {
    if let Some(writer) = output_writer.as_deref_mut() {
        writer.write_all(&out.s[..out.l]).is_ok()
    } else {
        emitted.push_str(&String::from_utf8_lossy(&out.s[..out.l]));
        true
    }
}

/// Original C static function `batch_smem` from `minibwa/fastmap.c:17`.
pub fn batch_smem(idx: &mb_idx_t, n: i32, t: &mut [batch_seq1_t], min_len: i32, min_occ: i32) {
    let mut tq = tiny_queue_t::default();
    batch_smem_with_queue(idx, n, t, min_len, min_occ, &mut tq);
}

fn batch_smem_with_queue(
    idx: &mb_idx_t,
    n: i32,
    t: &mut [batch_seq1_t],
    min_len: i32,
    min_occ: i32,
    tq: &mut tiny_queue_t,
) {
    let mut v = Vec::with_capacity(n as usize);
    for ti in t.iter_mut().take(n as usize) {
        mb_sai_v_clear(&mut ti.v);
        v.push(std::mem::take(&mut ti.v));
    }
    let mut s = Vec::with_capacity(n as usize);
    let v_ptr = v.as_mut_ptr();
    for (i, ti) in t.iter().take(n as usize).enumerate() {
        s.push(mb_smem_entry_ref {
            min_len,
            min_occ,
            st: 0,
            en: ti.l_seq,
            q: ti.seq.as_ptr(),
            v: unsafe { v_ptr.add(i) },
            stage: 0,
            x: 0,
            i: 0,
            kmer: 0,
            p: mb_sai_t::default(),
        });
    }
    mb_bwt_smem_batch_ref_with_queue((), &idx.bwt, n, &mut s, &mut v, tq);
    for (ti, vi) in t.iter_mut().zip(v.into_iter()).take(n as usize) {
        ti.v = vi;
    }
}

/// Original C static function `write_intv` from `minibwa/fastmap.c:35`.
pub fn write_intv(
    idx: &mb_idx_t,
    p: &mb_sai_t,
    max_size_out: i32,
    sa: &mut [u64],
    out: &mut kstring_t,
) {
    let len = (p.info & 0xffff_ffff).wrapping_sub(p.info >> 32);
    ks_put_bytes(out, b"EM\t");
    ks_put_u64(out, p.info >> 32);
    ks_put_bytes(out, b"\t");
    ks_put_u64(out, p.info & 0xffff_ffff);
    ks_put_bytes(out, b"\t");
    ks_put_u64(out, p.size);
    if p.size <= max_size_out as u64 {
        let n_sa = p.size as usize;
        for (j, slot) in sa.iter_mut().take(n_sa).enumerate() {
            *slot = p.x[0] + j as u64;
        }
        mb_bwt_sa_batch((), &idx.bwt, p.size as i64, sa);
        for &pos in sa.iter().take(n_sa) {
            let mut rev = 0;
            let mut cst = 0i64;
            let cid = l2b_intv2cid(&idx.l2b, pos, pos + len, &mut cst, &mut rev);
            if cid < 0 {
                ks_put_bytes(out, b"\t.");
            } else {
                let name = &idx.l2b.ctg[cid as usize].name;
                ks_put_bytes(out, b"\t");
                ks_put_c_bytes(out, name.as_bytes());
                ks_put_bytes(out, if rev != 0 { b":-" } else { b":+" });
                ks_put_i64(out, cst + 1);
            }
        }
    } else {
        ks_put_bytes(out, b"\t*");
    }
    ks_put_bytes(out, b"\n");
}

/// Original C static function `process_batch` from `minibwa/fastmap.c:56`.
pub fn process_batch(
    idx: &mb_idx_t,
    n: i32,
    t: &mut [batch_seq1_t],
    min_len: i32,
    min_occ: i32,
    max_size_out: i32,
    sa: &mut [u64],
    out: &mut kstring_t,
) -> String {
    let mut emitted = String::new();
    let mut tq = tiny_queue_t::default();
    let mut output_writer = None;
    if process_batch_append(
        idx,
        n,
        t,
        min_len,
        min_occ,
        max_size_out,
        sa,
        out,
        &mut emitted,
        &mut tq,
        &mut output_writer,
    ) {
        emitted
    } else {
        String::new()
    }
}

fn process_batch_append(
    idx: &mb_idx_t,
    n: i32,
    t: &mut [batch_seq1_t],
    min_len: i32,
    min_occ: i32,
    max_size_out: i32,
    sa: &mut [u64],
    out: &mut kstring_t,
    emitted: &mut String,
    tq: &mut tiny_queue_t,
    output_writer: &mut Option<&mut dyn Write>,
) -> bool {
    batch_smem_with_queue(idx, n, t, min_len, min_occ, tq);
    for ti in t.iter_mut().take(n as usize) {
        out.l = 0;
        ks_put_bytes(out, b"SQ\t");
        ks_put_c_bytes(out, &ti.name);
        ks_put_bytes(out, b"\t");
        ks_put_i32(out, ti.l_seq);
        ks_put_bytes(out, b"\n");
        for p in ti.v.a.iter().take(ti.v.n) {
            write_intv(idx, p, max_size_out, sa, out);
        }
        ks_put_bytes(out, b"//\n");
        if !emit_kstring(out, emitted, output_writer) {
            return false;
        }
    }
    true
}

/// Original C static function `usage_fastmap` from `minibwa/fastmap.c:63`.
pub fn usage_fastmap(
    to_stdout: bool,
    min_len: i32,
    min_occ: i32,
    max_size_out: i32,
    max_seq: i32,
) -> (i32, String) {
    let text = format!(
            "Usage: minibwa fastmap [options] <idx-prefix> <in.fq>\nOptions:\n  -l INT     min seed length [{min_len}]\n  -s INT     min interval size [{min_occ}]\n  -w INT     max interval size to output coordinates [{max_size_out}]\n  -b INT     batch size [{max_seq}]\n  --help     print this help message\n"
        );
    (if to_stdout { 0 } else { 1 }, text)
}

/// Original C global function `main_fastmap` from `minibwa/fastmap.c:73`.
pub fn main_fastmap(argv: &[String]) -> (i32, String) {
    main_fastmap_inner(argv, None)
}

pub fn main_fastmap_write(argv: &[String], output_writer: &mut dyn Write) -> (i32, String) {
    main_fastmap_inner(argv, Some(output_writer))
}

fn main_fastmap_inner(argv: &[String], mut output_writer: Option<&mut dyn Write>) -> (i32, String) {
    let long_opts = [ko_longopt_t {
        name: Some("help".into()),
        has_arg: 0,
        val: 901,
    }];
    let argc = argv.len() as i32;
    let mut args = argv.to_vec();
    let mut o = KETOPT_INIT.clone();
    let mut min_len = 19;
    let mut min_occ = 1;
    let mut max_size_out = 20;
    let mut max_seq = 1;
    loop {
        let c = ketopt(&mut o, argc, &mut args, 1, "l:s:w:b:", Some(&long_opts));
        if c < 0 {
            break;
        }
        if c == 'l' as i32 {
            min_len = kom_atoi(o.arg.as_deref().unwrap_or("0"));
        } else if c == 's' as i32 {
            min_occ = kom_atoi(o.arg.as_deref().unwrap_or("0"));
        } else if c == 'w' as i32 {
            max_size_out = kom_atoi(o.arg.as_deref().unwrap_or("0"));
        } else if c == 'b' as i32 {
            max_seq = kom_atoi(o.arg.as_deref().unwrap_or("0"));
        } else if c == 901 {
            return usage_fastmap(true, min_len, min_occ, max_size_out, max_seq);
        }
    }
    if argc - o.ind < 2 {
        return usage_fastmap(false, min_len, min_occ, max_size_out, max_seq);
    }
    let Some(idx) = mb_idx_load(&args[o.ind as usize], 0) else {
        raise_sigsegv();
    };
    let Some(mut fp) = RawKseqReader::open(&args[o.ind as usize + 1]) else {
        return (0, String::new());
    };
    let mut emitted = String::new();
    let mut out = kstring_t::default();
    let mut sa = vec![0u64; max_size_out.max(0) as usize];
    let mut tq = tiny_queue_t::default();
    let mut n_seq = 0;
    let mut batch = if max_seq > 1 {
        vec![batch_seq1_t::default(); max_seq as usize]
    } else {
        Vec::new()
    };
    while let Some(r) = fp.read() {
        if max_seq > 1 {
            if n_seq == max_seq {
                if !process_batch_append(
                    &idx,
                    n_seq,
                    &mut batch,
                    min_len,
                    min_occ,
                    max_size_out,
                    &mut sa,
                    &mut out,
                    &mut emitted,
                    &mut tq,
                    &mut output_writer,
                ) {
                    return (-1, String::new());
                }
                n_seq = 0;
            }
            let slot = &mut batch[n_seq as usize];
            n_seq += 1;
            slot.name.clear();
            slot.name.extend_from_slice(&r.name);
            slot.l_seq = r.seq.len() as i32;
            slot.seq = encode_nt4(&r.seq);
        } else {
            let seq = encode_nt4(&r.seq);
            out.l = 0;
            ks_put_bytes(&mut out, b"SQ\t");
            ks_put_c_bytes(&mut out, &r.name);
            ks_put_bytes(&mut out, b"\t");
            ks_put_u64(&mut out, r.seq.len() as u64);
            ks_put_bytes(&mut out, b"\n");
            let mut x = 0i64;
            let mut a = Vec::<mb_sai_t>::new();
            while x < r.seq.len() as i64 {
                let mut p = mb_sai_t::default();
                x = mb_bwt_smem(
                    &idx.bwt,
                    r.seq.len() as u32,
                    &seq,
                    x,
                    min_len as i64,
                    min_occ as i64,
                    &mut p,
                );
                if p.size > 0 {
                    a.push(p);
                }
            }
            for p in &a {
                write_intv(&idx, p, max_size_out, &mut sa, &mut out);
            }
            ks_put_bytes(&mut out, b"//\n");
            if !emit_kstring(&out, &mut emitted, &mut output_writer) {
                return (-1, String::new());
            }
        }
    }
    if max_seq > 1 && n_seq > 0 {
        if !process_batch_append(
            &idx,
            n_seq,
            &mut batch,
            min_len,
            min_occ,
            max_size_out,
            &mut sa,
            &mut out,
            &mut emitted,
            &mut tq,
            &mut output_writer,
        ) {
            return (-1, String::new());
        }
    }
    if let Some(writer) = output_writer {
        if writer.flush().is_err() {
            return (-1, String::new());
        }
    }
    (0, emitted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bwt::mb_bwt_init;
    use crate::l2bit::{l2b_ctg_t, l2b_t};

    #[test]
    fn write_intv_emits_interval_summary_and_size_cutoff() {
        let idx = mb_idx_t {
            is_meth: 0,
            l2b: l2b_t {
                tot_len: 100,
                n_ctg: 1,
                ctg: vec![l2b_ctg_t {
                    name: "ctg".to_string(),
                    len: 100,
                    off: 0,
                    ..Default::default()
                }],
                ..Default::default()
            },
            bwt: mb_bwt_init(),
        };
        let p = mb_sai_t {
            x: [5, 0],
            size: 3,
            info: (2 << 32) | 8,
        };
        let mut out = kstring_t::default();
        let mut sa = vec![0; 3];
        write_intv(&idx, &p, 2, &mut sa, &mut out);
        assert_eq!(String::from_utf8_lossy(&out.s[..out.l]), "EM\t2\t8\t3\t*\n");
    }

    #[test]
    fn fastmap_c_string_writer_truncates_at_nul() {
        let mut out = kstring_t::default();
        ks_put_c_bytes(&mut out, b"ctg\0hidden");
        assert_eq!(&out.s[..out.l], b"ctg");
    }

    #[test]
    fn usage_fastmap_formats_defaults_and_status() {
        let (status, text) = usage_fastmap(false, 19, 1, 20, 1);
        assert_eq!(status, 1);
        assert!(text.contains("Usage: minibwa fastmap"));
        assert!(text.contains("-l INT     min seed length [19]"));
    }

    #[test]
    fn main_fastmap_emits_smems_on_real_chrm_fixture() {
        let mut fq = std::env::temp_dir();
        fq.push(format!(
            "minibwa_rs_fastmap_{}_{}.fq",
            std::process::id(),
            crate::kommon::kom_realtime().to_bits()
        ));
        std::fs::write(
                &fq,
                b"@r0\nGATCACAGGTCTATCACCCTATTAACCACTCACGGGAGCTCTCCATGCAT\n+\nFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF\n",
            )
            .unwrap();
        let args = vec![
            "fastmap".to_string(),
            "-l".to_string(),
            "19".to_string(),
            "-w".to_string(),
            "2".to_string(),
            "minibwa/chrM-human".to_string(),
            fq.to_string_lossy().into_owned(),
        ];
        let (ret, out) = main_fastmap(&args);
        assert_eq!(ret, 0);
        assert!(out.starts_with("SQ\tr0\t"));
        assert!(out.contains("\nEM\t"));
        assert!(out.ends_with("//\n"));
        let _ = std::fs::remove_file(fq);
    }

    #[test]
    fn raw_kseq_reader_preserves_u_bases_for_fastmap_input() {
        let path = std::env::temp_dir().join(format!(
            "minibwa_rs_fastmap_raw_u_{}.fq",
            std::process::id()
        ));
        std::fs::write(&path, b"@r0 comment\nACUuT\n+\nIIIII\n").unwrap();

        let mut reader = RawKseqReader::open(&path.to_string_lossy()).expect("open raw fastq");
        let rec = reader.read().expect("read record");
        assert_eq!(rec.name, b"r0");
        assert_eq!(rec.seq, b"ACUuT");
        assert_eq!(encode_nt4(&rec.seq), vec![0, 1, 4, 4, 3]);
        assert!(reader.read().is_none());

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn raw_kseq_reader_preserves_non_utf8_name_and_sequence_bytes() {
        let path = std::env::temp_dir().join(format!(
            "minibwa_rs_fastmap_raw_bytes_{}.fa",
            std::process::id()
        ));
        std::fs::write(&path, b">r\xff comment\nAC\xfeT\n").unwrap();

        let mut reader = RawKseqReader::open(&path.to_string_lossy()).expect("open raw fasta");
        let rec = reader.read().expect("read record");
        assert_eq!(rec.name, b"r\xff");
        assert_eq!(rec.seq, b"AC\xfeT");
        assert_eq!(encode_nt4(&rec.seq), vec![0, 1, 4, 3]);
        assert!(reader.read().is_none());

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn raw_kseq_reader_open_path_stops_at_nul_like_gzopen() {
        let path = std::env::temp_dir().join(format!(
            "minibwa_rs_fastmap_nul_path_{}.fa",
            std::process::id()
        ));
        std::fs::write(&path, b">r0\nACGT\n").unwrap();
        let nul_path = format!("{}\0hidden", path.to_string_lossy());

        let mut reader = RawKseqReader::open(&nul_path).expect("open C-truncated path");
        let rec = reader.read().expect("read record");
        assert_eq!(rec.name, b"r0");
        assert_eq!(rec.seq, b"ACGT");

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn raw_kseq_reader_seeks_header_inside_leading_garbage_like_kseq() {
        let path = std::env::temp_dir().join(format!(
            "minibwa_rs_fastmap_seek_header_{}.fa",
            std::process::id()
        ));
        std::fs::write(&path, b"ignored bytes before >r0 comment\nACGT\n").unwrap();

        let mut reader = RawKseqReader::open(&path.to_string_lossy()).expect("open raw fasta");
        let rec = reader.read().expect("read record");
        assert_eq!(rec.name, b"r0");
        assert_eq!(rec.seq, b"ACGT");
        assert!(reader.read().is_none());

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn raw_kseq_reader_rejects_overlong_fastq_quality_like_kseq() {
        let path = std::env::temp_dir().join(format!(
            "minibwa_rs_fastmap_overlong_{}.fq",
            std::process::id()
        ));
        std::fs::write(&path, b"@bad\nACGT\n+\nFFFFF\n@next\nACGT\n+\nFFFF\n").unwrap();

        let mut reader = RawKseqReader::open(&path.to_string_lossy()).expect("open raw fastq");
        assert!(reader.read().is_none());

        let _ = std::fs::remove_file(path);
    }
}
