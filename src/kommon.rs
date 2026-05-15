#![allow(unused_variables, dead_code, non_snake_case, non_camel_case_types)]

use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

pub type c_long = libc::c_long;
pub type rusage = libc::rusage;
pub type timeval = libc::timeval;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct timezone {
    pub tz_minuteswest: i32,
    pub tz_dsttime: i32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct kstring_t {
    pub l: usize,
    pub m: usize,
    pub s: Vec<u8>,
}

const fn make_kom_nt4_table() -> [u8; 256] {
    let mut table = [4u8; 256];
    table[0] = 0;
    table[1] = 1;
    table[2] = 2;
    table[3] = 3;
    table[b'A' as usize] = 0;
    table[b'C' as usize] = 1;
    table[b'G' as usize] = 2;
    table[b'T' as usize] = 3;
    table[b'a' as usize] = 0;
    table[b'c' as usize] = 1;
    table[b'g' as usize] = 2;
    table[b't' as usize] = 3;
    table
}

pub const KOM_NT4_TABLE: [u8; 256] = make_kom_nt4_table();

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum kom_sprintf_arg<'a> {
    d(i32),
    ld(i64),
    u(u32),
    s(&'a str),
    c(i32),
}

/// Original C global function `kom_strdup` from `minibwa/kommon.c:9`.
pub fn kom_strdup(src: &str) -> String {
    src.to_string()
}

/// Original C global function `kom_parse_num` from `minibwa/kommon.c:19`.
pub fn kom_parse_num(str_: &str) -> (i64, usize) {
    let bytes = str_.as_bytes();
    let nul = bytes.iter().position(|&c| c == 0).unwrap_or(bytes.len());
    let mut cstr = Vec::with_capacity(nul + 1);
    cstr.extend_from_slice(&bytes[..nul]);
    cstr.push(0);
    let mut end: *mut libc::c_char = std::ptr::null_mut();
    let mut x = unsafe { libc::strtod(cstr.as_ptr() as *const libc::c_char, &mut end) };
    let mut p = if end.is_null() {
        0
    } else {
        unsafe { end.offset_from(cstr.as_ptr() as *const libc::c_char) as usize }
    };
    if p < bytes.len() {
        if bytes[p] == b'G' || bytes[p] == b'g' {
            x *= 1e9;
            p += 1;
        } else if bytes[p] == b'M' || bytes[p] == b'm' {
            x *= 1e6;
            p += 1;
        } else if bytes[p] == b'K' || bytes[p] == b'k' {
            x *= 1e3;
            p += 1;
        }
    }
    ((x + 0.499) as i64, p)
}

/// Original C global function `kom_panic` from `minibwa/kommon.c:31`.
pub fn kom_panic(func: &str, msg: &str) -> ! {
    eprintln!("[E::{func}] {msg} ABORT!");
    unsafe { libc::abort() }
}

/// Original C static function `str_enlarge` from `minibwa/kommon.c:45`.
pub fn str_enlarge(km: (), s: &mut kstring_t, l: i32) {
    let need = s.l + l as usize + 1;
    if need > s.m {
        s.m = need;
        s.m = s.m.wrapping_sub(1);
        s.m |= s.m >> 1;
        s.m |= s.m >> 2;
        s.m |= s.m >> 4;
        s.m |= s.m >> 8;
        s.m |= s.m >> 16;
        s.m |= s.m >> 32;
        s.m = s.m.wrapping_add(1);
        s.s.resize(s.m, 0);
    }
}

/// Original C static function `str_copy` from `minibwa/kommon.c:58`.
pub fn str_copy(km: (), s: &mut kstring_t, st: &[u8], en: usize) {
    str_enlarge(km, s, en as i32);
    let end = s.l + en;
    s.s[s.l..end].copy_from_slice(&st[..en]);
    s.l = end;
}

/// Original C static function `kom_splitmix64` from `minibwa/kommon.h:63`.
pub fn kom_splitmix64(x: &mut u64) -> u64 {
    *x = x.wrapping_add(0x9e3779b97f4a7c15);
    let mut z = *x;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}

/// Original C global function `kom_sprintf_lite_core` from `minibwa/kommon.c:65`.
pub fn kom_sprintf_lite_core(
    km: (),
    mut s: Option<&mut kstring_t>,
    fmt: &str,
    ap: &[kom_sprintf_arg<'_>],
) -> i64 {
    let mut len = 0i64;
    let bytes = fmt.as_bytes();
    let mut p = 0usize;
    let mut q = 0usize;
    let mut ai = 0usize;
    while p < bytes.len() {
        if bytes[p] == b'%' {
            if p > q {
                len += (p - q) as i64;
                if let Some(ref mut out) = s {
                    str_copy(km, out, &bytes[q..p], p - q);
                }
            }
            p += 1;
            let text = if p < bytes.len() && bytes[p] == b'd' {
                let v = match ap[ai] {
                    kom_sprintf_arg::d(v) => v,
                    _ => panic!("kom_sprintf_lite_core: expected %d argument"),
                };
                ai += 1;
                v.to_string()
            } else if p + 1 < bytes.len() && bytes[p] == b'l' && bytes[p + 1] == b'd' {
                let v = match ap[ai] {
                    kom_sprintf_arg::ld(v) => v,
                    _ => panic!("kom_sprintf_lite_core: expected %ld argument"),
                };
                ai += 1;
                p += 1;
                v.to_string()
            } else if p < bytes.len() && bytes[p] == b'u' {
                let v = match ap[ai] {
                    kom_sprintf_arg::u(v) => v,
                    _ => panic!("kom_sprintf_lite_core: expected %u argument"),
                };
                ai += 1;
                v.to_string()
            } else if p < bytes.len() && bytes[p] == b's' {
                let v = match ap[ai] {
                    kom_sprintf_arg::s(v) => v,
                    _ => panic!("kom_sprintf_lite_core: expected %s argument"),
                };
                ai += 1;
                v.to_string()
            } else if p < bytes.len() && bytes[p] == b'c' {
                let v = match ap[ai] {
                    kom_sprintf_arg::c(v) => v as u8,
                    _ => panic!("kom_sprintf_lite_core: expected %c argument"),
                };
                ai += 1;
                String::from_utf8(vec![v]).unwrap()
            } else {
                let ch = if p < bytes.len() {
                    bytes[p] as char
                } else {
                    '\0'
                };
                panic!("ERROR: unrecognized type '%{ch}'");
            };
            len += text.len() as i64;
            if let Some(ref mut out) = s {
                str_copy(km, out, text.as_bytes(), text.len());
            }
            q = p + 1;
        }
        p += 1;
    }
    if p > q {
        len += (p - q) as i64;
        if let Some(ref mut out) = s {
            str_copy(km, out, &bytes[q..p], p - q);
        }
    }
    if let Some(ref mut out) = s {
        if out.s.len() <= out.l {
            out.s.resize(out.l + 1, 0);
            out.m = out.s.len();
        }
        out.s[out.l] = 0;
    }
    len
}

/// Original C static function `kom_u64todbl` from `minibwa/kommon.h:71`.
pub fn kom_u64todbl(x: u64) -> f64 {
    f64::from_bits(0x3ffu64 << 52 | x >> 12) - 1.0
}

/// Original C global function `kom_sprintf_lite` from `minibwa/kommon.c:140`.
pub fn kom_sprintf_lite(s: &mut kstring_t, fmt: &str, ap: &[kom_sprintf_arg<'_>]) -> i64 {
    kom_sprintf_lite_core((), Some(s), fmt, ap)
}

/// Original C global function `km_sprintf_lite` from `minibwa/kommon.c:148`.
pub fn km_sprintf_lite(km: (), s: &mut kstring_t, fmt: &str, ap: &[kom_sprintf_arg<'_>]) -> i64 {
    kom_sprintf_lite_core(km, Some(s), fmt, ap)
}

/// Original C global function `kom_revcomp` from `minibwa/kommon.c:198`.
pub fn kom_revcomp(len: u64, seq: &mut [u8]) {
    const KOM_COMP_TABLE: [u8; 256] = [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47,
        48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 84, 86, 71, 72, 69, 70,
        67, 68, 73, 74, 77, 76, 75, 78, 79, 80, 81, 89, 83, 65, 65, 66, 87, 88, 82, 90, 91, 92, 93,
        94, 95, 96, 116, 118, 103, 104, 101, 102, 99, 100, 105, 106, 109, 108, 107, 110, 111, 112,
        113, 121, 115, 97, 97, 98, 119, 120, 114, 122, 123, 124, 125, 126, 127, 128, 129, 130, 131,
        132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147, 148, 149,
        150, 151, 152, 153, 154, 155, 156, 157, 158, 159, 160, 161, 162, 163, 164, 165, 166, 167,
        168, 169, 170, 171, 172, 173, 174, 175, 176, 177, 178, 179, 180, 181, 182, 183, 184, 185,
        186, 187, 188, 189, 190, 191, 192, 193, 194, 195, 196, 197, 198, 199, 200, 201, 202, 203,
        204, 205, 206, 207, 208, 209, 210, 211, 212, 213, 214, 215, 216, 217, 218, 219, 220, 221,
        222, 223, 224, 225, 226, 227, 228, 229, 230, 231, 232, 233, 234, 235, 236, 237, 238, 239,
        240, 241, 242, 243, 244, 245, 246, 247, 248, 249, 250, 251, 252, 253, 254, 255,
    ];
    for i in 0..(len >> 1) as usize {
        let t = seq[len as usize - i - 1];
        seq[len as usize - i - 1] = KOM_COMP_TABLE[seq[i] as usize];
        seq[i] = KOM_COMP_TABLE[t as usize];
    }
    if len & 1 != 0 {
        let mid = (len >> 1) as usize;
        seq[mid] = KOM_COMP_TABLE[seq[mid] as usize];
    }
}

/// Original C global function `gettimeofday` from `minibwa/kommon.c:261`.
pub fn gettimeofday() -> timeval {
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    timeval {
        tv_sec: dur.as_secs() as libc::time_t,
        tv_usec: dur.subsec_micros() as libc::suseconds_t,
    }
}

/// Original C global function `kom_cputime` from `minibwa/kommon.c:279`.
pub fn kom_cputime() -> f64 {
    let mut r: rusage = unsafe { std::mem::zeroed() };
    unsafe {
        libc::getrusage(libc::RUSAGE_SELF, &mut r);
    }
    r.ru_utime.tv_sec as f64
        + r.ru_stime.tv_sec as f64
        + 1e-6 * (r.ru_utime.tv_usec + r.ru_stime.tv_usec) as f64
}

/// Original C global function `kom_peakrss` from `minibwa/kommon.c:296`.
pub fn kom_peakrss() -> c_long {
    let mut r: rusage = unsafe { std::mem::zeroed() };
    unsafe {
        libc::getrusage(libc::RUSAGE_SELF, &mut r);
    }
    if cfg!(target_os = "linux") {
        r.ru_maxrss * 1024
    } else {
        r.ru_maxrss
    }
}

/// Original C global function `kom_realtime` from `minibwa/kommon.c:321`.
pub fn kom_realtime() -> f64 {
    static REALTIME0: OnceLock<Mutex<f64>> = OnceLock::new();
    let tp = gettimeofday();
    let t = tp.tv_sec as f64 + tp.tv_usec as f64 * 1e-6;
    let lock = REALTIME0.get_or_init(|| Mutex::new(-1.0));
    let mut realtime0 = lock.lock().unwrap();
    if *realtime0 < 0.0 {
        *realtime0 = t;
    }
    t - *realtime0
}

/// Original C global function `kom_percent_cpu` from `minibwa/kommon.c:332`.
pub fn kom_percent_cpu() -> f64 {
    (kom_cputime() + 1e-6) / (kom_realtime() + 1e-6)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_num_matches_suffix_rounding_rules() {
        assert_eq!(kom_parse_num("12.4k rest"), (12400, 5));
        assert_eq!(kom_parse_num("1.5M"), (1500000, 4));
        assert_eq!(kom_parse_num("2Gx"), (2000000000, 2));
        assert_eq!(kom_parse_num("17"), (17, 2));
        assert_eq!(kom_parse_num("  .5k"), (500, 5));
        assert_eq!(kom_parse_num("abc"), (0, 0));
    }

    #[test]
    fn splitmix_and_u64todbl_match_known_vectors() {
        let mut x = 0u64;
        assert_eq!(kom_splitmix64(&mut x), 0xe220a8397b1dcdaf);
        assert_eq!(x, 0x9e3779b97f4a7c15);
        assert_eq!(kom_u64todbl(0), 0.0);
        assert!(kom_u64todbl(u64::MAX) < 1.0);
    }

    #[test]
    fn sprintf_lite_writes_supported_conversions() {
        let mut s = kstring_t::default();
        let len = kom_sprintf_lite(
            &mut s,
            "%s:%d/%ld/%u/%c",
            &[
                kom_sprintf_arg::s("read"),
                kom_sprintf_arg::d(-7),
                kom_sprintf_arg::ld(123456789),
                kom_sprintf_arg::u(42),
                kom_sprintf_arg::c(b'X' as i32),
            ],
        );
        assert_eq!(len as usize, "read:-7/123456789/42/X".len());
        assert_eq!(&s.s[..s.l], b"read:-7/123456789/42/X");
        assert_eq!(s.s[s.l], 0);
    }

    #[test]
    fn revcomp_matches_real_chrm_prefix() {
        let mut seq = b"GATCACAGGTCTATCACCCTATTAACCACTCACGGGAGCTCTCCATGCAT".to_vec();
        let len = seq.len() as u64;
        kom_revcomp(len, &mut seq);
        assert_eq!(&seq, b"ATGCATGGAGAGCTCCCGTGAGTGGTTAATAGGGTGATAGACCTGTGATC");
    }

    #[test]
    fn timing_functions_return_nonnegative_values() {
        let tv = gettimeofday();
        assert!(tv.tv_sec > 0);
        assert!(kom_cputime() >= 0.0);
        assert!(kom_peakrss() >= 0);
        assert!(kom_realtime() >= 0.0);
        assert!(kom_percent_cpu() >= 0.0);
    }
}
