use thiserror::Error;

#[derive(Error, Debug)]
pub enum HdrImageErr {
    #[error("invalid indexes: {0:?}, expected hdr image shape: {1:?}")]
    OutOfBounds((usize, usize), (usize, usize)),
    #[error("invalid pfm file format: {0}")]
    InvalidPfmFileFormat(String),
    #[error("impossible to read binary data from the file")]
    InvalidPfmFileParse(#[from] std::io::Error),
}
