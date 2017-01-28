# xdrproto
Converts XDR/RPC files into Rust Services (tokio/mio/serde/nom)

### Background
Built primarily during OkCupid Hack Week 2017, the goal of the project was to Proof Of Concept standing up Rust services
that could easily interop with our existing C++ ones. Beyond building a typesafe (de)serialization library, we also
wanted to automatically generate the the whole service, except for the implementation specific details for an RPC call. We succeded in our goal, but there are a few TODOs before a "release"

## TODO
* Rename Project
* Pull xdr_serde into it's own repo
* serialization of Option<> types
* boolean discrimant unions
* Test Suite
* CI setup
* fix minor service codegen issues
* better DB setup
* Example service
* Better readme with instructions
