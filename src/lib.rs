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

impl Drop for JavaString {
    fn drop(&mut self) {
        if self.len() > Self::max_intern_len() {
            use std::alloc::*;
            unsafe {
                dealloc(
                    self.data.as_ptr(),
                    Layout::from_size_align_unchecked(self.len(), 2),
                );
            }
        }
    }
}

impl JavaString {
    /// Returns whether or not this string is interned.
    #[inline]
    fn is_interned(&self) -> bool {
        self.data.as_ptr() as usize % 2 == 1 // Check if the pointer value is even
    }

    /// Returns the maxiumum length of an interned string on the target architecture.
    #[inline]
    const fn max_intern_len() -> usize {
        mem::size_of::<usize>() * 2 - 1
    }

    /// Returns the length of this string.
    pub fn len(&self) -> usize {
        if self.is_interned() {
            ((self.data.as_ptr() as usize as u8) >> 1) as usize
        } else {
            self.len
        }
    }

    pub unsafe fn unsafe_get_bytes(&self) -> &[u8] {
        let len = self.len();
        if len <= Self::max_intern_len() {
            slice::from_raw_parts(&self.len as *const usize as *const u8, len)
        } else {
            slice::from_raw_parts(self.data.as_ptr(), len)
        }
    }

    pub unsafe fn unsafe_get_bytes_mut(&mut self) -> &mut [u8] {
        &mut *(self.unsafe_get_bytes() as *const [u8] as *mut [u8])
    }

    /// Creates a new, empty, JavaString.
    pub const fn new() -> Self {
        Self {
            len: 0,
            data: unsafe { NonNull::new_unchecked(1 as *mut u8) },
        }
    }

    pub unsafe fn from_raw_bytes(bytes: &[u8]) -> Self {
        let len = bytes.len();
        let mut new = Self::new();

        let (write_location, data_pointer_value) = if len <= Self::max_intern_len() {
            (
                &mut new.len as *mut usize as *mut u8,
                len as usize as *mut u8,
            )
        } else {
            use std::alloc::*;
            let ptr = alloc(Layout::from_size_align_unchecked(len, 2));
            (ptr, ptr)
        };

        new.data = NonNull::new_unchecked(data_pointer_value);
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), write_location, len);

        new
    }
}

#[cfg(test)]
mod tests {

    use super::*;

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
