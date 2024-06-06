#[cfg(feature = "cli")]
use image::{DynamicImage, GenericImage, GenericImageView, Rgba};

#[cfg(feature = "web")]
use wasm_bindgen::prelude::wasm_bindgen;

pub mod algorithms;

#[cfg_attr(feature = "web", wasm_bindgen)]
#[derive(Clone, Debug)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pixels: Vec<u8>,
}

#[cfg_attr(feature = "web", wasm_bindgen)]
impl Image {
    #[cfg_attr(feature = "web", wasm_bindgen(constructor))]
    pub fn new(width: u32, height: u32, pixels: Vec<u8>) -> Self {
        Self {
            width,
            height,
            pixels,
        }
    }

    pub fn from_rgb(width: u32, height: u32, data: Vec<u8>) -> Self {
        let pixels = data
            .chunks(4)
            .map(|c| {
                ((c[0] as f32) * 0.2126 + (c[1] as f32) * 0.7152 + (c[2] as f32) * 0.0722) as u8
            })
            .collect();

        Self::new(width, height, pixels)
    }

    #[cfg(feature = "web")]
    pub fn to_rgb(&self) -> wasm_bindgen::Clamped<Vec<u8>> {
        wasm_bindgen::Clamped(
            self.pixels
                .clone()
                .into_iter()
                .flat_map(|c| [c, c, c, 255])
                .collect(),
        )
    }

    pub fn get_pixel(&mut self, x: isize, y: isize) -> Option<u8> {
        if x < 0 || x >= self.width as isize || y < 0 || y >= self.height as isize {
            return None;
        }

        let x = x as usize;
        let y = y as usize;

        return Some(self.pixels[(y * self.width as usize + x) as usize]);
    }

    pub fn set_pixel(&mut self, x: isize, y: isize, value: u8) {
        if x < 0 || x >= self.width as isize || y < 0 || y >= self.height as isize {
            return;
        }

        let x = x as usize;
        let y = y as usize;

        self.pixels[(y * self.width as usize + x) as usize] = value;
    }
}

#[cfg(feature = "cli")]
impl From<DynamicImage> for Image {
    fn from(value: DynamicImage) -> Self {
        let gray_scale: DynamicImage = value.grayscale();

        let height = gray_scale.height();
        let width = gray_scale.width();

        let mut img = Image {
            width,
            height,
            pixels: Vec::with_capacity((height * width).try_into().expect("Image does not fit")),
        };

        for y in 0..height {
            for x in 0..width {
                img.pixels.push(gray_scale.get_pixel(x, y).0[0]);
            }
        }

        img
    }
}

#[cfg(feature = "cli")]
impl Into<DynamicImage> for Image {
    fn into(self) -> DynamicImage {
        let mut output = DynamicImage::new(self.width, self.height, image::ColorType::Rgb8);
        for y in 0..self.height {
            for x in 0..self.width {
                let v = self.pixels[(y * self.width + x) as usize].clone();
                assert!(v == 0 || v == 255);
                output.put_pixel(x, y, Rgba([v, v, v, 255]));
            }
        }
        output
    }
}

#[cfg_attr(feature = "web", wasm_bindgen)]
pub fn transform(img: Image, alg: algorithms::Options) -> Image {
    return algorithms::process(img, alg);
}
