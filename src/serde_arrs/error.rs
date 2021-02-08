use serde::{de, ser};
use std::{fmt, io, result};

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Message(String),
    MismatchTypes { expected: u8, received: u8 },
    UnexpectedEOF,
    TrailingBytes,
    NotImplemented { method: &'static str },
    Io(io::Error),
}

impl ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Error::Message(msg.to_string())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Message(msg) => msg.to_owned(),
                Self::MismatchTypes { expected, received } => {
                    format!(
                        "Expected dtype id: {}, found dtype id: {}",
                        expected, received
                    )
                }
                Self::UnexpectedEOF => "File ended unexpectedly".to_owned(),
                Self::TrailingBytes => "File has trailing bytes".to_owned(),
                Self::NotImplemented { method } => method.to_string(),
                Self::Io(err) => format!("{}", err),
            }
        )
    }
}

impl std::error::Error for Error {}
