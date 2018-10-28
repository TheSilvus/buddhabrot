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
        // z * z * z * Complex64::new((z.re * self.c.im).cos(), (self.c.re * z.im).cos()) + self.c
    }
}

#[derive(Clone)]
struct Config<'a> {
    function: CalculateNext,
    initial_z: Complex64,

    bailout_min: Complex64,
    bailout_max: Complex64,

    scan_min: Complex64,
    scan_max: Complex64,
    samples: usize,
    sample_section: usize,

    check_iterations: usize,
    images: Vec<ImageConfig<'a>>,

    eta_section: usize,
    eta_time: u64,

    threads: usize,
    channel_buffer: usize,
    thread_buffer: usize,

    file_buffer_size: usize,
    pixel_buffer_cutoff_size: usize,

    image_file_name: &'a str,
}
#[derive(Clone)]
struct ImageConfig<'a> {
    min_iterations: usize,
    max_iterations: usize,

    width: u64,
    height: u64,

    min: Complex64,
    max: Complex64,

    file_name: &'a str,
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
        samples: 1e8 as usize,
        sample_section: 1e6 as usize,

        check_iterations: 2000,
        images: vec![
            ImageConfig {
                min_iterations: 0,
                max_iterations: 100,

                width: 1000,
                height: 1000,

                min: Complex64::new(-2.0, -2.0),
                max: Complex64::new(2.0, 2.0),

                file_name: "image-1.mbh",
            },
            ImageConfig {
                min_iterations: 50,
                max_iterations: 500,

                width: 1000,
                height: 1000,

                min: Complex64::new(-2.0, -2.0),
                max: Complex64::new(2.0, 2.0),

                file_name: "image-2.mbh",
            },
            ImageConfig {
                min_iterations: 200,
                max_iterations: 2000,

                width: 1000,
                height: 1000,

                min: Complex64::new(-2.0, -2.0),
                max: Complex64::new(2.0, 2.0),

                file_name: "image-3.mbh",
            },
        ],

        eta_section: 10,
        eta_time: 1000,

        threads: 4,
        channel_buffer: 50,
        thread_buffer: 1_000_000,

        file_buffer_size: 1e6 as usize,
        pixel_buffer_cutoff_size: 1e6 as usize,

        image_file_name: "image.png",
    };

    generate(config.clone());
    image(config.clone());
}

fn generate(config: Config<'static>) {
    println!(
        "Estimated maximum RAM usage: {}mb",
        ((config.threads + config.channel_buffer) * config.thread_buffer * 2 * 8
            + (config
                .images
                .iter()
                .map(|image| image.width as usize * image.height as usize)
                .sum::<usize>()
                / config.file_buffer_size
                + 1)
                * config.pixel_buffer_cutoff_size)
            / 1000000
    );

    let location_generator = location_generators::UniformRandomLocationGenerator::new(
        config.scan_min,
        config.scan_max,
        config.samples,
        config.sample_section,
    );
    let eta = eta::ETA::new(config.samples, config.eta_section, config.eta_time);

    let mut senders = vec![];
    let mut receivers = vec![];
    for _ in &config.images {
        let (sender, receiver) =
            mpsc::sync_channel::<Option<Vec<Complex64>>>(config.channel_buffer);
        senders.push(sender);
        receivers.push(receiver);
    }

    for thread_id in 0..config.threads {
        let mut location_generator = location_generator.clone();
        let mut eta = eta.clone();

        let senders = senders.clone();
        let config = config.clone();

        thread::Builder::new()
            .name(format!("Calculator {}", thread_id))
            .spawn(move || {
                println!("Starting thread {}", thread_id);

                let mut result_caches =
                    vec![Some(Vec::with_capacity(config.thread_buffer)); config.images.len()];

                while let Some(c) = location_generator.next_location() {
                    eta.count();

                    if math::is_inside_mandelbrot_bulb(c) {
                        continue;
                    }

                    if let Some(bailout) = math::calculate_bailout_iteration(
                        &mut config.function.get(c),
                        config.initial_z,
                        config.bailout_min,
                        config.bailout_max,
                        config.check_iterations,
                    ) {
                        for (i, image) in config.images.iter().enumerate() {
                            if image.min_iterations <= bailout && bailout < image.max_iterations {
                                math::calculate_iteration_values(
                                    &mut config.function.get(c),
                                    config.initial_z,
                                    config.bailout_min,
                                    config.bailout_max,
                                    image.min_iterations,
                                    image.max_iterations,
                                    &mut result_caches[i].as_mut().unwrap(),
                                );
                            }
                        }
                    }

                    for (i, _) in config.images.iter().enumerate() {
                        if result_caches[i].as_ref().unwrap().len() > config.thread_buffer {
                            send_with_warning(&senders[i], Some(result_caches[i].take().unwrap()));
                            result_caches[i] = Some(Vec::with_capacity(config.thread_buffer));
                        }
                    }
                }

                for (i, _) in config.images.iter().enumerate() {
                    send_with_warning(&senders[i], Some(result_caches[i].take().unwrap()));
                    send_with_warning(&senders[i], None);
                }

                println!("Thread {} done", thread_id);
            }).expect("Unable to start thread");
    }

    let mut handles = Vec::<thread::JoinHandle<()>>::new();
    for (i, receiver) in receivers.drain(..).enumerate() {
        let config = config.clone();
        let image = config.images[i].clone();

        handles.push(
            thread::Builder::new()
                .name("Aggregator".to_owned())
                .spawn(move || {
                    let mut aggregator = aggregators::FileAggregator::create(
                        image.file_name,
                        image.width,
                        image.height,
                        image.min,
                        image.max,
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
                }).expect("Unable to start thread"),
        );
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
fn send_with_warning(
    sender: &mpsc::SyncSender<Option<Vec<Complex64>>>,
    value: Option<Vec<Complex64>>,
) {
    match sender.try_send(value) {
        Ok(_) => {}
        Err(mpsc::TrySendError::Full(result_cache)) => {
            println!("Bottleneck while sending");
            sender.send(result_cache).unwrap();
        }
        _ => panic!(),
    }
}

fn image(config: Config) {
    // TODO separate image size; downsampling
    println!("Preparing color channels");

    for image in config.images {
        println!("Loading image from {}", image.file_name);
        let mut file = OpenOptions::new()
            .read(true)
            .open(image.file_name)
            .expect("Could not open file");

        use num;
        use std::f64::consts::E;

        image::ImageData::read_fully(
                &mut file,
                image.width as usize,
                image.height as usize,
            ).expect("Could not read file")
            // .map(&|i: u32| ((i as f64).sqrt() * 10000.0) as u32)
            // .map_to_grayscale_linear(1.0)
            .map_to_image1(&|i, highest| num::clamp((1.0 - E.powf(-2.0 * (i as f64 / highest as f64))) * 255.0 * 2.0, 0.0, 255.0) as u8, file_image::Gray(8))
            .save(&(image.file_name.to_owned() + ".png"))
            .unwrap();
    }

    // TODO tiled writing?

    // println!("Joining images");
    // let image = image::Image::join(
    //     &images[2],
    //     &images[1],
    //     &images[0],
    //     file_image::ColorType::RGB(8),
    // );

    // println!("Saving image");
    // image.save(config.image_file_name).unwrap();
}
