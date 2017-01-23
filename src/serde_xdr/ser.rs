use std::fmt;
use std::error;
use std::result;
use std::io;
use std::str::From;
use serde::ser;
use serde::ser::Serialize;
use byteorder::{LittleEndian, BigEndian, WriteBytesExt};


#[derive(Clone, PartialEq, Debug)]
pub enum ErrorCode {
    Custom(String)
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ErrorCode::Custom(ref s) => fmt.write_str(s),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    None,
    //Bool(bool),
    //I64(i64),
}

#[derive(Debug)]
pub enum Error {
    // Some IO error occurred when serializing or deserializing a value.
    Io(io::Error),
    // The XDR had some error while interpreting.
    Eval(ErrorCode, usize),
    // Syntax error while transforming into Rust values.
    Syntax(ErrorCode),
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Io(ref error) => error::Error::description(error),
            Error::Eval(..) => "XDR eval error",
            Error::Syntax(..) => "serde decoding error",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref error) => error.fmt(fmt),
            Error::Eval(ref code, offset) => write!(fmt, "eval error at offset {}: {}",
                                                    offset, code),
            Error::Syntax(ref code) => write!(fmt, "decoding error: {}", code)
        }
    }
}

impl ser::Error for Error {
    fn custom<T: Into<String>>(msg: T) -> Error {
        Error::Syntax(ErrorCode::Custom(msg.into()))
    }
}

pub struct Serializer<W> {
    writer: W,
}

type Result<T> = result::Result<T, Error>;

impl<W: io::Write> Serializer<W> {
    pub fn new(writer: W, use_proto_3: bool) -> Self {
        Serializer {
            writer: writer,
        }
    }

    pub fn into_inner(self) -> W {
        self.writer
    }

    #[inline]
    fn write_opcode(&mut self, opcode: u8) -> Result<()> {
//        self.writer.write_all(&[opcode]).map_err(From::from)
        self.writer.write_all(&[opcode])
    }

    fn serialize_value(&mut self, value: &Value) -> Result<()> {
        use serde::Serializer;
        match *value {
            // Cases covered by the Serializer trait
            //Value::None    => self.serialize_unit(),
            //Value::Bool(b) => self.serialize_bool(b),
            //Value::I64(i)  => self.serialize_i64(i),
            //Value::F64(f)  => self.serialize_f64(f),
        }
    }
}

impl<W: io::Write> ser::Serializer for Serializer<W> {
    type Error = Error;
    type SeqState = Option<usize>;
    type TupleState = bool;
    type MapState = Option<usize>;
    type StructState = Self::MapState;
    type TupleStructState = bool;
    type StructVariantState = Self::MapState;
    type TupleVariantState = ();

    #[inline]
    fn serialize_bool(&mut self, value: bool) -> Result<()> {
		// TODO
	}

    #[inline]
    fn serialize_isize(&mut self, value: isize) -> Result<()> {
		// TODO
	}

    #[inline]
    fn serialize_i8(&mut self, value: i8) -> Result<()> {
		// TODO
	}

    #[inline]
    fn serialize_i16(&mut self, value: i16) -> Result<()> {
		// TODO
	}

    #[inline]
    fn serialize_i32(&mut self, value: i32) -> Result<()> {
		try!(self.write_opcode(b'0'));
		self.writer.write_i32::<LittleEndian>(value).map_err(From::from)
	}

    #[inline]
    fn serialize_i64(&mut self, value: i64) -> Result<()> {
		// TODO
	}

    #[inline]
    fn serialize_u8(&mut self, value: u8) -> Result<()> {
		// TODO
	}

    #[inline]
    fn serialize_u16(&mut self, value: u16) -> Result<()> {
		// TODO
	}

    #[inline]
    fn serialize_u32(&mut self, value: u32) -> Result<()> {
		// TODO
	}

    #[inline]
    fn serialize_u64(&mut self, value: u64) -> Result<()> {
		// TODO
	}

    #[inline]
    fn serialize_f32(&mut self, value: f32) -> Result<()> {
		// TODO
	}

    #[inline]
    fn serialize_f64(&mut self, value: f64) -> Result<()> {
		// TODO
	}

    #[inline]
    fn serialize_char(&mut self, value: char) -> Result<()> {
		// TODO
	}

    #[inline]
    fn serialize_str(&mut self, value: &str) -> Result<()> {
		// TODO
	}

    #[inline]
    fn serialize_bytes(&mut self, value: &[u8]) -> Result<()> {
		// TODO
	}

    #[inline]
    fn serialize_unit(&mut self) -> Result<()> {
		// TODO
	}

    #[inline]
    fn serialize_usize(&mut self, value: usize) -> Result<()> {
        // TODO
    }

    #[inline]
    fn serialize_none(&mut self) -> Result<()> {
    }

    #[inline]
    fn serialize_some<V>(&mut self, value: V) -> Result<()> where V: Serialize {
    }

    #[inline]
    fn serialize_unit_struct(&mut self, _name: &'static str) -> Result<()> {
    }

    #[inline]
    fn serialize_newtype_struct<T>(&mut self, _name: &'static str, value: T)
                                   -> Result<()> where T: Serialize {
    }

    #[inline]
    fn serialize_struct(&mut self, _name: &'static str, len: usize) -> Result<Self::MapState> {
    }

    #[inline]
    fn serialize_struct_elt<T: Serialize>(&mut self, state: &mut Self::MapState,
                                          key: &'static str, value: T) -> Result<()> {
    }

    #[inline]
    fn serialize_struct_end(&mut self, state: Self::MapState) -> Result<()> {
    }

    #[inline]
    fn serialize_unit_variant(&mut self, _name: &str, _variant_index: usize, variant: &str)
        -> Result<()> {
    }

    #[inline]
    fn serialize_newtype_variant<T>(&mut self, _name: &str, _variant_index: usize, variant: &str,
                                    value: T) -> Result<()> where T: Serialize {
    }

    #[inline]
    fn serialize_tuple_variant(&mut self, _name: &str, _variant_index: usize, variant: &str,
                               _len: usize) -> Result<()> {
    }

    #[inline]
    fn serialize_tuple_variant_elt<T: Serialize>(&mut self, _state: &mut (),
                                                 value: T) -> Result<()> {
    }

    #[inline]
    fn serialize_tuple_variant_end(&mut self, _state: ()) -> Result<()> {
    }

    #[inline]
    fn serialize_struct_variant(&mut self, _name: &str, _variant_index: usize, variant: &str,
                                len: usize) -> Result<Option<usize>> {
    }

    #[inline]
    fn serialize_struct_variant_elt<T: Serialize>(&mut self, state: &mut Option<usize>,
                                                       key: &'static str, value: T) -> Result<()> {
    }

    #[inline]
    fn serialize_struct_variant_end(&mut self, state: Option<usize>) -> Result<()> {
    }

    #[inline]
    fn serialize_tuple_struct(&mut self, _name: &'static str, len: usize) -> Result<bool> {
    }

    #[inline]
    fn serialize_tuple_struct_elt<T: Serialize>(&mut self, state: &mut bool, value: T) -> Result<()> {
    }

    #[inline]
    fn serialize_tuple_struct_end(&mut self, state: bool) -> Result<()> {
    }

    #[inline]
    fn serialize_tuple(&mut self, len: usize) -> Result<bool> {
    }

    #[inline]
    fn serialize_tuple_elt<T: Serialize>(&mut self, _state: &mut bool, value: T) -> Result<()> {
    }

    #[inline]
    fn serialize_tuple_end(&mut self, state: bool) -> Result<()> {
    }

    #[inline]
    fn serialize_seq(&mut self, len: Option<usize>) -> Result<Option<usize>> {
    }

    #[inline]
    fn serialize_seq_elt<T>(&mut self, state: &mut Option<usize>,
                            value: T) -> Result<()> where T: Serialize {
    }

    #[inline]
    fn serialize_seq_end(&mut self, state: Option<usize>) -> Result<()> {
    }

    #[inline]
    fn serialize_seq_fixed_size(&mut self, _len: usize) -> Result<Option<usize>> {
    }

    #[inline]
    fn serialize_map(&mut self, len: Option<usize>) -> Result<Option<usize>> {
    }

    #[inline]
    fn serialize_map_key<T: Serialize>(&mut self, _state: &mut Option<usize>, key: T) -> Result<()> {
    }

    #[inline]
    fn serialize_map_value<T: Serialize>(&mut self, state: &mut Option<usize>, value: T) -> Result<()> {
    }

    #[inline]
    fn serialize_map_end(&mut self, state: Option<usize>) -> Result<()> {
    }
}
