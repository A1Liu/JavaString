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
    /// Creates a new, empty, JavaString.
    pub fn new() -> Self {
        Self {
            len: 0,
            data: 1 as *const u8,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn size() {
        assert!(
            std::mem::size_of::<JavaString>() == 2 * std::mem::size_of::<usize>(),
            "Size of JavaString is incorrect!"
        );
    }
}
