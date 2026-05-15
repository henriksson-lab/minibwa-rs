#![allow(unused_variables, dead_code, non_snake_case)]

use libc::c_long;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FILETIME {
    pub dwLowDateTime: u32,
    pub dwHighDateTime: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SYSTEMTIME {
    pub wHour: u16,
    pub wMinute: u16,
    pub wSecond: u16,
    pub wMilliseconds: u16,
}

/// Original C Windows global function `kom_cputime` from `minibwa/kommon.c:279`.
pub fn kom_cputime() -> f64 {
    // The Windows body sums kernel and user FILETIME values after converting
    // them to SYSTEMTIME. The portable crate runtime uses `kommon::kom_cputime`;
    // this cfg variant records the alternate original body for audit.
    let stKernel = SYSTEMTIME::default();
    let stUser = SYSTEMTIME::default();
    let kernelModeTime = ((stKernel.wHour as f64 * 60.0) + stKernel.wMinute as f64 * 60.0)
        + stKernel.wSecond as f64
        + stKernel.wMilliseconds as f64 / 1000.0;
    let userModeTime = ((stUser.wHour as f64 * 60.0) + stUser.wMinute as f64 * 60.0)
        + stUser.wSecond as f64
        + stUser.wMilliseconds as f64 / 1000.0;
    kernelModeTime + userModeTime
}

/// Original C Windows global function `kom_peakrss` from `minibwa/kommon.c:296`.
pub fn kom_peakrss() -> c_long {
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_cfg_timing_variants_follow_stubbed_original_shape() {
        assert_eq!(kom_cputime(), 0.0);
        assert_eq!(kom_peakrss(), 0);
    }
}
