#[macro_use]
extern crate serde_derive;

extern crate serde;
#[macro_use]
extern crate serde_xdr;

mod gen_test;
use gen_test::*;

mod rpc;
use rpc::*;

fn main() {

    let test = RpcMsg {
        xid: 5777u32,
        body: Body::Reply {
            rbody: ReplyBody::MsgAccepted {
                areply: AcceptedReply {
                    verf: OpaqueAuth {
                        flavor: AuthFlavor::AuthNone,
                        body: Vec::new()
                    },
                    reply_data: ReplyData::Success {
                        vers: 2
                    }
                }
            }
        }
    };

    let mut bytes = Vec::<u8>::new();
    serde_xdr::to_bytes(&test, &mut bytes);
    println!("{:?}", bytes);

    let obj = serde_xdr::from_bytes::<RpcMsg>(&bytes);
    println!("{:?}", obj);
}
