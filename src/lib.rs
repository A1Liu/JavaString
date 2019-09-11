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

mod raw_string;

/*
use std::mem;
use std::ptr::NonNull;
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
    data: NonNull<u8>,
}

impl JavaString {
    /// Returns whether or not this string is interned.
    #[inline]
    fn is_interned(&self) -> bool {
        self.data.as_ptr() as usize % 2 == 1 // Check if the pointer value is even
    }

    fn read_ptr(&self) -> *mut u8 {
        usize::from_be(self.data.as_ptr() as usize) as *mut u8
    }

    pub unsafe fn unsafe_get_bytes_mut(&mut self) -> &mut [u8] {
        &mut *(self.unsafe_get_bytes() as *const [u8] as *mut [u8])
    }


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn new_does_not_use_heap() {
        let string = JavaString::new();
        assert!(
            string.is_interned(),
            "Size of Option<JavaString> is incorrect!"
        );
    }

    #[test]
    fn option_size() {
        assert!(
            mem::size_of::<Option<JavaString>>() == 2 * mem::size_of::<usize>(),
            "Size of Option<JavaString> is incorrect!"
        );
    }

    #[test]
    fn size() {
        assert!(
            mem::size_of::<JavaString>() == 2 * mem::size_of::<usize>(),
            "Size of JavaString is incorrect!"
        );
    }
}
*/
