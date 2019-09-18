/*!
# JavaString
`JavaString` uses short string optimizations and a lack of a "capacity" field to
reduce struct size and heap fragmentation in certain cases.

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

## API Compatibility and Acknoledgements
The API of `JavaString` is 100% compatible with the standard `String`. Additionally,
we use much of the same documentation, as it's really well written. We'd like to
give credit to the documentation of standard `String`.
*/

#![allow(dead_code)]

extern crate alloc;
extern crate serde;
pub mod raw_string;

use core::fmt;
use core::ops::{Deref, DerefMut};
use raw_string::RawJavaString;

/// A UTF-8 encoded, immutable string.
///
/// `JavaString` uses short string optimizations and a lack of a "capacity" field
/// to reduce struct size and heap fragmentation in certain cases.
///
/// It allows for character, but not for growth without reallocation.
#[derive(Clone, PartialEq, Eq)]
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
    /// # use jstring::*;
    /// let s = JavaString::new();
    /// ```
    pub const fn new() -> Self {
        Self {
            data: RawJavaString::new(),
        }
    }

    /// Included for API compatibility with standard `String` implementation.
    /// Creates a new empty `JavaString`.
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

    /// Converts a slice or compatible container of bytes to a `String`.
    ///
    /// A string slice (`&str`) is made of bytes (`u8`), and a slice of bytes
    /// (`[u8]`) is made of bytes, so this function converts between the
    /// two. Not all byte slices are valid `JavaString`s, however: `JavaString`
    /// requires that it is valid UTF-8. `from_utf8()` checks to ensure that
    /// the bytes are valid UTF-8, and then does the conversion.
    ///
    /// If you are sure that the byte slice is valid UTF-8, and you don't want
    /// to incur the overhead of the validity check, there is an unsafe version
    /// of this function, [`from_utf8_unchecked`], which has the same behavior
    /// but skips the check.
    ///
    /// If you need a `&str` instead of a `String`, consider
    /// `core::str::from_utf8`.
    ///
    /// The inverse of this method is [`as_bytes`].
    ///
    /// # Errors
    ///
    /// Returns `Err` if the slice is not UTF-8 with a description as to why the
    /// provided bytes are not UTF-8. The vector you moved in is also included.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use jstring::*;
    /// // some bytes, in a vector
    /// let sparkle_heart = vec![240, 159, 146, 150];
    ///
    /// // We know these bytes are valid, so we'll use `unwrap()`.
    /// let sparkle_heart = JavaString::from_utf8(sparkle_heart).unwrap();
    ///
    /// assert_eq!(sparkle_heart, "💖");
    /// ```
    ///
    /// Incorrect bytes:
    ///
    /// ```
    /// # use jstring::JavaString;
    ///
    /// // some invalid bytes, in a vector
    /// let sparkle_heart = vec![0, 159, 146, 150];
    ///
    /// assert!(JavaString::from_utf8(sparkle_heart).is_err());
    /// ```
    ///
    /// See the docs for `core::str::Utf8Error` for more details on what you can do
    /// with this error.
    ///
    /// [`from_utf8_unchecked`]: struct.String.html#method.from_utf8_unchecked
    /// [`as_bytes`]: struct.String.html#method.as_bytes
    pub fn from_utf8(bytes: impl Deref<Target = [u8]>) -> Result<Self, core::str::Utf8Error> {
        let raw_str = RawJavaString::from_bytes(bytes);
        core::str::from_utf8(raw_str.get_bytes())?;
        Ok(Self { data: raw_str })
    }

    /// Included for API compatibility.
    ///
    /// Calls to the `String` member function of the same name.
    pub fn from_utf8_lossy<'a>(v: &'a [u8]) -> alloc::borrow::Cow<'a, str> {
        String::from_utf8_lossy(v)
    }

    /// Decode a UTF-16 encoded vector `v` into a `JavaString`, returning `Err`
    /// if `v` contains any invalid data.
    ///
    /// May cause memory leaks depending on how your allocator is configured.
    pub fn from_utf16(v: &[u16]) -> Result<Self, alloc::string::FromUtf16Error> {
        Ok(String::from_utf16(v)?.into())
    }

    /// Converts a vector of bytes to a `JavaString` without checking that the string
    /// contains valid UTF-8.
    pub unsafe fn from_utf8_unchecked(bytes: Vec<u8>) -> JavaString {
        String::from_utf8_unchecked(bytes).into()
    }

    /// Converts a `JavaString` into a byte vector.
    ///
    /// This consumes the `String`, so we do not need to copy its contents.
    pub fn into_bytes(self) -> Vec<u8> {
        self.data.into_bytes()
    }

    /// Extracts a string slice containing the entire `JavaString`.
    pub fn as_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(self.data.get_bytes()) }
    }

    /// Extracts a mutable string slice containing the entire `JavaString`.
    pub fn as_mut_str(&mut self) -> &mut str {
        unsafe { core::str::from_utf8_unchecked_mut(self.data.get_bytes_mut()) }
    }

    /// Appends a given string slice onto the end of this `JavaString`.
    pub fn push_str(&mut self, string: &str) {
        let sl = vec![self.as_bytes(), string.as_bytes()];
        self.data = RawJavaString::from_bytes_array(sl);
    }

    /// Returns this `JavaString`'s capacity, in bytes. Always returns the
    /// same value as `self.len()`.
    pub fn capacity(&self) -> usize {
        self.len()
    }

    /// Included for API compatibility with standard `String` implementation.
    ///
    /// Does nothing.
    pub fn reserve(&mut self, _additional: usize) {}

    /// Included for API compatibility with standard `String` implementation.
    ///
    /// Does nothing.
    pub fn reserve_exact(&mut self, _additional: usize) {}

    /// Included for API compatibility with standard `String` implementation.
    ///
    /// Does nothing.
    #[cfg(nightly)]
    pub fn try_reserve(
        &mut self,
        _additional: usize,
    ) -> Result<(), std::collections::CollectionAllocErr> {
        Ok(())
    }
}

impl fmt::Display for JavaString {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let jstr: &str = &*self;
        jstr.fmt(formatter)
    }
}

impl fmt::Debug for JavaString {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let jstr: &str = &*self;
        jstr.fmt(formatter)
    }
}

impl Deref for JavaString {
    type Target = str;
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl DerefMut for JavaString {
    fn deref_mut(&mut self) -> &mut str {
        self.as_mut_str()
    }
}

impl From<String> for JavaString {
    fn from(string: String) -> Self {
        Self {
            data: RawJavaString::from_byte_vec(string.into_bytes()),
        }
    }
}

impl PartialOrd for JavaString {
    fn partial_cmp(&self, rhs: &Self) -> Option<core::cmp::Ordering> {
        let jstr: &str = &*self;
        jstr.partial_cmp(rhs)
    }
}

impl<'a> PartialEq<str> for &'a JavaString {
    fn eq(&self, rhs: &str) -> bool {
        let jstr: &str = &*self;
        jstr.eq(rhs)
    }
}

impl<'a> PartialEq<&'a str> for JavaString {
    fn eq(&self, rhs: &&'a str) -> bool {
        let jstr: &str = &*self;
        let jstr_2 = &jstr;
        jstr_2.eq(rhs)
    }
}

impl PartialEq<str> for JavaString {
    fn eq(&self, rhs: &str) -> bool {
        let jstr: &str = &*self;
        jstr.eq(rhs)
    }
}

impl Ord for JavaString {
    fn cmp(&self, rhs: &Self) -> core::cmp::Ordering {
        let jstr: &str = &*self;
        jstr.cmp(rhs)
    }
}

impl serde::Serialize for JavaString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let jstr: &str = &*self;
        jstr.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for JavaString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self {
            data: RawJavaString::from_byte_vec(String::deserialize(deserializer)?.into_bytes()),
        })
    }
}
