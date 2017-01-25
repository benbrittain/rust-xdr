use std::{io, error};
use std::fmt::{self, Debug, Display};
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

impl Into<io::Error> for EncoderError {
    fn into(self) -> io::Error {
        match self {
            EncoderError::Io(e) => {
                e
            },
            EncoderError::Unknown(e) => {
                io::Error::new(io::ErrorKind::Other, e)
            }
        }
    }
}

//impl fmt::Display for EncoderError {
//    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
//        match self {
//            &EncoderError::Io(ref inner) => inner.fmt(fmt),
//            &EncoderError::Unknown(ref inner) => inner.fmt(fmt),
//        }
//    }
//}

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
    fn custom<T: Display>(msg: T) -> EncoderError {
        EncoderError::Unknown(msg.to_string())
    }
}

impl de::Error for EncoderError {
    fn custom<T: Display>(msg: T) -> EncoderError {
        EncoderError::Unknown(msg.to_string())
    }
}

impl Display for EncoderError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &EncoderError::Unknown(ref s) => {
                    write!(fmt, "{}", s)
            }
            &EncoderError::Io(ref error) => fmt::Display::fmt(error, fmt),
        }
    }
}

    //fn end_of_stream() -> EncoderError {
    //    EncoderError::Unknown(String::from("End of File!"))
    //}
//}

pub type EncoderResult<T> = Result<T, EncoderError>;
pub type DecoderResult<T> = Result<T, EncoderError>;
