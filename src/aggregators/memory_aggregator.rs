use std::fs::{File, OpenOptions};

use num::complex::Complex64;

use aggregators::Aggregator;
use file;
use math;
use vec;

pub struct MemoryAggregator {
    file: File,

    width: u64,
    height: u64,

    min: Complex64,
    max: Complex64,

    file_buffer_size: usize,

    data: Vec<u32>,
}
impl MemoryAggregator {
    pub fn new(
        file: &str,
        width: u64,
        height: u64,
        min: Complex64,
        max: Complex64,
        file_buffer_size: usize,
    ) -> MemoryAggregator {
        MemoryAggregator {
            file: OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(file)
                .unwrap(),
            width,
            height,
            min,
            max,

            file_buffer_size,

            data: vec::filled_with(0u32, width as usize * height as usize),
        }
    }
}
impl Aggregator for MemoryAggregator {
    fn aggregate(&mut self, c: Complex64) {
        let (x, y) = math::complex_to_image(c, self.min, self.max, self.width, self.height);

        if x < self.width && y < self.width {
            let l = y * self.width + x;
            self.data[l as usize] += 1;
        }
    }
}
impl Drop for MemoryAggregator {
    fn drop(&mut self) {
        for (i, chunk) in self.data[..].chunks(self.file_buffer_size).enumerate() {
            file::write_u32(&mut self.file, (i * self.file_buffer_size) as u64, chunk).unwrap();
        }
    }
}
