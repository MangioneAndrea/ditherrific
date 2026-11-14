use crate::Image;

#[cfg(feature = "web")]
use wasm_bindgen::prelude::wasm_bindgen;

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
#[cfg_attr(feature = "web", wasm_bindgen)]
#[cfg_attr(
    feature = "strum",
    derive(
        strum_macros::EnumIter,
        strum_macros::Display,
        strum_macros::EnumString
    )
)]
pub enum Options {
    FloydSteinberg,
    Aktinson,
    Burkes,
    Sierra,
    SierraTwoRow,
    SierraLite,
    None,
}

impl Into<ErrorDiffusion> for Options {
    fn into(self) -> ErrorDiffusion {
        match self {
            Self::FloydSteinberg => ErrorDiffusion::new([[0., 0., 7.], [3., 5., 1.]], (1, 0)),
            Self::Aktinson => ErrorDiffusion::new(
                [[0., 0., 1., 1.], [1., 1., 1., 0.], [0., 1., 0., 0.]],
                (1, 0),
            ),
            Self::Burkes => {
                ErrorDiffusion::new([[0., 0., 0., 8., 4.], [2., 4., 8., 4., 2.]], (2, 0))
            }

            Self::Sierra => ErrorDiffusion::new(
                [
                    [0., 0., 0., 5., 3.],
                    [2., 4., 5., 4., 2.],
                    [0., 2., 3., 2., 0.],
                ],
                (2, 0),
            ),
            Self::SierraTwoRow => {
                ErrorDiffusion::new([[0., 0., 0., 4., 3.], [1., 2., 3., 2., 1.]], (2, 0))
            }
            Self::SierraLite => ErrorDiffusion::new([[0., 0., 2.], [1., 1., 0.]], (1, 0)),
            Self::None => ErrorDiffusion::new::<0, 0>([], (0, 1)),
        }
    }
}

pub(crate) fn process(mut img: Image, ed: impl Into<ErrorDiffusion>) -> Image {
    let ed = ed.into();

    let mut offsets = Vec::new();

    for y in 0..ed.height as usize {
        for x in 0..ed.width as usize {
            let weight = ed.get(x, y);

            if weight != 0.0 {
                offsets.push((
                    x as isize - ed.center.0 as isize,
                    y as isize - ed.center.1 as isize,
                    weight,
                ));
            }
        }
    }

    let width = img.width as usize;
    let height = img.height as usize;

    for y in 0..height {
        for x in 0..width {
            let index = y * width + x;
            let target = img.pixels[index];
            let clamped = if target < 0x80 { 0 } else { 0xff };

            img.pixels[index] = clamped;

            let error = target as f32 - clamped as f32;

            for &(dx, dy, weight) in &offsets {
                let px = x as isize + dx;
                let py = y as isize + dy;

                if px >= 0 && py >= 0 && px < width as isize && py < height as isize {
                    let index = py as usize * width + px as usize;
                    img.pixels[index] = (img.pixels[index] as f32 + weight * error) as u8;
                }
            }
        }
    }

    img
}

pub struct ErrorDiffusion {
    width: u32,
    height: u32,
    kernel: Vec<f32>,
    center: (u32, u32),
    sum: f32,
}

impl ErrorDiffusion {
    pub fn get(&self, x: usize, y: usize) -> f32 {
        return self.kernel[y * self.width as usize + x] / self.sum;
    }
    pub fn new<const W: usize, const H: usize>(
        pixels: [[f32; W]; H],
        center: (u32, u32),
    ) -> ErrorDiffusion {
        let mut v = Vec::with_capacity(W * H);

        let mut sum = 0.;

        for y in 0..H {
            for x in 0..W {
                v.push(pixels[y][x]);
                sum += pixels[y][x];
            }
        }

        ErrorDiffusion {
            width: W as u32,
            height: H as u32,
            kernel: v,
            center,
            sum,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Image;

    use super::ErrorDiffusion;

    #[test]
    fn all_black() {
        let img = Image {
            width: 3,
            height: 3,
            pixels: vec![0, 0, 0, 0, 0, 0, 0, 0, 0],
        };
        let ed = ErrorDiffusion::new([[1., 2., 3.], [4., 5., 7.], [7., 8., 9.]], (1, 1));

        let img = super::process(img, ed);

        assert_eq!(img.pixels, vec![0, 0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn all_white() {
        let img = Image {
            width: 3,
            height: 3,
            pixels: vec![255, 255, 255, 255, 255, 255, 255, 255, 255],
        };
        let ed = ErrorDiffusion::new([[1., 2., 3.], [4., 5., 7.], [7., 8., 9.]], (1, 1));

        let img = super::process(img, ed);

        assert_eq!(
            img.pixels,
            vec![255, 255, 255, 255, 255, 255, 255, 255, 255]
        );
    }

    #[test]
    fn shift_left() {
        let img = Image {
            width: 3,
            height: 1,
            pixels: vec![60, 60, 60],
        };
        let ed = ErrorDiffusion::new([[0., 1.]], (0, 0));

        let img = super::process(img, ed);

        assert_eq!(img.pixels, vec![0, 0, 255]);
    }

    #[test]
    fn shift_left2() {
        let img = Image {
            width: 3,
            height: 1,
            pixels: vec![60, 60, 60],
        };
        let ed = ErrorDiffusion::new([[0., 1.]], (1, 0));

        let img = super::process(img, ed);

        assert_eq!(img.pixels, vec![60, 60, 60]);
    }

    #[test]
    #[ignore = "performance benchmark"]
    fn benchmark_error_diffusion() {
        use std::hint::black_box;
        use std::time::Instant;

        let sizes = [(128, 128), (512, 512), (1024, 1024), (1920, 1080)];

        let algorithms = [
            super::Options::FloydSteinberg,
            super::Options::Aktinson,
            super::Options::Burkes,
            super::Options::Sierra,
            super::Options::SierraTwoRow,
            super::Options::SierraLite,
            super::Options::None,
        ];

        println!("\n{:<20} {:<15} {:>12}", "Algorithm", "Image", "Time (ms)");
        println!("{}", "-".repeat(50));

        for (width, height) in sizes {
            let img = Image {
                width,
                height,
                pixels: (0..height)
                    .flat_map(|y| {
                        (0..width).map(move |x| ((x * 31 + y * 17 + (x ^ y)) % 256) as u8)
                    })
                    .collect(),
            };

            for algorithm in algorithms {
                let iterations = 5;

                // Warmup
                black_box(super::process(img.clone(), algorithm));

                let start = Instant::now();

                for _ in 0..iterations {
                    black_box(super::process(black_box(img.clone()), black_box(algorithm)));
                }

                let elapsed = start.elapsed();
                let per_iteration = elapsed.as_secs_f64() * 1000.0 / iterations as f64;

                println!(
                    "{:<20} {:<15} {:>10.3}",
                    format!("{algorithm:?}"),
                    format!("{width}x{height}"),
                    per_iteration,
                );
            }
        }
    }
}
