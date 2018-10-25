extern crate num;
extern crate rand;

use std::thread;

use num::complex::Complex64;
use std::sync::mpsc;

pub mod aggregators;
pub mod location_generators;
pub mod math;
use location_generators::LocationGenerator;

fn main() {
    // Calculating - Mathematics
    let function = |c: Complex64| move |z: Complex64| z * z + c;

    let bailout_min = Complex64::new(-2.0, -2.0);
    let bailout_max = Complex64::new(2.0, 2.0);

    // Calculating - Algorithms
    let scan_min = Complex64::new(-2.0, -2.0);
    let scan_max = Complex64::new(2.0, 2.0);
    let iterations = 5;
    let samples = 1e8 as usize;
    let sample_section = 1e5 as usize;

    // Threading
    let threads = 4;
    let channel_buffer = 4;

    // Aggregation
    let file_buffer_size = 1e7 as usize;
    let pixel_buffer_cutoff_size = 1e5 as usize;

    // MBH output
    let mbh_width = 3000;
    let mbh_height = 3000;
    let mbh_min = Complex64::new(-2.0, -2.0);
    let mbh_max = Complex64::new(2.0, 2.0);

    let file_name = "image.mbh";

    {
        let location_generator = location_generators::UniformRandomLocationGenerator::new(
            scan_min,
            scan_max,
            samples,
            sample_section,
        );

        let (sender, receiver) = mpsc::sync_channel::<Option<Vec<Complex64>>>(channel_buffer);

        for thread_id in 0..threads {
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

                    if result.len() > 0
                        && !math::complex_between(
                            bailout_min,
                            result[result.len() - 1],
                            bailout_max,
                        ) {
                        sender.send(Some(result)).expect("Sender closed too early");
                    }
                }
                sender.send(None).expect("Sender closed too early");

                println!("Thread {} done", thread_id);
            });
        }

        let mut aggregator = aggregators::FileAggregator::create(
            file_name,
            mbh_width,
            mbh_height,
            mbh_min,
            mbh_max,
            file_buffer_size,
            pixel_buffer_cutoff_size,
        ).expect("Error while setting up aggregator");

        let mut received = 0;
        while received < threads {
            let result = receiver.recv().unwrap();
            if let Some(result) = result {
                for c in result {
                    aggregator.aggregate(c);
                }
            } else {
                received += 1;
            }
        }
    }
}
