#[cfg(not(any(windows, target_arch = "wasm32")))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    std::process::exit(minibwa_rs::cli::run_from_env());
}
