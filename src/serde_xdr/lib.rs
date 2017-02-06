#![cfg_attr(test, feature(custom_attribute, custom_derive, plugin))]

extern crate serde;
extern crate byteorder;

pub mod serializer;
pub mod deserializer;
pub mod error;

use std::io::{self, Read};
use serde::{Serialize, Deserialize};
pub use self::error::{EncoderError, DecoderResult, EncoderResult};

pub use self::serializer::Serializer;
pub use self::deserializer::Deserializer;

pub fn to_bytes<T>(value: &T, buf: &mut Vec<u8>) -> EncoderResult<()>
    where T: Serialize
{
    let mut ser = Serializer::new(buf);
    try!(value.serialize(&mut ser));
    Ok(())
}

pub fn from_reader<T: Deserialize, R: Read>(reader: R) -> DecoderResult<(T, usize)> {
    let mut de = Deserializer::new(reader);
    let value = try!(Deserialize::deserialize(&mut de));
    Ok((value, de.get_bytes_consumed()))
}

pub fn from_bytes<T: Deserialize>(v: &[u8]) -> DecoderResult<(T, usize)> {
    from_reader(v)
}

#[macro_export]
macro_rules! xdr_enum {
    ($name:ident { $($variant:ident = $value:expr, )* }) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum $name {
            $($variant = $value,)*
        }

        impl ::serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: ::serde::Serializer {
                serializer.serialize_i32(*self as i32) // All Enums are signed ints in XDR
            }
        }

        impl ::serde::Deserialize for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: ::serde::Deserializer {

                struct Visitor;

                impl ::serde::de::Visitor for Visitor {
                    type Value = $name;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("i32")
                    }

                    fn visit_i32<E>(self, value: i32) -> Result<$name, E> where E: ::serde::de::Error {
                        match value {
                            $( $value => Ok($name::$variant), )*
                            _ => Err(E::custom(
                                format!("unknown {} value: {}",
                                stringify!($name), value))),
                        }
                    }
                }
                deserializer.deserialize_i32(Visitor)
            }
        }
    }
}
