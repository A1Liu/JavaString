use std::fmt;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::slice;

/// String whose contents can't be mutated, just like how Java strings work.
///
/// Operations like mutation are, in all but a select few cases, O(n) time.
/// No amortization here buddy.
///
/// Maintains invariants:
/// 1. Internal pointer is always big endian if valid
/// 2. `data` is only a valid pointer if its big-endian representation is aligned
///    to 2 bytes.
pub struct RawJavaString {
    len: usize,
    data: NonNull<u8>,
}

impl RawJavaString {
    /// Returns the maxiumum length of an interned string on the target architecture.
    #[inline(always)]
    pub const fn max_intern_len() -> usize {
        mem::size_of::<usize>() * 2 - 1
    }

    /// Returns whether or not this string is interned.
    #[inline(always)]
    pub fn is_interned(&self) -> bool {
        self.data.as_ptr() as usize % 2 == 1 // Check if the pointer value is even
    }

    #[inline(always)]
    pub fn read_ptr(&self) -> *mut u8 {
        usize::from_be(self.data.as_ptr() as usize) as *mut u8
    }

    #[inline(always)]
    pub fn write_ptr(&mut self, ptr: *mut u8) {
        self.data = NonNull::new(usize::to_be(ptr as usize) as *mut u8)
            .expect("Wrote null to JavaString pointer.");
    }

    #[inline(always)]
    pub unsafe fn write_ptr_unchecked(&mut self, ptr: *mut u8) {
        self.data = NonNull::new_unchecked(usize::to_be(ptr as usize) as *mut u8);
    }

    /// Returns the length of this string.
    #[inline(always)]
    pub fn len(&self) -> usize {
        if self.is_interned() {
            (self.read_ptr() as usize as u8 >> 1) as usize
        } else {
            self.len
        }
    }

    /// Returns the current memory layout of this object. If None, then we're looking
    /// at an interned string.
    #[inline(always)]
    fn get_memory_layout(&self) -> Option<core::alloc::Layout> {
        if self.len() > Self::max_intern_len() {
            Some(unsafe { core::alloc::Layout::from_size_align_unchecked(self.len(), 2) })
        } else {
            None
        }
    }

    pub fn get_bytes(&self) -> &[u8] {
        let (ptr, len) = if self.is_interned() {
            let len = (self.read_ptr() as usize as u8 >> 1) as usize;
            let ptr = (&self.len) as *const usize as *const u8 as *mut u8;
            (ptr, len)
        } else {
            (self.data.as_ptr(), self.len)
        };

        unsafe { slice::from_raw_parts(ptr, len) }
    }

    #[inline]
    pub fn get_bytes_mut(&mut self) -> &mut [u8] {
        unsafe { &mut *(self.get_bytes() as *const [u8] as *mut [u8]) }
    }

    /// Creates a new, empty, RawJavaString.
    pub const fn new() -> Self {
        Self {
            len: 0,
            data: unsafe { NonNull::new_unchecked(1 as *mut u8) },
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut new = Self::new();
        let len = bytes.len();

        let (write_location, data_pointer_value) = if len <= Self::max_intern_len() {
            (
                &mut new.len as *mut usize as *mut u8,
                len as usize as *mut u8,
            )
        } else {
            use std::alloc::*;
            let ptr = unsafe { alloc(Layout::from_size_align_unchecked(len, 2)) };
            (ptr, ptr)
        };

        unsafe {
            new.data = NonNull::new_unchecked(data_pointer_value);
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), write_location, len);
        }

        new
    }

    #[inline(always)]
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        *self = Self::from_bytes(bytes);
    }
}

impl Drop for RawJavaString {
    fn drop(&mut self) {
        if !self.is_interned() {
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

impl Clone for RawJavaString {
    fn clone(&self) -> Self {
        Self::from_bytes(self.get_bytes())
    }
}

impl fmt::Debug for RawJavaString {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(formatter, "JavaString {:?}", &self.get_bytes())
    }
}

impl Deref for RawJavaString {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        self.get_bytes()
    }
}

impl DerefMut for RawJavaString {
    fn deref_mut(&mut self) -> &mut [u8] {
        self.get_bytes_mut()
    }
}
