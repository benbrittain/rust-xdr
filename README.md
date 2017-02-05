# rust-xdr &emsp; [![Build Status](https://travis-ci.org/cavedweller/rust-xdr.svg?branch=master)](https://travis-ci.org/cavedweller/rust-xdr)
**Converts XDR RPC definitions into Rust Services**

## Background
Built primarily during OkCupid Hack Week 2017, the goal of the project was to Proof Of Concept standing up Rust services
that could easily interop with our existing C++ ones. Beyond building a typesafe (de)serialization library, we also
wanted to automatically generate the the whole service, except for the implementation specific details for an RPC call. We succeded in our goal, but there are a few TODOs before a "release"

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

## TODO
* make serde_xdr a stand-alone crate
* serialization of Option<> types
* boolean discrimant unions
* Test Suite
* fix minor service codegen issues
* better DB setup (might be a rust ecosystem problem)

## Contacts
Please file a github issue if you experience any problems
* [Ben Brittain](https://github.com/cavedweller)
* [Brendon Scheinman](https://github.com/bscheinman)

