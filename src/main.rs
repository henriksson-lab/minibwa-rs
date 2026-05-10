fn main() {
    let argv = std::env::args().collect::<Vec<_>>();
    let argc = argv.len();

    let status = if argc == 1 {
        let (ret, out) = minibwa_rs::main::usage(true, 0);
        print!("{out}");
        ret
    } else if argv[1] == "map" || argv[1] == "mem" {
        let (ret, out) = minibwa_rs::map_main::main_map(&argv[1..]);
        if ret == 0 {
            print!("{out}");
        } else {
            eprint!("{out}");
        }
        0
    } else if argv[1] == "fastmap" {
        let (ret, out) = minibwa_rs::fastmap::main_fastmap(&argv[1..]);
        if ret == 0 {
            print!("{out}");
        } else {
            eprint!("{out}");
        }
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
        0
    } else if argv[1] == "fa2bit" {
        let (ret, out) = minibwa_rs::index::main_fa2bit(&argv[1..]);
        if ret == 0 {
            print!("{out}");
        } else {
            eprint!("{out}");
        }
        0
    } else if argv[1] == "genraw" {
        let (ret, out) = minibwa_rs::index::main_genraw(&argv[1..]);
        if ret == 0 {
            print!("{out}");
        } else {
            eprint!("{out}");
        }
        0
    } else if argv[1] == "raw2bwt" {
        let (ret, out) = minibwa_rs::index::main_raw2bwt(&argv[1..]);
        if ret == 0 {
            print!("{out}");
        } else {
            eprint!("{out}");
        }
        0
    } else if argv[1] == "genbwt" {
        let (ret, out) = minibwa_rs::index::main_genbwt(&argv[1..]);
        if ret == 0 {
            print!("{out}");
        } else {
            eprint!("{out}");
        }
        0
    } else if argv[1] == "gensa" {
        let (ret, out) = minibwa_rs::index::main_gensa(&argv[1..]);
        if ret == 0 {
            print!("{out}");
        } else {
            eprint!("{out}");
        }
        0
    } else if argv[1] == "bench" {
        let (_ret, out, err) = minibwa_rs::main::main_bench(&argv[1..]);
        print!("{out}");
        eprint!("{err}");
        0
    } else {
        eprintln!("ERROR: unknown command '{}'", argv[1]);
        1
    };

    std::process::exit(status);
}
