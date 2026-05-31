use std::io::Write;
use std::process::{Command, Stdio};

use minibwa_rs::cli::run_with_writers;

#[test]
fn cli_prints_translated_top_level_usage_and_version() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");

    let usage = Command::new(rust_bin).output().unwrap();
    assert_eq!(usage.status.code(), Some(0));
    let usage_stdout = String::from_utf8(usage.stdout).unwrap();
    assert!(usage_stdout.starts_with("Usage: minibwt <command> <arguments>\n"));
    assert!(usage_stdout.contains("  index      index reference FASTA\n"));
    assert!(usage.stderr.is_empty());

    let help = Command::new(rust_bin).arg("--help").output().unwrap();
    assert_eq!(help.status.code(), Some(0));
    let help_stdout = String::from_utf8(help.stdout).unwrap();
    assert!(help_stdout.contains("  Separate indexing routines:\n"));
    assert!(help_stdout.contains("    --help     print this help message\n"));
    assert!(help.stderr.is_empty());

    let version = Command::new(rust_bin).arg("version").output().unwrap();
    assert_eq!(version.status.code(), Some(0));
    assert_eq!(
        String::from_utf8(version.stdout).unwrap(),
        "0.0-r352-dirty\n"
    );
    assert!(version.stderr.is_empty());
}

#[test]
fn cli_top_level_usage_version_and_unknown_match_original() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");

    for args in [
        Vec::<&str>::new(),
        vec!["--help"],
        vec!["version"],
        vec!["definitely-not-a-command"],
    ] {
        let rust = Command::new(rust_bin).args(&args).output().unwrap();
        let original = Command::new(original_bin).args(&args).output().unwrap();
        assert_eq!(
            rust.status.code(),
            original.status.code(),
            "status for {args:?}"
        );
        assert_eq!(rust.stdout, original.stdout, "stdout for {args:?}");
        assert_eq!(rust.stderr, original.stderr, "stderr for {args:?}");
    }
}

#[test]
fn cli_in_process_dispatch_and_footer_use_c_string_boundaries() {
    let argv = vec![
        "minibwa-rs\0argv-tail".to_string(),
        "map\0hidden-command".to_string(),
        "--help".to_string(),
    ];
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let status = run_with_writers(&argv, &mut stdout, &mut stderr).unwrap();
    assert_eq!(status, 0);
    assert!(stdout.starts_with(b"Usage: minibwa map [options] <in.idx> <in.fastq>\n"));

    let stderr = String::from_utf8(stderr).unwrap();
    assert!(stderr.contains("[M::main] CMD: minibwa-rs map --help\n"));
    assert!(!stderr.contains("argv-tail"));
    assert!(!stderr.contains("hidden-command"));
}

#[test]
fn cli_in_process_unknown_command_diagnostic_uses_c_string_boundary() {
    let argv = vec![
        "minibwa-rs".to_string(),
        "not-a-command\0hidden-command".to_string(),
    ];
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let status = run_with_writers(&argv, &mut stdout, &mut stderr).unwrap();
    assert_eq!(status, 1);
    assert!(stdout.is_empty());
    assert_eq!(
        String::from_utf8(stderr).unwrap(),
        "ERROR: unknown command 'not-a-command'\n"
    );
}

#[test]
fn cli_prints_translated_subcommand_text() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");

    let map_help = Command::new(rust_bin)
        .args(["map", "--help"])
        .output()
        .unwrap();
    assert_eq!(map_help.status.code(), Some(0));
    let map_stdout = String::from_utf8(map_help.stdout).unwrap();
    assert!(map_stdout.starts_with("Usage: minibwa map [options] <in.idx> <in.fastq>\n"));
    assert!(map_stdout.contains("    --version        print version number\n"));
    assert!(String::from_utf8(map_help.stderr)
        .unwrap()
        .starts_with("[M::main] Version: "));

    let fastmap_help = Command::new(rust_bin)
        .args(["fastmap", "--help"])
        .output()
        .unwrap();
    assert_eq!(fastmap_help.status.code(), Some(0));
    let fastmap_stdout = String::from_utf8(fastmap_help.stdout).unwrap();
    assert!(fastmap_stdout.starts_with("Usage: minibwa fastmap [options] <idx-prefix> <in.fq>\n"));
    assert!(fastmap_stdout.contains("  --help     print this help message\n"));
    assert!(String::from_utf8(fastmap_help.stderr)
        .unwrap()
        .starts_with("[M::main] Version: "));
}

#[test]
fn cli_map_and_fastmap_help_stdout_matches_original() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");

    for args in [
        vec!["map", "--help"],
        vec!["map", "-k", "21", "-w", "9", "-A", "3", "--help"],
        vec!["fastmap", "--help"],
    ] {
        let rust = Command::new(rust_bin).args(&args).output().unwrap();
        let original = Command::new(original_bin).args(&args).output().unwrap();
        assert_eq!(rust.status.code(), Some(0), "rust status for {args:?}");
        assert_eq!(
            original.status.code(),
            Some(0),
            "original status for {args:?}"
        );
        assert_eq!(rust.stdout, original.stdout, "stdout for {args:?}");
    }
}

#[test]
fn cli_map_and_mem_version_match_original() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");

    for command in ["map", "mem"] {
        let rust = Command::new(rust_bin)
            .args([command, "--version"])
            .output()
            .unwrap();
        let original = Command::new(original_bin)
            .args([command, "--version"])
            .output()
            .unwrap();
        assert_eq!(rust.status.code(), Some(0), "rust status for {command}");
        assert_eq!(
            original.status.code(),
            Some(0),
            "original status for {command}"
        );
        assert_eq!(rust.stdout, original.stdout, "stdout for {command}");
        assert_eq!(rust.stderr, original.stderr, "stderr for {command}");
    }
}

#[test]
fn cli_prints_translated_index_subcommand_usage() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");

    for (command, usage) in [
        (
            "index",
            "Usage: minibwa index [options] <in.fasta> [out.prefix]\n",
        ),
        (
            "fa2bit",
            "Usage: minibwa fa2bit [options] <in.fa> <out.l2b>\n",
        ),
        (
            "genraw",
            "Usage: minibwa genraw [options] <in.pac> <out.raw-bwt>\n",
        ),
        ("raw2bwt", "Usage: minibwa raw2bwt <raw.bwt> <recode.bwt>\n"),
        (
            "genbwt",
            "Usage: minibwa genbwt [options] <in.l2b> <out.bwt>\n",
        ),
        (
            "gensa",
            "Usage: minibwa gensa [options] <in.bwt> <out.bwt>\n",
        ),
    ] {
        let help = Command::new(rust_bin)
            .args([command, "--help"])
            .output()
            .unwrap();
        assert_eq!(help.status.code(), Some(0), "help status for {command}");
        assert!(
            String::from_utf8(help.stdout).unwrap().starts_with(usage),
            "help stdout for {command}"
        );
        assert!(
            String::from_utf8(help.stderr)
                .unwrap()
                .starts_with("[M::main] Version: "),
            "help stderr for {command}"
        );

        let missing = Command::new(rust_bin).arg(command).output().unwrap();
        assert_eq!(
            missing.status.code(),
            Some(0),
            "original top-level main returns 0 for {command} usage errors"
        );
        assert!(
            String::from_utf8(missing.stderr)
                .unwrap()
                .starts_with(usage),
            "missing-args stderr for {command}"
        );
    }
}

#[test]
fn cli_index_stage_help_stdout_matches_original() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");

    for args in [
        vec!["index", "--help"],
        vec!["fa2bit", "--help"],
        vec!["fa2bit", "-s", "7", "--help"],
        vec!["genraw", "--help"],
        vec!["genraw", "--meth", "--help"],
        vec!["raw2bwt", "--help"],
        vec!["genbwt", "--help"],
        vec!["genbwt", "--meth", "--help"],
        vec!["genbwt", "-u", "3", "-t", "2", "--help"],
        vec!["gensa", "--help"],
        vec!["gensa", "--meth", "--help"],
        vec!["gensa", "-u", "6", "--help"],
    ] {
        let rust = Command::new(rust_bin).args(&args).output().unwrap();
        let original = Command::new(original_bin).args(&args).output().unwrap();
        assert_eq!(rust.status.code(), Some(0), "rust status for {args:?}");
        assert_eq!(
            original.status.code(),
            Some(0),
            "original status for {args:?}"
        );
        assert_eq!(rust.stdout, original.stdout, "stdout for {args:?}");
    }
}

#[test]
fn cli_index_stage_option_usage_errors_match_original() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");

    for args in [
        vec!["index", "-u"],
        vec!["index", "--unknown"],
        vec!["fa2bit", "-s"],
        vec!["fa2bit", "--unknown"],
        vec!["genraw", "-b"],
        vec!["genraw", "--meth"],
        vec!["genraw", "--unknown"],
        vec!["genbwt", "-u"],
        vec!["genbwt", "--meth"],
        vec!["genbwt", "--unknown"],
        vec!["gensa", "-u"],
        vec!["gensa", "--meth"],
        vec!["gensa", "--unknown"],
        vec!["raw2bwt", "--unknown"],
    ] {
        let rust = Command::new(rust_bin).args(&args).output().unwrap();
        let original = Command::new(original_bin).args(&args).output().unwrap();
        assert_eq!(
            rust.status.code(),
            original.status.code(),
            "status for {args:?}"
        );
        assert_eq!(rust.stdout, original.stdout, "stdout for {args:?}");
        assert_eq!(rust.stderr, original.stderr, "stderr for {args:?}");
    }
}

#[cfg(unix)]
#[test]
fn cli_fa2bit_missing_input_segfaults_like_original() {
    use std::os::unix::process::ExitStatusExt;

    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let dir =
        std::env::temp_dir().join(format!("minibwa_rs_missing_fa2bit_{}", std::process::id()));
    let missing = dir.join("missing.fa").to_string_lossy().into_owned();
    let out = dir.join("out").to_string_lossy().into_owned();

    for args in [
        vec!["fa2bit", &missing, &out],
        vec!["fa2bit", "-p", &missing, &out],
    ] {
        let rust = Command::new(rust_bin).args(&args).output().unwrap();
        let original = Command::new(original_bin).args(&args).output().unwrap();
        assert_eq!(
            rust.status.signal(),
            original.status.signal(),
            "signal for {args:?}"
        );
        assert_eq!(rust.status.signal(), Some(11), "rust signal for {args:?}");
        assert_eq!(rust.stdout, original.stdout, "stdout for {args:?}");
        assert_eq!(rust.stderr, original.stderr, "stderr for {args:?}");
    }
}

#[test]
fn cli_genraw_missing_input_diagnostic_matches_original() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let dir =
        std::env::temp_dir().join(format!("minibwa_rs_genraw_missing_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let missing = dir.join("missing.pac");
    let empty = dir.join("empty.pac");
    let out = dir.join("out.raw");
    std::fs::write(&empty, b"").unwrap();

    for input in [&missing, &empty] {
        let args = ["genraw", &input.to_string_lossy(), &out.to_string_lossy()];
        let rust = Command::new(rust_bin).args(args).output().unwrap();
        let original = Command::new(original_bin).args(args).output().unwrap();
        assert_eq!(rust.status.code(), original.status.code());
        assert_eq!(rust.stdout, original.stdout);
        assert_eq!(rust.stderr, original.stderr);
    }

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn cli_genraw_single_strand_pac_failure_matches_original() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let dir = std::env::temp_dir().join(format!(
        "minibwa_rs_genraw_single_pac_{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let fasta = dir.join("in.fa");
    let pac = dir.join("in.pac");
    let rust_raw = dir.join("rust.raw");
    let original_raw = dir.join("original.raw");
    std::fs::write(&fasta, b">ctg\nACGTACGT\n").unwrap();
    let fa2bit = Command::new(rust_bin)
        .args([
            "fa2bit",
            "-p",
            &fasta.to_string_lossy(),
            &pac.to_string_lossy(),
        ])
        .output()
        .unwrap();
    assert_eq!(fa2bit.status.code(), Some(0));

    let rust = Command::new(rust_bin)
        .args([
            "genraw",
            &pac.to_string_lossy(),
            &rust_raw.to_string_lossy(),
        ])
        .output()
        .unwrap();
    let original = Command::new(original_bin)
        .args([
            "genraw",
            &pac.to_string_lossy(),
            &original_raw.to_string_lossy(),
        ])
        .output()
        .unwrap();
    assert_eq!(rust.status.code(), original.status.code());
    assert_eq!(rust.stdout, original.stdout);
    assert_eq!(rust.stderr, original.stderr);

    let full = std::path::Path::new("/dev/full");
    if full.exists() {
        let args = ["genraw", &pac.to_string_lossy(), "/dev/full"];
        let rust = Command::new(rust_bin).args(args).output().unwrap();
        let original = Command::new(original_bin).args(args).output().unwrap();
        assert_eq!(rust.status.code(), original.status.code());
        assert_eq!(rust.stdout, original.stdout);
        assert_eq!(rust.stderr, original.stderr);
    }

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn cli_genraw_output_open_failure_matches_original() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let dir = std::env::temp_dir().join(format!(
        "minibwa_rs_genraw_output_failure_{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let fasta = dir.join("in.fa");
    let pac = dir.join("in.pac");
    let out = dir.join("missing-dir").join("out.raw");
    std::fs::write(&fasta, b">ctg\nACGTACGTACGTACGTACGTACGTACGTACGT\n").unwrap();
    let fa2bit = Command::new(rust_bin)
        .args([
            "fa2bit",
            "-p",
            "-2",
            &fasta.to_string_lossy(),
            &pac.to_string_lossy(),
        ])
        .output()
        .unwrap();
    assert_eq!(fa2bit.status.code(), Some(0));

    let args = ["genraw", &pac.to_string_lossy(), &out.to_string_lossy()];
    let rust = Command::new(rust_bin).args(args).output().unwrap();
    let original = Command::new(original_bin).args(args).output().unwrap();
    assert_eq!(rust.status.code(), original.status.code());
    assert_eq!(rust.stdout, original.stdout);
    assert_eq!(rust.stderr, original.stderr);

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn cli_bench_help_stdout_matches_original() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");

    let rust = Command::new(rust_bin)
        .args(["bench", "--help"])
        .output()
        .unwrap();
    let original = Command::new(original_bin)
        .args(["bench", "--help"])
        .output()
        .unwrap();
    assert_eq!(rust.status.code(), Some(0));
    assert_eq!(original.status.code(), Some(0));
    assert_eq!(rust.stdout, original.stdout);
}

#[test]
fn cli_bench_usage_errors_match_original() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");

    for args in [
        vec!["bench"],
        vec!["bench", "-n"],
        vec!["bench", "--unknown"],
        vec!["bench", "-v"],
    ] {
        let rust = Command::new(rust_bin).args(&args).output().unwrap();
        let original = Command::new(original_bin).args(&args).output().unwrap();
        assert_eq!(
            rust.status.code(),
            original.status.code(),
            "status for {args:?}"
        );
        assert_eq!(rust.stdout, original.stdout, "stdout for {args:?}");
        assert_eq!(rust.stderr, original.stderr, "stderr for {args:?}");
    }
}

#[cfg(unix)]
#[test]
fn cli_bench_unknown_type_aborts_like_original() {
    use std::os::unix::process::ExitStatusExt;

    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let args = ["bench", "-b", "unknown"];

    let rust = Command::new(rust_bin).args(args).output().unwrap();
    let original = Command::new(original_bin).args(args).output().unwrap();
    assert_eq!(rust.status.signal(), Some(6));
    assert_eq!(original.status.signal(), Some(6));
    assert_eq!(rust.stdout, original.stdout);
    assert_eq!(rust.stderr, original.stderr);
}

#[cfg(unix)]
#[test]
fn cli_bench_missing_bwt_segfaults_like_original() {
    use std::os::unix::process::ExitStatusExt;

    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let missing = std::env::temp_dir().join(format!(
        "minibwa_rs_missing_bench_bwt_{}",
        std::process::id()
    ));
    let missing = missing.to_string_lossy().into_owned();
    let args = ["bench", &missing];

    let rust = Command::new(rust_bin).args(args).output().unwrap();
    let original = Command::new(original_bin).args(args).output().unwrap();
    assert_eq!(rust.status.signal(), Some(11));
    assert_eq!(original.status.signal(), Some(11));
    assert_eq!(rust.stdout, original.stdout);
    assert_eq!(rust.stderr, original.stderr);
}

#[cfg(unix)]
#[test]
fn cli_raw2bwt_missing_raw_segfaults_like_original() {
    use std::os::unix::process::ExitStatusExt;

    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let dir =
        std::env::temp_dir().join(format!("minibwa_rs_missing_raw2bwt_{}", std::process::id()));
    let missing = dir.join("missing.raw");
    let out = dir.join("out.mbw");
    let args = [
        "raw2bwt",
        &missing.to_string_lossy(),
        &out.to_string_lossy(),
    ];

    let rust = Command::new(rust_bin).args(args).output().unwrap();
    let original = Command::new(original_bin).args(args).output().unwrap();
    assert_eq!(rust.status.signal(), original.status.signal());
    assert_eq!(rust.status.signal(), Some(11));
    assert_eq!(rust.stdout, original.stdout);
    assert_eq!(rust.stderr, original.stderr);
}

#[cfg(unix)]
#[test]
fn cli_gensa_missing_input_segfaults_like_original() {
    use std::os::unix::process::ExitStatusExt;

    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let dir = std::env::temp_dir().join(format!("minibwa_rs_missing_gensa_{}", std::process::id()));
    let missing_mbw = dir.join("missing.mbw").to_string_lossy().into_owned();
    let missing_raw = dir.join("missing.raw").to_string_lossy().into_owned();
    let out = dir.join("out.mbw").to_string_lossy().into_owned();

    for args in [
        vec!["gensa", &missing_mbw, &out],
        vec!["gensa", "-r", &missing_raw, &out],
    ] {
        let rust = Command::new(rust_bin).args(&args).output().unwrap();
        let original = Command::new(original_bin).args(&args).output().unwrap();
        assert_eq!(rust.status.signal(), original.status.signal());
        assert_eq!(rust.status.signal(), Some(11));
        assert_eq!(rust.stdout, original.stdout);
        assert_eq!(rust.stderr, original.stderr);
    }
}

#[cfg(unix)]
#[test]
fn cli_index_truncated_binary_assertions_match_original() {
    use std::os::unix::process::ExitStatusExt;

    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let dir = std::env::temp_dir().join(format!(
        "minibwa_rs_truncated_index_bin_{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();

    let magic_l2b = dir.join("magic-only.l2b");
    let short_raw = dir.join("short.raw");
    std::fs::write(&magic_l2b, b"L2B\x01").unwrap();
    std::fs::write(&short_raw, [0u8; 8]).unwrap();

    for args in [
        vec![
            "genbwt".to_string(),
            magic_l2b.to_string_lossy().into_owned(),
            dir.join("out.mbw").to_string_lossy().into_owned(),
        ],
        vec![
            "raw2bwt".to_string(),
            short_raw.to_string_lossy().into_owned(),
            dir.join("out2.mbw").to_string_lossy().into_owned(),
        ],
    ] {
        let rust = Command::new(rust_bin).args(&args).output().unwrap();
        let original = Command::new(original_bin).args(&args).output().unwrap();
        assert_eq!(rust.status.signal(), Some(6), "rust signal for {args:?}");
        assert_eq!(
            original.status.signal(),
            Some(6),
            "original signal for {args:?}"
        );
        assert_eq!(rust.stdout, original.stdout, "stdout for {args:?}");
        assert_eq!(rust.stderr, original.stderr, "stderr for {args:?}");
    }

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn cli_genraw_truncated_pac_seek_failure_matches_original() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let dir = std::env::temp_dir().join(format!(
        "minibwa_rs_truncated_genraw_pac_{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let pac = dir.join("short.pac");
    let out = dir.join("out.raw");
    std::fs::write(&pac, [0u8, 4]).unwrap();
    let args = ["genraw", &pac.to_string_lossy(), &out.to_string_lossy()];

    let rust = Command::new(rust_bin).args(args).output().unwrap();
    let original = Command::new(original_bin).args(args).output().unwrap();
    assert_eq!(rust.status.code(), Some(1));
    assert_eq!(original.status.code(), Some(1));
    assert_eq!(rust.stdout, original.stdout);
    assert_eq!(rust.stderr, original.stderr);

    let _ = std::fs::remove_dir_all(dir);
}

#[cfg(unix)]
#[test]
fn cli_assertion_abort_paths_match_original() {
    use std::os::unix::process::ExitStatusExt;

    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let dir = std::env::temp_dir().join(format!("minibwa_rs_assert_abort_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let reads = dir.join("reads.fa");
    std::fs::write(&reads, b">r\nACGT\n").unwrap();
    let missing_index = dir.join("missing-index");
    let missing_l2b = dir.join("missing.l2b");
    let missing_fasta = dir.join("missing.fa");
    let out_prefix = dir.join("out");
    let out_mbw = dir.join("out.mbw");

    let cases = [
        vec![
            "map".to_string(),
            missing_index.to_string_lossy().into_owned(),
            reads.to_string_lossy().into_owned(),
        ],
        vec![
            "genbwt".to_string(),
            missing_l2b.to_string_lossy().into_owned(),
            out_mbw.to_string_lossy().into_owned(),
        ],
        vec![
            "index".to_string(),
            missing_fasta.to_string_lossy().into_owned(),
            out_prefix.to_string_lossy().into_owned(),
        ],
    ];

    for args in cases {
        let rust = Command::new(rust_bin).args(&args).output().unwrap();
        let original = Command::new(original_bin).args(&args).output().unwrap();
        assert_eq!(rust.status.signal(), Some(6), "rust signal for {args:?}");
        assert_eq!(
            original.status.signal(),
            Some(6),
            "original signal for {args:?}"
        );
        assert_eq!(rust.stdout, original.stdout, "stdout for {args:?}");
        assert_eq!(rust.stderr, original.stderr, "stderr for {args:?}");
    }

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn cli_prints_translated_bench_usage_and_real_output() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let mbw = format!("{manifest_dir}/minibwa/chrM-human.mbw");

    let help = Command::new(rust_bin)
        .args(["bench", "--help"])
        .output()
        .unwrap();
    assert_eq!(help.status.code(), Some(0));
    let help_stdout = String::from_utf8(help.stdout).unwrap();
    assert!(help_stdout.starts_with("Usage: minibwa bench [options] <in.mbw>\n"));
    assert!(help_stdout.contains("  --help         print this help message\n"));
    assert!(String::from_utf8(help.stderr)
        .unwrap()
        .starts_with("[M::main] Version: "));

    let output = Command::new(rust_bin)
        .args(["bench", "-n", "3", "-p", &mbw])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(String::from_utf8(output.stdout).unwrap().lines().count(), 3);
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("checksum = "));
    assert!(stderr.contains("\nt = "));
}

#[test]
fn cli_bench_values_match_original_real_fixture_vectors() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let mbw = format!("{manifest_dir}/minibwa/chrM-human.mbw");

    for (args, stdout, checksum) in [
        (
            vec!["bench", "-n", "5", "-p"],
            "4800\n6913\n2458\n378\n977\n",
            "checksum = 71ff6660b4bb79\n",
        ),
        (
            vec!["bench", "-b", "sa", "-n", "5", "-p"],
            "26041\n2767\n672\n23577\n14291\n",
            "checksum = 6b6eeb2a6d826425\n",
        ),
        (
            vec!["bench", "-b", "msa", "-n", "5", "-p", "-v", "4"],
            "14346\n15735\n878\n7117\n31745\n",
            "checksum = 893e3c2e7d9b9c26\n",
        ),
    ] {
        let mut full_args = args;
        full_args.push(&mbw);
        let output = Command::new(rust_bin).args(full_args).output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        assert_eq!(String::from_utf8(output.stdout).unwrap(), stdout);
        assert!(
            String::from_utf8(output.stderr)
                .unwrap()
                .starts_with(checksum),
            "stderr checksum did not start with {checksum:?}"
        );
    }
}

#[test]
fn cli_runs_translated_fastmap_on_real_fixture() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let index = format!("{manifest_dir}/minibwa/chrM-human");
    let reads = format!("{manifest_dir}/minibwa/test/chrM-read_1.fa.gz");

    let output = Command::new(rust_bin)
        .args(["fastmap", "-l", "19", "-w", "2", &index, &reads])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("SQ\t"));
    assert!(stdout.contains("\nEM\t"));
    assert!(stdout.contains("//\n"));
}

#[test]
fn cli_fastmap_stdout_matches_original_on_real_index_single_read() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let index = format!("{manifest_dir}/minibwa/chrM-human");
    let reads = format!("{manifest_dir}/minibwa/test/chrM-read_1.fa.gz");

    let args = ["fastmap", "-l", "19", "-w", "2", &index, &reads];
    let rust = Command::new(rust_bin).args(args).output().unwrap();
    let original = Command::new(original_bin).args(args).output().unwrap();
    assert_eq!(rust.status.code(), Some(0));
    assert_eq!(original.status.code(), Some(0));
    if rust.stdout != original.stdout {
        let first_diff = rust
            .stdout
            .iter()
            .zip(original.stdout.iter())
            .position(|(a, b)| a != b)
            .unwrap_or_else(|| rust.stdout.len().min(original.stdout.len()));
        panic!(
            "fastmap stdout mismatch: rust_len={} original_len={} first_diff={}",
            rust.stdout.len(),
            original.stdout.len(),
            first_diff
        );
    }
}

#[test]
fn cli_fastmap_batch_stdout_matches_original_on_real_index_reads() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let index = format!("{manifest_dir}/minibwa/chrM-human");
    let reads = format!("{manifest_dir}/minibwa/test/chrM-read_1.fa.gz");

    let args = ["fastmap", "-l", "19", "-w", "2", "-b", "7", &index, &reads];
    let rust = Command::new(rust_bin).args(args).output().unwrap();
    let original = Command::new(original_bin).args(args).output().unwrap();
    assert_eq!(rust.status.code(), Some(0));
    assert_eq!(original.status.code(), Some(0));
    assert_eq!(rust.stdout, original.stdout);
}

#[test]
fn cli_fastmap_option_matrix_stdout_matches_original() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let index = format!("{manifest_dir}/minibwa/chrM-human");
    let reads = format!("{manifest_dir}/minibwa/test/chrM-read_1.fa.gz");

    for extra in [
        vec!["-s", "2"],
        vec!["-l", "30"],
        vec!["-w", "1"],
        vec!["-b", "1"],
        vec!["-b", "25"],
        vec!["-l", "21", "-s", "2", "-w", "3", "-b", "11"],
    ] {
        let mut args = vec!["fastmap".to_string()];
        args.extend(extra.iter().map(|s| s.to_string()));
        args.extend([index.clone(), reads.clone()]);
        let rust = Command::new(rust_bin).args(&args).output().unwrap();
        let original = Command::new(original_bin).args(&args).output().unwrap();
        assert_eq!(rust.status.code(), Some(0), "rust status for {args:?}");
        assert_eq!(
            original.status.code(),
            Some(0),
            "original status for {args:?}"
        );
        assert_eq!(rust.stdout, original.stdout, "stdout for {args:?}");
    }
}

#[test]
fn cli_fastmap_stdin_stdout_matches_original() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let index = format!("{manifest_dir}/minibwa/chrM-human");
    let input = b">stdin_read\nACTCACCTGAGTTGTAAAAAACTCCAGTTGACACAAAATAGACTACGAAAGTGGCTTTAACATATCTGAACACACAATAGCTAAGACCCAAACTGGGATTAGATACCCCACTATGCTTAGCCCTAAACCTCAACAGTTAAATCAACAAAAC\n";

    let mut rust_child = Command::new(rust_bin)
        .args(["fastmap", "-l", "19", "-w", "2", &index, "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    rust_child.stdin.take().unwrap().write_all(input).unwrap();
    let rust = rust_child.wait_with_output().unwrap();

    let mut original_child = Command::new(original_bin)
        .args(["fastmap", "-l", "19", "-w", "2", &index, "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    original_child
        .stdin
        .take()
        .unwrap()
        .write_all(input)
        .unwrap();
    let original = original_child.wait_with_output().unwrap();

    assert_eq!(rust.status.code(), Some(0));
    assert_eq!(original.status.code(), Some(0));
    assert_eq!(rust.stdout, original.stdout);
}

#[test]
fn cli_fastmap_malformed_fastq_stops_like_original_without_bseq_warning() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let index = format!("{manifest_dir}/minibwa/chrM-human");
    let dir = std::env::temp_dir().join(format!(
        "minibwa_rs_fastmap_malformed_{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let reads = dir.join("next.fq");
    std::fs::write(&reads, b"@good\nACGT\n+\nFFFF\n@bad\nACGTACGT\n+\nFFFF\n").unwrap();
    let reads_s = reads.to_string_lossy().into_owned();

    let args = ["fastmap", "-l", "2", "-w", "2", &index, &reads_s];
    let rust = Command::new(rust_bin).args(args).output().unwrap();
    let original = Command::new(original_bin).args(args).output().unwrap();
    assert_eq!(rust.status.code(), Some(0));
    assert_eq!(original.status.code(), Some(0));
    assert_eq!(rust.stdout, original.stdout);

    let rust_stderr = String::from_utf8(rust.stderr).unwrap();
    let original_stderr = String::from_utf8(original.stderr).unwrap();
    let warning = "failed to parse the FASTA/FASTQ record";
    assert!(!rust_stderr.contains(warning));
    assert!(!original_stderr.contains(warning));

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn cli_fastmap_raw_non_utf8_read_name_stdout_matches_original() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let index = format!("{manifest_dir}/minibwa/chrM-human");
    let dir = std::env::temp_dir().join(format!(
        "minibwa_rs_fastmap_raw_name_{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let reads = dir.join("raw.fa");
    std::fs::write(
        &reads,
        b">r\xff comment\nGATCACAGGTCTATCACCCTATTAACCACTCACGGGAGCTCTCCATGCAT\n",
    )
    .unwrap();
    let reads_s = reads.to_string_lossy().into_owned();

    let args = ["fastmap", "-l", "19", "-w", "2", &index, &reads_s];
    let rust = Command::new(rust_bin).args(args).output().unwrap();
    let original = Command::new(original_bin).args(args).output().unwrap();
    assert_eq!(rust.status.code(), Some(0));
    assert_eq!(original.status.code(), Some(0));
    assert_eq!(rust.stdout, original.stdout);

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn cli_fastmap_overlong_fastq_quality_stops_like_original() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let index = format!("{manifest_dir}/minibwa/chrM-human");
    let dir = std::env::temp_dir().join(format!(
        "minibwa_rs_fastmap_overlong_{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let reads = dir.join("overlong.fq");
    std::fs::write(
        &reads,
        b"@bad\nGATCACAGGTCTATCACCCTATTAACCACTCACGGGAGCTCTCCATGCAT\n+\nFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF\n",
    )
    .unwrap();
    let reads_s = reads.to_string_lossy().into_owned();

    let args = ["fastmap", "-l", "19", "-w", "2", &index, &reads_s];
    let rust = Command::new(rust_bin).args(args).output().unwrap();
    let original = Command::new(original_bin).args(args).output().unwrap();
    assert_eq!(rust.status.code(), Some(0));
    assert_eq!(original.status.code(), Some(0));
    assert_eq!(rust.stdout, original.stdout);
    assert!(rust.stdout.is_empty());

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn cli_fastmap_usage_errors_match_original() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");

    for args in [
        vec!["fastmap"],
        vec!["fastmap", "-l"],
        vec!["fastmap", "--unknown"],
    ] {
        let rust = Command::new(rust_bin).args(&args).output().unwrap();
        let original = Command::new(original_bin).args(&args).output().unwrap();
        assert_eq!(
            rust.status.code(),
            original.status.code(),
            "status for {args:?}"
        );
        assert_eq!(rust.stdout, original.stdout, "stdout for {args:?}");
        assert_eq!(rust.stderr, original.stderr, "stderr for {args:?}");
    }
}

#[test]
fn cli_fastmap_missing_inputs_match_original() {
    use std::os::unix::process::ExitStatusExt;

    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let index = format!("{manifest_dir}/minibwa/chrM-human");
    let dir =
        std::env::temp_dir().join(format!("minibwa_rs_fastmap_missing_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let missing_index = dir.join("missing-index");
    let missing_reads = dir.join("missing.fa");

    let args = [
        "fastmap".to_string(),
        missing_index.to_string_lossy().into_owned(),
        missing_reads.to_string_lossy().into_owned(),
    ];
    let rust = Command::new(rust_bin).args(&args).output().unwrap();
    let original = Command::new(original_bin).args(&args).output().unwrap();
    assert_eq!(rust.status.signal(), original.status.signal());
    assert_eq!(rust.status.signal(), Some(11));
    assert_eq!(rust.stdout, original.stdout);
    assert_eq!(rust.stderr, original.stderr);

    let args = [
        "fastmap".to_string(),
        index,
        missing_reads.to_string_lossy().into_owned(),
    ];
    let rust = Command::new(rust_bin).args(&args).output().unwrap();
    let original = Command::new(original_bin).args(&args).output().unwrap();
    assert_eq!(rust.status.code(), original.status.code());
    assert_eq!(rust.stdout, original.stdout);
    assert!(rust.stdout.is_empty());
    let rust_stderr = String::from_utf8(rust.stderr).unwrap();
    let original_stderr = String::from_utf8(original.stderr).unwrap();
    assert!(rust_stderr.starts_with("[M::main] Version: "));
    assert!(original_stderr.starts_with("[M::main] Version: "));
    assert!(rust_stderr.contains("[M::main] CMD:"));
    assert!(original_stderr.contains("[M::main] CMD:"));

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn cli_map_chain_only_stdout_matches_original_on_real_index_single_read() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let index = format!("{manifest_dir}/minibwa/chrM-human");
    let reads = format!("{manifest_dir}/minibwa/test/chrM-read_1.fa.gz");

    let args = [
        "map",
        "--chain-only",
        "-t",
        "1",
        "-K",
        "1k,1k",
        &index,
        &reads,
    ];
    let rust = Command::new(rust_bin).args(args).output().unwrap();
    let original = Command::new(original_bin).args(args).output().unwrap();
    assert_eq!(rust.status.code(), Some(0));
    assert_eq!(original.status.code(), Some(0));
    if rust.stdout != original.stdout {
        let first_diff = rust
            .stdout
            .iter()
            .zip(original.stdout.iter())
            .position(|(a, b)| a != b)
            .unwrap_or_else(|| rust.stdout.len().min(original.stdout.len()));
        panic!(
            "map stdout mismatch: rust_len={} original_len={} first_diff={}",
            rust.stdout.len(),
            original.stdout.len(),
            first_diff
        );
    }
}

#[test]
fn cli_map_stdin_stdout_matches_original() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let index = format!("{manifest_dir}/minibwa/chrM-human");
    let input = b">stdin_read\nACTCACCTGAGTTGTAAAAAACTCCAGTTGACACAAAATAGACTACGAAAGTGGCTTTAACATATCTGAACACACAATAGCTAAGACCCAAACTGGGATTAGATACCCCACTATGCTTAGCCCTAAACCTCAACAGTTAAATCAACAAAAC\n";

    let mut rust_child = Command::new(rust_bin)
        .args(["map", "--chain-only", "-t", "1", "-K", "1k,1k", &index, "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    rust_child.stdin.take().unwrap().write_all(input).unwrap();
    let rust = rust_child.wait_with_output().unwrap();

    let mut original_child = Command::new(original_bin)
        .args(["map", "--chain-only", "-t", "1", "-K", "1k,1k", &index, "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    original_child
        .stdin
        .take()
        .unwrap()
        .write_all(input)
        .unwrap();
    let original = original_child.wait_with_output().unwrap();

    assert_eq!(rust.status.code(), Some(0));
    assert_eq!(original.status.code(), Some(0));
    assert_eq!(rust.stdout, original.stdout);
}

#[test]
fn cli_map_debug_seed_anchor_stderr_matches_original_on_real_index_single_read() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let index = format!("{manifest_dir}/minibwa/chrM-human");
    let reads = format!("{manifest_dir}/minibwa/test/chrM-read_1.fa.gz");

    let args = [
        "map",
        "--dbg-qname",
        "--dbg-seed",
        "--dbg-anchor",
        "--chain-only",
        &index,
        &reads,
    ];
    let rust = Command::new(rust_bin).args(args).output().unwrap();
    let original = Command::new(original_bin).args(args).output().unwrap();
    assert_eq!(rust.status.code(), Some(0));
    assert_eq!(original.status.code(), Some(0));
    assert_eq!(rust.stdout, original.stdout);

    let rust_stderr = String::from_utf8(rust.stderr).unwrap();
    let original_stderr = String::from_utf8(original.stderr).unwrap();
    let rust_debug = rust_stderr
        .lines()
        .filter(|l| l.starts_with("QN\t") || l.starts_with("SD\t") || l.starts_with("AC\t"))
        .collect::<Vec<_>>();
    let original_debug = original_stderr
        .lines()
        .filter(|l| l.starts_with("QN\t") || l.starts_with("SD\t") || l.starts_with("AC\t"))
        .collect::<Vec<_>>();
    assert_eq!(rust_debug, original_debug);
    assert!(!rust_debug.is_empty());
}

#[test]
fn cli_map_debug_alignment_stderr_matches_original_on_real_index_reads() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let index = format!("{manifest_dir}/minibwa/chrM-human");
    let read1 = format!("{manifest_dir}/minibwa/test/chrM-read_1.fa.gz");
    let read2 = format!("{manifest_dir}/minibwa/test/chrM-read_2.fa.gz");

    let args = [
        "map",
        "--dbg-aln-seq",
        "-a",
        "-t",
        "1",
        "-K",
        "1k,1k",
        &index,
        &read1,
    ];
    let rust = Command::new(rust_bin).args(args).output().unwrap();
    let original = Command::new(original_bin).args(args).output().unwrap();
    assert_eq!(rust.status.code(), Some(0));
    assert_eq!(original.status.code(), Some(0));
    assert_eq!(rust.stdout, original.stdout);

    let rust_stderr = String::from_utf8(rust.stderr).unwrap();
    let original_stderr = String::from_utf8(original.stderr).unwrap();
    let rust_debug = rust_stderr
        .lines()
        .filter(|l| {
            l.starts_with("===>")
                || l.starts_with("cigar=")
                || !l.is_empty()
                    && l.bytes()
                        .all(|b| matches!(b, b'A' | b'C' | b'G' | b'T' | b'N'))
        })
        .collect::<Vec<_>>();
    let original_debug = original_stderr
        .lines()
        .filter(|l| {
            l.starts_with("===>")
                || l.starts_with("cigar=")
                || !l.is_empty()
                    && l.bytes()
                        .all(|b| matches!(b, b'A' | b'C' | b'G' | b'T' | b'N'))
        })
        .collect::<Vec<_>>();
    assert_eq!(rust_debug, original_debug);
    assert!(!rust_debug.is_empty());

    let args = [
        "map",
        "--dbg-aln-pe",
        "-a",
        "--pe-predef",
        "-t",
        "1",
        "-K",
        "1k,1k",
        &index,
        &read1,
        &read2,
    ];
    let rust = Command::new(rust_bin).args(args).output().unwrap();
    let original = Command::new(original_bin).args(args).output().unwrap();
    assert_eq!(rust.status.code(), Some(0));
    assert_eq!(original.status.code(), Some(0));
    assert_eq!(rust.stdout, original.stdout);

    let rust_stderr = String::from_utf8(rust.stderr).unwrap();
    let original_stderr = String::from_utf8(original.stderr).unwrap();
    let rust_debug = rust_stderr
        .lines()
        .filter(|l| {
            l.starts_with("===>")
                || l.starts_with("max=")
                || !l.is_empty()
                    && l.bytes()
                        .all(|b| matches!(b, b'A' | b'C' | b'G' | b'T' | b'N'))
        })
        .collect::<Vec<_>>();
    let original_debug = original_stderr
        .lines()
        .filter(|l| {
            l.starts_with("===>")
                || l.starts_with("max=")
                || !l.is_empty()
                    && l.bytes()
                        .all(|b| matches!(b, b'A' | b'C' | b'G' | b'T' | b'N'))
        })
        .collect::<Vec<_>>();
    assert_eq!(rust_debug, original_debug);
    assert!(!rust_debug.is_empty());
}

#[test]
fn cli_map_debug_anchor_positions_stderr_matches_original_on_real_index_single_read() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let index = format!("{manifest_dir}/minibwa/chrM-human");
    let reads = format!("{manifest_dir}/minibwa/test/chrM-read_1.fa.gz");

    let args = [
        "map",
        "--dbg-qname",
        "--dbg-an-pos",
        "-t",
        "1",
        "-K",
        "1k,1k",
        &index,
        &reads,
    ];
    let rust = Command::new(rust_bin).args(args).output().unwrap();
    let original = Command::new(original_bin).args(args).output().unwrap();
    assert_eq!(rust.status.code(), Some(0));
    assert_eq!(original.status.code(), Some(0));
    assert_eq!(rust.stdout, original.stdout);

    let rust_stderr = String::from_utf8(rust.stderr).unwrap();
    let original_stderr = String::from_utf8(original.stderr).unwrap();
    let rust_debug = rust_stderr
        .lines()
        .filter(|l| l.starts_with("QN\t") || l.starts_with("AF\t") || l.starts_with("AD\t"))
        .collect::<Vec<_>>();
    let original_debug = original_stderr
        .lines()
        .filter(|l| l.starts_with("QN\t") || l.starts_with("AF\t") || l.starts_with("AD\t"))
        .collect::<Vec<_>>();
    assert_eq!(rust_debug, original_debug);
    assert!(rust_debug.iter().any(|l| l.starts_with("AF\t")));
    assert!(rust_debug.iter().any(|l| l.starts_with("AD\t")));
}

#[test]
fn cli_map_warns_like_original_for_unbalanced_paired_inputs() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let index = format!("{manifest_dir}/minibwa/chrM-human");
    let dir = std::env::temp_dir().join(format!(
        "minibwa_rs_unbalanced_pairs_{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let r1 = dir.join("r1.fa");
    let r2 = dir.join("r2.fa");
    std::fs::write(
        &r1,
        b">A00744:46:HV3C3DSXX:2:2452:16161:4210\nACTCACCTGAGTTGTAAAAAACTCCAGTTGACACAAAATAGACTACGAAAGTGGCTTTAACATATCTGAACACACAATAGCTAAGACCCAAACTGGGATTAGATACCCCACTATGCTTAGCCCTAAACCTCAACAGTTAAATCAACAAAAC\n>A00744:46:HV3C3DSXX:2:1614:12373:19727\nATCTGACAACAGAGGCTTACGACCCCTTATTTACCGAGAAAGCTCACAAGAACTGCTAACTCATGCCCCCATGTCTAACAACATGGCTTTCTCAACTTTTAAAGGATAACAGCTATCCATTGGTCTTAGGCCCCAAAAATTTTGGTGCAAC\n",
    )
    .unwrap();
    std::fs::write(
        &r2,
        b">A00744:46:HV3C3DSXX:2:2452:16161:4210\nACCTCATGGGCTACACCTTGACCTAACGTCTTTACGTGGGTACTTGCGCTTACTTTGTAGCCTTCATCAGGGTTTGCTGAAGATGGCGGTATATAGGCTGAGCAAGAGGTGGTGAGGTTGATCGGGGTTTATCGATTACAGAACAGGCTCC\n",
    )
    .unwrap();

    let args = [
        "map",
        "--chain-only",
        "-t",
        "1",
        "-K",
        "1k,1k",
        &index,
        &r1.to_string_lossy(),
        &r2.to_string_lossy(),
    ];
    let rust = Command::new(rust_bin).args(args).output().unwrap();
    let original = Command::new(original_bin).args(args).output().unwrap();
    let warning =
        "[W::mb_bseq_read_frag]\u{1b}[1;31m query files have different number of records; extra records skipped.\u{1b}[0m";
    let rust_stderr = String::from_utf8(rust.stderr).unwrap();
    let original_stderr = String::from_utf8(original.stderr).unwrap();
    assert!(!rust.status.success());
    assert!(!original.status.success());
    assert_eq!(rust_stderr.matches(warning).count(), 1);
    assert!(original_stderr.contains(warning));
}

#[test]
fn cli_map_warns_like_original_for_empty_sequence_name() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let index = format!("{manifest_dir}/minibwa/chrM-human");
    let dir = std::env::temp_dir().join(format!("minibwa_rs_empty_name_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let reads = dir.join("empty-name.fa");
    std::fs::write(
        &reads,
        b">\nACTCACCTGAGTTGTAAAAAACTCCAGTTGACACAAAATAGACTACGAAAGTGGCTTTAACATATCTGAACACACAATAGCTAAGACCCAAACTGGGATTAGATACCCCACTATGCTTAGCCCTAAACCTCAACAGTTAAATCAACAAAAC\n",
    )
    .unwrap();

    let args = [
        "map",
        "--chain-only",
        "-t",
        "1",
        "-K",
        "1k,1k",
        &index,
        &reads.to_string_lossy(),
    ];
    let rust = Command::new(rust_bin).args(args).output().unwrap();
    let original = Command::new(original_bin).args(args).output().unwrap();
    assert_eq!(rust.status.code(), Some(0));
    assert_eq!(original.status.code(), Some(0));
    assert_eq!(rust.stdout, original.stdout);

    let warning = "[WARNING]\u{1b}[1;31m empty sequence name in the input.\u{1b}[0m";
    let rust_stderr = String::from_utf8(rust.stderr).unwrap();
    let original_stderr = String::from_utf8(original.stderr).unwrap();
    assert!(rust_stderr.contains(warning));
    assert!(original_stderr.contains(warning));
}

#[test]
fn cli_map_warns_like_original_for_truncated_fastq_quality() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let index = format!("{manifest_dir}/minibwa/chrM-human");
    let dir =
        std::env::temp_dir().join(format!("minibwa_rs_truncated_fastq_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let first = dir.join("first.fq");
    let next = dir.join("next.fq");
    let overlong = dir.join("overlong.fq");
    std::fs::write(&first, b"@bad\nACGTACGT\n+\nFFFF\n").unwrap();
    std::fs::write(&next, b"@good\nACGT\n+\nFFFF\n@bad\nACGTACGT\n+\nFFFF\n").unwrap();
    std::fs::write(&overlong, b"@bad\nACGT\n+\nFFFFFF\n").unwrap();

    for (reads, chunk_size, warning) in [
        (
            &first,
            "1k,1k",
            "[WARNING]\u{1b}[1;31m failed to parse the first FASTA/FASTQ record. Continue anyway.\u{1b}[0m",
        ),
        (
            &next,
            "1k,1k",
            "[WARNING]\u{1b}[1;31m failed to parse the FASTA/FASTQ record next to 'good'. Continue anyway.\u{1b}[0m",
        ),
        (
            &overlong,
            "1k,1k",
            "[WARNING]\u{1b}[1;31m failed to parse the first FASTA/FASTQ record. Continue anyway.\u{1b}[0m",
        ),
    ] {
        let reads_s = reads.to_string_lossy().into_owned();
        let args = [
            "map",
            "--chain-only",
            "-t",
            "1",
            "-K",
            chunk_size,
            &index,
            &reads_s,
        ];
        let rust = Command::new(rust_bin).args(args).output().unwrap();
        let original = Command::new(original_bin).args(args).output().unwrap();
        assert_eq!(rust.status.code(), Some(0));
        assert_eq!(original.status.code(), Some(0));
        assert_eq!(rust.stdout, original.stdout);

        let rust_stderr = String::from_utf8(rust.stderr).unwrap();
        let original_stderr = String::from_utf8(original.stderr).unwrap();
        assert!(rust_stderr.contains(warning));
        assert!(original_stderr.contains(warning));
    }

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn cli_map_full_alignment_stdout_matches_original_on_real_index_single_read() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let index = format!("{manifest_dir}/minibwa/chrM-human");
    let reads = format!("{manifest_dir}/minibwa/test/chrM-read_1.fa.gz");

    let args = ["map", "-t", "1", "-K", "1k,1k", &index, &reads];
    let rust = Command::new(rust_bin).args(args).output().unwrap();
    let original = Command::new(original_bin).args(args).output().unwrap();
    assert_eq!(rust.status.code(), Some(0));
    assert_eq!(original.status.code(), Some(0));
    assert_eq!(rust.stdout, original.stdout);
}

#[test]
fn cli_mem_alias_stdout_matches_original_on_real_index_single_read() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let index = format!("{manifest_dir}/minibwa/chrM-human");
    let reads = format!("{manifest_dir}/minibwa/test/chrM-read_1.fa.gz");

    for extra in [Vec::<&str>::new(), vec!["-a"]] {
        let mut args = vec!["mem"];
        args.extend(extra);
        args.extend(["-t", "1", "-K", "1k,1k", &index, &reads]);
        let rust = Command::new(rust_bin).args(&args).output().unwrap();
        let original = Command::new(original_bin).args(&args).output().unwrap();
        assert_eq!(rust.status.code(), Some(0), "rust status for {args:?}");
        assert_eq!(
            original.status.code(),
            Some(0),
            "original status for {args:?}"
        );
        assert_eq!(rust.stdout, original.stdout, "stdout for {args:?}");
    }
}

#[test]
fn cli_mem_alias_paired_stdout_matches_original_on_real_index_reads() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let index = format!("{manifest_dir}/minibwa/chrM-human");
    let read1 = format!("{manifest_dir}/minibwa/test/chrM-read_1.fa.gz");
    let read2 = format!("{manifest_dir}/minibwa/test/chrM-read_2.fa.gz");

    for extra in [Vec::<&str>::new(), vec!["-a"]] {
        let mut args = vec!["mem"];
        args.extend(extra);
        args.extend(["-t", "1", "-K", "1k,1k", &index, &read1, &read2]);
        let rust = Command::new(rust_bin).args(&args).output().unwrap();
        let original = Command::new(original_bin).args(&args).output().unwrap();
        assert_eq!(rust.status.code(), Some(0), "rust status for {args:?}");
        assert_eq!(
            original.status.code(),
            Some(0),
            "original status for {args:?}"
        );
        assert_eq!(rust.stdout, original.stdout, "stdout for {args:?}");
    }
}

#[test]
fn cli_map_output_file_matches_original_on_real_index_single_read() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let index = format!("{manifest_dir}/minibwa/chrM-human");
    let reads = format!("{manifest_dir}/minibwa/test/chrM-read_1.fa.gz");
    let dir = std::env::temp_dir().join(format!("minibwa_rs_cli_map_o_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();

    for (mode, extra) in [("paf", Vec::<&str>::new()), ("sam", vec!["-a"])] {
        let out_path = dir.join(format!("shared.{mode}"));
        let mut rust_args = vec!["map".to_string()];
        rust_args.extend(extra.iter().map(|s| s.to_string()));
        rust_args.extend([
            "-t".to_string(),
            "1".to_string(),
            "-K".to_string(),
            "1k,1k".to_string(),
            "-o".to_string(),
            out_path.to_string_lossy().into_owned(),
            index.clone(),
            reads.clone(),
        ]);
        let mut original_args = vec!["map".to_string()];
        original_args.extend(extra.iter().map(|s| s.to_string()));
        original_args.extend([
            "-t".to_string(),
            "1".to_string(),
            "-K".to_string(),
            "1k,1k".to_string(),
            "-o".to_string(),
            out_path.to_string_lossy().into_owned(),
            index.clone(),
            reads.clone(),
        ]);
        let rust = Command::new(rust_bin).args(&rust_args).output().unwrap();
        let rust_file = std::fs::read(&out_path).unwrap();
        let original = Command::new(original_bin)
            .args(&original_args)
            .output()
            .unwrap();
        let original_file = std::fs::read(&out_path).unwrap();
        assert_eq!(rust.status.code(), Some(0));
        assert_eq!(original.status.code(), Some(0));
        assert_eq!(rust.stdout, original.stdout);
        assert_eq!(rust_file, original_file);
    }

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn cli_map_output_file_open_failure_matches_original() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let index = format!("{manifest_dir}/minibwa/chrM-human");
    let dir =
        std::env::temp_dir().join(format!("minibwa_rs_cli_map_o_fail_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let reads = dir.join("read.fa");
    std::fs::write(
        &reads,
        b">r\nACTCACCTGAGTTGTAAAAAACTCCAGTTGACACAAAATAGACTACGAAAGTGGCTTTAACATATCTGAACACACAATAGCTAAGACCCAAACTGGGATTAGATACCCCACTATGCTTAGCCCTAAACCTCAACAGTTAAATCAACAAAAC\n",
    )
    .unwrap();

    for extra in [Vec::<&str>::new(), vec!["-a"]] {
        let out = dir.join("missing-dir").join("out");
        let out_s = out.to_string_lossy().into_owned();
        let reads_s = reads.to_string_lossy().into_owned();
        let mut args = vec!["map"];
        args.extend(extra);
        args.extend(["-t", "1", "-K", "1k,1k", "-o", &out_s, &index, &reads_s]);
        let rust = Command::new(rust_bin).args(&args).output().unwrap();
        let original = Command::new(original_bin).args(&args).output().unwrap();
        assert_eq!(rust.status.code(), Some(0), "rust status for {args:?}");
        assert_eq!(
            original.status.code(),
            Some(0),
            "original status for {args:?}"
        );
        assert_eq!(rust.stdout, original.stdout, "stdout for {args:?}");
    }

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn cli_map_paired_sam_stdout_matches_original_on_real_index_reads() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let index = format!("{manifest_dir}/minibwa/chrM-human");
    let read1 = format!("{manifest_dir}/minibwa/test/chrM-read_1.fa.gz");
    let read2 = format!("{manifest_dir}/minibwa/test/chrM-read_2.fa.gz");

    let args = [
        "map", "-a", "-t", "1", "-K", "1k,1k", &index, &read1, &read2,
    ];
    let rust = Command::new(rust_bin).args(args).output().unwrap();
    let original = Command::new(original_bin).args(args).output().unwrap();
    assert_eq!(rust.status.code(), Some(0));
    assert_eq!(original.status.code(), Some(0));
    assert_eq!(rust.stdout, original.stdout);
}

#[test]
fn cli_map_paired_names_stop_at_nul_like_original() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let index = format!("{manifest_dir}/minibwa/chrM-human");
    let dir = std::env::temp_dir().join(format!("minibwa_rs_pair_nul_name_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let r1 = dir.join("r1.fa");
    let r2 = dir.join("r2.fa");
    std::fs::write(
        &r1,
        b">pair\0left/1\nACTCACCTGAGTTGTAAAAAACTCCAGTTGACACAAAATAGACTACGAAAGTGGCTTTAACATATCTGAACACACAATAGCTAAGACCCAAACTGGGATTAGATACCCCACTATGCTTAGCCCTAAACCTCAACAGTTAAATCAACAAAAC\n",
    )
    .unwrap();
    std::fs::write(
        &r2,
        b">pair\0right/2\nACCTCATGGGCTACACCTTGACCTAACGTCTTTACGTGGGTACTTGCGCTTACTTTGTAGCCTTCATCAGGGTTTGCTGAAGATGGCGGTATATAGGCTGAGCAAGAGGTGGTGAGGTTGATCGGGGTTTATCGATTACAGAACAGGCTCC\n",
    )
    .unwrap();
    let r1_s = r1.to_string_lossy().into_owned();
    let r2_s = r2.to_string_lossy().into_owned();

    let args = ["map", "-a", "-t", "1", "-K", "1k,1k", &index, &r1_s, &r2_s];
    let rust = Command::new(rust_bin).args(args).output().unwrap();
    let original = Command::new(original_bin).args(args).output().unwrap();
    assert_eq!(rust.status.code(), Some(0));
    assert_eq!(original.status.code(), Some(0));
    assert_eq!(rust.stdout, original.stdout);

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn cli_map_sam_tag_modes_match_original_on_real_index_single_read() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let index = format!("{manifest_dir}/minibwa/chrM-human");
    let reads = format!("{manifest_dir}/minibwa/test/chrM-read_1.fa.gz");

    for extra in [
        vec!["-b", "cs"],
        vec!["-b", "md"],
        vec!["-b", "MD"],
        vec!["-b", "ds"],
        vec!["--eqx"],
    ] {
        let mut args = vec!["map", "-a"];
        args.extend(extra);
        args.extend(["-t", "1", "-K", "1k,1k", &index, &reads]);
        let rust = Command::new(rust_bin).args(&args).output().unwrap();
        let original = Command::new(original_bin).args(&args).output().unwrap();
        assert_eq!(rust.status.code(), Some(0));
        assert_eq!(original.status.code(), Some(0));
        assert_eq!(rust.stdout, original.stdout);
    }
}

#[test]
fn cli_map_read_group_output_matches_original_on_real_index_single_read() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let index = format!("{manifest_dir}/minibwa/chrM-human");
    let reads = format!("{manifest_dir}/minibwa/test/chrM-read_1.fa.gz");

    let args = [
        "map",
        "-a",
        "-R",
        "@RG\\tID:grp1\\tSM:sample",
        "-t",
        "1",
        "-K",
        "1k,1k",
        &index,
        &reads,
    ];
    let rust = Command::new(rust_bin).args(args).output().unwrap();
    let original = Command::new(original_bin).args(args).output().unwrap();
    assert_eq!(rust.status.code(), Some(0));
    assert_eq!(original.status.code(), Some(0));
    assert_eq!(rust.stdout, original.stdout);

    let stdout = String::from_utf8(rust.stdout).unwrap();
    assert!(stdout.contains("@RG\tID:grp1\tSM:sample\n"));
    assert!(stdout.contains("\tRG:Z:grp1\t"));
}

#[test]
fn cli_map_read_group_diagnostics_match_original() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let index = format!("{manifest_dir}/minibwa/chrM-human");
    let reads = format!("{manifest_dir}/minibwa/test/chrM-read_1.fa.gz");
    let long_id = format!("@RG\\tID:{}", "a".repeat(256));

    for (rg, diagnostic) in [
        ("XX\\tID:foo".to_string(), "[ERROR] the read group line is not started with @RG"),
        (
            "@RG\tID:foo".to_string(),
            "[ERROR] the read group line contained literal <tab> characters -- replace with escaped tabs: \\t",
        ),
        ("@RG\\tSM:bar".to_string(), "[ERROR] no ID within the read group line"),
        (long_id, "[ERROR] @RG:ID is longer than 255 characters"),
    ] {
        let args = ["map", "-a", "-R", &rg, "-t", "1", "-K", "1k,1k", &index, &reads];
        let rust = Command::new(rust_bin).args(args).output().unwrap();
        let original = Command::new(original_bin).args(args).output().unwrap();
        assert_eq!(rust.status.code(), Some(0), "rust status for {rg:?}");
        assert_eq!(
            original.status.code(),
            Some(0),
            "original status for {rg:?}"
        );
        assert_eq!(rust.stdout, original.stdout, "stdout for {rg:?}");

        let rust_stderr = String::from_utf8(rust.stderr).unwrap();
        let original_stderr = String::from_utf8(original.stderr).unwrap();
        assert!(rust_stderr.contains(diagnostic), "rust stderr for {rg:?}");
        assert!(
            original_stderr.contains(diagnostic),
            "original stderr for {rg:?}"
        );
    }
}

#[test]
fn cli_map_copy_comment_matches_original_for_fasta_comment() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let index = format!("{manifest_dir}/minibwa/chrM-human");
    let dir = std::env::temp_dir().join(format!("minibwa_rs_copy_comment_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let reads = dir.join("comment.fa");
    std::fs::write(
        &reads,
        b">commented sample=chrM lane=1\nACTCACCTGAGTTGTAAAAAACTCCAGTTGACACAAAATAGACTACGAAAGTGGCTTTAACATATCTGAACACACAATAGCTAAGACCCAAACTGGGATTAGATACCCCACTATGCTTAGCCCTAAACCTCAACAGTTAAATCAACAAAAC\n",
    )
    .unwrap();
    let reads_s = reads.to_string_lossy().into_owned();

    for extra in [Vec::<&str>::new(), vec!["-a"]] {
        let mut args = vec!["map", "-y"];
        args.extend(extra);
        args.extend(["-t", "1", "-K", "1k,1k", &index, &reads_s]);
        let rust = Command::new(rust_bin).args(&args).output().unwrap();
        let original = Command::new(original_bin).args(&args).output().unwrap();
        assert_eq!(rust.status.code(), Some(0));
        assert_eq!(original.status.code(), Some(0));
        assert_eq!(rust.stdout, original.stdout);
    }

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn cli_map_copy_comment_embedded_nul_truncates_like_original() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let index = format!("{manifest_dir}/minibwa/chrM-human");
    let dir = std::env::temp_dir().join(format!(
        "minibwa_rs_copy_comment_nul_{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let reads = dir.join("nul-comment.fa");
    std::fs::write(
        &reads,
        b">nulled cc:Z:ok\0hidden\nACTCACCTGAGTTGTAAAAAACTCCAGTTGACACAAAATAGACTACGAAAGTGGCTTTAACATATCTGAACACACAATAGCTAAGACCCAAACTGGGATTAGATACCCCACTATGCTTAGCCCTAAACCTCAACAGTTAAATCAACAAAAC\n",
    )
    .unwrap();
    let reads_s = reads.to_string_lossy().into_owned();

    for extra in [Vec::<&str>::new(), vec!["-a"]] {
        let mut args = vec!["map", "-y"];
        args.extend(extra);
        args.extend(["-t", "1", "-K", "1k,1k", &index, &reads_s]);
        let rust = Command::new(rust_bin).args(&args).output().unwrap();
        let original = Command::new(original_bin).args(&args).output().unwrap();
        assert_eq!(rust.status.code(), Some(0));
        assert_eq!(original.status.code(), Some(0));
        assert_eq!(rust.stdout, original.stdout);
        assert!(!rust.stdout.windows(b"hidden".len()).any(|w| w == b"hidden"));
    }

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn cli_map_converts_u_to_t_like_original() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let index = format!("{manifest_dir}/minibwa/chrM-human");
    let dir = std::env::temp_dir().join(format!("minibwa_rs_u_to_t_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let reads = dir.join("u.fa");
    std::fs::write(
        &reads,
        b">rna_style\nACUCACCUCAGUUGUAAAAAACUCCAGUUGACACAAAAUAGACUACGAAAGUGGCUUUAACAUAUCUGAACACACAAUAGCUAAGACCCAAACUGGGAUUAGAUACCCCACUAUGCUUAGCCCUAAACCUCAACAGUUAAAUCAACAAAAC\n",
    )
    .unwrap();
    let reads_s = reads.to_string_lossy().into_owned();

    for extra in [Vec::<&str>::new(), vec!["-a"], vec!["--chain-only"]] {
        let mut args = vec!["map"];
        args.extend(extra);
        args.extend(["-t", "1", "-K", "1k,1k", &index, &reads_s]);
        let rust = Command::new(rust_bin).args(&args).output().unwrap();
        let original = Command::new(original_bin).args(&args).output().unwrap();
        assert_eq!(rust.status.code(), Some(0), "rust status for {args:?}");
        assert_eq!(
            original.status.code(),
            Some(0),
            "original status for {args:?}"
        );
        assert_eq!(rust.stdout, original.stdout, "stdout for {args:?}");
    }

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn cli_map_fastq_quality_output_matches_original() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let index = format!("{manifest_dir}/minibwa/chrM-human");
    let dir = std::env::temp_dir().join(format!("minibwa_rs_fastq_qual_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let reads = dir.join("reads.fq");
    let seq = "ACTCACCTGAGTTGTAAAAAACTCCAGTTGACACAAAATAGACTACGAAAGTGGCTTTAACATATCTGAACACACAATAGCTAAGACCCAAACTGGGATTAGATACCCCACTATGCTTAGCCCTAAACCTCAACAGTTAAATCAACAAAAC";
    let alphabet =
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!#$%&()*+,-./:;<=>?@[]^_`{|}~";
    let qual = alphabet.repeat(seq.len() / alphabet.len() + 1);
    let qual = &qual[..seq.len()];
    std::fs::write(
        &reads,
        format!("@fq_read comment=qual\n{seq}\n+\n{qual}\n").as_bytes(),
    )
    .unwrap();
    let reads_s = reads.to_string_lossy().into_owned();

    for extra in [Vec::<&str>::new(), vec!["-a"], vec!["-a", "-y"]] {
        let mut args = vec!["map"];
        args.extend(extra);
        args.extend(["-t", "1", "-K", "1k,1k", &index, &reads_s]);
        let rust = Command::new(rust_bin).args(&args).output().unwrap();
        let original = Command::new(original_bin).args(&args).output().unwrap();
        assert_eq!(rust.status.code(), Some(0), "rust status for {args:?}");
        assert_eq!(
            original.status.code(),
            Some(0),
            "original status for {args:?}"
        );
        assert_eq!(rust.stdout, original.stdout, "stdout for {args:?}");
    }

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn cli_map_wrapped_fasta_fastq_records_match_original() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let index = format!("{manifest_dir}/minibwa/chrM-human");
    let dir = std::env::temp_dir().join(format!("minibwa_rs_wrapped_reads_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let seq = "ACTCACCTGAGTTGTAAAAAACTCCAGTTGACACAAAATAGACTACGAAAGTGGCTTTAACATATCTGAACACACAATAGCTAAGACCCAAACTGGGATTAGATACCCCACTATGCTTAGCCCTAAACCTCAACAGTTAAATCAACAAAAC";
    let (seq_a, seq_b) = seq.split_at(73);

    let fasta = dir.join("wrapped.fa");
    std::fs::write(
        &fasta,
        format!(">wrapped_fa comment=fasta\n{seq_a}\n{seq_b}\n").as_bytes(),
    )
    .unwrap();

    let fastq = dir.join("wrapped.fq");
    let alphabet =
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!#$%&()*+,-./:;<=>?@[]^_`{|}~";
    let qual = alphabet.repeat(seq.len() / alphabet.len() + 1);
    let qual = &qual[..seq.len()];
    let (qual_a, qual_b) = qual.split_at(73);
    std::fs::write(
        &fastq,
        format!("@wrapped_fq comment=fastq\n{seq_a}\n{seq_b}\n+\n{qual_a}\n{qual_b}\n").as_bytes(),
    )
    .unwrap();

    for (reads, extra) in [
        (fasta.to_string_lossy().into_owned(), Vec::<&str>::new()),
        (fasta.to_string_lossy().into_owned(), vec!["-a", "-y"]),
        (fastq.to_string_lossy().into_owned(), Vec::<&str>::new()),
        (fastq.to_string_lossy().into_owned(), vec!["-a", "-y"]),
    ] {
        let mut args = vec!["map"];
        args.extend(extra);
        args.extend(["-t", "1", "-K", "1k,1k", &index, &reads]);
        let rust = Command::new(rust_bin).args(&args).output().unwrap();
        let original = Command::new(original_bin).args(&args).output().unwrap();
        assert_eq!(rust.status.code(), Some(0), "rust status for {args:?}");
        assert_eq!(
            original.status.code(),
            Some(0),
            "original status for {args:?}"
        );
        assert_eq!(rust.stdout, original.stdout, "stdout for {args:?}");
    }

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn cli_map_unmapped_output_matches_original() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let index = format!("{manifest_dir}/minibwa/chrM-human");
    let dir = std::env::temp_dir().join(format!("minibwa_rs_unmapped_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let reads = dir.join("nohit.fa");
    std::fs::write(
        &reads,
        b">nohit\nNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNN\n",
    )
    .unwrap();
    let reads_s = reads.to_string_lossy().into_owned();

    for extra in [vec!["--chain-only"], vec!["--chain-only", "-u"], vec!["-a"]] {
        let mut args = vec!["map"];
        args.extend(extra);
        args.extend(["-t", "1", "-K", "1k,1k", &index, &reads_s]);
        let rust = Command::new(rust_bin).args(&args).output().unwrap();
        let original = Command::new(original_bin).args(&args).output().unwrap();
        assert_eq!(rust.status.code(), Some(0), "rust status for {args:?}");
        assert_eq!(
            original.status.code(),
            Some(0),
            "original status for {args:?}"
        );
        assert_eq!(rust.stdout, original.stdout, "stdout for {args:?}");
    }

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn cli_map_option_diagnostics_match_original() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let index = format!("{manifest_dir}/minibwa/chrM-human");
    let reads = format!("{manifest_dir}/minibwa/test/chrM-read_1.fa.gz");

    for args in [
        vec!["map", "-x", "unknown"],
        vec!["map", "-k"],
        vec!["map", "--definitely-not-an-option"],
        vec!["map", "--kalloc=maybe"],
        vec!["map", "--pe=maybe"],
        vec!["map", "--long=maybe"],
        vec!["map", "--adap=maybe"],
    ] {
        let rust = Command::new(rust_bin).args(&args).output().unwrap();
        let original = Command::new(original_bin).args(&args).output().unwrap();
        assert_eq!(rust.status.code(), Some(0));
        assert_eq!(original.status.code(), Some(0));
        assert_eq!(rust.stdout, original.stdout);
        assert_eq!(rust.stderr, original.stderr);
    }

    let args = [
        "map", "-a", "-b", "bad", "-t", "1", "-K", "1k,1k", &index, &reads,
    ];
    let rust = Command::new(rust_bin).args(args).output().unwrap();
    let original = Command::new(original_bin).args(args).output().unwrap();
    assert_eq!(rust.status.code(), Some(0));
    assert_eq!(original.status.code(), Some(0));
    assert_eq!(rust.stdout, original.stdout);
    let rust_stderr = String::from_utf8(rust.stderr).unwrap();
    let original_stderr = String::from_utf8(original.stderr).unwrap();
    let warning = "-b only takes 'cs', 'ds' or 'MD'. Invalid values are assumed to be 'cs'.";
    assert!(rust_stderr.contains(warning));
    assert!(original_stderr.contains(warning));
}

#[test]
fn cli_map_missing_query_file_diagnostic_matches_original() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let index = format!("{manifest_dir}/minibwa/chrM-human");
    let missing = std::env::temp_dir().join(format!(
        "minibwa_rs_missing_query_{}.fa",
        std::process::id()
    ));
    let missing = missing.to_string_lossy().into_owned();

    let args = ["map", "--chain-only", &index, &missing];
    let rust = Command::new(rust_bin).args(args).output().unwrap();
    let original = Command::new(original_bin).args(args).output().unwrap();
    assert_eq!(rust.status.code(), Some(0));
    assert_eq!(original.status.code(), Some(0));
    assert_eq!(rust.stdout, original.stdout);

    let expected = format!("ERROR: failed to open file '{missing}': No such file or directory");
    let rust_stderr = String::from_utf8(rust.stderr).unwrap();
    let original_stderr = String::from_utf8(original.stderr).unwrap();
    assert!(rust_stderr.contains(&expected));
    assert!(original_stderr.contains(&expected));
    assert!(rust_stderr.contains("[M::main] Version: 0.0-r352-dirty\n"));
    assert!(original_stderr.contains("[M::main] Version: 0.0-r352-dirty\n"));
    assert!(!rust_stderr.contains("os error"));
    assert!(!original_stderr.contains("os error"));
}

#[cfg(unix)]
#[test]
fn cli_map_bare_kalloc_fatal_before_index_load_like_original() {
    use std::os::unix::process::ExitStatusExt;

    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let index = format!("{manifest_dir}/minibwa/chrM-human");
    let reads = format!("{manifest_dir}/minibwa/test/chrM-read_1.fa.gz");

    let args = ["map", "--chain-only", "--kalloc", &index, &reads];
    let rust = Command::new(rust_bin).args(args).output().unwrap();
    let original = Command::new(original_bin).args(args).output().unwrap();

    assert_eq!(rust.status.signal(), original.status.signal());
    assert_eq!(rust.status.signal(), Some(11));
    assert_eq!(rust.stdout, original.stdout);
    assert_eq!(rust.stderr, original.stderr);
}

#[cfg(unix)]
#[test]
fn cli_map_nonpositive_threads_fatal_after_index_load_like_original() {
    use std::os::unix::process::ExitStatusExt;

    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let index = format!("{manifest_dir}/minibwa/chrM-human");
    let reads = format!("{manifest_dir}/minibwa/test/chrM-read_1.fa.gz");

    for threads in ["0", "-1"] {
        let args = ["map", "--chain-only", "-t", threads, &index, &reads];
        let rust = Command::new(rust_bin).args(args).output().unwrap();
        let original = Command::new(original_bin).args(args).output().unwrap();

        assert_eq!(rust.status.signal(), original.status.signal());
        assert_eq!(rust.status.signal(), Some(11));
        assert_eq!(rust.stdout, original.stdout);

        let rust_stderr = String::from_utf8(rust.stderr).unwrap();
        let original_stderr = String::from_utf8(original.stderr).unwrap();
        assert!(rust_stderr.starts_with("[M::main_map::"));
        assert!(original_stderr.starts_with("[M::main_map::"));
        assert!(rust_stderr.contains("] index loaded\n"));
        assert!(original_stderr.contains("] index loaded\n"));
    }
}

#[test]
fn cli_map_output_open_failure_footer_matches_original() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let index = format!("{manifest_dir}/minibwa/chrM-human");
    let reads = format!("{manifest_dir}/minibwa/test/chrM-read_1.fa.gz");
    let out = std::env::temp_dir()
        .join(format!("minibwa_rs_missing_out_dir_{}", std::process::id()))
        .join("out.paf");
    let out = out.to_string_lossy().into_owned();

    let args = ["map", "--chain-only", "-o", &out, &index, &reads];
    let rust = Command::new(rust_bin).args(args).output().unwrap();
    let original = Command::new(original_bin).args(args).output().unwrap();
    assert_eq!(rust.status.code(), Some(0));
    assert_eq!(original.status.code(), Some(0));
    assert_eq!(rust.stdout, original.stdout);

    let rust_stderr = String::from_utf8(rust.stderr).unwrap();
    let original_stderr = String::from_utf8(original.stderr).unwrap();
    assert!(rust_stderr.contains("[M::main] Version: 0.0-r352-dirty\n"));
    assert!(original_stderr.contains("[M::main] Version: 0.0-r352-dirty\n"));
}

#[test]
fn cli_map_long_preset_matches_original_on_real_index_single_read() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let index = format!("{manifest_dir}/minibwa/chrM-human");
    let reads = format!("{manifest_dir}/minibwa/test/chrM-read_1.fa.gz");

    for extra in [
        vec!["--long"],
        vec!["--long", "-a"],
        vec!["--long", "--chain-only"],
    ] {
        let mut args = vec!["map"];
        args.extend(extra);
        args.extend(["-t", "1", "-K", "1k,1k", &index, &reads]);
        let rust = Command::new(rust_bin).args(&args).output().unwrap();
        let original = Command::new(original_bin).args(&args).output().unwrap();
        assert_eq!(rust.status.code(), Some(0));
        assert_eq!(original.status.code(), Some(0));
        assert_eq!(rust.stdout, original.stdout);
    }
}

#[test]
fn cli_map_scoring_and_chaining_option_matrix_stdout_matches_original() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let index = format!("{manifest_dir}/minibwa/chrM-human");
    let reads = format!("{manifest_dir}/minibwa/test/chrM-read_1.fa.gz");

    for extra in [
        vec!["-A", "2", "-B", "6", "-O", "3,9", "-E", "1,2", "-a"],
        vec!["-k", "19", "-c", "500", "-g", "5k", "-w", "50", "-W", "200"],
        vec!["-m", "15", "-p", "0.75", "-N", "2"],
        vec!["-s", "10", "-a"],
        vec!["--eqx", "-a"],
    ] {
        let mut args = vec!["map"];
        args.extend(extra);
        args.extend(["-t", "1", "-K", "1k,1k", &index, &reads]);
        let rust = Command::new(rust_bin).args(&args).output().unwrap();
        let original = Command::new(original_bin).args(&args).output().unwrap();
        assert_eq!(rust.status.code(), Some(0), "rust status for {args:?}");
        assert_eq!(
            original.status.code(),
            Some(0),
            "original status for {args:?}"
        );
        assert_eq!(rust.stdout, original.stdout, "stdout for {args:?}");
    }
}

#[test]
fn cli_map_single_read_option_matrix_stdout_matches_original() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let index = format!("{manifest_dir}/minibwa/chrM-human");
    let reads = format!("{manifest_dir}/minibwa/test/chrM-read_1.fa.gz");

    for extra in [
        vec!["--outn=1"],
        vec!["-u"],
        vec!["-5"],
        vec!["-Y", "-a"],
        vec!["--hic"],
        vec!["--rescue=0", "-a"],
        vec!["--kalloc=yes"],
        vec!["--kalloc=no"],
        vec!["--kalloc=y"],
        vec!["--kalloc=n"],
        vec!["--pe=yes"],
        vec!["--pe=no"],
        vec!["--pe=y"],
        vec!["--pe=n"],
        vec!["--adap=yes"],
        vec!["--adap=no"],
        vec!["--long=yes"],
        vec!["--long=no"],
        vec!["--long=y"],
        vec!["--long=n"],
        vec!["--pe-predef", "-a"],
        vec!["--pe-predef=no", "-a"],
    ] {
        let mut args = vec!["map"];
        args.extend(extra);
        args.extend(["-t", "1", "-K", "1k,1k", &index, &reads]);
        let rust = Command::new(rust_bin).args(&args).output().unwrap();
        let original = Command::new(original_bin).args(&args).output().unwrap();
        assert_eq!(rust.status.code(), Some(0), "rust status for {args:?}");
        assert_eq!(
            original.status.code(),
            Some(0),
            "original status for {args:?}"
        );
        assert_eq!(rust.stdout, original.stdout, "stdout for {args:?}");
    }
}

#[test]
fn cli_map_paired_option_matrix_stdout_matches_original() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let index = format!("{manifest_dir}/minibwa/chrM-human");
    let read1 = format!("{manifest_dir}/minibwa/test/chrM-read_1.fa.gz");
    let read2 = format!("{manifest_dir}/minibwa/test/chrM-read_2.fa.gz");

    for extra in [
        vec!["--pe-predef"],
        vec!["--rescue=0"],
        vec!["-P"],
        vec!["--pe=no"],
        vec!["--pe=yes"],
        vec!["--pe=n"],
        vec!["--pe=y"],
        vec!["--kalloc=no"],
        vec!["--kalloc=yes"],
        vec!["--kalloc=n"],
        vec!["--kalloc=y"],
        vec!["--adap=no"],
        vec!["--adap=yes"],
        vec!["--long=no"],
        vec!["--long=yes"],
        vec!["--hic"],
        vec!["--outn=1"],
        vec!["-Y"],
        vec!["-5"],
        vec!["-u"],
    ] {
        let mut args = vec!["map", "-a"];
        args.extend(extra);
        args.extend(["-t", "1", "-K", "1k,1k", &index, &read1, &read2]);
        let rust = Command::new(rust_bin).args(&args).output().unwrap();
        let original = Command::new(original_bin).args(&args).output().unwrap();
        assert_eq!(rust.status.code(), Some(0), "rust status for {args:?}");
        assert_eq!(
            original.status.code(),
            Some(0),
            "original status for {args:?}"
        );
        assert_eq!(rust.stdout, original.stdout, "stdout for {args:?}");
    }
}

#[test]
fn cli_index_outputs_match_original_for_small_fasta() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let dir = std::env::temp_dir().join(format!("minibwa_rs_cli_index_cmp_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let fasta = dir.join("in.fa");
    let rust_prefix = dir.join("rust");
    let original_prefix = dir.join("original");
    std::fs::write(&fasta, b">ctg\nACGTACGTACGT\n").unwrap();

    let rust = Command::new(rust_bin)
        .args([
            "index",
            "-u",
            "2",
            &fasta.to_string_lossy(),
            &rust_prefix.to_string_lossy(),
        ])
        .output()
        .unwrap();
    let original = Command::new(original_bin)
        .args([
            "index",
            "-u",
            "2",
            &fasta.to_string_lossy(),
            &original_prefix.to_string_lossy(),
        ])
        .output()
        .unwrap();

    assert_eq!(rust.status.code(), Some(0));
    assert_eq!(original.status.code(), Some(0));
    assert_eq!(
        std::fs::read(rust_prefix.with_extension("l2b")).unwrap(),
        std::fs::read(original_prefix.with_extension("l2b")).unwrap()
    );
    assert_eq!(
        std::fs::read(rust_prefix.with_extension("mbw")).unwrap(),
        std::fs::read(original_prefix.with_extension("mbw")).unwrap()
    );

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn cli_index_low_memory_outputs_match_original_for_small_fasta() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let dir = std::env::temp_dir().join(format!(
        "minibwa_rs_cli_index_lowmem_cmp_{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let fasta = dir.join("in.fa");
    let rust_prefix = dir.join("rust");
    let original_prefix = dir.join("original");
    std::fs::write(&fasta, b">ctg\nACGTACGTACGTACGTACGTACGTACGTACGT\n").unwrap();

    let rust = Command::new(rust_bin)
        .args([
            "index",
            "-l",
            "-u",
            "2",
            &fasta.to_string_lossy(),
            &rust_prefix.to_string_lossy(),
        ])
        .output()
        .unwrap();
    let original = Command::new(original_bin)
        .args([
            "index",
            "-l",
            "-u",
            "2",
            &fasta.to_string_lossy(),
            &original_prefix.to_string_lossy(),
        ])
        .output()
        .unwrap();

    assert_eq!(rust.status.code(), Some(0));
    assert_eq!(original.status.code(), Some(0));
    assert_eq!(
        std::fs::read(rust_prefix.with_extension("l2b")).unwrap(),
        std::fs::read(original_prefix.with_extension("l2b")).unwrap()
    );
    assert_eq!(
        std::fs::read(rust_prefix.with_extension("mbw")).unwrap(),
        std::fs::read(original_prefix.with_extension("mbw")).unwrap()
    );

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn cli_index_low_memory_meth_outputs_match_original_for_small_fasta() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let dir = std::env::temp_dir().join(format!(
        "minibwa_rs_cli_index_lowmem_meth_cmp_{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let fasta = dir.join("in.fa");
    let rust_prefix = dir.join("rust");
    let original_prefix = dir.join("original");
    std::fs::write(&fasta, b">ctg\nACGTACGTACGTACGTACGTACGTACGTACGT\n").unwrap();

    let rust = Command::new(rust_bin)
        .args([
            "index",
            "-l",
            "--meth",
            "-u",
            "2",
            &fasta.to_string_lossy(),
            &rust_prefix.to_string_lossy(),
        ])
        .output()
        .unwrap();
    let original = Command::new(original_bin)
        .args([
            "index",
            "-l",
            "--meth",
            "-u",
            "2",
            &fasta.to_string_lossy(),
            &original_prefix.to_string_lossy(),
        ])
        .output()
        .unwrap();

    assert_eq!(rust.status.code(), Some(0));
    assert_eq!(original.status.code(), Some(0));
    for ext in ["l2b", "mbw", "meth.mbw"] {
        assert_eq!(
            std::fs::read(rust_prefix.with_extension(ext)).unwrap(),
            std::fs::read(original_prefix.with_extension(ext)).unwrap(),
            "low-memory meth index mismatch for {ext}"
        );
    }

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn cli_index_outputs_match_original_for_real_chrm_fasta() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let fasta = format!("{manifest_dir}/minibwa/test/chrM-human.fa.gz");
    let dir =
        std::env::temp_dir().join(format!("minibwa_rs_cli_index_real_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let rust_prefix = dir.join("rust");
    let original_prefix = dir.join("original");

    let rust = Command::new(rust_bin)
        .args(["index", "-u", "2", &fasta, &rust_prefix.to_string_lossy()])
        .output()
        .unwrap();
    let original = Command::new(original_bin)
        .args([
            "index",
            "-u",
            "2",
            &fasta,
            &original_prefix.to_string_lossy(),
        ])
        .output()
        .unwrap();

    assert_eq!(rust.status.code(), Some(0));
    assert_eq!(original.status.code(), Some(0));
    assert_eq!(
        std::fs::read(rust_prefix.with_extension("l2b")).unwrap(),
        std::fs::read(original_prefix.with_extension("l2b")).unwrap()
    );
    assert_eq!(
        std::fs::read(rust_prefix.with_extension("mbw")).unwrap(),
        std::fs::read(original_prefix.with_extension("mbw")).unwrap()
    );

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn cli_fa2bit_output_matches_original_for_small_fasta() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let dir =
        std::env::temp_dir().join(format!("minibwa_rs_cli_fa2bit_cmp_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let fasta = dir.join("in.fa");
    let rust_l2b = dir.join("rust.l2b");
    let original_l2b = dir.join("original.l2b");
    std::fs::write(&fasta, b">ctg\nACGTACGTACGT\n").unwrap();

    let rust = Command::new(rust_bin)
        .args([
            "fa2bit",
            &fasta.to_string_lossy(),
            &rust_l2b.to_string_lossy(),
        ])
        .output()
        .unwrap();
    let original = Command::new(original_bin)
        .args([
            "fa2bit",
            &fasta.to_string_lossy(),
            &original_l2b.to_string_lossy(),
        ])
        .output()
        .unwrap();

    assert_eq!(rust.status.code(), Some(0));
    assert_eq!(original.status.code(), Some(0));
    assert_eq!(
        std::fs::read(rust_l2b).unwrap(),
        std::fs::read(original_l2b).unwrap()
    );

    let rust_stdout = Command::new(rust_bin)
        .args(["fa2bit", &fasta.to_string_lossy(), "-"])
        .output()
        .unwrap();
    let original_stdout = Command::new(original_bin)
        .args(["fa2bit", &fasta.to_string_lossy(), "-"])
        .output()
        .unwrap();
    assert_eq!(rust_stdout.status.code(), Some(0));
    assert_eq!(original_stdout.status.code(), Some(0));
    assert_eq!(rust_stdout.stdout, original_stdout.stdout);

    let rust_pac_stdout = Command::new(rust_bin)
        .args(["fa2bit", "-p", "-2", &fasta.to_string_lossy(), "-"])
        .output()
        .unwrap();
    let original_pac_stdout = Command::new(original_bin)
        .args(["fa2bit", "-p", "-2", &fasta.to_string_lossy(), "-"])
        .output()
        .unwrap();
    assert_eq!(rust_pac_stdout.status.code(), Some(0));
    assert_eq!(original_pac_stdout.status.code(), Some(0));
    assert_eq!(rust_pac_stdout.stdout, original_pac_stdout.stdout);

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn cli_fa2bit_metadata_control_and_nul_bytes_match_original() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let dir = std::env::temp_dir().join(format!(
        "minibwa_rs_cli_fa2bit_nul_meta_{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let fasta = dir.join("in.fa");
    let rust_l2b = dir.join("rust.l2b");
    let original_l2b = dir.join("original.l2b");
    std::fs::write(&fasta, b">ctg\x01raw\0hidden comment\0tail\nACGT\n").unwrap();

    let rust = Command::new(rust_bin)
        .args([
            "fa2bit",
            &fasta.to_string_lossy(),
            &rust_l2b.to_string_lossy(),
        ])
        .output()
        .unwrap();
    let original = Command::new(original_bin)
        .args([
            "fa2bit",
            &fasta.to_string_lossy(),
            &original_l2b.to_string_lossy(),
        ])
        .output()
        .unwrap();
    assert_eq!(rust.status.code(), Some(0));
    assert_eq!(original.status.code(), Some(0));
    assert_eq!(
        std::fs::read(rust_l2b).unwrap(),
        std::fs::read(original_l2b).unwrap()
    );

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn cli_fa2bit_stdin_stdout_matches_original() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let input = b">ctg comment here\nACGTNNACGT\n";

    for extra in [Vec::<&str>::new(), vec!["-p", "-2"]] {
        let mut rust_args = vec!["fa2bit"];
        rust_args.extend(extra.iter().copied());
        rust_args.extend(["-", "-"]);
        let mut rust_child = Command::new(rust_bin)
            .args(&rust_args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        rust_child.stdin.take().unwrap().write_all(input).unwrap();
        let rust = rust_child.wait_with_output().unwrap();

        let mut original_args = vec!["fa2bit"];
        original_args.extend(extra.iter().copied());
        original_args.extend(["-", "-"]);
        let mut original_child = Command::new(original_bin)
            .args(&original_args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        original_child
            .stdin
            .take()
            .unwrap()
            .write_all(input)
            .unwrap();
        let original = original_child.wait_with_output().unwrap();

        assert_eq!(rust.status.code(), Some(0), "rust status for {rust_args:?}");
        assert_eq!(
            original.status.code(),
            Some(0),
            "original status for {original_args:?}"
        );
        assert_eq!(rust.stdout, original.stdout, "stdout for {rust_args:?}");
    }
}

#[test]
fn cli_fa2bit_output_open_failure_matches_original() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let dir = std::env::temp_dir().join(format!(
        "minibwa_rs_cli_fa2bit_o_fail_{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let fasta = dir.join("in.fa");
    std::fs::write(&fasta, b">ctg\nACGTACGTACGT\n").unwrap();

    for extra in [Vec::<&str>::new(), vec!["-p", "-2"]] {
        let out = dir.join("missing-dir").join("out");
        let out_s = out.to_string_lossy().into_owned();
        let fasta_s = fasta.to_string_lossy().into_owned();
        let mut args = vec!["fa2bit".to_string()];
        args.extend(extra.iter().map(|s| s.to_string()));
        args.extend([fasta_s, out_s]);

        let rust = Command::new(rust_bin).args(&args).output().unwrap();
        let original = Command::new(original_bin).args(&args).output().unwrap();

        assert_eq!(rust.status.code(), Some(0), "rust status for {args:?}");
        assert_eq!(
            original.status.code(),
            Some(0),
            "original status for {args:?}"
        );
        assert_eq!(rust.stdout, original.stdout, "stdout for {args:?}");
    }

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn cli_index_save_failure_exit_paths_match_original() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let dir = std::env::temp_dir().join(format!(
        "minibwa_rs_cli_index_save_fail_{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let fasta = dir.join("in.fa");
    let l2b = dir.join("in.l2b");
    let pac = dir.join("in.pac");
    let raw = dir.join("in.raw");
    let bwt = dir.join("in.mbw");
    std::fs::write(&fasta, b">ctg\nACGTACGTNNACGTACGTACGTACGTACGT\n").unwrap();

    for args in [
        vec![
            "fa2bit".to_string(),
            fasta.to_string_lossy().into_owned(),
            l2b.to_string_lossy().into_owned(),
        ],
        vec![
            "fa2bit".to_string(),
            "-p".to_string(),
            "-2".to_string(),
            fasta.to_string_lossy().into_owned(),
            pac.to_string_lossy().into_owned(),
        ],
        vec![
            "genraw".to_string(),
            pac.to_string_lossy().into_owned(),
            raw.to_string_lossy().into_owned(),
        ],
        vec![
            "raw2bwt".to_string(),
            raw.to_string_lossy().into_owned(),
            bwt.to_string_lossy().into_owned(),
        ],
    ] {
        let output = Command::new(rust_bin).args(&args).output().unwrap();
        assert_eq!(output.status.code(), Some(0), "setup command {args:?}");
    }

    for args in [
        vec![
            "raw2bwt".to_string(),
            raw.to_string_lossy().into_owned(),
            dir.join("missing-dir/raw2bwt.mbw")
                .to_string_lossy()
                .into_owned(),
        ],
        vec![
            "genbwt".to_string(),
            "-u".to_string(),
            "2".to_string(),
            l2b.to_string_lossy().into_owned(),
            dir.join("missing-dir/genbwt.mbw")
                .to_string_lossy()
                .into_owned(),
        ],
        vec![
            "gensa".to_string(),
            bwt.to_string_lossy().into_owned(),
            dir.join("missing-dir/gensa.mbw")
                .to_string_lossy()
                .into_owned(),
        ],
        vec![
            "gensa".to_string(),
            "-r".to_string(),
            raw.to_string_lossy().into_owned(),
            dir.join("missing-dir/gensa-raw.mbw")
                .to_string_lossy()
                .into_owned(),
        ],
        vec![
            "index".to_string(),
            fasta.to_string_lossy().into_owned(),
            dir.join("missing-dir/index").to_string_lossy().into_owned(),
        ],
        vec![
            "index".to_string(),
            "--meth".to_string(),
            fasta.to_string_lossy().into_owned(),
            dir.join("missing-dir/index-meth")
                .to_string_lossy()
                .into_owned(),
        ],
    ] {
        let rust = Command::new(rust_bin).args(&args).output().unwrap();
        let original = Command::new(original_bin).args(&args).output().unwrap();
        assert_eq!(rust.status.code(), Some(0), "rust status for {args:?}");
        assert_eq!(
            original.status.code(),
            Some(0),
            "original status for {args:?}"
        );
        assert_eq!(rust.stdout, original.stdout, "stdout for {args:?}");
    }

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn cli_index_low_memory_missing_prefix_diagnostic_matches_original() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let dir = std::env::temp_dir().join(format!(
        "minibwa_rs_cli_index_lowmem_fail_{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let fasta = dir.join("in.fa");
    let prefix = dir.join("missing-dir").join("index");
    std::fs::write(&fasta, b">ctg\nACGTACGTACGTACGTACGTACGTACGTACGT\n").unwrap();
    let args = [
        "index",
        "-l",
        &fasta.to_string_lossy(),
        &prefix.to_string_lossy(),
    ];

    let rust = Command::new(rust_bin).args(args).output().unwrap();
    let original = Command::new(original_bin).args(args).output().unwrap();
    assert_eq!(rust.status.code(), original.status.code());
    assert_eq!(rust.stdout, original.stdout);
    assert_eq!(rust.stderr, original.stderr);

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn cli_genbwt_output_matches_original_for_small_fasta() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let dir =
        std::env::temp_dir().join(format!("minibwa_rs_cli_genbwt_cmp_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let fasta = dir.join("in.fa");
    let rust_l2b = dir.join("rust.l2b");
    let original_l2b = dir.join("original.l2b");
    std::fs::write(&fasta, b">ctg\nACGTACGTNNACGTACGTACGT\n").unwrap();

    for (bin, l2b) in [(rust_bin, &rust_l2b), (original_bin, &original_l2b)] {
        let output = Command::new(bin)
            .args(["fa2bit", &fasta.to_string_lossy(), &l2b.to_string_lossy()])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(0));
    }

    for extra in [Vec::<&str>::new(), vec!["-1"]] {
        let suffix = if extra.is_empty() { "both" } else { "forward" };
        let rust_bwt = dir.join(format!("rust.{suffix}.mbw"));
        let original_bwt = dir.join(format!("original.{suffix}.mbw"));
        let mut rust_args = vec!["genbwt".to_string()];
        rust_args.extend(extra.iter().map(|x| x.to_string()));
        rust_args.extend([
            "-u".to_string(),
            "2".to_string(),
            rust_l2b.to_string_lossy().into_owned(),
            rust_bwt.to_string_lossy().into_owned(),
        ]);
        let mut original_args = vec!["genbwt".to_string()];
        original_args.extend(extra.into_iter().map(|x| x.to_string()));
        original_args.extend([
            "-u".to_string(),
            "2".to_string(),
            original_l2b.to_string_lossy().into_owned(),
            original_bwt.to_string_lossy().into_owned(),
        ]);

        let rust = Command::new(rust_bin).args(rust_args).output().unwrap();
        let original = Command::new(original_bin)
            .args(original_args)
            .output()
            .unwrap();
        assert_eq!(rust.status.code(), Some(0));
        assert_eq!(original.status.code(), Some(0));
        assert_eq!(
            std::fs::read(rust_bwt).unwrap(),
            std::fs::read(original_bwt).unwrap()
        );
    }

    let rust_stdin_bwt = dir.join("rust.stdin.mbw");
    let original_stdin_bwt = dir.join("original.stdin.mbw");
    let mut rust_child = Command::new(rust_bin)
        .args(["genbwt", "-u", "2", "-", &rust_stdin_bwt.to_string_lossy()])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    rust_child
        .stdin
        .take()
        .unwrap()
        .write_all(&std::fs::read(&rust_l2b).unwrap())
        .unwrap();
    let rust_stdin = rust_child.wait_with_output().unwrap();
    let mut original_child = Command::new(original_bin)
        .args([
            "genbwt",
            "-u",
            "2",
            "-",
            &original_stdin_bwt.to_string_lossy(),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    original_child
        .stdin
        .take()
        .unwrap()
        .write_all(&std::fs::read(&original_l2b).unwrap())
        .unwrap();
    let original_stdin = original_child.wait_with_output().unwrap();
    assert_eq!(rust_stdin.status.code(), Some(0));
    assert_eq!(original_stdin.status.code(), Some(0));
    assert_eq!(
        std::fs::read(rust_stdin_bwt).unwrap(),
        std::fs::read(original_stdin_bwt).unwrap()
    );

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn cli_standalone_bwt_stages_match_original_for_small_fasta() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let dir =
        std::env::temp_dir().join(format!("minibwa_rs_cli_bwt_stages_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let fasta = dir.join("in.fa");
    let rust_pac = dir.join("rust.pac");
    let original_pac = dir.join("original.pac");
    let rust_raw = dir.join("rust.raw");
    let original_raw = dir.join("original.raw");
    let rust_bwt = dir.join("rust.bwt");
    let original_bwt = dir.join("original.bwt");
    let rust_rsa = dir.join("rust.rsa");
    let original_rsa = dir.join("original.rsa");
    let rust_sa = dir.join("rust.sa");
    let original_sa = dir.join("original.sa");
    let rust_sa_u2 = dir.join("rust.u2.sa");
    let original_sa_u2 = dir.join("original.u2.sa");
    let rust_rsa_u3 = dir.join("rust.u3.rsa");
    let original_rsa_u3 = dir.join("original.u3.rsa");
    std::fs::write(&fasta, b">ctg\nACGTACGTNNACGTACGTACGT\n").unwrap();

    for (bin, pac) in [(rust_bin, &rust_pac), (original_bin, &original_pac)] {
        let output = Command::new(bin)
            .args([
                "fa2bit",
                "-p",
                "-2",
                &fasta.to_string_lossy(),
                &pac.to_string_lossy(),
            ])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(0));
    }
    assert_eq!(
        std::fs::read(&rust_pac).unwrap(),
        std::fs::read(&original_pac).unwrap()
    );

    for (bin, pac, raw) in [
        (rust_bin, &rust_pac, &rust_raw),
        (original_bin, &original_pac, &original_raw),
    ] {
        let output = Command::new(bin)
            .args(["genraw", &pac.to_string_lossy(), &raw.to_string_lossy()])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(0));
    }

    for (bin, raw, bwt) in [
        (rust_bin, &rust_raw, &rust_bwt),
        (original_bin, &original_raw, &original_bwt),
    ] {
        let output = Command::new(bin)
            .args(["raw2bwt", &raw.to_string_lossy(), &bwt.to_string_lossy()])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(0));
    }

    for (bin, raw, rsa) in [
        (rust_bin, &rust_raw, &rust_rsa),
        (original_bin, &original_raw, &original_rsa),
    ] {
        let output = Command::new(bin)
            .args([
                "gensa",
                "-r",
                &raw.to_string_lossy(),
                &rsa.to_string_lossy(),
            ])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(0));
    }

    for (bin, bwt, sa) in [
        (rust_bin, &rust_bwt, &rust_sa),
        (original_bin, &original_bwt, &original_sa),
    ] {
        let output = Command::new(bin)
            .args(["gensa", &bwt.to_string_lossy(), &sa.to_string_lossy()])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(0));
    }

    for (bin, bwt, sa) in [
        (rust_bin, &rust_bwt, &rust_sa_u2),
        (original_bin, &original_bwt, &original_sa_u2),
    ] {
        let output = Command::new(bin)
            .args([
                "gensa",
                "-u",
                "2",
                &bwt.to_string_lossy(),
                &sa.to_string_lossy(),
            ])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(0));
    }

    for (bin, raw, rsa) in [
        (rust_bin, &rust_raw, &rust_rsa_u3),
        (original_bin, &original_raw, &original_rsa_u3),
    ] {
        let output = Command::new(bin)
            .args([
                "gensa",
                "-r",
                "-u",
                "3",
                &raw.to_string_lossy(),
                &rsa.to_string_lossy(),
            ])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(0));
    }

    assert_eq!(
        std::fs::read(rust_raw).unwrap(),
        std::fs::read(original_raw).unwrap()
    );
    assert_eq!(
        std::fs::read(rust_bwt).unwrap(),
        std::fs::read(original_bwt).unwrap()
    );
    assert_eq!(
        std::fs::read(rust_rsa).unwrap(),
        std::fs::read(original_rsa).unwrap()
    );
    assert_eq!(
        std::fs::read(rust_sa).unwrap(),
        std::fs::read(original_sa).unwrap()
    );
    assert_eq!(
        std::fs::read(rust_sa_u2).unwrap(),
        std::fs::read(original_sa_u2).unwrap()
    );
    assert_eq!(
        std::fs::read(rust_rsa_u3).unwrap(),
        std::fs::read(original_rsa_u3).unwrap()
    );

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn cli_index_meth_outputs_match_original_for_small_fasta() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let dir =
        std::env::temp_dir().join(format!("minibwa_rs_cli_index_meth_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let fasta = dir.join("in.fa");
    let reads = dir.join("reads.fa");
    let rust_prefix = dir.join("rust");
    let original_prefix = dir.join("original");
    std::fs::write(&fasta, b">ctg\nACGTCGACGTACGTCGACGTACGTACGTCGACGT\n").unwrap();
    std::fs::write(
        &reads,
        b">r1\nACGTCGACGTACGTCGACGT\n>r2\nTCGACGTACGTACGTCGACG\n",
    )
    .unwrap();

    let rust = Command::new(rust_bin)
        .args([
            "index",
            "--meth",
            "-u",
            "2",
            &fasta.to_string_lossy(),
            &rust_prefix.to_string_lossy(),
        ])
        .output()
        .unwrap();
    let original = Command::new(original_bin)
        .args([
            "index",
            "--meth",
            "-u",
            "2",
            &fasta.to_string_lossy(),
            &original_prefix.to_string_lossy(),
        ])
        .output()
        .unwrap();

    assert_eq!(rust.status.code(), Some(0));
    assert_eq!(original.status.code(), Some(0));
    assert_eq!(
        std::fs::read(rust_prefix.with_extension("l2b")).unwrap(),
        std::fs::read(original_prefix.with_extension("l2b")).unwrap()
    );
    assert_eq!(
        std::fs::read(rust_prefix.with_extension("mbw")).unwrap(),
        std::fs::read(original_prefix.with_extension("mbw")).unwrap()
    );
    assert_eq!(
        std::fs::read(dir.join("rust.meth.mbw")).unwrap(),
        std::fs::read(dir.join("original.meth.mbw")).unwrap()
    );

    let map_prefix = rust_prefix.to_string_lossy().into_owned();
    let reads_s = reads.to_string_lossy().into_owned();
    for extra in [
        Vec::<&str>::new(),
        vec!["-a"],
        vec!["--chain-only"],
        vec!["--outn=1"],
    ] {
        let mut rust_args = vec!["map".to_string(), "--meth".to_string()];
        rust_args.extend(extra.iter().map(|s| s.to_string()));
        rust_args.extend([
            "-t".to_string(),
            "1".to_string(),
            "-K".to_string(),
            "1k,1k".to_string(),
            map_prefix.clone(),
            reads_s.clone(),
        ]);
        let mut original_args = vec!["map".to_string(), "--meth".to_string()];
        original_args.extend(extra.iter().map(|s| s.to_string()));
        original_args.extend([
            "-t".to_string(),
            "1".to_string(),
            "-K".to_string(),
            "1k,1k".to_string(),
            map_prefix.clone(),
            reads_s.clone(),
        ]);

        let rust_map = Command::new(rust_bin).args(&rust_args).output().unwrap();
        let original_map = Command::new(original_bin)
            .args(&original_args)
            .output()
            .unwrap();
        assert_eq!(
            rust_map.status.code(),
            Some(0),
            "rust status for {rust_args:?}"
        );
        assert_eq!(
            original_map.status.code(),
            Some(0),
            "original status for {original_args:?}"
        );
        assert_eq!(
            rust_map.stdout, original_map.stdout,
            "stdout for {rust_args:?}"
        );
    }

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
#[ignore = "requires .tmp/compare-yeast-now fixtures prepared from the real yeast conformance data"]
fn cli_genbwt_threaded_matches_original_on_yeast() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let fixture_dir = std::path::Path::new(manifest_dir).join(".tmp/compare-yeast-now");
    let l2b = fixture_dir.join("ref.split.rust.l2b");
    let original_l2b = fixture_dir.join("ref.split.orig.l2b");
    let original_bwt = fixture_dir.join("ref.split.orig.mbw");
    let rust_bwt = fixture_dir.join("ref.split.rust.t4.test.mbw");

    assert!(l2b.exists(), "missing fixture {}", l2b.display());
    if !original_bwt.exists() {
        let original_input = if original_l2b.exists() {
            &original_l2b
        } else {
            &l2b
        };
        let original = Command::new(original_bin)
            .args([
                "genbwt",
                "-u",
                "2",
                "-t",
                "4",
                &original_input.to_string_lossy(),
                &original_bwt.to_string_lossy(),
            ])
            .output()
            .unwrap();
        assert_eq!(original.status.code(), Some(0));
    }

    let rust = Command::new(rust_bin)
        .args([
            "genbwt",
            "-u",
            "2",
            "-t",
            "4",
            &l2b.to_string_lossy(),
            &rust_bwt.to_string_lossy(),
        ])
        .output()
        .unwrap();
    assert_eq!(rust.status.code(), Some(0));
    assert_eq!(
        std::fs::read(&rust_bwt).unwrap(),
        std::fs::read(&original_bwt).unwrap()
    );
}

#[test]
#[ignore = "requires .tmp/large-real/yeast fixtures and the external yeast FASTQ"]
fn cli_map_full_alignment_matches_original_on_yeast_10k_subset() {
    let rust_bin = env!("CARGO_BIN_EXE_minibwa-rs");
    let original_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/minibwa/minibwa");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let index = format!("{manifest_dir}/.tmp/large-real/yeast/ref.orig");
    let reads_src = "/data/henriksson/github/claude/star/.tmp/yeast_conformance/SRR10143877.fastq";
    let reads = std::path::Path::new(manifest_dir).join(".tmp/large-real/yeast/reads_10k.fq");

    if !reads.exists() {
        let data = std::fs::read_to_string(reads_src).expect("read external yeast FASTQ");
        let subset = data.lines().take(40_000).collect::<Vec<_>>().join("\n") + "\n";
        std::fs::write(&reads, subset).expect("write yeast subset");
    }
    for extra in [Vec::<&str>::new(), vec!["-a"]] {
        let mut args = vec!["map"];
        args.extend(extra);
        args.extend([
            "-t",
            "1",
            "-K",
            "1m,1m",
            &index,
            reads.to_str().expect("utf8 path"),
        ]);
        let rust = Command::new(rust_bin).args(&args).output().unwrap();
        let original = Command::new(original_bin).args(&args).output().unwrap();
        assert_eq!(rust.status.code(), Some(0), "rust status for {args:?}");
        assert_eq!(
            original.status.code(),
            Some(0),
            "original status for {args:?}"
        );
        assert_eq!(rust.stdout, original.stdout, "stdout for {args:?}");
    }
}
