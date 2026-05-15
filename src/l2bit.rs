#![allow(unused_variables, dead_code, non_snake_case, non_camel_case_types)]

use flate2::read::MultiGzDecoder;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use std::path::Path;

const L2B_MAGIC: &[u8; 4] = b"L2B\x01";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum l2b_meth_t {
    #[default]
    L2B_METH_NONE = 0,
    L2B_METH_C2T = 1,
    L2B_METH_G2A = 2,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct l2b_ctg_t {
    pub name: String,
    pub comm: Option<String>,
    pub len: u64,
    pub off: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct l2b_intv_t {
    pub st: u64,
    pub en: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct l2b_t {
    pub tot_len: u64,
    pub n_ctg: u64,
    pub m_ctg: u64,
    pub ctg: Vec<l2b_ctg_t>,
    pub n_pac: u64,
    pub m_pac: u64,
    pub n_ambi: u64,
    pub m_ambi: u64,
    pub n_mask: u64,
    pub m_mask: u64,
    pub ambi: Vec<l2b_intv_t>,
    pub mask: Vec<l2b_intv_t>,
    pub pac: Vec<u64>,
    pub cat_name: Vec<u8>,
    pub cat_comm: Vec<u8>,
}

/// Original C static function `l2b_pos2cid` from `minibwa/l2bit.c:9`.
pub fn l2b_pos2cid(l2b: &l2b_t, s: i64, len: i64, cst: &mut i64) -> i64 {
    let mut lo = 0i64;
    let mut hi = l2b.n_ctg as i64;
    let mut mid = 0i64;
    while lo < hi {
        mid = (lo + hi) / 2;
        let ctg = &l2b.ctg[mid as usize];
        if ctg.off as i64 <= s && s < (ctg.off + ctg.len) as i64 {
            break;
        } else if s < ctg.off as i64 {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    let ctg = &l2b.ctg[mid as usize];
    *cst = s - ctg.off as i64;
    if s + len <= (ctg.off + ctg.len) as i64 {
        mid
    } else {
        -1
    }
}

/// Original C global function `l2b_intv2cid` from `minibwa/l2bit.c:9`.
pub fn l2b_intv2cid(l2b: &l2b_t, st: u64, en: u64, cst: &mut i64, rev: &mut i32) -> i64 {
    assert!(st < en);
    if en > l2b.tot_len * 2 {
        return -3;
    }
    if st < l2b.tot_len && l2b.tot_len < en {
        return -2;
    }
    *rev = (st >= l2b.tot_len) as i32;
    let s = if st < l2b.tot_len {
        st
    } else {
        l2b.tot_len * 2 - en
    } as i64;
    l2b_pos2cid(l2b, s, (en - st) as i64, cst)
}

/// Original C global function `l2b_intv2cid_meth` from `minibwa/l2bit.c:36`.
pub fn l2b_intv2cid_meth(
    l2b: &l2b_t,
    st: u64,
    en: u64,
    mt: &mut l2b_meth_t,
    cst: &mut i64,
    rev: &mut i32,
) -> i64 {
    assert!(st < en);
    let len = (en - st) as i64;
    let tot_len = l2b.tot_len;
    if en > tot_len * 4 {
        return -3;
    }
    let copy = (st / tot_len) as i32;
    *mt = if copy == 0 || copy == 3 {
        l2b_meth_t::L2B_METH_C2T
    } else {
        l2b_meth_t::L2B_METH_G2A
    };
    *rev = (copy >= 2) as i32;
    let mut s = (st - tot_len * copy as u64) as i64;
    if copy >= 2 {
        s = tot_len as i64 - len - s;
    }
    if s < 0 {
        -2
    } else {
        l2b_pos2cid(l2b, s, len, cst)
    }
}

/// Original C global function `l2b_getseq` from `minibwa/l2bit.c:29`.
pub fn l2b_getseq(l2b: &l2b_t, tid: i64, mut st: i64, mut en: i64, seq: &mut [u8]) -> i64 {
    if tid < 0 || tid >= l2b.n_ctg as i64 {
        return -1;
    }
    let ctg = &l2b.ctg[tid as usize];
    if st < 0 {
        st = 0;
    }
    if en > ctg.len as i64 {
        en = ctg.len as i64;
    }
    st += ctg.off as i64;
    en += ctg.off as i64;
    let len = (en - st).max(0) as usize;
    let pac = l2b.pac.as_ptr();
    let seq_ptr = seq.as_mut_ptr();
    let mut pos = st as u64;
    let mut out = 0usize;
    while out < len {
        let in_word = (pos & 31) as usize;
        let take = (32 - in_word).min(len - out);
        let mut word = unsafe { *pac.add((pos >> 5) as usize) >> (in_word * 2) };
        for j in 0..take {
            unsafe {
                *seq_ptr.add(out + j) = (word & 3) as u8;
            }
            word >>= 2;
        }
        out += take;
        pos += take as u64;
    }
    let mut n_ambi = 0i32;
    let aid = l2b_getambi(
        l2b,
        tid,
        st - ctg.off as i64,
        en - ctg.off as i64,
        &mut n_ambi,
    );
    if aid >= 0 {
        for i in 0..n_ambi as i64 {
            let iv = l2b.ambi[(aid + i) as usize];
            let mut s = iv.st as i64;
            let mut e = iv.en as i64;
            if s < st {
                s = st;
            }
            if e > en {
                e = en;
            }
            if s < e {
                let off = (s - st) as usize;
                let len = (e - s) as usize;
                unsafe {
                    std::ptr::write_bytes(seq_ptr.add(off), 4, len);
                }
            }
        }
    }
    en - st
}

/// Original C global function `l2b_getseq_meth` from `minibwa/l2bit.c:45`.
pub fn l2b_getseq_meth(
    l2b: &l2b_t,
    tid: i64,
    st: i64,
    en: i64,
    mt: l2b_meth_t,
    seq: &mut [u8],
) -> i64 {
    let len = l2b_getseq(l2b, tid, st, en, seq);
    if len <= 0 || mt == l2b_meth_t::L2B_METH_NONE {
        return len;
    }
    let seq_ptr = seq.as_mut_ptr();
    if mt == l2b_meth_t::L2B_METH_C2T {
        for i in 0..len as usize {
            unsafe {
                let p = seq_ptr.add(i);
                if *p == 1 {
                    *p = 3;
                }
            }
        }
    } else {
        for i in 0..len as usize {
            unsafe {
                let p = seq_ptr.add(i);
                if *p == 2 {
                    *p = 0;
                }
            }
        }
    }
    len
}

/// Original C global function `l2b_meth_convert` from `minibwa/l2bit.c:93`.
pub fn l2b_meth_convert(mt: l2b_meth_t, len: i64, seq: &mut [u8]) {
    if mt == l2b_meth_t::L2B_METH_C2T {
        for i in 0..len as usize {
            if seq[i] == 1 {
                seq[i] = 3;
            }
        }
    } else if mt == l2b_meth_t::L2B_METH_G2A {
        for i in 0..len as usize {
            if seq[i] == 2 {
                seq[i] = 0;
            }
        }
    }
}

/// Original C static function `l2b_get0` from `minibwa/l2bit.h:43`.
#[inline(always)]
pub fn l2b_get0(l2b: &l2b_t, i: u64) -> i32 {
    ((l2b.pac[(i >> 5) as usize] >> ((i & 31) << 1)) & 3) as i32
}

/// Original C static function `l2b_seq_prefetch` from `minibwa/l2bit.h:48`.
pub fn l2b_seq_prefetch(l2b: &l2b_t, tid: i64, st: i64) {
    if tid >= 0 && tid < l2b.n_ctg as i64 && st >= 0 && st < l2b.ctg[tid as usize].len as i64 {
        let idx = (st as u64 + l2b.ctg[tid as usize].off) >> 5;
        if let Some(cell) = l2b.pac.get(idx as usize) {
            crate::s2n_lite::_mm_prefetch(cell as *const u64 as *const u8, 3);
        }
    }
}

/// Original C static function `l2b_meth_rev` from `minibwa/l2bit.h:54`.
pub fn l2b_meth_rev(mt: l2b_meth_t) -> l2b_meth_t {
    if mt == l2b_meth_t::L2B_METH_NONE {
        l2b_meth_t::L2B_METH_NONE
    } else if mt == l2b_meth_t::L2B_METH_C2T {
        l2b_meth_t::L2B_METH_G2A
    } else {
        l2b_meth_t::L2B_METH_C2T
    }
}

/// Original C global function `l2b_getambi` from `minibwa/l2bit.c:56`.
pub fn l2b_getambi(l2b: &l2b_t, tid: i64, mut st: i64, mut en: i64, n_ambi: &mut i32) -> i64 {
    *n_ambi = 0;
    if tid < 0 || tid >= l2b.n_ctg as i64 {
        return -1;
    }
    if st < 0 {
        st = 0;
    }
    if en > l2b.ctg[tid as usize].len as i64 {
        en = l2b.ctg[tid as usize].len as i64;
    }
    if st >= en {
        return -1;
    }
    let g_beg = l2b.ctg[tid as usize].off as i64 + st;
    let g_end = l2b.ctg[tid as usize].off as i64 + en;

    let mut lo = 0i64;
    let mut hi = l2b.n_ambi as i64;
    while lo < hi {
        let mid = (lo + hi) / 2;
        if l2b.ambi[mid as usize].en as i64 > g_beg {
            hi = mid;
        } else {
            lo = mid + 1;
        }
    }
    let i_st = lo;
    lo = i_st;
    hi = l2b.n_ambi as i64;
    while lo < hi {
        let mid = (lo + hi) / 2;
        if l2b.ambi[mid as usize].st as i64 >= g_end {
            hi = mid;
        } else {
            lo = mid + 1;
        }
    }
    let i_en = lo;
    *n_ambi = (i_en - i_st) as i32;
    if *n_ambi == 0 {
        -1
    } else {
        i_st
    }
}

/// Original C static function `l2b_format_seq` from `minibwa/l2bit.c:88`.
pub fn l2b_format_seq(len: u64, seq: &mut [u8], rng: &mut u64) {
    for i in 0..len as usize {
        let b = seq[i];
        let mut c = match b {
            b'A' | b'a' => 0,
            b'C' | b'c' => 1,
            b'G' | b'g' => 2,
            b'T' | b't' | b'U' | b'u' => 3,
            _ => 4,
        };
        if c == 4 {
            *rng = rng.wrapping_add(0x9e3779b97f4a7c15);
            let mut z = *rng;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
            c |= ((z ^ (z >> 31)) & 3) as u8;
        }
        if b < b'A' || b > b'Z' {
            c |= 1 << 3;
        }
        seq[i] = c;
    }
}

/// Original C static function `l2b_add_seq` from `minibwa/l2bit.c:101`.
pub fn l2b_add_seq(
    l2b: &mut l2b_t,
    len: u64,
    seq: &[u8],
    name: &str,
    comm: Option<&str>,
    rng: &mut u64,
) {
    let off = l2b.tot_len;
    l2b.ctg.push(l2b_ctg_t {
        name: name.to_string(),
        comm: comm.map(str::to_string),
        len,
        off: l2b.tot_len,
    });
    l2b.n_ctg += 1;
    l2b.m_ctg = l2b.ctg.capacity() as u64;
    l2b.tot_len += len;

    l2b.n_pac = (l2b.tot_len + 31) / 32;
    if l2b.pac.len() < l2b.n_pac as usize {
        l2b.pac.resize(l2b.n_pac as usize, 0);
    }
    l2b.m_pac = l2b.pac.capacity() as u64;

    let mut ambi_len = 0u64;
    let mut mask_len = 0u64;
    for i in 0..len {
        let c = seq[i as usize] as u64;
        let x = off + i;
        if (c & (1 << 3)) != 0 {
            mask_len += 1;
        } else if mask_len > 0 {
            l2b.mask.push(l2b_intv_t {
                st: x - mask_len,
                en: x,
            });
            l2b.n_mask += 1;
            mask_len = 0;
        }
        if (c & (1 << 2)) != 0 {
            ambi_len += 1;
        } else if ambi_len > 0 {
            l2b.ambi.push(l2b_intv_t {
                st: x - ambi_len,
                en: x,
            });
            l2b.n_ambi += 1;
            ambi_len = 0;
        }
        l2b.pac[(x >> 5) as usize] |= (c & 3) << ((x & 0x1f) * 2);
    }
    l2b.m_mask = l2b.mask.capacity() as u64;
    l2b.m_ambi = l2b.ambi.capacity() as u64;
}

/// Original C static function `l2b_collate_str` from `minibwa/l2bit.c:145`.
pub fn l2b_collate_str(l2b: &mut l2b_t) {
    if !l2b.cat_name.is_empty() || !l2b.cat_comm.is_empty() {
        return;
    }
    for ctg in &l2b.ctg {
        l2b.cat_name.extend_from_slice(ctg.name.as_bytes());
        l2b.cat_name.push(0);
        if let Some(comm) = &ctg.comm {
            l2b.cat_comm.extend_from_slice(comm.as_bytes());
        }
        l2b.cat_comm.push(0);
    }
}

/// Original C global function `l2b_import` from `minibwa/l2bit.c:175`.
pub fn l2b_import<P: AsRef<Path>>(fn_: P, seed: u64) -> Option<l2b_t> {
    let path = fn_.as_ref().to_string_lossy();
    let raw: Box<dyn Read> = if path == "-" {
        Box::new(io::stdin())
    } else {
        Box::new(File::open(fn_.as_ref()).ok()?)
    };
    let mut buffered = BufReader::with_capacity(1 << 20, raw);
    let is_gzip = buffered.fill_buf().ok()?.starts_with(&[0x1f, 0x8b]);
    let mut reader: Box<dyn BufRead> = if is_gzip {
        Box::new(BufReader::with_capacity(
            1 << 20,
            MultiGzDecoder::new(buffered),
        ))
    } else {
        Box::new(buffered)
    };
    let mut l2b = l2b_t::default();
    let mut rng = seed;
    let mut name = String::new();
    let mut comm: Option<String> = None;
    let mut seq = Vec::new();
    let mut line = String::new();
    loop {
        line.clear();
        if reader.read_line(&mut line).ok()? == 0 {
            break;
        }
        let line = line.trim_end_matches(['\n', '\r']);
        if let Some(header) = line.strip_prefix('>') {
            if !name.is_empty() {
                l2b_format_seq(seq.len() as u64, &mut seq, &mut rng);
                l2b_add_seq(
                    &mut l2b,
                    seq.len() as u64,
                    &seq,
                    &name,
                    comm.as_deref(),
                    &mut rng,
                );
                seq.clear();
            }
            let mut parts = header.splitn(2, char::is_whitespace);
            name = parts.next().unwrap_or("").to_string();
            comm = parts.next().filter(|s| !s.is_empty()).map(str::to_string);
        } else {
            seq.extend_from_slice(line.trim().as_bytes());
        }
    }
    if !name.is_empty() {
        l2b_format_seq(seq.len() as u64, &mut seq, &mut rng);
        l2b_add_seq(
            &mut l2b,
            seq.len() as u64,
            &seq,
            &name,
            comm.as_deref(),
            &mut rng,
        );
    }
    l2b_collate_str(&mut l2b);
    Some(l2b)
}

/// Original C global function `l2b_destroy` from `minibwa/l2bit.c:196`.
pub fn l2b_destroy(l2b: Option<l2b_t>) {
    drop(l2b);
}

/// Original C global function `l2b_save` from `minibwa/l2bit.c:202`.
pub fn l2b_save<P: AsRef<Path>>(fn_: P, l2b: &l2b_t) -> i32 {
    let path = fn_.as_ref().to_string_lossy();
    let mut fp: Box<dyn Write> = if path == "-" {
        Box::new(BufWriter::with_capacity(1 << 20, io::stdout()))
    } else {
        match File::create(fn_.as_ref()) {
            Ok(fp) => Box::new(BufWriter::with_capacity(1 << 20, fp)),
            Err(_) => return -1,
        }
    };
    let len_name: u64 = l2b.ctg.iter().map(|c| c.name.len() as u64 + 1).sum();
    let len_comm: u64 = l2b
        .ctg
        .iter()
        .map(|c| c.comm.as_ref().map(|s| s.len() as u64 + 1).unwrap_or(1))
        .sum();
    if fp.write_all(L2B_MAGIC).is_err() || fp.write_all(&0u32.to_le_bytes()).is_err() {
        return -1;
    }
    for x in [
        l2b.n_ctg,
        l2b.tot_len,
        l2b.n_ambi,
        l2b.n_mask,
        len_name,
        len_comm,
        l2b.n_pac,
    ] {
        if fp.write_all(&x.to_le_bytes()).is_err() {
            return -1;
        }
    }
    for ctg in &l2b.ctg {
        if fp.write_all(&ctg.len.to_le_bytes()).is_err() {
            return -1;
        }
    }
    for iv in &l2b.ambi {
        if fp.write_all(&iv.st.to_le_bytes()).is_err()
            || fp.write_all(&iv.en.to_le_bytes()).is_err()
        {
            return -1;
        }
    }
    for iv in &l2b.mask {
        if fp.write_all(&iv.st.to_le_bytes()).is_err()
            || fp.write_all(&iv.en.to_le_bytes()).is_err()
        {
            return -1;
        }
    }
    if write_u64_slice_le(&mut fp, &l2b.pac).is_err() {
        return -1;
    }
    for ctg in &l2b.ctg {
        if fp.write_all(ctg.name.as_bytes()).is_err() || fp.write_all(&[0]).is_err() {
            return -1;
        }
    }
    for ctg in &l2b.ctg {
        if let Some(comm) = &ctg.comm {
            if fp.write_all(comm.as_bytes()).is_err() {
                return -1;
            }
        }
        if fp.write_all(&[0]).is_err() {
            return -1;
        }
    }
    if fp.flush().is_err() {
        -1
    } else {
        0
    }
}

fn write_u64_slice_le<W: Write + ?Sized>(writer: &mut W, words: &[u64]) -> std::io::Result<()> {
    #[cfg(target_endian = "little")]
    {
        let bytes = unsafe {
            std::slice::from_raw_parts(words.as_ptr().cast::<u8>(), std::mem::size_of_val(words))
        };
        writer.write_all(bytes)
    }
    #[cfg(not(target_endian = "little"))]
    {
        for &word in words {
            writer.write_all(&word.to_le_bytes())?;
        }
        Ok(())
    }
}

fn read_intv_vec_le<R: Read + ?Sized>(reader: &mut R, n: u64) -> Option<Vec<l2b_intv_t>> {
    let n = n as usize;
    let mut v = Vec::<l2b_intv_t>::with_capacity(n);
    let bytes = unsafe {
        std::slice::from_raw_parts_mut(
            v.as_mut_ptr() as *mut u8,
            n.checked_mul(std::mem::size_of::<l2b_intv_t>())?,
        )
    };
    reader.read_exact(bytes).ok()?;
    unsafe {
        v.set_len(n);
    }
    #[cfg(target_endian = "big")]
    {
        for x in &mut v {
            x.st = u64::from_le(x.st);
            x.en = u64::from_le(x.en);
        }
    }
    Some(v)
}

/// Original C global function `l2b_load` from `minibwa/l2bit.c:234`.
pub fn l2b_load<P: AsRef<Path>>(fn_: P) -> Option<l2b_t> {
    let path = fn_.as_ref().to_string_lossy();
    let mut fp: Box<dyn Read> = if path == "-" {
        Box::new(io::stdin())
    } else {
        Box::new(File::open(fn_.as_ref()).ok()?)
    };
    let mut magic = [0u8; 4];
    fp.read_exact(&mut magic).ok()?;
    if &magic != L2B_MAGIC {
        return None;
    }
    let mut dummy = [0u8; 4];
    fp.read_exact(&mut dummy).ok()?;
    let mut hdr = [0u8; 56];
    fp.read_exact(&mut hdr).ok()?;
    let mut fields = [0u64; 7];
    for i in 0..7usize {
        fields[i] = u64::from_le_bytes(hdr[i * 8..i * 8 + 8].try_into().unwrap());
    }
    let (n_ctg, tot_len, n_ambi, n_mask, len_name, len_comm, n_pac) = (
        fields[0], fields[1], fields[2], fields[3], fields[4], fields[5], fields[6],
    );
    let mut l2b = l2b_t {
        tot_len,
        n_ctg,
        m_ctg: n_ctg,
        n_pac,
        m_pac: n_pac,
        n_ambi,
        m_ambi: n_ambi,
        n_mask,
        m_mask: n_mask,
        ..Default::default()
    };
    let mut off = 0u64;
    for _ in 0..n_ctg {
        let mut len_buf = [0u8; 8];
        fp.read_exact(&mut len_buf).ok()?;
        let len = u64::from_le_bytes(len_buf);
        l2b.ctg.push(l2b_ctg_t {
            name: String::new(),
            comm: None,
            len,
            off,
        });
        off += len;
    }
    if off != tot_len {
        return None;
    }
    l2b.ambi = read_intv_vec_le(&mut *fp, n_ambi)?;
    l2b.mask = read_intv_vec_le(&mut *fp, n_mask)?;
    l2b.pac = Vec::with_capacity(n_pac as usize);
    let pac_bytes = unsafe {
        std::slice::from_raw_parts_mut(l2b.pac.as_mut_ptr() as *mut u8, n_pac as usize * 8)
    };
    fp.read_exact(pac_bytes).ok()?;
    unsafe {
        l2b.pac.set_len(n_pac as usize);
    }
    #[cfg(target_endian = "big")]
    {
        for x in &mut l2b.pac {
            *x = u64::from_le(*x);
        }
    }
    l2b.cat_name = vec![0; len_name as usize];
    l2b.cat_comm = vec![0; len_comm as usize];
    fp.read_exact(&mut l2b.cat_name).ok()?;
    fp.read_exact(&mut l2b.cat_comm).ok()?;
    let mut p_name = 0usize;
    let mut p_comm = 0usize;
    for i in 0..n_ctg as usize {
        let name_end = p_name + l2b.cat_name[p_name..].iter().position(|&b| b == 0)?;
        l2b.ctg[i].name = String::from_utf8_lossy(&l2b.cat_name[p_name..name_end]).into_owned();
        p_name = name_end + 1;
        if l2b.cat_comm[p_comm] != 0 {
            let comm_end = p_comm + l2b.cat_comm[p_comm..].iter().position(|&b| b == 0)?;
            l2b.ctg[i].comm =
                Some(String::from_utf8_lossy(&l2b.cat_comm[p_comm..comm_end]).into_owned());
            p_comm = comm_end + 1;
        } else {
            p_comm += 1;
        }
    }
    if p_name != len_name as usize || p_comm != len_comm as usize {
        return None;
    }
    Some(l2b)
}

/// Original C global function `l2b_save_pac` from `minibwa/l2bit.c:286`.
pub fn l2b_save_pac<P: AsRef<Path>>(fn_: P, l2b: &l2b_t, both_strand: i32) -> i32 {
    let path = fn_.as_ref().to_string_lossy();
    let mut fp: Box<dyn Write> = if path == "-" {
        Box::new(io::stdout())
    } else {
        match File::create(fn_.as_ref()) {
            Ok(fp) => Box::new(fp),
            Err(_) => return -1,
        }
    };
    let n_pac = ((if both_strand != 0 {
        l2b.tot_len * 2
    } else {
        l2b.tot_len
    }) + 3)
        / 4;
    let mut pac = vec![0u8; n_pac as usize];
    let mut x = 0u64;
    for i in 0..l2b.tot_len {
        pac[(x >> 2) as usize] |= (l2b_get0(l2b, i) as u8) << ((!x & 3) * 2);
        x += 1;
    }
    if both_strand != 0 {
        for i in (0..l2b.tot_len).rev() {
            pac[(x >> 2) as usize] |= (3 - l2b_get0(l2b, i) as u8) << ((!x & 3) * 2);
            x += 1;
        }
    }
    let n_write = (x >> 2) + if (x & 3) == 0 { 0 } else { 1 };
    if fp.write_all(&pac[..n_write as usize]).is_err() {
        return -1;
    }
    if x % 4 == 0 && fp.write_all(&[0]).is_err() {
        return -1;
    }
    if fp.write_all(&[(x % 4) as u8]).is_err() {
        return -1;
    }
    0
}

/// Original C static function `l2b_c2t` from `minibwa/l2bit.c:321`.
pub fn l2b_c2t(b: u8) -> u8 {
    if b == 1 {
        3
    } else {
        b
    }
}

/// Original C static function `l2b_g2a` from `minibwa/l2bit.c:322`.
pub fn l2b_g2a(b: u8) -> u8 {
    if b == 2 {
        0
    } else {
        b
    }
}

/// Original C global function `l2b_save_pac_meth` from `minibwa/l2bit.c:324`.
pub fn l2b_save_pac_meth<P: AsRef<Path>>(fn_: P, l2b: &l2b_t, both_strand: i32) -> i32 {
    let path = fn_.as_ref().to_string_lossy();
    let mut fp: Box<dyn Write> = if path == "-" {
        Box::new(io::stdout())
    } else {
        match File::create(fn_.as_ref()) {
            Ok(fp) => Box::new(fp),
            Err(_) => return -1,
        }
    };
    let mut len = l2b.tot_len * 2;
    if both_strand != 0 {
        len *= 2;
    }
    let n_pac = (len + 3) / 4;
    let mut pac = vec![0u8; n_pac as usize];
    let mut x = 0u64;
    for i in 0..l2b.tot_len {
        let b = l2b_c2t(l2b_get0(l2b, i) as u8);
        pac[(x >> 2) as usize] |= b << ((!x & 3) * 2);
        x += 1;
    }
    for i in 0..l2b.tot_len {
        let b = l2b_g2a(l2b_get0(l2b, i) as u8);
        pac[(x >> 2) as usize] |= b << ((!x & 3) * 2);
        x += 1;
    }
    if both_strand != 0 {
        for i in (0..l2b.tot_len).rev() {
            let b = 3 - l2b_g2a(l2b_get0(l2b, i) as u8);
            pac[(x >> 2) as usize] |= b << ((!x & 3) * 2);
            x += 1;
        }
        for i in (0..l2b.tot_len).rev() {
            let b = 3 - l2b_c2t(l2b_get0(l2b, i) as u8);
            pac[(x >> 2) as usize] |= b << ((!x & 3) * 2);
            x += 1;
        }
    }
    let n_write = (x >> 2) + if (x & 3) == 0 { 0 } else { 1 };
    if fp.write_all(&pac[..n_write as usize]).is_err() {
        return -1;
    }
    if x % 4 == 0 && fp.write_all(&[0]).is_err() {
        return -1;
    }
    if fp.write_all(&[(x % 4) as u8]).is_err() {
        return -1;
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_real_chrM_l2b_and_get_reference_prefix() {
        let l2b = l2b_load("minibwa/chrM-human.l2b").expect("load chrM-human.l2b");
        assert_eq!(l2b.n_ctg, 1);
        assert_eq!(l2b.ctg[0].name, "chrM");
        assert_eq!(l2b.tot_len, l2b.ctg[0].len);

        let mut seq = vec![0u8; 32];
        assert_eq!(l2b_getseq(&l2b, 0, 0, 32, &mut seq), 32);
        let expected = b"GATCACAGGTCTATCACCCTATTAACCACTCA"
            .iter()
            .map(|&b| match b {
                b'A' => 0,
                b'C' => 1,
                b'G' => 2,
                b'T' => 3,
                _ => 4,
            })
            .collect::<Vec<_>>();
        assert_eq!(seq, expected);
    }

    #[test]
    fn real_chrM_l2b_roundtrips_through_save_load() {
        let l2b = l2b_load("minibwa/chrM-human.l2b").expect("load chrM-human.l2b");
        let path = std::env::temp_dir().join("minibwa-rs-roundtrip.l2b");
        assert_eq!(l2b_save(&path, &l2b), 0);
        let loaded = l2b_load(&path).expect("reload l2b");
        let _ = std::fs::remove_file(&path);
        assert_eq!(loaded, l2b);
    }

    #[test]
    fn intv2cid_handles_forward_reverse_and_invalid_real_chrM_ranges() {
        let l2b = l2b_load("minibwa/chrM-human.l2b").expect("load chrM-human.l2b");
        let mut cst = -1;
        let mut rev = -1;
        assert_eq!(l2b_intv2cid(&l2b, 10, 20, &mut cst, &mut rev), 0);
        assert_eq!(cst, 10);
        assert_eq!(rev, 0);

        let st = l2b.tot_len * 2 - 20;
        let en = l2b.tot_len * 2 - 10;
        assert_eq!(l2b_intv2cid(&l2b, st, en, &mut cst, &mut rev), 0);
        assert_eq!(cst, 10);
        assert_eq!(rev, 1);

        assert_eq!(
            l2b_intv2cid(&l2b, l2b.tot_len - 1, l2b.tot_len + 1, &mut cst, &mut rev),
            -2
        );
        assert_eq!(
            l2b_intv2cid(&l2b, 0, l2b.tot_len * 2 + 1, &mut cst, &mut rev),
            -3
        );
    }

    #[test]
    fn save_pac_emits_expected_small_packed_sequence() {
        let mut l2b = l2b_t::default();
        let mut seq = b"ACGTAC".to_vec();
        let mut rng = 11;
        l2b_format_seq(seq.len() as u64, &mut seq, &mut rng);
        l2b_add_seq(&mut l2b, seq.len() as u64, &seq, "s", None, &mut rng);
        l2b_collate_str(&mut l2b);
        let path = std::env::temp_dir().join("minibwa-rs-small.pac");
        assert_eq!(l2b_save_pac(&path, &l2b, 0), 0);
        let bytes = std::fs::read(&path).expect("read pac");
        let _ = std::fs::remove_file(&path);
        assert_eq!(bytes, vec![0x1b, 0x10, 2]);
    }

    #[test]
    fn import_reads_real_gzipped_chrM_fasta() {
        let imported =
            l2b_import("minibwa/test/chrM-human.fa.gz", 11).expect("import gzipped FASTA");
        let loaded = l2b_load("minibwa/chrM-human.l2b").expect("load chrM-human.l2b");
        assert_eq!(imported.n_ctg, loaded.n_ctg);
        assert_eq!(imported.ctg[0].name, loaded.ctg[0].name);
        assert_eq!(imported.ctg[0].len, loaded.ctg[0].len);
        assert_eq!(imported.tot_len, loaded.tot_len);
        let mut imported_seq = vec![0; 64];
        let mut loaded_seq = vec![0; 64];
        l2b_getseq(&imported, 0, 0, 64, &mut imported_seq);
        l2b_getseq(&loaded, 0, 0, 64, &mut loaded_seq);
        assert_eq!(imported_seq, loaded_seq);
    }

    #[test]
    fn methylation_interval_and_sequence_conversion_match_rules() {
        let l2b = l2b_load("minibwa/chrM-human.l2b").expect("load chrM-human.l2b");
        let mut cst = -1;
        let mut rev = -1;
        let mut mt = l2b_meth_t::L2B_METH_NONE;
        assert_eq!(
            l2b_intv2cid_meth(&l2b, 0, 10, &mut mt, &mut cst, &mut rev),
            0
        );
        assert_eq!(mt, l2b_meth_t::L2B_METH_C2T);
        assert_eq!(cst, 0);
        assert_eq!(rev, 0);
        assert_eq!(
            l2b_intv2cid_meth(
                &l2b,
                l2b.tot_len,
                l2b.tot_len + 10,
                &mut mt,
                &mut cst,
                &mut rev
            ),
            0
        );
        assert_eq!(mt, l2b_meth_t::L2B_METH_G2A);
        assert_eq!(rev, 0);
        assert_eq!(
            l2b_intv2cid_meth(
                &l2b,
                l2b.tot_len * 3,
                l2b.tot_len * 3 + 10,
                &mut mt,
                &mut cst,
                &mut rev
            ),
            0
        );
        assert_eq!(mt, l2b_meth_t::L2B_METH_C2T);
        assert_eq!(rev, 1);
        assert_eq!(
            l2b_meth_rev(l2b_meth_t::L2B_METH_C2T),
            l2b_meth_t::L2B_METH_G2A
        );

        let mut seq = vec![0; 16];
        l2b_getseq_meth(&l2b, 0, 0, 16, l2b_meth_t::L2B_METH_C2T, &mut seq);
        assert!(!seq.contains(&1));
        l2b_getseq_meth(&l2b, 0, 0, 16, l2b_meth_t::L2B_METH_G2A, &mut seq);
        assert!(!seq.contains(&2));
    }

    #[test]
    fn save_pac_meth_emits_expected_small_packed_sequence() {
        let mut l2b = l2b_t::default();
        let mut seq = b"ACGT".to_vec();
        let mut rng = 3;
        l2b_format_seq(seq.len() as u64, &mut seq, &mut rng);
        l2b_add_seq(&mut l2b, seq.len() as u64, &seq, "s", None, &mut rng);
        let path = std::env::temp_dir().join("minibwa-rs-small-meth.pac");
        assert_eq!(l2b_save_pac_meth(&path, &l2b, 1), 0);
        let bytes = std::fs::read(&path).expect("read meth pac");
        let _ = std::fs::remove_file(&path);
        assert_eq!(bytes, vec![0x3b, 0x13, 0x3b, 0x13, 0, 0]);
    }
}
