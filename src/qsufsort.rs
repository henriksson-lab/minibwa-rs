#![allow(unused_variables, dead_code, non_snake_case)]

pub const INSERT_SORT_NUM_ITEM: i64 = 16;

/// Original C global function `QSufSortSuffixSort` from `minibwa/QSufSort.c:56`.
pub fn QSufSortSuffixSort(
    V: &mut [i64],
    I: &mut [i64],
    numChar: i64,
    largestInputSymbol: i64,
    smallestInputSymbol: i64,
    skipTransform: i32,
) {
    let mut numSortedPos = 1i64;
    if skipTransform == 0 {
        let mut numSymbolAggregated = 0;
        let newAlphabetSize = QSufSortTransform(
            V,
            I,
            numChar,
            largestInputSymbol,
            smallestInputSymbol,
            numChar,
            &mut numSymbolAggregated,
        );
        QSufSortBucketSort(V, I, numChar, newAlphabetSize);
        I[0] = -1;
        V[numChar as usize] = 0;
        numSortedPos = numSymbolAggregated;
    }

    while I[0] >= -numChar {
        let mut i = 0i64;
        let mut negatedSortedGroupLength = 0i64;
        loop {
            let s = I[i as usize];
            if s < 0 {
                i -= s;
                negatedSortedGroupLength += s;
            } else {
                if negatedSortedGroupLength != 0 {
                    I[(i + negatedSortedGroupLength) as usize] = negatedSortedGroupLength;
                    negatedSortedGroupLength = 0;
                }
                let j = V[s as usize] + 1;
                QSufSortSortSplit(V, I, i, j - 1, numSortedPos);
                i = j;
            }
            if i > numChar {
                break;
            }
        }
        if negatedSortedGroupLength != 0 {
            I[(i + negatedSortedGroupLength) as usize] = negatedSortedGroupLength;
        }
        numSortedPos *= 2;
    }
}

/// Original C global function `QSufSortGenerateSaFromInverse` from `minibwa/QSufSort.c:101`.
pub fn QSufSortGenerateSaFromInverse(V: &[i64], I: &mut [i64], numChar: i64) {
    for i in 0..=numChar as usize {
        I[V[i] as usize] = i as i64 + 1;
    }
}

/// Original C static function `QSufSortSortSplit` from `minibwa/QSufSort.c:113`.
pub fn QSufSortSortSplit(
    V: &mut [i64],
    I: &mut [i64],
    lowestPos: i64,
    highestPos: i64,
    numSortedChar: i64,
) {
    let numItem = highestPos - lowestPos + 1;
    if numItem <= INSERT_SORT_NUM_ITEM {
        QSufSortInsertSortSplit(V, I, lowestPos, highestPos, numSortedChar);
        return;
    }

    let v = QSufSortChoosePivot(V, I, lowestPos, highestPos, numSortedChar);
    let mut a = lowestPos;
    let mut b = lowestPos;
    let mut c = highestPos;
    let mut d = highestPos;

    loop {
        while c >= b && V[(I[b as usize] + numSortedChar) as usize] <= v {
            if V[(I[b as usize] + numSortedChar) as usize] == v {
                I.swap(a as usize, b as usize);
                a += 1;
            }
            b += 1;
        }
        while c >= b && V[(I[c as usize] + numSortedChar) as usize] >= v {
            if V[(I[c as usize] + numSortedChar) as usize] == v {
                I.swap(c as usize, d as usize);
                d -= 1;
            }
            c -= 1;
        }
        if b > c {
            break;
        }
        I.swap(b as usize, c as usize);
        b += 1;
        c -= 1;
    }

    let mut s = a - lowestPos;
    let mut t = b - a;
    s = s.min(t);
    let mut l = lowestPos;
    let mut m = b - s;
    while m < b {
        I.swap(l as usize, m as usize);
        l += 1;
        m += 1;
    }

    s = d - c;
    t = highestPos - d;
    s = s.min(t);
    l = b;
    m = highestPos - s + 1;
    while m <= highestPos {
        I.swap(l as usize, m as usize);
        l += 1;
        m += 1;
    }

    s = b - a;
    t = d - c;
    if s > 0 {
        QSufSortSortSplit(V, I, lowestPos, lowestPos + s - 1, numSortedChar);
    }

    a = lowestPos + s;
    b = highestPos - t;
    if a == b {
        V[I[a as usize] as usize] = a;
        I[a as usize] = -1;
    } else {
        c = a;
        while c <= b {
            V[I[c as usize] as usize] = b;
            c += 1;
        }
    }

    if t > 0 {
        QSufSortSortSplit(V, I, highestPos - t + 1, highestPos, numSortedChar);
    }
}

/// Original C static function `QSufSortChoosePivot` from `minibwa/QSufSort.c:194`.
pub fn QSufSortChoosePivot(
    V: &[i64],
    I: &[i64],
    lowestPos: i64,
    highestPos: i64,
    numSortedChar: i64,
) -> i64 {
    let numItem = highestPos - lowestPos + 1;
    let m = lowestPos + numItem / 2;
    let s = numItem / 8;
    let key1 = V[(I[lowestPos as usize] + numSortedChar) as usize];
    let key2 = V[(I[(lowestPos + s) as usize] + numSortedChar) as usize];
    let key3 = V[(I[(lowestPos + 2 * s) as usize] + numSortedChar) as usize];
    let keyl = if key1 < key2 {
        if key2 < key3 {
            key2
        } else if key1 < key3 {
            key3
        } else {
            key1
        }
    } else if key2 > key3 {
        key2
    } else if key1 > key3 {
        key3
    } else {
        key1
    };
    let key1 = V[(I[(m - s) as usize] + numSortedChar) as usize];
    let key2 = V[(I[m as usize] + numSortedChar) as usize];
    let key3 = V[(I[(m + s) as usize] + numSortedChar) as usize];
    let keym = if key1 < key2 {
        if key2 < key3 {
            key2
        } else if key1 < key3 {
            key3
        } else {
            key1
        }
    } else if key2 > key3 {
        key2
    } else if key1 > key3 {
        key3
    } else {
        key1
    };
    let key1 = V[(I[(highestPos - 2 * s) as usize] + numSortedChar) as usize];
    let key2 = V[(I[(highestPos - s) as usize] + numSortedChar) as usize];
    let key3 = V[(I[highestPos as usize] + numSortedChar) as usize];
    let keyn = if key1 < key2 {
        if key2 < key3 {
            key2
        } else if key1 < key3 {
            key3
        } else {
            key1
        }
    } else if key2 > key3 {
        key2
    } else if key1 > key3 {
        key3
    } else {
        key1
    };
    if keyl < keym {
        if keym < keyn {
            keym
        } else if keyl < keyn {
            keyn
        } else {
            keyl
        }
    } else if keym > keyn {
        keym
    } else if keyl > keyn {
        keyn
    } else {
        keyl
    }
}

/// Original C static function `QSufSortInsertSortSplit` from `minibwa/QSufSort.c:227`.
pub fn QSufSortInsertSortSplit(
    V: &mut [i64],
    I: &mut [i64],
    lowestPos: i64,
    highestPos: i64,
    numSortedChar: i64,
) {
    let numItem = highestPos - lowestPos + 1;
    let mut key = vec![0i64; numItem as usize];
    let mut pos = vec![0i64; numItem as usize];
    for i in 0..numItem as usize {
        pos[i] = I[lowestPos as usize + i];
        key[i] = V[(pos[i] + numSortedChar) as usize];
    }
    for i in 1..numItem as usize {
        let tmpKey = key[i];
        let tmpPos = pos[i];
        let mut j = i;
        while j > 0 && key[j - 1] > tmpKey {
            key[j] = key[j - 1];
            pos[j] = pos[j - 1];
            j -= 1;
        }
        key[j] = tmpKey;
        pos[j] = tmpPos;
    }

    let mut negativeSortedLength = -1i64;
    let mut i = numItem - 1;
    let mut groupNum = highestPos;
    while i > 0 {
        I[(i + lowestPos) as usize] = pos[i as usize];
        V[I[(i + lowestPos) as usize] as usize] = groupNum;
        if key[(i - 1) as usize] == key[i as usize] {
            negativeSortedLength = 0;
        } else {
            if negativeSortedLength < 0 {
                I[(i + lowestPos) as usize] = negativeSortedLength;
            }
            groupNum = i + lowestPos - 1;
            negativeSortedLength -= 1;
        }
        i -= 1;
    }
    I[lowestPos as usize] = pos[0];
    V[I[lowestPos as usize] as usize] = groupNum;
    if negativeSortedLength < 0 {
        I[lowestPos as usize] = negativeSortedLength;
    }
}

/// Original C static function `QSufSortBucketSort` from `minibwa/QSufSort.c:288`.
pub fn QSufSortBucketSort(V: &mut [i64], I: &mut [i64], numChar: i64, alphabetSize: i64) {
    for i in 0..alphabetSize as usize {
        I[i] = -1;
    }
    for i in 0..=numChar as usize {
        let c = V[i] as usize;
        V[i] = I[c];
        I[c] = i as i64;
    }
    let mut currentIndex = numChar;
    let mut i = alphabetSize;
    while i > 0 {
        let mut c = I[(i - 1) as usize];
        let mut d = V[c as usize];
        let groupNum = currentIndex;
        V[c as usize] = groupNum;
        if d >= 0 {
            I[currentIndex as usize] = c;
            while d >= 0 {
                c = d;
                d = V[c as usize];
                V[c as usize] = groupNum;
                currentIndex -= 1;
                I[currentIndex as usize] = c;
            }
        } else {
            I[currentIndex as usize] = -1;
        }
        currentIndex -= 1;
        i -= 1;
    }
}

/// Original C static function `QSufSortTransform` from `minibwa/QSufSort.c:344`.
pub fn QSufSortTransform(
    V: &mut [i64],
    I: &mut [i64],
    numChar: i64,
    largestInputSymbol: i64,
    smallestInputSymbol: i64,
    maxNewAlphabetSize: i64,
    numSymbolAggregated: &mut i64,
) -> i64 {
    let maxNumInputSymbol = largestInputSymbol - smallestInputSymbol + 1;
    let mut maxNumBit = 0i64;
    let mut i = maxNumInputSymbol;
    while i != 0 {
        maxNumBit += 1;
        i >>= 1;
    }
    let maxSymbol = i64::MAX >> maxNumBit;
    let mut c = maxNumInputSymbol;
    let mut a = 0i64;
    let mut minSymbolInChunk = 0i64;
    let mut maxSymbolInChunk = 0i64;
    while a < numChar && maxSymbolInChunk <= maxSymbol && c <= maxNewAlphabetSize {
        minSymbolInChunk =
            (minSymbolInChunk << maxNumBit) | (V[a as usize] - smallestInputSymbol + 1);
        maxSymbolInChunk = c;
        c = (maxSymbolInChunk << maxNumBit) | maxNumInputSymbol;
        a += 1;
    }

    let mask = (1i64 << ((a - 1) * maxNumBit)) - 1;
    V[numChar as usize] = smallestInputSymbol - 1;
    for i in 0..=maxSymbolInChunk as usize {
        I[i] = 0;
    }
    c = minSymbolInChunk;
    i = a;
    while i <= numChar {
        I[c as usize] = 1;
        c = ((c & mask) << maxNumBit) | (V[i as usize] - smallestInputSymbol + 1);
        i += 1;
    }
    i = 1;
    while i < a {
        I[c as usize] = 1;
        c = (c & mask) << maxNumBit;
        i += 1;
    }
    let mut newAlphabetSize = 1i64;
    for i in 0..=maxSymbolInChunk as usize {
        if I[i] != 0 {
            I[i] = newAlphabetSize;
            newAlphabetSize += 1;
        }
    }
    c = minSymbolInChunk;
    i = 0;
    let mut j = a;
    while j <= numChar {
        V[i as usize] = I[c as usize];
        c = ((c & mask) << maxNumBit) | (V[j as usize] - smallestInputSymbol + 1);
        i += 1;
        j += 1;
    }
    while i < numChar {
        V[i as usize] = I[c as usize];
        c = (c & mask) << maxNumBit;
        i += 1;
    }
    V[numChar as usize] = 0;
    *numSymbolAggregated = a;
    newAlphabetSize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_sa_from_inverse_follows_original_one_based_output() {
        let v = vec![2, 0, 3, 1];
        let mut i = vec![0; 4];
        QSufSortGenerateSaFromInverse(&v, &mut i, 3);
        assert_eq!(i, vec![2, 4, 1, 3]);
    }

    #[test]
    fn suffix_sort_matches_naive_suffix_order_for_small_text() {
        let text = [2, 1, 3, 1, 3, 1];
        let n = text.len() as i64;
        let mut v = vec![0i64; text.len() + 1];
        let mut i = vec![0i64; text.len() + 1];
        for (k, &b) in text.iter().enumerate() {
            v[k] = b;
        }
        QSufSortSuffixSort(&mut v, &mut i, n, 3, 1, 0);
        QSufSortGenerateSaFromInverse(&v, &mut i, n);
        let got: Vec<usize> = i.iter().map(|&x| x as usize - 1).collect();
        let mut expected: Vec<usize> = (0..=text.len()).collect();
        expected.sort_by(|&a, &b| text[a..].cmp(&text[b..]));
        assert_eq!(got, expected);
    }
}
