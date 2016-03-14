//! This crate implements the
//! [package-merge](https://en.wikipedia.org/wiki/Package-merge_algorithm)
//! algorithm. The package-merge
//! algorithm is able to compute an optimal length-limited prefix-free code.
//! As such, it might be useful for data compression purposes much like the
//! Huffman algorithm. But Huffman's algorithm does not allow you do
//! constrain the maximum length of all code words.

extern crate itertools;

use std::mem;
use std::cmp;
use itertools::Itertools;

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

/// Given all symbol frequencies (or probabilities) and a limit on the
/// maximum length of code words (up to 32), this function will apply
/// the package merge algorithm to compute optimal code word lengths
/// for the symbols so that the expected code word length is minimized.
pub fn package_merge(frequencies: &[f64], max_len: u32) -> Vec<u32> {

    assert!(max_len <= 32);

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

    let mut code_lens = vec![0u32; frequencies.len()];
    let mut n = frequencies.len() * 2 - 2;
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

    code_lens
}

#[cfg(test)]
mod tests {
    use super::package_merge;

    #[test]
    fn it_works() {
        let freqs = [1.0, 32.0, 16.0, 4.0, 8.0, 2.0, 1.0];
        let cl = package_merge(&freqs, 8);
        assert_eq!(&cl[..], &[6, 1, 2, 4, 3, 5, 6]);
        let freqs = [1.0, 32.0, 16.0, 4.0, 8.0, 2.0, 1.0];
        let cl = package_merge(&freqs, 5);
        assert_eq!(&cl[..], &[5, 1, 2, 5, 3, 5, 5]);
    }
}

