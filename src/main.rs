extern crate num;
extern crate rand;

use num::complex::Complex64;

pub mod location_generators;
use location_generators::LocationGenerator;

fn main() {
    println!("Hello, world!");

    let mut lg = location_generators::UniformRandomLocationGenerator::new(Complex64::new(0.0, 0.0), Complex64::new(1.0, 1.0), 10, 1);

    while let Some(c) = lg.next_task() {
        println!("{}", c);
    }
}
