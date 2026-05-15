#![allow(unused_variables, dead_code, non_snake_case, non_camel_case_types)]

use crate::bwt::{mb_bwt_destroy, mb_bwt_load, mb_bwt_rank2a, mb_bwt_sa, mb_bwt_sa_batch};
use crate::fastmap::main_fastmap;
use crate::index::{main_fa2bit, main_genbwt, main_genraw, main_gensa, main_index, main_raw2bwt};
use crate::ketopt::{ketopt, ko_longopt_t, KETOPT_INIT};
use crate::kommon::{
    kom_cputime, kom_panic, kom_parse_num, kom_peakrss, kom_realtime, kom_splitmix64,
};
use crate::map_main::main_map;

pub const MB_VERSION: &str = "0.0-r310-dirty";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum mb_bench_type_t {
    MB_BENCH_2A = 0,
    MB_BENCH_SA = 1,
    MB_BENCH_MSA = 2,
}

/// Original C static function `usage` from `minibwa/main.c:23`.
pub fn usage(to_stdout: bool, is_long: i32) -> (i32, String) {
    let mut out = String::new();
    out.push_str("Usage: minibwt <command> <arguments>\n");
    out.push_str("Commands:\n");
    if is_long != 0 {
        out.push_str("  General:\n");
        out.push_str("    index      index reference FASTA\n");
        out.push_str("    map        read alignment\n");
        out.push_str("    version    print the version number\n");
        out.push_str("  Separate indexing routines:\n");
        out.push_str("    fa2bit     convert FASTA to the long-2bit format\n");
        out.push_str("    genraw     generate BWT from pac with the BWT-SW algorithm\n");
        out.push_str("    raw2bwt    recode bwtgen raw BWT\n");
        out.push_str("    gensa      generate sampled SA from BWT\n");
        out.push_str("    genbwt     generate BWT+SSA from long-2bit with libsais\n");
        out.push_str("  Debugging:\n");
        out.push_str("    bench      performance evaluation\n");
        out.push_str("    fastmap    test seeding strategies\n");
        out.push_str("  Help:\n");
        out.push_str("    --help     print this help message\n");
    } else {
        out.push_str("  index      index reference FASTA\n");
        out.push_str("  map        read alignment\n");
        out.push_str("  version    print the version number\n");
    }
    (if to_stdout { 0 } else { 1 }, out)
}

/// Original C global function `main` from `minibwa/main.c:49`.
pub fn main(argv: &[String]) -> i32 {
    let argc = argv.len();
    let _ = kom_realtime();
    if argc == 1 {
        return usage(true, 0).0;
    }
    let ret = if argv[1] == "index" {
        main_index(&argv[1..]).0
    } else if argv[1] == "map" || argv[1] == "mem" {
        main_map(&argv[1..]).0
    } else if argv[1] == "fa2bit" {
        main_fa2bit(&argv[1..]).0
    } else if argv[1] == "genraw" {
        main_genraw(&argv[1..]).0
    } else if argv[1] == "raw2bwt" {
        main_raw2bwt(&argv[1..]).0
    } else if argv[1] == "genbwt" {
        main_genbwt(&argv[1..]).0
    } else if argv[1] == "gensa" {
        main_gensa(&argv[1..]).0
    } else if argv[1] == "bench" {
        main_bench(&argv[1..]).0
    } else if argv[1] == "fastmap" {
        main_fastmap(&argv[1..]).0
    } else if argv[1] == "--help" {
        return usage(true, 1).0;
    } else if argv[1] == "version" {
        return 0;
    } else {
        return 1;
    };
    let _ = (ret, kom_cputime(), kom_peakrss());
    0
}

/// Original C static function `usage_bench` from `minibwa/main.c:78`.
pub fn usage_bench(to_stdout: bool, intv: i32) -> (i32, String) {
    let mut out = String::new();
    out.push_str("Usage: minibwa bench [options] <in.mbw>\n");
    out.push_str("Options:\n");
    out.push_str("  -b STR         type: 2a, sa or msa [2a]\n");
    out.push_str("  -n NUM         number of data points [1m]\n");
    out.push_str(&format!(
        "  -v INT         interval size for msa [{}]\n",
        intv
    ));
    out.push_str("  -p             print results for each data point\n");
    out.push_str("  -1             use unbatched sa for msa\n");
    out.push_str("  --help         print this help message\n");
    (if to_stdout { 0 } else { 1 }, out)
}

/// Original C global function `main_bench` from `minibwa/main.c:85`.
pub fn main_bench(argv: &[String]) -> (i32, String, String) {
    let long_opts = [ko_longopt_t {
        name: Some("help".into()),
        has_arg: 0,
        val: 901,
    }];
    let argc = argv.len() as i32;
    let mut args = argv.to_vec();
    let mut o = KETOPT_INIT.clone();
    let mut type_ = mb_bench_type_t::MB_BENCH_2A;
    let mut x = 11u64;
    let mut cs = 1u64;
    let mut n = 1_000_000i64;
    let mut print_val = 0;
    let mut use_single = 0;
    let mut intv = 20i32;
    loop {
        let c = ketopt(&mut o, argc, &mut args, 1, "pn:b:v:1", Some(&long_opts));
        if c < 0 {
            break;
        }
        if c == 'n' as i32 {
            n = kom_parse_num(o.arg.as_deref().unwrap_or("0")).0;
        } else if c == 'p' as i32 {
            print_val = 1;
        } else if c == '1' as i32 {
            use_single = 1;
        } else if c == 'v' as i32 {
            intv = o.arg.as_deref().unwrap_or("0").parse().unwrap_or(0);
        } else if c == 'b' as i32 {
            type_ = match o.arg.as_deref().unwrap_or("2a") {
                "2a" => mb_bench_type_t::MB_BENCH_2A,
                "sa" => mb_bench_type_t::MB_BENCH_SA,
                "msa" => mb_bench_type_t::MB_BENCH_MSA,
                _ => kom_panic("main_bench", "unknown type"),
            };
        } else if c == 901 {
            let (ret, out) = usage_bench(true, intv);
            return (ret, out, String::new());
        }
    }
    if argc - o.ind < 1 {
        let (ret, err) = usage_bench(false, intv);
        return (ret, String::new(), err);
    }
    let Some(bwt) = mb_bwt_load(&args[o.ind as usize]) else {
        return (1, String::new(), String::new());
    };
    let t = kom_cputime();
    let mut out = String::new();
    if type_ == mb_bench_type_t::MB_BENCH_2A {
        for _ in 0..n {
            let k = kom_splitmix64(&mut x) % bwt.seq_len;
            let l = kom_splitmix64(&mut x) % bwt.seq_len;
            let mut cntk = [0u64; 4];
            let mut cntl = [0u64; 4];
            mb_bwt_rank2a(&bwt, k, l, &mut cntk, &mut cntl);
            cs = cs.wrapping_mul(cntk[1]).wrapping_add(cntl[0]);
            if print_val != 0 {
                out.push_str(&format!("{}\n", cntk[1]));
            }
        }
    } else if type_ == mb_bench_type_t::MB_BENCH_SA {
        for _ in 0..n {
            let k = kom_splitmix64(&mut x) % bwt.seq_len;
            let s = mb_bwt_sa(&bwt, k);
            cs = cs.wrapping_mul(0xbf58476d1ce4e5b9) ^ s;
            if print_val != 0 {
                out.push_str(&format!("{s}\n"));
            }
        }
    } else {
        for _ in 0..n {
            let k = kom_splitmix64(&mut x) % bwt.seq_len;
            let l = if k + (intv as u64) < bwt.seq_len {
                k + intv as u64
            } else {
                bwt.seq_len
            };
            let mut xor = 0u64;
            if use_single != 0 {
                for j in k..l {
                    xor ^= mb_bwt_sa(&bwt, j);
                }
            } else {
                let n_sa = (l - k) as usize;
                let mut sa = (0..n_sa).map(|j| k + j as u64).collect::<Vec<_>>();
                mb_bwt_sa_batch((), &bwt, (l - k) as i64, &mut sa);
                for s in sa {
                    xor ^= s;
                }
            }
            cs = cs.wrapping_mul(0xbf58476d1ce4e5b9) ^ xor;
            if print_val != 0 {
                out.push_str(&format!("{xor}\n"));
            }
        }
    }
    mb_bwt_destroy(Some(bwt));
    let err = format!("checksum = {cs:x}\nt = {:.3}\n", kom_cputime() - t);
    (0, out, err)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage_formats_short_and_long_command_lists() {
        let (ret, short) = usage(true, 0);
        assert_eq!(ret, 0);
        assert!(short.contains("Usage: minibwt <command> <arguments>\n"));
        assert!(short.contains("  map        read alignment\n"));
        assert!(!short.contains("Separate indexing routines"));

        let (ret, long) = usage(false, 1);
        assert_eq!(ret, 1);
        assert!(long.contains("    genbwt     generate BWT+SSA from long-2bit with libsais\n"));
        assert!(long.contains("    fastmap    test seeding strategies\n"));
    }

    #[test]
    fn usage_bench_formats_options() {
        assert_eq!(
            (
                mb_bench_type_t::MB_BENCH_2A as i32,
                mb_bench_type_t::MB_BENCH_SA as i32,
                mb_bench_type_t::MB_BENCH_MSA as i32,
            ),
            (0, 1, 2)
        );
        let (ret, out) = usage_bench(false, 20);
        assert_eq!(ret, 1);
        assert!(out.contains("Usage: minibwa bench [options] <in.mbw>\n"));
        assert!(out.contains("  -v INT         interval size for msa [20]\n"));
        assert!(out.contains("  --help         print this help message\n"));
    }
}
