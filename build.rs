fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let mut build = cc::Build::new();
    build
        .include("c/ksw2")
        .define("NDEBUG", None)
        .warnings(false)
        .flag_if_supported("-std=gnu99")
        .flag_if_supported("-O3");

    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let c_ksw2 = std::env::var_os("CARGO_FEATURE_C_KSW2").is_some();
    if c_ksw2 && (arch == "x86" || arch == "x86_64") {
        println!("cargo:rerun-if-changed=c/ksw2/ksw2.h");
        println!("cargo:rerun-if-changed=c/ksw2/ksw2_extz2_sse.c");
        println!("cargo:rerun-if-changed=c/ksw2/ksw2_extd2_sse.c");
        println!("cargo:rerun-if-changed=c/ksw2/ksw2_ll_sse.c");
        build
            .file("c/ksw2/ksw2_extz2_sse.c")
            .file("c/ksw2/ksw2_extd2_sse.c")
            .file("c/ksw2/ksw2_ll_sse.c")
            .flag_if_supported("-march=native")
            .flag_if_supported("-msse4.2")
            .flag_if_supported("-mpopcnt");
        build.compile("minibwa_c");
    }
}
