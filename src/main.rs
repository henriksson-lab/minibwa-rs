struct MiMalloc;

unsafe extern "C" {
    fn mi_malloc_aligned(size: usize, alignment: usize) -> *mut std::ffi::c_void;
    fn mi_realloc_aligned(
        p: *mut std::ffi::c_void,
        newsize: usize,
        alignment: usize,
    ) -> *mut std::ffi::c_void;
    fn mi_free(p: *mut std::ffi::c_void);
}

unsafe impl std::alloc::GlobalAlloc for MiMalloc {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        mi_malloc_aligned(layout.size().max(1), layout.align()).cast()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: std::alloc::Layout) {
        mi_free(ptr.cast());
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: std::alloc::Layout, new_size: usize) -> *mut u8 {
        mi_realloc_aligned(ptr.cast(), new_size.max(1), layout.align()).cast()
    }
}

#[global_allocator]
static GLOBAL_ALLOCATOR: MiMalloc = MiMalloc;

fn main() {
    let argv = std::env::args().collect::<Vec<_>>();
    let argc = argv.len();
    let _ = minibwa_rs::kommon::kom_realtime();
    minibwa_rs::stage_time::init_from_env();

    let mut command_ret = None;
    let status = if argc == 1 {
        let (ret, out) = minibwa_rs::main::usage(true, 0);
        print!("{out}");
        ret
    } else if argv[1] == "map" || argv[1] == "mem" {
        let stdout = std::io::stdout();
        let mut stdout = std::io::BufWriter::with_capacity(1 << 20, stdout.lock());
        let (ret, out) = minibwa_rs::map_main::main_map_write(&argv[1..], &mut stdout);
        if !out.is_empty() && (ret == 0 || out.starts_with("@HD\t")) {
            print!("{out}");
        } else if !out.is_empty() {
            eprint!("{out}");
        }
        command_ret = Some(ret);
        0
    } else if argv[1] == "fastmap" {
        let stdout = std::io::stdout();
        let mut stdout = std::io::BufWriter::with_capacity(1 << 20, stdout.lock());
        let (ret, out) = minibwa_rs::fastmap::main_fastmap_write(&argv[1..], &mut stdout);
        if !out.is_empty() && ret == 0 {
            print!("{out}");
        } else if !out.is_empty() {
            eprint!("{out}");
        }
        command_ret = Some(ret);
        0
    } else if argv[1] == "--help" {
        let (ret, out) = minibwa_rs::main::usage(true, 1);
        print!("{out}");
        ret
    } else if argv[1] == "version" {
        println!("{}", minibwa_rs::main::MB_VERSION);
        0
    } else if argv[1] == "index" {
        let (ret, out) = minibwa_rs::index::main_index(&argv[1..]);
        if ret == 0 {
            print!("{out}");
        } else {
            eprint!("{out}");
        }
        command_ret = Some(ret);
        0
    } else if argv[1] == "fa2bit" {
        let (ret, out) = minibwa_rs::index::main_fa2bit(&argv[1..]);
        if ret == 0 {
            print!("{out}");
        } else {
            eprint!("{out}");
        }
        command_ret = Some(ret);
        0
    } else if argv[1] == "genraw" {
        let (ret, out) = minibwa_rs::index::main_genraw(&argv[1..]);
        if ret == 0 {
            print!("{out}");
        } else {
            eprint!("{out}");
        }
        command_ret = Some(ret);
        0
    } else if argv[1] == "raw2bwt" {
        let (ret, out) = minibwa_rs::index::main_raw2bwt(&argv[1..]);
        if ret == 0 {
            print!("{out}");
        } else {
            eprint!("{out}");
        }
        command_ret = Some(ret);
        0
    } else if argv[1] == "genbwt" {
        let (ret, out) = minibwa_rs::index::main_genbwt(&argv[1..]);
        if ret == 0 {
            print!("{out}");
        } else {
            eprint!("{out}");
        }
        command_ret = Some(ret);
        0
    } else if argv[1] == "gensa" {
        let (ret, out) = minibwa_rs::index::main_gensa(&argv[1..]);
        if ret == 0 {
            print!("{out}");
        } else {
            eprint!("{out}");
        }
        command_ret = Some(ret);
        0
    } else if argv[1] == "bench" {
        let (_ret, out, err) = minibwa_rs::main::main_bench(&argv[1..]);
        print!("{out}");
        eprint!("{err}");
        command_ret = Some(_ret);
        0
    } else {
        eprintln!("ERROR: unknown command '{}'", argv[1]);
        1
    };

    let is_help_or_version = argv[1..]
        .iter()
        .any(|arg| arg == "--help" || arg == "--version" || arg == "version");
    if argc > 2 && command_ret == Some(0) && !is_help_or_version {
        eprintln!("[M::main] Version: {}", minibwa_rs::main::MB_VERSION);
        eprint!("[M::main] CMD:");
        for arg in &argv {
            eprint!(" {arg}");
        }
        eprintln!(
            "\n[M::main] Real time: {:.3} sec; CPU: {:.3} sec; Peak RSS: {:.3} GB",
            minibwa_rs::kommon::kom_realtime(),
            minibwa_rs::kommon::kom_cputime(),
            minibwa_rs::kommon::kom_peakrss() as f64 / 1024.0 / 1024.0 / 1024.0
        );
        minibwa_rs::stage_time::report();
    }

    std::process::exit(status);
}
