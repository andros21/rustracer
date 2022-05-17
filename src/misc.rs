//! Miscellanea module.
//!
//! Provides cross modules useful enums, functions, traits.

/// Default error tolerance used inside [`IsClose`] trait.
pub const EPSILON: f32 = 1e-4;

/// Trait for equivalence between two objects,
/// up to a certain error [`tolerance`](constant@EPSILON).
///
/// Primitive elements that compose those objects must
/// implement [`IsClose`] trait to derive this trait.
pub trait IsClose<Rhs = Self> {
    fn is_close(&self, other: Self) -> bool;
}

impl IsClose for f32 {
    /// Return `true` if absolute value between two `f32`
    /// is less than [`EPSILON`]
    fn is_close(&self, other: f32) -> bool {
        (self - other).abs() < EPSILON
    }
}

/// Variants of byte/bit endianness.
pub enum ByteOrder {
    BigEndian,
    LittleEndian,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vector2D {
    pub u: f32,
    pub v: f32,
}

impl IsClose for Vector2D {
    fn is_close(&self, other: Self) -> bool {
        self.u.is_close(other.u) && self.v.is_close(other.v)
    }
}

/// Macro for wrap exit logic inside [`main`](fn.main.html).
#[macro_export]
macro_rules! exit {
    ($a:expr) => {
        match $a {
            Ok(()) => exit(0),
            Err(e) => {
                eprintln!("[error] {:#}", e);
                exit(1)
            }
        }
    };
}

/// Macro for fail fast in [`main`](fn.main.html) subcommands
/// (e.g. inside [`convert`](fn.convert.html)).
///
/// When invalid or unsupported ldr file is provided via cli fail immediately.
///
/// Re-use of built-in logic inside [`write_ldr_file`](hdrimage/struct.HdrImage.html#method.write_ldr_file).
#[macro_export]
macro_rules! check {
    ($a:expr) => {
        match ImageFormat::from_path($a).map_err(HdrImageErr::LdrFileWriteFailure) {
            Ok(ImageFormat::Png) => Ok(()),
            Ok(ImageFormat::Farbfeld) => Ok(()),
            Ok(_) => Err(HdrImageErr::UnsupportedLdrFileFormat(String::from(
                $a.extension().unwrap().to_str().unwrap_or(""),
            ))),
            Err(err) => Err(err),
        }
    };
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_is_close_float() {
        assert!((EPSILON * 1e-1 + 1.0).is_close(1.0));
        assert!(!(EPSILON + 1.0).is_close(1.0))
    }
}
