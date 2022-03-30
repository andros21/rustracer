use byteorder::{ReadBytesExt, WriteBytesExt};
use image::{DynamicImage, ImageFormat, Rgb, Rgba};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::str::FromStr;
use std::vec::Vec;

use crate::color::Color;
use crate::error::HdrImageErr;

const DELTA: f32 = 1e-10;

#[derive(Clone, Debug, PartialEq)]
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
        x < self.width && y < self.height
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
            self.pixels[pixel_offset] = new_color;
            Ok(())
        } else {
            Err(HdrImageErr::OutOfBounds((x, y), (self.width, self.height)))
        }
    }

    fn read_pfm_image<R: BufRead>(buf_reader: &mut R) -> Result<HdrImage, HdrImageErr> {
        let mut line = String::new();
        buf_reader
            .read_line(&mut line)
            .map_err(HdrImageErr::PfmFileReadFailure)?;
        check_eol(line.ends_with('\n'))?;
        if line.trim_end() != "PF" {
            return Err(HdrImageErr::InvalidPfmFileFormat(String::from(
                "wrong magic inside header",
            )));
        }
        line.clear();
        buf_reader
            .read_line(&mut line)
            .map_err(HdrImageErr::PfmFileReadFailure)?;
        check_eol(line.ends_with('\n'))?;
        let (width, height) = parse_img_shape(line.trim_end())?;
        line.clear();
        buf_reader
            .read_line(&mut line)
            .map_err(HdrImageErr::PfmFileReadFailure)?;
        check_eol(line.ends_with('\n'))?;
        let endianness: ByteOrder = parse_endianness(line.trim_end())?;
        line.clear();
        let mut buffer = [0_f32; 3];
        let mut hdr_img = HdrImage::new(width, height);
        for y in (0..height).rev() {
            for x in 0..width {
                match endianness {
                    ByteOrder::LittleEndian => buf_reader
                        .read_f32_into::<byteorder::LittleEndian>(&mut buffer)
                        .map_err(HdrImageErr::PfmFileReadFailure)?,
                    ByteOrder::BigEndian => buf_reader
                        .read_f32_into::<byteorder::BigEndian>(&mut buffer)
                        .map_err(HdrImageErr::PfmFileReadFailure)?,
                }
                hdr_img.set_pixel(x, y, (buffer[0], buffer[1], buffer[2]).into())?;
            }
        }
        if buf_reader.read_line(&mut line).unwrap_or(1) == 0 {
            Ok(hdr_img)
        } else {
            Err(HdrImageErr::InvalidPfmFileFormat(String::from(
                "find binary content, expected eof",
            )))
        }
    }

    pub fn read_pfm_file(path: &Path) -> Result<HdrImage, HdrImageErr> {
        let file = File::open(path).map_err(HdrImageErr::PfmFileReadFailure)?;
        let mut buf_reader = BufReader::new(file);
        HdrImage::read_pfm_image(&mut buf_reader)
    }

    fn write_pfm_image<W: Write>(
        &self,
        stream: &mut W,
        endianness: ByteOrder,
    ) -> std::io::Result<()> {
        let mut header = format!("PF\n{} {}\n", self.width, self.height);
        match endianness {
            ByteOrder::BigEndian => header.push_str("1.0\n"),
            ByteOrder::LittleEndian => header.push_str("-1.0\n"),
        }
        stream.write_all(header.as_bytes())?;
        for y in (0..self.height).rev() {
            for x in 0..self.width {
                for el in self.get_pixel(x, y).unwrap().into_iter() {
                    write_float(stream, el, &endianness)?;
                }
            }
        }
        Ok(())
    }

    pub fn write_pfm_file(&self, path: &Path, endianness: ByteOrder) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        self.write_pfm_image(&mut writer, endianness)
    }

    fn average_luminosity(&self) -> f32 {
        let mut sum = 0.0;
        for pixel in self.pixels.iter() {
            sum += f32::log10(DELTA + pixel.luminosity());
        }
        f32::powf(10.0, sum / (self.pixels.len() as f32))
    }

    pub fn normalize_image(&mut self, factor: f32, luminosity: Luminosity) {
        let lum = match luminosity {
            Luminosity::AverageLuminosity => self.average_luminosity(),
            Luminosity::FloatValue(val) => val,
        };
        for i in 0..self.pixels.len() {
            self.pixels[i] = self.pixels[i] * (factor / lum);
        }
    }

    pub fn clamp_image(&mut self) {
        for pixel in self.pixels.iter_mut() {
            pixel.r = clamp(pixel.r);
            pixel.g = clamp(pixel.g);
            pixel.b = clamp(pixel.b);
        }
    }

    pub fn write_ldr_file(&self, path: &Path, gamma: f32) -> Result<(), HdrImageErr> {
        let format = ImageFormat::from_path(&path).map_err(HdrImageErr::LdrFileWriteFailure)?;
        match format {
            ImageFormat::Farbfeld => {
                let mut ldr_img = DynamicImage::new_rgb16(self.width, self.height).into_rgba16();
                for y in 0..self.height {
                    for x in 0..self.width {
                        let pixel = self.get_pixel(x, y).unwrap();
                        ldr_img.put_pixel(
                            x,
                            y,
                            Rgba([
                                (65535.0 * f32::powf(pixel.r, 1.0 / gamma)) as u16,
                                (65535.0 * f32::powf(pixel.g, 1.0 / gamma)) as u16,
                                (65535.0 * f32::powf(pixel.b, 1.0 / gamma)) as u16,
                                65535_u16,
                            ]),
                        )
                    }
                }
                ldr_img
                    .save_with_format(&path, format)
                    .map_err(HdrImageErr::LdrFileWriteFailure)
            }
            ImageFormat::Png => {
                let mut ldr_img = DynamicImage::new_rgb8(self.width, self.height).into_rgb8();
                for y in 0..self.height {
                    for x in 0..self.width {
                        let pixel = self.get_pixel(x, y).unwrap();
                        ldr_img.put_pixel(
                            x,
                            y,
                            Rgb([
                                (255.0 * f32::powf(pixel.r, 1.0 / gamma)) as u8,
                                (255.0 * f32::powf(pixel.g, 1.0 / gamma)) as u8,
                                (255.0 * f32::powf(pixel.b, 1.0 / gamma)) as u8,
                            ]),
                        )
                    }
                }
                ldr_img
                    .save_with_format(&path, format)
                    .map_err(HdrImageErr::LdrFileWriteFailure)
            }
            _ => {
                return Err(HdrImageErr::UnsupportedLdrFileFormat(String::from(
                    path.extension().unwrap().to_str().unwrap_or(""),
                )))
            }
        }
    }
}

fn check_eol(eol: bool) -> Result<(), HdrImageErr> {
    if eol {
        Ok(())
    } else {
        Err(HdrImageErr::InvalidPfmFileFormat(String::from(
            "expected eol as separator inside header",
        )))
    }
}

pub enum ByteOrder {
    BigEndian,
    LittleEndian,
}

fn parse_img_shape(line: &str) -> Result<(u32, u32), HdrImageErr> {
    let shape: Vec<&str> = line.split(' ').filter(|s| s != &"").collect();
    if shape.len() == 2 {
        let width = u32::from_str(shape[0])
            .map_err(|e| HdrImageErr::PfmIntParseFailure(e, String::from("image width")))?;
        let height = u32::from_str(shape[1])
            .map_err(|e| HdrImageErr::PfmIntParseFailure(e, String::from("image height")))?;
        Ok((width, height))
    } else {
        Err(HdrImageErr::InvalidPfmFileFormat(String::from(
            "wrong image shape inside header",
        )))
    }
}

fn parse_endianness(line: &str) -> Result<ByteOrder, HdrImageErr> {
    let value = f32::from_str(line)
        .map_err(|e| HdrImageErr::PfmFloatParseFailure(e, String::from("endianness")))?;
    if value > 0.0 {
        Ok(ByteOrder::BigEndian)
    } else if value < 0.0 {
        Ok(ByteOrder::LittleEndian)
    } else {
        Err(HdrImageErr::InvalidPfmFileFormat(String::from(
            "invalid endianness inside header",
        )))
    }
}

fn write_float<W: Write>(
    stream: &mut W,
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
    use crate::color::IsClose;
    use std::io::Cursor;

    #[test]
    fn test_image_creation() {
        let hdr_img = HdrImage::new(5, 7);

        assert_eq!(hdr_img.width, 5);
        assert_eq!(hdr_img.height, 7)
    }

    #[test]
    fn test_validate_coordinates() {
        let hdr_img = HdrImage::new(5, 7);

        assert!(hdr_img.valid_coordinates(0, 0));
        assert!(hdr_img.valid_coordinates(4, 3));
        assert!(!hdr_img.valid_coordinates(7, 0));
        assert!(!hdr_img.valid_coordinates(0, 9));
        assert!(!hdr_img.valid_coordinates(7, 9))
    }

    #[test]
    fn test_pixel_offset() {
        let hdr_img = HdrImage::new(7, 4);

        assert_eq!(hdr_img.pixel_offset(0, 0), 0);
        assert_eq!(hdr_img.pixel_offset(3, 2), 17);
        assert_eq!(hdr_img.pixel_offset(6, 3), 7 * 4 - 1)
    }

    #[test]
    fn test_get_pixel() {
        let color: Color = Default::default();

        assert!(matches!(HdrImage::new(3, 3).get_pixel(0, 0), Ok(col) if col == color));
        assert!(matches!(HdrImage::new(3, 3).get_pixel(3, 3),
            Err(HdrImageErr::OutOfBounds(a, b)) if a == (3, 3) && b == (3, 3)))
    }

    #[test]
    fn test_set_pixel() {
        let color1 = Color::from((1.0, 1.0, 1.0));
        let color2 = Color::from((1.23, 4.56, 7.89));

        let mut hdr_img = HdrImage::new(3, 3);
        assert!(matches!(hdr_img.set_pixel(0, 0, color1), Ok(())));
        assert!(matches!(hdr_img.set_pixel(2, 2, color2), Ok(())));
        assert!(matches!(hdr_img.get_pixel(0, 0), Ok(col1) if col1 == color1));
        assert!(hdr_img.get_pixel(2, 2).unwrap().is_close(color2));
        assert!(matches!(hdr_img.set_pixel(5, 5, color1),
            Err(HdrImageErr::OutOfBounds(a, b)) if a == (5, 5) && b == (3, 3)))
    }

    #[test]
    fn test_parse_img_shape() {
        let mut line = "10 20";

        assert!(matches!(parse_img_shape(&line), Ok((10, 20))));
        line = " 10    20  ";
        assert!(matches!(parse_img_shape(&line), Ok((10, 20))));
        line = "10 20 30";
        assert!(matches!(
            parse_img_shape(&line),
            Err(HdrImageErr::InvalidPfmFileFormat(_))
        ));
        line = "10 ";
        assert!(matches!(
            parse_img_shape(&line),
            Err(HdrImageErr::InvalidPfmFileFormat(_))
        ));
        line = "102030";
        assert!(matches!(
            parse_img_shape(&line),
            Err(HdrImageErr::InvalidPfmFileFormat(_))
        ));
        line = "10 20.1";
        assert!(matches!(
            parse_img_shape(&line),
            Err(HdrImageErr::PfmIntParseFailure(_, msg)) if msg.as_str() == "image height"
        ));
        line = "-10 20";
        assert!(matches!(
            parse_img_shape(&line),
            Err(HdrImageErr::PfmIntParseFailure(_, msg)) if msg.as_str() == "image width"
        ));
        line = "abc 20";
        assert!(matches!(
            parse_img_shape(&line),
            Err(HdrImageErr::PfmIntParseFailure(_, msg)) if msg.as_str() == "image width"
        ))
    }

    #[test]
    fn test_parse_endianness() {
        let mut line = "-3.2";

        assert!(matches!(
            parse_endianness(&line),
            Ok(ByteOrder::LittleEndian)
        ));
        line = "1e15";
        assert!(matches!(parse_endianness(&line), Ok(ByteOrder::BigEndian)));
        line = "0";
        assert!(matches!(
            parse_endianness(&line),
            Err(HdrImageErr::InvalidPfmFileFormat(_))
        ));
        line = "abc";
        assert!(matches!(
            parse_endianness(&line),
            Err(HdrImageErr::PfmFloatParseFailure(..))
        ));
    }

    #[test]
    fn test_read_pfm_image() {
        let mut hdr_img = HdrImage::new(3, 2);

        hdr_img
            .set_pixel(0, 0, Color::from((1.0e1, 2.0e1, 3.0e1)))
            .unwrap();
        hdr_img
            .set_pixel(1, 0, Color::from((4.0e1, 5.0e1, 6.0e1)))
            .unwrap();
        hdr_img
            .set_pixel(2, 0, Color::from((7.0e1, 8.0e1, 9.0e1)))
            .unwrap();
        hdr_img
            .set_pixel(0, 1, Color::from((1.0e2, 2.0e2, 3.0e2)))
            .unwrap();
        hdr_img
            .set_pixel(1, 1, Color::from((4.0e2, 5.0e2, 6.0e2)))
            .unwrap();
        hdr_img
            .set_pixel(2, 1, Color::from((7.0e2, 8.0e2, 9.0e2)))
            .unwrap();

        let reference_bytes_be = vec![
            0x50, 0x46, 0x0a, 0x33, 0x20, 0x32, 0x0a, 0x31, 0x2e, 0x30, 0x0a, 0x42, 0xc8, 0x00,
            0x00, 0x43, 0x48, 0x00, 0x00, 0x43, 0x96, 0x00, 0x00, 0x43, 0xc8, 0x00, 0x00, 0x43,
            0xfa, 0x00, 0x00, 0x44, 0x16, 0x00, 0x00, 0x44, 0x2f, 0x00, 0x00, 0x44, 0x48, 0x00,
            0x00, 0x44, 0x61, 0x00, 0x00, 0x41, 0x20, 0x00, 0x00, 0x41, 0xa0, 0x00, 0x00, 0x41,
            0xf0, 0x00, 0x00, 0x42, 0x20, 0x00, 0x00, 0x42, 0x48, 0x00, 0x00, 0x42, 0x70, 0x00,
            0x00, 0x42, 0x8c, 0x00, 0x00, 0x42, 0xa0, 0x00, 0x00, 0x42, 0xb4, 0x00, 0x00,
        ];
        let mut reference_bytes_le = vec![
            0x50, 0x46, 0x0a, 0x33, 0x20, 0x32, 0x0a, 0x2d, 0x31, 0x2e, 0x30, 0x0a, 0x00, 0x00,
            0xc8, 0x42, 0x00, 0x00, 0x48, 0x43, 0x00, 0x00, 0x96, 0x43, 0x00, 0x00, 0xc8, 0x43,
            0x00, 0x00, 0xfa, 0x43, 0x00, 0x00, 0x16, 0x44, 0x00, 0x00, 0x2f, 0x44, 0x00, 0x00,
            0x48, 0x44, 0x00, 0x00, 0x61, 0x44, 0x00, 0x00, 0x20, 0x41, 0x00, 0x00, 0xa0, 0x41,
            0x00, 0x00, 0xf0, 0x41, 0x00, 0x00, 0x20, 0x42, 0x00, 0x00, 0x48, 0x42, 0x00, 0x00,
            0x70, 0x42, 0x00, 0x00, 0x8c, 0x42, 0x00, 0x00, 0xa0, 0x42, 0x00, 0x00, 0xb4, 0x42,
        ];

        let mut stream = Cursor::new(reference_bytes_be);
        let hdr_img_result = HdrImage::read_pfm_image(&mut stream);
        assert!(matches!(hdr_img_result, Ok(ref img) if img == &hdr_img));
        assert_eq!(&hdr_img_result.unwrap(), &hdr_img);

        let mut stream = Cursor::new(&reference_bytes_le);
        let hdr_img_result = HdrImage::read_pfm_image(&mut stream);
        assert!(matches!(hdr_img_result, Ok(ref img) if img == &hdr_img));
        assert_eq!(&hdr_img_result.unwrap(), &hdr_img);

        let wrong_magic = vec![0x46, 0x50, 0x0a];
        let wrong_st = vec![0xff, 0xff, 0x0a];
        let no_st_eol = vec![0x50, 0x46];
        let wrong_nd = vec![0x50, 0x46, 0x0a, 0xff, 0xff, 0x0a];
        let no_rd_eol = vec![
            0x50, 0x46, 0x0a, 0x33, 0x20, 0x32, 0x0a, 0x2d, 0x31, 0x2e, 0x30,
        ];

        let mut stream = Cursor::new(wrong_magic);
        let hdr_img_result = HdrImage::read_pfm_image(&mut stream);
        assert!(matches!(
        hdr_img_result,
        Err(HdrImageErr::InvalidPfmFileFormat(msg)) if msg.as_str() == "wrong magic inside header"
        ));

        let mut stream = Cursor::new(wrong_st);
        let hdr_img_result = HdrImage::read_pfm_image(&mut stream);
        assert!(matches!(
            hdr_img_result,
            Err(HdrImageErr::PfmFileReadFailure(_))
        ));

        let mut stream = Cursor::new(no_st_eol);
        let hdr_img_result = HdrImage::read_pfm_image(&mut stream);
        assert!(matches!(
        hdr_img_result,
        Err(HdrImageErr::InvalidPfmFileFormat(msg)) if msg.as_str() == "expected eol as separator inside header"
        ));

        let mut stream = Cursor::new(wrong_nd);
        let hdr_img_result = HdrImage::read_pfm_image(&mut stream);
        assert!(matches!(
            hdr_img_result,
            Err(HdrImageErr::PfmFileReadFailure(_))
        ));

        let mut stream = Cursor::new(no_rd_eol);
        let hdr_img_result = HdrImage::read_pfm_image(&mut stream);
        assert!(matches!(
        hdr_img_result,
        Err(HdrImageErr::InvalidPfmFileFormat(msg)) if msg.as_str() == "expected eol as separator inside header"
        ));

        reference_bytes_le.push(0x00);
        let mut stream = Cursor::new(&reference_bytes_le);
        let hdr_img_result = HdrImage::read_pfm_image(&mut stream);
        assert!(matches!(
        hdr_img_result,
        Err(HdrImageErr::InvalidPfmFileFormat(msg)) if msg.as_str() == "find binary content, expected eof"
        ));

        reference_bytes_le.truncate(&reference_bytes_le.len() - 2);
        let mut stream = Cursor::new(&reference_bytes_le);
        let hdr_img_result = HdrImage::read_pfm_image(&mut stream);
        assert!(matches!(
            hdr_img_result,
            Err(HdrImageErr::PfmFileReadFailure(_))
        ));
    }

    #[test]
    fn test_write_pfm_image() {
        let mut hdr_img = HdrImage::new(3, 2);

        hdr_img
            .set_pixel(0, 0, Color::from((1.0e1, 2.0e1, 3.0e1)))
            .unwrap();
        hdr_img
            .set_pixel(1, 0, Color::from((4.0e1, 5.0e1, 6.0e1)))
            .unwrap();
        hdr_img
            .set_pixel(2, 0, Color::from((7.0e1, 8.0e1, 9.0e1)))
            .unwrap();
        hdr_img
            .set_pixel(0, 1, Color::from((1.0e2, 2.0e2, 3.0e2)))
            .unwrap();
        hdr_img
            .set_pixel(1, 1, Color::from((4.0e2, 5.0e2, 6.0e2)))
            .unwrap();
        hdr_img
            .set_pixel(2, 1, Color::from((7.0e2, 8.0e2, 9.0e2)))
            .unwrap();

        let reference_bytes_be = vec![
            0x50, 0x46, 0x0a, 0x33, 0x20, 0x32, 0x0a, 0x31, 0x2e, 0x30, 0x0a, 0x42, 0xc8, 0x00,
            0x00, 0x43, 0x48, 0x00, 0x00, 0x43, 0x96, 0x00, 0x00, 0x43, 0xc8, 0x00, 0x00, 0x43,
            0xfa, 0x00, 0x00, 0x44, 0x16, 0x00, 0x00, 0x44, 0x2f, 0x00, 0x00, 0x44, 0x48, 0x00,
            0x00, 0x44, 0x61, 0x00, 0x00, 0x41, 0x20, 0x00, 0x00, 0x41, 0xa0, 0x00, 0x00, 0x41,
            0xf0, 0x00, 0x00, 0x42, 0x20, 0x00, 0x00, 0x42, 0x48, 0x00, 0x00, 0x42, 0x70, 0x00,
            0x00, 0x42, 0x8c, 0x00, 0x00, 0x42, 0xa0, 0x00, 0x00, 0x42, 0xb4, 0x00, 0x00,
        ];
        let reference_bytes_le = vec![
            0x50, 0x46, 0x0a, 0x33, 0x20, 0x32, 0x0a, 0x2d, 0x31, 0x2e, 0x30, 0x0a, 0x00, 0x00,
            0xc8, 0x42, 0x00, 0x00, 0x48, 0x43, 0x00, 0x00, 0x96, 0x43, 0x00, 0x00, 0xc8, 0x43,
            0x00, 0x00, 0xfa, 0x43, 0x00, 0x00, 0x16, 0x44, 0x00, 0x00, 0x2f, 0x44, 0x00, 0x00,
            0x48, 0x44, 0x00, 0x00, 0x61, 0x44, 0x00, 0x00, 0x20, 0x41, 0x00, 0x00, 0xa0, 0x41,
            0x00, 0x00, 0xf0, 0x41, 0x00, 0x00, 0x20, 0x42, 0x00, 0x00, 0x48, 0x42, 0x00, 0x00,
            0x70, 0x42, 0x00, 0x00, 0x8c, 0x42, 0x00, 0x00, 0xa0, 0x42, 0x00, 0x00, 0xb4, 0x42,
        ];

        let mut stream = Cursor::new(Vec::new());
        assert!(matches!(
            hdr_img.write_pfm_image(&mut stream, ByteOrder::BigEndian),
            Ok(())
        ));
        assert_eq!(reference_bytes_be, stream.into_inner());

        let mut stream = Cursor::new(Vec::new());
        assert!(matches!(
            hdr_img.write_pfm_image(&mut stream, ByteOrder::LittleEndian),
            Ok(())
        ));
        assert_eq!(reference_bytes_le, stream.into_inner());

        let mut stream = Cursor::new(reference_bytes_le);
        assert!(matches!(
            hdr_img.write_pfm_image(&mut stream, ByteOrder::LittleEndian),
            Ok(())
        ));
        stream.set_position(0);
        let hdr_img_result = HdrImage::read_pfm_image(&mut stream);
        assert!(matches!(hdr_img_result, Ok(ref img) if img == &hdr_img));
        assert_eq!(hdr_img_result.unwrap(), hdr_img);
    }

    #[test]
    fn test_write_pfm_file() {
        let mut hdr_img = HdrImage::new(3, 2);

        hdr_img
            .set_pixel(0, 0, Color::from((1.0e1, 2.0e1, 3.0e1)))
            .unwrap();
        hdr_img
            .set_pixel(1, 0, Color::from((4.0e1, 5.0e1, 6.0e1)))
            .unwrap();
        hdr_img
            .set_pixel(2, 0, Color::from((7.0e1, 8.0e1, 9.0e1)))
            .unwrap();
        hdr_img
            .set_pixel(0, 1, Color::from((1.0e2, 2.0e2, 3.0e2)))
            .unwrap();
        hdr_img
            .set_pixel(1, 1, Color::from((4.0e2, 5.0e2, 6.0e2)))
            .unwrap();
        hdr_img
            .set_pixel(2, 1, Color::from((7.0e2, 8.0e2, 9.0e2)))
            .unwrap();

        let reference_file_be = Path::new("/tmp/reference_be.pfm");
        let reference_file_le = Path::new("/tmp/reference_le.pfm");
        let invalid_path = Path::new("/invalid");
        let invalid_file = Path::new("/tmp/invalid");

        assert!(matches!(
            hdr_img.write_pfm_file(&reference_file_be, ByteOrder::BigEndian),
            Ok(())
        ));
        assert!(matches!(
            hdr_img.write_pfm_file(&reference_file_le, ByteOrder::LittleEndian),
            Ok(())
        ));
        assert!(matches!(
            hdr_img.write_pfm_file(&invalid_path, ByteOrder::LittleEndian),
            Err(std::io::Error { .. })
        ));

        let hdr_img_result = HdrImage::read_pfm_file(&reference_file_be);
        assert!(matches!(hdr_img_result, Ok(ref img) if img == &hdr_img));
        assert_eq!(hdr_img_result.unwrap(), hdr_img);

        let hdr_img_result = HdrImage::read_pfm_file(&reference_file_le);
        assert!(matches!(hdr_img_result, Ok(ref img) if img == &hdr_img));
        assert_eq!(hdr_img_result.unwrap(), hdr_img);

        let hdr_img_result = HdrImage::read_pfm_file(&invalid_file);
        assert!(matches!(
            hdr_img_result,
            Err(HdrImageErr::PfmFileReadFailure(_))
        ))
    }

    #[test]
    fn test_average_luminosity() {
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
    fn test_normalize_image() {
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
    fn test_clamp_image() {
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

    #[test]
    fn test_write_ldr_file() {
        let reference_bytes_le = vec![
            0x50, 0x46, 0x0a, 0x33, 0x20, 0x32, 0x0a, 0x2d, 0x31, 0x2e, 0x30, 0x0a, 0x00, 0x00,
            0xc8, 0x42, 0x00, 0x00, 0x48, 0x43, 0x00, 0x00, 0x96, 0x43, 0x00, 0x00, 0xc8, 0x43,
            0x00, 0x00, 0xfa, 0x43, 0x00, 0x00, 0x16, 0x44, 0x00, 0x00, 0x2f, 0x44, 0x00, 0x00,
            0x48, 0x44, 0x00, 0x00, 0x61, 0x44, 0x00, 0x00, 0x20, 0x41, 0x00, 0x00, 0xa0, 0x41,
            0x00, 0x00, 0xf0, 0x41, 0x00, 0x00, 0x20, 0x42, 0x00, 0x00, 0x48, 0x42, 0x00, 0x00,
            0x70, 0x42, 0x00, 0x00, 0x8c, 0x42, 0x00, 0x00, 0xa0, 0x42, 0x00, 0x00, 0xb4, 0x42,
        ];
        let mut stream = Cursor::new(reference_bytes_le);
        let mut hdr_img = HdrImage::read_pfm_image(&mut stream).unwrap();
        hdr_img.normalize_image(1000.0, Luminosity::FloatValue(100.0));
        hdr_img.clamp_image();

        let invalid_format = Path::new("/tmp/reference_le.mkv");
        let unsupported_format = Path::new("/tmp/reference_le.jpeg");
        let invalid_path = Path::new("/invalid_path/reference_le.png");
        let reference_png = Path::new("/tmp/reference_le.png");
        let reference_ff = Path::new("/tmp/reference_le.ff");

        assert!(matches!(
            hdr_img.write_ldr_file(&invalid_format, 1.0),
            Err(HdrImageErr::LdrFileWriteFailure(_))
        ));
        assert!(matches!(
            hdr_img.write_ldr_file(&unsupported_format, 1.0),
            Err(HdrImageErr::UnsupportedLdrFileFormat(format)) if format.as_str() == "jpeg"
        ));
        assert!(matches!(
            hdr_img.write_ldr_file(&invalid_path, 1.0),
            Err(HdrImageErr::LdrFileWriteFailure(_))
        ));
        assert!(matches!(
            hdr_img.write_ldr_file(&reference_png, 1.0),
            Ok(())
        ));
        assert!(matches!(hdr_img.write_ldr_file(&reference_ff, 1.0), Ok(())))
    }
}
