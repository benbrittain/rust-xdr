// autogenerated by xdrust
// boilerplate for tokio services
use std::{io, result};
use tokio_core::io::{Io, Framed, Codec, EasyBuf};
use xdrgen::{xdr_codec, xdr_rpc};
use xdrgen::xdr_codec::{AppCodec, XdrCodec};
use tokio_proto::pipeline::ServerProto;
use serde_xdr;
use xdrgen::prot::*;
use xdrgen::service::*;


pub struct ExampledbdAppCodec;
impl Codec for ExampledbdAppCodec {
  type In =   ExampledbdProgRequest;
  type Out =   ExampledbdProgResponse;
  fn decode(&mut self, buf: &mut EasyBuf) -> io::Result<Option<Self::In>> {
    unreachable!()
  }
  fn encode(&mut self, msg: Self::Out, buf: &mut Vec<u8>) -> io::Result<()> {
    encode(msg, buf)
  }
}
impl AppCodec for ExampledbdAppCodec {
  fn app_decode(&mut self, prog: u32, version: u32, procedure: u32, buf: &mut EasyBuf) -> io::Result<Option<Self::In>> {
    exampledbd_prog_decode(prog, version, procedure, buf)
  }
}
pub type ExampledbdCodec = XdrCodec<ExampledbdAppCodec>;

pub struct ExampledbdProtocol;
impl<T: Io + 'static> ServerProto<T> for ExampledbdProtocol {
  type Request =   xdr_rpc::XdrRequest<ExampledbdProgRequest>;
  type Response =   xdr_rpc::XdrResponse<ExampledbdProgResponse>;
  type Transport =   Framed<T, ExampledbdCodec>;
  type BindTransport =   io::Result<Self::Transport>;
  fn bind_transport(&self, io: T) -> Self::BindTransport {
    Ok(io.framed(ExampledbdCodec::new(ExampledbdAppCodec)))
  }
}