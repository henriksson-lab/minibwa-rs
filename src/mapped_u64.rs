#[cfg(all(unix, not(target_arch = "wasm32"), target_endian = "little"))]
use std::fs::File;
use std::ops::{Deref, DerefMut};
use std::path::Path;
#[cfg(all(unix, not(target_arch = "wasm32"), target_endian = "little"))]
use std::sync::Arc;

#[derive(Debug)]
pub enum U64Storage {
    Owned(Vec<u64>),
    #[cfg(all(unix, not(target_arch = "wasm32"), target_endian = "little"))]
    Mapped {
        map: Arc<UnixMmap>,
        byte_offset: usize,
        len: usize,
    },
}

impl Default for U64Storage {
    fn default() -> Self {
        Self::Owned(Vec::new())
    }
}

impl Clone for U64Storage {
    fn clone(&self) -> Self {
        match self {
            Self::Owned(v) => Self::Owned(v.clone()),
            #[cfg(all(unix, not(target_arch = "wasm32"), target_endian = "little"))]
            Self::Mapped {
                map,
                byte_offset,
                len,
            } => Self::Mapped {
                map: Arc::clone(map),
                byte_offset: *byte_offset,
                len: *len,
            },
        }
    }
}

impl PartialEq for U64Storage {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl Eq for U64Storage {}

impl PartialEq<Vec<u64>> for U64Storage {
    fn eq(&self, other: &Vec<u64>) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl From<Vec<u64>> for U64Storage {
    fn from(value: Vec<u64>) -> Self {
        Self::Owned(value)
    }
}

impl U64Storage {
    #[inline]
    pub fn new() -> Self {
        Self::Owned(Vec::new())
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self::Owned(Vec::with_capacity(capacity))
    }

    #[inline]
    pub fn as_slice(&self) -> &[u64] {
        match self {
            Self::Owned(v) => v.as_slice(),
            #[cfg(all(unix, not(target_arch = "wasm32"), target_endian = "little"))]
            Self::Mapped {
                map,
                byte_offset,
                len,
            } => unsafe {
                std::slice::from_raw_parts(map.as_ptr().add(*byte_offset).cast::<u64>(), *len)
            },
        }
    }

    #[inline]
    pub fn as_mut_vec(&mut self) -> &mut Vec<u64> {
        if !matches!(self, Self::Owned(_)) {
            *self = Self::Owned(self.as_slice().to_vec());
        }
        match self {
            Self::Owned(v) => v,
            #[cfg(all(unix, not(target_arch = "wasm32"), target_endian = "little"))]
            Self::Mapped { .. } => unreachable!(),
        }
    }

    #[inline]
    pub fn resize(&mut self, new_len: usize, value: u64) {
        self.as_mut_vec().resize(new_len, value);
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        match self {
            Self::Owned(v) => v.capacity(),
            #[cfg(all(unix, not(target_arch = "wasm32"), target_endian = "little"))]
            Self::Mapped { len, .. } => *len,
        }
    }

    #[inline]
    pub fn is_mapped(&self) -> bool {
        #[cfg(all(unix, not(target_arch = "wasm32"), target_endian = "little"))]
        {
            matches!(self, Self::Mapped { .. })
        }
        #[cfg(not(all(unix, not(target_arch = "wasm32"), target_endian = "little")))]
        {
            false
        }
    }

    #[inline]
    pub fn push(&mut self, value: u64) {
        self.as_mut_vec().push(value);
    }

    #[inline]
    pub fn extend_from_slice(&mut self, values: &[u64]) {
        self.as_mut_vec().extend_from_slice(values);
    }

    #[cfg(all(unix, not(target_arch = "wasm32"), target_endian = "little"))]
    fn mapped(map: Arc<UnixMmap>, byte_offset: usize, len: usize) -> Option<Self> {
        let byte_len = len.checked_mul(std::mem::size_of::<u64>())?;
        if byte_offset % std::mem::align_of::<u64>() != 0 {
            return None;
        }
        if byte_offset.checked_add(byte_len)? > map.len() {
            return None;
        }
        Some(Self::Mapped {
            map,
            byte_offset,
            len,
        })
    }
}

impl Deref for U64Storage {
    type Target = [u64];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl DerefMut for U64Storage {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_vec().as_mut_slice()
    }
}

#[cfg(all(unix, not(target_arch = "wasm32"), target_endian = "little"))]
#[derive(Debug)]
pub struct UnixMmap {
    ptr: *mut libc::c_void,
    len: usize,
}

#[cfg(all(unix, not(target_arch = "wasm32"), target_endian = "little"))]
unsafe impl Send for UnixMmap {}
#[cfg(all(unix, not(target_arch = "wasm32"), target_endian = "little"))]
unsafe impl Sync for UnixMmap {}

#[cfg(all(unix, not(target_arch = "wasm32"), target_endian = "little"))]
impl UnixMmap {
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr.cast::<u8>(), self.len) }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        self.ptr.cast::<u8>()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn preload(&self) {
        let page = 4096usize;
        let bytes = self.as_bytes();
        let mut acc = 0u8;
        let mut i = 0usize;
        while i < bytes.len() {
            acc ^= bytes[i];
            i = i.saturating_add(page);
        }
        if let Some(&last) = bytes.last() {
            acc ^= last;
        }
        std::hint::black_box(acc);
    }
}

#[cfg(all(unix, not(target_arch = "wasm32"), target_endian = "little"))]
impl Drop for UnixMmap {
    fn drop(&mut self) {
        if self.len != 0 {
            unsafe {
                libc::munmap(self.ptr, self.len);
            }
        }
    }
}

#[cfg(all(unix, not(target_arch = "wasm32"), target_endian = "little"))]
pub fn mmap_file<P: AsRef<Path>>(path: P, preload: i32) -> Option<Arc<UnixMmap>> {
    use std::os::fd::AsRawFd;

    let file = File::open(path).ok()?;
    let len = file.metadata().ok()?.len() as usize;
    if len == 0 {
        return None;
    }
    let ptr = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            len,
            libc::PROT_READ,
            libc::MAP_SHARED,
            file.as_raw_fd(),
            0,
        )
    };
    if ptr == libc::MAP_FAILED {
        return None;
    }
    let map = Arc::new(UnixMmap { ptr, len });
    if preload != 0 {
        map.preload();
    }
    Some(map)
}

#[cfg(not(all(unix, not(target_arch = "wasm32"), target_endian = "little")))]
pub fn mmap_file<P: AsRef<Path>>(_path: P, _preload: i32) -> Option<()> {
    None
}

#[cfg(all(unix, not(target_arch = "wasm32"), target_endian = "little"))]
pub fn mapped_u64_slice(map: Arc<UnixMmap>, byte_offset: usize, len: usize) -> Option<U64Storage> {
    U64Storage::mapped(map, byte_offset, len)
}
