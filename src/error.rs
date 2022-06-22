//! Error reporting module.
//!
//! Provides internal [`rustracer`](..) errors,
//! using [`thiserror`](https://github.com/dtolnay/thiserror) library.

use crate::scene::SourceLocation;
use colored::Colorize;
use thiserror::Error;

/// Error enum for [`HdrImage`](../hdrimage) module.
#[derive(Error, Debug)]
pub enum HdrImageErr {
    #[error("invalid indexes: {0:?}, expected hdr image shape: {1:?}")]
    OutOfBounds((u32, u32), (u32, u32)),
    #[error("invalid pixels vector size: {0}, expected size: {1}")]
    InvalidPixelsSize(u32, u32),
    #[error("invalid pfm file format: {0:?}")]
    InvalidPfmFileFormat(String),
    #[error("impossible to read from pfm file\n\tsource: {}",
        format!("{}", .0).to_lowercase())]
    PfmFileReadFailure(#[source] std::io::Error),
    #[error("impossible to write to pfm file\n\tsource: {}",
        format!("{}", .0).to_lowercase())]
    PfmFileWriteFailure(#[source] std::io::Error),
    #[error("impossible to parse {1} as integer from pfm file\n\tsource: {}",
        format!("{}", .0).to_lowercase())]
    PfmIntParseFailure(#[source] std::num::ParseIntError, String),
    #[error("impossible to parse {1} as float from pfm file\n\tsource: {}",
        format!("{}", .0).to_lowercase())]
    PfmFloatParseFailure(#[source] std::num::ParseFloatError, String),
    #[error("impossible to write to ldr file\n\tsource: {}",
        format!("{}", .0).to_lowercase())]
    LdrFileWriteFailure(#[source] image::ImageError),
    #[error("unsupported {0:?} ldr file format, only \"ff\" or \"png\" supported")]
    UnsupportedLdrFileFormat(String),
}

/// Error enum for [`convert`](../fn.convert.html) function inside [`main`](../fn.main.html).
#[derive(Error, Debug)]
pub enum ConvertErr {
    #[error("{}\n\tsource: {0}",
        format!("{:?} flag invalid value, expected floating-point number", .1).bold())]
    FloatParseFailure(#[source] std::num::ParseFloatError, String),
    #[error("{}\n\tsource: {0}", "image convert input/output error".bold())]
    IoError(#[source] HdrImageErr),
}

/// Error enum for [`demo`](../fn.demo.html) function inside [`main`](../fn.main.html).
#[derive(Error, Debug)]
pub enum DemoErr {
    #[error("{}\n\tsource: {0}",
        format!("{:?} flag invalid value, expected integer number", .1).bold())]
    IntParseFailure(#[source] std::num::ParseIntError, String),
    #[error("{}\n\tsource: {0}",
        format!("{:?} flag invalid value, expected floating-point number", .1).bold())]
    FloatParseFailure(#[source] std::num::ParseFloatError, String),
    #[error("{}\n\tsource: {0}", "demo render input/output error".bold())]
    IoError(#[source] HdrImageErr),
}

/// Error enum for [`Scene`](../scene) module.
#[derive(Error, Debug)]
pub enum SceneErr {
    #[error("{} {}",
        format!(":{}:{}", loc.line_num, loc.col_num).yellow(), msg)]
    InvalidCharacter { loc: SourceLocation, msg: String },
    #[error("{} {}",
        format!(":{}:{}", loc.line_num, loc.col_num).yellow(), msg)]
    UnclosedString { loc: SourceLocation, msg: String },
    #[error("{} {}",
        format!(":{}:{}", loc.line_num, loc.col_num).yellow(), msg)]
    FloatParseFailure {
        loc: SourceLocation,
        msg: String,
        src: std::num::ParseFloatError,
    },
    #[error("{} {}",
        format!(":{}:{}", loc.line_num, loc.col_num).yellow(), msg)]
    NotMatch { loc: SourceLocation, msg: String },
    #[error("{} {}\n\tsource: {}",
        format!(":{}:{}", loc.line_num, loc.col_num).yellow(), msg, src)]
    PfmFileReadFailure {
        loc: SourceLocation,
        msg: String,
        src: HdrImageErr,
    },
    #[error("{0}")]
    UnexpectedMatch(String),
    #[error("{} {}",
        format!(":{}:{}", loc.line_num, loc.col_num).yellow(), msg)]
    UndefinedIdentifier { loc: SourceLocation, msg: String },
    #[error("{} {}",
        format!(":{}:{}", loc.line_num, loc.col_num).yellow(), msg)]
    InvalidCamera { loc: SourceLocation, msg: String },
    #[error("{} impossible to read from scene file\n\tsource: {0}", "::".yellow())]
    SceneFileReadFailure(#[source] std::io::Error),
}

/// Error enum for [`render`](../fn.render.html) function inside [`main`](../fn.main.html).
#[derive(Error, Debug)]
pub enum RenderErr {
    #[error("{}\n\tsource: {0}",
        format!("{:?} flag invalid value, expected integer number", .1).bold())]
    IntParseFailure(#[source] std::num::ParseIntError, String),
    #[error("{}\n\tsource: {0}",
        format!("{:?} flag invalid value, expected floating-point number", .1).bold())]
    FloatParseFailure(#[source] std::num::ParseFloatError, String),
    #[error("{}\n\tsource: {0}", "render input/output error".bold())]
    IoError(#[source] HdrImageErr),
    #[error("{}\n\tsource: {}",
        format!("render parsing scene from {:?}", .1).bold(),
        format!("{file}{src}",
            file = std::path::Path::new(.1).file_name()
                                           .unwrap_or_else(|| std::ffi::OsStr::new(.1))
                                           .to_str().unwrap().yellow(),
            src = .0))]
    SceneError(#[source] SceneErr, String),
}

/// Error enum for [`completion`](../fn.completion.html) function inside [`main`](../fn.main.html).
#[derive(Error, Debug)]
pub enum CompletionErr {
    #[error("impossible to create {1:?}: {0}")]
    WriteCompletionFailure(#[source] std::io::Error, String),
}
