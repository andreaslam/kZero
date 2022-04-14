use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::iter::Zip;
use std::time::Instant;

use itertools::{zip, Itertools};
use rand::{Error, Rng, RngCore};

/// An Rng implementation that panics as soon as it is called.
/// Useful to assert that something doesn't actually use any randomness.
#[derive(Debug)]
pub struct PanicRng;

impl RngCore for PanicRng {
    fn next_u32(&mut self) -> u32 {
        panic!("Tried to use PanicRng")
    }

    fn next_u64(&mut self) -> u64 {
        panic!("Tried to use PanicRng")
    }

    fn fill_bytes(&mut self, _: &mut [u8]) {
        panic!("Tried to use PanicRng")
    }

    fn try_fill_bytes(&mut self, _: &mut [u8]) -> Result<(), Error> {
        panic!("Tried to use PanicRng")
    }
}

/// Similar to [rand::seq::IteratorRandom::choose] but will only pick items with the maximum key.
/// Equivalent to first finding the max key, then filtering items matching that key and then choosing a random element,
/// but implemented in a single pass over the iterator.
pub fn choose_max_by_key<T, I: IntoIterator<Item = T>, K: Ord, F: FnMut(&T) -> K>(
    iter: I,
    mut key: F,
    rng: &mut impl Rng,
) -> Option<T> {
    let mut iter = iter.into_iter();

    let mut curr = iter.next()?;
    let mut max_key = key(&curr);
    let mut i = 1;

    for next in iter {
        let next_key = key(&next);
        match next_key.cmp(&max_key) {
            Ordering::Less => continue,
            Ordering::Equal => {
                i += 1;
                if rng.gen_range(0..i) == 0 {
                    curr = next;
                }
            }
            Ordering::Greater => {
                i = 1;
                curr = next;
                max_key = next_key;
            }
        }
    }

    Some(curr)
}

pub trait IndexOf<T> {
    fn index_of(self, element: T) -> Option<usize>;
}

impl<T: PartialEq, I: Iterator<Item = T>> IndexOf<T> for I {
    fn index_of(mut self, element: I::Item) -> Option<usize> {
        self.position(|cand| cand == element)
    }
}

pub fn display_option<T: Display>(value: Option<T>) -> impl Display {
    struct Wrapper<T>(Option<T>);
    impl<T: Display> Display for Wrapper<T> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            match &self.0 {
                None => write!(f, "None"),
                Some(value) => write!(f, "Some({})", value),
            }
        }
    }
    Wrapper(value)
}

pub fn display_option_empty<T: Display>(value: Option<T>) -> impl Display {
    struct Wrapper<T>(Option<T>);
    impl<T: Display> Display for Wrapper<T> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            match &self.0 {
                None => write!(f, ""),
                Some(value) => write!(f, "{}", value),
            }
        }
    }
    Wrapper(value)
}

/// Calculates `D_KDL(P || Q)`, read as _the divergence of P from Q_,
/// a measure of how different two probability distributions are.
/// See https://en.wikipedia.org/wiki/Kullback%E2%80%93Leibler_divergence.
pub fn kdl_divergence(p: &[f32], q: &[f32]) -> f32 {
    assert_eq!(p.len(), q.len());

    zip(p, q).map(|(&p, &q)| p * (p / q).ln()).sum()
}

pub fn zip_eq_exact<L, R, LI, RI>(left: L, right: R) -> Zip<LI, RI>
where
    L: IntoIterator<IntoIter = LI>,
    R: IntoIterator<IntoIter = RI>,
    LI: ExactSizeIterator,
    RI: ExactSizeIterator,
{
    let left = left.into_iter();
    let right = right.into_iter();
    assert_eq!(left.len(), right.len(), "Both iterators must have the same length");
    left.zip(right)
}

#[derive(Debug)]
pub struct PrintThroughput {
    name: String,
    total_count: u64,
    delta_count: u64,
    update_count: u64,
    last_print: Instant,
}

impl PrintThroughput {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            total_count: 0,
            delta_count: 0,
            update_count: 0,
            last_print: Instant::now(),
        }
    }

    pub fn total_count(&self) -> u64 {
        self.total_count
    }

    pub fn update_delta(&mut self, count: u64) -> bool {
        self.delta_count += count;
        self.total_count += count;
        self.update_count += 1;

        let now = Instant::now();
        let delta = now - self.last_print;

        let print = delta.as_secs() >= 1 && self.update_count >= 10;
        if print {
            self.print_tp(now);
        }
        print
    }

    pub fn update_total(&mut self, count: u64) -> bool {
        assert!(count >= self.total_count, "Count must be increasing");
        self.update_delta(count - self.total_count)
    }

    fn print_tp(&mut self, now: Instant) {
        let delta = now - self.last_print;
        let throughput = self.delta_count as f32 / delta.as_secs_f32();
        println!(
            "{:.3} {}/s => {:.3} {}",
            throughput, self.name, self.total_count, self.name
        );

        self.last_print = now;
        self.delta_count = 0;
        self.update_count = 0;
    }
}

impl Drop for PrintThroughput {
    fn drop(&mut self) {
        self.print_tp(Instant::now());
    }
}

pub trait Pad {
    type T;
    fn pad(&mut self, result_size: usize, value: Self::T);
}

impl<T: Clone> Pad for Vec<T> {
    type T = T;
    fn pad(&mut self, result_size: usize, value: T) {
        assert!(
            result_size >= self.len(),
            "Cannot pad to smaller size, curr {} target {}",
            self.len(),
            result_size
        );
        self.resize(result_size, value)
    }
}

/// Get the indices of the highest `k` values. The indices themselves are sorted from high to low as well.
/// `NaN` values are allowed but considered higher then any others, to ensure they don't go unnoticed.
pub fn top_k_indices_sorted(values: &[f32], k: usize) -> Vec<usize> {
    fn compare(a: f32, b: f32) -> Ordering {
        let ord = a.partial_cmp(&b);
        let eq = a == b || (a.is_nan() && b.is_nan());
        let first_nan = a.is_nan();

        match (ord, eq, first_nan) {
            (Some(ord), _, _) => ord,
            (None, true, _) => Ordering::Equal,
            (None, false, true) => Ordering::Greater,
            (None, false, false) => Ordering::Less,
        }
    }

    let compare_index = |&i: &usize, &j: &usize| compare(values[i], values[j]).reverse();

    let n = values.len();
    let mut result = (0..n).collect_vec();
    if k < n {
        result.select_nth_unstable_by(k, compare_index);
        result.truncate(k);
    }
    result.sort_by(compare_index);

    result
}

#[cfg(test)]
mod test {
    use crate::top_k_indices_sorted;

    #[test]
    fn top_k() {
        assert_eq!(top_k_indices_sorted(&[0.0, 2.0, 1.0], 2), vec![1, 2]);
        assert_eq!(top_k_indices_sorted(&[1.0, 2.0, 3.0], 20), vec![2, 1, 0]);
        assert_eq!(top_k_indices_sorted(&[1.0, 2.0, 3.0], 0), vec![]);
        assert_eq!(top_k_indices_sorted(&[f32::NAN, 2.0, 1.0], 2), vec![0, 1]);

        let result = top_k_indices_sorted(&[f32::NAN, 2.0, f32::NAN], 2);
        assert!(result == vec![0, 2] || result == vec![2, 0]);
    }
}
