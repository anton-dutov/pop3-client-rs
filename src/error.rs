use thiserror::Error;

#[derive(Error, Debug)]
pub enum Pop3Error {
    #[error("Stream connection closed")]
    ConnectionClosed,

    #[error("Already authenticated")]
    AlreadyAuthenticated,

    #[error("IO: {0}")]
    Io(#[from] std::io::Error),


    #[error("Number parsing error: {0}")]
    InvalidNumber(std::num::ParseIntError),

    #[error("String parsing error: {0}")]
    InvalidString(std::str::Utf8Error),

    #[error("Invalid response")]
    InvalidResponse,

    #[error("Other error: {0}")]
    OtherString(String),

    // #[error("invalid header (expected {expected:?}, found {found:?})")]
    // InvalidHeader {
    //     expected: String,
    //     found: String,
    // },
    // #[error("unknown data store error")]
    // Unknown,
}

impl Pop3Error {
    pub fn other<E: AsRef<str>>(err: E) -> Self {
        Self::OtherString(err.as_ref().to_string())
    }
}