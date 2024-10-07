#![no_std]
#![doc = include_str!("../README.md")]

extern crate alloc;

mod string;
mod slice;

pub use string::*;
pub use slice::*;
