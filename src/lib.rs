#![crate_name="free"]
#![crate_type="lib"]

#![license = "MIT"]
#![doc(html_root_url = "http://www.rust-ci.org/epsilonz/free.rs/doc/free/")]

#![feature(overloaded_calls)]
#![feature(phase)]
#![feature(unboxed_closures)]

#[phase(link, plugin)]
extern crate free_macros;

pub mod free;
