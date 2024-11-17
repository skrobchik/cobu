use std::{
    io::stdin,
    ops::{Add, BitXor, Mul, Range, Rem, Sub},
};

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
