//! Dependency-free unambiguous encoding core for udigest.
//!
//! This crate provides encoding traits and helper modules.
//! Hashing helpers that depend on the `digest` crate live in `udigest`.

#![no_std]
#![forbid(missing_docs)]
#![cfg_attr(not(test), forbid(unused_crate_dependencies))]
#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

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
///       udigest::hash::<sha2::Sha256>(&person_a),
///       udigest::hash::<sha2::Sha256>(&person_b),
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
///   fn encoder(value: &T, encoder: udigest::encoding::EncodeValue)
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
///   fn encode_instant(
///       instant: &std::time::Instant,
///       encoder: udigest::encoding::EncodeValue
///   ) {
///       todo!()
///   }
///   ```
/// * `#[udigest(as = ...)]` \
///   Tells to encode the field as another type `U`. Proc macro will use
///   [`<U as DigestAs<FieldType>>::digest_as`](DigestAs) to encode this field.
///
///   It is similar to `#[udigest(with = ...)]` attribute described above which allows to specify
///   a function which will be used to encode a field value. `#[udigest(as = ...)]` is designed for
///   more complex use-cases. For instance, it can be used to digest a hash map:
///
///   ```rust
///   #[derive(udigest::Digestable)]
///   pub struct Attributes(
///       #[udigest(as = std::collections::BTreeMap<_, udigest::Bytes>)]
///       std::collections::HashMap<String, Vec<u8>>,
///   );
///   ```
///
///   When structure is digested, the hash map is converted into btree map. `_` in `BTreeMap<_, udigest::Bytes>`
///   says that the key should be kept as it is: `String` string will be digested. The macro
///   replaces underscores (also called "infer types") with [`Same`](crate::as_::Same), which
///   indicates that the same digestion rules should be used as for the original type.
///   `udigest::Bytes` indicates that the value `Vec<u8>` should be digested as bytes, not as
///   list of u8 which would be a default behavior.
///
///   Similarly, if we have a field of type `Option<Vec<u8>>` and we want it to be encoded as
///   bytes, we cannot use `#[udigest(as_bytes)]` as the field does not implement `AsRef<[u8]>`:
///
///   ```compile_fail
///   #[derive(udigest::Digestable)]
///   pub struct Data(
///       #[udigest(as_bytes)]
///       Option<Vec<u8>>,
///   );
///   ```
///
///   We can use `as` attribute instead:
///   ```rust
///   #[derive(udigest::Digestable)]
///   pub struct Data(
///       #[udigest(as = Option<udigest::Bytes>)]
///       Option<Vec<u8>>,
///   );
///   ```
///
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
#[cfg(feature = "inline-struct")]
pub mod inline_struct;

pub mod as_;
pub use as_::DigestAs;

#[doc(hidden)]
pub mod _macros;

/// A value that can be unambiguously digested
pub trait Digestable {
    /// Unambiguously encodes the value
    fn unambiguously_encode(&self, encoder: encoding::EncodeValue);
}

impl<T: Digestable + ?Sized> Digestable for &T {
    fn unambiguously_encode(&self, encoder: encoding::EncodeValue) {
        (*self).unambiguously_encode(encoder)
    }
}

impl Digestable for core::convert::Infallible {
    fn unambiguously_encode(&self, _encoder: encoding::EncodeValue) {
        match *self {}
    }
}

/// Wrapper for a bytestring
///
/// Wraps any bytestring that `impl AsRef<[u8]>` and provides [`Digestable`] trait implementation
pub struct Bytes<T: ?Sized = [u8; 0]>(pub T);

impl<T: AsRef<[u8]> + ?Sized> Digestable for Bytes<T> {
    fn unambiguously_encode(&self, encoder: encoding::EncodeValue) {
        encoder.encode_leaf_value(self.0.as_ref())
    }
}

macro_rules! digestable_signed_integers {
    ($($type:ty),*) => {$(
        impl Digestable for $type {
            fn unambiguously_encode(&self, encoder: encoding::EncodeValue) {
                encode_signed_integer(
                    self.is_positive(),
                    &self.unsigned_abs().to_be_bytes(),
                    encoder,
                )
            }
        }
    )*};
}

/// Encodes an integer without leading zeroes
fn encode_signed_integer(is_positive: bool, abs_be_bytes: &[u8], encoder: encoding::EncodeValue) {
    let leading_zeroes = abs_be_bytes.iter().take_while(|b| **b == 0).count();
    let truncated_be_bytes = &abs_be_bytes[leading_zeroes..];
    if truncated_be_bytes.is_empty() {
        // zero is encoded as empty bytestring
        encoder.encode_leaf_value([])
    } else {
        encoder
            .encode_leaf()
            .chain([u8::from(is_positive)])
            .chain(truncated_be_bytes)
            .finish()
    }
}

macro_rules! digestable_unsigned_integers {
    ($($type:ty),*) => {$(
        impl Digestable for $type {
            fn unambiguously_encode(&self, encoder: encoding::EncodeValue) {
                encode_unsigned_integer(&self.to_be_bytes(), encoder)
            }
        }
    )*};
}

/// Encodes an integer without leading zeroes
fn encode_unsigned_integer(be_bytes: &[u8], encoder: encoding::EncodeValue) {
    let leading_zeroes = be_bytes.iter().take_while(|b| **b == 0).count();
    let truncated_be_bytes = &be_bytes[leading_zeroes..];
    encoder.encode_leaf_value(truncated_be_bytes)
}

digestable_signed_integers!(i8, i16, i32, i64, i128, isize);
digestable_unsigned_integers!(u8, u16, u32, u64, u128, usize);

impl Digestable for bool {
    fn unambiguously_encode(&self, encoder: encoding::EncodeValue) {
        u8::from(*self).unambiguously_encode(encoder)
    }
}

impl Digestable for char {
    fn unambiguously_encode(&self, encoder: encoding::EncodeValue) {
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
            fn unambiguously_encode(&self, encoder: encoding::EncodeValue) {
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
    fn unambiguously_encode(&self, encoder: encoding::EncodeValue) {
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
    fn unambiguously_encode(&self, encoder: encoding::EncodeValue) {
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
        impl<$($letter: Digestable),+> Digestable for ($($letter,)+) {
            fn unambiguously_encode(&self, encoder: encoding::EncodeValue) {
                #[allow(non_snake_case)]
                let ($($letter,)+) = self;
                let mut list = encoder.encode_list();
                $(
                    let item_encoder = list.add_item();
                    $letter.unambiguously_encode(item_encoder);
                )+
            }
        }
    };
}

// We support tuples with up to 16 elements
digestable_tuple!(A);
digestable_tuple!(A, B);
digestable_tuple!(A, B, C);
digestable_tuple!(A, B, C, D);
digestable_tuple!(A, B, C, D, E);
digestable_tuple!(A, B, C, D, E, F);
digestable_tuple!(A, B, C, D, E, F, G);
digestable_tuple!(A, B, C, D, E, F, G, H);
digestable_tuple!(A, B, C, D, E, F, G, H, I);
digestable_tuple!(A, B, C, D, E, F, G, H, I, J);
digestable_tuple!(A, B, C, D, E, F, G, H, I, J, K);
digestable_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);
digestable_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M);
digestable_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
digestable_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
digestable_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);

fn unambiguously_encode_iter<T: Digestable>(
    encoder: encoding::EncodeValue,
    iter: impl IntoIterator<Item = T>,
) {
    let mut list = encoder.encode_list();
    for item in iter {
        let item_encoder = list.add_item();
        item.unambiguously_encode(item_encoder);
    }
}

impl<T: Digestable> Digestable for [T] {
    fn unambiguously_encode(&self, encoder: encoding::EncodeValue) {
        unambiguously_encode_iter(encoder, self)
    }
}

impl<T: Digestable, const N: usize> Digestable for [T; N] {
    fn unambiguously_encode(&self, encoder: encoding::EncodeValue) {
        self.as_slice().unambiguously_encode(encoder)
    }
}

#[cfg(feature = "alloc")]
impl<T: Digestable> Digestable for alloc::vec::Vec<T> {
    fn unambiguously_encode(&self, encoder: encoding::EncodeValue) {
        self.as_slice().unambiguously_encode(encoder)
    }
}

#[cfg(feature = "alloc")]
impl<T: Digestable> Digestable for alloc::collections::LinkedList<T> {
    fn unambiguously_encode(&self, encoder: encoding::EncodeValue) {
        unambiguously_encode_iter(encoder, self)
    }
}

#[cfg(feature = "alloc")]
impl<T: Digestable> Digestable for alloc::collections::VecDeque<T> {
    fn unambiguously_encode(&self, encoder: encoding::EncodeValue) {
        unambiguously_encode_iter(encoder, self)
    }
}

#[cfg(feature = "alloc")]
impl<T: Digestable> Digestable for alloc::collections::BTreeSet<T> {
    fn unambiguously_encode(&self, encoder: encoding::EncodeValue) {
        unambiguously_encode_iter(encoder, self)
    }
}

#[cfg(feature = "alloc")]
impl<K: Digestable, V: Digestable> Digestable for alloc::collections::BTreeMap<K, V> {
    fn unambiguously_encode(&self, encoder: encoding::EncodeValue) {
        unambiguously_encode_iter(encoder, self)
    }
}

// Implements digestable for wrappers like Box<T>
#[cfg(feature = "alloc")]
macro_rules! digestable_wrapper {
    ($($wrapper:ty),*) => {$(
        impl<T: Digestable + ?Sized> Digestable for $wrapper {
            fn unambiguously_encode(&self, encoder: encoding::EncodeValue) {
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
    fn unambiguously_encode(&self, encoder: encoding::EncodeValue) {
        self.as_ref().unambiguously_encode(encoder);
    }
}

impl<T> Digestable for core::marker::PhantomData<T> {
    fn unambiguously_encode(&self, encoder: encoding::EncodeValue) {
        // Encode an empty list
        encoder.encode_list();
    }
}
