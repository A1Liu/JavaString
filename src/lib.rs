/*!
# JavaString
The JavaString uses short string optimizations and a lack of a "capacity"
field to reduce struct size and heap fragmentation in certain cases.

## Features

- Supports String API (very little at the moment but steadily growing)
- Smaller size than standard string (16 vs 24 bytes on 64-bit platforms)
- String interning for up to 15 bytes on 64-bit architectures (or 7 bytes on 32-bit)

## How it works
Here's how it works:

1. We store `len`, the length of the string, and `data`, the pointer to the
   string itself.
2. We maintain the invariant that `data` is a valid pointer if and only if
   it points to something that's aligned to 2 bytes.
3. Now, any time we wanna read the string, we first check the lowest significance
   bit on `data`, and use that to see whether or not to dereference it.
4. Since `data` only uses one bit for its flag, we can use the entire lower
   order byte for length information when it's interned. We do this with a
   bitshift right.
5. When interning, we have `std::mem::size_of::<usize>() * 2 - 1` bytes of space.
   On x64, this is 15 bytes, and on 32-bit architectures, this is 7 bytes.
*/

#![allow(dead_code)]
// #![cfg_attr(not(any(test, docs)), no_std)]

extern crate alloc;

pub mod raw_string;

use core::ops::{Deref, DerefMut};
use raw_string::RawJavaString;

/// JavaString uses short string optimizations and a lack of a "capacity" field
/// to reduce struct size and heap fragmentation in certain cases.
///
/// It allows for mutation, but not for growth without reallocation.
pub struct JavaString {
    data: RawJavaString,
}

impl JavaString {
    /// Creates a new empty `JavaString`.
    ///
    /// Given that the `JavaString` is empty, this will not allocate any initial
    /// buffer.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use java_string::*;
    /// let s = JavaString::new();
    /// ```
    pub const fn new() -> Self {
        Self {
            data: RawJavaString::new(),
        }
    }

    /// Creates a new empty `JavaString`. Included for API compatibility with standard
    /// `String` implementation.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// let mut s = String::with_capacity(10);
    /// ```
    pub const fn with_capacity(_len: usize) -> Self {
        Self::new()
    }
}

impl Deref for JavaString {
    type Target = str;
    fn deref(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(self.data.get_bytes()) }
    }
}

impl DerefMut for JavaString {
    fn deref_mut(&mut self) -> &mut str {
        unsafe { std::str::from_utf8_unchecked_mut(self.data.get_bytes_mut()) }
    }
}
