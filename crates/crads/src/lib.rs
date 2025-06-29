use std::{
    io::stdin,
    ops::{Add, BitXor, Index, IndexMut, Mul, Range, Rem, Sub},
};

pub struct Grid<T: Default + Clone> {
    inner: Vec<T>,
    cols: usize,
}


impl<T: Default + Clone> Grid<T> {
    pub fn new(rows: usize, cols: usize) -> Grid<T> {
        let inner = vec![Default::default(); rows * cols];
        Grid {
            inner,
            cols
        }
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn rows(&self) -> usize {
        self.inner.len() / self.cols
    }

    fn inner_index(&self, index: (usize, usize)) -> usize {
        index.0 * self.cols + index.1
    }
}

#[derive(Debug)]
pub struct NotAGridError;

impl<T: Default + Clone> TryFrom<Vec<Vec<T>>> for Grid<T> {
    type Error = NotAGridError;

    fn try_from(value: Vec<Vec<T>>) -> Result<Self, Self::Error> {
        let rows = value.len();
        if rows == 0 {
            return Ok(Grid::new(0, 0));
        }
        let cols = value[0].len();
        for row in &value[1..] {
            if row.len() != cols {
                return Err(NotAGridError);
            }
        }
        Ok(Grid {
            inner: value.into_iter().flatten().collect(),
            cols,
        })
    }
}


impl<T: Default + Clone> Index<(usize, usize)> for Grid<T> {
    type Output = T;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        self.inner.index(self.inner_index(index))
    }
}

impl<T: Default + Clone> IndexMut<(usize, usize)> for Grid<T> {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        self.inner.index_mut(self.inner_index(index))
    }
}

pub struct GridSum<T: Add<Output = T> + Default + Copy + Sub<Output = T>> {
    grid: Grid<T>,
}

impl<T: Add<Output = T> + Default + Copy + Sub<Output = T>> GridSum<T> {
    pub fn sum(&self, topleft: (usize, usize), bottomright: (usize, usize)) -> T {
        assert!(bottomright.0 >= topleft.0);
        assert!(bottomright.1 >= topleft.1);
        let mut result = self.grid[bottomright];
        if topleft.0 > 0 && topleft.1 > 0 {
            result = result + self.grid[(topleft.0-1, topleft.1-1)];
        }
        if topleft.1 > 0 {
            result = result - self.grid[(bottomright.0, topleft.1-1)];
        }
        if topleft.0 > 0 {
            result = result - self.grid[(topleft.0-1, bottomright.1)];
        }
        result
    }
}

impl<T: Add<Output = T> + Default + Copy + Sub<Output = T>> From<Grid<T>> for GridSum<T> {
    fn from(mut grid: Grid<T>) -> Self {
        for (i, j) in (0..grid.rows()).cartesian_product(0..grid.cols()) {
            if i > 0 {
                grid[(i,j)] = grid[(i,j)] + grid[(i-1,j)];
            }
            if j > 0 {
                grid[(i,j)] = grid[(i,j)] + grid[(i,j-1)];
            }
            if i > 0 && j > 0 {
                grid[(i,j)] = grid[(i,j)] - grid[(i-1,j-1)];
            }
        }
        GridSum { grid }
    }
}


pub struct PrefixSum<T: Add<Output = T> + Default + Copy + Sub<Output = T>> {
    sums: Vec<T>,
}
impl<T: Add<Output = T> + Default + Copy + Sub<Output = T>> PrefixSum<T> {
    pub fn new<X: Into<T> + Copy>(values: &[X]) -> Self {
        let sums: Vec<T> = values
            .iter()
            .map(|&x| x.into())
            .scan(T::default(), |sum, x| {
                *sum = sum.add(x);
                Some(*sum)
            })
            .collect();
        PrefixSum { sums }
    }
    pub fn sum(&self, range: std::ops::Range<usize>) -> T {
        assert!(range.end <= self.sums.len());
        if range.is_empty() {
            return T::default();
        }
        self.sums[range.end - 1].sub(if range.start == 0 {
            T::default()
        } else {
            self.sums[range.start - 1]
        })
    }
}

pub struct PrefixXor<T: Default + BitXor<Output = T> + Copy> {
    xors: Vec<T>,
}
impl<T: Default + BitXor<Output = T> + Copy> PrefixXor<T> {
    pub fn new(values: &[T]) -> Self {
        let xors: Vec<T> = values
            .iter()
            .scan(T::default(), |xor, x| {
                *xor = *xor ^ *x;
                Some(*xor)
            })
            .collect();
        PrefixXor { xors }
    }
    pub fn xor(&self, range: Range<usize>) -> T {
        assert!(range.end <= self.xors.len());
        if range.is_empty() {
            return T::default();
        }
        self.xors[range.end - 1]
            ^ (if range.start == 0 {
                T::default()
            } else {
                self.xors[range.start - 1]
            })
    }
}

/// Scanner from EbTech
#[derive(Default)]
pub struct Scanner {
    buffer: Vec<String>,
}
impl Scanner {
    #[allow(clippy::should_implement_trait)]
    pub fn next<T: std::str::FromStr>(&mut self) -> T {
        loop {
            if let Some(token) = self.buffer.pop() {
                return token.parse().ok().expect("Failed parse");
            }
            let mut input = String::new();
            stdin().read_line(&mut input).expect("Failed read");
            self.buffer = input.split_whitespace().rev().map(String::from).collect();
        }
    }
}

#[allow(dead_code)]
pub trait One {
    fn one() -> Self;
}

macro_rules! one_impl {
    ($num_type:ty) => {
        impl One for $num_type {
            fn one() -> Self {
                1
            }
        }
    };
}

one_impl!(i8);
one_impl!(i16);
one_impl!(i32);
one_impl!(i64);
one_impl!(i128);
one_impl!(u8);
one_impl!(u16);
one_impl!(u32);
one_impl!(u64);
one_impl!(u128);

#[allow(dead_code)]
pub trait Zero {
    fn zero() -> Self;
}

macro_rules! zero_impl {
    ($type:ty) => {
        impl Zero for $type {
            fn zero() -> Self {
                0
            }
        }
    };
}

zero_impl!(i8);
zero_impl!(i16);
zero_impl!(i32);
zero_impl!(i64);
zero_impl!(i128);
zero_impl!(u8);
zero_impl!(u16);
zero_impl!(u32);
zero_impl!(u64);
zero_impl!(u128);

pub fn mat_mul_mod<
    T: Copy + Zero + Mul<Output = T> + Add<Output = T> + Rem<Output = T>,
    const N: usize,
>(
    a: &[[T; N]; N],
    b: &[[T; N]; N],
    modulo: T,
) -> [[T; N]; N] {
    let zero = T::zero();
    let mut c: [[T; N]; N] = [[zero; N]; N];
    for row in 0..N {
        for col in 0..N {
            #[allow(clippy::needless_range_loop)]
            for i in 0..N {
                c[row][col] = (c[row][col] + ((a[row][i] * b[i][col]) % modulo)) % modulo;
            }
        }
    }
    c
}

pub fn matrix_power_mod<
    T: Zero + One + Copy + Default + Mul<Output = T> + Add<Output = T> + Rem<Output = T>,
    const N: usize,
>(
    mut a: [[T; N]; N],
    mut p: u32,
    modulo: T,
) -> [[T; N]; N] {
    let identity = {
        let zero = T::zero();
        let one = T::one();
        let mut identity = [[zero; N]; N];
        #[allow(clippy::needless_range_loop)]
        for i in 0..N {
            identity[i][i] = one;
        }
        identity
    };
    let mut b = identity;
    while p > 0 {
        if p % 2 == 1 {
            b = mat_mul_mod(&b, &a, modulo);
        }
        p /= 2;
        a = mat_mul_mod(&a, &a, modulo);
    }
    b
}

pub struct AllVectorsIterator<T> {
    vector: Vec<T>,
    min_val: T,
    max_val: T,
}

impl<T: Clone + PartialOrd> AllVectorsIterator<T> {
    fn new(len: usize, min_val: T, max_val: T) -> Self {
        assert!(min_val <= max_val);
        Self {
            vector: vec![min_val.clone(); len],
            min_val,
            max_val,
        }
    }
}

impl<T: PartialOrd + One + Add<Output = T> + Clone> Iterator for AllVectorsIterator<T> {
    type Item = Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        // find the last position in the vector that can be incremented
        if let Some((i, _)) = self
            .vector
            .iter()
            .enumerate()
            .rev()
            .find(|(_i, x)| x < &&self.max_val)
        {
            self.vector[i] = self.vector[i].clone() + T::one();
            for j in i + 1..self.vector.len() {
                self.vector[j] = self.min_val.clone();
            }
            Some(self.vector.clone())
        } else {
            // no positions are left to increment
            debug_assert!(self.vector.iter().all(|x| x == &self.max_val));
            None
        }
    }
}

pub fn iter_all_vectors<T: PartialOrd + One + Add<Output = T> + Clone>(
    min_len: usize,
    max_len: usize,
    min_val: T,
    max_val: T,
) -> impl Iterator<Item = Vec<T>> {
    (min_len..=max_len)
        .flat_map(move |len| AllVectorsIterator::new(len, min_val.clone(), max_val.clone()))
}

pub fn sorted<I: IntoIterator<Item = T>, T: Ord>(iter: I) -> bool {
    let mut iter = iter.into_iter();
    if let Some(mut prev) = iter.next() {
        while let Some(curr) = iter.next() {
            if prev > curr {
                return false;
            }
            prev = curr
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sorted() {
        assert!(sorted([1, 1, 2, 2, 3, 4, 4]));
        assert!(!sorted([0, 1, 1, 0, 1, 2, 3]));
        assert!(sorted::<[i32; 0], i32>([]));
        assert!(sorted([1]));
        assert!(!sorted([2, 1]));
    }
}

pub use self::itertools::Itertools;

mod itertools {
    // Adapted from itertools (https://github.com/rust-itertools/itertools)

    use self::adaptors::Product;
    pub trait Itertools: Iterator {
        /// Return an iterator adaptor that iterates over the cartesian product of
        /// the element sets of two iterators `self` and `J`.
        ///
        /// Iterator element type is `(Self::Item, J::Item)`.
        ///
        /// ```
        /// use crads::Itertools;
        ///
        /// let it = (0..2).cartesian_product("αβ".chars());
        /// assert_eq!(it.collect::<Vec<_>>(), vec![(0, 'α'), (0, 'β'), (1, 'α'), (1, 'β')]);
        /// ```
        fn cartesian_product<J>(self, other: J) -> Product<Self, J::IntoIter>
        where
            Self: Sized,
            Self::Item: Clone,
            J: IntoIterator,
            J::IntoIter: Clone,
        {
            adaptors::cartesian_product(self, other.into_iter())
        }
    }

    impl<T> Itertools for T where T: Iterator + ?Sized {}

    mod adaptors {
        use super::size_hint;

        #[derive(Debug, Clone)]
        /// An iterator adaptor that iterates over the cartesian product of
        /// the element sets of two iterators `I` and `J`.
        ///
        /// Iterator element type is `(I::Item, J::Item)`.
        ///
        /// See [`.cartesian_product()`](crate::Itertools::cartesian_product) for more information.
        #[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
        pub struct Product<I, J>
        where
            I: Iterator,
        {
            a: I,
            /// `a_cur` is `None` while no item have been taken out of `a` (at definition).
            /// Then `a_cur` will be `Some(Some(item))` until `a` is exhausted,
            /// in which case `a_cur` will be `Some(None)`.
            a_cur: Option<Option<I::Item>>,
            b: J,
            b_orig: J,
        }

        /// Create a new cartesian product iterator
        ///
        /// Iterator element type is `(I::Item, J::Item)`.
        pub fn cartesian_product<I, J>(i: I, j: J) -> Product<I, J>
        where
            I: Iterator,
            J: Clone + Iterator,
            I::Item: Clone,
        {
            Product {
                a_cur: None,
                a: i,
                b: j.clone(),
                b_orig: j,
            }
        }

        impl<I, J> Iterator for Product<I, J>
        where
            I: Iterator,
            J: Clone + Iterator,
            I::Item: Clone,
        {
            type Item = (I::Item, J::Item);

            fn next(&mut self) -> Option<Self::Item> {
                let Self {
                    a,
                    a_cur,
                    b,
                    b_orig,
                } = self;
                let elt_b = match b.next() {
                    None => {
                        *b = b_orig.clone();
                        match b.next() {
                            None => return None,
                            Some(x) => {
                                *a_cur = Some(a.next());
                                x
                            }
                        }
                    }
                    Some(x) => x,
                };
                a_cur
                    .get_or_insert_with(|| a.next())
                    .as_ref()
                    .map(|a| (a.clone(), elt_b))
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                // Not ExactSizeIterator because size may be larger than usize
                // Compute a * b_orig + b for both lower and upper bound
                let mut sh = size_hint::mul(self.a.size_hint(), self.b_orig.size_hint());
                if matches!(self.a_cur, Some(Some(_))) {
                    sh = size_hint::add(sh, self.b.size_hint());
                }
                sh
            }

            fn fold<Acc, G>(self, mut accum: Acc, mut f: G) -> Acc
            where
                G: FnMut(Acc, Self::Item) -> Acc,
            {
                // use a split loop to handle the loose a_cur as well as avoiding to
                // clone b_orig at the end.
                let Self {
                    mut a,
                    a_cur,
                    mut b,
                    b_orig,
                } = self;
                if let Some(mut elt_a) = a_cur.unwrap_or_else(|| a.next()) {
                    loop {
                        accum = b.fold(accum, |acc, elt| f(acc, (elt_a.clone(), elt)));

                        // we can only continue iterating a if we had a first element;
                        if let Some(next_elt_a) = a.next() {
                            b = b_orig.clone();
                            elt_a = next_elt_a;
                        } else {
                            break;
                        }
                    }
                }
                accum
            }
        }
    }

    mod size_hint {
        //! Arithmetic on `Iterator.size_hint()` values.
        //!

        /// `SizeHint` is the return type of `Iterator::size_hint()`.
        pub type SizeHint = (usize, Option<usize>);

        /// Add `SizeHint` correctly.
        #[inline]
        pub fn add(a: SizeHint, b: SizeHint) -> SizeHint {
            let min = a.0.saturating_add(b.0);
            let max = match (a.1, b.1) {
                (Some(x), Some(y)) => x.checked_add(y),
                _ => None,
            };

            (min, max)
        }

        /// Multiply `SizeHint` correctly
        #[inline]
        pub fn mul(a: SizeHint, b: SizeHint) -> SizeHint {
            let low = a.0.saturating_mul(b.0);
            let hi = match (a.1, b.1) {
                (Some(x), Some(y)) => x.checked_mul(y),
                (Some(0), None) | (None, Some(0)) => Some(0),
                _ => None,
            };
            (low, hi)
        }
    }
}
