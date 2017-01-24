use std::io::{self, Read};
use byteorder::{BigEndian, ReadBytesExt};
use serde::de::{self, EnumVisitor, Visitor, Deserialize};
use serde::bytes::ByteBuf;

use std::result;
use error::{DecoderResult, EncoderError};
use serde::de::value::ValueDeserializer;

//impl<R: Read> Deserializer<R> {
//    pub fn new(rdr: R) -> Deserializer<R> {
//        Deserializer {
//            //rdr: BufReader::new(rdr),
//            //pos: 0,
//            //value: None,
//            //memo: BTreeMap::new(),
//            //stack: Vec::with_capacity(128),
//            //stacks: Vec::with_capacity(16),
//        }
//    }
// }

macro_rules! not_implemented {
    ($($name:ident($($arg:ident: $ty:ty,)*);)*) => {
        $(fn $name<V: Visitor>(&mut self, $($arg: $ty,)* visitor: V) -> DecoderResult<V::Value> {
            Err(EncoderError::Unknown(format!("Deserialize Not Implemented for {}", stringify!($name))))
        })*
    }
}

macro_rules! impl_num {
    ($ty:ty, $deserialize_method:ident, $visitor_method:ident, $read_method:ident) => {
        fn $deserialize_method<V>(&mut self, mut visitor: V) -> DecoderResult<V::Value>
            where V: de::Visitor, {
            visitor.$visitor_method(self.$read_method::<BigEndian>()?)
        }
    }
}


pub struct Deserializer<R: Read> {
    reader: R,
//    first: Option<u8>,
}

impl<R: Read> Deserializer<R> {
    pub fn new(reader: R) -> Deserializer<R> {
        Deserializer {
            reader: reader,
//            first: None,
        }
    }
}

impl<R: Read> de::Deserializer for Deserializer<R> {
    type Error = EncoderError;

    // Implementing all the numbers that use the simple read_TYPE syntax
    impl_num!(u16, deserialize_u16, visit_u16, read_u16);
    impl_num!(u32, deserialize_u32, visit_u32, read_u32);
    impl_num!(u64, deserialize_u64, visit_u64, read_u64);

    impl_num!(i16, deserialize_i16, visit_i16, read_i16);
    impl_num!(i32, deserialize_i32, visit_i32, read_i32);
    impl_num!(i64, deserialize_i64, visit_i64, read_i64);

    impl_num!(f32, deserialize_f32, visit_f32, read_f32);
    impl_num!(f64, deserialize_f64, visit_f64, read_f64);

    not_implemented!(
        deserialize();
        deserialize_bool();
        deserialize_isize();
        deserialize_usize();
        deserialize_char();
        deserialize_str();
        deserialize_string();
        deserialize_unit();
        deserialize_seq();
        deserialize_option();
        deserialize_seq_fixed_size(_len: usize,);
        deserialize_bytes();
        deserialize_map();
        deserialize_unit_struct(_name: &'static str,);
        deserialize_tuple_struct(_name: &'static str, _len: usize,);
        deserialize_tuple(_len: usize,);
        deserialize_struct_field();
        deserialize_ignored_any();
   );

   fn deserialize_u8<V: Visitor>(&mut self, mut visitor: V) -> DecoderResult<V::Value> {
       visitor.visit_u8(self.read_u8()?)
   }

   fn deserialize_i8<V: Visitor>(&mut self, mut visitor: V) -> DecoderResult<V::Value> {
       visitor.visit_i8(self.read_i8()?)
   }

   fn deserialize_struct<V>(&mut self,
                            name: &'static str,
                            fields: &'static [&'static str],
                            mut visitor: V) -> DecoderResult<V::Value> where V: de::Visitor {
       visitor.visit_seq(StructVisitor { deserializer: self, len: fields.len() })
   }


   fn deserialize_newtype_struct<V>(&mut self,
                                    name: &'static str,
                                    mut visitor: V) -> DecoderResult<V::Value> where V: de::Visitor {
       visitor.visit_newtype_struct(self)
   }

   fn deserialize_enum<V: EnumVisitor>(&mut self,
                                       _enum: &'static str,
                                       _variants: &'static [&'static str],
                                       mut visitor: V) -> DecoderResult<V::Value> {
       visitor.visit(self)
   }
}

impl<R: Read> Read for Deserializer<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.reader.read(buf)
    }
}

struct StructVisitor<'a, R: Read + 'a> {
    deserializer: &'a mut Deserializer<R>,
    len: usize,
}

impl<'a, R: Read> de::SeqVisitor for StructVisitor<'a, R> {
    type Error = EncoderError;

    fn visit<T>(&mut self) -> DecoderResult<Option<T>> where T: de::Deserialize {
        if self.len > 0 {
            self.len -= 1;
            let value = Deserialize::deserialize(self.deserializer)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    fn end(&mut self) -> DecoderResult<()> {
        if self.len != 0 {
            Err(EncoderError::Unknown(String::from("Expected an end for the struct")))
        } else {
            Ok(())
        }
    }
}

impl<R: Read> de::VariantVisitor for Deserializer<R> {
    type Error = EncoderError;

    fn visit_variant<V>(&mut self) -> DecoderResult<V> where V: de::Deserialize {
        let idx: u32 = Deserialize::deserialize(self)?;
        let mut deserializer = (idx as usize).into_deserializer();
        let attempt: DecoderResult<V> = Deserialize::deserialize(&mut deserializer);
        Ok(attempt?)
    }

    fn visit_unit(&mut self) -> DecoderResult<()> {
        Ok(())
    }

    fn visit_newtype<T>(&mut self) -> DecoderResult<T> where T: de::Deserialize {
        de::Deserialize::deserialize(self)
    }

    fn visit_tuple<V>(&mut self, _len: usize, visitor: V) -> DecoderResult<V::Value> where V: de::Visitor {
        de::Deserializer::deserialize(self, visitor)
    }

    fn visit_struct<V>(&mut self, _fields: &'static [&'static str], visitor: V)
                                            -> DecoderResult<V::Value> where V: de::Visitor {
        //TODO might need fancier stuff here
        de::Deserializer::deserialize(self, visitor)
    }
}
