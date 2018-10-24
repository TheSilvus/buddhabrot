extern crate num;
extern crate rand;

use std::thread;

use num::complex::Complex64;
use std::sync::mpsc;

pub mod location_generators;
pub mod math;
use location_generators::LocationGenerator;

const THREADS: usize = 4;
const CHANNEL_BUFFER: usize = 4;

fn main() {
    let iterations = 5;
    let function = |c: Complex64| move |z: Complex64| z * z + c;

    let bailout_min = Complex64::new(-2.0, -2.0);
    let bailout_max = Complex64::new(2.0, 2.0);

    {
        let location_generator = location_generators::UniformRandomLocationGenerator::new(
            Complex64::new(-2.0, -2.0),
            Complex64::new(2.0, 2.0),
            50,
            10,
        );

        let (sender, receiver) = mpsc::sync_channel::<Option<Vec<Complex64>>>(CHANNEL_BUFFER);

        for thread_id in 0..THREADS {
            let mut location_generator = location_generator.clone();
            let sender = sender.clone();

            thread::spawn(move || {
                println!("Starting thread {}", thread_id);
                while let Some(c) = location_generator.next_location() {
                    let result = math::calculate_iteration_values(
                        &function(c),
                        Complex64::new(0.0, 0.0),
                        bailout_min,
                        bailout_max,
                        iterations,
                    );

                    if result.len() > 0 && !math::complex_between(bailout_min, result[result.len() - 1], bailout_max) {
                        sender.send(Some(result)).expect("Sender closed too early");
                    }
                }
                sender.send(None).expect("Sender closed too early");

                println!("Thread {} done", thread_id);
            });
        }

        let mut received = 0;
        while received < THREADS {
            let result = receiver.recv().unwrap();
            if let Some(result) = result {
                println!("{:?}", result);
            } else {
                received += 1;
            }
        }

        thread::sleep_ms(10);
    }
}
