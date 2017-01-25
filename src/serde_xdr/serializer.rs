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
        $(fn $name<>(self, $($arg: $ty,)*) -> EncoderResult<()> {
            Err(EncoderError::Unknown(format!("Serialize Not Implemented for {}", stringify!($name))))
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

impl<'a, W: io::Write> ser::Serializer for &'a mut Serializer<W> {
    type Error = EncoderError;
    type Ok = ();

    type SerializeSeq = Compound<'a, W>;
    type SerializeTuple = Compound<'a, W>;
    type SerializeTupleStruct = Compound<'a, W>;
    type SerializeTupleVariant = Compound<'a, W>;
    type SerializeMap = Compound<'a, W>;
    type SerializeStruct = Compound<'a, W>;
    type SerializeStructVariant = Compound<'a, W>;

    fn serialize_i8(self, value: i8) -> EncoderResult<()> {
        self.writer.write_i8(value).map_err(From::from)
	}

    fn serialize_i16(self, value: i16) -> EncoderResult<()> {
        self.writer.write_i16::<BigEndian>(value).map_err(From::from)
	}

    fn serialize_i32(self, value: i32) -> EncoderResult<()> {
        self.writer.write_i32::<BigEndian>(value).map_err(From::from)
	}

    fn serialize_i64(self, value: i64) -> EncoderResult<()> {
        self.writer.write_i64::<BigEndian>(value).map_err(From::from)
	}

    fn serialize_u8(self, value: u8) -> EncoderResult<()> {
        self.writer.write_u8(value).map_err(From::from)
	}

    fn serialize_u16(self, value: u16) -> EncoderResult<()> {
        self.writer.write_u16::<BigEndian>(value).map_err(From::from)
	}

    fn serialize_u32(self, value: u32) -> EncoderResult<()> {
        self.writer.write_u32::<BigEndian>(value).map_err(From::from)
	}

    fn serialize_u64(self, value: u64) -> EncoderResult<()> {
        self.writer.write_u64::<BigEndian>(value).map_err(From::from)
	}

    not_implemented!(
        serialize_f32(val: f32,);
        serialize_f64(val: f64,);
        serialize_char(val: char,);
        serialize_str(val: &str,);
        serialize_none();
        serialize_bytes(val: &[u8],);
        serialize_unit_struct(_name: &'static str,);
    );

    fn serialize_bool(self, v: bool) -> EncoderResult<()> {
        self.writer.write_u8(if v {1} else {0}).map_err(From::from)
    }

    fn serialize_unit(self) -> EncoderResult<()> {
        Ok(())
	}

    fn serialize_some<T: ?Sized>(self, value: &T) -> EncoderResult<()> where T: ser::Serialize, {
        Err(EncoderError::Unknown(String::from("Not yet implemented")))
    }


    fn serialize_newtype_struct<T: ?Sized>( self, _name: &'static str, value: &T) -> EncoderResult<()> where T: ser::Serialize {
        Err(EncoderError::Unknown(String::from("Not yet implemented")))
    }


    fn serialize_newtype_variant<T: ?Sized>(self, _name: &'static str, _variant_index: usize, _variant: &'static str, _value: &T) -> EncoderResult<()> where T: ser::Serialize, {
        Err(EncoderError::Unknown(String::from("Not yet implemented")))
    }

    fn serialize_seq_fixed_size(self, size: usize) -> EncoderResult<Self::SerializeSeq> {
        Ok(Compound { ser: self, size: None})
    }

    fn serialize_struct(self, name: &'static str, len: usize) -> EncoderResult<Self::SerializeStruct> {
        Ok(Compound { ser: self, size: Some(len) })
    }

    fn serialize_map(self, len: Option<usize>) -> EncoderResult<Self::SerializeMap> {
        Err(EncoderError::Unknown(String::from("Not yet implemented")))
    }

    fn serialize_unit_variant(self, _name: &str, variant_index: usize, variant: &str) -> EncoderResult<()> {
        self.serialize_i32(variant_index as i32)
    }

    fn serialize_seq(self, len: Option<usize>) -> EncoderResult<Self::SerializeSeq> {
        self.serialize_u32((len.unwrap() as u32));
        Ok(Compound { ser: self, size: len})
    }

    fn serialize_tuple(self, len: usize) -> EncoderResult<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct( self, _name: &'static str, len: usize) -> EncoderResult<Self::SerializeTupleStruct> {
        // println!("{}", _name);
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant( self, _name: &'static str, _variant_index: usize, variant: &'static str, len: usize) -> EncoderResult<Self::SerializeTupleVariant> {
        Err(EncoderError::Unknown(String::from("Not Implemented Tuple Elttv")))
    }

    fn serialize_struct_variant(self, _name: &'static str, _variant_index: usize, variant: &'static str, len: usize) -> EncoderResult<Self::SerializeStructVariant> {
        // println!("sv: {}", variant);
        // println!("len: {}", len);
        let descr_idx = variant.parse::<u32>();
        match descr_idx {
            Ok(idx) => {
                self.serialize_u32(idx);
                Ok(Compound { ser: self, size: Some(idx as usize)})
            },
            Err(_) => {
                println!("You probably modified a codegen'd file. stop that shit");
                Err(EncoderError::Unknown(String::from("You probably modified a codegen'd file. stop that shit")))
            }
        }
    }

    //fn serialize_seq_elt<T>(self, state: &mut Option<usize>,
    //                        value: T) -> EncoderResult<()> where T: Serialize {
    //    value.serialize(self);
    //    let mut len = state.unwrap();
    //    if len > 0 {
    //        match state.iter_mut().next() { // TODO there is probably an easier way to grab a mut ref to an option
    //            Some(v) => *v = len - 1,
    //            None => {},
    //        }
    //        Ok(())
    //    } else {
    //        Err(EncoderError::Unknown(String::from("Sequence Serializer ran out of items!")))
    //    }
    //}
}

pub struct Compound<'a, W: 'a> {
    ser: &'a mut Serializer<W>,
    size: Option<usize>,
}

impl<'a, W> ser::SerializeSeq for Compound<'a, W>
    where W: io::Write
{
    type Ok = ();
    type Error = EncoderError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> EncoderResult<()> where T: ser::Serialize {
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> EncoderResult<()> {
        Ok(())
    }
}

impl<'a, W> ser::SerializeTupleVariant for Compound<'a, W>
    where W: io::Write
{
    type Ok = ();
    type Error = EncoderError;


    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> EncoderResult<()> where T: ser::Serialize {
        Err(EncoderError::Unknown(String::from("Not Implemented fix 4")))
    }

    fn end(self) -> EncoderResult<()> {
        Err(EncoderError::Unknown(String::from("Not Implemented fix 3 ")))
    }
}

impl<'a, W> ser::SerializeTuple for Compound<'a, W>
    where W: io::Write
{
    type Ok = ();
    type Error = EncoderError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> EncoderResult<()> where T: ser::Serialize {
        Err(EncoderError::Unknown(String::from("Not Implemented fix 2")))
    }

    fn end(self) -> EncoderResult<()> {
        Err(EncoderError::Unknown(String::from("Not Implemented fix 1")))
    }
}

impl<'a, W> ser::SerializeMap for Compound<'a, W>
    where W: io::Write
{
    type Ok = ();
    type Error = EncoderError;

    fn serialize_key<T: ?Sized>(&mut self, value: &T) -> EncoderResult<()> where T: ser::Serialize {
        Err(EncoderError::Unknown(String::from("her Not Implemented fix here?")))
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> EncoderResult<()> where T: ser::Serialize {
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> EncoderResult<()> {
        Ok(())
    }
}

impl<'a, W> ser::SerializeStruct for Compound<'a, W> where W: io::Write {

    type Ok = ();
    type Error = EncoderError;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> EncoderResult<()> where T: ser::Serialize {
        ser::SerializeMap::serialize_value(self, value)
    }

    fn end(self) -> EncoderResult<()> {
        ser::SerializeMap::end(self)
    }
}


impl<'a, W> ser::SerializeStructVariant for Compound<'a, W>
    where W: io::Write
{
    type Ok = ();
    type Error = EncoderError;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> EncoderResult<()> where T: ser::Serialize {
        ser::SerializeMap::serialize_value(self, value)
    }

    fn end(self) -> EncoderResult<()> {
        Ok(())
    }
}

impl<'a, W> ser::SerializeTupleStruct for Compound<'a, W>
    where W: io::Write
{
    type Ok = ();
    type Error = EncoderError;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> EncoderResult<()> where T: ser::Serialize {
        Err(EncoderError::Unknown(String::from("Not Implemented fix aoeu")))
    }

    fn end(self) -> EncoderResult<()> {
        Err(EncoderError::Unknown(String::from("Not Implemented fix")))
    }
}
