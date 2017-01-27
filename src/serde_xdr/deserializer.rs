use std::io::{self, Read};
use byteorder::{BigEndian, ReadBytesExt};
use serde::de::{self, EnumVisitor, Visitor, Deserialize};
use serde::bytes::ByteBuf;

use std::result;
use error::{DecoderResult, EncoderError};
use serde::de::value::ValueDeserializer;

macro_rules! not_implemented {
    ($($name:ident($($arg:ident: $ty:ty,)*);)*) => {
        $(fn $name<V: Visitor>(self, $($arg: $ty,)* visitor: V) -> DecoderResult<V::Value> {
            Err(EncoderError::Unknown(format!("XDR deserialize not implemented for {}", stringify!($name))))
        })*
    }
}

macro_rules! impl_num {
    ($ty:ty, $deserialize_method:ident, $visitor_method:ident, $read_method:ident, $byte_size:expr) => {
        fn $deserialize_method<V>(self, mut visitor: V) -> DecoderResult<V::Value>
            where V: de::Visitor, {
                let res = visitor.$visitor_method(self.$read_method::<BigEndian>()?);
                self.bytes_consumed += $byte_size;
                res
        }
    }
}

pub struct Deserializer<R: Read> {
    reader: R,
    bytes_consumed: usize
}

impl<R: Read> Deserializer<R> {
    pub fn new(reader: R) -> Deserializer<R> {
        Deserializer {
            reader: reader,
            bytes_consumed: 0
        }
    }

   pub fn get_bytes_consumed(&self) -> usize {
        self.bytes_consumed
   }
}


enum xdr_enum_type{
    Enum,
    Union
}


impl<'a, R: Read> de::Deserializer for &'a mut Deserializer<R> {
    type Error = EncoderError;

    // Implementing all the numbers that use the simple read_TYPE syntax
    impl_num!(u16, deserialize_u16, visit_u16, read_u16, 2);
    impl_num!(u32, deserialize_u32, visit_u32, read_u32, 4);
    impl_num!(u64, deserialize_u64, visit_u64, read_u64, 8);

    impl_num!(i16, deserialize_i16, visit_i16, read_i16, 2);
    impl_num!(i32, deserialize_i32, visit_i32, read_i32, 4);
    impl_num!(i64, deserialize_i64, visit_i64, read_i64, 8);

    impl_num!(f32, deserialize_f32, visit_f32, read_f32, 4);
    impl_num!(f64, deserialize_f64, visit_f64, read_f64, 8);

    not_implemented!(
        deserialize_char();
        deserialize_str();
        deserialize_unit();
        deserialize_option();
        deserialize_bytes();
        deserialize_map();
        deserialize_unit_struct(_name: &'static str,);
        deserialize_tuple_struct(_name: &'static str, _len: usize,);
        deserialize_tuple(_len: usize,);
        deserialize_ignored_any();
    );


    fn deserialize_struct_field<V>(self, visitor: V) -> DecoderResult<V::Value> where V: de::Visitor {
        self.deserialize(visitor)
    }


    fn deserialize_string<V>(self, mut visitor: V) -> DecoderResult<V::Value> where V: de::Visitor {
        let count: u32 = self.read_u32::<BigEndian>()?;
        let extra_bytes = 4 - count % 4;
        let mut accum = String::new();
        for c in 0 .. count {
            accum.push(self.read_u8()? as char);
        }
        self.bytes_consumed += (extra_bytes + count + 4) as usize;
        return visitor.visit_string(accum);
    }


    fn deserialize_enum<V>(self, name: &str, variants: &'static [&'static str], visitor: V) -> DecoderResult<V::Value> where V: de::Visitor, {
        if name == "__UNION_SYMBOL__"  {
            visitor.visit_enum(VariantVisitor::new(self, xdr_enum_type::Union, variants))
        } else {
            visitor.visit_enum(VariantVisitor::new(self, xdr_enum_type::Enum, variants))
        }
    }

    fn deserialize_byte_buf<V:Visitor>(self, mut visitor: V) ->  DecoderResult<V::Value> {
        Err(EncoderError::Unknown(String::from("not done implementing")))
    }

    fn deserialize<V: Visitor>(self, mut visitor: V) -> DecoderResult<V::Value> {
        Err(EncoderError::Unknown(String::from("Generic Deserialize method not implemented since XDR is not self describing")))
    }

    fn deserialize_bool<V: Visitor>(self, mut visitor: V) -> DecoderResult<V::Value> {
        let value: u8 = try!(Deserialize::deserialize(self));
        match value {
            1 => visitor.visit_bool(true),
            0 => visitor.visit_bool(false),
            _ => { Err(EncoderError::Unknown(String::from("invalid u8 when decoding bool, 0 or 1 needed"))) }
        }
    }

    fn deserialize_u8<V: Visitor>(self, mut visitor: V) -> DecoderResult<V::Value> {
        let res = visitor.visit_u8(self.read_u8()?);
        self.bytes_consumed += 1;
        res
    }

    fn deserialize_i8<V: Visitor>(self, mut visitor: V) -> DecoderResult<V::Value> {
        let res = visitor.visit_i8(self.read_i8()?);
        self.bytes_consumed += 1;
        res
    }

    fn deserialize_struct<V>(self, name: &'static str, fields: &'static [&'static str], mut visitor: V) -> DecoderResult<V::Value> where V: de::Visitor {
        visitor.visit_seq(SeqVisitor::new(self, Some(fields.len() as u32)))
    }


    fn deserialize_newtype_struct<V>(self,
                                     name: &'static str,
                                     mut visitor: V) -> DecoderResult<V::Value> where V: de::Visitor {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V: Visitor>(self, mut visitor: V) -> DecoderResult<V::Value> {
        visitor.visit_seq(SeqVisitor::new(self, None))
    }

    fn deserialize_seq_fixed_size<V: Visitor>(self, len: usize, mut visitor: V) -> DecoderResult<V::Value> {
        visitor.visit_seq(SeqVisitor::new(self, Some(len as u32)))
    }
}

impl<R: Read> Read for Deserializer<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.reader.read(buf)
    }
}

struct SeqVisitor<'a, R: Read + 'a> {
    deserializer: &'a mut Deserializer<R>,
    len: Option<u32>,
}

impl<'a, R: Read + 'a> SeqVisitor<'a, R> {
    fn new(de: &'a mut Deserializer<R>, size: Option<u32>) -> Self {
        SeqVisitor {
            deserializer: de,
            len: size,
        }
    }
}

impl<'a, R: Read + 'a> de::SeqVisitor for SeqVisitor<'a, R> {
    type Error = EncoderError;

    fn visit<T>(&mut self) -> DecoderResult<Option<T>> where T: de::Deserialize {
        if let None = self.len {
            self.len = Some(Deserialize::deserialize(&mut *self.deserializer)?);
        }
        let len = self.len.unwrap();
        if len > 0 {
            match self.len.iter_mut().next() { // TODO there is probably an easier way to grab a mut ref to an option
                Some(v) => *v = len - 1,
                None => {},
            }
            let value = Deserialize::deserialize(&mut *self.deserializer)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    fn visit_seed<T>(&mut self, seed: T) -> DecoderResult<Option<T::Value>> where T: de::DeserializeSeed, {
        Err(EncoderError::Unknown(format!("XDR deserialize not implemented for visit seed" )))
    }
}

impl<R: Read> de::VariantVisitor for Deserializer<R> {
    type Error = EncoderError;


    fn visit_newtype_seed<T>(self, seed: T) -> DecoderResult<T::Value> where T: de::DeserializeSeed,{
        Err(EncoderError::Unknown(format!("XDR deserialize not implemented for" )))
        //seed.deserialize(self)
    }

    fn visit_unit(self) -> DecoderResult<()> {
        Ok(())
    }

    fn visit_newtype<T>(self) -> DecoderResult<T> where T: de::Deserialize {
        Err(EncoderError::Unknown(format!("XDR deserialize not implemented for" )))
        //de::Deserialize::deserialize(self)
    }

    fn visit_tuple<V>(self, _len: usize, visitor: V) -> DecoderResult<V::Value> where V: de::Visitor {
        Err(EncoderError::Unknown(format!("XDR deserialize not implemented for" )))
        //de::Deserializer::deserialize(self, visitor)
    }

    fn visit_struct<V>(self, _fields: &'static [&'static str], visitor: V)
                                            -> DecoderResult<V::Value> where V: de::Visitor {
        Err(EncoderError::Unknown(format!("XDR deserialize not implemented for" )))
    }
}


struct VariantVisitor<'a, R: Read + 'a> {
    de: &'a mut Deserializer<R>,
    style: xdr_enum_type,
    variants: &'static [&'static str]
}

impl<'a, R: Read + 'a> VariantVisitor<'a, R> {
    fn new(de: &'a mut Deserializer<R>, style: xdr_enum_type, variants: &'static [&'static str]) -> Self {
        VariantVisitor {
            de: de,
            style: style,
            variants: variants,
        }
    }
}

impl<'a, R: Read + 'a> de::EnumVisitor for VariantVisitor<'a, R> {
    type Error = EncoderError;
    type Variant = Self;

    fn visit_variant_seed<V>(self, seed: V) -> DecoderResult<(V::Value, Self)> where V: de::DeserializeSeed, {
        match self.style {
            xdr_enum_type::Union => {
                let enum_index: u32 = Deserialize::deserialize(&mut *self.de)?;
                let mut union_index: u32 = (self.variants.len() - 1) as u32;
                if enum_index < self.variants.len() as u32 {
                    let ids = self.variants.iter().map(|x| x.parse::<u32>().unwrap()).position(|x| x == enum_index);
                    union_index = match ids {
                        Some(idx) => idx as u32,
                        None => {
                            return Err(EncoderError::Unknown(format!("Bad Index for Union, the codegen annotations are broken probably" )));
                        }
                    };
                }
                let mut des = union_index.into_deserializer();
                let val: Result<V::Value, de::value::Error> = seed.deserialize(des);
                Ok((val.unwrap(), self))
            },
            xdr_enum_type::Enum => {
                let val = try!(seed.deserialize(&mut *self.de));
                Ok((val, self))
            }
        }
    }
}

impl<'a, R: Read + 'a> de::VariantVisitor for VariantVisitor<'a, R> {
    type Error = EncoderError;

    fn visit_unit(self) -> DecoderResult<()> {
        de::Deserialize::deserialize(self.de)
    }

    fn visit_newtype_seed<T>(self, seed: T) -> DecoderResult<T::Value> where T: de::DeserializeSeed, {
        seed.deserialize(self.de)
    }

    fn visit_tuple<V>(self, len: usize, visitor: V) -> DecoderResult<V::Value> where V: de::Visitor, {
        visitor.visit_seq(SeqVisitor::new(self.de, Some(len as u32)))
    }

    fn visit_struct<V>( self, fields: &'static [&'static str], visitor: V) -> DecoderResult<V::Value> where V: de::Visitor, {
        visitor.visit_seq(SeqVisitor::new(self.de, Some(fields.len() as u32)))
    }
}

