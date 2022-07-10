//! High Dynamic Range Image module.
//!
//! Provides [`HdrImage`](struct@HdrImage) struct.

use byteorder::{ReadBytesExt, WriteBytesExt};
use image::{DynamicImage, ImageFormat, Rgb, Rgba};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::str::FromStr;
use std::vec::Vec;

use crate::color::Color;
use crate::error::HdrImageErr;
use crate::misc::ByteOrder;

const DELTA: f32 = 1e-10;

/// High Dynamic Range Image struct.
#[derive(Clone, Debug, PartialEq)]
pub struct HdrImage {
    /// Number of columns in the 2D matrix of colors.
    width: u32,
    /// Number of rows in the 2D matrix of colors.
    height: u32,
    /// The 2D matrix, represented as a 1D [`std::vec::Vec`] of [`Color`].
    pixels: Vec<Color>,
}

impl HdrImage {
    /// Create a black image with the specified resolution.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            pixels: vec![Color::default(); (width * height) as usize],
        }
    }

    /// Get pixels matrix shape `(width, heigth)`.
    pub fn shape(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Return the position in the 1D array of the specified pixel.
    fn pixel_offset(&self, x: u32, y: u32) -> usize {
        (y * self.width + x) as usize
    }

    /// Return `true` if `(x, y)` are coordinates within the 2D matrix.
    fn valid_coordinates(&self, x: u32, y: u32) -> bool {
        x < self.width && y < self.height
    }

    /// Return the [`Color`] value for a pixel in the image inside a `std::result::Result`.
    ///
    /// For invalid `(x, y)` coordinates the result is an [`HdrImageErr`] error variant.
    ///
    /// The pixel at the top-left corner has coordinates `(0, 0)`.
    pub fn get_pixel(&self, x: u32, y: u32) -> Result<Color, HdrImageErr> {
        if self.valid_coordinates(x, y) {
            Ok(self.pixels[self.pixel_offset(x, y)])
        } else {
            Err(HdrImageErr::OutOfBounds((x, y), (self.width, self.height)))
        }
    }

    /// Set the new [`Color`] for a pixel in the image.
    ///
    /// For invalid `(x, y)` coordinates the result is an [`HdrImageErr`] error variant.
    ///
    /// The pixel at the top-left corner has coordinates `(0, 0)`.
    pub fn set_pixel(&mut self, x: u32, y: u32, new_color: Color) -> Result<(), HdrImageErr> {
        if self.valid_coordinates(x, y) {
            let pixel_offset = self.pixel_offset(x, y);
            self.pixels[pixel_offset] = new_color;
            Ok(())
        } else {
            Err(HdrImageErr::OutOfBounds((x, y), (self.width, self.height)))
        }
    }

    /// Set pixels matrix, from underlying [`Vec`] structure, of the correct size.
    ///
    /// Otherwise return [`HdrImageErr::InvalidPixelsSize`] variant.
    pub fn set_pixels(&mut self, pixels: Vec<Color>) -> Result<(), HdrImageErr> {
        if self.pixels.len() == pixels.len() {
            self.pixels = pixels;
            Ok(())
        } else {
            Err(HdrImageErr::InvalidPixelsSize(
                pixels.len() as u32,
                self.pixels.len() as u32,
            ))
        }
    }

    /// Read a pfm image from `buf_reader` with [`std::io::BufRead`] trait implementation.
    ///
    /// The expected input buffer must respect pfm format:
    /// ```text
    /// PF              # ASCII text, '\n' as line separator
    /// width height    # ASCII text, '\n' as line separator
    /// ±1.0            # ASCII text, '\n' as line separator
    /// pixels matrix   # binary content (float RGB x #pixels)
    /// ```
    /// Little description:
    /// * `PF` - the magic
    /// * `width` `height` - image shape (**note:** space separated)
    /// * `±1.0` - endianness (`+1` big endian, `-1` little endian) float codification
    /// * `pixels matrix` - matrix of RGB float pixels encoded as function of endianness
    ///
    /// Possible read failures, in precedence order:
    /// 1. Lack of first end line ([`check_eol`])
    /// 2. Invalid magic (no `PF`)
    /// 3. Lack of second end line ([`check_eol`])
    /// 4. Invalid image shape ([`parse_img_shape`])
    /// 5. Lack of third end line ([`check_eol`])
    /// 6. Invalid endianness ([`parse_endianness`])
    /// 7. Invalid pixels matrix (error parsing float RGB)
    /// 8. Invalid EOF (unexpected binary content after pixels matrix read)
    ///
    ///  Parse of [`f32`] from binary stream using
    ///  [`byteorder`](https://github.com/BurntSushi/byteorder) library.\
    ///  Return a [`HdrImage`] object containing the image inside a [`std::result::Result`].\
    ///  If an error occurs the result contains an [`HdrImageErr`] error variant.
    fn read_pfm_image<R: BufRead>(buf_reader: &mut R) -> Result<Self, HdrImageErr> {
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
        match endianness {
            ByteOrder::LittleEndian => {
                for y in (0..height).rev() {
                    for x in 0..width {
                        buf_reader
                            .read_f32_into::<byteorder::LittleEndian>(&mut buffer)
                            .map_err(HdrImageErr::PfmFileReadFailure)?;
                        hdr_img.set_pixel(x, y, (buffer[0], buffer[1], buffer[2]).into())?;
                    }
                }
            },
            ByteOrder::BigEndian => {
                for y in (0..height).rev() {
                    for x in 0..width {
                        buf_reader
                            .read_f32_into::<byteorder::BigEndian>(&mut buffer)
                            .map_err(HdrImageErr::PfmFileReadFailure)?;
                        hdr_img.set_pixel(x, y, (buffer[0], buffer[1], buffer[2]).into())?;
                    }
                }
            },
        };
        if buf_reader.read_line(&mut line).unwrap_or(1) == 0 {
            Ok(hdr_img)
        } else {
            Err(HdrImageErr::InvalidPfmFileFormat(String::from(
                "find binary content, expected eof",
            )))
        }
    }

    /// Read a pfm image from `path`, wrapper function around
    /// [`read_pfm_image`](#method.read_pfm_image).
    ///
    ///  Return a [`HdrImage`] object containing the image inside a [`std::result::Result`].\
    ///  If an error occurs the result contains an [`HdrImageErr`] error variant.
    pub fn read_pfm_file(path: &Path) -> Result<Self, HdrImageErr> {
        let file = File::open(path).map_err(HdrImageErr::PfmFileReadFailure)?;
        let mut buf_reader = BufReader::new(file);
        HdrImage::read_pfm_image(&mut buf_reader)
    }

    /// Write a pfm image to `stream` with [`std::io::Write`] trait implementation.
    ///
    /// The enum [`endianness`](enum@ByteOrder) specifies the byte endianness
    /// to be used in the file.
    ///
    /// If an error occurs the result contains an [`HdrImageErr`] error variant.
    fn write_pfm_image<W: Write>(
        &self,
        stream: &mut W,
        endianness: ByteOrder,
    ) -> Result<(), HdrImageErr> {
        let mut header = format!("PF\n{} {}\n", self.width, self.height);
        match endianness {
            ByteOrder::BigEndian => header.push_str("1.0\n"),
            ByteOrder::LittleEndian => header.push_str("-1.0\n"),
        }
        stream
            .write_all(header.as_bytes())
            .map_err(HdrImageErr::PfmFileWriteFailure)?;
        for y in (0..self.height).rev() {
            for x in 0..self.width {
                for el in self.get_pixel(x, y).unwrap().into_iter() {
                    write_float(stream, el, &endianness)
                        .map_err(HdrImageErr::PfmFileWriteFailure)?;
                }
            }
        }
        Ok(())
    }

    /// Write a pfm image to `path`, wrapper function around
    /// [`write_pfm_image`](#method.write_pfm_image).
    ///
    /// If an error occurs the result contains an [`HdrImageErr`] error variant.
    pub fn write_pfm_file(&self, path: &Path, endianness: ByteOrder) -> Result<(), HdrImageErr> {
        let file = File::create(path).map_err(HdrImageErr::PfmFileWriteFailure)?;
        let mut writer = BufWriter::new(file);
        self.write_pfm_image(&mut writer, endianness)
    }

    /// Return the average luminosity of the image.
    ///
    /// The [`DELTA`] constant is used to prevent  numerical problems
    /// for under illuminated pixels.
    fn average_luminosity(&self) -> f32 {
        let mut sum = 0.0;
        for pixel in self.pixels.iter() {
            sum += f32::log10(DELTA + pixel.luminosity());
        }
        f32::powf(10.0, sum / (self.pixels.len() as f32))
    }

    /// Normalize the image for a given luminosity.
    ///
    /// `factor` is normalization factor.\
    /// Different variants of [`luminosity`](enum@Luminosity) enum can be chosen.
    pub fn normalize_image(&mut self, factor: f32, luminosity: Luminosity) {
        let lum = match luminosity {
            Luminosity::AverageLuminosity => self.average_luminosity(),
            Luminosity::FloatValue(val) => val,
        };
        for i in 0..self.pixels.len() {
            self.pixels[i] = self.pixels[i] * (factor / lum);
        }
    }

    /// Adjust the color levels of the brightest pixels in the image.
    pub fn clamp_image(&mut self) {
        for pixel in self.pixels.iter_mut() {
            pixel.r = clamp(pixel.r);
            pixel.g = clamp(pixel.g);
            pixel.b = clamp(pixel.b);
        }
    }

    /// Save the image in a Low Dynamic Range (LDR) format,
    /// using [`image`](https://github.com/image-rs/image) library.
    ///
    /// `gamma` is transfer function parameter.
    ///
    /// **Note:** the output format is auto-detected from the file name extension,\
    /// only two LDR image format are supported `.ff` ([`farbfeld`](https://tools.suckless.org/farbfeld/))
    /// and `.png` ([`PNG`](https://en.wikipedia.org/wiki/Portable_Network_Graphics)).
    ///
    /// **Note:** before calling this function, you should apply a
    /// tone-mapping algorithm to the image and \
    /// be sure that the RGB values of the colors in the image are all in the range `[0, 1]`.\
    /// Use [`normalize_image`](#method.normalize_image)
    /// and [`clamp_image`](#method.clamp_image) to do this.
    ///
    /// In case of errors, `std::result::Result` is an [`HdrImageErr`] error variant.
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
            },
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
            },
            _ => {
                return Err(HdrImageErr::UnsupportedLdrFileFormat(String::from(
                    path.extension().unwrap().to_str().unwrap_or(""),
                )))
            },
        }
    }
}

/// Boolean check end of line.
///
/// Return an [`HdrImageErr::InvalidPfmFileFormat`]
/// as function of passed boolean `eol`.
/// Where `eol` is `true` if contain '\n', `false` if not contain `\n`.
fn check_eol(eol: bool) -> Result<(), HdrImageErr> {
    if eol {
        Ok(())
    } else {
        Err(HdrImageErr::InvalidPfmFileFormat(String::from(
            "expected eol as separator inside header",
        )))
    }
}

/// Parse image shape, [`u32`] tuple, from string.
///
/// Width and height inside string must be separated by one (or more spaces).
///
/// If parse fails, the [`std::result::Result`] will be an [`HdrImageErr`] error variant.
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

/// Parse image endianness from string.
///
/// If parse successes a particular variant of [`ByteOrder`] is returned
/// wrapped around a [`std::result::Result`].\
/// If parse fails the [`std::result::Result`] will be an [`HdrImageErr`] error variant.
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

/// Write [`f32`] value to stream. \
///
/// With `stream` implemented [`std::io::Write`] trait and `value` encoded as function of
/// [`endianness`](enum@ByteOrder).
///
/// Using [`byteorder`](https://github.com/BurntSushi/byteorder) library
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

/// Luminosity enum.
pub enum Luminosity {
    /// Variant for using [`average_luminosity`](../hdrimage/struct.HdrImage.html#method.average_luminosity)
    /// inside [`normalize_image`](../hdrimage/struct.HdrImage.html#method.normalize_image) method
    AverageLuminosity,
    /// Variant for setting a constant [`f32`] value for luminosity inside
    /// [`normalize_image`](../hdrimage/struct.HdrImage.html#method.normalize_image) method.
    FloatValue(f32),
}

/// Adjust the color levels of the brightest pixels in the image.
fn clamp(x: f32) -> f32 {
    x / (1.0 + x)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::color::WHITE;
    use crate::misc::IsClose;
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
        let color = Color::default();

        assert!(matches!(HdrImage::new(3, 3).get_pixel(0, 0), Ok(col) if col == color));
        assert!(matches!(HdrImage::new(3, 3).get_pixel(3, 3),
            Err(HdrImageErr::OutOfBounds(a, b)) if a == (3, 3) && b == (3, 3)))
    }

    #[test]
    fn test_set_pixels() {
        let color1 = Color::from((1.0, 1.0, 1.0));
        let color2 = Color::from((1.23, 4.56, 7.89));

        let mut hdr_img = HdrImage::new(3, 3);
        assert!(matches!(hdr_img.set_pixel(0, 0, color1), Ok(())));
        assert!(matches!(hdr_img.set_pixel(2, 2, color2), Ok(())));
        assert!(matches!(hdr_img.get_pixel(0, 0), Ok(col1) if col1 == color1));
        assert!(hdr_img.get_pixel(2, 2).unwrap().is_close(color2));
        assert!(matches!(hdr_img.set_pixel(5, 5, color1),
            Err(HdrImageErr::OutOfBounds(a, b)) if a == (5, 5) && b == (3, 3)));
        assert!(matches!(hdr_img.set_pixels(vec![WHITE; 9]), Ok(())));
        assert!(hdr_img.get_pixel(2, 2).unwrap().is_close(WHITE));
        assert!(matches!(
            hdr_img.set_pixels(vec![Color::default(); 11]),
            Err(HdrImageErr::InvalidPixelsSize(a, b)) if a == 11 && b == 9))
    }

    #[test]
    fn test_parse_img_shape() {
        let mut line = "10 20";

        assert!(matches!(parse_img_shape(line), Ok((10, 20))));
        line = " 10    20  ";
        assert!(matches!(parse_img_shape(line), Ok((10, 20))));
        line = "10 20 30";
        assert!(matches!(
            parse_img_shape(line),
            Err(HdrImageErr::InvalidPfmFileFormat(_))
        ));
        line = "10 ";
        assert!(matches!(
            parse_img_shape(line),
            Err(HdrImageErr::InvalidPfmFileFormat(_))
        ));
        line = "102030";
        assert!(matches!(
            parse_img_shape(line),
            Err(HdrImageErr::InvalidPfmFileFormat(_))
        ));
        line = "10 20.1";
        assert!(matches!(
            parse_img_shape(line),
            Err(HdrImageErr::PfmIntParseFailure(_, msg)) if msg.as_str() == "image height"
        ));
        line = "-10 20";
        assert!(matches!(
            parse_img_shape(line),
            Err(HdrImageErr::PfmIntParseFailure(_, msg)) if msg.as_str() == "image width"
        ));
        line = "abc 20";
        assert!(matches!(
            parse_img_shape(line),
            Err(HdrImageErr::PfmIntParseFailure(_, msg)) if msg.as_str() == "image width"
        ))
    }

    #[test]
    fn test_parse_endianness() {
        let mut line = "-3.2";

        assert!(matches!(
            parse_endianness(line),
            Ok(ByteOrder::LittleEndian)
        ));
        line = "1e15";
        assert!(matches!(parse_endianness(line), Ok(ByteOrder::BigEndian)));
        line = "0";
        assert!(matches!(
            parse_endianness(line),
            Err(HdrImageErr::InvalidPfmFileFormat(_))
        ));
        line = "abc";
        assert!(matches!(
            parse_endianness(line),
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
            hdr_img.write_pfm_file(reference_file_be, ByteOrder::BigEndian),
            Ok(())
        ));
        assert!(matches!(
            hdr_img.write_pfm_file(reference_file_le, ByteOrder::LittleEndian),
            Ok(())
        ));
        assert!(matches!(
            hdr_img.write_pfm_file(invalid_path, ByteOrder::LittleEndian),
            Err(HdrImageErr::PfmFileWriteFailure(_))
        ));

        let hdr_img_result = HdrImage::read_pfm_file(reference_file_be);
        assert!(matches!(hdr_img_result, Ok(ref img) if img == &hdr_img));
        assert_eq!(hdr_img_result.unwrap(), hdr_img);

        let hdr_img_result = HdrImage::read_pfm_file(reference_file_le);
        assert!(matches!(hdr_img_result, Ok(ref img) if img == &hdr_img));
        assert_eq!(hdr_img_result.unwrap(), hdr_img);

        let hdr_img_result = HdrImage::read_pfm_file(invalid_file);
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
                assert!((0.0..1.0).contains(&el));
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
        hdr_img.normalize_image(1000.0, Luminosity::AverageLuminosity);
        hdr_img.clamp_image();

        let invalid_format = Path::new("/tmp/reference_le.mkv");
        let unsupported_format = Path::new("/tmp/reference_le.jpeg");
        let invalid_path = Path::new("/invalid_path/reference_le.png");
        let reference_png = Path::new("/tmp/reference_le.png");
        let reference_ff = Path::new("/tmp/reference_le.ff");

        assert!(matches!(
            hdr_img.write_ldr_file(invalid_format, 1.0),
            Err(HdrImageErr::LdrFileWriteFailure(_))
        ));
        assert!(matches!(
            hdr_img.write_ldr_file(unsupported_format, 1.0),
            Err(HdrImageErr::UnsupportedLdrFileFormat(format)) if format.as_str() == "jpeg"
        ));
        assert!(matches!(
            hdr_img.write_ldr_file(invalid_path, 1.0),
            Err(HdrImageErr::LdrFileWriteFailure(_))
        ));
        assert!(matches!(hdr_img.write_ldr_file(reference_png, 1.0), Ok(())));
        assert!(matches!(hdr_img.write_ldr_file(reference_ff, 1.0), Ok(())))
    }
}
