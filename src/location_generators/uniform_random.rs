use std::sync::{atomic::AtomicUsize, atomic::Ordering, Arc};

use num::complex::Complex64;
use rand;
use rand::{Rng, SeedableRng};

pub struct UniformRandomLocationGenerator {
    min: Complex64,
    max: Complex64,
    total: usize,
    current: Arc<AtomicUsize>,

    section_total: usize,
    section_current: usize,

    rng: rand::prng::XorShiftRng,
}

impl UniformRandomLocationGenerator {
    pub fn new(
        min: Complex64,
        max: Complex64,
        total: usize,
        section_total: usize,
    ) -> UniformRandomLocationGenerator {
        UniformRandomLocationGenerator {
            min,
            max,
            total,
            current: Arc::new(AtomicUsize::new(0)),

            section_total,
            section_current: 0,

            rng: rand::prng::XorShiftRng::from_rng(rand::thread_rng()).unwrap(),
        }
    }
}

impl ::location_generators::LocationGenerator<Complex64> for UniformRandomLocationGenerator {
    fn next_location(&mut self) -> Option<Complex64> {
        self.section_current += 1;
        if self.section_current >= self.section_total {
            let current = self.current.load(Ordering::Relaxed) + self.section_total;

            println!(
                "Finished section {}/{}",
                current / self.section_total,
                self.total / self.section_total
            );

            if current >= self.total {
                return None;
            }

            self.current
                .fetch_add(self.section_current, Ordering::Relaxed);
            self.section_current = 0;
        }

        Some(Complex64::new(
            self.rng.gen_range(self.min.re, self.max.re),
            self.rng.gen_range(self.min.im, self.max.im),
        ))
    }
}

impl Clone for UniformRandomLocationGenerator {
    fn clone(&self) -> UniformRandomLocationGenerator {
        UniformRandomLocationGenerator {
            min: self.min,
            max: self.max,
            total: self.total,
            current: self.current.clone(),

            section_total: self.section_total,
            section_current: 0,

            rng: rand::prng::XorShiftRng::from_rng(rand::thread_rng()).unwrap(),
        }
    }
}
