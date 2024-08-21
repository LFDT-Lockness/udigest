//! Provides utilities for custom digesting rules
//!
//! It's supposed to be used in a pair with derive proc macro and `as` attribute.
//! For instance, it can be used to digest a hash map "as a btree map":
//!   ```rust
//!   #[derive(udigest::Digestable)]
//!   pub struct Attributes(
//!       #[udigest(as = std::collections::BTreeMap<_, udigest::Bytes>)]
//!       std::collections::HashMap<String, Vec<u8>>,
//!   );
//!   ```
//!
//! See more examples in [macro@Digestable] macro docs.

use crate::{encoding, Buffer, Digestable};

/// Custom rule for digesting an instance of `T`
pub trait DigestAs<T: ?Sized> {
    /// Digests `value`
    fn digest_as<B: Buffer>(value: &T, encoder: encoding::EncodeValue<B>);
}

impl<T, U> DigestAs<&T> for &U
where
    U: DigestAs<T>,
{
    fn digest_as<B: Buffer>(value: &&T, encoder: encoding::EncodeValue<B>) {
        U::digest_as(*value, encoder)
    }
}

/// Stores `T`, digests it using `DigestAs<T>` implementation of `U`
pub struct As<T, U> {
    value: T,
    _rule: core::marker::PhantomData<U>,
}

impl<T, U> As<T, U> {
    /// Constructor
    pub const fn new(value: T) -> Self {
        Self {
            value,
            _rule: core::marker::PhantomData,
        }
    }

    /// Returns stored value
    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<T, U> Digestable for As<T, U>
where
    U: DigestAs<T>,
{
    fn unambiguously_encode<B: Buffer>(&self, encoder: encoding::EncodeValue<B>) {
        U::digest_as(&self.value, encoder)
    }
}

impl<T: core::cmp::PartialEq, U> core::cmp::PartialEq for As<T, U> {
    fn eq(&self, other: &Self) -> bool {
        self.value.eq(&other.value)
    }
}
impl<T: core::cmp::Eq, U> core::cmp::Eq for As<T, U> {}
impl<T: core::cmp::PartialOrd, U> core::cmp::PartialOrd for As<T, U> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}
impl<T: core::cmp::Ord, U> core::cmp::Ord for As<T, U> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

/// Digests any type `T` via its own implementation of [`Digestable`] trait
pub struct Same;

impl<T> DigestAs<T> for Same
where
    T: Digestable,
{
    fn digest_as<B: Buffer>(value: &T, encoder: encoding::EncodeValue<B>) {
        value.unambiguously_encode(encoder)
    }
}

pub use crate::Bytes;

impl<T> DigestAs<T> for Bytes
where
    T: AsRef<[u8]> + ?Sized,
{
    fn digest_as<B: Buffer>(value: &T, encoder: encoding::EncodeValue<B>) {
        encoder.encode_leaf_value(value.as_ref())
    }
}

impl<T, U> DigestAs<Option<T>> for Option<U>
where
    U: DigestAs<T>,
{
    fn digest_as<B: Buffer>(value: &Option<T>, encoder: encoding::EncodeValue<B>) {
        value
            .as_ref()
            .map(As::<&T, &U>::new)
            .unambiguously_encode(encoder)
    }
}

impl<T1, T2, E1, E2> DigestAs<Result<T1, E1>> for Result<T2, E2>
where
    T2: DigestAs<T1>,
    E2: DigestAs<E1>,
{
    fn digest_as<B: Buffer>(value: &Result<T1, E1>, encoder: encoding::EncodeValue<B>) {
        value
            .as_ref()
            .map(As::<&T1, &T2>::new)
            .map_err(As::<&E1, &E2>::new)
            .unambiguously_encode(encoder)
    }
}

impl<T, U> DigestAs<[T]> for [U]
where
    U: DigestAs<T>,
{
    fn digest_as<B: Buffer>(value: &[T], encoder: encoding::EncodeValue<B>) {
        crate::unambiguously_encode_iter(encoder, value.iter().map(As::<&T, &U>::new))
    }
}

impl<T, U, const N: usize> DigestAs<[T; N]> for [U; N]
where
    U: DigestAs<T>,
{
    fn digest_as<B: Buffer>(value: &[T; N], encoder: encoding::EncodeValue<B>) {
        crate::unambiguously_encode_iter(encoder, value.iter().map(As::<&T, &U>::new))
    }
}

#[cfg(feature = "alloc")]
impl<T, U> DigestAs<alloc::vec::Vec<T>> for alloc::vec::Vec<U>
where
    U: DigestAs<T>,
{
    fn digest_as<B: Buffer>(value: &alloc::vec::Vec<T>, encoder: encoding::EncodeValue<B>) {
        crate::unambiguously_encode_iter(encoder, value.iter().map(As::<&T, &U>::new))
    }
}
#[cfg(feature = "alloc")]
impl<T, U> DigestAs<alloc::collections::LinkedList<T>> for alloc::collections::LinkedList<U>
where
    U: DigestAs<T>,
{
    fn digest_as<B: Buffer>(
        value: &alloc::collections::LinkedList<T>,
        encoder: encoding::EncodeValue<B>,
    ) {
        crate::unambiguously_encode_iter(encoder, value.iter().map(As::<&T, &U>::new))
    }
}
#[cfg(feature = "alloc")]
impl<T, U> DigestAs<alloc::collections::VecDeque<T>> for alloc::collections::VecDeque<U>
where
    U: DigestAs<T>,
{
    fn digest_as<B: Buffer>(
        value: &alloc::collections::VecDeque<T>,
        encoder: encoding::EncodeValue<B>,
    ) {
        crate::unambiguously_encode_iter(encoder, value.iter().map(As::<&T, &U>::new))
    }
}
#[cfg(feature = "alloc")]
impl<T, U> DigestAs<alloc::collections::BTreeSet<T>> for alloc::collections::BTreeSet<U>
where
    U: DigestAs<T>,
{
    fn digest_as<B: Buffer>(
        value: &alloc::collections::BTreeSet<T>,
        encoder: encoding::EncodeValue<B>,
    ) {
        crate::unambiguously_encode_iter(encoder, value.iter().map(As::<&T, &U>::new))
    }
}

#[cfg(feature = "alloc")]
impl<K1, K2, V1, V2> DigestAs<alloc::collections::BTreeMap<K1, V1>>
    for alloc::collections::BTreeMap<K2, V2>
where
    K2: DigestAs<K1>,
    V2: DigestAs<V1>,
{
    fn digest_as<B: Buffer>(
        value: &alloc::collections::BTreeMap<K1, V1>,
        encoder: encoding::EncodeValue<B>,
    ) {
        crate::unambiguously_encode_iter(
            encoder,
            value
                .iter()
                .map(|(key, value)| (As::<&K1, &K2>::new(key), As::<&V1, &V2>::new(value))),
        )
    }
}

/// Digests `HashMap` by transforming it into `BTreeMap`
#[cfg(feature = "std")]
impl<K1, K2, V1, V2> DigestAs<std::collections::HashMap<K1, V1>>
    for alloc::collections::BTreeMap<K2, V2>
where
    K2: DigestAs<K1>,
    V2: DigestAs<V1>,
    K1: core::cmp::Ord,
{
    fn digest_as<B: Buffer>(
        value: &std::collections::HashMap<K1, V1>,
        encoder: encoding::EncodeValue<B>,
    ) {
        let ordered_map = value
            .iter()
            .map(|(key, value)| (As::<&K1, &K2>::new(key), As::<&V1, &V2>::new(value)))
            .collect::<alloc::collections::BTreeMap<_, _>>();

        // ordered map has deterministic order, so we can reproducibly hash it
        ordered_map.unambiguously_encode(encoder)
    }
}

#[cfg(feature = "alloc")]
impl<T, U> DigestAs<alloc::boxed::Box<T>> for alloc::boxed::Box<U>
where
    U: DigestAs<T>,
{
    fn digest_as<B: Buffer>(value: &alloc::boxed::Box<T>, encoder: encoding::EncodeValue<B>) {
        U::digest_as(value, encoder)
    }
}

#[cfg(feature = "alloc")]
impl<T, U> DigestAs<alloc::rc::Rc<T>> for alloc::rc::Rc<U>
where
    U: DigestAs<T>,
{
    fn digest_as<B: Buffer>(value: &alloc::rc::Rc<T>, encoder: encoding::EncodeValue<B>) {
        U::digest_as(value, encoder)
    }
}

#[cfg(feature = "alloc")]
impl<T, U> DigestAs<alloc::sync::Arc<T>> for alloc::sync::Arc<U>
where
    U: DigestAs<T>,
{
    fn digest_as<B: Buffer>(value: &alloc::sync::Arc<T>, encoder: encoding::EncodeValue<B>) {
        U::digest_as(value, encoder)
    }
}

#[cfg(feature = "alloc")]
impl<'a, T, U> DigestAs<alloc::borrow::Cow<'a, T>> for alloc::borrow::Cow<'a, U>
where
    U: DigestAs<T> + alloc::borrow::ToOwned,
    T: alloc::borrow::ToOwned + ?Sized + 'a,
{
    fn digest_as<B: Buffer>(value: &alloc::borrow::Cow<'a, T>, encoder: encoding::EncodeValue<B>) {
        U::digest_as(value.as_ref(), encoder)
    }
}

#[cfg(feature = "alloc")]
impl<'a, T, U> DigestAs<alloc::borrow::Cow<'a, T>> for &'a U
where
    U: DigestAs<T>,
    T: alloc::borrow::ToOwned + ?Sized + 'a,
{
    fn digest_as<B: Buffer>(value: &alloc::borrow::Cow<'a, T>, encoder: encoding::EncodeValue<B>) {
        U::digest_as(value.as_ref(), encoder)
    }
}
