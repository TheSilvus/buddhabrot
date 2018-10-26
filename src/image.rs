use std::collections::BTreeMap;
use std::fs::File;
use std::io;

use file_image;

use file;
use vec;

pub struct ImageData {
    data: Vec<u32>,

    width: usize,
    height: usize,
}
impl ImageData {
    pub fn read_fully(file: &mut File, width: usize, height: usize) -> io::Result<ImageData> {
        let mut data = vec::filled_with(0, width * height);
        file::read_u32(file, 0, &mut data)?;

        Ok(ImageData {
            data,
            width,
            height,
        })
    }

    pub fn count_heights(&self) -> BTreeMap<u32, u64> {
        let mut values = BTreeMap::new();

        for height in &self.data {
            if values.contains_key(height) {
                *values.get_mut(height).unwrap() += 1;
            } else {
                values.insert(*height, 1);
            }
        }

        values
    }

    pub fn map_and_save(&self, file: &str, map: &Fn(u32) -> u8) -> io::Result<()> {
        let converted = self.data.iter().map(|i| *i).map(map).collect::<Vec<_>>();

        file_image::save_buffer(
            file,
            &converted[..],
            self.width as u32,
            self.height as u32,
            file_image::Gray(8),
        )
    }
}
