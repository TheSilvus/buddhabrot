use std::fs::File;
use std::io;
use std::io::{Read, Seek, SeekFrom, Write};
use std::mem;
use std::slice;

pub fn read_u32(file: &mut File, location: u64, buffer: &mut Vec<u32>) -> io::Result<usize> {
    file.seek(SeekFrom::Start(location * mem::size_of::<u32>() as u64))?;

    let buffer = &buffer[..];
    let buffer = unsafe {
        slice::from_raw_parts_mut(
            buffer.as_ptr() as *mut u8,
            buffer.len() * mem::size_of::<u32>(),
        )
    };
    file.read(buffer)
}

pub fn write_u32(file: &mut File, location: u64, buffer: &mut Vec<u32>) -> io::Result<usize> {
    file.seek(SeekFrom::Start(location * mem::size_of::<u32>() as u64))?;

    let buffer = &buffer[..];
    let buffer = unsafe {
        slice::from_raw_parts(
            buffer.as_ptr() as *const u8,
            buffer.len() * mem::size_of::<u32>(),
        )
    };
    file.write(buffer)
}
