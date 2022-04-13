//! Error reporting module.
//!
//! Provides internal [`rustracer`](..) errors,
//! using [`thiserror`](https://github.com/dtolnay/thiserror) library

use thiserror::Error;

/// Error enum for [`HdrImage`](../hdrimage) module.
#[derive(Error, Debug)]
pub enum HdrImageErr {
    #[error("invalid indexes: {0:?}, expected hdr image shape: {1:?}")]
    OutOfBounds((u32, u32), (u32, u32)),
    #[error("invalid pfm file format: {0}")]
    InvalidPfmFileFormat(String),
    #[error("impossible to read from pfm file: {0}")]
    PfmFileReadFailure(#[source] std::io::Error),
    #[error("impossible to parse {1} as integer from pfm file: {0}")]
    PfmIntParseFailure(#[source] std::num::ParseIntError, String),
    #[error("impossible to parse {1} as float from pfm file: {0}")]
    PfmFloatParseFailure(#[source] std::num::ParseFloatError, String),
    #[error("impossible to write to ldr file: {0}")]
    LdrFileWriteFailure(#[source] image::ImageError),
    #[error("unsupported {0} ldr file format, only ff or png supported")]
    UnsupportedLdrFileFormat(String),
}

/// Error enum for [`convert`](../fn.convert.html) function inside [`main`](../fn.main).
#[derive(Error, Debug)]
pub enum ConvertErr {
    #[error("invalid {1}, expected floating-point number: {0}")]
    FloatParseFailure(#[source] std::num::ParseFloatError, String),
    #[error("{0}")]
    IoError(#[source] HdrImageErr),
}

#[derive(Error, Debug)]
pub enum GeometryErr {
    #[error("object with norm {0} can't be normalized")]
    UnableToNormalize(f32),
}
