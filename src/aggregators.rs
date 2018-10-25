use std::fs::{File, OpenOptions};
use std::io;
use std::io::{Read, Seek, SeekFrom, Write};
use std::mem;
use std::slice;

use num::complex::Complex64;

use math;
use vec;
use file;

pub struct FileAggregator {
    file: File,

    file_width: u64,
    file_height: u64,
    file_min: Complex64,
    file_max: Complex64,

    file_buffer_size: usize,
    pixel_buffer_cutoff_size: usize,

    file_buffer: Vec<u32>,
    pixel_buffers: Vec<Vec<(u64, u64)>>,
}
impl FileAggregator {
    pub fn new(
        file: File,
        file_width: u64,
        file_height: u64,
        file_min: Complex64,
        file_max: Complex64,
        file_buffer_size: usize,
        pixel_buffer_cutoff_size: usize,
    ) -> io::Result<FileAggregator> {
        let mut aggregator = FileAggregator {
            file,
            file_width,
            file_height,
            file_min,
            file_max,
            file_buffer_size,
            pixel_buffer_cutoff_size,

            file_buffer: vec::filled_with(0, file_buffer_size),

            pixel_buffers: vec::filled_with(Vec::new(),
                (file_width * file_height / file_buffer_size as u64) as usize + 1,
            ),
        };

        aggregator.setup_file()?;

        Ok(aggregator)
    }

    pub fn create(
        file_name: &str,
        file_width: u64,
        file_height: u64,
        file_min: Complex64,
        file_max: Complex64,
        file_buffer_size: usize,
        pixel_buffer_cutoff_size: usize,
    ) -> io::Result<FileAggregator> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(true)
            .create(true)
            .open(file_name)
            .unwrap();

        FileAggregator::new(
            file,
            file_width,
            file_height,
            file_min,
            file_max,
            file_buffer_size,
            pixel_buffer_cutoff_size,
        )
    }


    fn setup_file(&mut self) -> io::Result<()> {
        let buffer = vec![0u8]
            .iter()
            .map(|i| *i)
            .cycle()
            .take(self.file_buffer_size * mem::size_of::<u32>())
            .collect::<Vec<u8>>();

        for _ in 0..(self.file_width * self.file_height) / self.file_buffer_size as u64 + 1 {
            self.file.write(&buffer)?;
        }

        Ok(())
    }

    fn write_pixel_buffer(&mut self, buffer: usize) -> io::Result<()> {
        file::read_u32(&mut self.file, buffer as u64 * self.file_buffer_size as u64, &mut self.file_buffer)?;

        for (x, y) in self.pixel_buffers[buffer].iter() {
            let location = y * self.file_width + x;
            let location = (location as usize) % self.file_buffer_size;

            self.file_buffer[location as usize] += 1;
        }

        file::write_u32(&mut self.file, buffer as u64 * self.file_buffer_size as u64, &mut self.file_buffer)?;

        Ok(())
    }

    pub fn aggregate(&mut self, c: Complex64) {
        if math::complex_between(self.file_min, c, self.file_max) {
            let (x, y) = math::complex_to_image(
                c,
                self.file_min,
                self.file_max,
                self.file_width,
                self.file_height,
            );
            if x < self.file_width && y < self.file_height {
                let location = (y * self.file_width + x) as usize;
                let buffer = location / self.file_buffer_size;
                self.pixel_buffers[buffer].push((x, y));

                if self.pixel_buffers[buffer].len() > self.pixel_buffer_cutoff_size {
                    self.write_pixel_buffer(buffer)
                        .expect("Error while writing pixel buffer");
                    self.pixel_buffers[buffer].clear();
                }
            }
        }
    }
}
impl Drop for FileAggregator {
    fn drop(&mut self) {
        for i in 0..self.pixel_buffers.len() {
            if self.pixel_buffers[i].len() > 0 {
                // TODO fight borrow checker
                self.write_pixel_buffer(i).expect("Error while writing pixel buffer");
            }
        }
    }
}
