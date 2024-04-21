use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum Error {
    AddressNotFound,
    InvalidOffset,
    InvalidDataType(u8),
    InvalidRecordSize(usize),
    InvalidSearchTreeSize,
    InvalidNode,
    MetadataNotFound,
    CorruptSearchTree,
    Open(std::io::Error),
    UnknownField(String),

    #[cfg(not(feature = "unsafe-str"))]
    InvalidUtf8(std::str::Utf8Error),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        // clean up and clean up Error generally
        Error::Open(err)
    }
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Error::AddressNotFound => fmt.write_str("address not found in database")?,
            Error::InvalidOffset => fmt.write_str("invalid offset")?,
            Error::InvalidDataType(_typ) => fmt.write_str("invalid data type")?,
            Error::InvalidRecordSize(size) => write!(fmt, "invalid record size {size}")?,
            Error::InvalidSearchTreeSize => fmt.write_str("invalid search tree size")?,
            Error::InvalidNode => fmt.write_str("invalid node")?,
            Error::MetadataNotFound => fmt.write_str("metadata is not found")?,
            Error::CorruptSearchTree => fmt.write_str("search tree is corrupt")?,
            Error::Open(err) => write!(fmt, "open file failed, {err}")?,
            Error::UnknownField(field) => write!(fmt, "unknown field {field}")?,
            #[cfg(not(feature = "unsafe-str"))]
            Error::InvalidUtf8(err) => Display::fmt(err, fmt)?,
        }

        Ok(())
    }
}

impl std::error::Error for Error {}
