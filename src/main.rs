extern crate image;
extern crate num;
extern crate rand;

use std::fs::OpenOptions;
use std::sync::mpsc;
use std::thread;

use num::complex::Complex64;

pub mod aggregators;
pub mod file;
pub mod location_generators;
pub mod math;
pub mod vec;
pub mod eta;
use location_generators::LocationGenerator;

fn main() {
    // Calculating - Mathematics
    let function = |c: Complex64| move |z: Complex64| z * z + c;
    let initial_z = Complex64::new(0.0, 0.0);

    let bailout_min = Complex64::new(-2.0, -2.0);
    let bailout_max = Complex64::new(2.0, 2.0);

    // Calculating - Algorithms
    let scan_min = Complex64::new(-2.0, -2.0);
    let scan_max = Complex64::new(2.0, 2.0);
    let iterations: usize = 5000;
    let samples: usize = 1e8 as usize;
    let sample_section: usize = 5e5 as usize;

    // ETA
    let eta_section: usize = 100000;
    let eta_time: u64 = 1000;

    // Threading
    let threads: usize = 4;
    let channel_buffer: usize = 500;

    let thread_buffer: usize = 100_000;

    // Aggregation
    let file_buffer_size: usize = 1e6 as usize;
    let pixel_buffer_cutoff_size: usize = 1e6 as usize;

    // MBH output
    let mbh_width: u64 = 5000;
    let mbh_height: u64 = 5000;
    let mbh_min = Complex64::new(-2.0, -2.0);
    let mbh_max = Complex64::new(2.0, 2.0);

    let mbh_file_name = "image.mbh";

    // Image output
    let image_file_name = "image.png";

    println!(
        "Estimated maximum RAM usage: {}mb",
        (threads * thread_buffer * 2 * 8
            + channel_buffer * thread_buffer * 2 * 8
            + (mbh_width as usize * mbh_height as usize / file_buffer_size + 1) * pixel_buffer_cutoff_size)
            / 1000000
    );
    println!(
        "Estimated typical maximum RAM usage: {}mb",
        (threads * thread_buffer * 2 * 8
            + (channel_buffer / 2) * thread_buffer * 2 * 8
            + (mbh_width as usize * mbh_height as usize / file_buffer_size + 1) * pixel_buffer_cutoff_size)
            / 1000000
    );

    {
        let location_generator = location_generators::UniformRandomLocationGenerator::new(
            scan_min,
            scan_max,
            samples,
            sample_section,
        );
        let eta = eta::ETA::new(samples, eta_section, eta_time);

        let (sender, receiver) = mpsc::sync_channel::<Option<Vec<Complex64>>>(channel_buffer);

        for thread_id in 0..threads {
            let mut location_generator = location_generator.clone();
            let mut eta = eta.clone();
            let sender = sender.clone();

            thread::Builder::new()
                .name(format!("Calculator {}", thread_id))
                .spawn(move || {
                    println!("Starting thread {}", thread_id);

                    let mut result_cache = Vec::with_capacity(thread_buffer);
                    while let Some(c) = location_generator.next_location() {
                        eta.count();

                        math::calculate_iteration_values(
                            &function(c),
                            initial_z,
                            bailout_min,
                            bailout_max,
                            iterations,
                            &mut result_cache,
                        );

                        if result_cache.len() > thread_buffer {
                            match sender.try_send(Some(result_cache)) {
                                Ok(_) => {}
                                Err(mpsc::TrySendError::Full(result_cache)) => {
                                    println!("Bottleneck while sending");
                                    sender.send(result_cache).expect("Sender closed too early");
                                }
                                _ => panic!(),
                            }

                            result_cache = Vec::with_capacity(thread_buffer);
                        }
                    }
                    sender.send(None).expect("Sender closed too early");

                    println!("Thread {} done", thread_id);
                }).expect("Unable to start thread");
        }

        let mut aggregator = aggregators::FileAggregator::create(
            mbh_file_name,
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

    {
        // TODO extract into reusable functions
        // TODO separate image size; downsampling
        let mut file = OpenOptions::new()
            .read(true)
            .open(mbh_file_name)
            .expect("Could not open file");

        // TODO tiled writing?
        let mut image = vec::filled_with(0, mbh_width as usize * mbh_height as usize);
        file::read_u32(&mut file, 0, &mut image).unwrap();

        let mut highest_value = 0;
        for height in &image {
            if *height > highest_value {
                highest_value = *height;
            }
        }

        let converted = image
            .iter()
            .map(|i| ((*i as f64 / highest_value as f64) * 256 as f64) as u8)
            .collect::<Vec<_>>();

        image::save_buffer(
            image_file_name,
            &converted[..],
            mbh_width as u32,
            mbh_height as u32,
            image::Gray(8),
        ).unwrap();
    }
}
