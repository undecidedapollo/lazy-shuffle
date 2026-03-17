use hashbrown::HashMap;

use rand::{Rng, SeedableRng, rngs::SmallRng};

pub struct ShuffledIter {
    remaining: usize,
    displaced: HashMap<usize, usize>,
    rng: SmallRng,
}

impl ShuffledIter {
    pub fn new(len: usize) -> Self {
        Self {
            remaining: len,
            displaced: HashMap::new(),
            rng: SmallRng::from_os_rng(),
        }
    }
}

impl Iterator for ShuffledIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }

        let last_idx = self.remaining - 1;
        let random_idx = self.rng.random_range(0..=last_idx);
        self.remaining -= 1;

        if random_idx == last_idx {
            return Some(self.displaced.remove(&last_idx).unwrap_or(last_idx));
        }

        let val_random = *self.displaced.get(&random_idx).unwrap_or(&random_idx);
        let val_last = *self.displaced.get(&last_idx).unwrap_or(&last_idx);

        if val_last != random_idx {
            self.displaced.insert(random_idx, val_last);
            self.displaced.remove(&last_idx);
        } else {
            self.displaced.remove(&random_idx);
        }

        Some(val_random)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

// Free: gives you .len() since we know exact size
impl ExactSizeIterator for ShuffledIter {}

pub struct ShuffledSliceIter<'a, T> {
    slice: &'a [T],
    inner: ShuffledIter,
}

impl<'a, T> ShuffledSliceIter<'a, T> {
    pub fn new(slice: &'a [T]) -> Self {
        Self {
            inner: ShuffledIter::new(slice.len()),
            slice,
        }
    }
}

impl<'a, T> Iterator for ShuffledSliceIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|i| &self.slice[i])
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, T> ExactSizeIterator for ShuffledSliceIter<'a, T> {}

pub trait ShuffleExt<T> {
    fn shuffled(&self) -> ShuffledSliceIter<'_, T>;
}

impl<T, C: AsRef<[T]>> ShuffleExt<T> for C {
    fn shuffled(&self) -> ShuffledSliceIter<'_, T> {
        ShuffledSliceIter::new(self.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permutation_is_complete() {
        let expected: Vec<usize> = (0..100).collect();
        for _ in 0..10_000 {
            let mut result: Vec<usize> = ShuffledIter::new(100).collect();
            result.sort_unstable();
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn size_hint_and_exact_size() {
        let mut iter = ShuffledIter::new(5);
        assert_eq!(iter.size_hint(), (5, Some(5)));
        assert_eq!(iter.len(), 5);

        iter.next();
        assert_eq!(iter.size_hint(), (4, Some(4)));
        assert_eq!(iter.len(), 4);

        // Drain remaining
        for _ in 0..4 {
            iter.next();
        }
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.len(), 0);
        assert!(iter.next().is_none());
    }

    #[test]
    fn shuffle_ext_slice_values() {
        let source = vec!["a", "b", "c", "d", "e"];
        let mut expected = source.clone();
        expected.sort_unstable();

        for _ in 0..10_000 {
            let mut result: Vec<&str> = source.shuffled().copied().collect();
            result.sort_unstable();
            assert_eq!(result, expected);
        }
    }
}
