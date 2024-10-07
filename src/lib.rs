#![no_std]
#![doc = include_str!("../README.md")]

extern crate alloc;

mod string;
mod bytes;

pub use string::*;
pub use bytes::*;
