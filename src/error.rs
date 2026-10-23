use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum AppKitError {
    InvalidArgument(String),
    InvalidImageData,
}

impl fmt::Display for AppKitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidArgument(message) => write!(f, "invalid argument: {message}"),
            Self::InvalidImageData => f.write_str("AppKit cannot read the data as an image"),
        }
    }
}

impl std::error::Error for AppKitError {}
