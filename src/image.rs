use std::collections::BTreeMap;
use std::fs::File;
use std::io;

use file_image;
use num;

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

    pub fn join(mut image1: ImageData, image2: ImageData) -> ImageData {
        for (i, mut value) in image1.data.iter_mut().enumerate() {
            *value = image2.data[i];
        }

        image1
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

    pub fn highest(&self) -> u32 {
        *self.data.iter().max().unwrap()
    }
    pub fn sum(&self) -> u32 {
        self.data.iter().sum::<u32>()
    }

    pub fn map_to_image1(&self, map: &Fn(u32, u32) -> u8, color_type: file_image::ColorType) -> Image {
        let highest = self.highest();

        let mut mapped = Vec::with_capacity(self.data.len());
        for i in &self.data {
            mapped.push(map(*i, highest));
        }

        Image {
            data: mapped,
            color_type: color_type,

            width: self.width,
            height: self.height,
        }
    }
    pub fn map_to_grayscale_linear(&self, exposure: f64) -> Image {
        self.map_to_image1(&|i, highest| num::clamp((i as f64 / highest as f64) * exposure * 255.0, 0.0, 255.0) as u8, file_image::Gray(8))
    }


    pub fn map_to_image3(&self, map: &Fn(u32, u32) -> [u8; 3], color_type: file_image::ColorType) -> Image {
        let highest = self.highest();

        let mut mapped = Vec::with_capacity(self.data.len());
        for i in &self.data {
            mapped.extend(map(*i, highest).iter());
        }

        Image {
            data: mapped,
            color_type: color_type,

            width: self.width,
            height: self.height,
        }
    }
    // TODO map_to_image3/4 methods for converting single picture to color



    pub fn map(&mut self, map: &Fn(u32) -> u32) -> &mut ImageData {
        for i in &mut self.data {
            *i = map(*i);
        }

        self
    }


    // pub fn map_linear_count(&self, minimum_height: u32) -> Image {
    //     let heights = self.count_heights();
    //     let sum: u64 = heights.values().skip(minimum_height as usize).sum();

    //     let mut current_sum = 0;

    //     let height_colors = heights
    //         .iter()
    //         .map(|(height, count)| (*height, *count))
    //         .map(|(height, count)| {
    //             if height < minimum_height {
    //                 (height, 0)
    //             } else {
    //                 current_sum += count;
    //                 (height, (current_sum / (sum / 255)) as u8)
    //             }
    //         }).collect::<BTreeMap<_, _>>();

    //     self.map_color1(&|i| *height_colors.get(&i).unwrap())
    // }
}

pub struct Image {
    data: Vec<u8>,

    color_type: file_image::ColorType,

    width: usize,
    height: usize,
}
impl Image {
    pub fn join(
        image1: &Image,
        image2: &Image,
        image3: &Image,
        color_type: file_image::ColorType,
    ) -> Image {
        let mut data = Vec::with_capacity(image1.data.len() * 3);

        for i in 0..image1.data.len() {
            data.push(image1.data[i]);
            data.push(image2.data[i]);
            data.push(image3.data[i]);
        }

        Image {
            data,
            color_type,
            width: image1.width,
            height: image1.height,
        }
    }

    pub fn save(&self, file: &str) -> io::Result<()> {
        file_image::save_buffer(
            file,
            &self.data[..],
            self.width as u32,
            self.height as u32,
            self.color_type,
        )
    }
}
