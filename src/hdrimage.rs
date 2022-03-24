use std::vec::Vec;
use std::io::{BufRead, BufReader};
use std::fs::File;
use std::str::FromStr;
use byteorder::{ReadBytesExt};
use std::path::Path;

use crate::color::{Color, IsClose};
use crate::error::HdrImageErr;

#[derive(Clone, PartialEq)]
pub struct HdrImage {
    width: usize,
    height: usize,
    pixels: Vec<Color>,
}

impl HdrImage {
    pub fn new(width: usize, height: usize) -> HdrImage {
        HdrImage {
            width,
            height,
            pixels: vec![Default::default(); width * height],
        }
    }

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

    fn read_pfm_image<R: BufRead>(buf_reader: &mut R) -> Result<HdrImage, HdrImageErr> {
        let mut line = String::new();
        buf_reader.read_line(&mut line).map_err(|e| HdrImageErr::PfmFileReadFailure(e));
        if line != "PF" {
            return Err(HdrImageErr::InvalidPfmFileFormat(String::from("i can only read pfm files")));
        }
        line.clear();
        buf_reader.read_line(&mut line).map_err(|e| HdrImageErr::PfmFileReadFailure(e));
        let (width, height) = parse_img_shape(&line)?;
        line.clear();
        buf_reader.read_line(&mut line).map_err(|e| HdrImageErr::PfmFileReadFailure(e));
        let endianness: ByteOrder = parse_endianness(&line)?;
        line.clear();
        let mut buffer = [0_f32; 3];
        let mut image = HdrImage::new(width, height);
        for y in (0..height).rev() {
            for x in 0..width {
                match endianness {
                    ByteOrder::LittleEndian => buf_reader.read_f32_into::<byteorder::LittleEndian>(&mut buffer).map_err(|e| HdrImageErr::PfmFileReadFailure(e))?,
                    ByteOrder::BigEndian => buf_reader.read_f32_into::<byteorder::BigEndian>(&mut buffer).map_err(|e| HdrImageErr::PfmFileReadFailure(e))?,
                };
                image.set_pixel(x, y, (buffer[0], buffer[1], buffer[2]).into())?
            }
        }
        if buf_reader.read_line(&mut line).map_err(|e| HdrImageErr::PfmFileReadFailure(e))? == 0 {
            Ok(image)
        } else {
            Err(HdrImageErr::InvalidPfmFileFormat(String::from("wrong size specification")))
        }
    }

    pub fn read_pfm_file<T: AsRef<Path>>(path: T) -> Result<HdrImage, HdrImageErr> {
        let file = File::open(path).map_err(|e| HdrImageErr::PfmFileReadFailure(e))?;
        let mut buf_reader = BufReader::new(file);
        return HdrImage::read_pfm_image(&mut buf_reader);
    }
}

fn parse_img_shape(line: &String) -> Result<(usize, usize), HdrImageErr> {
    let shape: Vec<&str> = line.split(' ').filter(|s| s != &"").collect();
    if shape.len() == 2 {
        let width = usize::from_str(shape[0]).map_err(|e| HdrImageErr::ImgShapeParseFailure(e))?;
        let height = usize::from_str(shape[1]).map_err(|e| HdrImageErr::ImgShapeParseFailure(e))?;
        Ok((width, height))
    } else {
        Err(HdrImageErr::InvalidPfmFileFormat(String::from("invalid image size specification")))
    }
}

fn parse_endianness(line: &String) -> Result<ByteOrder, HdrImageErr> {
    let value = f32::from_str(line.as_str()).map_err(|e| HdrImageErr::EndiannessParseFailure(e))?;
    if value > 0.0 {
        Ok(ByteOrder::BigEndian)
    } else if value < 0.0 {
        Ok(ByteOrder::LittleEndian)
    } else {
        Err(HdrImageErr::InvalidPfmFileFormat(String::from("invalid endianness specification")))
    }
}

pub enum ByteOrder {
    LittleEndian,
    BigEndian,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn image_creation() {
        let hdr_img = HdrImage::new(5, 7);
        assert_eq!(hdr_img.width, 5);
        assert_eq!(hdr_img.height, 7)
    }

    #[test]
    fn validate_coordinates_0() {
        assert!(HdrImage::new(3, 3).valid_coordinates(0, 0))
    }

    #[test]
    fn validate_coordinates_1() {
        assert!(HdrImage::new(3, 3).valid_coordinates(1, 2))
    }

    #[test]
    fn validate_coordinates_2() {
        assert!(HdrImage::new(3, 3).valid_coordinates(2, 1))
    }

    #[test]
    fn neg_validate_coordinates_0() {
        assert!(!HdrImage::new(3, 3).valid_coordinates(3, 3))
    }

    #[test]
    fn neg_validate_coordinates_1() {
        assert!(!HdrImage::new(3, 3).valid_coordinates(3, 1))
    }

    #[test]
    fn neg_validate_coordinates_2() {
        assert!(!HdrImage::new(3, 3).valid_coordinates(1, 3))
    }

    #[test]
    fn get_pixel_ok() {
        assert_eq!(
            HdrImage::new(3, 3).get_pixel(0, 0).unwrap(),
            Color {
                ..Default::default()
            }
        )
    }

    #[test]
    fn get_pixel_err() {
        assert_eq!(
            HdrImage::new(3, 3).get_pixel(3, 3).unwrap_err(),
            HdrImageErr::OutOfBounds((3, 3), (3, 3))
        )
    }

    #[test]
    fn set_pixel_ok() {
        let color = Color::from((1.0, 1.0, 1.0));
        let mut hdr_image = HdrImage::new(3, 3);
        assert_eq!(hdr_image.set_pixel(0, 0, color), Ok(()));
        assert_eq!(hdr_image.get_pixel(0, 0).unwrap(), color)
    }

    #[test]
    fn set_pixel_err() {
        let color = Color::from((1.0, 1.0, 1.0));
        let mut hdr_image = HdrImage::new(3, 3);
        assert_eq!(
            hdr_image.set_pixel(3, 3, color).unwrap_err(),
            HdrImageErr::OutOfBounds((3, 3), (3, 3))
        );
        assert_ne!(hdr_image.get_pixel(0, 0).unwrap(), color)
    }

    #[test]
    fn is_close_get_set_pixel() -> Result<(), HdrImageErr> {
        let color = Color::from((1.2, 2.3, 3.4));
        let mut hdr_img = HdrImage::new(5, 7);
        hdr_img.set_pixel(1, 3, color)?;
        hdr_img.get_pixel(1, 3)?.is_close(color);
        Ok(())
    }

    #[test]
    fn parse_img_shape_ok() {
        let mut line = String::from("10 20");
        assert!(matches!(parse_img_shape(&line), Ok((10, 20))));
        line = String::from(" 10    20  ");
        assert!(matches!(parse_img_shape(&line), Ok((10, 20))))
    }

    #[test]
    fn parse_image_shape_invalid() {
        let mut line = String::from("10 20 30");
        assert!(matches!(parse_img_shape(&line), Err(HdrImageErr::InvalidPfmFileFormat(_))));
        line = String::from("10 ");
        assert!(matches!(parse_img_shape(&line), Err(HdrImageErr::InvalidPfmFileFormat(_))))
    }

    #[test]
    fn parse_image_shape_failure() {
        let mut line = String::from("10 20.1");
        assert!(matches!(parse_img_shape(&line), Err(HdrImageErr::ImgShapeParseFailure(..))));
        line = String::from("-10 20");
        assert!(matches!(parse_img_shape(&line), Err(HdrImageErr::ImgShapeParseFailure(..))))
    }

    #[test]
    fn parse_endianness_ok() {
        let mut line = String::from("-3.2");
        assert!(matches!(parse_endianness(&line), Ok(ByteOrder::LittleEndian)));
        line = String::from("1e15");
        assert!(matches!(parse_endianness(&line), Ok(ByteOrder::BigEndian)));
        line = String::from("0");
        assert!(matches!(parse_endianness(&line), Err(HdrImageErr::InvalidPfmFileFormat(_))));
    }
}