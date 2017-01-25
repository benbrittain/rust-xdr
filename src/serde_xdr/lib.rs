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
//  Ok(ser.into_inner())
    Ok(())
}

pub fn from_reader<T: Deserialize, R: Read>(reader: R) -> DecoderResult<(T, usize)> {
    let mut de = Deserializer::new(reader);
    let value = try!(Deserialize::deserialize(&mut de));
//    try!(de.end());
    Ok((value, de.get_bytes_consumed()))
}

pub fn from_bytes<T: Deserialize>(v: &[u8]) -> DecoderResult<(T, usize)> {
    from_reader(v)
}
