#![allow(unused_variables, dead_code, non_snake_case, non_camel_case_types)]

use crate::QSufSort::QSufSortSuffixSort;
use std::io::Seek;

pub type bgint_t = u64;
pub type sbgint_t = i64;

pub const ALPHABET_SIZE: usize = 4;
pub const BIT_PER_CHAR: u32 = 2;
pub const CHAR_PER_WORD: u64 = 16;
pub const CHAR_PER_BYTE: u32 = 4;
pub const BITS_IN_WORD: u32 = 32;
pub const BITS_IN_BYTE: u32 = 8;
pub const BYTES_IN_WORD: usize = 4;
pub const ALL_ONE_MASK: u32 = 0xffff_ffff;
pub const DNA_OCC_CNT_TABLE_SIZE_IN_WORD: usize = 65536;
pub const BITS_PER_OCC_VALUE: u32 = 16;
pub const OCC_VALUE_PER_WORD: u64 = 2;
pub const OCC_INTERVAL: u64 = 256;
pub const OCC_INTERVAL_MAJOR: u64 = 65536;
pub const MIN_AVAILABLE_WORD: u64 = 0x10000;
pub const BWTINC_INSERT_SORT_NUM_ITEM: i64 = 7;

fn c_path(path: &std::path::Path) -> std::path::PathBuf {
    let lossy = path.to_string_lossy();
    let end = lossy
        .as_bytes()
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(lossy.len());
    std::path::PathBuf::from(lossy[..end].to_string())
}

fn c_strerror(errno: i32) -> String {
    std::io::Error::from_raw_os_error(errno)
        .to_string()
        .trim_end_matches(&format!(" (os error {errno})"))
        .to_string()
}

fn bwt_save_error_and_exit(path: &std::path::Path, err: std::io::Error) -> ! {
    let reason = err
        .raw_os_error()
        .map(c_strerror)
        .unwrap_or_else(|| err.to_string());
    eprintln!(
        "BWTSaveBwtCodeAndOcc(): Error writing to {} : {}",
        path.display(),
        reason
    );
    std::process::exit(1);
}

fn bwt_packed_seek_error_and_exit(path: &std::path::Path) -> ! {
    eprintln!(
        "BWTIncConstructFromPacked() : Can't seek on {} : {}",
        path.display(),
        c_strerror(libc::EINVAL)
    );
    std::process::exit(1);
}

fn bwt_packed_read_error_and_exit(path: &std::path::Path, err: std::io::Error) -> ! {
    let reason = err
        .raw_os_error()
        .map(c_strerror)
        .unwrap_or_else(|| err.to_string());
    eprintln!(
        "BWTIncConstructFromPacked() : Can't read from {} : {}",
        path.display(),
        reason
    );
    std::process::exit(1);
}

fn bwt_packed_seek_back(fp: &mut std::fs::File, path: &std::path::Path, n: u64) {
    let Ok(delta) = i64::try_from(n) else {
        bwt_packed_seek_error_and_exit(path);
    };
    if fp.seek(std::io::SeekFrom::Current(-delta)).is_err() {
        bwt_packed_seek_error_and_exit(path);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BWT {
    pub textLength: bgint_t,
    pub inverseSa0: bgint_t,
    pub cumulativeFreq: Vec<bgint_t>,
    pub bwtCode: Vec<u32>,
    pub occValue: Vec<u32>,
    pub occValueMajor: Vec<bgint_t>,
    pub decodeTable: Vec<u32>,
    pub bwtSizeInWord: bgint_t,
    pub occSizeInWord: bgint_t,
    pub occMajorSizeInWord: bgint_t,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BWTInc {
    pub bwt: BWT,
    pub numberOfIterationDone: u32,
    pub cumulativeCountInCurrentBuild: Vec<bgint_t>,
    pub availableWord: bgint_t,
    pub buildSize: bgint_t,
    pub initialMaxBuildSize: bgint_t,
    pub incMaxBuildSize: bgint_t,
    pub firstCharInLastIteration: u32,
    pub workingMemory: Vec<u32>,
    pub packedText: Vec<u32>,
    pub textBuffer: Vec<u8>,
    pub packedShift: Vec<u32>,
    pub packedTextOffset: usize,
    pub textBufferOffset: usize,
}

/// Original C static function `TextLengthFromBytePacked` from `minibwa/bwtgen.c:97`.
pub fn TextLengthFromBytePacked(
    bytePackedLength: bgint_t,
    bitPerChar: u32,
    lastByteLength: u32,
) -> bgint_t {
    (bytePackedLength - 1) * (BITS_IN_BYTE / bitPerChar) as u64 + lastByteLength as u64
}

/// Original C static function `initializeVAL` from `minibwa/bwtgen.c:103`.
pub fn initializeVAL(startAddr: &mut [u32], length: bgint_t, initValue: u32) {
    for x in startAddr.iter_mut().take(length as usize) {
        *x = initValue;
    }
}

/// Original C static function `initializeVAL_bg` from `minibwa/bwtgen.c:109`.
pub fn initializeVAL_bg(startAddr: &mut [bgint_t], length: bgint_t, initValue: bgint_t) {
    for x in startAddr.iter_mut().take(length as usize) {
        *x = initValue;
    }
}

#[inline]
fn as_qsint_slice_mut(values: &mut [bgint_t]) -> &mut [sbgint_t] {
    unsafe { std::slice::from_raw_parts_mut(values.as_mut_ptr() as *mut sbgint_t, values.len()) }
}

fn uninit_vec<T>(len: usize) -> Vec<T> {
    let mut values = Vec::<std::mem::MaybeUninit<T>>::with_capacity(len);
    unsafe {
        values.set_len(len);
        let ptr = values.as_mut_ptr() as *mut T;
        let cap = values.capacity();
        std::mem::forget(values);
        Vec::from_raw_parts(ptr, len, cap)
    }
}

/// Original C static function `GenerateDNAOccCountTable` from `minibwa/bwtgen.c:115`.
pub fn GenerateDNAOccCountTable(dnaDecodeTable: &mut [u32]) {
    for i in 0..DNA_OCC_CNT_TABLE_SIZE_IN_WORD {
        let mut c = i as u32;
        dnaDecodeTable[i] = 0;
        for _ in 0..8 {
            let t = c & 3;
            dnaDecodeTable[i] = dnaDecodeTable[i].wrapping_add(1 << (t * 8));
            c >>= 2;
        }
    }
}

/// Original C static function `BWTOccValueMajorSizeInWord` from `minibwa/bwtgen.c:131`.
pub fn BWTOccValueMajorSizeInWord(numChar: bgint_t) -> bgint_t {
    let numOfOccValue = (numChar + OCC_INTERVAL - 1) / OCC_INTERVAL + 1;
    let numOfOccIntervalPerMajor = OCC_INTERVAL_MAJOR / OCC_INTERVAL;
    (numOfOccValue + numOfOccIntervalPerMajor - 1) / numOfOccIntervalPerMajor * ALPHABET_SIZE as u64
}

/// Original C static function `BWTOccValueMinorSizeInWord` from `minibwa/bwtgen.c:140`.
pub fn BWTOccValueMinorSizeInWord(numChar: bgint_t) -> bgint_t {
    let numOfOccValue = (numChar + OCC_INTERVAL - 1) / OCC_INTERVAL + 1;
    (numOfOccValue + OCC_VALUE_PER_WORD - 1) / OCC_VALUE_PER_WORD * ALPHABET_SIZE as u64
}

/// Original C static function `BWTResidentSizeInWord` from `minibwa/bwtgen.c:147`.
pub fn BWTResidentSizeInWord(numChar: bgint_t) -> bgint_t {
    let numCharRoundUpToOccInterval = (numChar + OCC_INTERVAL - 1) / OCC_INTERVAL * OCC_INTERVAL;
    (numCharRoundUpToOccInterval + CHAR_PER_WORD - 1) / CHAR_PER_WORD
}

/// Original C static function `BWTIncSetBuildSizeAndTextAddr` from `minibwa/bwtgen.c:158`.
pub fn BWTIncSetBuildSizeAndTextAddr(bwtInc: &mut BWTInc) {
    let word_scale = (std::mem::size_of::<bgint_t>() / 4) as u64;
    let maxBuildSize;
    if bwtInc.bwt.textLength == 0 {
        maxBuildSize = (bwtInc.availableWord - (2 + OCC_INTERVAL / CHAR_PER_WORD) * word_scale)
            / (2 * CHAR_PER_WORD + 1)
            * CHAR_PER_WORD
            / word_scale;
        if bwtInc.initialMaxBuildSize > 0 {
            bwtInc.buildSize = bwtInc.initialMaxBuildSize.min(maxBuildSize);
        } else {
            bwtInc.buildSize = maxBuildSize;
        }
    } else {
        maxBuildSize = (bwtInc.availableWord
            - bwtInc.bwt.bwtSizeInWord
            - bwtInc.bwt.occSizeInWord
            - (3 + bwtInc.numberOfIterationDone as u64 * OCC_INTERVAL / BIT_PER_CHAR as u64)
                * word_scale)
            / 3
            / word_scale;
        if maxBuildSize < CHAR_PER_WORD {
            eprintln!("BWTIncSetBuildSizeAndTextAddr(): Not enough space allocated to continue construction!");
            std::process::exit(1);
        }
        if bwtInc.incMaxBuildSize > 0 {
            bwtInc.buildSize = bwtInc.incMaxBuildSize.min(maxBuildSize);
        } else {
            bwtInc.buildSize = maxBuildSize;
        }
        if bwtInc.buildSize < CHAR_PER_WORD {
            bwtInc.buildSize = CHAR_PER_WORD;
        }
    }
    if bwtInc.buildSize < CHAR_PER_WORD {
        eprintln!(
            "BWTIncSetBuildSizeAndTextAddr(): Not enough space allocated to continue construction!"
        );
        std::process::exit(1);
    }
    bwtInc.buildSize = bwtInc.buildSize / CHAR_PER_WORD * CHAR_PER_WORD;
    bwtInc.packedTextOffset = (2 * (bwtInc.buildSize + 1) * word_scale) as usize;
    bwtInc.textBufferOffset = ((bwtInc.buildSize + 1) * word_scale) as usize;
}

/// Original C global function `leadingZero` from `minibwa/bwtgen.c:203`.
pub fn leadingZero(input: u32) -> u32 {
    const LEADING_ZERO_8BIT: [u32; 256] = [
        8, 7, 6, 6, 5, 5, 5, 5, 4, 4, 4, 4, 4, 4, 4, 4, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
        3, 3, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
        2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    if (input & 0xffff0000) != 0 {
        if (input & 0xff000000) != 0 {
            LEADING_ZERO_8BIT[(input >> 24) as usize]
        } else {
            8 + LEADING_ZERO_8BIT[(input >> 16) as usize]
        }
    } else if (input & 0x0000ff00) != 0 {
        16 + LEADING_ZERO_8BIT[(input >> 8) as usize]
    } else {
        24 + LEADING_ZERO_8BIT[input as usize]
    }
}

/// Original C static function `ceilLog2` from `minibwa/bwtgen.c:232`.
pub fn ceilLog2(input: u32) -> u32 {
    if input <= 1 {
        0
    } else {
        BITS_IN_WORD - leadingZero(input - 1)
    }
}

/// Original C static function `BitPerBytePackedChar` from `minibwa/bwtgen.c:239`.
pub fn BitPerBytePackedChar(alphabetSize: u32) -> u32 {
    let mut bitPerChar = ceilLog2(alphabetSize);
    if BITS_IN_BYTE / (BITS_IN_BYTE / bitPerChar) > bitPerChar {
        bitPerChar = BITS_IN_BYTE / (BITS_IN_BYTE / bitPerChar);
    }
    bitPerChar
}

/// Original C static function `BitPerWordPackedChar` from `minibwa/bwtgen.c:249`.
pub fn BitPerWordPackedChar(alphabetSize: u32) -> u32 {
    ceilLog2(alphabetSize)
}

/// Original C static function `ConvertBytePackedToWordPacked` from `minibwa/bwtgen.c:254`.
pub fn ConvertBytePackedToWordPacked(
    input: &[u8],
    output: &mut [u32],
    alphabetSize: u32,
    textLength: bgint_t,
) {
    let bitPerBytePackedChar = BitPerBytePackedChar(alphabetSize);
    let bitPerWordPackedChar = BitPerWordPackedChar(alphabetSize);
    let charPerByte = BITS_IN_BYTE / bitPerBytePackedChar;
    let charPerWord = BITS_IN_WORD / bitPerWordPackedChar;
    let bytePerIteration = charPerWord / charPerByte;
    let mask = ALL_ONE_MASK >> (BITS_IN_WORD - bitPerWordPackedChar)
        << (BITS_IN_WORD - bitPerWordPackedChar);
    let shift = BITS_IN_WORD - BITS_IN_BYTE + bitPerBytePackedChar - bitPerWordPackedChar;
    let mut byteProcessed = 0usize;
    let mut wordProcessed = 0usize;
    let mut buffer = [0u32; BITS_IN_WORD as usize];
    while ((wordProcessed + 1) as u64) * (charPerWord as u64) < textLength {
        let mut k = 0usize;
        for _ in 0..bytePerIteration {
            let mut c = (input[byteProcessed] as u32) << shift;
            for _ in 0..charPerByte {
                buffer[k] = c & mask;
                c <<= bitPerBytePackedChar;
                k += 1;
            }
            byteProcessed += 1;
        }
        let mut c = 0u32;
        for (i, &b) in buffer.iter().enumerate().take(charPerWord as usize) {
            c |= b >> (bitPerWordPackedChar * i as u32);
        }
        output[wordProcessed] = c;
        wordProcessed += 1;
    }
    let mut k = 0usize;
    let remaining = textLength - wordProcessed as u64 * charPerWord as u64;
    for _ in 0..((remaining - 1) / charPerByte as u64 + 1) {
        let mut c = (input[byteProcessed] as u32) << shift;
        for _ in 0..charPerByte {
            buffer[k] = c & mask;
            c <<= bitPerBytePackedChar;
            k += 1;
        }
        byteProcessed += 1;
    }
    let mut c = 0u32;
    for (i, &b) in buffer.iter().enumerate().take(remaining as usize) {
        c |= b >> (bitPerWordPackedChar * i as u32);
    }
    output[wordProcessed] = c;
}

/// Original C global function `BWTCreate` from `minibwa/bwtgen.c:319`.
pub fn BWTCreate(textLength: bgint_t, decodeTable: Option<Vec<u32>>) -> BWT {
    let mut cumulativeFreq = vec![0; ALPHABET_SIZE + 1];
    initializeVAL_bg(&mut cumulativeFreq, (ALPHABET_SIZE + 1) as u64, 0);
    let decodeTable = match decodeTable {
        Some(t) => t,
        None => {
            let mut t = vec![0; DNA_OCC_CNT_TABLE_SIZE_IN_WORD];
            GenerateDNAOccCountTable(&mut t);
            t
        }
    };
    let occMajorSizeInWord = BWTOccValueMajorSizeInWord(textLength);
    BWT {
        textLength: 0,
        inverseSa0: 0,
        cumulativeFreq,
        bwtCode: Vec::new(),
        occValue: Vec::new(),
        occValueMajor: vec![0; occMajorSizeInWord as usize],
        decodeTable,
        bwtSizeInWord: 0,
        occSizeInWord: 0,
        occMajorSizeInWord,
    }
}

/// Original C global function `BWTIncCreate` from `minibwa/bwtgen.c:350`.
pub fn BWTIncCreate(
    textLength: bgint_t,
    mut initialMaxBuildSize: u32,
    mut incMaxBuildSize: u32,
) -> BWTInc {
    if textLength < incMaxBuildSize as u64 {
        incMaxBuildSize = textLength as u32;
    }
    if textLength < initialMaxBuildSize as u64 {
        initialMaxBuildSize = textLength as u32;
    }
    let mut packedShift = vec![0; CHAR_PER_WORD as usize];
    for (i, x) in packedShift.iter_mut().enumerate() {
        *x = BITS_IN_WORD - (i as u32 + 1) * BIT_PER_CHAR;
    }
    let n_iter = (textLength - initialMaxBuildSize as u64) / incMaxBuildSize as u64 + 1;
    let word_scale = (std::mem::size_of::<bgint_t>() / 4) as u64;
    let mut availableWord = BWTResidentSizeInWord(textLength)
        + BWTOccValueMinorSizeInWord(textLength)
        + OCC_INTERVAL / BIT_PER_CHAR as u64 * n_iter * 2 * word_scale
        + incMaxBuildSize as u64 / 5 * 3 * word_scale;
    if availableWord < MIN_AVAILABLE_WORD {
        availableWord = MIN_AVAILABLE_WORD;
    }
    eprintln!("[BWTIncCreate] textLength={textLength}, availableWord={availableWord}");
    BWTInc {
        bwt: BWTCreate(textLength, None),
        numberOfIterationDone: 0,
        cumulativeCountInCurrentBuild: vec![0; ALPHABET_SIZE + 1],
        availableWord,
        buildSize: 0,
        initialMaxBuildSize: initialMaxBuildSize as u64,
        incMaxBuildSize: incMaxBuildSize as u64,
        firstCharInLastIteration: 0,
        workingMemory: Vec::new(),
        packedText: Vec::new(),
        textBuffer: Vec::new(),
        packedShift,
        packedTextOffset: 0,
        textBufferOffset: 0,
    }
}

/// Original C static function `BWTIncPutPackedTextToRank` from `minibwa/bwtgen.c:382`.
pub fn BWTIncPutPackedTextToRank(
    packedText: &[u32],
    rank: &mut [bgint_t],
    cumulativeCount: &mut [bgint_t],
    numChar: bgint_t,
) {
    let lastWord = (numChar - 1) / CHAR_PER_WORD;
    let numCharInLastWord = numChar - lastWord * CHAR_PER_WORD;
    let packedMask = ALL_ONE_MASK >> (BITS_IN_WORD - BIT_PER_CHAR);
    let mut rankIndex = numChar - 1;

    let mut t =
        packedText[lastWord as usize] >> (BITS_IN_WORD - numCharInLastWord as u32 * BIT_PER_CHAR);
    for _ in 0..numCharInLastWord {
        let c = t & packedMask;
        cumulativeCount[c as usize + 1] += 1;
        rank[rankIndex as usize] = c as bgint_t;
        rankIndex = rankIndex.wrapping_sub(1);
        t >>= BIT_PER_CHAR;
    }

    for i in (0..lastWord as usize).rev() {
        t = packedText[i];
        for _ in 0..CHAR_PER_WORD {
            let c = t & packedMask;
            cumulativeCount[c as usize + 1] += 1;
            rank[rankIndex as usize] = c as bgint_t;
            rankIndex = rankIndex.wrapping_sub(1);
            t >>= BIT_PER_CHAR;
        }
    }

    cumulativeCount[2] += cumulativeCount[1];
    cumulativeCount[3] += cumulativeCount[2];
    cumulativeCount[4] += cumulativeCount[3];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_table_and_size_helpers_match_expected_values() {
        assert_eq!(TextLengthFromBytePacked(3, 2, 2), 10);
        assert_eq!(BWTOccValueMajorSizeInWord(0), 4);
        assert_eq!(BWTOccValueMajorSizeInWord(65_536), 8);
        assert_eq!(BWTOccValueMinorSizeInWord(0), 4);
        assert_eq!(BWTOccValueMinorSizeInWord(256), 4);
        assert_eq!(BWTOccValueMinorSizeInWord(257), 8);
        assert_eq!(BWTResidentSizeInWord(1), 16);
        assert_eq!(BWTResidentSizeInWord(256), 16);
        assert_eq!(BWTResidentSizeInWord(257), 32);

        let mut table = vec![0u32; DNA_OCC_CNT_TABLE_SIZE_IN_WORD];
        GenerateDNAOccCountTable(&mut table);
        assert_eq!(table[0], 8);
        assert_eq!(table[0xffff], 8 << 24);
        assert_eq!(table[0x1b1b], 0x02020202);
    }

    #[test]
    fn bit_width_helpers_follow_original_rounding() {
        assert_eq!(leadingZero(0), 32);
        assert_eq!(leadingZero(1), 31);
        assert_eq!(leadingZero(0x8000_0000), 0);
        assert_eq!(ceilLog2(1), 0);
        assert_eq!(ceilLog2(2), 1);
        assert_eq!(ceilLog2(3), 2);
        assert_eq!(BitPerBytePackedChar(4), 2);
        assert_eq!(BitPerWordPackedChar(4), 2);
    }

    #[test]
    fn byte_packed_to_word_packed_preserves_dna_order() {
        let input = [0x1b, 0x1b, 0x1b, 0x1b, 0x10];
        let mut output = [0u32; 2];
        ConvertBytePackedToWordPacked(&input, &mut output, 4, 18);
        assert_eq!(output[0], 0x1b1b1b1b);
        assert_eq!(output[1], 0x10000000);
    }

    #[test]
    fn bwt_create_and_inc_create_initialize_metadata() {
        let bwt = BWTCreate(1000, None);
        assert_eq!(bwt.textLength, 0);
        assert_eq!(bwt.cumulativeFreq, vec![0; 5]);
        assert_eq!(bwt.occMajorSizeInWord, BWTOccValueMajorSizeInWord(1000));
        assert_eq!(bwt.occValueMajor.len(), bwt.occMajorSizeInWord as usize);
        assert_eq!(bwt.decodeTable[0x1b1b], 0x02020202);

        let mut inc = BWTIncCreate(1000, 320, 160);
        assert_eq!(inc.packedShift[0], 30);
        assert_eq!(inc.packedShift[15], 0);
        assert!(inc.availableWord >= MIN_AVAILABLE_WORD);
        BWTIncSetBuildSizeAndTextAddr(&mut inc);
        assert_eq!(inc.buildSize % CHAR_PER_WORD, 0);
        assert!(inc.packedTextOffset > inc.textBufferOffset);
    }

    #[test]
    fn packed_text_to_rank_decodes_reverse_then_cumulative_counts() {
        let packed = [0x1b1b1b1b, 0x10000000];
        let mut rank = [0u64; 18];
        let mut cumulative = [0u64; 5];
        BWTIncPutPackedTextToRank(&packed, &mut rank, &mut cumulative, 18);
        assert_eq!(rank, [0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1]);
        assert_eq!(cumulative, [0, 5, 10, 14, 18]);
    }

    #[test]
    fn occurrence_counters_match_bruteforce_on_packed_words() {
        let dna = [0x1b1b1b1b, 0x10000000];
        let mut table = vec![0u32; DNA_OCC_CNT_TABLE_SIZE_IN_WORD];
        GenerateDNAOccCountTable(&mut table);

        assert_eq!(ForwardDNAOccCount(&dna, 10, 0, &table), 3);
        assert_eq!(ForwardDNAOccCount(&dna, 10, 1, &table), 3);
        assert_eq!(ForwardDNAOccCount(&dna, 10, 2, &table), 2);
        assert_eq!(ForwardDNAOccCount(&dna, 10, 3, &table), 2);

        let mut occ = [0u64; 4];
        ForwardDNAAllOccCountNoLimit(&dna, 18, &mut occ, &table);
        assert_eq!(occ, [5, 5, 4, 4]);
        assert_eq!(BackwardDNAOccCount(&dna, 4, 0, &table), 4);
        assert_eq!(BackwardDNAOccCount(&dna, 4, 1, &table), 0);
    }

    #[test]
    fn packed_bwt_builder_and_occ_value_follow_original_layout() {
        let relative = [3, 0, 1, 2, 4];
        let cumulative = [0, 1, 2, 3, 4];
        let shifts = (0..16).map(|i| 30 - i * 2).collect::<Vec<_>>();
        let mut bwt_words = [0u32; 1];
        BWTIncBuildPackedBwt(&relative, &mut bwt_words, 4, &cumulative, &shifts);
        assert_eq!(bwt_words[0], 0x81000000);

        let mut bwt = BWTCreate(64, None);
        bwt.inverseSa0 = 100;
        bwt.bwtCode = vec![0x1b1b1b1b, 0x10000000];
        bwt.occValue = vec![0; 4];
        bwt.occValue[2] = (7 << 16) | 3;
        bwt.occValueMajor[2] = 100;
        assert_eq!(BWTOccValueExplicit(&bwt, 0, 2), 107);
        assert_eq!(BWTOccValueExplicit(&bwt, 1, 2), 103);
        assert_eq!(BWTOccValue(&bwt, 18, 0), 5);
        bwt.inverseSa0 = 8;
        assert_eq!(BWTOccValue(&bwt, 18, 0), 5);
    }

    #[test]
    fn incremental_sort_and_rank_builders_follow_key_groups() {
        let mut key = [5, 1, 3, 1, 4, 3, 2, 5, 0, 4];
        let mut seq = [50, 10, 30, 11, 40, 31, 20, 51, 0, 41];
        BWTIncSortKey(&mut key, &mut seq, 10);
        assert_eq!(key, [0, 1, 1, 2, 3, 3, 4, 4, 5, 5]);
        for pair in key.windows(2) {
            assert!(pair[0] <= pair[1]);
        }

        let mut sorted_rank = [1, 2, 2, 5, 8, 9];
        let mut grouped_seq = [0, 1, 2, 3, 4, 5];
        let mut relative = [0u64; 6];
        BWTIncBuildRelativeRank(
            &mut sorted_rank,
            &mut grouped_seq,
            &mut relative,
            5,
            4,
            &[0, 2, 4, 5, 6],
        );
        assert_eq!(relative[5], 5);
        assert_eq!(relative[1], 1);
        assert_eq!(relative[2], 2);
        assert_eq!(grouped_seq[5], (-1i64) as u64);
    }

    #[test]
    fn generated_occ_values_match_bruteforce_counts() {
        let bases: Vec<u32> = (0..512).map(|i| ((i * 7 + i / 3) & 3) as u32).collect();
        let mut bwt_code =
            vec![0u32; (bases.len() + CHAR_PER_WORD as usize - 1) / CHAR_PER_WORD as usize];
        for (i, &base) in bases.iter().enumerate() {
            let word = i / CHAR_PER_WORD as usize;
            let shift = BITS_IN_WORD - ((i % CHAR_PER_WORD as usize) as u32 + 1) * BIT_PER_CHAR;
            bwt_code[word] |= base << shift;
        }
        let mut decode = vec![0u32; DNA_OCC_CNT_TABLE_SIZE_IN_WORD];
        GenerateDNAOccCountTable(&mut decode);
        let mut occ_value = vec![0u32; BWTOccValueMinorSizeInWord(bases.len() as u64) as usize];
        let mut occ_major = vec![0u64; BWTOccValueMajorSizeInWord(bases.len() as u64) as usize];
        BWTGenerateOccValueFromBwt(
            &bwt_code,
            &mut occ_value,
            &mut occ_major,
            bases.len() as u64,
            &decode,
        );

        let bwt = BWT {
            textLength: bases.len() as u64,
            inverseSa0: bases.len() as u64 + 100,
            cumulativeFreq: vec![0; ALPHABET_SIZE + 1],
            bwtCode: bwt_code,
            occValue: occ_value,
            occValueMajor: occ_major,
            decodeTable: decode,
            bwtSizeInWord: BWTResidentSizeInWord(bases.len() as u64),
            occSizeInWord: BWTOccValueMinorSizeInWord(bases.len() as u64),
            occMajorSizeInWord: BWTOccValueMajorSizeInWord(bases.len() as u64),
        };
        for &idx in &[0u64, 1, 17, 255, 256, 257, 511, 512] {
            for c in 0..4u32 {
                let expected = bases[..idx as usize].iter().filter(|&&x| x == c).count() as u64;
                assert_eq!(BWTOccValue(&bwt, idx, c), expected, "idx={idx} c={c}");
            }
        }
    }

    #[test]
    fn trailing_bwt_code_is_cleared_after_text_length() {
        let mut bwt = BWTCreate(300, None);
        bwt.textLength = 18;
        bwt.bwtCode = vec![0xffff_ffff; BWTResidentSizeInWord(18) as usize];
        BWTClearTrailingBwtCode(&mut bwt);
        assert_eq!(bwt.bwtCode[0], 0xffff_ffff);
        assert_eq!(bwt.bwtCode[1], 0xf000_0000);
        assert!(bwt.bwtCode[2..].iter().all(|&x| x == 0));
    }

    #[test]
    fn bwt_file_size_and_save_emit_original_binary_layout() {
        assert_eq!(BWTFileSizeInWord(0), 0);
        assert_eq!(BWTFileSizeInWord(1), 1);
        assert_eq!(BWTFileSizeInWord(16), 1);
        assert_eq!(BWTFileSizeInWord(17), 2);

        let mut bwt = BWTCreate(32, None);
        bwt.textLength = 17;
        bwt.inverseSa0 = 9;
        bwt.cumulativeFreq = vec![0, 3, 7, 11, 17];
        bwt.bwtCode = vec![0x0123_4567, 0x89ab_cdef, 0xffff_ffff];
        let path = std::env::temp_dir().join("minibwa-rs-test.bwt");
        BWTSaveBwtCodeAndOcc(&bwt, &path, None).expect("save BWT");
        let bytes = std::fs::read(&path).expect("read BWT");
        let _ = std::fs::remove_file(&path);

        let mut expected = Vec::new();
        expected.extend_from_slice(&9u64.to_le_bytes());
        for value in [3u64, 7, 11, 17] {
            expected.extend_from_slice(&value.to_le_bytes());
        }
        expected.extend_from_slice(&0x0123_4567u32.to_le_bytes());
        expected.extend_from_slice(&0x89ab_cdefu32.to_le_bytes());
        assert_eq!(bytes, expected);
    }

    #[test]
    fn bwt_inc_free_accepts_null_and_owned_state() {
        BWTIncFree(None);
        BWTIncFree(Some(BWTIncCreate(64, 16, 16)));
    }

    #[test]
    fn construct_from_packed_and_mb_bwtgen_handle_small_pac_file() {
        let bases: Vec<u8> = (0..32).map(|i| ((i * 5 + 1) & 3) as u8).collect();
        let mut packed =
            vec![0u8; (bases.len() + CHAR_PER_BYTE as usize - 1) / CHAR_PER_BYTE as usize];
        for (i, &base) in bases.iter().enumerate() {
            packed[i / CHAR_PER_BYTE as usize] |=
                base << (BITS_IN_BYTE - ((i % CHAR_PER_BYTE as usize) as u32 + 1) * BIT_PER_CHAR);
        }
        let last_len = (bases.len() % CHAR_PER_BYTE as usize) as u8;
        if last_len == 0 {
            packed.push(0);
        }
        packed.push(last_len);

        let pac_path = std::env::temp_dir().join(format!(
            "minibwa-rs-bwtgen-small-{}.pac",
            std::process::id()
        ));
        let bwt_path = std::env::temp_dir().join(format!(
            "minibwa-rs-bwtgen-small-{}.bwt",
            std::process::id()
        ));
        std::fs::write(&pac_path, &packed).expect("write pac");

        let bwt_inc = BWTIncConstructFromPacked(&pac_path, 64, 64).expect("construct BWT");
        assert_eq!(bwt_inc.bwt.textLength, bases.len() as u64);
        let mut counts = [0u64; ALPHABET_SIZE];
        for &base in &bases {
            counts[base as usize] += 1;
        }
        assert_eq!(bwt_inc.bwt.cumulativeFreq[1], counts[0]);
        assert_eq!(bwt_inc.bwt.cumulativeFreq[2], counts[0] + counts[1]);
        assert_eq!(
            bwt_inc.bwt.cumulativeFreq[3],
            counts[0] + counts[1] + counts[2]
        );
        assert_eq!(bwt_inc.bwt.cumulativeFreq[4], bases.len() as u64);
        assert!(bwt_inc.bwt.inverseSa0 <= bases.len() as u64);

        mb_bwtgen(&pac_path, &bwt_path, 64).expect("save generated BWT");
        let saved = std::fs::read(&bwt_path).expect("read generated BWT");
        let _ = std::fs::remove_file(&pac_path);
        let _ = std::fs::remove_file(&bwt_path);
        assert_eq!(
            saved.len(),
            std::mem::size_of::<bgint_t>() * (1 + ALPHABET_SIZE)
                + BWTFileSizeInWord(bases.len() as u64) as usize * BYTES_IN_WORD
        );
    }

    #[test]
    fn public_file_helpers_stop_paths_at_embedded_nul_like_c() {
        let bases: Vec<u8> = (0..32).map(|i| ((i * 5 + 1) & 3) as u8).collect();
        let mut packed =
            vec![0u8; (bases.len() + CHAR_PER_BYTE as usize - 1) / CHAR_PER_BYTE as usize];
        for (i, &base) in bases.iter().enumerate() {
            packed[i / CHAR_PER_BYTE as usize] |=
                base << (BITS_IN_BYTE - ((i % CHAR_PER_BYTE as usize) as u32 + 1) * BIT_PER_CHAR);
        }
        let last_len = (bases.len() % CHAR_PER_BYTE as usize) as u8;
        if last_len == 0 {
            packed.push(0);
        }
        packed.push(last_len);

        let dir =
            std::env::temp_dir().join(format!("minibwa-rs-bwtgen-api-nul-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let pac = dir.join("in.pac");
        let raw = dir.join("out.raw");
        std::fs::write(&pac, &packed).expect("write pac");
        let pac_arg = std::path::PathBuf::from(format!("{}\0hidden", pac.to_string_lossy()));
        let raw_arg = std::path::PathBuf::from(format!("{}\0hidden", raw.to_string_lossy()));

        let bwt_inc = BWTIncConstructFromPacked(&pac_arg, 64, 64).expect("construct BWT");
        assert_eq!(bwt_inc.bwt.textLength, bases.len() as u64);
        BWTSaveBwtCodeAndOcc(&bwt_inc.bwt, &raw_arg, None).expect("save BWT");
        assert!(raw.exists());

        let raw2 = dir.join("out2.raw");
        let raw2_arg = std::path::PathBuf::from(format!("{}\0hidden", raw2.to_string_lossy()));
        mb_bwtgen(&pac_arg, &raw2_arg, 64).expect("generate BWT");
        assert!(raw2.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}

/// Original C static function `ForwardDNAAllOccCountNoLimit` from `minibwa/bwtgen.c:426`.
pub fn ForwardDNAAllOccCountNoLimit(
    dna: &[u32],
    index: bgint_t,
    occCount: &mut [bgint_t],
    dnaDecodeTable: &[u32],
) {
    const TRUNCATE_RIGHT_MASK: [u32; 16] = [
        0x00000000, 0xC0000000, 0xF0000000, 0xFC000000, 0xFF000000, 0xFFC00000, 0xFFF00000,
        0xFFFC0000, 0xFFFF0000, 0xFFFFC000, 0xFFFFF000, 0xFFFFFC00, 0xFFFFFF00, 0xFFFFFFC0,
        0xFFFFFFF0, 0xFFFFFFFC,
    ];

    occCount[0] = 0;
    occCount[1] = 0;
    occCount[2] = 0;
    occCount[3] = 0;

    let iteration = index / 256;
    let wordToCount = (index - iteration * 256) / 16;
    let charToCount = index - iteration * 256 - wordToCount * 16;
    let mut dna_i = 0usize;

    for _ in 0..iteration {
        let mut sum = 0u32;
        for _ in 0..16 {
            let word = dna[dna_i];
            sum = sum.wrapping_add(dnaDecodeTable[(word >> 16) as usize]);
            sum = sum.wrapping_add(dnaDecodeTable[(word & 0x0000ffff) as usize]);
            dna_i += 1;
        }
        if (sum & 0xfefefeff) != 0 {
            occCount[0] += (sum & 0x000000ff) as bgint_t;
            sum >>= 8;
            occCount[1] += (sum & 0x000000ff) as bgint_t;
            sum >>= 8;
            occCount[2] += (sum & 0x000000ff) as bgint_t;
            sum >>= 8;
            occCount[3] += sum as bgint_t;
        } else if sum == 0x00000100 {
            occCount[0] += 256;
        } else if sum == 0x00010000 {
            occCount[1] += 256;
        } else if sum == 0x01000000 {
            occCount[2] += 256;
        } else if sum == 0x00000000 {
            occCount[3] += 256;
        } else {
            eprintln!("ForwardDNAAllOccCountNoLimit(): DNA occ sum exception!");
            std::process::exit(1);
        }
    }

    let mut sum = 0u32;
    for _ in 0..wordToCount {
        let word = dna[dna_i];
        sum = sum.wrapping_add(dnaDecodeTable[(word >> 16) as usize]);
        sum = sum.wrapping_add(dnaDecodeTable[(word & 0x0000ffff) as usize]);
        dna_i += 1;
    }

    if charToCount > 0 {
        let c = dna[dna_i] & TRUNCATE_RIGHT_MASK[charToCount as usize];
        sum = sum.wrapping_add(dnaDecodeTable[(c >> 16) as usize]);
        sum = sum.wrapping_add(dnaDecodeTable[(c & 0xffff) as usize]);
        sum = sum.wrapping_add(charToCount as u32).wrapping_sub(16);
    }

    occCount[0] += (sum & 0x000000ff) as bgint_t;
    sum >>= 8;
    occCount[1] += (sum & 0x000000ff) as bgint_t;
    sum >>= 8;
    occCount[2] += (sum & 0x000000ff) as bgint_t;
    sum >>= 8;
    occCount[3] += sum as bgint_t;
}

/// Original C static function `BWTIncBuildPackedBwt` from `minibwa/bwtgen.c:499`.
pub fn BWTIncBuildPackedBwt(
    relativeRank: &[bgint_t],
    bwt: &mut [u32],
    numChar: bgint_t,
    cumulativeCount: &[bgint_t],
    packedShift: &[u32],
) {
    let inverseSa0 = relativeRank[0];
    let mut previousRank = relativeRank[0];
    for i in 1..=numChar as usize {
        let currentRank = relativeRank[i];
        let c = ((previousRank > cumulativeCount[1]) as u32)
            + ((previousRank > cumulativeCount[2]) as u32)
            + ((previousRank > cumulativeCount[3]) as u32);
        if c > 0 {
            let mut r = currentRank;
            if r > inverseSa0 {
                r -= 1;
            }
            let wordIndex = r / CHAR_PER_WORD;
            let charIndex = r - wordIndex * CHAR_PER_WORD;
            bwt[wordIndex as usize] |= c << packedShift[charIndex as usize];
        }
        previousRank = currentRank;
    }
}

/// Original C static function `BWTOccValueExplicit` from `minibwa/bwtgen.c:531`.
pub fn BWTOccValueExplicit(bwt: &BWT, occIndexExplicit: bgint_t, character: u32) -> bgint_t {
    let occIndexMajor = occIndexExplicit * OCC_INTERVAL / OCC_INTERVAL_MAJOR;
    if occIndexExplicit % OCC_VALUE_PER_WORD == 0 {
        bwt.occValueMajor[(occIndexMajor * ALPHABET_SIZE as u64 + character as u64) as usize]
            + (bwt.occValue[(occIndexExplicit / OCC_VALUE_PER_WORD * ALPHABET_SIZE as u64
                + character as u64) as usize]
                >> 16) as u64
    } else {
        bwt.occValueMajor[(occIndexMajor * ALPHABET_SIZE as u64 + character as u64) as usize]
            + (bwt.occValue[(occIndexExplicit / OCC_VALUE_PER_WORD * ALPHABET_SIZE as u64
                + character as u64) as usize]
                & 0xffff) as u64
    }
}

#[inline]
unsafe fn BWTOccValueExplicit_ptr(bwt: &BWT, occIndexExplicit: bgint_t, character: u32) -> bgint_t {
    let occIndexMajor = occIndexExplicit * OCC_INTERVAL / OCC_INTERVAL_MAJOR;
    let major = bwt.occValueMajor.as_ptr();
    let minor = bwt.occValue.as_ptr();
    let major_idx = occIndexMajor * ALPHABET_SIZE as u64 + character as u64;
    let minor_idx = occIndexExplicit / OCC_VALUE_PER_WORD * ALPHABET_SIZE as u64 + character as u64;
    let packed = unsafe { *minor.add(minor_idx as usize) };
    unsafe {
        *major.add(major_idx as usize)
            + if occIndexExplicit % OCC_VALUE_PER_WORD == 0 {
                (packed >> 16) as u64
            } else {
                (packed & 0xffff) as u64
            }
    }
}

/// Original C static function `ForwardDNAOccCount` from `minibwa/bwtgen.c:549`.
pub fn ForwardDNAOccCount(dna: &[u32], index: u32, character: u32, dnaDecodeTable: &[u32]) -> u32 {
    const TRUNCATE_RIGHT_MASK: [u32; 16] = [
        0x00000000, 0xC0000000, 0xF0000000, 0xFC000000, 0xFF000000, 0xFFC00000, 0xFFF00000,
        0xFFFC0000, 0xFFFF0000, 0xFFFFC000, 0xFFFFF000, 0xFFFFFC00, 0xFFFFFF00, 0xFFFFFFC0,
        0xFFFFFFF0, 0xFFFFFFFC,
    ];

    let wordToCount = index / 16;
    let charToCount = index - wordToCount * 16;
    let mut sum = 0u32;

    for i in 0..wordToCount as usize {
        sum = sum.wrapping_add(dnaDecodeTable[(dna[i] >> 16) as usize]);
        sum = sum.wrapping_add(dnaDecodeTable[(dna[i] & 0x0000ffff) as usize]);
    }

    if charToCount > 0 {
        let c = dna[wordToCount as usize] & TRUNCATE_RIGHT_MASK[charToCount as usize];
        sum = sum.wrapping_add(dnaDecodeTable[(c >> 16) as usize]);
        sum = sum.wrapping_add(dnaDecodeTable[(c & 0xffff) as usize]);
        sum = sum.wrapping_add(charToCount).wrapping_sub(16);
    }

    (sum >> (character * 8)) & 0x000000ff
}

#[inline]
unsafe fn ForwardDNAOccCount_ptr(
    dna: *const u32,
    index: u32,
    character: u32,
    dnaDecodeTable: *const u32,
) -> u32 {
    const TRUNCATE_RIGHT_MASK: [u32; 16] = [
        0x00000000, 0xC0000000, 0xF0000000, 0xFC000000, 0xFF000000, 0xFFC00000, 0xFFF00000,
        0xFFFC0000, 0xFFFF0000, 0xFFFFC000, 0xFFFFF000, 0xFFFFFC00, 0xFFFFFF00, 0xFFFFFFC0,
        0xFFFFFFF0, 0xFFFFFFFC,
    ];

    let wordToCount = index / 16;
    let charToCount = index - wordToCount * 16;
    let mut sum = 0u32;

    for i in 0..wordToCount as usize {
        let word = unsafe { *dna.add(i) };
        sum = sum.wrapping_add(unsafe { *dnaDecodeTable.add((word >> 16) as usize) });
        sum = sum.wrapping_add(unsafe { *dnaDecodeTable.add((word & 0x0000ffff) as usize) });
    }

    if charToCount > 0 {
        let c = unsafe { *dna.add(wordToCount as usize) }
            & unsafe { *TRUNCATE_RIGHT_MASK.as_ptr().add(charToCount as usize) };
        sum = sum.wrapping_add(unsafe { *dnaDecodeTable.add((c >> 16) as usize) });
        sum = sum.wrapping_add(unsafe { *dnaDecodeTable.add((c & 0xffff) as usize) });
        sum = sum.wrapping_add(charToCount).wrapping_sub(16);
    }

    (sum >> (character * 8)) & 0x000000ff
}

/// Original C static function `BackwardDNAOccCount` from `minibwa/bwtgen.c:580`.
pub fn BackwardDNAOccCount(dna: &[u32], index: u32, character: u32, dnaDecodeTable: &[u32]) -> u32 {
    const TRUNCATE_LEFT_MASK: [u32; 16] = [
        0x00000000, 0x00000003, 0x0000000f, 0x0000003f, 0x000000ff, 0x000003ff, 0x00000fff,
        0x00003fff, 0x0000ffff, 0x0003ffff, 0x000fffff, 0x003fffff, 0x00ffffff, 0x03ffffff,
        0x0fffffff, 0x3fffffff,
    ];

    let wordToCount = index / 16;
    let charToCount = index - wordToCount * 16;
    let mut sum = 0u32;
    let mut pos = dna.len() - wordToCount as usize - 1;

    if charToCount > 0 {
        let c = dna[pos] & TRUNCATE_LEFT_MASK[charToCount as usize];
        sum = sum.wrapping_add(dnaDecodeTable[(c >> 16) as usize]);
        sum = sum.wrapping_add(dnaDecodeTable[(c & 0xffff) as usize]);
        sum = sum.wrapping_add(charToCount).wrapping_sub(16);
    }

    for _ in 0..wordToCount {
        pos += 1;
        sum = sum.wrapping_add(dnaDecodeTable[(dna[pos] >> 16) as usize]);
        sum = sum.wrapping_add(dnaDecodeTable[(dna[pos] & 0x0000ffff) as usize]);
    }

    (sum >> (character * 8)) & 0x000000ff
}

#[inline]
unsafe fn BackwardDNAOccCount_ptr(
    dna: *const u32,
    dna_len: usize,
    index: u32,
    character: u32,
    dnaDecodeTable: *const u32,
) -> u32 {
    const TRUNCATE_LEFT_MASK: [u32; 16] = [
        0x00000000, 0x00000003, 0x0000000f, 0x0000003f, 0x000000ff, 0x000003ff, 0x00000fff,
        0x00003fff, 0x0000ffff, 0x0003ffff, 0x000fffff, 0x003fffff, 0x00ffffff, 0x03ffffff,
        0x0fffffff, 0x3fffffff,
    ];

    let wordToCount = index / 16;
    let charToCount = index - wordToCount * 16;
    let mut sum = 0u32;
    let mut pos = dna_len - wordToCount as usize - 1;

    if charToCount > 0 {
        let c = unsafe { *dna.add(pos) }
            & unsafe { *TRUNCATE_LEFT_MASK.as_ptr().add(charToCount as usize) };
        sum = sum.wrapping_add(unsafe { *dnaDecodeTable.add((c >> 16) as usize) });
        sum = sum.wrapping_add(unsafe { *dnaDecodeTable.add((c & 0xffff) as usize) });
        sum = sum.wrapping_add(charToCount).wrapping_sub(16);
    }

    for _ in 0..wordToCount {
        pos += 1;
        let word = unsafe { *dna.add(pos) };
        sum = sum.wrapping_add(unsafe { *dnaDecodeTable.add((word >> 16) as usize) });
        sum = sum.wrapping_add(unsafe { *dnaDecodeTable.add((word & 0x0000ffff) as usize) });
    }

    (sum >> (character * 8)) & 0x000000ff
}

/// Original C global function `BWTOccValue` from `minibwa/bwtgen.c:614`.
pub fn BWTOccValue(bwt: &BWT, mut index: bgint_t, character: u32) -> bgint_t {
    if index > bwt.inverseSa0 {
        index -= 1;
    }

    let occExplicitIndex = (index + OCC_INTERVAL / 2 - 1) / OCC_INTERVAL;
    let occIndex = occExplicitIndex * OCC_INTERVAL;
    let occValue = BWTOccValueExplicit(bwt, occExplicitIndex, character);

    if occIndex == index {
        return occValue;
    }

    let wordIndex = (occIndex / CHAR_PER_WORD) as usize;
    if occIndex < index {
        occValue
            + ForwardDNAOccCount(
                &bwt.bwtCode[wordIndex..],
                (index - occIndex) as u32,
                character,
                &bwt.decodeTable,
            ) as u64
    } else {
        occValue
            - BackwardDNAOccCount(
                &bwt.bwtCode[..wordIndex],
                (occIndex - index) as u32,
                character,
                &bwt.decodeTable,
            ) as u64
    }
}

#[inline]
unsafe fn BWTOccValue_ptr(bwt: &BWT, mut index: bgint_t, character: u32) -> bgint_t {
    if index > bwt.inverseSa0 {
        index -= 1;
    }

    let occExplicitIndex = (index + OCC_INTERVAL / 2 - 1) / OCC_INTERVAL;
    let occIndex = occExplicitIndex * OCC_INTERVAL;
    let occValue = unsafe { BWTOccValueExplicit_ptr(bwt, occExplicitIndex, character) };

    if occIndex == index {
        return occValue;
    }

    let wordIndex = (occIndex / CHAR_PER_WORD) as usize;
    let bwt_code = bwt.bwtCode.as_ptr();
    let decode = bwt.decodeTable.as_ptr();
    if occIndex < index {
        occValue
            + unsafe {
                ForwardDNAOccCount_ptr(
                    bwt_code.add(wordIndex),
                    (index - occIndex) as u32,
                    character,
                    decode,
                )
            } as u64
    } else {
        occValue
            - unsafe {
                BackwardDNAOccCount_ptr(
                    bwt_code,
                    wordIndex,
                    (occIndex - index) as u32,
                    character,
                    decode,
                )
            } as u64
    }
}

/// Original C static function `BWTIncGetAbsoluteRank` from `minibwa/bwtgen.c:638`.
pub fn BWTIncGetAbsoluteRank(
    bwt: &BWT,
    absoluteRank: &mut [bgint_t],
    seq: &mut [bgint_t],
    packedText: &[u32],
    numChar: bgint_t,
    cumulativeCount: &[bgint_t],
    firstCharInLastIteration: u32,
) -> bgint_t {
    unsafe {
        BWTIncGetAbsoluteRank_ptr(
            bwt,
            absoluteRank.as_mut_ptr(),
            seq.as_mut_ptr(),
            packedText.as_ptr(),
            numChar,
            cumulativeCount.as_ptr(),
            firstCharInLastIteration,
        )
    }
}

unsafe fn BWTIncGetAbsoluteRank_ptr(
    bwt: &BWT,
    absoluteRank: *mut bgint_t,
    seq: *mut bgint_t,
    packedText: *const u32,
    numChar: bgint_t,
    cumulativeCount: *const bgint_t,
    firstCharInLastIteration: u32,
) -> bgint_t {
    let mut seqIndexFromStart = [0u64; ALPHABET_SIZE];
    let mut seqIndexFromEnd = [0u64; ALPHABET_SIZE];
    for i in 0..ALPHABET_SIZE {
        seqIndexFromStart[i] = unsafe { *cumulativeCount.add(i) };
        seqIndexFromEnd[i] = unsafe { *cumulativeCount.add(i + 1) } - 1;
    }

    let shift = BITS_IN_WORD - BIT_PER_CHAR;
    let packedMask = ALL_ONE_MASK >> shift;
    let mut saIndex = bwt.inverseSa0;
    let mut rankIndex = numChar - 1;
    let lastWord = numChar / CHAR_PER_WORD;

    for i in (0..lastWord as usize).rev() {
        let mut t = unsafe { *packedText.add(i) };
        for _ in 0..CHAR_PER_WORD {
            let c = t & packedMask;
            saIndex = unsafe { *bwt.cumulativeFreq.as_ptr().add(c as usize) }
                + unsafe { BWTOccValue_ptr(bwt, saIndex, c) }
                + 1;
            if saIndex > bwt.inverseSa0 {
                let idx = seqIndexFromEnd[c as usize] as usize;
                unsafe {
                    *seq.add(idx) = rankIndex;
                    *absoluteRank.add(idx) = saIndex;
                }
                seqIndexFromEnd[c as usize] -= 1;
            } else {
                let idx = seqIndexFromStart[c as usize] as usize;
                unsafe {
                    *seq.add(idx) = rankIndex;
                    *absoluteRank.add(idx) = saIndex;
                }
                seqIndexFromStart[c as usize] += 1;
            }
            rankIndex = rankIndex.wrapping_sub(1);
            t >>= BIT_PER_CHAR;
        }
    }

    let idx = seqIndexFromStart[firstCharInLastIteration as usize] as usize;
    unsafe {
        *absoluteRank.add(idx) = bwt.inverseSa0;
        *seq.add(idx) = numChar;
    }
    seqIndexFromStart[firstCharInLastIteration as usize]
}

/// Original C static function `BWTIncSortKey` from `minibwa/bwtgen.c:690`.
pub fn BWTIncSortKey(key: &mut [bgint_t], seq: &mut [bgint_t], numItem: bgint_t) {
    unsafe { BWTIncSortKey_ptr(key.as_mut_ptr(), seq.as_mut_ptr(), numItem) }
}

#[inline]
unsafe fn swap_key_seq(key: *mut bgint_t, seq: *mut bgint_t, a: usize, b: usize) {
    let temp_seq = unsafe { *seq.add(a) };
    let temp_key = unsafe { *key.add(a) };
    unsafe {
        *seq.add(a) = *seq.add(b);
        *key.add(a) = *key.add(b);
        *seq.add(b) = temp_seq;
        *key.add(b) = temp_key;
    }
}

unsafe fn BWTIncSortKey_ptr(key: *mut bgint_t, seq: *mut bgint_t, numItem: bgint_t) {
    const INSERT_SORT_NUM_ITEM: usize = BWTINC_INSERT_SORT_NUM_ITEM as usize;
    const EQUAL_KEY_THRESHOLD: usize = 4;
    if numItem < 2 {
        return;
    }

    let mut lowIndex = 0usize;
    let mut highIndex = numItem as usize - 1;
    let mut lowStack = [0usize; 32];
    let mut highStack = [0usize; 32];
    let mut stackDepth = 0usize;

    loop {
        loop {
            if highIndex - lowIndex < INSERT_SORT_NUM_ITEM {
                for i in lowIndex + 1..=highIndex {
                    let tempSeq = unsafe { *seq.add(i) };
                    let tempKey = unsafe { *key.add(i) };
                    let mut j = i;
                    while j > lowIndex && unsafe { *key.add(j - 1) } > tempKey {
                        unsafe {
                            *seq.add(j) = *seq.add(j - 1);
                            *key.add(j) = *key.add(j - 1);
                        }
                        j -= 1;
                    }
                    if j != i {
                        unsafe {
                            *seq.add(j) = tempSeq;
                            *key.add(j) = tempKey;
                        }
                    }
                }
                break;
            }

            let mut midIndex = (lowIndex & highIndex) + ((lowIndex ^ highIndex) / 2);
            if unsafe { *key.add(lowIndex) > *key.add(midIndex) } {
                unsafe { swap_key_seq(key, seq, lowIndex, midIndex) };
            }
            if unsafe { *key.add(lowIndex) > *key.add(highIndex) } {
                unsafe { swap_key_seq(key, seq, lowIndex, highIndex) };
            }
            if unsafe { *key.add(midIndex) > *key.add(highIndex) } {
                unsafe { swap_key_seq(key, seq, midIndex, highIndex) };
            }

            let mut numberOfEqualKey = 0usize;
            let mut lowPartitionIndex = lowIndex + 1;
            let mut highPartitionIndex = highIndex - 1;

            loop {
                while lowPartitionIndex <= highPartitionIndex
                    && unsafe { *key.add(lowPartitionIndex) <= *key.add(midIndex) }
                {
                    numberOfEqualKey +=
                        unsafe { (*key.add(lowPartitionIndex) == *key.add(midIndex)) as usize };
                    lowPartitionIndex += 1;
                }
                while lowPartitionIndex < highPartitionIndex {
                    if unsafe { *key.add(midIndex) >= *key.add(highPartitionIndex) } {
                        numberOfEqualKey += unsafe {
                            (*key.add(midIndex) == *key.add(highPartitionIndex)) as usize
                        };
                        break;
                    }
                    highPartitionIndex -= 1;
                }
                if lowPartitionIndex >= highPartitionIndex {
                    break;
                }
                unsafe { swap_key_seq(key, seq, lowPartitionIndex, highPartitionIndex) };
                if highPartitionIndex == midIndex {
                    midIndex = lowPartitionIndex;
                }
                lowPartitionIndex += 1;
                highPartitionIndex -= 1;
            }

            highPartitionIndex = lowPartitionIndex;
            lowPartitionIndex -= 1;

            unsafe { swap_key_seq(key, seq, midIndex, lowPartitionIndex) };

            if highIndex - lowIndex + INSERT_SORT_NUM_ITEM <= EQUAL_KEY_THRESHOLD * numberOfEqualKey
            {
                midIndex = lowIndex;
                loop {
                    while midIndex < lowPartitionIndex
                        && unsafe { *key.add(midIndex) < *key.add(lowPartitionIndex) }
                    {
                        midIndex += 1;
                    }
                    while midIndex < lowPartitionIndex
                        && unsafe { *key.add(lowPartitionIndex) == *key.add(lowPartitionIndex - 1) }
                    {
                        lowPartitionIndex -= 1;
                    }
                    if midIndex >= lowPartitionIndex {
                        break;
                    }
                    unsafe { swap_key_seq(key, seq, midIndex, lowPartitionIndex - 1) };
                    midIndex += 1;
                    lowPartitionIndex -= 1;
                }
            }

            if lowPartitionIndex - lowIndex > highIndex - highPartitionIndex {
                lowStack[stackDepth] = lowIndex;
                highStack[stackDepth] = lowPartitionIndex - 1;
                stackDepth += 1;
                lowIndex = highPartitionIndex;
            } else {
                lowStack[stackDepth] = highPartitionIndex;
                highStack[stackDepth] = highIndex;
                stackDepth += 1;
                if lowPartitionIndex > lowIndex {
                    highIndex = lowPartitionIndex - 1;
                } else {
                    break;
                }
            }
        }

        if stackDepth > 0 {
            stackDepth -= 1;
            lowIndex = lowStack[stackDepth];
            highIndex = highStack[stackDepth];
        } else {
            return;
        }
    }
}

/// Original C static function `BWTIncBuildRelativeRank` from `minibwa/bwtgen.c:868`.
pub fn BWTIncBuildRelativeRank(
    sortedRank: &mut [bgint_t],
    seq: &mut [bgint_t],
    relativeRank: &mut [bgint_t],
    numItem: bgint_t,
    oldInverseSa0: bgint_t,
    cumulativeCount: &[bgint_t],
) {
    unsafe {
        BWTIncBuildRelativeRank_ptr(
            sortedRank.as_mut_ptr(),
            seq.as_mut_ptr(),
            relativeRank.as_mut_ptr(),
            numItem,
            oldInverseSa0,
            cumulativeCount.as_ptr(),
        )
    }
}

unsafe fn BWTIncBuildRelativeRank_ptr(
    sortedRank: *mut bgint_t,
    seq: *mut bgint_t,
    relativeRank: *mut bgint_t,
    numItem: bgint_t,
    mut oldInverseSa0: bgint_t,
    cumulativeCount: *const bgint_t,
) {
    let mut lastIndex = numItem;
    let mut lastRank = unsafe { *sortedRank.add(numItem as usize) };
    if lastRank > oldInverseSa0 {
        unsafe { *sortedRank.add(numItem as usize) -= 1 };
    }
    let mut s = unsafe { *seq.add(numItem as usize) };
    unsafe { *relativeRank.add(s as usize) = numItem };
    if lastRank == oldInverseSa0 {
        oldInverseSa0 += 1;
        lastRank += 1;
    }

    let mut c = ALPHABET_SIZE as u64 - 1;
    let mut freq = unsafe { *cumulativeCount.add(c as usize) };
    for i in (0..numItem).rev() {
        let r = unsafe { *sortedRank.add(i as usize) };
        if r > oldInverseSa0 {
            unsafe { *sortedRank.add(i as usize) -= 1 };
        }
        s = unsafe { *seq.add(i as usize) };
        if i < freq {
            if lastIndex >= freq {
                lastRank += 1;
            }
            c -= 1;
            freq = unsafe { *cumulativeCount.add(c as usize) };
        }
        if r == lastRank {
            unsafe { *relativeRank.add(s as usize) = lastIndex };
        } else {
            if i == lastIndex - 1 {
                if lastIndex < numItem
                    && (unsafe { *seq.add((lastIndex + 1) as usize) } as sbgint_t) < 0
                {
                    unsafe {
                        *seq.add(lastIndex as usize) =
                            (*seq.add((lastIndex + 1) as usize)).wrapping_sub(1);
                    }
                } else {
                    unsafe { *seq.add(lastIndex as usize) = (-1i64) as bgint_t };
                }
            }
            lastIndex = i;
            lastRank = r;
            unsafe { *relativeRank.add(s as usize) = i };
            if r == oldInverseSa0 {
                oldInverseSa0 += 1;
                lastRank += 1;
            }
        }
    }
}

/// Original C static function `BWTIncBuildBwt` from `minibwa/bwtgen.c:925`.
pub fn BWTIncBuildBwt(
    insertBwt: &mut [u32],
    relativeRank: &[bgint_t],
    numChar: bgint_t,
    cumulativeCount: &[bgint_t],
) {
    let mut previousRank = relativeRank[0];
    for i in 1..=numChar as usize {
        let currentRank = relativeRank[i];
        let c = ((previousRank >= cumulativeCount[1]) as u32)
            + ((previousRank >= cumulativeCount[2]) as u32)
            + ((previousRank >= cumulativeCount[3]) as u32);
        insertBwt[currentRank as usize] = c;
        previousRank = currentRank;
    }
}

/// Original C static function `BWTIncMergeBwt` from `minibwa/bwtgen.c:943`.
#[allow(unused_assignments)]
pub fn BWTIncMergeBwt(
    sortedRank: &[bgint_t],
    oldBwt: &[u32],
    insertBwt: &[u32],
    mergedBwt: &mut [u32],
    numOldBwt: bgint_t,
    numInsertBwt: bgint_t,
) {
    unsafe {
        BWTIncMergeBwt_ptr(
            sortedRank.as_ptr(),
            oldBwt.as_ptr(),
            insertBwt.as_ptr(),
            mergedBwt.as_mut_ptr(),
            numOldBwt,
            numInsertBwt,
        )
    }
}

unsafe fn BWTIncMergeBwt_ptr(
    sortedRank: *const bgint_t,
    oldBwt: *const u32,
    insertBwt: *const u32,
    mergedBwt: *mut u32,
    numOldBwt: bgint_t,
    numInsertBwt: bgint_t,
) {
    let mut oIndex = 0u64;
    let mut iIndex = 0u64;
    let mut mIndex = 0u64;
    let mut mWord = 0u64;
    let mut mChar = 0u64;
    unsafe { *mergedBwt = 0 };

    while oIndex < numOldBwt {
        while iIndex <= numInsertBwt && unsafe { *sortedRank.add(iIndex as usize) } <= oIndex {
            let rank = unsafe { *sortedRank.add(iIndex as usize) };
            if rank != 0 {
                unsafe {
                    *mergedBwt.add(mWord as usize) |= *insertBwt.add(iIndex as usize)
                        << (BITS_IN_WORD - (mChar as u32 + 1) * BIT_PER_CHAR);
                }
                mIndex += 1;
                mChar += 1;
                if mChar == CHAR_PER_WORD {
                    mChar = 0;
                    mWord += 1;
                    unsafe { *mergedBwt.add(mWord as usize) = 0 };
                }
            }
            iIndex += 1;
        }

        let o = if iIndex <= numInsertBwt {
            unsafe { *sortedRank.add(iIndex as usize) }
        } else {
            numOldBwt
        };
        let numInsert = o - oIndex;
        let mut oWord = oIndex / CHAR_PER_WORD;
        let oChar = oIndex - oWord * CHAR_PER_WORD;

        if oChar > mChar {
            let leftShift = ((oChar - mChar) as u32) * BIT_PER_CHAR;
            let rightShift = ((CHAR_PER_WORD + mChar - oChar) as u32) * BIT_PER_CHAR;
            unsafe {
                *mergedBwt.add(mWord as usize) |= (*oldBwt.add(oWord as usize)
                    << (oChar as u32 * BIT_PER_CHAR)
                    >> (mChar as u32 * BIT_PER_CHAR))
                    | (*oldBwt.add(oWord as usize + 1) >> rightShift);
            }
            oIndex += numInsert.min(CHAR_PER_WORD - mChar);
            while o > oIndex {
                oWord += 1;
                mWord += 1;
                unsafe {
                    *mergedBwt.add(mWord as usize) = (*oldBwt.add(oWord as usize) << leftShift)
                        | (*oldBwt.add(oWord as usize + 1) >> rightShift);
                }
                oIndex += CHAR_PER_WORD;
            }
        } else if oChar < mChar {
            let rightShift = ((mChar - oChar) as u32) * BIT_PER_CHAR;
            let leftShift = ((CHAR_PER_WORD + oChar - mChar) as u32) * BIT_PER_CHAR;
            unsafe {
                *mergedBwt.add(mWord as usize) |= *oldBwt.add(oWord as usize)
                    << (oChar as u32 * BIT_PER_CHAR)
                    >> (mChar as u32 * BIT_PER_CHAR);
            }
            oIndex += numInsert.min(CHAR_PER_WORD - mChar);
            while o > oIndex {
                oWord += 1;
                mWord += 1;
                unsafe {
                    *mergedBwt.add(mWord as usize) = (*oldBwt.add(oWord as usize - 1) << leftShift)
                        | (*oldBwt.add(oWord as usize) >> rightShift);
                }
                oIndex += CHAR_PER_WORD;
            }
        } else {
            unsafe {
                *mergedBwt.add(mWord as usize) |= *oldBwt.add(oWord as usize)
                    << (mChar as u32 * BIT_PER_CHAR)
                    >> (mChar as u32 * BIT_PER_CHAR);
            }
            oIndex += numInsert.min(CHAR_PER_WORD - mChar);
            while o > oIndex {
                oWord += 1;
                mWord += 1;
                unsafe { *mergedBwt.add(mWord as usize) = *oldBwt.add(oWord as usize) };
                oIndex += CHAR_PER_WORD;
            }
        }

        oIndex = o;
        mIndex += numInsert;
        mWord = mIndex / CHAR_PER_WORD;
        mChar = mIndex - mWord * CHAR_PER_WORD;
        if mChar == 0 {
            unsafe { *mergedBwt.add(mWord as usize) = 0 };
        } else {
            let offset = BITS_IN_WORD - mChar as u32 * BIT_PER_CHAR;
            unsafe {
                let word = mergedBwt.add(mWord as usize);
                *word = *word >> offset << offset;
            }
        }
    }

    while iIndex <= numInsertBwt {
        if unsafe { *sortedRank.add(iIndex as usize) } != 0 {
            unsafe {
                *mergedBwt.add(mWord as usize) |= *insertBwt.add(iIndex as usize)
                    << (BITS_IN_WORD - (mChar as u32 + 1) * BIT_PER_CHAR);
            }
            mChar += 1;
            if mChar == CHAR_PER_WORD {
                mChar = 0;
                mWord += 1;
                unsafe { *mergedBwt.add(mWord as usize) = 0 };
            }
        }
        iIndex += 1;
    }
}

/// Original C global function `BWTClearTrailingBwtCode` from `minibwa/bwtgen.c:1053`.
pub fn BWTClearTrailingBwtCode(bwt: &mut BWT) {
    let bwtResidentSizeInWord = BWTResidentSizeInWord(bwt.textLength);
    let wordIndex = bwt.textLength / CHAR_PER_WORD;
    let offset = (bwt.textLength - wordIndex * CHAR_PER_WORD) as u32 * BIT_PER_CHAR;
    if offset > 0 {
        let rshift = BITS_IN_WORD - offset;
        bwt.bwtCode[wordIndex as usize] = bwt.bwtCode[wordIndex as usize] >> rshift << rshift;
    } else if wordIndex < bwtResidentSizeInWord {
        bwt.bwtCode[wordIndex as usize] = 0;
    }
    for i in wordIndex + 1..bwtResidentSizeInWord {
        bwt.bwtCode[i as usize] = 0;
    }
}

/// Original C global function `BWTGenerateOccValueFromBwt` from `minibwa/bwtgen.c:1077`.
pub fn BWTGenerateOccValueFromBwt(
    bwt: &[u32],
    occValue: &mut [u32],
    occValueMajor: &mut [bgint_t],
    textLength: bgint_t,
    decodeTable: &[u32],
) {
    let wordBetweenOccValue = OCC_INTERVAL / CHAR_PER_WORD;
    let numberOfOccValue = (textLength + OCC_INTERVAL - 1) / OCC_INTERVAL + 1;
    let numberOfOccIntervalPerMajor = OCC_INTERVAL_MAJOR / OCC_INTERVAL;
    let numberOfOccValueMajor =
        (numberOfOccValue + numberOfOccIntervalPerMajor - 1) / numberOfOccIntervalPerMajor;

    let mut tempOccValue0 = [0u64; ALPHABET_SIZE];
    let mut tempOccValue1 = [0u64; ALPHABET_SIZE];
    occValueMajor[0] = 0;
    occValueMajor[1] = 0;
    occValueMajor[2] = 0;
    occValueMajor[3] = 0;

    let mut occIndex = 0u64;
    let mut bwtIndex = 0usize;
    for occMajorIndex in 1..numberOfOccValueMajor {
        for _ in 0..numberOfOccIntervalPerMajor / 2 {
            let mut sum = 0u64;
            tempOccValue1.copy_from_slice(&tempOccValue0);
            for _ in 0..wordBetweenOccValue {
                let c = bwt[bwtIndex];
                sum += decodeTable[(c >> 16) as usize] as u64;
                sum += decodeTable[(c & 0x0000ffff) as usize] as u64;
                bwtIndex += 1;
            }
            if (sum & 0xfefefeff) != 0 {
                tempOccValue1[0] += sum & 0x000000ff;
                sum >>= 8;
                tempOccValue1[1] += sum & 0x000000ff;
                sum >>= 8;
                tempOccValue1[2] += sum & 0x000000ff;
                sum >>= 8;
                tempOccValue1[3] += sum;
            } else if sum == 0x00000100 {
                tempOccValue1[0] += 256;
            } else if sum == 0x00010000 {
                tempOccValue1[1] += 256;
            } else if sum == 0x01000000 {
                tempOccValue1[2] += 256;
            } else {
                tempOccValue1[3] += 256;
            }
            for c in 0..ALPHABET_SIZE {
                occValue[occIndex as usize * 4 + c] =
                    ((tempOccValue0[c] << 16) | tempOccValue1[c]) as u32;
                tempOccValue0[c] = tempOccValue1[c];
            }
            occIndex += 1;

            sum = 0;
            for _ in 0..wordBetweenOccValue {
                let c = bwt[bwtIndex];
                sum += decodeTable[(c >> 16) as usize] as u64;
                sum += decodeTable[(c & 0x0000ffff) as usize] as u64;
                bwtIndex += 1;
            }
            if (sum & 0xfefefeff) != 0 {
                tempOccValue0[0] += sum & 0x000000ff;
                sum >>= 8;
                tempOccValue0[1] += sum & 0x000000ff;
                sum >>= 8;
                tempOccValue0[2] += sum & 0x000000ff;
                sum >>= 8;
                tempOccValue0[3] += sum;
            } else if sum == 0x00000100 {
                tempOccValue0[0] += 256;
            } else if sum == 0x00010000 {
                tempOccValue0[1] += 256;
            } else if sum == 0x01000000 {
                tempOccValue0[2] += 256;
            } else {
                tempOccValue0[3] += 256;
            }
        }

        for c in 0..ALPHABET_SIZE {
            occValueMajor[occMajorIndex as usize * 4 + c] =
                occValueMajor[(occMajorIndex - 1) as usize * 4 + c] + tempOccValue0[c];
            tempOccValue0[c] = 0;
        }
    }

    while occIndex < (numberOfOccValue - 1) / 2 {
        let mut sum = 0u64;
        tempOccValue1.copy_from_slice(&tempOccValue0);
        for _ in 0..wordBetweenOccValue {
            let c = bwt[bwtIndex];
            sum += decodeTable[(c >> 16) as usize] as u64;
            sum += decodeTable[(c & 0x0000ffff) as usize] as u64;
            bwtIndex += 1;
        }
        if (sum & 0xfefefeff) != 0 {
            tempOccValue1[0] += sum & 0x000000ff;
            sum >>= 8;
            tempOccValue1[1] += sum & 0x000000ff;
            sum >>= 8;
            tempOccValue1[2] += sum & 0x000000ff;
            sum >>= 8;
            tempOccValue1[3] += sum;
        } else if sum == 0x00000100 {
            tempOccValue1[0] += 256;
        } else if sum == 0x00010000 {
            tempOccValue1[1] += 256;
        } else if sum == 0x01000000 {
            tempOccValue1[2] += 256;
        } else {
            tempOccValue1[3] += 256;
        }
        for c in 0..ALPHABET_SIZE {
            occValue[occIndex as usize * 4 + c] =
                ((tempOccValue0[c] << 16) | tempOccValue1[c]) as u32;
            tempOccValue0[c] = tempOccValue1[c];
        }
        occIndex += 1;

        sum = 0;
        for _ in 0..wordBetweenOccValue {
            let c = bwt[bwtIndex];
            sum += decodeTable[(c >> 16) as usize] as u64;
            sum += decodeTable[(c & 0x0000ffff) as usize] as u64;
            bwtIndex += 1;
        }
        if (sum & 0xfefefeff) != 0 {
            tempOccValue0[0] += sum & 0x000000ff;
            sum >>= 8;
            tempOccValue0[1] += sum & 0x000000ff;
            sum >>= 8;
            tempOccValue0[2] += sum & 0x000000ff;
            sum >>= 8;
            tempOccValue0[3] += sum;
        } else if sum == 0x00000100 {
            tempOccValue0[0] += 256;
        } else if sum == 0x00010000 {
            tempOccValue0[1] += 256;
        } else if sum == 0x01000000 {
            tempOccValue0[2] += 256;
        } else {
            tempOccValue0[3] += 256;
        }
    }

    let mut sum = 0u64;
    tempOccValue1.copy_from_slice(&tempOccValue0);
    if occIndex * 2 < numberOfOccValue - 1 {
        for _ in 0..wordBetweenOccValue {
            let c = bwt[bwtIndex];
            sum += decodeTable[(c >> 16) as usize] as u64;
            sum += decodeTable[(c & 0x0000ffff) as usize] as u64;
            bwtIndex += 1;
        }
        if (sum & 0xfefefeff) != 0 {
            tempOccValue1[0] += sum & 0x000000ff;
            sum >>= 8;
            tempOccValue1[1] += sum & 0x000000ff;
            sum >>= 8;
            tempOccValue1[2] += sum & 0x000000ff;
            sum >>= 8;
            tempOccValue1[3] += sum;
        } else if sum == 0x00000100 {
            tempOccValue1[0] += 256;
        } else if sum == 0x00010000 {
            tempOccValue1[1] += 256;
        } else if sum == 0x01000000 {
            tempOccValue1[2] += 256;
        } else {
            tempOccValue1[3] += 256;
        }
    }

    for c in 0..ALPHABET_SIZE {
        occValue[occIndex as usize * 4 + c] = ((tempOccValue0[c] << 16) | tempOccValue1[c]) as u32;
    }
}

/// Original C static function `BWTIncConstruct` from `minibwa/bwtgen.c:1289`.
pub fn BWTIncConstruct(bwtInc: &mut BWTInc, numChar: bgint_t) {
    let mergedBwtSizeInWord = BWTResidentSizeInWord(bwtInc.bwt.textLength + numChar);
    let mergedOccSizeInWord = BWTOccValueMinorSizeInWord(bwtInc.bwt.textLength + numChar);
    initializeVAL_bg(
        &mut bwtInc.cumulativeCountInCurrentBuild,
        (ALPHABET_SIZE + 1) as u64,
        0,
    );

    let firstCharInThisIteration;
    let newInverseSa0;
    let mut mergedBwt;

    if bwtInc.bwt.textLength == 0 {
        let num_items = numChar as usize + 1;
        let mut seq = uninit_vec::<u64>(num_items);
        let mut relativeRank = uninit_vec::<u64>(num_items);

        BWTIncPutPackedTextToRank(
            &bwtInc.packedText,
            &mut relativeRank,
            &mut bwtInc.cumulativeCountInCurrentBuild,
            numChar,
        );

        firstCharInThisIteration = relativeRank[0] as u32;
        relativeRank[numChar as usize] = 0;

        let qs_v = as_qsint_slice_mut(&mut relativeRank[..numChar as usize + 1]);
        let qs_i = as_qsint_slice_mut(&mut seq[..numChar as usize + 1]);
        QSufSortSuffixSort(qs_v, qs_i, numChar as i64, ALPHABET_SIZE as i64 - 1, 0, 0);
        newInverseSa0 = relativeRank[0];
        drop(seq);

        mergedBwt = uninit_vec::<u32>(mergedBwtSizeInWord as usize + 1);
        initializeVAL(&mut mergedBwt, mergedBwtSizeInWord, 0);
        BWTIncBuildPackedBwt(
            &relativeRank,
            &mut mergedBwt,
            numChar,
            &bwtInc.cumulativeCountInCurrentBuild,
            &bwtInc.packedShift,
        );

        bwtInc.firstCharInLastIteration = ALPHABET_SIZE as u32;
    } else {
        let num_items = numChar as usize + 1;
        let mut sortedRank = uninit_vec::<u64>(num_items);
        let mut seq = uninit_vec::<u64>(num_items);
        let mut relativeRank = uninit_vec::<u64>(num_items);

        firstCharInThisIteration = bwtInc.packedText[0] >> (BITS_IN_WORD - BIT_PER_CHAR);

        ForwardDNAAllOccCountNoLimit(
            &bwtInc.packedText,
            numChar,
            &mut bwtInc.cumulativeCountInCurrentBuild[1..],
            &bwtInc.bwt.decodeTable,
        );
        bwtInc.cumulativeCountInCurrentBuild[bwtInc.firstCharInLastIteration as usize + 1] += 1;
        bwtInc.cumulativeCountInCurrentBuild[2] += bwtInc.cumulativeCountInCurrentBuild[1];
        bwtInc.cumulativeCountInCurrentBuild[3] += bwtInc.cumulativeCountInCurrentBuild[2];
        bwtInc.cumulativeCountInCurrentBuild[4] += bwtInc.cumulativeCountInCurrentBuild[3];

        let oldInverseSa0RelativeRank = BWTIncGetAbsoluteRank(
            &bwtInc.bwt,
            &mut sortedRank,
            &mut seq,
            &bwtInc.packedText,
            numChar,
            &bwtInc.cumulativeCountInCurrentBuild,
            bwtInc.firstCharInLastIteration,
        );

        for i in 0..ALPHABET_SIZE {
            let lo = bwtInc.cumulativeCountInCurrentBuild[i];
            let hi = bwtInc.cumulativeCountInCurrentBuild[i + 1];
            if lo > oldInverseSa0RelativeRank || hi <= oldInverseSa0RelativeRank {
                BWTIncSortKey(
                    &mut sortedRank[lo as usize..],
                    &mut seq[lo as usize..],
                    hi - lo,
                );
            } else {
                if lo < oldInverseSa0RelativeRank {
                    BWTIncSortKey(
                        &mut sortedRank[lo as usize..],
                        &mut seq[lo as usize..],
                        oldInverseSa0RelativeRank - lo,
                    );
                }
                if hi > oldInverseSa0RelativeRank + 1 {
                    BWTIncSortKey(
                        &mut sortedRank[(oldInverseSa0RelativeRank + 1) as usize..],
                        &mut seq[(oldInverseSa0RelativeRank + 1) as usize..],
                        hi - oldInverseSa0RelativeRank - 1,
                    );
                }
            }
        }

        BWTIncBuildRelativeRank(
            &mut sortedRank,
            &mut seq,
            &mut relativeRank,
            numChar,
            bwtInc.bwt.inverseSa0,
            &bwtInc.cumulativeCountInCurrentBuild,
        );
        assert_eq!(relativeRank[numChar as usize], oldInverseSa0RelativeRank);

        let qs_v = as_qsint_slice_mut(&mut relativeRank[..numChar as usize + 1]);
        let qs_i = as_qsint_slice_mut(&mut seq[..numChar as usize + 1]);
        QSufSortSuffixSort(qs_v, qs_i, numChar as i64, numChar as i64, 1, 1);
        drop(seq);

        let newInverseSa0RelativeRank = relativeRank[0];
        newInverseSa0 = sortedRank[newInverseSa0RelativeRank as usize] + newInverseSa0RelativeRank;
        sortedRank[newInverseSa0RelativeRank as usize] = 0;

        let mut insertBwt = uninit_vec::<u32>(numChar as usize + 1);
        BWTIncBuildBwt(
            &mut insertBwt,
            &relativeRank,
            numChar,
            &bwtInc.cumulativeCountInCurrentBuild,
        );
        drop(relativeRank);

        mergedBwt = uninit_vec::<u32>(mergedBwtSizeInWord as usize + 2);
        let mut oldBwt = std::mem::take(&mut bwtInc.bwt.bwtCode);
        oldBwt.resize(oldBwt.len() + 2, 0);
        BWTIncMergeBwt(
            &sortedRank,
            &oldBwt,
            &insertBwt,
            &mut mergedBwt,
            bwtInc.bwt.textLength,
            numChar,
        );
    }

    bwtInc.bwt.textLength += numChar;
    bwtInc.bwt.bwtCode = mergedBwt;
    bwtInc.bwt.bwtSizeInWord = mergedBwtSizeInWord;
    bwtInc.bwt.occSizeInWord = mergedOccSizeInWord;
    bwtInc.bwt.occValue.clear();
    bwtInc.bwt.occValue.resize(mergedOccSizeInWord as usize, 0);

    BWTClearTrailingBwtCode(&mut bwtInc.bwt);
    BWTGenerateOccValueFromBwt(
        &bwtInc.bwt.bwtCode,
        &mut bwtInc.bwt.occValue,
        &mut bwtInc.bwt.occValueMajor,
        bwtInc.bwt.textLength,
        &bwtInc.bwt.decodeTable,
    );

    bwtInc.bwt.inverseSa0 = newInverseSa0;

    bwtInc.bwt.cumulativeFreq[1] +=
        bwtInc.cumulativeCountInCurrentBuild[1] - (bwtInc.firstCharInLastIteration <= 0) as u64;
    bwtInc.bwt.cumulativeFreq[2] +=
        bwtInc.cumulativeCountInCurrentBuild[2] - (bwtInc.firstCharInLastIteration <= 1) as u64;
    bwtInc.bwt.cumulativeFreq[3] +=
        bwtInc.cumulativeCountInCurrentBuild[3] - (bwtInc.firstCharInLastIteration <= 2) as u64;
    bwtInc.bwt.cumulativeFreq[4] +=
        bwtInc.cumulativeCountInCurrentBuild[4] - (bwtInc.firstCharInLastIteration <= 3) as u64;

    bwtInc.firstCharInLastIteration = firstCharInThisIteration;
    BWTIncSetBuildSizeAndTextAddr(bwtInc);
    bwtInc.numberOfIterationDone += 1;
}

/// Original C global function `BWTIncConstructFromPacked` from `minibwa/bwtgen.c:1427`.
pub fn BWTIncConstructFromPacked(
    inputFileName: &std::path::Path,
    initialMaxBuildSize: bgint_t,
    incMaxBuildSize: bgint_t,
) -> std::io::Result<BWTInc> {
    let inputFileName = c_path(inputFileName);
    let mut fp = match std::fs::File::open(&inputFileName) {
        Ok(fp) => fp,
        Err(err) => {
            let reason = err
                .raw_os_error()
                .map(c_strerror)
                .unwrap_or_else(|| err.to_string());
            eprintln!(
                "BWTIncConstructFromPacked() : Cannot open {} : {}",
                inputFileName.display(),
                reason
            );
            std::process::exit(1);
        }
    };
    use std::io::{Read, Seek};

    if fp.seek(std::io::SeekFrom::End(-1)).is_err() {
        eprintln!(
            "BWTIncConstructFromPacked() : Can't seek on {} : {}",
            inputFileName.display(),
            c_strerror(libc::EINVAL)
        );
        std::process::exit(1);
    }
    let packedFileLen = match fp.stream_position() {
        Ok(pos) => pos,
        Err(_) => bwt_packed_seek_error_and_exit(&inputFileName),
    };
    let mut last = [0u8; 1];
    if let Err(err) = fp.read_exact(&mut last) {
        bwt_packed_read_error_and_exit(&inputFileName, err);
    }
    let lastByteLength = last[0] as u32;
    let totalTextLength = TextLengthFromBytePacked(packedFileLen, BIT_PER_CHAR, lastByteLength);

    let mut bwtInc = BWTIncCreate(
        totalTextLength,
        initialMaxBuildSize as u32,
        incMaxBuildSize as u32,
    );
    BWTIncSetBuildSizeAndTextAddr(&mut bwtInc);

    let mut textToLoad = if bwtInc.buildSize > totalTextLength {
        totalTextLength
    } else {
        totalTextLength
            - ((totalTextLength - bwtInc.buildSize + CHAR_PER_WORD - 1) / CHAR_PER_WORD
                * CHAR_PER_WORD)
    };
    let mut textSizeInByte = textToLoad / CHAR_PER_BYTE as u64;
    bwtInc.packedText.clear();
    bwtInc.packedText.resize(
        (textToLoad as usize + CHAR_PER_WORD as usize - 1) / CHAR_PER_WORD as usize + 1,
        0,
    );
    bwtInc.textBuffer.resize(textSizeInByte as usize + 1, 0);
    bwt_packed_seek_back(&mut fp, &inputFileName, textSizeInByte + 2);
    if let Err(err) = fp.read_exact(&mut bwtInc.textBuffer) {
        bwt_packed_read_error_and_exit(&inputFileName, err);
    }
    bwt_packed_seek_back(&mut fp, &inputFileName, textSizeInByte + 1);
    ConvertBytePackedToWordPacked(
        &bwtInc.textBuffer,
        &mut bwtInc.packedText,
        ALPHABET_SIZE as u32,
        textToLoad,
    );
    BWTIncConstruct(&mut bwtInc, textToLoad);

    let mut processedTextLength = textToLoad;
    while processedTextLength < totalTextLength {
        textToLoad = bwtInc.buildSize / CHAR_PER_WORD * CHAR_PER_WORD;
        if textToLoad > totalTextLength - processedTextLength {
            textToLoad = totalTextLength - processedTextLength;
        }
        textSizeInByte = textToLoad / CHAR_PER_BYTE as u64;
        bwtInc.packedText.clear();
        bwtInc.packedText.resize(
            (textToLoad as usize + CHAR_PER_WORD as usize - 1) / CHAR_PER_WORD as usize + 1,
            0,
        );
        bwtInc.textBuffer.resize(textSizeInByte as usize, 0);
        bwt_packed_seek_back(&mut fp, &inputFileName, textSizeInByte);
        if let Err(err) = fp.read_exact(&mut bwtInc.textBuffer) {
            bwt_packed_read_error_and_exit(&inputFileName, err);
        }
        bwt_packed_seek_back(&mut fp, &inputFileName, textSizeInByte);
        ConvertBytePackedToWordPacked(
            &bwtInc.textBuffer,
            &mut bwtInc.packedText,
            ALPHABET_SIZE as u32,
            textToLoad,
        );
        BWTIncConstruct(&mut bwtInc, textToLoad);
        processedTextLength += textToLoad;
        if bwtInc.numberOfIterationDone % 10 == 0 {
            eprintln!(
                "[BWTIncConstructFromPacked] {} iterations done. {} characters processed.",
                bwtInc.numberOfIterationDone, processedTextLength
            );
        }
    }

    Ok(bwtInc)
}

/// Original C global function `BWTIncFree` from `minibwa/bwtgen.c:1537`.
pub fn BWTIncFree(bwtInc: Option<BWTInc>) {
    if bwtInc.is_none() {
        return;
    }
}

/// Original C static function `BWTFileSizeInWord` from `minibwa/bwtgen.c:1550`.
pub fn BWTFileSizeInWord(numChar: bgint_t) -> bgint_t {
    (numChar + CHAR_PER_WORD - 1) / CHAR_PER_WORD
}

/// Original C global function `BWTSaveBwtCodeAndOcc` from `minibwa/bwtgen.c:1556`.
pub fn BWTSaveBwtCodeAndOcc(
    bwt: &BWT,
    bwtFileName: &std::path::Path,
    occValueFileName: Option<&std::path::Path>,
) -> std::io::Result<()> {
    let bwtFileName = c_path(bwtFileName);
    let bwtFile = match std::fs::File::create(&bwtFileName) {
        Ok(file) => file,
        Err(err) => {
            let reason = err
                .raw_os_error()
                .map(c_strerror)
                .unwrap_or_else(|| err.to_string());
            eprintln!(
                "BWTSaveBwtCodeAndOcc(): Cannot open {} for writing: {}",
                bwtFileName.display(),
                reason
            );
            std::process::exit(1);
        }
    };
    let mut bwtFile = std::io::BufWriter::with_capacity(64 * 1024, bwtFile);
    let bwtLength = BWTFileSizeInWord(bwt.textLength);
    use std::io::Write;
    if let Err(err) = bwtFile.write_all(&bwt.inverseSa0.to_le_bytes()) {
        bwt_save_error_and_exit(&bwtFileName, err);
    }
    for i in 1..=ALPHABET_SIZE {
        if let Err(err) = bwtFile.write_all(&bwt.cumulativeFreq[i].to_le_bytes()) {
            bwt_save_error_and_exit(&bwtFileName, err);
        }
    }
    for &word in bwt.bwtCode.iter().take(bwtLength as usize) {
        if let Err(err) = bwtFile.write_all(&word.to_le_bytes()) {
            bwt_save_error_and_exit(&bwtFileName, err);
        }
    }
    if let Err(err) = bwtFile.flush() {
        bwt_save_error_and_exit(&bwtFileName, err);
    }
    let _ = occValueFileName;
    Ok(())
}

/// Original C global function `mb_bwtgen` from `minibwa/bwtgen.c:1588`.
pub fn mb_bwtgen(
    fn_pac: &std::path::Path,
    fn_bwt: &std::path::Path,
    block_size: i32,
) -> std::io::Result<()> {
    let fn_pac = c_path(fn_pac);
    let fn_bwt = c_path(fn_bwt);
    let bwtInc = BWTIncConstructFromPacked(&fn_pac, block_size as u64, block_size as u64)?;
    eprintln!(
        "[bwt_gen] Finished constructing BWT in {} iterations.",
        bwtInc.numberOfIterationDone
    );
    BWTSaveBwtCodeAndOcc(&bwtInc.bwt, &fn_bwt, None)?;
    BWTIncFree(Some(bwtInc));
    Ok(())
}
