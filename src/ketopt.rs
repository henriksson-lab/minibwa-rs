
#![allow(
    unused_variables,
    dead_code,
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals
)]

pub const ko_no_argument: i32 = 0;
pub const ko_required_argument: i32 = 1;
pub const ko_optional_argument: i32 = 2;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ketopt_t {
    pub ind: i32,
    pub opt: i32,
    pub arg: Option<String>,
    pub longidx: i32,
    pub i: i32,
    pub pos: i32,
    pub n_args: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ko_longopt_t {
    pub name: Option<String>,
    pub has_arg: i32,
    pub val: i32,
}

pub static KETOPT_INIT: ketopt_t = ketopt_t {
    ind: 1,
    opt: 0,
    arg: None,
    longidx: -1,
    i: 1,
    pos: 0,
    n_args: 0,
};

/// Original C static function `ketopt_permute` from `minibwa/ketopt.h:27`.
pub fn ketopt_permute(argv: &mut [String], j: i32, n: i32) {
    let mut k = 0;
    let p = argv[j as usize].clone();
    while k < n {
        let dst = (j - k) as usize;
        let src = (j - k - 1) as usize;
        argv[dst] = argv[src].clone();
        k += 1;
    }
    argv[(j - k) as usize] = p;
}

/// Original C static function `ketopt` from `minibwa/ketopt.h:56`.
pub fn ketopt(
    s: &mut ketopt_t,
    argc: i32,
    argv: &mut [String],
    permute: i32,
    ostr: &str,
    longopts: Option<&[ko_longopt_t]>,
) -> i32 {
    let mut opt: i32;
    if permute != 0 {
        while s.i < argc {
            let a = argv[s.i as usize].as_bytes();
            if a.first() == Some(&b'-') && a.get(1) != Some(&b'\0') && a.len() > 1 {
                break;
            }
            s.i += 1;
            s.n_args += 1;
        }
    }
    s.arg = None;
    s.longidx = -1;
    let i0 = s.i;
    if s.i >= argc {
        s.ind = s.i - s.n_args;
        return -1;
    }
    let cur = argv[s.i as usize].clone();
    let cur_b = cur.as_bytes();
    if cur_b.first() != Some(&b'-') || cur_b.get(1).is_none() {
        s.ind = s.i - s.n_args;
        return -1;
    }
    if cur_b.first() == Some(&b'-') && cur_b.get(1) == Some(&b'-') {
        if cur_b.get(2).is_none() {
            ketopt_permute(argv, s.i, s.n_args);
            s.i += 1;
            s.ind = s.i - s.n_args;
            return -1;
        }
        s.opt = 0;
        opt = '?' as i32;
        s.pos = -1;
        if let Some(longopts) = longopts {
            let mut j = 2usize;
            while j < cur_b.len() && cur_b[j] != b'=' {
                j += 1;
            }
            let query = &cur[2..j];
            let mut n_exact = 0;
            let mut n_partial = 0;
            let mut exact_idx = 0usize;
            let mut partial_idx = 0usize;
            for (k, o) in longopts.iter().enumerate() {
                let Some(name) = &o.name else {
                    break;
                };
                if name.starts_with(query) {
                    if name.len() == query.len() {
                        n_exact += 1;
                        exact_idx = k;
                    } else {
                        n_partial += 1;
                        partial_idx = k;
                    }
                }
            }
            if n_exact > 1 || (n_exact == 0 && n_partial > 1) {
                s.i += 1;
                return '?' as i32;
            }
            let o_idx = if n_exact == 1 {
                Some(exact_idx)
            } else if n_partial == 1 {
                Some(partial_idx)
            } else {
                None
            };
            if let Some(o_idx) = o_idx {
                let o = &longopts[o_idx];
                s.opt = o.val;
                opt = o.val;
                s.longidx = o_idx as i32;
                if j < cur_b.len() && cur_b[j] == b'=' {
                    s.arg = Some(cur[j + 1..].to_string());
                }
                if o.has_arg == ko_required_argument && j == cur_b.len() {
                    if s.i < argc - 1 {
                        s.i += 1;
                        s.arg = Some(argv[s.i as usize].clone());
                    } else {
                        opt = ':' as i32;
                    }
                }
            }
        }
    } else {
        if s.pos == 0 {
            s.pos = 1;
        }
        let pos = s.pos as usize;
        opt = cur_b[pos] as i32;
        s.opt = opt;
        s.pos += 1;
        if let Some(p) = ostr.as_bytes().iter().position(|&c| c as i32 == opt) {
            if ostr.as_bytes().get(p + 1) == Some(&b':') {
                if s.pos as usize >= cur_b.len() {
                    if s.i < argc - 1 {
                        s.i += 1;
                        s.arg = Some(argv[s.i as usize].clone());
                    } else {
                        opt = ':' as i32;
                    }
                } else {
                    s.arg = Some(cur[s.pos as usize..].to_string());
                }
                s.pos = -1;
            }
        } else {
            opt = '?' as i32;
        }
    }
    if s.pos < 0 || s.pos as usize >= argv[s.i as usize].len() {
        s.i += 1;
        s.pos = 0;
        if s.n_args > 0 {
            let mut j = i0;
            while j < s.i {
                ketopt_permute(argv, j, s.n_args);
                j += 1;
            }
        }
    }
    s.ind = s.i - s.n_args;
    opt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ketopt_parses_short_options_and_permutes_arguments() {
        let mut argv = vec![
            "prog".to_string(),
            "input.fa".to_string(),
            "-abval".to_string(),
            "tail".to_string(),
        ];
        let mut st = KETOPT_INIT.clone();
        assert_eq!(ketopt(&mut st, 4, &mut argv, 1, "ab:", None), 'a' as i32);
        assert_eq!(st.arg, None);
        assert_eq!(ketopt(&mut st, 4, &mut argv, 1, "ab:", None), 'b' as i32);
        assert_eq!(st.arg.as_deref(), Some("val"));
        assert_eq!(ketopt(&mut st, 4, &mut argv, 1, "ab:", None), -1);
        assert_eq!(st.ind, 2);
        assert_eq!(argv[1], "-abval");
        assert_eq!(argv[2], "input.fa");
    }

    #[test]
    fn ketopt_parses_long_options_and_errors() {
        let longopts = vec![
            ko_longopt_t {
                name: Some("threads".to_string()),
                has_arg: ko_required_argument,
                val: 't' as i32,
            },
            ko_longopt_t {
                name: Some("verbose".to_string()),
                has_arg: ko_no_argument,
                val: 'v' as i32,
            },
            ko_longopt_t {
                name: None,
                has_arg: 0,
                val: 0,
            },
        ];
        let mut argv = vec![
            "prog".to_string(),
            "--threads=8".to_string(),
            "--verb".to_string(),
            "--unknown".to_string(),
        ];
        let mut st = KETOPT_INIT.clone();
        assert_eq!(
            ketopt(&mut st, 4, &mut argv, 0, "", Some(&longopts)),
            't' as i32
        );
        assert_eq!(st.longidx, 0);
        assert_eq!(st.arg.as_deref(), Some("8"));
        assert_eq!(
            ketopt(&mut st, 4, &mut argv, 0, "", Some(&longopts)),
            'v' as i32
        );
        assert_eq!(st.longidx, 1);
        assert_eq!(
            ketopt(&mut st, 4, &mut argv, 0, "", Some(&longopts)),
            '?' as i32
        );
    }
}
