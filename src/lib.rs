//! The JavaString uses short string optimizations and a lack of a "capacity"
//! field to reduce size and heap allocations in certain cases.
//!
//! Here's how it works:
//! 1. We store `len`, the length of the string, and `data`, the pointer to the
//!    string itself.
//! 2. We maintain the invariant that `data` is a valid pointer if and only if
//!    it points to something that's aligned to 2 bytes.
//! 3. Now, any time we wanna read the string, we first check the lowest significance
//!    bit on `data`, and use that to see whether or not to dereference it.
//! 4. Since `data` only uses one bit for its flag, we can use the entire lower
//!    order byte for length information when it's interned. We do this with a
//!    bitshift right.
//! 5. When interning, we have `std::mem::size_of::<usize>() * 2 - 1` bytes of space.
//!    On x64, this is 15 bytes, and on 32-bit architectures, this is 7 bytes.

#![allow(dead_code)]
#![cfg_attr(not(test), no_std)]

extern crate alloc;

pub mod raw_string;

use raw_string::RawJavaString;

pub struct JavaString {
    data: RawJavaString,
}

impl JavaString {}
