//! Hidden module that powers `Digestable` derive macro
//!
//! No stability guarantees for API in this module. It is not supposed to be used by anything else
//! but Digestable derive macro.

/// Compares two byte slices in `const` expression
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

/// Given two arguments, both of which can be either string or slice of bytes, returns `true` if they are equal.
/// Works in `const` expressions.
#[doc(hidden)]
#[macro_export]
macro_rules! const_eq {
    ($a:expr, $b:expr $(,)?) => {{
        let a = $a;
        let b = $b;

        // macro can only be called on str, String, slice of bytes, array of bytes, or Vec of bytes
        $crate::_macros::can_be_compared_in_const(&a);
        $crate::_macros::can_be_compared_in_const(&b);

        // since `AsRef<[u8]>` is not available in `const` expressions, we use the hack below
        // to cast to slice of bytes either string or a slice of bytes, both of which have
        // `.as_ptr()` and `.len()` methods available in `const` context

        // SAFETY: These unsafe block are sound because the pointer and length
        // are obtained from a valid, existing slice or string (note that calls
        // to `can_be_compared_in_const` guarantees that `a` and `b` are either
        // str or slice of bytes)
        let a_bytes: &[u8] = unsafe { core::slice::from_raw_parts(a.as_ptr(), a.len()) };
        // SAFETY: see notice above
        let b_bytes: &[u8] = unsafe { core::slice::from_raw_parts(b.as_ptr(), b.len()) };

        $crate::_macros::const_bytes_eq(a_bytes, b_bytes)
    }};
}

/// Marker which is implemented for types that can be compared by `const_eq` macro
trait Comparable {}

impl Comparable for str {}
#[cfg(feature = "alloc")]
impl Comparable for alloc::string::String {}
impl Comparable for [u8] {}
impl<const N: usize> Comparable for [u8; N] {}
#[cfg(feature = "alloc")]
impl Comparable for alloc::vec::Vec<u8> {}
#[cfg(feature = "alloc")]
impl Comparable for alloc::borrow::Cow<'_, str> {}
#[cfg(feature = "alloc")]
impl Comparable for alloc::borrow::Cow<'_, [u8]> {}
impl<T: Comparable + ?Sized> Comparable for &T {}

/// Statically asserts that `T` implements `Comparable` trait
#[allow(private_bounds)]
pub const fn can_be_compared_in_const<T: Comparable>(_: &T) {}

#[cfg(test)]
mod test {
    const ONE: &[u8] = b"thing";
    const TWO: &str = "different";

    const TRUE: bool = const_eq!(ONE, ONE);
    const FALSE: bool = const_eq!(ONE, TWO);
    #[allow(clippy::assertions_on_constants)]
    const _: () = {
        assert!(TRUE);
        assert!(!FALSE);
    };
}
