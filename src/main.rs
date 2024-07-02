#![feature(stdarch_x86_avx512)]
#![feature(portable_simd)]
#![feature(iterator_try_collect)]
// use std::arch::x86_64::_mm256_permutexvar_epi8;
use std::fmt;
use std::iter;
use std::ops;
use std::simd::u8x32;

#[derive(Clone, Copy, Debug)]
struct Permutation(u8x32);

impl Permutation {
    fn cycles(&self) -> Vec<Vec<u8>> {
        let mut done = [false; 32];
        let mut cycles = Vec::with_capacity(1);
        for i in 0..32 {
            if done[i as usize] {
                continue;
            }
            if self[i] == i {
                done[i as usize] = true;
                continue;
            }
            let mut cycle = Vec::with_capacity(2);
            let mut j = i;
            while !done[j as usize] {
                cycle.push(j);
                done[j as usize] = true;
                j = self[j];
            }
            cycles.push(cycle);
        }
        cycles
    }

    fn from_array(arr: &[u8]) -> Permutation {
        Permutation(u8x32::load_or(arr, u8x32::from_array(OMEGA)).into())
    }

    fn from_cycles<T, R>(cycles: T) -> Permutation
    where
        T: IntoIterator<Item = R>,
        R: IntoIterator<Item = u8>,
    {
        let mut p = OMEGA;
        for cycle in cycles {
            let mut iter = cycle.into_iter();
            if let Some(fst) = iter.next() {
                let mut i = fst;
                while let Some(j) = iter.next() {
                    p[i as usize] = j;
                    i = j;
                }
                p[i as usize] = fst;
            }
        }
        Permutation::from_array(&p)
    }

    fn parse(s: &str) -> Option<Permutation> {
        Some(Permutation::from_cycles(
            s.strip_prefix("(")?
                .strip_suffix(")")?
                .split(")(")
                .map(|x| {
                    x.chars()
                        .map(|c| u8::try_from(c.to_digit(32)?).ok())
                        .try_collect::<Vec<u8>>()
                })
                .try_collect::<Vec<Vec<u8>>>()?,
        ))
    }
}

impl ops::Mul for Permutation {
    type Output = Permutation;

    fn mul(self, rhs: Permutation) -> Permutation {
        // Permutation(unsafe { _mm256_permutexvar_epi8(self.0.into(), rhs.0.into()).into() })
        Permutation(self.0.swizzle_dyn(rhs.0))
    }
}

impl ops::Index<u8> for Permutation {
    type Output = u8;

    fn index(&self, index: u8) -> &u8 {
        self.0.index(index as usize)
    }
}

impl fmt::Display for Permutation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let cycles = self.cycles();
        if cycles.len() == 0 {
            write!(f, "()")
        } else {
            cycles
                .into_iter()
                .map(|cycle| {
                    write!(f, "(")?;
                    cycle
                        .into_iter()
                        .map(|i| write!(f, "{}", i))
                        .collect::<fmt::Result>()?;
                    write!(f, ")")
                })
                .collect()
        }
    }
}

const OMEGA: [u8; 32] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31,
];

fn orbit<T>(generators: T, point: u8) -> [Option<Permutation>; 32]
where
    T: IntoIterator<Item = Permutation>,
{
    let mut reps = [None; 32];
    reps[point as usize] = Some(Permutation::from_array(&OMEGA));
    let mut orbit = Vec::with_capacity(32);
    orbit.push(point);
    for generator in generators {
        let mut i = 0;
        while i < orbit.len() {
            let rep = reps[orbit[i] as usize].unwrap();
            let image = generator[orbit[i]];
            let action = rep * generator;
            if reps[image as usize].is_none() {
                reps[image as usize] = Some(action);
                orbit.push(image);
            }
            i += 1;
        }
    }
    reps
}

fn g<'a, T>(
    generators: T,
) -> iter::FilterMap<<T as IntoIterator>::IntoIter, for<'b> fn(&'b str) -> Option<Permutation>>
where
    T: IntoIterator<Item = &'a str>,
{
    generators.into_iter().filter_map(Permutation::parse)
}

fn main() {
    let res = orbit(g(["(123)", "(12)", "(14)(25)(36)"]), 1);
    for (i, g) in res.into_iter().enumerate() {
        if let Some(g) = g {
            println!("{}: {}", i, g);
        }
    }
}
