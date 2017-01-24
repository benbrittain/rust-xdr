use std::fmt;
use std::error;
use std::result;
use std::io;
use serde::ser;
use serde::ser::Serialize;
use byteorder::{BigEndian, WriteBytesExt}; // This is unfair for the VAX

use error::{EncoderResult, EncoderError};
use super::to_bytes;


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

    #[inline]
    fn serialize_bool(&mut self, value: bool) -> EncoderResult<()> {
        Err(EncoderError::Unknown(String::from("Not Implemented")))
	}

    #[inline]
    fn serialize_isize(&mut self, value: isize) -> EncoderResult<()> {
        Err(EncoderError::Unknown(String::from("Not Implemented")))
	}

    #[inline]
    fn serialize_i8(&mut self, value: i8) -> EncoderResult<()> {
        self.writer.write_i8(value).map_err(From::from)
	}

    #[inline]
    fn serialize_i16(&mut self, value: i16) -> EncoderResult<()> {
        self.writer.write_i16::<BigEndian>(value).map_err(From::from)
	}

    #[inline]
    fn serialize_i32(&mut self, value: i32) -> EncoderResult<()> {
        self.writer.write_i32::<BigEndian>(value).map_err(From::from)
	}

    #[inline]
    fn serialize_i64(&mut self, value: i64) -> EncoderResult<()> {
        self.writer.write_i64::<BigEndian>(value).map_err(From::from)
	}

    #[inline]
    fn serialize_u8(&mut self, value: u8) -> EncoderResult<()> {
        self.writer.write_u8(value).map_err(From::from)
	}

    #[inline]
    fn serialize_u16(&mut self, value: u16) -> EncoderResult<()> {
        self.writer.write_u16::<BigEndian>(value).map_err(From::from)
	}

    #[inline]
    fn serialize_u32(&mut self, value: u32) -> EncoderResult<()> {
        self.writer.write_u32::<BigEndian>(value).map_err(From::from)
	}

    #[inline]
    fn serialize_u64(&mut self, value: u64) -> EncoderResult<()> {
        self.writer.write_u64::<BigEndian>(value).map_err(From::from)
	}

    #[inline]
    fn serialize_f32(&mut self, value: f32) -> EncoderResult<()> {
        Err(EncoderError::Unknown(String::from("Not Implemented")))
	}

    #[inline]
    fn serialize_f64(&mut self, value: f64) -> EncoderResult<()> {
        Err(EncoderError::Unknown(String::from("Not Implemented")))
	}

    #[inline]
    fn serialize_char(&mut self, value: char) -> EncoderResult<()> {
        Err(EncoderError::Unknown(String::from("Not Implemented")))
	}

    #[inline]
    fn serialize_str(&mut self, value: &str) -> EncoderResult<()> {
        Err(EncoderError::Unknown(String::from("Not Implemented")))
	}

    #[inline]
    fn serialize_bytes(&mut self, value: &[u8]) -> EncoderResult<()> {
        Err(EncoderError::Unknown(String::from("Not Implemented")))
	}

    #[inline]
    fn serialize_unit(&mut self) -> EncoderResult<()> {
        Ok(())
	}

    #[inline]
    fn serialize_usize(&mut self, value: usize) -> EncoderResult<()> {
        Err(EncoderError::Unknown(String::from("Not Implemented")))
    }

    #[inline]
    fn serialize_none(&mut self) -> EncoderResult<()> {
        Err(EncoderError::Unknown(String::from("Not Implemented")))
    }

    #[inline]
    fn serialize_some<V>(&mut self, value: V) -> EncoderResult<()> where V: Serialize {
        Err(EncoderError::Unknown(String::from("Not Implemented")))
    }

    #[inline]
    fn serialize_unit_struct(&mut self, _name: &'static str) -> EncoderResult<()> {
        Err(EncoderError::Unknown(String::from("Unit Struct")))
    }

    #[inline]
    fn serialize_newtype_struct<T>(&mut self, _name: &'static str, value: T)
                                   -> EncoderResult<()> where T: Serialize {
        Err(EncoderError::Unknown(String::from("Newtype struct")))
    }

    #[inline]
    fn serialize_struct(&mut self, name: &'static str, len: usize) -> EncoderResult<Self::MapState> {
        Ok(MapState {
            size: len,
            slots: Vec::new(),
        })
    }

    #[inline]
    fn serialize_struct_elt<T: Serialize>(&mut self, state: &mut Self::MapState,
                                          key: &'static str, value: T) -> EncoderResult<()> {
        // keep state around in case we need to do something fancy
        state.slots.append(&mut to_bytes(&value).unwrap());
        value.serialize(self)
    }

    #[inline]
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
        Err(EncoderError::Unknown(String::from("Not Implemented")))
    }

    #[inline]
    fn serialize_tuple_variant(&mut self, _name: &str, _variant_index: usize, variant: &str,
                               _len: usize) -> EncoderResult<()> {
        Err(EncoderError::Unknown(String::from("Not Implemented")))
    }

    #[inline]
    fn serialize_tuple_variant_elt<T: Serialize>(&mut self, _state: &mut (),
                                                 value: T) -> EncoderResult<()> {
        Err(EncoderError::Unknown(String::from("Not Implemented")))
    }

    #[inline]
    fn serialize_tuple_variant_end(&mut self, _state: ()) -> EncoderResult<()> {
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
        Ok(state.slots.append(&mut to_bytes(&value).unwrap()))
    }

    #[inline]
    fn serialize_struct_variant_end(&mut self, state: MapState) -> EncoderResult<()> {
        Err(EncoderError::Unknown(String::from("Not Implemented")))
    }

    #[inline]
    fn serialize_tuple_struct(&mut self, _name: &'static str, len: usize) -> EncoderResult<bool> {
        Err(EncoderError::Unknown(String::from("Not Implemented")))
    }

    #[inline]
    fn serialize_tuple_struct_elt<T: Serialize>(&mut self, state: &mut bool, value: T) -> EncoderResult<()> {
        Err(EncoderError::Unknown(String::from("Not Implemented")))
    }

    #[inline]
    fn serialize_tuple_struct_end(&mut self, state: bool) -> EncoderResult<()> {
        Err(EncoderError::Unknown(String::from("Not Implemented")))
    }

    #[inline]
    fn serialize_tuple(&mut self, len: usize) -> EncoderResult<bool> {
        Err(EncoderError::Unknown(String::from("Not Implemented")))
    }

    #[inline]
    fn serialize_tuple_elt<T: Serialize>(&mut self, _state: &mut bool, value: T) -> EncoderResult<()> {
        Err(EncoderError::Unknown(String::from("Not Implemented")))
    }

    #[inline]
    fn serialize_tuple_end(&mut self, state: bool) -> EncoderResult<()> {
        Err(EncoderError::Unknown(String::from("Not Implemented")))
    }

    #[inline]
    fn serialize_seq(&mut self, len: Option<usize>) -> EncoderResult<Option<usize>> {
        Err(EncoderError::Unknown(String::from("Not Implemented")))
    }

    #[inline]
    fn serialize_seq_elt<T>(&mut self, state: &mut Option<usize>,
                            value: T) -> EncoderResult<()> where T: Serialize {
        Err(EncoderError::Unknown(String::from("Not Implemented")))
    }

    #[inline]
    fn serialize_seq_end(&mut self, state: Option<usize>) -> EncoderResult<()> {
        Err(EncoderError::Unknown(String::from("Not Implemented")))
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

    #[inline]
    fn serialize_map_end(&mut self, state: MapState) -> EncoderResult<()> {
        Err(EncoderError::Unknown(String::from("Not Implemented")))
    }
}
