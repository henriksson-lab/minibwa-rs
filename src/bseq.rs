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

    pub fn from_bytes(bytes: &[u8]) -> Self {
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

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.len()) }
    }
}

impl Clone for mb_opt_str_t {
    fn clone(&self) -> Self {
        Self::from_bytes(self.as_bytes())
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
        String::from_utf8_lossy(self.as_bytes()).fmt(f)
    }
}

impl PartialEq for mb_opt_str_t {
    fn eq(&self, other: &Self) -> bool {
        self.as_bytes() == other.as_bytes()
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
    pub comment: Vec<u8>,
    pub seq: String,
    pub qual: Vec<u8>,
}

pub struct mb_bseq_file_s {
    pub records: Vec<kseq_t>,
    pub pos: usize,
    pub s: Option<mb_bseq1_t>,
    reader: Option<Box<dyn BufRead + Send>>,
    #[cfg(feature = "plain-fastq-batch-io")]
    plain_fastq_batch_reader: Option<PlainFastqBatchState>,
    pending_header: Option<(u8, Vec<u8>)>,
    last_record_name: Option<String>,
    eof: bool,
    pub parse_error: bool,
    pub parse_error_after: Option<String>,
    pub parse_error_reported: bool,
    pub suppress_parse_warnings: bool,
}
pub type mb_bseq_file_t = mb_bseq_file_s;

#[cfg(feature = "plain-fastq-batch-io")]
struct PlainFastqBatchState {
    reader: PlainFastqBatchReader,
    records: Vec<mb_bseq1_t>,
    pos: usize,
    chunk_config: PlainFastqChunkConfig,
    pending_error: bool,
}

#[cfg(feature = "plain-fastq-batch-io")]
impl PlainFastqBatchState {
    fn new(path: &str) -> Option<Self> {
        let reader = File::open(path).ok()?;
        let chunk_config = plain_fastq_chunk_config();
        Some(Self {
            reader: PlainFastqBatchReader::new(reader, plain_fastq_buffer_size()),
            records: Vec::with_capacity(chunk_config.min_records),
            pos: 0,
            chunk_config,
            pending_error: false,
        })
    }

    fn next_bseq(&mut self, with_qual: i32, with_comment: i32) -> Result<Option<mb_bseq1_t>, ()> {
        loop {
            if self.pos < self.records.len() {
                let record = std::mem::take(&mut self.records[self.pos]);
                self.pos += 1;
                return Ok(Some(record));
            }
            if self.pending_error {
                self.pending_error = false;
                return Err(());
            }
            self.records.clear();
            self.pos = 0;
            let mut sink = PlainFastqBseqSink {
                records: &mut self.records,
                with_qual,
                with_comment,
            };
            match self
                .reader
                .next_chunk_with_sink(self.chunk_config, &mut sink)
            {
                Ok(Some(_chunk)) => {}
                Ok(None) => return Ok(None),
                Err(()) if !self.records.is_empty() => self.pending_error = true,
                Err(()) => return Err(()),
            }
        }
    }
}

#[cfg(feature = "plain-fastq-batch-io")]
struct PlainFastqBseqSink<'a> {
    records: &'a mut Vec<mb_bseq1_t>,
    with_qual: i32,
    with_comment: i32,
}

#[cfg(feature = "plain-fastq-batch-io")]
impl PlainFastqRecordSink for PlainFastqBseqSink<'_> {
    fn record(&mut self, record: PlainFastqVisitRecord<'_>) -> Result<(), ()> {
        self.records.push(plain_fastq_record_to_bseq(
            record,
            self.with_qual,
            self.with_comment,
        ));
        Ok(())
    }
}

#[cfg(feature = "plain-fastq-batch-io")]
#[derive(Clone, Copy)]
struct PlainFastqChunkConfig {
    target_bases: u64,
    min_records: usize,
    max_bases: u64,
}

#[cfg(feature = "plain-fastq-batch-io")]
impl PlainFastqChunkConfig {
    fn new(target_bases: u64) -> Self {
        Self {
            target_bases,
            min_records: 1,
            max_bases: 0,
        }
    }

    fn min_records(mut self, min_records: usize) -> Self {
        self.min_records = min_records.max(1);
        self
    }

    fn max_bases(mut self, max_bases: u64) -> Self {
        self.max_bases = max_bases;
        self
    }

    fn should_stop(self, records: u64, bases: u64) -> bool {
        if self.max_bases != 0 && bases >= self.max_bases {
            return true;
        }
        self.target_bases != 0 && bases >= self.target_bases && records >= self.min_records as u64
    }
}

#[cfg(feature = "plain-fastq-batch-io")]
#[derive(Clone, Copy)]
struct PlainFastqChunkStats {
    records: u64,
    bases: u64,
}

#[cfg(feature = "plain-fastq-batch-io")]
impl PlainFastqChunkStats {
    fn records(self) -> u64 {
        self.records
    }
}

#[cfg(feature = "plain-fastq-batch-io")]
trait PlainFastqRecordSink {
    fn record(&mut self, record: PlainFastqVisitRecord<'_>) -> Result<(), ()>;
}

#[cfg(feature = "plain-fastq-batch-io")]
#[derive(Clone, Copy)]
struct PlainFastqVisitRecord<'a> {
    name: &'a [u8],
    seq: &'a [u8],
    qual: &'a [u8],
}

#[cfg(feature = "plain-fastq-batch-io")]
impl<'a> PlainFastqVisitRecord<'a> {
    fn name(self) -> &'a [u8] {
        self.name
    }

    fn seq(self) -> &'a [u8] {
        self.seq
    }

    fn qual(self) -> &'a [u8] {
        self.qual
    }
}

#[cfg(feature = "plain-fastq-batch-io")]
struct PlainFastqBatchReader {
    reader: File,
    buf: Vec<u8>,
    len: usize,
    next_start: usize,
    eof: bool,
}

#[cfg(feature = "plain-fastq-batch-io")]
impl PlainFastqBatchReader {
    fn new(reader: File, buffer_size: usize) -> Self {
        Self {
            reader,
            buf: vec![0; buffer_size.max(1024)],
            len: 0,
            next_start: 0,
            eof: false,
        }
    }

    fn next_chunk_with_sink<S>(
        &mut self,
        config: PlainFastqChunkConfig,
        sink: &mut S,
    ) -> Result<Option<PlainFastqChunkStats>, ()>
    where
        S: PlainFastqRecordSink,
    {
        let mut stats = PlainFastqChunkStats {
            records: 0,
            bases: 0,
        };
        loop {
            self.compact_carry();
            self.fill_buffer()?;

            let mut pos = 0usize;
            loop {
                let Some(record) = parse_fastq_record(&self.buf[..self.len], pos, self.eof)? else {
                    break;
                };
                pos = record.next_start;
                sink.record(record.record)?;
                stats.records += 1;
                stats.bases += record.record.seq().len() as u64;
                self.next_start = pos;
                if config.should_stop(stats.records, stats.bases) {
                    return Ok(Some(stats));
                }
            }

            if stats.records > 0 && self.len == 0 && self.eof {
                return Ok(Some(stats));
            }
            if stats.records > 0 && self.next_start < self.len && !self.eof {
                continue;
            }
            if self.len == 0 && self.eof {
                return Ok(None);
            }
            if self.len == self.buf.len() {
                return Err(());
            }
        }
    }

    fn compact_carry(&mut self) {
        if self.next_start == 0 {
            return;
        }
        if self.next_start >= self.len {
            self.len = 0;
            self.next_start = 0;
            return;
        }
        let carry = self.len - self.next_start;
        self.buf.copy_within(self.next_start..self.len, 0);
        self.len = carry;
        self.next_start = 0;
    }

    fn fill_buffer(&mut self) -> Result<(), ()> {
        while !self.eof && self.len < self.buf.len() {
            let n = self
                .reader
                .read(&mut self.buf[self.len..])
                .map_err(|_| ())?;
            if n == 0 {
                self.eof = true;
                break;
            }
            self.len += n;
        }
        Ok(())
    }
}

#[cfg(feature = "plain-fastq-batch-io")]
struct ParsedPlainFastqRecord<'a> {
    record: PlainFastqVisitRecord<'a>,
    next_start: usize,
}

#[cfg(feature = "plain-fastq-batch-io")]
fn parse_fastq_record(
    buf: &[u8],
    start: usize,
    eof: bool,
) -> Result<Option<ParsedPlainFastqRecord<'_>>, ()> {
    if start >= buf.len() {
        return Ok(None);
    }
    let Some(name_end) = find_lf(buf, start) else {
        return if eof { Err(()) } else { Ok(None) };
    };
    let seq_start = name_end + 1;
    let Some(seq_end) = find_lf(buf, seq_start) else {
        return if eof { Err(()) } else { Ok(None) };
    };
    let plus_start = seq_end + 1;
    let Some(plus_end) = find_lf(buf, plus_start) else {
        return if eof { Err(()) } else { Ok(None) };
    };
    let qual_start = plus_end + 1;
    let Some(qual_end) = find_lf(buf, qual_start) else {
        return if eof { Err(()) } else { Ok(None) };
    };

    let name = strip_one_cr(&buf[start..name_end]);
    let seq = strip_one_cr(&buf[seq_start..seq_end]);
    let plus = strip_one_cr(&buf[plus_start..plus_end]);
    let qual = strip_one_cr(&buf[qual_start..qual_end]);
    if name.first() != Some(&b'@') || plus.first() != Some(&b'+') || seq.len() != qual.len() {
        return Err(());
    }

    Ok(Some(ParsedPlainFastqRecord {
        record: PlainFastqVisitRecord { name, seq, qual },
        next_start: qual_end + 1,
    }))
}

#[cfg(feature = "plain-fastq-batch-io")]
fn find_lf(buf: &[u8], start: usize) -> Option<usize> {
    let haystack = buf.get(start..)?;
    find_byte(haystack, b'\n').map(|offset| start + offset)
}

#[cfg(feature = "plain-fastq-batch-io")]
fn find_byte(haystack: &[u8], byte: u8) -> Option<usize> {
    if haystack.is_empty() {
        return None;
    }
    // SAFETY: `memchr` only reads the provided byte range. A non-null result
    // must point inside `haystack`.
    let ptr = unsafe { libc::memchr(haystack.as_ptr().cast(), byte as i32, haystack.len()) };
    if ptr.is_null() {
        None
    } else {
        Some(unsafe { (ptr as *const u8).offset_from(haystack.as_ptr()) as usize })
    }
}

#[cfg(feature = "plain-fastq-batch-io")]
fn strip_one_cr(bytes: &[u8]) -> &[u8] {
    bytes.strip_suffix(b"\r").unwrap_or(bytes)
}

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
    let l = b.iter().position(|&c| c == 0).unwrap_or(b.len());
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
    mb_bseq_open_with_plain_fastq_batch(fn_, true)
}

pub(crate) fn mb_bseq_open_with_plain_fastq_batch(
    fn_: Option<&str>,
    allow_plain_fastq_batch: bool,
) -> Option<mb_bseq_file_t> {
    let path = fn_.map(cstr_path);
    #[cfg(feature = "plain-fastq-batch-io")]
    let mut plain_fastq_batch_reader = None;
    #[cfg(not(feature = "plain-fastq-batch-io"))]
    let plain_fastq_batch_reader: Option<()> = None;
    #[cfg(feature = "plain-fastq-batch-io")]
    if let Some(path) = path {
        if allow_plain_fastq_batch && should_use_plain_fastq_batch_reader(path) {
            plain_fastq_batch_reader = PlainFastqBatchState::new(path);
        }
    }

    let raw: Box<dyn Read + Send> = match path {
        Some(path) if path != "-" && plain_fastq_batch_reader.is_none() => {
            Box::new(File::open(path).ok()?)
        }
        #[cfg(feature = "plain-fastq-batch-io")]
        Some(_) if plain_fastq_batch_reader.is_some() => Box::new(io::empty()),
        _ => Box::new(io::stdin()),
    };
    let mut buffered = BufReader::new(raw);
    let is_gzip = buffered.fill_buf().ok()?.starts_with(&[0x1f, 0x8b]);
    let reader: Option<Box<dyn BufRead + Send>> = if cfg!(feature = "plain-fastq-batch-io")
        && is_plain_fastq_batch_active(&plain_fastq_batch_reader)
    {
        None
    } else if is_gzip {
        Some(Box::new(BufReader::new(MultiGzDecoder::new(buffered))))
    } else {
        Some(Box::new(buffered))
    };
    Some(mb_bseq_file_t {
        records: Vec::new(),
        pos: 0,
        s: None,
        reader,
        #[cfg(feature = "plain-fastq-batch-io")]
        plain_fastq_batch_reader,
        pending_header: None,
        last_record_name: None,
        eof: false,
        parse_error: false,
        parse_error_after: None,
        parse_error_reported: false,
        suppress_parse_warnings: false,
    })
}

#[cfg(feature = "plain-fastq-batch-io")]
fn is_plain_fastq_batch_active(reader: &Option<PlainFastqBatchState>) -> bool {
    reader.is_some()
}

#[cfg(not(feature = "plain-fastq-batch-io"))]
fn is_plain_fastq_batch_active<T>(_reader: &T) -> bool {
    false
}

#[cfg(feature = "plain-fastq-batch-io")]
fn should_use_plain_fastq_batch_reader(path: &str) -> bool {
    if path == "-" || std::env::var_os("MINIBWA_RS_DISABLE_PLAIN_FASTQ_BATCH_IO").is_some() {
        return false;
    }
    if std::env::var_os("MINIBWA_RS_FORCE_PLAIN_FASTQ_BATCH_IO").is_some() {
        return true;
    }
    let lower = path.to_ascii_lowercase();
    lower.ends_with(".fq") || lower.ends_with(".fastq")
}

#[cfg(feature = "plain-fastq-batch-io")]
fn plain_fastq_buffer_size() -> usize {
    std::env::var("MINIBWA_RS_PLAIN_FASTQ_BUFFER_BYTES")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|&value| value >= 1024)
        .unwrap_or(8 * 1024 * 1024)
}

#[cfg(feature = "plain-fastq-batch-io")]
fn plain_fastq_chunk_config() -> PlainFastqChunkConfig {
    let target_bases = std::env::var("MINIBWA_RS_PLAIN_FASTQ_TARGET_BASES")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(10_000_000);
    let min_records = std::env::var("MINIBWA_RS_PLAIN_FASTQ_MIN_RECORDS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(40_000);
    PlainFastqChunkConfig::new(target_bases).min_records(min_records)
}

fn mb_bseq_open_legacy_for_test(fn_: Option<&str>) -> Option<mb_bseq_file_t> {
    let raw: Box<dyn Read + Send> = match fn_ {
        Some(path) if cstr_path(path) != "-" => Box::new(File::open(cstr_path(path)).ok()?),
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
        #[cfg(feature = "plain-fastq-batch-io")]
        plain_fastq_batch_reader: None,
        pending_header: None,
        last_record_name: None,
        eof: false,
        parse_error: false,
        parse_error_after: None,
        parse_error_reported: false,
        suppress_parse_warnings: false,
    })
}

fn cstr_path(path: &str) -> &str {
    let bytes = path.as_bytes();
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    &path[..end]
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
        Some(mb_opt_str_t::from_bytes(&ks.qual))
    } else {
        None
    };
    s.comment = if with_comment != 0 && !ks.comment.is_empty() {
        Some(mb_opt_str_t::from_bytes(&ks.comment))
    } else {
        None
    };
}

#[cfg(feature = "plain-fastq-batch-io")]
fn plain_fastq_record_to_bseq(
    record: PlainFastqVisitRecord<'_>,
    with_qual: i32,
    with_comment: i32,
) -> mb_bseq1_t {
    let header = record.name().strip_prefix(b"@").unwrap_or(record.name());
    let split = header
        .iter()
        .position(|&c| c.is_ascii_whitespace())
        .unwrap_or(header.len());
    let name = String::from_utf8_lossy(&header[..split]).into_owned();
    if name.is_empty() {
        eprintln!("[WARNING]\u{1b}[1;31m empty sequence name in the input.\u{1b}[0m");
    }
    let comment = if with_comment != 0 && split < header.len() {
        Some(mb_opt_str_t::from_bytes(&header[split + 1..]))
    } else {
        None
    };
    let seq = if find_byte(record.seq(), b'u').is_some() || find_byte(record.seq(), b'U').is_some()
    {
        let mut seq = record.seq().to_vec();
        for c in &mut seq {
            if *c == b'u' || *c == b'U' {
                *c -= 1;
            }
        }
        seq
    } else {
        record.seq().to_vec()
    };
    let l_seq = seq.len() as u64;
    let seq =
        String::from_utf8(seq).unwrap_or_else(|err| panic!("invalid UTF-8 FASTQ sequence: {err}"));
    let qual = if with_qual != 0 && !record.qual().is_empty() {
        Some(mb_opt_str_t::from_bytes(record.qual()))
    } else {
        None
    };
    mb_bseq1_t {
        l_seq,
        id: 0,
        name: name.into_boxed_str(),
        seq: seq.into_boxed_str(),
        qual,
        comment,
    }
}

#[cfg(feature = "plain-fastq-batch-io")]
fn read_plain_fastq_bseq(
    fp: &mut mb_bseq_file_t,
    with_qual: i32,
    with_comment: i32,
) -> Option<mb_bseq1_t> {
    let state = fp.plain_fastq_batch_reader.as_mut()?;
    match state.next_bseq(with_qual, with_comment) {
        Ok(Some(rec)) => {
            fp.pos += 1;
            fp.last_record_name = Some(rec.name.to_string());
            Some(rec)
        }
        Ok(None) => {
            fp.eof = true;
            None
        }
        Err(_) => {
            fp.parse_error = true;
            None
        }
    }
}

#[cfg(not(feature = "plain-fastq-batch-io"))]
fn read_plain_fastq_bseq(
    _fp: &mut mb_bseq_file_t,
    _with_qual: i32,
    _with_comment: i32,
) -> Option<mb_bseq1_t> {
    None
}

#[cfg(feature = "plain-fastq-batch-io")]
fn plain_fastq_chunk_config_for_bseq(
    chunk_size: i64,
    min_cnt: i32,
    max_chunk_size: i64,
) -> Option<PlainFastqChunkConfig> {
    if chunk_size <= 0 || max_chunk_size <= 0 {
        return None;
    }
    Some(
        PlainFastqChunkConfig::new(chunk_size as u64)
            .min_records(min_cnt.max(1) as usize)
            .max_bases(max_chunk_size as u64),
    )
}

#[cfg(feature = "plain-fastq-batch-io")]
fn read_plain_fastq_bseq_direct(
    fp: &mut mb_bseq_file_t,
    chunk_size: i64,
    with_qual: i32,
    with_comment: i32,
    min_cnt: i32,
    max_chunk_size: i64,
    n_: &mut i32,
) -> Option<Vec<mb_bseq1_t>> {
    let state = fp.plain_fastq_batch_reader.as_mut()?;
    let config = plain_fastq_chunk_config_for_bseq(chunk_size, min_cnt, max_chunk_size)?;
    let mut records = Vec::with_capacity(config.min_records);
    let mut sink = PlainFastqBseqSink {
        records: &mut records,
        with_qual,
        with_comment,
    };
    match state.reader.next_chunk_with_sink(config, &mut sink) {
        Ok(Some(stats)) => {
            fp.pos += stats.records() as usize;
            fp.last_record_name = records.last().map(|rec| rec.name.to_string());
            *n_ = records.len() as i32;
            Some(records)
        }
        Ok(None) => {
            fp.eof = true;
            *n_ = 0;
            Some(records)
        }
        Err(_) => {
            fp.pos += records.len();
            fp.parse_error = true;
            fp.last_record_name = records.last().map(|rec| rec.name.to_string());
            *n_ = records.len() as i32;
            Some(records)
        }
    }
}

#[cfg(not(feature = "plain-fastq-batch-io"))]
fn read_plain_fastq_bseq_direct(
    _fp: &mut mb_bseq_file_t,
    _chunk_size: i64,
    _with_qual: i32,
    _with_comment: i32,
    _min_cnt: i32,
    _max_chunk_size: i64,
    _n_: &mut i32,
) -> Option<Vec<mb_bseq1_t>> {
    None
}

fn report_bseq_parse_error(fp: &mut mb_bseq_file_t, records: &[mb_bseq1_t]) {
    if fp.parse_error && !fp.parse_error_reported && !fp.suppress_parse_warnings {
        if let Some(last) = records.last() {
            eprintln!(
                    "[WARNING]\u{1b}[1;31m failed to parse the FASTA/FASTQ record next to '{}'. Continue anyway.\u{1b}[0m",
                    last.name
                );
        } else {
            eprintln!(
                    "[WARNING]\u{1b}[1;31m failed to parse the first FASTA/FASTQ record. Continue anyway.\u{1b}[0m"
                );
        }
        fp.parse_error_reported = true;
    }
}

fn strip_kseq_line_ending(bytes: &mut Vec<u8>) {
    if bytes.last() == Some(&b'\n') {
        bytes.pop();
    }
    if bytes.len() > 1 && bytes.last() == Some(&b'\r') {
        bytes.pop();
    }
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
                let mut line = Vec::new();
                loop {
                    line.clear();
                    let n = reader.read_until(b'\n', &mut line).ok()?;
                    if n == 0 {
                        fp.eof = true;
                        return None;
                    }
                    if let Some(pos) = line.iter().position(|&c| c == b'>' || c == b'@') {
                        let start_ch = line[pos];
                        let mut header = line[pos + 1..].to_vec();
                        strip_kseq_line_ending(&mut header);
                        break (start_ch, header);
                    }
                }
            };
            let split = header
                .iter()
                .position(|&c| c.is_ascii_whitespace())
                .unwrap_or(header.len());
            let name = String::from_utf8_lossy(&header[..split]).into_owned();
            let comment = if split < header.len() {
                header[split + 1..].to_vec()
            } else {
                Vec::new()
            };
            let mut seq = Vec::new();
            let mut saw_plus = false;
            let mut line = Vec::new();
            loop {
                line.clear();
                let n = fp.reader.as_mut()?.read_until(b'\n', &mut line).ok()?;
                if n == 0 {
                    fp.eof = true;
                    break;
                }
                if line.first() == Some(&b'\n') {
                    continue;
                }
                if line[0] == b'>' || line[0] == b'@' {
                    let mut header = line[1..].to_vec();
                    strip_kseq_line_ending(&mut header);
                    fp.pending_header = Some((line[0], header));
                    break;
                }
                if line[0] == b'+' {
                    saw_plus = true;
                    break;
                }
                strip_kseq_line_ending(&mut line);
                seq.extend_from_slice(&line);
            }
            let mut qual = Vec::new();
            if saw_plus {
                while qual.len() < seq.len() {
                    line.clear();
                    let n = fp.reader.as_mut()?.read_until(b'\n', &mut line).ok()?;
                    if n == 0 {
                        fp.eof = true;
                        break;
                    }
                    strip_kseq_line_ending(&mut line);
                    qual.extend_from_slice(&line);
                    if qual.len() >= seq.len() {
                        break;
                    }
                }
                if qual.len() != seq.len() {
                    fp.parse_error = true;
                    return None;
                }
            }
            fp.pos += 1;
            let rec = kseq_t {
                name,
                comment,
                seq: String::from_utf8(seq).unwrap(),
                qual,
            };
            fp.last_record_name = Some(rec.name.clone());
            Some(rec)
        })()
    }};
}

fn read_next_bseq(
    fp: &mut mb_bseq_file_t,
    with_qual: i32,
    with_comment: i32,
) -> Option<mb_bseq1_t> {
    #[cfg(feature = "plain-fastq-batch-io")]
    if fp.plain_fastq_batch_reader.is_some() {
        return read_plain_fastq_bseq(fp, with_qual, with_comment);
    }

    let ks = kseq_read!(fp)?;
    let mut s = mb_bseq1_t::default();
    kseq2bseq(ks, &mut s, with_qual, with_comment);
    Some(s)
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
    if max_chunk_size < chunk_size {
        max_chunk_size = chunk_size;
    }
    if frag_mode == 0 && fp.s.is_none() {
        if let Some(records) = read_plain_fastq_bseq_direct(
            fp,
            chunk_size,
            with_qual,
            with_comment,
            min_cnt,
            max_chunk_size,
            n_,
        ) {
            report_bseq_parse_error(fp, &records);
            return records;
        }
    }
    if let Some(s) = fp.s.take() {
        size = s.l_seq as i64;
        a.push(s);
    }
    while let Some(s) = read_next_bseq(fp, with_qual, with_comment) {
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
                while let Some(s) = read_next_bseq(fp, with_qual, with_comment) {
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
    report_bseq_parse_error(fp, &a);
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
            let rec = read_next_bseq(f, with_qual, with_comment);
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
        for s in read.into_iter().flatten() {
            size += s.l_seq as i64;
            a.push(s);
        }
        if size >= chunk_size {
            break;
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
    fn qname_helpers_stop_at_nul_like_strlen() {
        assert_eq!(mb_qname_len("read\0hidden/1"), 4);
        assert_eq!(mb_qname_same("read\0left/1", "read\0right/2"), 1);
        assert_eq!(mb_qname_same("read1\0x", "read2\0x"), 0);
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

    #[test]
    fn read_sequence_strips_only_one_cr_before_lf_like_kseq() {
        let path = std::env::temp_dir().join(format!(
            "minibwa_rs_bseq_extra_cr_{}.fa",
            std::process::id()
        ));
        std::fs::write(&path, b">r0\nAC\r\r\n").unwrap();
        let mut fp = mb_bseq_open(Some(&path.to_string_lossy())).expect("open CR fasta");
        let mut n = 0;
        let reads = mb_bseq_read(&mut fp, 1_000_000, 0, 0, 0, 1, 1_000_000, &mut n);
        assert_eq!(n, 1);
        assert_eq!(&*reads[0].seq, "AC\r");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn read_fastq_preserves_raw_quality_and_comment_bytes() {
        let path = std::env::temp_dir().join(format!(
            "minibwa_rs_bseq_raw_qual_comment_{}.fq",
            std::process::id()
        ));
        std::fs::write(&path, b"@r0 comment\xff\nACGT\n+\n!\xff#$\n").unwrap();
        let mut fp = mb_bseq_open(Some(&path.to_string_lossy())).expect("open raw fastq");
        let mut n = 0;
        let reads = mb_bseq_read(&mut fp, 1_000_000, 1, 1, 0, 1, 1_000_000, &mut n);
        assert_eq!(n, 1);
        assert_eq!(&*reads[0].name, "r0");
        assert_eq!(
            reads[0].comment.as_ref().unwrap().as_bytes(),
            b"comment\xff"
        );
        assert_eq!(reads[0].qual.as_ref().unwrap().as_bytes(), b"!\xff#$");
        let _ = std::fs::remove_file(path);
    }

    #[cfg(feature = "plain-fastq-batch-io")]
    #[test]
    fn plain_fastq_extension_uses_plain_fastq_batch_reader() {
        let path = std::env::temp_dir().join(format!(
            "minibwa_rs_bseq_plain_fastq_batch_active_{}.fq",
            std::process::id()
        ));
        std::fs::write(&path, b"@r0 comment\nACGT\n+\n!!!!\n").unwrap();

        let fp =
            mb_bseq_open(Some(&path.to_string_lossy())).expect("open plain FASTQ batch reader");
        assert!(fp.plain_fastq_batch_reader.is_some());
        assert!(fp.reader.is_none());

        let _ = std::fs::remove_file(path);
    }

    #[cfg(feature = "plain-fastq-batch-io")]
    #[test]
    fn plain_fastq_batch_reader_skips_gzip_extension() {
        let path = std::env::temp_dir().join(format!(
            "minibwa_rs_bseq_plain_fastq_batch_skip_gzip_{}.fq.gz",
            std::process::id()
        ));
        std::fs::write(&path, b"not really gzip").unwrap();

        let fp = mb_bseq_open(Some(&path.to_string_lossy())).expect("open gzip-looking path");
        assert!(fp.plain_fastq_batch_reader.is_none());
        assert!(fp.reader.is_some());

        let _ = std::fs::remove_file(path);
    }

    #[cfg(feature = "plain-fastq-batch-io")]
    #[test]
    fn plain_fastq_batch_reader_can_be_disabled_by_caller() {
        let path = std::env::temp_dir().join(format!(
            "minibwa_rs_bseq_plain_fastq_batch_disabled_{}.fq",
            std::process::id()
        ));
        std::fs::write(&path, b"@r0\nACGT\n+\n!!!!\n").unwrap();

        let fp = mb_bseq_open_with_plain_fastq_batch(Some(&path.to_string_lossy()), false)
            .expect("open fastq");
        assert!(fp.plain_fastq_batch_reader.is_none());
        assert!(fp.reader.is_some());

        let _ = std::fs::remove_file(path);
    }

    #[cfg(feature = "plain-fastq-batch-io")]
    #[test]
    fn plain_fastq_batch_reader_records_match_legacy_reader() {
        let path = std::env::temp_dir().join(format!(
            "minibwa_rs_bseq_plain_fastq_batch_match_{}.fq",
            std::process::id()
        ));
        std::fs::write(
            &path,
            b"@r0 comment\nACGT\n+\n!!!!\n@r1/1 other\nUUAA\n+\n####\n",
        )
        .unwrap();

        let mut plain_fastq_fp =
            mb_bseq_open(Some(&path.to_string_lossy())).expect("open plain FASTQ batch reader");
        let mut legacy_fp =
            mb_bseq_open_legacy_for_test(Some(&path.to_string_lossy())).expect("open legacy fastq");
        let mut plain_fastq_n = 0;
        let mut legacy_n = 0;
        let plain_fastq_reads = mb_bseq_read(
            &mut plain_fastq_fp,
            1_000_000,
            1,
            1,
            0,
            1,
            1_000_000,
            &mut plain_fastq_n,
        );
        let legacy_reads = mb_bseq_read(
            &mut legacy_fp,
            1_000_000,
            1,
            1,
            0,
            1,
            1_000_000,
            &mut legacy_n,
        );

        assert_eq!(plain_fastq_n, legacy_n);
        assert_eq!(plain_fastq_reads, legacy_reads);

        let _ = std::fs::remove_file(path);
    }

    #[cfg(feature = "plain-fastq-batch-io")]
    #[test]
    fn plain_fastq_batch_reader_small_chunks_match_legacy_reader() {
        let path = std::env::temp_dir().join(format!(
            "minibwa_rs_bseq_plain_fastq_batch_small_chunks_{}.fq",
            std::process::id()
        ));
        std::fs::write(
            &path,
            b"@r0 comment\nACGT\n+\n!!!!\n@r1\nTTTT\n+\n####\n@r2\nGGGG\n+\nIIII\n",
        )
        .unwrap();

        let mut plain_fastq_fp =
            mb_bseq_open(Some(&path.to_string_lossy())).expect("open plain FASTQ batch reader");
        let mut legacy_fp =
            mb_bseq_open_legacy_for_test(Some(&path.to_string_lossy())).expect("open legacy fastq");
        let mut plain_fastq_reads = Vec::new();
        let mut legacy_reads = Vec::new();
        loop {
            let mut n = 0;
            let reads = mb_bseq_read(&mut plain_fastq_fp, 4, 1, 1, 0, 1, 4, &mut n);
            if reads.is_empty() {
                break;
            }
            plain_fastq_reads.extend(reads);
        }
        loop {
            let mut n = 0;
            let reads = mb_bseq_read(&mut legacy_fp, 4, 1, 1, 0, 1, 4, &mut n);
            if reads.is_empty() {
                break;
            }
            legacy_reads.extend(reads);
        }

        assert_eq!(plain_fastq_reads, legacy_reads);

        let _ = std::fs::remove_file(path);
    }

    #[cfg(feature = "plain-fastq-batch-io")]
    #[test]
    fn plain_fastq_batch_reader_parse_error_keeps_records_before_error() {
        let path = std::env::temp_dir().join(format!(
            "minibwa_rs_bseq_plain_fastq_batch_error_after_record_{}.fq",
            std::process::id()
        ));
        std::fs::write(&path, b"@good\nACGT\n+\nFFFF\n@bad\nACGTACGT\n+\nFFFF\n").unwrap();
        let mut fp =
            mb_bseq_open(Some(&path.to_string_lossy())).expect("open plain FASTQ batch reader");
        let mut n = 0;

        let reads = mb_bseq_read(&mut fp, 10_000_000, 1, 0, 0, 1, 10_000_000, &mut n);

        assert_eq!(n, 1);
        assert_eq!(&*reads[0].name, "good");
        assert_eq!(fp.pos, 1);
        assert!(fp.parse_error);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn bseq_open_path_stops_at_nul_like_gzopen() {
        let path = std::env::temp_dir().join(format!(
            "minibwa_rs_bseq_nul_path_{}.fa",
            std::process::id()
        ));
        std::fs::write(&path, b">r0\nACGT\n").unwrap();
        let nul_path = format!("{}\0hidden", path.to_string_lossy());

        let mut fp = mb_bseq_open(Some(&nul_path)).expect("open C-truncated path");
        let mut n = 0;
        let reads = mb_bseq_read(&mut fp, 1_000_000, 0, 0, 0, 1, 1_000_000, &mut n);
        assert_eq!(n, 1);
        assert_eq!(&*reads[0].name, "r0");
        assert_eq!(&*reads[0].seq, "ACGT");

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn fasta_header_plus_line_reads_quality_like_kseq() {
        let path = std::env::temp_dir().join(format!(
            "minibwa_rs_bseq_fasta_plus_quality_{}.fa",
            std::process::id()
        ));
        std::fs::write(&path, b">r0\nACGT\n+\n!!!!\n").unwrap();
        let mut fp = mb_bseq_open(Some(&path.to_string_lossy())).expect("open plus fasta");
        let mut n = 0;
        let reads = mb_bseq_read(&mut fp, 1_000_000, 1, 0, 0, 1, 1_000_000, &mut n);
        assert_eq!(n, 1);
        assert_eq!(&*reads[0].seq, "ACGT");
        assert_eq!(reads[0].qual.as_ref().unwrap().as_bytes(), b"!!!!");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn read_frag_does_not_report_truncated_quality_warning_state() {
        let good = std::env::temp_dir().join(format!(
            "minibwa_rs_bseq_frag_good_{}.fq",
            std::process::id()
        ));
        let bad = std::env::temp_dir().join(format!(
            "minibwa_rs_bseq_frag_bad_{}.fq",
            std::process::id()
        ));
        std::fs::write(&good, b"@r0\nACGT\n+\n!!!!\n").unwrap();
        std::fs::write(&bad, b"@r0\nACGT\n+\n!!\n").unwrap();
        let fp1 = mb_bseq_open(Some(&good.to_string_lossy())).expect("open good fastq");
        let fp2 = mb_bseq_open(Some(&bad.to_string_lossy())).expect("open bad fastq");
        let mut fps = vec![fp1, fp2];
        let mut n = 0;
        let reads = mb_bseq_read_frag(2, &mut fps, 1_000_000, 1, 0, &mut n);
        assert_eq!(n, 0);
        assert!(reads.is_empty());
        assert!(fps[1].parse_error);
        assert!(!fps[1].parse_error_reported);
        let _ = std::fs::remove_file(good);
        let _ = std::fs::remove_file(bad);
    }

    #[test]
    fn read_parse_error_after_chunk_boundary_is_first_record_state() {
        let path = std::env::temp_dir().join(format!(
            "minibwa_rs_bseq_parse_boundary_{}.fq",
            std::process::id()
        ));
        std::fs::write(&path, b"@good\nACGT\n+\nFFFF\n@bad\nACGTACGT\n+\nFFFF\n").unwrap();
        let mut fp = mb_bseq_open_with_plain_fastq_batch(Some(&path.to_string_lossy()), false)
            .expect("open fastq");
        let mut n = 0;

        let first = mb_bseq_read(&mut fp, 4, 1, 0, 0, 1, 4, &mut n);
        assert_eq!(n, 1);
        assert_eq!(&*first[0].name, "good");
        assert!(!fp.parse_error);

        let second = mb_bseq_read(&mut fp, 4, 1, 0, 0, 1, 4, &mut n);
        assert_eq!(n, 0);
        assert!(second.is_empty());
        assert!(fp.parse_error);
        assert!(fp.parse_error_after.is_none());

        let _ = std::fs::remove_file(path);
    }
}
