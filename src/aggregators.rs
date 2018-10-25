use std::fs::{File, OpenOptions};
use std::io;
use std::io::{Read, Seek, SeekFrom, Write};
use std::mem;
use std::slice;

use num::complex::Complex64;

use math;

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

            file_buffer: Vec::with_capacity(file_buffer_size),

            pixel_buffers: Vec::with_capacity(
                (file_width * file_height / file_buffer_size as u64) as usize + 1,
            ),
        };

        aggregator.fill_buffers();
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

    fn fill_buffers(&mut self) {
        for _ in 0..self.file_buffer.capacity() {
            self.file_buffer.push(0);
        }

        for _ in 0..self.pixel_buffers.capacity() {
            self.pixel_buffers.push(Vec::new());
        }
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

    fn file_read(&mut self, location: u64) -> io::Result<usize> {
        self.file
            .seek(SeekFrom::Start(location * mem::size_of::<u32>() as u64))?;

        let buffer = &self.file_buffer[..];
        let buffer = unsafe {
            slice::from_raw_parts_mut(
                buffer.as_ptr() as *mut u8,
                buffer.len() * mem::size_of::<u32>(),
            )
        };
        self.file.read(buffer)
    }

    fn file_write(&mut self, location: u64) -> io::Result<usize> {
        self.file
            .seek(SeekFrom::Start(location * mem::size_of::<u32>() as u64))?;

        let buffer = &self.file_buffer[..];
        let buffer = unsafe {
            slice::from_raw_parts(
                buffer.as_ptr() as *const u8,
                buffer.len() * mem::size_of::<u32>(),
            )
        };
        self.file.write(buffer)
    }

    fn write_pixel_buffer(&mut self, buffer: usize) -> io::Result<()> {
        let file_buffer_size = self.file_buffer_size;
        self.file_read(buffer as u64 * file_buffer_size as u64)?;

        for (x, y) in self.pixel_buffers[buffer].iter() {
            let location = y * self.file_width + x;
            let location = (location as usize) % file_buffer_size;

            self.file_buffer[location as usize] += 1;
        }

        self.file_write(buffer as u64 * file_buffer_size as u64)?;

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
