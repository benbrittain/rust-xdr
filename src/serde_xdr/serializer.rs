use std::fmt;
use std::error;
use std::result;
use std::io;
use serde::ser;
use serde::ser::Serialize;
use byteorder::{BigEndian, WriteBytesExt}; // This is unfair for the VAX

use error::{EncoderResult, EncoderError};
use super::to_bytes;

macro_rules! not_implemented {
    ($($name:ident($($arg:ident: $ty:ty,)*);)*) => {
        $(fn $name<>(&mut self, $($arg: $ty,)*) -> EncoderResult<()> {
            Err(EncoderError::Unknown(format!("Deserialize Not Implemented for {}", stringify!($name))))
        })*
    }
}

pub struct Serializer<W> {
    writer: W,
}

impl<W: io::Write> Serializer<W> {
    pub fn new(writer: W) -> Self {
        Serializer {
            writer: writer,
        }
    }

    pub fn into_inner(self) -> W {
        self.writer
    }

}

#[derive(Debug)]
pub struct MapState {
    size: usize,
    slots: Vec<u8>
}

impl<W: io::Write> ser::Serializer for Serializer<W> {
    type Error = EncoderError;
    // TODO These are all wrong
    type SeqState = Option<usize>;
    type MapState = MapState;
    type StructState = Self::MapState;
    type StructVariantState = Self::MapState;
    type TupleState = bool;
    type TupleStructState = bool;
    type TupleVariantState = ();

    fn serialize_i8(&mut self, value: i8) -> EncoderResult<()> {
        self.writer.write_i8(value).map_err(From::from)
	}

    fn serialize_i16(&mut self, value: i16) -> EncoderResult<()> {
        self.writer.write_i16::<BigEndian>(value).map_err(From::from)
	}

    fn serialize_i32(&mut self, value: i32) -> EncoderResult<()> {
        self.writer.write_i32::<BigEndian>(value).map_err(From::from)
	}

    fn serialize_i64(&mut self, value: i64) -> EncoderResult<()> {
        self.writer.write_i64::<BigEndian>(value).map_err(From::from)
	}

    fn serialize_u8(&mut self, value: u8) -> EncoderResult<()> {
        self.writer.write_u8(value).map_err(From::from)
	}

    fn serialize_u16(&mut self, value: u16) -> EncoderResult<()> {
        self.writer.write_u16::<BigEndian>(value).map_err(From::from)
	}

    fn serialize_u32(&mut self, value: u32) -> EncoderResult<()> {
        self.writer.write_u32::<BigEndian>(value).map_err(From::from)
	}

    fn serialize_u64(&mut self, value: u64) -> EncoderResult<()> {
        self.writer.write_u64::<BigEndian>(value).map_err(From::from)
	}

    not_implemented!(
        serialize_f32(val: f32,);
        serialize_f64(val: f64,);
        serialize_char(val: char,);
        serialize_str(val: &str,);
        serialize_none();
        serialize_usize(val: usize,);
        serialize_bytes(val: &[u8],);
        serialize_isize(val: isize,);
        serialize_bool(val: bool,);
        serialize_tuple_variant(_name: &str, _variant_index: usize, variant: &str, _len: usize,);
        serialize_tuple_variant_end(_state: (),);
        serialize_unit_struct(_name: &'static str,);
        serialize_tuple_end(name: bool,);
        serialize_struct_variant_end(state: MapState,);
        serialize_tuple_struct_end(state: bool,);
        serialize_map_end(state: MapState,);
    );

    #[inline]
    fn serialize_unit(&mut self) -> EncoderResult<()> {
        Ok(())
	}

    #[inline]
    fn serialize_some<V>(&mut self, value: V) -> EncoderResult<()> where V: Serialize {
        Err(EncoderError::Unknown(String::from("Not Implemented Some")))
    }


    fn serialize_newtype_struct<T>(&mut self, _name: &'static str, value: T)
                                   -> EncoderResult<()> where T: Serialize {
        Err(EncoderError::Unknown(String::from("Newtype struct")))
    }

    fn serialize_struct(&mut self, name: &'static str, len: usize) -> EncoderResult<Self::MapState> {
        Ok(MapState {
            size: len,
            slots: Vec::new(),
        })
    }

    fn serialize_struct_elt<T: Serialize>(&mut self, state: &mut Self::MapState,
                                          key: &'static str, value: T) -> EncoderResult<()> {
        // keep state around in case we need to do something fancy
        let mut buf = Vec::<u8>::with_capacity(128);
        try!(to_bytes(&value, &mut buf));
        state.slots.append(&mut buf);
        value.serialize(self)
    }

    fn serialize_struct_end(&mut self, state: Self::MapState) -> EncoderResult<()> {
        Ok(())
    }

    #[inline]
    fn serialize_unit_variant(&mut self, _name: &str, variant_index: usize, variant: &str) -> EncoderResult<()> {
        self.serialize_i32(variant_index as i32)
    }

    #[inline]
    fn serialize_newtype_variant<T>(&mut self, _name: &str, _variant_index: usize, variant: &str,
                                    value: T) -> EncoderResult<()> where T: Serialize {
        Err(EncoderError::Unknown(String::from("Not Implemented Newtype")))
    }

    #[inline]
    fn serialize_tuple_variant_elt<T: Serialize>(&mut self, _state: &mut (), value: T) -> EncoderResult<()> {
        Err(EncoderError::Unknown(String::from("Not Implemented")))
    }

    #[inline]
    fn serialize_struct_variant(&mut self, _name: &str, _variant_index: usize, variant: &str,
                                len: usize) -> EncoderResult<MapState> {
        Err(EncoderError::Unknown(String::from("Not Implemented")))
    }

    #[inline]
    fn serialize_struct_variant_elt<T: Serialize>(&mut self, state: &mut MapState,
                                                       key: &'static str, value: T) -> EncoderResult<()> {
        let mut buf = Vec::<u8>::with_capacity(128);
        try!(to_bytes(&value, &mut buf));
        Ok(state.slots.append(&mut buf))
    }


    #[inline]
    fn serialize_tuple_struct(&mut self, _name: &'static str, len: usize) -> EncoderResult<bool> {
        Err(EncoderError::Unknown(String::from("Not Implemented")))
    }

    #[inline]
    fn serialize_tuple_struct_elt<T: Serialize>(&mut self, state: &mut bool, value: T) -> EncoderResult<()> {
        Err(EncoderError::Unknown(String::from("Not Implemented")))
    }

    fn serialize_tuple(&mut self, len: usize,) -> EncoderResult<bool> {
        Err(EncoderError::Unknown(String::from("Not Implemented Tuple End")))
    }

    #[inline]
    fn serialize_tuple_elt<T: Serialize>(&mut self, _state: &mut bool, value: T) -> EncoderResult<()> {
        Err(EncoderError::Unknown(String::from("Not Implemented Tuple Elt")))
    }

    #[inline]
    fn serialize_seq(&mut self, len: Option<usize>) -> EncoderResult<Option<usize>> {
        self.serialize_u32((len.unwrap() as u32));
        Ok(len)
    }

    #[inline]
    fn serialize_seq_elt<T>(&mut self, state: &mut Option<usize>,
                            value: T) -> EncoderResult<()> where T: Serialize {
        value.serialize(self);
        let mut len = state.unwrap();
        if len > 0 {
            match state.iter_mut().next() { // TODO there is probably an easier way to grab a mut ref to an option
                Some(v) => *v = len - 1,
                None => {},
            }
            Ok(())
        } else {
            Err(EncoderError::Unknown(String::from("Sequence Serializer ran out of items!")))
        }
    }

    #[inline]
    fn serialize_seq_end(&mut self, state: Option<usize>) -> EncoderResult<()> {
        let len = state.unwrap();
        if len != 0 {
            Err(EncoderError::Unknown(String::from("Expected an end for the sequence")))
        } else {
          Ok(())
        }
    }

    #[inline]
    fn serialize_seq_fixed_size(&mut self, _len: usize) -> EncoderResult<Option<usize>> {
        Err(EncoderError::Unknown(String::from("Not Implemented")))
    }

    #[inline]
    fn serialize_map(&mut self, len: Option<usize>) -> EncoderResult<MapState> {
        Err(EncoderError::Unknown(String::from("Not Implemented")))
    }

    #[inline]
    fn serialize_map_key<T: Serialize>(&mut self, _state: &mut MapState, key: T) -> EncoderResult<()> {
        Err(EncoderError::Unknown(String::from("Not Implemented")))
    }

    #[inline]
    fn serialize_map_value<T: Serialize>(&mut self, state: &mut MapState, value: T) -> EncoderResult<()> {
        Err(EncoderError::Unknown(String::from("Not Implemented")))
    }

}
