//! This crate implements the
//! [package-merge](https://en.wikipedia.org/wiki/Package-merge_algorithm)
//! algorithm. The package-merge
//! algorithm is able to compute an optimal length-limited prefix-free code.
//! As such, it might be useful for data compression purposes much like the
//! Huffman algorithm. But Huffman's algorithm does not allow you do
//! constrain the maximum length of all code words.

extern crate itertools;

use std::cmp;
use std::error;
use std::fmt;
use std::mem;

use itertools::Itertools;

use Error::*;

fn order_non_nan(a: f64, b: f64) -> cmp::Ordering {
    if a < b { cmp::Ordering::Less } else
    if a > b { cmp::Ordering::Greater } else
    { cmp::Ordering::Equal }
}

fn complete_chunks<T>(mut slice: &[T], csize: usize) -> std::slice::Chunks<T> {
    let remainder = slice.len() % csize;
    if remainder > 0 {
        slice = &slice[0..(slice.len() - remainder)];
    }
    slice.chunks(csize)
}

/// The error type for the package-merge algorithm
#[derive(Copy,Clone,PartialEq,Eq,Debug)]
pub enum Error {
    /// The given frequencies slice was empty.
    NoSymbols,
    /// The given `max_len` constraint was too small.
    MaxLenTooSmall,
    /// The given `max_len` constraint was too large.
    MaxLenTooLarge,
}

impl Error {
    fn descr(&self) -> &str {
        match *self {
            NoSymbols =>
                "package-merge error: frequencies slice was empty",
            MaxLenTooSmall =>
                "package-merge error: max_len parameter was chosen too small",
            MaxLenTooLarge =>
                "package-merge error: max_len parameter was chosen too large",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.descr())
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        self.descr()
    }
}

/// Given all symbol frequencies (or probabilities) and a limit on the
/// maximum length of code words (up to 32), this function will apply
/// the package merge algorithm to compute optimal code word lengths
/// for the symbols so that the expected code word length is minimized.
pub fn package_merge(frequencies: &[f64], max_len: u32) -> Result<Vec<u32>, Error> {

    if frequencies.is_empty() {
        return Err(Error::NoSymbols);
    }
    if frequencies.len() > (1usize << max_len) {
        return Err(Error::MaxLenTooSmall);
    }
    if max_len > 32 {
        return Err(Error::MaxLenTooLarge);
    }

    let sorted = {
        let mut tmp = Vec::new();
        tmp.extend(0..frequencies.len());
        tmp.sort_by( |&a, &b| order_non_nan(frequencies[a],frequencies[b]) );
        tmp
    };

    let capa = frequencies.len() * 2 - 1;
    let mut list: Vec<f64> = Vec::with_capacity(capa);
    let mut flags: Vec<u32> = vec![0; capa];
    let mut merged: Vec<f64> = Vec::with_capacity(capa);

    for depth in 0..max_len {
        {
            merged.clear();
            let mask = 1u32 << depth;
            let pairs = complete_chunks(&list, 2).map( |s| (s[0] + s[1], true) );
            let srted = sorted.iter().map( |&i| (frequencies[i], false) );
            for (p, m) in pairs.merge_by(srted, |a, b| a.0 < b.0 ) {
                if m { // was this a merged item?
                    flags[merged.len()] |= mask;
                }
                merged.push(p);
            }
        }
        mem::swap(&mut merged, &mut list);
    }

    let mut n = frequencies.len() * 2 - 2;
    debug_assert!(list.len() >= n);
    let mut code_lens = vec![0u32; frequencies.len()];
    let mut depth = max_len;
    while depth > 0 && n > 0 {
        depth -= 1;
        let mask = 1u32 << depth;
        let mut merged = 0;
        for i in 0..n {
            if (flags[i] & mask) == 0 {
                code_lens[sorted[i - merged]] += 1;
            } else {
                merged += 1;
            }
        }
        n = merged * 2;
    }

    Ok(code_lens)
}

#[cfg(test)]
mod tests {
    use super::package_merge;

    #[test]
    fn it_works() {
        let freqs = [1.0, 32.0, 16.0, 4.0, 8.0, 2.0, 1.0];
        let cl = package_merge(&freqs, 8).unwrap();
        assert_eq!(&cl[..], &[6, 1, 2, 4, 3, 5, 6]);
        let cl = package_merge(&freqs, 5).unwrap();
        assert_eq!(&cl[..], &[5, 1, 2, 5, 3, 5, 5]);
    }

    #[test]
    #[should_panic]
    fn it_fails() {
        let freqs = [1.0, 32.0, 16.0, 4.0, 8.0, 2.0, 1.0];
        package_merge(&freqs, 2).unwrap();
    }
}

