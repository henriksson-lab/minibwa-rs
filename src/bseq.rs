#![allow(unused_variables, dead_code, non_snake_case, non_camel_case_types)]

use flate2::bufread::MultiGzDecoder;
use std::alloc::{alloc, dealloc, handle_alloc_error, Layout};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::ptr::NonNull;

const CHECK_PAIR_THRES: u64 = 1000000;

pub struct mb_opt_str_t {
    ptr: NonNull<u8>,
}

unsafe impl Send for mb_opt_str_t {}
unsafe impl Sync for mb_opt_str_t {}

impl mb_opt_str_t {
    #[inline]
    pub fn from_string(s: String) -> Self {
        let bytes = s.into_bytes();
        Self::from_bytes(&bytes)
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let header = std::mem::size_of::<usize>();
        let size = header + bytes.len();
        let layout = Layout::from_size_align(size.max(header), std::mem::align_of::<usize>())
            .expect("valid optional string layout");
        let base = unsafe { alloc(layout) };
        let Some(base) = NonNull::new(base) else {
            handle_alloc_error(layout);
        };
        unsafe {
            (base.as_ptr() as *mut usize).write(bytes.len());
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), base.as_ptr().add(header), bytes.len());
            Self {
                ptr: NonNull::new_unchecked(base.as_ptr().add(header)),
            }
        }
    }

    #[inline]
    fn len(&self) -> usize {
        unsafe {
            let header = std::mem::size_of::<usize>();
            *((self.ptr.as_ptr().sub(header)) as *const usize)
        }
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        unsafe {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(self.ptr.as_ptr(), self.len()))
        }
    }
}

impl Clone for mb_opt_str_t {
    fn clone(&self) -> Self {
        Self::from_bytes(self.as_str().as_bytes())
    }
}

impl Drop for mb_opt_str_t {
    fn drop(&mut self) {
        let header = std::mem::size_of::<usize>();
        let len = self.len();
        let layout =
            Layout::from_size_align((header + len).max(header), std::mem::align_of::<usize>())
                .expect("valid optional string layout");
        unsafe {
            dealloc(self.ptr.as_ptr().sub(header), layout);
        }
    }
}

impl std::fmt::Debug for mb_opt_str_t {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

impl PartialEq for mb_opt_str_t {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for mb_opt_str_t {}

impl From<String> for mb_opt_str_t {
    fn from(value: String) -> Self {
        Self::from_string(value)
    }
}

impl From<&str> for mb_opt_str_t {
    fn from(value: &str) -> Self {
        Self::from_bytes(value.as_bytes())
    }
}

impl std::ops::Deref for mb_opt_str_t {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct mb_bseq1_t {
    pub l_seq: u64,
    pub id: u64,
    pub name: Box<str>,
    pub seq: Box<str>,
    pub qual: Option<mb_opt_str_t>,
    pub comment: Option<mb_opt_str_t>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct kvec_t<T> {
    pub n: usize,
    pub m: usize,
    pub a: Vec<T>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct kseq_t {
    pub name: String,
    pub comment: String,
    pub seq: String,
    pub qual: String,
}

pub struct mb_bseq_file_s {
    pub records: Vec<kseq_t>,
    pub pos: usize,
    pub s: Option<mb_bseq1_t>,
    reader: Option<Box<dyn BufRead + Send>>,
    pending_header: Option<(u8, String)>,
    last_record_name: Option<String>,
    eof: bool,
    pub parse_error: bool,
    pub parse_error_after: Option<String>,
    pub parse_error_reported: bool,
    pub suppress_parse_warnings: bool,
}
pub type mb_bseq_file_t = mb_bseq_file_s;

/// Original C global function `kvec_t` from `minibwa/bseq.c:8`.
pub fn kvec_t<T>() -> kvec_t<T> {
    kvec_t {
        n: 0,
        m: 0,
        a: Vec::new(),
    }
}

/// Original C static function `mb_qname_len` from `minibwa/bseq.h:26`.
pub fn mb_qname_len(s: &str) -> i32 {
    let b = s.as_bytes();
    let l = b.len();
    if l >= 3 && b[l - 1].is_ascii_digit() && b[l - 2] == b'/' {
        (l - 2) as i32
    } else {
        l as i32
    }
}

/// Original C static function `mb_qname_same` from `minibwa/bseq.h:33`.
pub fn mb_qname_same(s1: &str, s2: &str) -> i32 {
    let l1 = mb_qname_len(s1) as usize;
    let l2 = mb_qname_len(s2) as usize;
    (l1 == l2 && s1.as_bytes()[..l1] == s2.as_bytes()[..l2]) as i32
}

/// Original C global function `mb_bseq_open` from `minibwa/bseq.c:43`.
pub fn mb_bseq_open(fn_: Option<&str>) -> Option<mb_bseq_file_t> {
    let raw: Box<dyn Read + Send> = match fn_ {
        Some(path) if path != "-" => Box::new(File::open(path).ok()?),
        _ => Box::new(io::stdin()),
    };
    let mut buffered = BufReader::new(raw);
    let is_gzip = buffered.fill_buf().ok()?.starts_with(&[0x1f, 0x8b]);
    let reader: Box<dyn BufRead + Send> = if is_gzip {
        Box::new(BufReader::new(MultiGzDecoder::new(buffered)))
    } else {
        Box::new(buffered)
    };
    Some(mb_bseq_file_t {
        records: Vec::new(),
        pos: 0,
        s: None,
        reader: Some(reader),
        pending_header: None,
        last_record_name: None,
        eof: false,
        parse_error: false,
        parse_error_after: None,
        parse_error_reported: false,
        suppress_parse_warnings: false,
    })
}

/// Original C global function `mb_bseq_close` from `minibwa/bseq.c:55`.
pub fn mb_bseq_close(fp: Option<mb_bseq_file_t>) {
    drop(fp);
}

/// Original C static function `kstrdup` from `minibwa/bseq.c:62`.
pub fn kstrdup(s: &str) -> String {
    s.to_string()
}

/// Original C static function `kseq2bseq` from `minibwa/bseq.c:70`.
pub fn kseq2bseq(ks: kseq_t, s: &mut mb_bseq1_t, with_qual: i32, with_comment: i32) {
    if ks.name.is_empty() {
        eprintln!("[WARNING]\u{1b}[1;31m empty sequence name in the input.\u{1b}[0m");
    }
    s.name = ks.name.into_boxed_str();
    let mut seq = ks.seq.into_bytes();
    for c in &mut seq {
        if *c == b'u' || *c == b'U' {
            *c -= 1;
        }
    }
    s.l_seq = seq.len() as u64;
    s.seq = String::from_utf8(seq).unwrap().into_boxed_str();
    s.qual = if with_qual != 0 && !ks.qual.is_empty() {
        Some(mb_opt_str_t::from_string(ks.qual))
    } else {
        None
    };
    s.comment = if with_comment != 0 && !ks.comment.is_empty() {
        Some(mb_opt_str_t::from_string(ks.comment))
    } else {
        None
    };
}

macro_rules! kseq_read {
    ($fp:expr) => {{
        (|| -> Option<kseq_t> {
            let fp = &mut *$fp;
            if fp.parse_error || fp.eof {
                return None;
            }
            let (start_ch, header) = if let Some(header) = fp.pending_header.take() {
                header
            } else {
                let reader = fp.reader.as_mut()?;
                let mut line = String::new();
                loop {
                    line.clear();
                    let n = reader.read_line(&mut line).ok()?;
                    if n == 0 {
                        fp.eof = true;
                        return None;
                    }
                    let bytes = line.as_bytes();
                    if let Some(pos) = bytes.iter().position(|&c| c == b'>' || c == b'@') {
                        let start_ch = bytes[pos];
                        let mut header = String::from_utf8_lossy(&bytes[pos + 1..]).into_owned();
                        while header.ends_with('\n') || header.ends_with('\r') {
                            header.pop();
                        }
                        break (start_ch, header);
                    }
                }
            };
            let mut it = header.splitn(2, |c: char| c.is_ascii_whitespace());
            let name = it.next().unwrap_or("").to_string();
            let comment = it.next().unwrap_or("").to_string();
            let mut seq = Vec::new();
            let mut saw_plus = false;
            let mut line = String::new();
            loop {
                line.clear();
                let n = fp.reader.as_mut()?.read_line(&mut line).ok()?;
                if n == 0 {
                    fp.eof = true;
                    break;
                }
                let bytes = line.as_bytes();
                if bytes.iter().all(|&c| c == b'\n' || c == b'\r') {
                    continue;
                }
                if bytes[0] == b'>' || bytes[0] == b'@' {
                    let mut header = String::from_utf8_lossy(&bytes[1..]).into_owned();
                    while header.ends_with('\n') || header.ends_with('\r') {
                        header.pop();
                    }
                    fp.pending_header = Some((bytes[0], header));
                    break;
                }
                if bytes[0] == b'+' {
                    saw_plus = true;
                    break;
                }
                let mut end = bytes.len();
                while end > 0 && matches!(bytes[end - 1], b'\n' | b'\r') {
                    end -= 1;
                }
                seq.extend_from_slice(&bytes[..end]);
            }
            let mut qual = Vec::new();
            if start_ch == b'@' && saw_plus {
                while qual.len() < seq.len() {
                    line.clear();
                    let n = fp.reader.as_mut()?.read_line(&mut line).ok()?;
                    if n == 0 {
                        fp.eof = true;
                        break;
                    }
                    let bytes = line.as_bytes();
                    let mut end = bytes.len();
                    while end > 0 && matches!(bytes[end - 1], b'\n' | b'\r') {
                        end -= 1;
                    }
                    qual.extend_from_slice(&bytes[..end]);
                    if qual.len() >= seq.len() {
                        break;
                    }
                }
                if qual.len() != seq.len() {
                    fp.parse_error = true;
                    if fp.parse_error_after.is_none() {
                        fp.parse_error_after = fp.last_record_name.clone();
                    }
                    return None;
                }
            }
            fp.pos += 1;
            let rec = kseq_t {
                name,
                comment,
                seq: String::from_utf8(seq).unwrap(),
                qual: String::from_utf8(qual).unwrap(),
            };
            fp.last_record_name = Some(rec.name.clone());
            Some(rec)
        })()
    }};
}

/// Original C global function `mb_bseq_read` from `minibwa/bseq.c:85`.
pub fn mb_bseq_read(
    fp: &mut mb_bseq_file_t,
    chunk_size: i64,
    with_qual: i32,
    with_comment: i32,
    frag_mode: i32,
    min_cnt: i32,
    mut max_chunk_size: i64,
    n_: &mut i32,
) -> Vec<mb_bseq1_t> {
    let mut size = 0i64;
    let mut a = Vec::new();
    *n_ = 0;
    if let Some(s) = fp.s.take() {
        size = s.l_seq as i64;
        a.push(s);
    }
    if max_chunk_size < chunk_size {
        max_chunk_size = chunk_size;
    }
    while let Some(ks) = kseq_read!(fp) {
        let mut s = mb_bseq1_t::default();
        kseq2bseq(ks, &mut s, with_qual, with_comment);
        size += s.l_seq as i64;
        a.push(s);
        let to_stop = chunk_size <= 0
            || max_chunk_size <= 0
            || size >= max_chunk_size
            || (size >= chunk_size && a.len() >= min_cnt as usize);
        if to_stop {
            if frag_mode != 0
                && a.last()
                    .map(|x| x.l_seq < CHECK_PAIR_THRES)
                    .unwrap_or(false)
            {
                while let Some(ks) = kseq_read!(fp) {
                    let mut s = mb_bseq1_t::default();
                    kseq2bseq(ks, &mut s, with_qual, with_comment);
                    if mb_qname_same(&s.name, &a[a.len() - 1].name) != 0 {
                        a.push(s);
                    } else {
                        fp.s = Some(s);
                        break;
                    }
                }
            }
            break;
        }
    }
    if fp.parse_error && !fp.parse_error_reported && !fp.suppress_parse_warnings {
        if let Some(name) = &fp.parse_error_after {
            eprintln!(
                    "[WARNING]\u{1b}[1;31m failed to parse the FASTA/FASTQ record next to '{}'. Continue anyway.\u{1b}[0m",
                    name
                );
        } else {
            eprintln!(
                    "[WARNING]\u{1b}[1;31m failed to parse the first FASTA/FASTQ record. Continue anyway.\u{1b}[0m"
                );
        }
        fp.parse_error_reported = true;
    }
    *n_ = a.len() as i32;
    a
}

/// Original C global function `mb_bseq_read_frag` from `minibwa/bseq.c:133`.
pub fn mb_bseq_read_frag(
    n_fp: i32,
    fp: &mut [mb_bseq_file_t],
    chunk_size: i64,
    with_qual: i32,
    with_comment: i32,
    n_: &mut i32,
) -> Vec<mb_bseq1_t> {
    let mut size = 0i64;
    let mut a = Vec::new();
    *n_ = 0;
    if n_fp < 1 {
        return a;
    }
    loop {
        let mut read = Vec::with_capacity(n_fp as usize);
        let mut n_read = 0;
        for f in fp.iter_mut().take(n_fp as usize) {
            let rec = kseq_read!(f);
            if rec.is_some() {
                n_read += 1;
            }
            read.push(rec);
        }
        if n_read < n_fp {
            if n_read > 0 {
                eprintln!(
                        "[W::mb_bseq_read_frag]\u{1b}[1;31m query files have different number of records; extra records skipped.\u{1b}[0m"
                    );
            }
            break;
        }
        for rec in read.into_iter().flatten() {
            let mut s = mb_bseq1_t::default();
            kseq2bseq(rec, &mut s, with_qual, with_comment);
            size += s.l_seq as i64;
            a.push(s);
        }
        if size >= chunk_size {
            break;
        }
    }
    for f in fp.iter_mut().take(n_fp as usize) {
        if f.parse_error && !f.parse_error_reported && !f.suppress_parse_warnings {
            if let Some(name) = &f.parse_error_after {
                eprintln!(
                        "[WARNING]\u{1b}[1;31m failed to parse the FASTA/FASTQ record next to '{}'. Continue anyway.\u{1b}[0m",
                        name
                    );
            } else {
                eprintln!(
                        "[WARNING]\u{1b}[1;31m failed to parse the first FASTA/FASTQ record. Continue anyway.\u{1b}[0m"
                    );
            }
            f.parse_error_reported = true;
        }
    }
    *n_ = a.len() as i32;
    a
}

/// Original C global function `mb_bseq_eof` from `minibwa/bseq.c:163`.
pub fn mb_bseq_eof(fp: &mb_bseq_file_t) -> i32 {
    (fp.eof && fp.s.is_none()) as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qname_helpers_strip_pair_suffixes() {
        assert_eq!(mb_qname_len("read/1"), 4);
        assert_eq!(mb_qname_len("read/x"), 6);
        assert_eq!(mb_qname_same("read/1", "read/2"), 1);
        assert_eq!(mb_qname_same("read/1", "readx/2"), 0);
    }

    #[test]
    fn read_real_chrm_fasta_chunk() {
        let mut fp = mb_bseq_open(Some("minibwa/test/chrM-human.fa.gz")).expect("open fasta");
        let mut n = 0;
        let reads = mb_bseq_read(&mut fp, 1_000_000, 0, 1, 0, 1, 1_000_000, &mut n);
        assert_eq!(n, 1);
        assert_eq!(&*reads[0].name, "chrM");
        assert_eq!(reads[0].l_seq, 16569);
        assert!(reads[0].seq.starts_with("GATCACAGGTCTATCACCCT"));
        assert_eq!(mb_bseq_eof(&fp), 1);
    }

    #[test]
    fn read_real_paired_fragments_from_two_files() {
        let fp1 = mb_bseq_open(Some("minibwa/test/chrM-read_1.fa.gz")).expect("open read1");
        let fp2 = mb_bseq_open(Some("minibwa/test/chrM-read_2.fa.gz")).expect("open read2");
        let mut fps = vec![fp1, fp2];
        let mut n = 0;
        let reads = mb_bseq_read_frag(2, &mut fps, 200, 0, 0, &mut n);
        assert_eq!(n, 2);
        assert_eq!(reads.len(), 2);
        assert_eq!(mb_qname_same(&reads[0].name, &reads[1].name), 1);
        assert!(reads[0].l_seq > 0);
        assert!(reads[1].l_seq > 0);
    }

    #[test]
    fn read_sequence_keeps_internal_marker_characters() {
        let path = std::env::temp_dir().join(format!(
            "minibwa_rs_bseq_internal_markers_{}.fa",
            std::process::id()
        ));
        std::fs::write(&path, b">r0\nAC@GT+AC\n>r1\nTT\n").unwrap();
        let mut fp = mb_bseq_open(Some(&path.to_string_lossy())).expect("open marker fasta");
        let mut n = 0;
        let reads = mb_bseq_read(&mut fp, 1_000_000, 0, 0, 0, 1, 1_000_000, &mut n);
        assert_eq!(n, 2);
        assert_eq!(&*reads[0].name, "r0");
        assert_eq!(&*reads[0].seq, "AC@GT+AC");
        assert_eq!(&*reads[1].name, "r1");
        let _ = std::fs::remove_file(path);
    }
}
