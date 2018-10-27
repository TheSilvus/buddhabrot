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

    pub fn map_color1(&self, map: &Fn(u32) -> u8) -> Image {
        let mapped = self.data.iter().map(|i| *i).map(map).collect::<Vec<_>>();

        Image {
            data: mapped,
            color_type: file_image::Gray(8),

            width: self.width,
            height: self.height,
        }
    }
    pub fn map_color3(&self, map: &Fn(u32) -> [u8; 3], color_type: file_image::ColorType) -> Image {
        let mapped = self
            .data
            .iter()
            .map(|i| *i)
            .map(map)
            .map(|a| vec![a[0], a[1], a[2]])
            .flatten()
            .collect::<Vec<_>>();

        Image {
            data: mapped,
            color_type: color_type,

            width: self.width,
            height: self.height,
        }
    }
    pub fn map_color4(&self, map: &Fn(u32) -> [u8; 4], color_type: file_image::ColorType) -> Image {
        let mapped = self
            .data
            .iter()
            .map(|i| *i)
            .map(map)
            .map(|a| vec![a[0], a[1], a[2], a[3]])
            .flatten()
            .collect::<Vec<_>>();

        Image {
            data: mapped,
            color_type: color_type,

            width: self.width,
            height: self.height,
        }
    }

    pub fn map_linear_height(&self) -> Image {
        let highest = *self.data.iter().max().unwrap();

        self.map_color1(&|i| ((i as f64 / highest as f64) * 255.0) as u8)
    }

    pub fn map_sqrt_height(&self) -> Image {
        let highest = (*self.data.iter().max().unwrap() as f64).sqrt();

        self.map_color1(&|i| (((i as f64).sqrt() / highest as f64) * 255.0) as u8)
    }

    pub fn map_linear_count(&self, minimum_height: u32) -> Image {
        let heights = self.count_heights();
        let sum: u64 = heights.values().skip(minimum_height as usize).sum();

        let mut current_sum = 0;

        let height_colors = heights
            .iter()
            .map(|(height, count)| (*height, *count))
            .map(|(height, count)| {
                if height < minimum_height {
                    (height, 0)
                } else {
                    current_sum += count;
                    (height, (current_sum / (sum / 255)) as u8)
                }
            }).collect::<BTreeMap<_, _>>();

        self.map_color1(&|i| *height_colors.get(&i).unwrap())
    }
}

pub struct Image {
    data: Vec<u8>,

    color_type: file_image::ColorType,

    width: usize,
    height: usize,
}
impl Image {
    pub fn join(&self, image1: Image, image2: Image, image3: Image, color_type: file_image::ColorType) -> Image {
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
