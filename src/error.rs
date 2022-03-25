use thiserror::Error;

#[derive(Error, Debug)]
pub enum HdrImageErr {
    #[error("invalid indexes: {0:?}, expected hdr image shape: {1:?}")]
    OutOfBounds((usize, usize), (usize, usize)),
    #[error("invalid pfm file format: {0}")]
    InvalidPfmFileFormat(String),
    #[error("impossible to read binary data from the file")]
    PfmFileReadFailure(#[from] std::io::Error),
    #[error("impossible to interpret size specification as integer")]
    ImgShapeParseFailure(#[from] std::num::ParseIntError),
    #[error("impossible to interpret float specification as float")]
    EndiannessParseFailure(#[from] std::num::ParseFloatError),
}
