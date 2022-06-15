//! Error reporting module.
//!
//! Provides internal [`rustracer`](..) errors,
//! using [`thiserror`](https://github.com/dtolnay/thiserror) library.

use crate::scene::SourceLocation;
use thiserror::Error;

/// Error enum for [`HdrImage`](../hdrimage) module.
#[derive(Error, Debug)]
pub enum HdrImageErr {
    #[error("invalid indexes: {0:?}, expected hdr image shape: {1:?}")]
    OutOfBounds((u32, u32), (u32, u32)),
    #[error("invalid pixels vector size: {0}, expected size: {1}")]
    InvalidPixelsSize(u32, u32),
    #[error("invalid pfm file format: {0}")]
    InvalidPfmFileFormat(String),
    #[error("impossible to read from pfm file: {0}")]
    PfmFileReadFailure(#[source] std::io::Error),
    #[error("impossible to write to pfm file: {0}")]
    PfmFileWriteFailure(#[source] std::io::Error),
    #[error("impossible to parse {1} as integer from pfm file: {0}")]
    PfmIntParseFailure(#[source] std::num::ParseIntError, String),
    #[error("impossible to parse {1} as float from pfm file: {0}")]
    PfmFloatParseFailure(#[source] std::num::ParseFloatError, String),
    #[error("impossible to write to ldr file: {0}")]
    LdrFileWriteFailure(#[source] image::ImageError),
    #[error("unsupported {0} ldr file format, only ff or png supported")]
    UnsupportedLdrFileFormat(String),
}

/// Error enum for [`convert`](../fn.convert.html) function inside [`main`](../fn.main.html).
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

/// Error enum for [`demo`](../fn.demo.html) function inside [`main`](../fn.main.html).
#[derive(Error, Debug)]
pub enum DemoErr {
    #[error("invalid {1}, expected integer number: {0}")]
    IntParseFailure(#[source] std::num::ParseIntError, String),
    #[error("invalid {1}, expected floating-point number: {0}")]
    FloatParseFailure(#[source] std::num::ParseFloatError, String),
    #[error("{0}")]
    IoError(#[source] HdrImageErr),
}

/// Error enum for [`Scene`](../scene) module.
#[derive(Error, Debug)]
pub enum SceneErr {
    #[error("<{}:{}> {}", .loc.line_num, .loc.col_num, msg)]
    InvalidCharacter { loc: SourceLocation, msg: String },
    #[error("<{}:{}> {}", .loc.line_num, .loc.col_num, msg)]
    UnclosedString { loc: SourceLocation, msg: String },
    #[error("<{}:{}> {}", .loc.line_num, .loc.col_num, msg)]
    FloatParseFailure {
        loc: SourceLocation,
        msg: String,
        src: std::num::ParseFloatError,
    },
    #[error("<{}:{}> {}", .loc.line_num, .loc.col_num, msg)]
    NotMatch { loc: SourceLocation, msg: String },
    #[error("<{}:{}> {}", .loc.line_num, .loc.col_num, msg)]
    PfmFileReadFailure {
        loc: SourceLocation,
        msg: String,
        src: HdrImageErr,
    },
    #[error("<{}:{}> {}", .loc.line_num, .loc.col_num, msg)]
    MaxSpaces { loc: SourceLocation, msg: String },
    #[error("{0}")]
    UnexpectedMatch(String),
    #[error("<{}:{}> {}", .loc.line_num, .loc.col_num, msg)]
    UndefinedIdentifier { loc: SourceLocation, msg: String },
    #[error("<{}:{}> {}", .loc.line_num, .loc.col_num, msg)]
    InvalidCamera { loc: SourceLocation, msg: String },
    #[error("impossible to read from scene file: {0}")]
    SceneFileReadFailure(#[source] std::io::Error),
}

/// Error enum for [`render`](../fn.render.html) function inside [`main`](../fn.main.html).
#[derive(Error, Debug)]
pub enum RenderErr {
    #[error("invalid {1}, expected integer number: {0}")]
    IntParseFailure(#[source] std::num::ParseIntError, String),
    #[error("invalid {1}, expected floating-point number: {0}")]
    FloatParseFailure(#[source] std::num::ParseFloatError, String),
    #[error("{0}")]
    IoError(#[source] HdrImageErr),
    #[error("({1}) {0}")]
    SceneError(#[source] SceneErr, String),
}
