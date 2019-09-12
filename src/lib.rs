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
    /// # use java_string::*;
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
    /// assert_eq!("💖", sparkle_heart);
    /// ```
    ///
    /// Incorrect bytes:
    ///
    /// ```
    /// // some invalid bytes, in a vector
    /// let sparkle_heart = vec![0, 159, 146, 150];
    ///
    /// assert!(String::from_utf8(sparkle_heart).is_err());
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
        unsafe { core::str::from_utf8_unchecked(self.data.get_bytes()) }
    }
}

impl DerefMut for JavaString {
    fn deref_mut(&mut self) -> &mut str {
        unsafe { core::str::from_utf8_unchecked_mut(self.data.get_bytes_mut()) }
    }
}

impl PartialOrd for JavaString {
    fn partial_cmp(&self, rhs: &Self) -> Option<core::cmp::Ordering> {
        let jstr: &str = &*self;
        jstr.partial_cmp(rhs)
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
