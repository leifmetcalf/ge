#![feature(stdarch_x86_avx512)]
#![feature(portable_simd)]
// use std::arch::x86_64::_mm256_permutexvar_epi8;
use std::fmt;
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

    fn from_cycles(cycles: &[&[u8]]) -> Permutation {
        let mut p = OMEGA;
        for cycle in cycles {
            for i in 0..cycle.len() {
                p[cycle[i] as usize] = p[cycle[i + 1 % cycle.len()] as usize]
            }
        }
        Permutation::from_array(&p)
    }

    fn from_array(arr: &[u8]) -> Permutation {
        Permutation(u8x32::load_or(arr, u8x32::from_array(OMEGA)).into())
    }

}

impl std::ops::Mul for Permutation {
    type Output = Permutation;

    fn mul(self, rhs: Permutation) -> Permutation {
        // Permutation(unsafe { _mm256_permutexvar_epi8(self.0.into(), rhs.0.into()).into() })
        Permutation(self.0.swizzle_dyn(rhs.0))
    }
}

impl std::ops::Index<u8> for Permutation {
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

fn _semi_orbit(generators: &[Permutation]) -> [u8; 32] {
    let mut partitions = OMEGA;
    for generator in generators {
        for i in 0..32 {
            let j = generator[i];
            let pi = partitions[i as usize];
            let pj = partitions[j as usize];
            if pi != pj {
                for x in 0..32 {
                    if x == pj {
                        partitions[x as usize] = pi;
                    }
                }
            }
        }
    }
    partitions
}

fn orbit(generators: &[Permutation], point: u8) -> [Option<Permutation>; 32] {
    let mut reps = [None; 32];
    reps[point as usize] = Some(Permutation::from_array(&OMEGA));
    let mut queue = Vec::with_capacity(32);
    queue.push(point);
    for &generator in generators {
        let mut i = 0;
        while i < queue.len() {
            if let Some(rep) = reps[i] {
                let action = rep * generator;
                let image = action[point];
                if reps[image as usize].is_none() {
                    reps[image as usize] = Some(action);
                }
            }
            i += 1;
        }
    }
    reps
}

fn main() {
    let res = orbit(&[Permutation::from_array(&[1, 0, 3, 2])], 0);
    for (i, g) in res.into_iter().enumerate() {
        if let Some(g) = g {
            println!("{}: {}", i, g);
        }
    }
}
