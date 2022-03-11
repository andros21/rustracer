use std::vec::Vec;

use crate::color::Color;
use crate::error::HdrImageErr;

#[derive(Clone, PartialEq)]
pub struct HdrImage {
    width: usize,
    height: usize,
    pixels: Vec<Color>,
}

impl From<(usize, usize)> for HdrImage {
    fn from(shape: (usize, usize)) -> Self {
        Self {
            width: shape.0,
            height: shape.1,
            pixels: vec![Color::from(()); shape.0 * shape.1],
        }
    }
}

impl HdrImage {
    fn pixel_offset(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    fn valid_coordinates(&self, x: usize, y: usize) -> bool {
        return (x < self.width) & (y < self.height);
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Result<Color, HdrImageErr> {
        if self.valid_coordinates(x, y) {
            Ok(self.pixels[self.pixel_offset(x, y)])
        } else {
            Err(HdrImageErr::OutOfBounds((x, y), (self.width, self.height)))
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, new_color: Color) -> Result<(), HdrImageErr> {
        if self.valid_coordinates(x, y) {
            let pixel_offset = self.pixel_offset(x, y);
            Ok(self.pixels[pixel_offset] = new_color)
        } else {
            Err(HdrImageErr::OutOfBounds((x, y), (self.width, self.height)))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn validate_coordinates_0() {
        assert!(HdrImage::from((3, 3)).valid_coordinates(0, 0))
    }

    #[test]
    fn validate_coordinates_1() {
        assert!(HdrImage::from((3, 3)).valid_coordinates(1, 2))
    }

    #[test]
    fn validate_coordinates_2() {
        assert!(HdrImage::from((3, 3)).valid_coordinates(2, 1))
    }

    #[test]
    fn neg_validate_coordinates_0() {
        assert!(!HdrImage::from((3, 3)).valid_coordinates(3, 3))
    }

    #[test]
    fn neg_validate_coordinates_1() {
        assert!(!HdrImage::from((3, 3)).valid_coordinates(3, 1))
    }

    #[test]
    fn neg_validate_coordinates_2() {
        assert!(!HdrImage::from((3, 3)).valid_coordinates(1, 3))
    }

    #[test]
    fn get_pixel_ok() {
        assert_eq!(
            HdrImage::from((3, 3)).get_pixel(0, 0).unwrap(),
            Color::from(())
        )
    }

    #[test]
    fn get_pixel_err() {
        assert_eq!(
            HdrImage::from((3, 3)).get_pixel(3, 3).unwrap_err(),
            HdrImageErr::OutOfBounds((3, 3), (3, 3))
        )
    }

    #[test]
    fn set_pixel_ok() {
        let color = Color::from((1.0, 1.0, 1.0));
        let mut hdrimage = HdrImage::from((3, 3));
        assert_eq!(hdrimage.set_pixel(0, 0, color), Ok(()));
        assert_eq!(hdrimage.get_pixel(0, 0).unwrap(), color)
    }

    #[test]
    fn set_pixel_err() {
        let color = Color::from((1.0, 1.0, 1.0));
        let mut hdrimage = HdrImage::from((3, 3));
        assert_eq!(
            hdrimage.set_pixel(3, 3, color).unwrap_err(),
            HdrImageErr::OutOfBounds((3, 3), (3, 3))
        );
        assert_ne!(hdrimage.get_pixel(0, 0).unwrap(), color)
    }
}
