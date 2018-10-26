use std::sync::{Arc, Mutex};

use num::complex::Complex64;
use rand;
use rand::Rng;

pub struct ArrayLocationGenerator {
    locations: Arc<Mutex<Vec<Complex64>>>,

    current: Option<Complex64>,
    count: u64,
    per_point: u64,
    delta: f64,
}
impl ArrayLocationGenerator {
    pub fn new(locations: Vec<Complex64>, per_point: u64, delta: f64) -> ArrayLocationGenerator {
        ArrayLocationGenerator {
            locations: Arc::new(Mutex::new(locations)),
            current: None,
            count: 0,
            per_point,
            delta,
        }
    }
}

impl ::location_generators::LocationGenerator<Complex64> for ArrayLocationGenerator {
    fn next_location(&mut self) -> Option<Complex64> {
        if self.count >= self.per_point || self.current.is_none() {
            self.count = 0;
            self.current = self.locations.lock().unwrap().pop();

            if !self.current.is_some() {
                return None;
            }
        }

        self.count += 1;

        let current = self.current.unwrap();
        Some(Complex64::new(
            current.re + rand::thread_rng().gen_range(-self.delta, self.delta),
            current.im + rand::thread_rng().gen_range(-self.delta, self.delta),
        ))
    }
}

impl Clone for ArrayLocationGenerator {
    fn clone(&self) -> ArrayLocationGenerator {
        ArrayLocationGenerator {
            locations: self.locations.clone(),
            current: None,
            count: 0,
            per_point: self.per_point,
            delta: self.delta,
        }
    }
}
