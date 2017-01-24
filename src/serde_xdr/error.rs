use std::{io, error, fmt};
use serde::ser;
use serde::de;

#[derive(Debug)]
pub enum EncoderError {
    Io(io::Error),
    Unknown(String),
}

impl From<io::Error> for EncoderError {
    fn from(err: io::Error) -> EncoderError {
        EncoderError::Io(err)
    }
}

impl fmt::Display for EncoderError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &EncoderError::Io(ref inner) => inner.fmt(fmt),
            &EncoderError::Unknown(ref inner) => inner.fmt(fmt),
        }
    }
}

impl error::Error for EncoderError {
    fn description(&self) -> &str {
        match self {
            &EncoderError::Io(ref inner) => inner.description(),
            &EncoderError::Unknown(ref inner) => inner,
        }
    }
    fn cause(&self) -> Option<&error::Error> {
        match self {
            &EncoderError::Io(ref inner) => Some(inner),
            _ => None,
        }
    }
}

impl ser::Error for EncoderError {
    fn custom<T: Into<String>>(msg: T) -> EncoderError {
        EncoderError::Unknown(msg.into())
    }
}

impl de::Error for EncoderError {
    fn custom<T: Into<String>>(msg: T) -> EncoderError {
        EncoderError::Unknown(msg.into())
    }

    fn end_of_stream() -> EncoderError {
        EncoderError::Unknown(String::from("End of File!"))
    }
}

pub type EncoderResult<T> = Result<T, EncoderError>;
pub type DecoderResult<T> = Result<T, EncoderError>;
