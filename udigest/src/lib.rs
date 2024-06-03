//! ![License](https://img.shields.io/crates/l/udigest.svg)
//! [![Docs](https://docs.rs/udigest/badge.svg)](https://docs.rs/udigest)
//! [![Crates io](https://img.shields.io/crates/v/udigest.svg)](https://crates.io/crates/udigest)
//!
//! # Unambiguously digest structured data
//!
//! `udigest` provides utilities for unambiguous hashing the structured data. Structured
//! data can be anything that implements [`Digestable`] trait:
//!
//! * `str`, `String`, `CStr`, `CString`
//! * Integers:
//!   `i8`, `i16`, `i32`, `i64`, `i128`,
//!   `u8`, `u16`, `u32`, `u64`, `u128`,
//!   `char`
//! * Containers: `Box`, `Arc`, `Rc`, `Cow`, `Option`, `Result`
//! * Collections: arrays, slices, `Vec`, `LinkedList`, `VecDeque`, `BTreeSet`, `BTreeMap`
//!
//! The trait is intentionally not implemented for certain types:
//!
//! * `HashMap`, `HashSet` as they can not be traversed in deterministic order
//! * `usize`, `isize` as their byte size varies on different platforms
//!
//! The `Digestable` trait can be implemented for the struct using [a macro](derive@Digestable):
//! ```rust
//! #[derive(udigest::Digestable)]
//! struct Person {
//!     name: String,
//!     job_title: String,   
//! }
//! let alice = Person {
//!     name: "Alice".into(),
//!     job_title: "cryptographer".into(),
//! };
//!
//! let hash = udigest::hash::<sha2::Sha256, _>(&alice);
//! ```
//!
//! The crate intentionally does not try to follow any existing standards for unambiguous
//! encoding. The format for encoding was designed specifically for `udigest` to provide
//! a better usage experience in Rust. The details of encoding format can be found in
//! [`encoding` module](encoding).
//!
//! ## Features
//! * `digest` enables support of hash functions that implement [`digest`] traits \
//!    If feature is not enabled, the crate is still usable via [`Digestable`] trait that
//!    generically implements unambiguous encoding
//! * `std` implements `Digestable` trait for types in standard library
//! * `alloc` implements `Digestable` trait for type in `alloc` crate
//! * `derive` enables `Digestable` proc macro

#![no_std]
#![forbid(missing_docs)]
#![cfg_attr(not(test), forbid(unused_crate_dependencies))]
#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

#[cfg(feature = "alloc")]
extern crate alloc;

pub use encoding::Buffer;

/// Derives a [`Digestable`] trait
///
/// Works with any struct and enum. Requires each field to be [`Digestable`] or, alternatively,
/// it can be specified how to digest a field via attributes.
///
/// ### Example
/// ```rust
/// #[derive(udigest::Digestable)]
/// #[udigest(tag = "udigest.example.Person.v1")]
/// struct Person {
///     name: String,
///     #[udigest(rename = "job")]
///     job_title: String,
/// }
/// ```
///
/// ### Notes
/// * Field and variant names are mixed into the hash, so changing the field/variant
///   name will result into a different hash even if field values are the same \
///   Field name used in hashing can be changed using `#[udigest(rename = "...")]`
///   attribute.
/// * Fields are hashed exactly in the order in which they are defined, so changing
///   the fields order will change the hashing
/// * Hashing different types, generally, may result into the same hash if they have
///   the same byte encoding. For instance:
///   ```rust
///   #[derive(udigest::Digestable, Debug)]
///   struct PersonA { name: String }
///   #[derive(udigest::Digestable, Debug)]
///   struct PersonB { #[udigest(as_bytes)] name: Vec<u8> }
///   
///   let person_a = PersonA{ name: "Alice".into() };
///   let person_b = PersonB{ name: b"Alice".to_vec() };
///   
///   assert_eq!(
///       udigest::hash::<sha2::Sha256, _>(&person_a),
///       udigest::hash::<sha2::Sha256, _>(&person_b),
///   )
///   ```
///   `person_a` and `person_b` have exactly the same hash as they have the same bytes
///   representation. If you need to distinguish them, you can specify a domain-separation
///   tag using `#[udigest(tag = "...")]` attribute.
///
/// ### Container attributes
/// * `#[udigest(tag = "...")]` \
///   Specifies a domain separation tag for the container. The tag makes bytes representation of one type
///   distinguishable from another type even if they have exactly the same fields but different tags. The
///   tag may include a version to distinguish hashes of the same structures across different versions.
/// * `#[udigest(bound = "...")]` \
///   Specifies which generic bounds to use. By default, `udigest` will generate `T: Digestable` bound per
///   each generic `T`. This behavior can be overridden via this attribute. Example:
///   ```rust
///   #[derive(udigest::Digestable)]
///   #[udigest(bound = "")]
///   struct Foo<T> {
///       field1: String,
///       field2: std::marker::PhantomData<T>,
///   }
///   ```
/// * `#[udigest(root = ...)]` \
///   Specifies a path to `udigest` library. Default: `udigest`.
///   ```rust
///   use ::udigest as udigest2;
///   # mod udigest {}
///   #[derive(udigest2::Digestable)]
///   #[udigest(root = udigest2)]
///   struct Person {
///       name: String,
///       job_title: String,
///   }
///   ```
///
/// ### Field attributes
/// * `#[udigest(as_bytes)]` \
///   Tells that the field should be treated as a bytestring. Field must implement
///   `AsRef<[u8]>`.
///   ```rust
///   #[derive(udigest::Digestable)]
///   struct Data(#[udigest(as_bytes)] Vec<u8>);
///   ```
/// * `#[udigest(as_bytes = ...)]` \
///   Tells that the field should be converted to a bytestring. Uses specified function
///   that accepts a reference of the field value, and returns `impl AsRef<[u8]>`
///   ```rust
///   struct Data(Vec<u8>);
///   impl Data {
///       fn as_bytes(&self) -> &[u8] {
///           &self.0
///       }
///   }
///   
///   #[derive(udigest::Digestable)]
///   struct Packet {
///       seq: u16,
///       #[udigest(as_bytes = Data::as_bytes)]
///       data: Data
///   }
///   ```
/// * `#[udigest(with = ...)]` \
///   Can be used to override the field encoding. Accepts as input a function with a signature:
///   ```rust,no_run
///   # type T = String;
///   fn encoder<B>(value: &T, encoder: udigest::encoding::EncodeValue<B>)
///   where
///       B: udigest::Buffer
///   # {}
///   ```
///   Example:
///   ```rust
///   #[derive(udigest::Digestable)]
///   pub struct User {
///       name: String,
///       // `Instant` encoding is not provided by `udigest` crate, but it
///       // can be manually provided
///       #[udigest(with = encode_instant)]
///       created_at: std::time::Instant,
///   }
///   fn encode_instant<B: udigest::Buffer>(
///       instant: &std::time::Instant,
///       encoder: udigest::encoding::EncodeValue<B>
///   ) {
///       todo!()
///   }
///   ```
/// * `#[udigest(rename = "...")]` \
///   Specifies another name to use for the field. As field name gets mixed into the hash,
///   changing the field name will change the hash. Sometimes, it may be required to change
///   the field name without affecting the hashing, so this attribute can be used
///   ```rust
///   #[derive(udigest::Digestable)]
///   struct Person {
///       name: String,
///       #[udigest(rename = "job")]
///       job_title: String,
///   }
///   ```
/// * `#[udigest(skip)]` \
///   Removes this field from hashing process
#[cfg(feature = "derive")]
pub use udigest_derive::Digestable;

pub mod encoding;

/// Digests a structured `value` using fixed-output hash function (like sha2-256)
#[cfg(feature = "digest")]
pub fn hash<D: digest::Digest, T: Digestable>(value: &T) -> digest::Output<D> {
    let mut hash = encoding::BufferDigest(D::new());
    value.unambiguously_encode(encoding::EncodeValue::new(&mut hash));
    hash.0.finalize()
}

/// Digests a list of structured data using fixed-output hash function (like sha2-256)
#[cfg(feature = "digest")]
pub fn hash_iter<D: digest::Digest>(
    iter: impl IntoIterator<Item = impl Digestable>,
) -> digest::Output<D> {
    let mut hash = encoding::BufferDigest(D::new());
    let mut encoder = encoding::EncodeList::new(&mut hash).with_tag(b"udigest.list");
    for value in iter {
        let item_encoder = encoder.add_item();
        value.unambiguously_encode(item_encoder);
    }
    encoder.finish();
    hash.0.finalize()
}

/// Digests a structured `value` using extendable-output hash function (like shake-256)
#[cfg(feature = "digest")]
pub fn hash_xof<D, T>(value: &T) -> D::Reader
where
    T: Digestable,
    D: Default + digest::Update + digest::ExtendableOutput,
{
    let mut hash = encoding::BufferUpdate(D::default());
    value.unambiguously_encode(encoding::EncodeValue::new(&mut hash));
    hash.0.finalize_xof()
}

/// Digests a list of structured data using extendable-output hash function (like shake-256)
#[cfg(feature = "digest")]
pub fn hash_xof_iter<D>(iter: impl IntoIterator<Item = impl Digestable>) -> D::Reader
where
    D: Default + digest::Update + digest::ExtendableOutput,
{
    let mut hash = encoding::BufferUpdate(D::default());
    let mut encoder = encoding::EncodeList::new(&mut hash).with_tag(b"udigest.list");
    for value in iter {
        let item_encoder = encoder.add_item();
        value.unambiguously_encode(item_encoder);
    }
    encoder.finish();
    hash.0.finalize_xof()
}

/// Digests a structured `value` using variable-output hash function (like blake2b)
#[cfg(feature = "digest")]
pub fn hash_vof<D, T>(value: &T, out: &mut [u8]) -> Result<(), digest::InvalidOutputSize>
where
    T: Digestable,
    D: digest::VariableOutput + digest::Update,
{
    let mut hash = encoding::BufferUpdate(D::new(out.len())?);
    value.unambiguously_encode(encoding::EncodeValue::new(&mut hash));
    hash.0
        .finalize_variable(out)
        .map_err(|_| digest::InvalidOutputSize)
}

/// Digests a list of structured data using variable-output hash function (like blake2b)
#[cfg(feature = "digest")]
pub fn hash_vof_iter<D>(
    iter: impl IntoIterator<Item = impl Digestable>,
    out: &mut [u8],
) -> Result<(), digest::InvalidOutputSize>
where
    D: digest::VariableOutput + digest::Update,
{
    let mut hash = encoding::BufferUpdate(D::new(out.len())?);
    let mut encoder = encoding::EncodeList::new(&mut hash).with_tag(b"udigest.list");
    for value in iter {
        let item_encoder = encoder.add_item();
        value.unambiguously_encode(item_encoder);
    }
    encoder.finish();
    hash.0
        .finalize_variable(out)
        .map_err(|_| digest::InvalidOutputSize)
}

/// A value that can be unambiguously digested
pub trait Digestable {
    /// Unambiguously encodes the value
    fn unambiguously_encode<B: Buffer>(&self, encoder: encoding::EncodeValue<B>);
}

impl<T: Digestable + ?Sized> Digestable for &T {
    fn unambiguously_encode<B: Buffer>(&self, encoder: encoding::EncodeValue<B>) {
        (*self).unambiguously_encode(encoder)
    }
}

/// Wrapper for a bytestring
///
/// Wraps any bytestring that `impl AsRef<[u8]>` and provides [`Digestable`] trait implementation
pub struct Bytes<T: ?Sized>(pub T);

impl<T: AsRef<[u8]> + ?Sized> Digestable for Bytes<T> {
    fn unambiguously_encode<B: Buffer>(&self, encoder: encoding::EncodeValue<B>) {
        self.0.as_ref().unambiguously_encode(encoder)
    }
}

macro_rules! digestable_integers {
    ($($type:ty),*) => {$(
        impl Digestable for $type {
            fn unambiguously_encode<B: Buffer>(&self, encoder: encoding::EncodeValue<B>) {
                encoder.encode_leaf().chain(self.to_be_bytes());
            }
        }
    )*};
}

digestable_integers!(i8, u8, i16, u16, i32, u32, i64, u64, i128, u128);

impl Digestable for char {
    fn unambiguously_encode<B: Buffer>(&self, encoder: encoding::EncodeValue<B>) {
        // Any char can be represented using two bytes, but strangely Rust does not provide
        // conversion into `u16`, so we convert it into `u32`
        let c: u32 = (*self).into();
        c.unambiguously_encode(encoder);
    }
}

// Implements `Digestable` for the types that can be converted to bytes
macro_rules! digestable_as_bytes {
    ($($type:ty as $to_bytes:ident),*) => {$(
        impl Digestable for $type {
            fn unambiguously_encode<B: Buffer>(&self, encoder: encoding::EncodeValue<B>) {
                let bytes: &[u8] = self.$to_bytes();
                encoder.encode_leaf().chain(bytes);
            }
        }
    )*};
}

#[cfg(feature = "alloc")]
digestable_as_bytes!(
    alloc::string::String as as_ref,
    alloc::ffi::CString as to_bytes
);

digestable_as_bytes!(str as as_ref, core::ffi::CStr as to_bytes);

impl<T: Digestable> Digestable for Option<T> {
    fn unambiguously_encode<B: Buffer>(&self, encoder: encoding::EncodeValue<B>) {
        match self {
            Some(value) => {
                let mut encoder = encoder.encode_enum().with_variant("Some");
                let value_encoder = encoder.add_field("0");
                value.unambiguously_encode(value_encoder);
            }
            None => {
                encoder.encode_enum().with_variant("None");
            }
        }
    }
}

impl<T: Digestable, E: Digestable> Digestable for Result<T, E> {
    fn unambiguously_encode<B: Buffer>(&self, encoder: encoding::EncodeValue<B>) {
        match self {
            Ok(value) => {
                let mut encoder = encoder.encode_enum().with_variant("Ok");
                let value_encoder = encoder.add_field("0");
                value.unambiguously_encode(value_encoder);
            }
            Err(value) => {
                let mut encoder = encoder.encode_enum().with_variant("Err");
                let value_encoder = encoder.add_field("0");
                value.unambiguously_encode(value_encoder);
            }
        }
    }
}

macro_rules! digestable_tuple {
    ($($letter:ident),+) => {
        impl<$($letter: Digestable),+> Digestable for ($($letter),+) {
            fn unambiguously_encode<BUF: Buffer>(&self, encoder: encoding::EncodeValue<BUF>) {
                #[allow(non_snake_case)]
                let ($($letter),+) = self;
                let mut list = encoder.encode_list();
                $(
                    let item_encoder = list.add_item();
                    $letter.unambiguously_encode(item_encoder);
                )+
            }
        }
    };
}

macro_rules! digestable_tuples {
    ($letter:ident) => {};
    ($letter:ident, $($others:ident),+) => {
        digestable_tuple!($letter, $($others),+);
        digestable_tuples!($($others),+);
    }
}

// We support tuples with up to 16 elements
digestable_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);

fn unambiguously_encode_iter<B: Buffer, T: Digestable>(
    encoder: encoding::EncodeValue<B>,
    iter: impl IntoIterator<Item = T>,
) {
    let mut list = encoder.encode_list();
    for item in iter {
        let item_encoder = list.add_item();
        item.unambiguously_encode(item_encoder);
    }
}

impl<T: Digestable> Digestable for [T] {
    fn unambiguously_encode<B: Buffer>(&self, encoder: encoding::EncodeValue<B>) {
        unambiguously_encode_iter(encoder, self)
    }
}

impl<T: Digestable, const N: usize> Digestable for [T; N] {
    fn unambiguously_encode<B: Buffer>(&self, encoder: encoding::EncodeValue<B>) {
        self.as_slice().unambiguously_encode(encoder)
    }
}

#[cfg(feature = "alloc")]
impl<T: Digestable> Digestable for alloc::vec::Vec<T> {
    fn unambiguously_encode<B: Buffer>(&self, encoder: encoding::EncodeValue<B>) {
        self.as_slice().unambiguously_encode(encoder)
    }
}

#[cfg(feature = "alloc")]
impl<T: Digestable> Digestable for alloc::collections::LinkedList<T> {
    fn unambiguously_encode<B: Buffer>(&self, encoder: encoding::EncodeValue<B>) {
        unambiguously_encode_iter(encoder, self)
    }
}

#[cfg(feature = "alloc")]
impl<T: Digestable> Digestable for alloc::collections::VecDeque<T> {
    fn unambiguously_encode<B: Buffer>(&self, encoder: encoding::EncodeValue<B>) {
        unambiguously_encode_iter(encoder, self)
    }
}

#[cfg(feature = "alloc")]
impl<T: Digestable> Digestable for alloc::collections::BTreeSet<T> {
    fn unambiguously_encode<B: Buffer>(&self, encoder: encoding::EncodeValue<B>) {
        unambiguously_encode_iter(encoder, self)
    }
}

#[cfg(feature = "alloc")]
impl<K: Digestable, V: Digestable> Digestable for alloc::collections::BTreeMap<K, V> {
    fn unambiguously_encode<B: Buffer>(&self, encoder: encoding::EncodeValue<B>) {
        unambiguously_encode_iter(encoder, self)
    }
}

// Implements digestable for wrappers like Box<T>
#[cfg(feature = "alloc")]
macro_rules! digestable_wrapper {
    ($($wrapper:ty),*) => {$(
        impl<T: Digestable + ?Sized> Digestable for $wrapper {
            fn unambiguously_encode<B: Buffer>(&self, encoder: encoding::EncodeValue<B>) {
                (&**self).unambiguously_encode(encoder)
            }
        }
    )*};
}

#[cfg(feature = "alloc")]
digestable_wrapper!(alloc::boxed::Box<T>, alloc::rc::Rc<T>, alloc::sync::Arc<T>);

#[cfg(feature = "alloc")]
impl<'a, T> Digestable for alloc::borrow::Cow<'a, T>
where
    T: Digestable + alloc::borrow::ToOwned + ?Sized + 'a,
{
    fn unambiguously_encode<B: Buffer>(&self, encoder: encoding::EncodeValue<B>) {
        self.as_ref().unambiguously_encode(encoder);
    }
}

impl<T> Digestable for core::marker::PhantomData<T> {
    fn unambiguously_encode<B: Buffer>(&self, encoder: encoding::EncodeValue<B>) {
        // Encode an empty list
        encoder.encode_list();
    }
}
