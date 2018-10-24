extern crate num;
extern crate rand;

use std::thread;

use num::complex::Complex64;

pub mod location_generators;
use location_generators::LocationGenerator;

const THREADS: usize = 8;

fn main() {
    {
        let mut location_generator = location_generators::UniformRandomLocationGenerator::new(
            Complex64::new(-2.0, -2.0),
            Complex64::new(2.0, 2.0),
            50,
            10,
        );

        while let Some(c) = location_generator.next_location() {
            println!("{}", c);
        }

        for thread_id in 0..THREADS {
            thread::spawn(move || {});
        }
    }
}
