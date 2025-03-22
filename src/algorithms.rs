use crate::Image;

#[cfg(feature = "web")]
use wasm_bindgen::prelude::wasm_bindgen;

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
#[cfg_attr(feature = "web", wasm_bindgen)]
#[cfg_attr(feature = "strum", derive(strum_macros::EnumIter, strum_macros::Display, strum_macros::EnumString))]
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

pub(crate) fn process(mut img: Image, ed: impl Into<ErrorDiffusion>) -> super::Image {
    let ed = ed.into();
    for iy in 0..img.height {
        for ix in 0..img.width {
            // Pixel in "center"
            let target = img
                .get_pixel(ix as _, iy as _)
                .expect("target is always there");

            let clamped = if target < 0x80 { 0 } else { 0xff };
            img.set_pixel(ix as _, iy as _, clamped);

            let error = target as f32 - clamped as f32;

            for edy in 0..ed.height {
                for edx in 0..ed.width {
                    // Diffusion scalar
                    let error = ed.get(edx as _, edy as _) * error as f32;

                    if error == 0. {
                        continue;
                    }

                    let px = (ix + edx).saturating_sub(ed.center.0) as isize;
                    let py = (iy + edy).saturating_sub(ed.center.1) as isize;

                    if let Some(p) = img.get_pixel(px, py) {
                        img.set_pixel(px, py, (p as f32 + error) as u8);
                    }
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
}
