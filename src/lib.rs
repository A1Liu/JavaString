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

use std::mem;
use std::slice;

/// String whose contents can't be mutated, just like how Java strings work.
///
/// Operations like mutation are, in all but a select few cases, O(n) time.
/// No amortization here buddy.
#[repr(C)]
pub struct JavaString {
    /// Length of data.
    len: usize,
    /// Pointer to data. When not aligned to 2 bytes, the entire structure is
    /// used as an inline string, with the last byte used to hold length information.
    data: *const u8,
}

impl JavaString {
    /// Returns whether or not this string is interned.
    #[inline]
    fn is_interned(&self) -> bool {
        self.data as usize % 2 == 1 // Check if the pointer value is even
    }

    /// Returns the maxiumum length of an interned string on the target architecture.
    const fn max_intern_len() -> usize {
        mem::size_of::<usize>() * 2 - 1
    }

    /// Returns the length of this string.
    pub fn len(&self) -> usize {
        if self.is_interned() {
            ((self.data as usize as u8) >> 1) as usize
        } else {
            self.len
        }
    }

    pub unsafe fn unsafe_get_str(&self) -> &str {
        let len = self.len();
        let u8_slice = if len <= Self::max_intern_len() {
            slice::from_raw_parts(&self.len as *const usize as *const u8, len)
        } else {
            slice::from_raw_parts(self.data, len)
        };
        std::str::from_utf8_unchecked(u8_slice)
    }

    pub unsafe fn unsafe_get_str_mut(&mut self) -> &mut str {
        &mut *(self.unsafe_get_str() as *const str as *mut str)
    }

    /// Creates a new, empty, JavaString.
    pub const fn new() -> Self {
        Self {
            len: 0,
            data: 1 as *const u8,
        }
    }

    /// Creates a new JavaString from an &str
    pub fn from_str(_string: &str) -> Self {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn size() {
        assert!(
            mem::size_of::<JavaString>() == 2 * mem::size_of::<usize>(),
            "Size of JavaString is incorrect!"
        );
    }
}
