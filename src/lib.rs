#![crate_name="free"]
#![crate_type="lib"]

#![doc(html_root_url = "http://www.rust-ci.org/epsilonz/free.rs/doc/free/")]

#![feature(phase)]
#![feature(unboxed_closures)]
#![feature(box_syntax)]
#![feature(box_patterns)]

#[macro_use]
extern crate free_macros;

pub mod free;
