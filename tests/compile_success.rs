#![cfg(not(feature = "unit-tests"))]
#![allow(unused)]
#![forbid(unused_unsafe)]
#![cfg_attr(feature = "nightly",
    feature(async_await),
)]

#[macro_use]
extern crate require_unsafe_in_body;

include! {
    "compile_success/mods.rs"
}

fn main () {}
