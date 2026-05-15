use std::io::{self, BufWriter, Write};

fn write_to<W: Write>(writer: &mut W, text: &str) -> io::Result<()> {
    if text.is_empty() {
        Ok(())
    } else {
        writer.write_all(text.as_bytes())
    }
}

fn write_command_result<W: Write, E: Write>(
    ret: i32,
    out: &str,
    stdout: &mut W,
    stderr: &mut E,
) -> io::Result<()> {
    if ret == 0 {
        write_to(stdout, out)
    } else {
        write_to(stderr, out)
    }
}

/// Run the minibwa-compatible command-line dispatcher with captured writers.
///
/// `argv` must include the executable name at `argv[0]`, matching
/// `std::env::args()`. The returned integer is the process status a binary
/// wrapper should pass to `std::process::exit`.
pub fn run_with_writers<W: Write, E: Write>(
    argv: &[String],
    stdout: &mut W,
    stderr: &mut E,
) -> io::Result<i32> {
    let argc = argv.len();
    let _ = crate::kommon::kom_realtime();
    crate::stage_time::init_from_env();

    let mut command_ret = None;
    let status = if argc == 1 {
        let (ret, out) = crate::main::usage(true, 0);
        write_to(stdout, &out)?;
        ret
    } else if argv[1] == "map" || argv[1] == "mem" {
        let mut buffered_stdout = BufWriter::with_capacity(1 << 20, stdout);
        let (ret, out) = crate::map_main::main_map_write(&argv[1..], &mut buffered_stdout);
        buffered_stdout.flush()?;
        if !out.is_empty() && (ret == 0 || out.starts_with("@HD\t")) {
            write_to(buffered_stdout.get_mut(), &out)?;
        } else if !out.is_empty() {
            write_to(stderr, &out)?;
        }
        command_ret = Some(ret);
        0
    } else if argv[1] == "fastmap" {
        let mut buffered_stdout = BufWriter::with_capacity(1 << 20, stdout);
        let (ret, out) = crate::fastmap::main_fastmap_write(&argv[1..], &mut buffered_stdout);
        buffered_stdout.flush()?;
        if !out.is_empty() && ret == 0 {
            write_to(buffered_stdout.get_mut(), &out)?;
        } else if !out.is_empty() {
            write_to(stderr, &out)?;
        }
        command_ret = Some(ret);
        0
    } else if argv[1] == "--help" {
        let (ret, out) = crate::main::usage(true, 1);
        write_to(stdout, &out)?;
        ret
    } else if argv[1] == "version" {
        writeln!(stdout, "{}", crate::main::MB_VERSION)?;
        0
    } else if argv[1] == "index" {
        let (ret, out) = crate::index::main_index(&argv[1..]);
        write_command_result(ret, &out, stdout, stderr)?;
        command_ret = Some(ret);
        0
    } else if argv[1] == "fa2bit" {
        let (ret, out) = crate::index::main_fa2bit(&argv[1..]);
        write_command_result(ret, &out, stdout, stderr)?;
        command_ret = Some(ret);
        0
    } else if argv[1] == "genraw" {
        let (ret, out) = crate::index::main_genraw(&argv[1..]);
        write_command_result(ret, &out, stdout, stderr)?;
        command_ret = Some(ret);
        0
    } else if argv[1] == "raw2bwt" {
        let (ret, out) = crate::index::main_raw2bwt(&argv[1..]);
        write_command_result(ret, &out, stdout, stderr)?;
        command_ret = Some(ret);
        0
    } else if argv[1] == "genbwt" {
        let (ret, out) = crate::index::main_genbwt(&argv[1..]);
        write_command_result(ret, &out, stdout, stderr)?;
        command_ret = Some(ret);
        0
    } else if argv[1] == "gensa" {
        let (ret, out) = crate::index::main_gensa(&argv[1..]);
        write_command_result(ret, &out, stdout, stderr)?;
        command_ret = Some(ret);
        0
    } else if argv[1] == "bench" {
        let (_ret, out, err) = crate::main::main_bench(&argv[1..]);
        write_to(stdout, &out)?;
        write_to(stderr, &err)?;
        command_ret = Some(_ret);
        0
    } else {
        writeln!(stderr, "ERROR: unknown command '{}'", argv[1])?;
        1
    };

    let is_help_or_version = argv[1..]
        .iter()
        .any(|arg| arg == "--help" || arg == "--version" || arg == "version");
    if argc > 2 && command_ret == Some(0) && !is_help_or_version {
        writeln!(stderr, "[M::main] Version: {}", crate::main::MB_VERSION)?;
        write!(stderr, "[M::main] CMD:")?;
        for arg in argv {
            write!(stderr, " {arg}")?;
        }
        writeln!(
            stderr,
            "\n[M::main] Real time: {:.3} sec; CPU: {:.3} sec; Peak RSS: {:.3} GB",
            crate::kommon::kom_realtime(),
            crate::kommon::kom_cputime(),
            crate::kommon::kom_peakrss() as f64 / 1024.0 / 1024.0 / 1024.0
        )?;
        crate::stage_time::report();
    }

    Ok(status)
}

/// Run the minibwa-compatible command-line dispatcher using `std::env::args`.
pub fn run_from_env() -> i32 {
    let argv = std::env::args().collect::<Vec<_>>();
    let stdout = io::stdout();
    let stderr = io::stderr();
    let mut stdout = stdout.lock();
    let mut stderr = stderr.lock();
    match run_with_writers(&argv, &mut stdout, &mut stderr) {
        Ok(status) => status,
        Err(err) => {
            let _ = writeln!(io::stderr().lock(), "ERROR: {err}");
            1
        }
    }
}
