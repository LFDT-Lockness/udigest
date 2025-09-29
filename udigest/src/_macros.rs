//! Hidden module that powers `Digestable` derive macro
//!
//! No stability guarantees for API in this module. It is not supposed to be used by anything else
//! but Digestable derive macro.

/// Compares two byte slices in constant time
///
/// Comparing slices via `==` operator is not stable yet
pub const fn const_bytes_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

/// Given two bytestring or strings, asserts that they are not equal. Throws compilation error if they are.
///
/// Macro accepts three arguments: two strings/bytestrings to compare, and an error message. First two arguments
/// can be both string, both bytestring, or one string and one bytestring in any order. They are coerced into
/// bytestrings before comparison.
#[doc(hidden)]
#[macro_export]
macro_rules! const_assert_neq {
    ($a:expr, $b:expr, $($msg:tt)+) => {
        const _: () = {
            // These unsafe block are sound because the pointer and length
            // are obtained from a valid, existing slice (`$a` or `$b`).
            let a_bytes: &[u8] = unsafe {
                // Both `&str` and `&[u8]` have `.as_ptr()` and `.len()` as const fns.
                core::slice::from_raw_parts($a.as_ptr(), $a.len())
            };
            let b_bytes: &[u8] = unsafe { core::slice::from_raw_parts($b.as_ptr(), $b.len()) };

            if $crate::_macros::const_bytes_eq(a_bytes, b_bytes) {
                panic!($($msg)+)
            }
        };
    };
}

#[cfg(test)]
mod test {
    const ONE: &[u8] = b"thing";
    const TWO: &str = "different";
    crate::const_assert_neq!(ONE, TWO, "they are equal");
}
