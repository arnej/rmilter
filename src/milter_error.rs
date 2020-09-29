use std::error::Error;
use std::fmt::{Display, Formatter};

/// Errors defined in the `rmilter` crate
#[derive(Debug)]
pub enum MilterError {
    /// An incomplete message was received by rmilter (e.g. missing non-optional fields)
    IncompleteMessage,
    /// An `std::io::Error` occured
    IoError(std::io::Error),
    /// A message was received by rmilter that doesn't contain a message identifier
    MissingMessageIdentifier,
    /// An `std::num::TryFromIntError` occured
    TryFromIntError(std::num::TryFromIntError),
    /// An `std::num::TryFromSliceError` occured
    TryFromSliceError(std::array::TryFromSliceError),
    /// A message with an unknown message identifier was received by rmilter
    UnknowMessageIdentifier(char),
}

impl Display for MilterError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MilterError::IncompleteMessage => write!(f, "Incomplete message"),
            MilterError::IoError(e) => e.fmt(f),
            MilterError::MissingMessageIdentifier => write!(f, "Missing message identifier"),
            MilterError::TryFromIntError(e) => e.fmt(f),
            MilterError::TryFromSliceError(e) => e.fmt(f),
            MilterError::UnknowMessageIdentifier(c) => {
                write!(f, "Unknown message identifier: '{}'", c)
            }
        }
    }
}

impl Error for MilterError {}

impl From<std::io::Error> for MilterError {
    fn from(e: std::io::Error) -> MilterError {
        MilterError::IoError(e)
    }
}

impl From<std::num::TryFromIntError> for MilterError {
    fn from(e: std::num::TryFromIntError) -> MilterError {
        MilterError::TryFromIntError(e)
    }
}

impl From<std::array::TryFromSliceError> for MilterError {
    fn from(e: std::array::TryFromSliceError) -> MilterError {
        MilterError::TryFromSliceError(e)
    }
}
