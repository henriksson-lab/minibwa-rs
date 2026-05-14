fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=c/mimalloc/static.c");
    println!("cargo:rerun-if-changed=c/mimalloc");
    println!("cargo:rerun-if-changed=c/ksw2/ksw2.h");
    println!("cargo:rerun-if-changed=c/ksw2/ksw2_extz2_sse.c");
    println!("cargo:rerun-if-changed=c/ksw2/ksw2_extd2_sse.c");
    println!("cargo:rerun-if-changed=c/ksw2/ksw2_ll_sse.c");

    let mut build = cc::Build::new();
    build
        .file("c/mimalloc/static.c")
        .include("c/mimalloc")
        .include("c/ksw2")
        .define("NDEBUG", None)
        .define("MI_MALLOC_OVERRIDE", None)
        .define("MI_OSX_INTERPOSE", Some("1"))
        .define("MI_OSX_ZONE", Some("1"))
        .warnings(false)
        .flag_if_supported("-std=gnu99")
        .flag_if_supported("-O3");

    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    if arch == "x86" || arch == "x86_64" {
        build
            .file("c/ksw2/ksw2_extz2_sse.c")
            .file("c/ksw2/ksw2_extd2_sse.c")
            .file("c/ksw2/ksw2_ll_sse.c")
            .flag_if_supported("-march=native")
            .flag_if_supported("-msse4.2")
            .flag_if_supported("-mpopcnt");
    }

    build.compile("minibwa_c");
}
