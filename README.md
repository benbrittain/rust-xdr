# rust-xdr &emsp; [![Build Status](https://travis-ci.org/cavedweller/rust-xdr.svg?branch=master)](https://travis-ci.org/cavedweller/rust-xdr)
**Converts XDR RPC definitions into Rust Services**

## Description
rust-xdr is a framework for building Rust services from XDR RPC specifications. This is 2 parts
* `serde_xdr`: A serialization/deserialization library implemented ontop of `serde`. This is a stand-alone crate
  as well.
* `rust-xdr`: A code generator for tokio based services. This depends on `serde_xdr` for
  serialization and some of the runtime code.

## Setup
Assuming a Rust/Cargo install
```
$ cargo install xdrgen
$ xdrgen --input <files that define your service or types> --output <directory>
```

If you are attempting to only use `serde_xdr` be aware that discrimant unions require extra serde
annotations due to limitations of XDR. The examples directory shows how to properly annotate these
if you are not codegening off a XDR file (which generates the annotations for you)

See the `/examples` directory.

## Examples
* `/examples/xdr_protocol_example/` shows how the XDR protocol implementation is primarily codegenned off
the RPC XDR specification.
* `/examples/rust_xdr_service` shows how to implement a service. The main.rs & service.rs are hand
  written files. `/examples/rust_xdr_service/xdrgen` is the output directory of `xdrgen`.  

  ```
  cargo run --bin rust-xdr -- -i examples/example_prot.v --output examples/service_example/xdrgen
  ```

## Background
Built primarily during OkCupid Hack Week 2017, the goal of the project was to Proof Of Concept standing up Rust services
that could easily interop with our existing C++ ones. Beyond building a typesafe (de)serialization library, we also
wanted to automatically generate the the whole service, except for the implementation specific details for an RPC call.

## Timeline
* Make serde_xdr a stand-alone crate
* Serialization of Option<> types as discriminant unions
* Prevent consumption of bytes in too small of packets
* OkCupid specific complilation flag for non-standard tweaks
* PoC client implementation
* Improve Error messages
* Implement more XDR rejection responses
* Test Suite

## Contacts
Please file a github issue if you experience any problems
* [Ben Brittain](https://github.com/cavedweller)
* [Brendon Scheinman](https://github.com/bscheinman)

## Specifications Implemented
* [XDR - RFC4506](https://tools.ietf.org/html/rfc4506.html)
* [RPC v2 - RFC1831](https://tools.ietf.org/html/rfc1831.html)
