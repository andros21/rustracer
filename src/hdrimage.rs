use byteorder::WriteBytesExt;
use std::fs::File;
use std::io::{BufWriter, Cursor, Seek, Write};
use std::path::Path;
use std::vec::Vec;

use crate::color::{Color, IsClose};
use crate::error::HdrImageErr;

const DELTA: f32 = 1e-10;

#[derive(Clone, PartialEq)]
pub struct HdrImage {
    width: u32,
    height: u32,
    pixels: Vec<Color>,
}

impl HdrImage {
    pub fn new(width: u32, height: u32) -> HdrImage {
        HdrImage {
            width,
            height,
            pixels: vec![Default::default(); (width * height) as usize],
        }
    }

    fn pixel_offset(&self, x: u32, y: u32) -> usize {
        (y * self.width + x) as usize
    }

    fn valid_coordinates(&self, x: u32, y: u32) -> bool {
        return (x < self.width) & (y < self.height);
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> Result<Color, HdrImageErr> {
        if self.valid_coordinates(x, y) {
            Ok(self.pixels[self.pixel_offset(x, y)])
        } else {
            Err(HdrImageErr::OutOfBounds((x, y), (self.width, self.height)))
        }
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, new_color: Color) -> Result<(), HdrImageErr> {
        if self.valid_coordinates(x, y) {
            let pixel_offset = self.pixel_offset(x, y);
            Ok(self.pixels[pixel_offset] = new_color)
        } else {
            Err(HdrImageErr::OutOfBounds((x, y), (self.width, self.height)))
        }
    }

    fn write_pfm_image<T: Write + Seek>(
        &self,
        stream: &mut T,
        endianness: ByteOrder,
    ) -> std::io::Result<()> {
        let mut header = format!("PF\n{} {}\n", self.width, self.height);
        match endianness {
            ByteOrder::BigEndian => header.push_str("1.0\n"),
            ByteOrder::LittleEndian => header.push_str("-1.0\n"),
        }
        stream.write(header.as_bytes())?;
        for y in (0..self.height).rev() {
            for x in 0..self.width {
                for el in self.get_pixel(x, y).unwrap().into_iter() {
                    write_float(stream, el, &endianness)?;
                }
            }
        }
        Ok(())
    }

    pub fn write_pfm_file<T: AsRef<Path>>(
        &self,
        path: T,
        endianness: ByteOrder,
    ) -> std::io::Result<()> {
        let out = File::create(path)?;
        let mut writer = BufWriter::new(out);
        self.write_pfm_image(&mut writer, endianness)
    }

    fn average_luminosity(&self) -> f32 {
        let mut cumsum = 0.0;
        for pixel in self.pixels.iter() {
            cumsum += f32::log10(DELTA + pixel.luminosity());
        }
        f32::powf(10.0, cumsum / (self.pixels.len() as f32))
    }

    fn normalize_image(&mut self, factor: f32, luminosity: Luminosity) {
        let lum = match luminosity {
            Luminosity::AverageLuminosity => self.average_luminosity(),
            Luminosity::FloatValue(val) => val,
        };
        for i in 0..self.pixels.len() {
            self.pixels[i] = self.pixels[i] * (factor / lum);
        }
    }

    fn clamp_image(&mut self) {
        for pixel in self.pixels.iter_mut() {
            pixel.r = clamp(pixel.r);
            pixel.g = clamp(pixel.g);
            pixel.b = clamp(pixel.b);
        }
    }
}

pub enum ByteOrder {
    BigEndian,
    LittleEndian,
}

fn write_float<T: Write>(
    stream: &mut T,
    value: f32,
    endianness: &ByteOrder,
) -> std::io::Result<()> {
    match endianness {
        ByteOrder::BigEndian => stream.write_f32::<byteorder::BigEndian>(value),
        ByteOrder::LittleEndian => stream.write_f32::<byteorder::LittleEndian>(value),
    }
}

pub enum Luminosity {
    AverageLuminosity,
    FloatValue(f32),
}

fn clamp(x: f32) -> f32 {
    x / (1.0 + x)
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
    fn validate_coordinates() {
        let hdr_img = HdrImage::new(5, 7);

        assert!(hdr_img.valid_coordinates(0, 0));
        assert!(hdr_img.valid_coordinates(4, 3));
        assert!(!hdr_img.valid_coordinates(7, 0));
        assert!(!hdr_img.valid_coordinates(0, 9));
        assert!(!hdr_img.valid_coordinates(7, 9))
    }

    #[test]
    fn pixel_offset() {
        let hdr_img = HdrImage::new(7, 4);

        assert_eq!(hdr_img.pixel_offset(0, 0), 0);
        assert_eq!(hdr_img.pixel_offset(3, 2), 17);
        assert_eq!(hdr_img.pixel_offset(6, 3), 7 * 4 - 1)
    }

    #[test]
    fn get_pixel_ok() {
        let color: Color = Default::default();

        assert!(matches!(HdrImage::new(3, 3).get_pixel(0, 0), Ok(color)))
    }

    #[test]
    fn get_pixel_err() {
        assert!(matches!(HdrImage::new(3, 3).get_pixel(3, 3),
            Err(HdrImageErr::OutOfBounds(a, b)) if a == (3, 3) && b == (3, 3)))
    }

    #[test]
    fn set_pixel_ok() {
        let color1 = Color::from((1.0, 1.0, 1.0));
        let color2 = Color::from((1.23, 4.56, 7.89));
        let mut hdr_image = HdrImage::new(3, 3);
        assert!(matches!(hdr_image.set_pixel(0, 0, color1), Ok(())));
        assert!(matches!(hdr_image.set_pixel(2, 2, color2), Ok(())));
        assert!(matches!(hdr_image.get_pixel(0, 0), Ok(color1)));
        assert!(hdr_image.get_pixel(2, 2).unwrap().is_close(color2))
    }

    #[test]
    fn set_pixel_err() {
        let color = Color::from((1.0, 1.0, 1.0));
        let mut hdr_image = HdrImage::new(3, 3);

        assert!(matches!(hdr_image.set_pixel(5, 5, color),
            Err(HdrImageErr::OutOfBounds(a, b)) if a == (5, 5) && b == (3, 3)))
    }

    #[test]
    fn write_pfm_image() {
        let mut hdr_image = HdrImage::new(3, 2);

        hdr_image
            .set_pixel(0, 0, Color::from((1.0e1, 2.0e1, 3.0e1)))
            .unwrap();
        hdr_image
            .set_pixel(1, 0, Color::from((4.0e1, 5.0e1, 6.0e1)))
            .unwrap();
        hdr_image
            .set_pixel(2, 0, Color::from((7.0e1, 8.0e1, 9.0e1)))
            .unwrap();
        hdr_image
            .set_pixel(0, 1, Color::from((1.0e2, 2.0e2, 3.0e2)))
            .unwrap();
        hdr_image
            .set_pixel(1, 1, Color::from((4.0e2, 5.0e2, 6.0e2)))
            .unwrap();
        hdr_image
            .set_pixel(2, 1, Color::from((7.0e2, 8.0e2, 9.0e2)))
            .unwrap();

        let _reference_bytes_be = vec![
            0x50, 0x46, 0x0a, 0x33, 0x20, 0x32, 0x0a, 0x31, 0x2e, 0x30, 0x0a, 0x42, 0xc8, 0x00,
            0x00, 0x43, 0x48, 0x00, 0x00, 0x43, 0x96, 0x00, 0x00, 0x43, 0xc8, 0x00, 0x00, 0x43,
            0xfa, 0x00, 0x00, 0x44, 0x16, 0x00, 0x00, 0x44, 0x2f, 0x00, 0x00, 0x44, 0x48, 0x00,
            0x00, 0x44, 0x61, 0x00, 0x00, 0x41, 0x20, 0x00, 0x00, 0x41, 0xa0, 0x00, 0x00, 0x41,
            0xf0, 0x00, 0x00, 0x42, 0x20, 0x00, 0x00, 0x42, 0x48, 0x00, 0x00, 0x42, 0x70, 0x00,
            0x00, 0x42, 0x8c, 0x00, 0x00, 0x42, 0xa0, 0x00, 0x00, 0x42, 0xb4, 0x00, 0x00,
        ];

        let _reference_bytes_le = vec![
            0x50, 0x46, 0x0a, 0x33, 0x20, 0x32, 0x0a, 0x2d, 0x31, 0x2e, 0x30, 0x0a, 0x00, 0x00,
            0xc8, 0x42, 0x00, 0x00, 0x48, 0x43, 0x00, 0x00, 0x96, 0x43, 0x00, 0x00, 0xc8, 0x43,
            0x00, 0x00, 0xfa, 0x43, 0x00, 0x00, 0x16, 0x44, 0x00, 0x00, 0x2f, 0x44, 0x00, 0x00,
            0x48, 0x44, 0x00, 0x00, 0x61, 0x44, 0x00, 0x00, 0x20, 0x41, 0x00, 0x00, 0xa0, 0x41,
            0x00, 0x00, 0xf0, 0x41, 0x00, 0x00, 0x20, 0x42, 0x00, 0x00, 0x48, 0x42, 0x00, 0x00,
            0x70, 0x42, 0x00, 0x00, 0x8c, 0x42, 0x00, 0x00, 0xa0, 0x42, 0x00, 0x00, 0xb4, 0x42,
        ];

        let mut stream = Cursor::new(Vec::new());
        assert!(matches!(
            hdr_image.write_pfm_image(&mut stream, ByteOrder::BigEndian),
            Ok(())
        ));
        assert_eq!(_reference_bytes_be, stream.into_inner());

        let mut stream = Cursor::new(Vec::new());
        assert!(matches!(
            hdr_image.write_pfm_image(&mut stream, ByteOrder::LittleEndian),
            Ok(())
        ));
        assert_eq!(_reference_bytes_le, stream.into_inner())
    }

    #[test]
    fn write_pfm_file() {
        let mut hdr_image = HdrImage::new(3, 2);

        hdr_image
            .set_pixel(0, 0, Color::from((1.0e1, 2.0e1, 3.0e1)))
            .unwrap();
        hdr_image
            .set_pixel(1, 0, Color::from((4.0e1, 5.0e1, 6.0e1)))
            .unwrap();
        hdr_image
            .set_pixel(2, 0, Color::from((7.0e1, 8.0e1, 9.0e1)))
            .unwrap();
        hdr_image
            .set_pixel(0, 1, Color::from((1.0e2, 2.0e2, 3.0e2)))
            .unwrap();
        hdr_image
            .set_pixel(1, 1, Color::from((4.0e2, 5.0e2, 6.0e2)))
            .unwrap();
        hdr_image
            .set_pixel(2, 1, Color::from((7.0e2, 8.0e2, 9.0e2)))
            .unwrap();

        assert!(matches!(
            hdr_image.write_pfm_file(String::from("/tmp/reference_be.pfm"), ByteOrder::BigEndian),
            Ok(())
        ));
        assert!(matches!(
            hdr_image.write_pfm_file("/tmp/reference_le.pfm", ByteOrder::LittleEndian),
            Ok(())
        ));
        assert!(matches!(
            hdr_image.write_pfm_file("/invalid_path", ByteOrder::LittleEndian),
            Err(std::io::Error { .. })
        ))
    }

    #[test]
    fn average_luminosity() {
        let mut hdr_img = HdrImage::new(2, 1);

        hdr_img
            .set_pixel(0, 0, Color::from((5.0, 10.0, 15.0)))
            .unwrap();
        hdr_img
            .set_pixel(1, 0, Color::from((500.0, 1000.0, 1500.0)))
            .unwrap();

        assert!(hdr_img.average_luminosity().is_close(100.0))
    }

    #[test]
    fn normalize_image() {
        let mut hdr_img = HdrImage::new(2, 1);

        hdr_img
            .set_pixel(0, 0, Color::from((5.0, 10.0, 15.0)))
            .unwrap();
        hdr_img
            .set_pixel(1, 0, Color::from((500.0, 1000.0, 1500.0)))
            .unwrap();

        hdr_img.normalize_image(1000.0, Luminosity::FloatValue(100.0));
        assert!(hdr_img
            .get_pixel(0, 0)
            .unwrap()
            .is_close(Color::from((0.5e2, 1.0e2, 1.5e2))));
        assert!(hdr_img
            .get_pixel(1, 0)
            .unwrap()
            .is_close(Color::from((0.5e4, 1.0e4, 1.5e4))))
    }

    #[test]
    fn clamp_image() {
        let mut hdr_img = HdrImage::new(2, 1);

        hdr_img
            .set_pixel(0, 0, Color::from((5.0, 10.0, 15.0)))
            .unwrap();
        hdr_img
            .set_pixel(1, 0, Color::from((500.0, 1000.0, 1500.0)))
            .unwrap();

        hdr_img.clamp_image();
        for pixel in hdr_img.pixels.iter() {
            for el in pixel.into_iter() {
                assert!(el >= 0.0 && el < 1.0);
            }
        }
    }
}
