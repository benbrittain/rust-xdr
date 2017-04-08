use std::io;

use serde_xdr;
use tokio_core::io::{Codec, EasyBuf};
use xdr_rpc;
use xdr_rpc::HasXid;

enum XdrCodecState {
    AwaitingCall,
    AwaitingBody
}

pub trait AppCodec : Codec {
    fn app_decode(&mut self, program: u32, version: u32, procedure: u32,
                  buf: &mut EasyBuf) -> io::Result<Option<Self::In>>;
}

pub struct XdrCodec<TApp: AppCodec> {
    state: XdrCodecState,
    prog: u32,
    vers: u32,
    proc_: u32,
    xid: u32,
    app_codec: TApp
}

fn wrap_error<T>(e: serde_xdr::EncoderError, desc: &str) -> io::Result<T> {
    match e {
        serde_xdr::EncoderError::Io(i) => {
            Err(i)
        },
        serde_xdr::EncoderError::Unknown(s) => {
            Err(io::Error::new(io::ErrorKind::Other,
                format!("{}: {}", desc, s)))
        }
    }
}

impl<TApp: AppCodec> XdrCodec<TApp> {
    pub fn new(c: TApp) -> XdrCodec<TApp> {
        XdrCodec {
            state: XdrCodecState::AwaitingCall,
            prog: 0,
            vers: 0,
            proc_: 0,
            xid: 0,
            app_codec: c,
        }
    }

    fn decode_call(&mut self, buf: &mut EasyBuf) -> io::Result<Option<()>> {
        // XXX: only drain this if we also successfully deserialize a message
        // below
        buf.drain_to(4);
        //println!("{:?}", buf.as_slice());
        let rpc_msg = serde_xdr::from_bytes::<xdr_rpc::RpcMsg>(buf.as_slice());

        let msg = match rpc_msg{
            Ok((c, consumed)) => {
                buf.drain_to(consumed);
                self.xid = c.xid;
                c
            },
            Err(e) => {
                println!("failed to decode message type");
                return Ok(None);
            }
        };

        match msg.body {
            xdr_rpc::Body::Call{cbody: body} => {
                match body.rpcvers {
                    2u32 => {
                        self.prog = body.prog;
                        self.vers = body.vers;
                        self.proc_ = body.proc_;
                        Ok (Some(()))
                    },
                    3u32 => {
                        self.prog = body.prog;
                        self.vers = body.vers;
                        self.proc_ = body.proc_;
                        Ok (Some(()))
                    },
                    _ => {
                        println!("unknown RPC version");
                        Ok(None)
                    }
                }
            },
            _ => {
                println!("unknown RPC version");
                Ok(None)
            }
        }
    }
}

impl<TApp: AppCodec> Codec for XdrCodec<TApp> {
    type In = xdr_rpc::XdrRequest<TApp::In>;
    type Out = xdr_rpc::XdrResponse<TApp::Out>;

    fn decode(&mut self, buf: &mut EasyBuf) -> io::Result<Option<Self::In>> {
        while buf.len() > 0 {
            match self.state {
                XdrCodecState::AwaitingCall => {
                    match self.decode_call(buf) {
                        Ok(o) => {
                            match o {
                                Some(_) => {
                                    self.state = XdrCodecState::AwaitingBody;
                                }
                                None => {
                                    return Ok(None);
                                }
                            }
                        },
                        Err(e) => {
                            return Err(e);
                        }
                    }
                },
                XdrCodecState::AwaitingBody => {
                    let val = self.app_codec.app_decode(self.prog, self.vers, self.proc_, buf);
                    return match val {
                        Ok(r) => {
                            self.state = XdrCodecState::AwaitingCall;
                            match r {
                                Some(v) => {
                                    Ok(Some(xdr_rpc::XdrRequest{ xid: self.xid, val: v}))
                                },
                                None => {
                                    Ok(None)
                                }
                            }
                            //Err(io::Error::new(io::ErrorKind::Other, "asdf"))
                        },
                        Err(e) => {
                            Err(e)
                        }
                    };
                }
            }
        }

        // no bytes left to read
        Ok(None)
    }

    fn encode(&mut self, msg: Self::Out, buf: &mut Vec<u8>) -> io::Result<()> {
        // XXX: for now just accept all calls
        let reply = xdr_rpc::RpcMsg {
            xid: msg.get_xid(),
            body: xdr_rpc::Body::Reply {
                rbody: xdr_rpc::ReplyBody::MsgAccepted {
                    areply: xdr_rpc::AcceptedReply {
                        verf: xdr_rpc::OpaqueAuth {
                            // XXX: no auth for now
                            flavor: xdr_rpc::AuthFlavor::AuthNone,
                            body: Vec::new()
                        },
                        // XXX: all calls succeed for now
                        // bubble up errors later
                        reply_data: xdr_rpc::ReplyData::Success{}
                    }
                }
            }
        };

        let mut scratch = Vec::new();
        try!(serde_xdr::to_bytes(&reply, &mut scratch));
        try!(self.app_codec.encode(msg.val, &mut scratch));

        let byte_len: u32 = (scratch.len() as u32) | (1u32 << (31 as usize));

        for i in 0..4 {
            let b = ((byte_len >> ((3 - i) * 8)) & 0xffu32) as u8;
            buf.push(b);
        }

        buf.extend(&scratch);

        //println!("response {:?}", msg);
        //println!("response buffer {:?}", buf);

        Ok(())
    }
}
