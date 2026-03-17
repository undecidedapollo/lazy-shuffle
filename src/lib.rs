#![no_std]
extern crate alloc;

use core::iter::FusedIterator;

use hashbrown::HashMap;
use rand::{Rng, SeedableRng, rngs::SmallRng};

#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct ShuffledIter<R: Rng = SmallRng> {
    remaining: usize,
    displaced: HashMap<usize, usize>,
    rng: R,
}

impl ShuffledIter {
    pub fn with_seed(len: usize, seed: u64) -> Self {
        Self {
            remaining: len,
            displaced: HashMap::new(),
            rng: SmallRng::seed_from_u64(seed),
        }
    }
}

#[cfg(feature = "std")]
impl ShuffledIter {
    pub fn new(len: usize) -> Self {
        Self {
            remaining: len,
            displaced: HashMap::new(),
            rng: SmallRng::from_os_rng(),
        }
    }
}

impl<R: Rng> ShuffledIter<R> {
    pub fn with_rng(len: usize, rng: R) -> Self {
        Self {
            remaining: len,
            displaced: HashMap::new(),
            rng,
        }
    }
}

impl<R: Rng> Iterator for ShuffledIter<R> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }

        let last_idx = self.remaining - 1;
        let random_idx = self.rng.random_range(0..=last_idx);
        self.remaining -= 1;

        let val_last = self.displaced.remove(&last_idx).unwrap_or(last_idx);

        if random_idx == last_idx {
            return Some(val_last);
        }

        let val_random = self.displaced.get(&random_idx).copied();

        if val_last != random_idx {
            self.displaced.insert(random_idx, val_last);
        } else if val_random.is_some() {
            self.displaced.remove(&random_idx);
        }

        Some(val_random.unwrap_or(random_idx))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<R: Rng> ExactSizeIterator for ShuffledIter<R> {}
impl<R: Rng> FusedIterator for ShuffledIter<R> {}

#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct ShuffledSliceIter<'a, T, R: Rng = SmallRng> {
    slice: &'a [T],
    inner: ShuffledIter<R>,
}

impl<'a, T> ShuffledSliceIter<'a, T> {
    pub fn with_seed(slice: &'a [T], seed: u64) -> Self {
        Self {
            inner: ShuffledIter::with_seed(slice.len(), seed),
            slice,
        }
    }
}

#[cfg(feature = "std")]
impl<'a, T> ShuffledSliceIter<'a, T> {
    pub fn new(slice: &'a [T]) -> Self {
        Self {
            inner: ShuffledIter::new(slice.len()),
            slice,
        }
    }
}

impl<'a, T, R: Rng> ShuffledSliceIter<'a, T, R> {
    pub fn with_rng(slice: &'a [T], rng: R) -> Self {
        Self {
            inner: ShuffledIter::with_rng(slice.len(), rng),
            slice,
        }
    }
}

impl<'a, T, R: Rng> Iterator for ShuffledSliceIter<'a, T, R> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|i| &self.slice[i])
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, T, R: Rng> ExactSizeIterator for ShuffledSliceIter<'a, T, R> {}
impl<T, R: Rng> FusedIterator for ShuffledSliceIter<'_, T, R> {}

pub trait ShuffleExt<T> {
    fn shuffled_with_seed(&self, seed: u64) -> ShuffledSliceIter<'_, T>;
    fn shuffled_with_rng<R: Rng>(&self, rng: R) -> ShuffledSliceIter<'_, T, R>;

    #[cfg(feature = "std")]
    fn shuffled(&self) -> ShuffledSliceIter<'_, T>;
}

impl<T, C: AsRef<[T]>> ShuffleExt<T> for C {
    fn shuffled_with_seed(&self, seed: u64) -> ShuffledSliceIter<'_, T> {
        ShuffledSliceIter::with_seed(self.as_ref(), seed)
    }

    fn shuffled_with_rng<R: Rng>(&self, rng: R) -> ShuffledSliceIter<'_, T, R> {
        ShuffledSliceIter::with_rng(self.as_ref(), rng)
    }

    #[cfg(feature = "std")]
    fn shuffled(&self) -> ShuffledSliceIter<'_, T> {
        ShuffledSliceIter::new(self.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;

    #[test]
    fn seeded_is_deterministic() {
        let a: Vec<usize> = ShuffledIter::with_seed(20, 42).collect();
        let b: Vec<usize> = ShuffledIter::with_seed(20, 42).collect();
        assert_eq!(a, b);
    }

    #[test]
    fn different_seeds_produce_different_orders() {
        let results: Vec<Vec<usize>> = (0..1000)
            .map(|seed| ShuffledIter::with_seed(1000, seed).collect())
            .collect();

        for i in 0..results.len() {
            for j in (i + 1)..results.len() {
                assert_ne!(results[i], results[j], "seed {} and {} collided", i, j);
            }
        }
    }

    #[test]
    fn fused_returns_none_forever() {
        let mut iter = ShuffledIter::with_seed(2, 0);
        iter.next();
        iter.next();
        assert!(iter.next().is_none());
        assert!(iter.next().is_none());
        assert!(iter.next().is_none());
    }

    #[test]
    fn shuffle_ext_with_seed() {
        let data = [10, 20, 30, 40, 50];
        let a: Vec<&i32> = data.shuffled_with_seed(99).collect();
        let b: Vec<&i32> = data.shuffled_with_seed(99).collect();
        assert_eq!(a, b);
        assert_eq!(a.len(), data.len());
    }

    #[test]
    fn shuffle_ext_with_rng() {
        let data = [10, 20, 30, 40, 50];
        let rng = SmallRng::seed_from_u64(7);
        let result: Vec<&i32> = data.shuffled_with_rng(rng).collect();
        assert_eq!(result.len(), data.len());
    }

    #[cfg(feature = "std")]
    #[test]
    fn permutation_is_complete() {
        let expected: Vec<usize> = (0..100).collect();
        for _ in 0..10_000 {
            let mut result: Vec<usize> = ShuffledIter::new(100).collect();
            result.sort_unstable();
            assert_eq!(result, expected);
        }
    }

    #[cfg(feature = "std")]
    #[test]
    fn size_hint_and_exact_size() {
        let mut iter = ShuffledIter::new(5);
        assert_eq!(iter.size_hint(), (5, Some(5)));
        assert_eq!(iter.len(), 5);

        iter.next();
        assert_eq!(iter.size_hint(), (4, Some(4)));
        assert_eq!(iter.len(), 4);

        for _ in 0..4 {
            iter.next();
        }
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.len(), 0);
        assert!(iter.next().is_none());
    }

    #[cfg(feature = "std")]
    #[test]
    fn shuffle_ext_slice_values() {
        let source = ["a", "b", "c", "d", "e"];
        let mut expected = Vec::from(source);
        expected.sort_unstable();

        for _ in 0..10_000 {
            let mut result: Vec<&str> = source.shuffled().copied().collect();
            result.sort_unstable();
            assert_eq!(result, expected);
        }
    }
}
