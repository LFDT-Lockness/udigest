//! Unambiguously digest structured data
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
//! * `HashMap`, `HashSet` as they can not be traversed in determenistic order
//! * `usize`, `isize` as their byte size varies on differnet platforms
//!
//! The `Digestable` trait can be implemented for the struct using a macro:
//! ```rust
//! #[derive(udigest::Digestable)]
//! struct Person {
//!     name: String,
//!     job_title: String,   
//! }
//!
//! let hash = udigest::Unambiguous::new("udigest.example")
//!     .digest(&Person {
//!         name: "Alice".into(),
//!         job_title: "cryptographer".into(),
//!     });
//! ```
//!
//! The crate intentionally does not try to follow any existing standards for unambiguous
//! encoding. The format for encoding was desingned specifically for `udigest` to provide
//! a better usage experience in Rust. The details of encoding format can be found
//! [here](encoding).

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

pub use encoding::Buffer;

#[cfg(feature = "derive")]
pub use udigest_derive::Digestable;

pub mod encoding;

/// Unambiguously digests structured data
pub struct Unambiguous<D: digest::Digest>(D);

impl<D: digest::Digest + Clone> Unambiguous<D> {
    /// Constructs a new digester
    ///
    /// Takes domain separation `tag` as an argument. Different tags lead to different
    /// hashes of the same value. It is recommended to define different tags per application
    /// for better hygiene.
    ///
    /// If the tag is represented by a structured data, [`Unambiguous::with_structured_tag`]
    /// constructor can be used instead.
    pub fn new(tag: impl AsRef<[u8]>) -> Self {
        Self::with_structured_tag(Bytes(tag))
    }

    /// Constructs a new digester
    ///
    /// Takes domain separation `tag` as an argument. Different tags lead to different
    /// hashes of the same value. It is recommended to define different tags per application
    /// for better hygiene.
    pub fn with_structured_tag(tag: impl Digestable) -> Self {
        Self::with_digest_and_structured_tag(D::new(), tag)
    }

    /// Constructs a new digester
    ///
    /// Similar to [`Unambiguous::with_structured_tag`] but takes also a digest to use
    pub fn with_digest_and_structured_tag(mut hash: D, tag: impl Digestable) -> Self {
        let mut header = encoding::EncodeStruct::new(&mut hash).with_tag(b"udigest.header");
        header.add_field("udigest_version").encode_leaf().chain("1");
        let tag_encoder = header.add_field("tag");
        tag.unambiguously_encode(tag_encoder);
        header.finish();

        Self(hash)
    }

    /// Digests a structured `value`
    pub fn digest<T: Digestable>(&self, value: &T) -> digest::Output<D> {
        let mut hash = self.0.clone();
        value.unambiguously_encode(encoding::EncodeValue::new(&mut hash));
        hash.finalize()
    }

    /// Digests a list of structured data
    pub fn digest_iter<T: Digestable>(
        &self,
        iter: impl IntoIterator<Item = T>,
    ) -> digest::Output<D> {
        let mut hash = self.0.clone();
        let mut encoder = encoding::EncodeList::new(&mut hash).with_tag(b"udigest.list");
        for value in iter {
            let item_encoder = encoder.add_item();
            value.unambiguously_encode(item_encoder);
        }
        encoder.finish();
        hash.finalize()
    }
}

/// A value that can be unambiguously digested
pub trait Digestable {
    fn unambiguously_encode<B: Buffer>(&self, encoder: encoding::EncodeValue<B>);
}

impl<T: Digestable> Digestable for &T {
    fn unambiguously_encode<B: Buffer>(&self, encoder: encoding::EncodeValue<B>) {
        (*self).unambiguously_encode(encoder)
    }
}

/// Wrapper for a bytestring
///
/// Wraps any bytestring than `impl AsRef<[u8]>` and provides [`Digestable`] trait implementation
pub struct Bytes<T>(pub T);

impl<T: AsRef<[u8]>> Digestable for Bytes<T> {
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
        impl<T: Digestable> Digestable for $wrapper {
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
