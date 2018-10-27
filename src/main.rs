extern crate image as file_image;
extern crate num;
extern crate rand;

use std::fs::OpenOptions;
use std::sync::mpsc;
use std::thread;

use num::complex::Complex64;

pub mod aggregators;
pub mod eta;
pub mod file;
pub mod image;
pub mod location_generators;
pub mod math;
pub mod vec;
use location_generators::LocationGenerator;

#[derive(Clone, Copy)]
struct CalculateNext {
    c: Complex64,
}
impl CalculateNext {
    fn get(&self, c: Complex64) -> CalculateNext {
        CalculateNext { c }
    }
}
impl math::CalculateNext for CalculateNext {
    fn next(&mut self, z: Complex64) -> Complex64 {
        z * z + self.c
    }
}

#[derive(Clone, Copy)]
struct Config<'a> {
    function: CalculateNext,
    initial_z: Complex64,

    bailout_min: Complex64,
    bailout_max: Complex64,

    scan_min: Complex64,
    scan_max: Complex64,
    iterations: usize,
    samples: usize,
    sample_section: usize,

    eta_section: usize,
    eta_time: u64,

    threads: usize,
    channel_buffer: usize,
    thread_buffer: usize,

    file_buffer_size: usize,
    pixel_buffer_cutoff_size: usize,

    mbh_width: u64,
    mbh_height: u64,
    mbh_min: Complex64,
    mbh_max: Complex64,
    mbh_file_name: &'a str,

    image_file_name: &'a str,
}


fn main() {
    let config = Config {
        function: CalculateNext {
            c: Complex64::new(0.0, 0.0),
        },
        initial_z: Complex64::new(0.0, 0.0),

        bailout_min: Complex64::new(-2.0, -2.0),
        bailout_max: Complex64::new(2.0, 2.0),

        scan_min: Complex64::new(-2.0, -2.0),
        scan_max: Complex64::new(2.0, 2.0),
        iterations: 10_000,
        samples: 1e8 as usize,
        sample_section: 1e6 as usize,

        eta_section: 10_000,
        eta_time: 1000,

        threads: 4,
        channel_buffer: 50,
        thread_buffer: 1_000_000,

        file_buffer_size: 1e6 as usize,
        pixel_buffer_cutoff_size: 1e6 as usize,

        mbh_width: 1000,
        mbh_height: 1000,
        mbh_min: Complex64::new(-2.0, -2.0),
        mbh_max: Complex64::new(2.0, 2.0),
        mbh_file_name: "image.mbh",

        image_file_name: "image.png",
    };

    {
        println!(
            "Estimated maximum RAM usage: {}mb",
            ((config.threads + config.channel_buffer) * config.thread_buffer * 2 * 8
                + (config.mbh_width as usize * config.mbh_height as usize
                    / config.file_buffer_size
                    + 1)
                    * config.pixel_buffer_cutoff_size)
                / 1000000
        );
        println!(
            "Estimated typical maximum RAM usage: {}mb",
            (config.threads * config.thread_buffer * 2 * 8
                + (config.channel_buffer / 2) * config.thread_buffer * 2 * 8
                + (config.mbh_width as usize * config.mbh_height as usize
                    / config.file_buffer_size
                    + 1)
                    * (config.pixel_buffer_cutoff_size / 2))
                / 1000000
        );
        let location_generator = location_generators::UniformRandomLocationGenerator::new(
            config.scan_min,
            config.scan_max,
            config.samples,
            config.sample_section,
        );
        let eta = eta::ETA::new(config.samples, config.eta_section, config.eta_time);

        let (sender, receiver) =
            mpsc::sync_channel::<Option<Vec<Complex64>>>(config.channel_buffer);

        for thread_id in 0..config.threads {
            let mut location_generator = location_generator.clone();
            let mut eta = eta.clone();
            let sender = sender.clone();

            thread::Builder::new()
                .name(format!("Calculator {}", thread_id))
                .spawn(move || {
                    println!("Starting thread {}", thread_id);

                    let mut result_cache = Vec::with_capacity(config.thread_buffer);
                    while let Some(c) = location_generator.next_location() {
                        eta.count();

                        if math::is_inside_mandelbrot_bulb(c) {
                            continue;
                        }

                        if math::calculate_bailout_iteration(
                            &mut config.function.get(c),
                            config.initial_z,
                            config.bailout_min,
                            config.bailout_max,
                            config.iterations,
                        ).is_some()
                        {
                            math::calculate_iteration_values(
                                &mut config.function.get(c),
                                config.initial_z,
                                config.bailout_min,
                                config.bailout_max,
                                config.iterations,
                                &mut result_cache,
                            );
                        }

                        if result_cache.len() > config.thread_buffer {
                            match sender.try_send(Some(result_cache)) {
                                Ok(_) => {}
                                Err(mpsc::TrySendError::Full(result_cache)) => {
                                    println!("Bottleneck while sending");
                                    sender.send(result_cache).expect("Sender closed too early");
                                }
                                _ => panic!(),
                            }

                            result_cache = Vec::with_capacity(config.thread_buffer);
                        }
                    }
                    sender
                        .send(Some(result_cache))
                        .expect("Sender closed too early");
                    sender.send(None).expect("Sender closed too early");

                    println!("Thread {} done", thread_id);
                }).expect("Unable to start thread");
        }

        let aggregator_thread = thread::Builder::new()
            .name("Aggregator".to_owned())
            .spawn(move || {
                let mut aggregator = aggregators::FileAggregator::create(
                    config.mbh_file_name,
                    config.mbh_width,
                    config.mbh_height,
                    config.mbh_min,
                    config.mbh_max,
                    config.file_buffer_size,
                    config.pixel_buffer_cutoff_size,
                ).expect("Error while setting up aggregator");

                let mut received = 0;
                while received < config.threads {
                    let result = receiver.recv().unwrap();
                    if let Some(result) = result {
                        for c in result {
                            aggregator.aggregate(c);
                        }
                    } else {
                        received += 1;
                    }
                }
            }).expect("Unable to start thread");
        aggregator_thread.join().expect("Aggregator crashed");
    }

    {
        // TODO separate image size; downsampling
        let mut file = OpenOptions::new()
            .read(true)
            .open(config.mbh_file_name)
            .expect("Could not open file");

        let image = image::ImageData::read_fully(
            &mut file,
            config.mbh_width as usize,
            config.mbh_height as usize,
        ).expect("Could not read file");

        // TODO tiled writing?

        println!("Saving image");
        image
            .map_sqrt_height()
            .save(config.image_file_name)
            .unwrap();
    }
}
