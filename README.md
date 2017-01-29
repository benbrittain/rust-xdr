# rust-xdr &emsp; [![Build Status](https://travis-ci.org/cavedweller/rust-xdr.svg?branch=master)](https://travis-ci.org/cavedweller/rust-xdr)
**Converts XDR RPC definitions into Rust Services**

### Background
Built primarily during OkCupid Hack Week 2017, the goal of the project was to Proof Of Concept standing up Rust services
that could easily interop with our existing C++ ones. Beyond building a typesafe (de)serialization library, we also
wanted to automatically generate the the whole service, except for the implementation specific details for an RPC call. We succeded in our goal, but there are a few TODOs before a "release"

## TODO
* make serde_xdr a stand-alone crate
* serialization of Option<> types
* boolean discrimant unions
* Test Suite
* fix minor service codegen issues
* better DB setup
* Example service
* Better readme with instructions

### Authors
* [Brendon Scheinman](https://github.com/bscheinman)
* [Ben Brittain](https://github.com/cavedweller)
