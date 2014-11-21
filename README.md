# free.rs

Macro-based free monads in Rust

[![build status](https://api.travis-ci.org/epsilonz/free.rs.svg?branch=master)](https://travis-ci.org/epsilonz/free.rs)

## Synopsis

This crate provides the machinery to create a free monad from a signature functor. See [monad](https://crates.io/crates/monad) for instances of concrete monads like `State`, `Reader`, etc.

## Documentation

See the API documentation [here](http://www.rust-ci.org/epsilonz/free.rs/doc/free/).

## Requirements

1.   [Rust](http://www.rust-lang.org/)
2.   [Cargo](http://crates.io/)

You can install both with the following:

```
$ curl -s https://static.rust-lang.org/rustup.sh | sudo sh
```

See [Installing Rust](http://doc.rust-lang.org/guide.html#installing-rust) for further details.

## Usage

```
$ cargo build       ## build library and binary
$ cargo test        ## run tests in ./tests
$ cargo bench       ## run benchmarks in ./benches
```
