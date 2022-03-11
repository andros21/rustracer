use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum HdrImageErr {
    #[error("(HdrImage) invalid indexes: {0:?}, hdr image shape: {1:?}")]
    OutOfBounds((usize, usize), (usize, usize)),
}
