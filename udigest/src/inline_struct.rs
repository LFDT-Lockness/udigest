//! Digestable inline structs
//!
//! If you find yourself in situation in which you have to define a struct just
//! to use it only once for `udigest` hashing, [`inline_struct!`] macro can be
//! used instead:
//!
//! ```rust
//! let hash = udigest::hash::<sha2::Sha256, _>(&udigest::inline_struct!({
//!     name: "Alice",
//!     age: 24_u32,
//! }));
//! ```
//!
//! Which will produce identical hash as below:
//!
//! ```rust
//! #[derive(udigest::Digestable)]
//! struct Person {
//!     name: &'static str,
//!     age: u32,
//! }
//!
//! let hash = udigest::hash::<sha2::Sha256, _>(&Person {
//!     name: "Alice",
//!     age: 24,
//! });
//! ```
//!
//! See [`inline_struct!`] macro for more examples.

/// Inline structure
///
/// Normally, you don't need to use it directly. Use [`inline_struct!`] macro instead.
pub struct InlineStruct<'a, F: FieldsList + 'a = Nil> {
    fields_list: F,
    tag: Option<&'a [u8]>,
}

impl InlineStruct<'static> {
    /// Creates inline struct with no fields
    ///
    /// Normally, you don't need to use it directly. Use [`inline_struct!`] macro instead.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            fields_list: Nil,
            tag: None,
        }
    }
}

impl<'a, F: FieldsList + 'a> InlineStruct<'a, F> {
    /// Adds field to the struct
    ///
    /// Normally, you don't need to use it directly. Use [`inline_struct!`] macro instead.
    pub fn add_field<V>(
        self,
        field_name: &'a str,
        field_value: &'a V,
    ) -> InlineStruct<'a, impl FieldsList + 'a>
    where
        F: 'a,
        V: crate::Digestable,
    {
        InlineStruct {
            fields_list: cons(field_name, field_value, self.fields_list),
            tag: self.tag,
        }
    }

    /// Sets domain-separation tag
    ///
    /// Normally, you don't need to use it directly. Use [`inline_struct!`] macro instead.
    pub fn set_tag<T: ?Sized + AsRef<[u8]>>(mut self, tag: &'a T) -> Self {
        self.tag = Some(tag.as_ref());
        self
    }
}

impl<'a, F: FieldsList + 'a> crate::Digestable for InlineStruct<'a, F> {
    fn unambiguously_encode<B: crate::Buffer>(&self, encoder: crate::encoding::EncodeValue<B>) {
        let mut struct_encode = encoder.encode_struct();
        if let Some(tag) = self.tag {
            struct_encode.set_tag(tag);
        }
        self.fields_list.encode(&mut struct_encode);
    }
}

/// Creates digestable inline struct
///
/// Macro creates "inlined" (anonymous) struct instance containing specified fields and their
/// values. The inlined struct implements [`Digestable` trait](crate::Digestable), and therefore
/// can be unambiguously hashed, for instance, using [`udigest::hash`](crate::hash). It helps
/// reducing amount of code when otherwise you'd have to define a separate struct which would
/// only be used one.
///
/// ## Usage
/// The code snippet below inlines `struct Person { name: &str, age: u32 }`.
/// ```rust
/// let hash = udigest::hash::<sha2::Sha256, _>(&udigest::inline_struct!({
///     name: "Alice",
///     age: 24_u32,
/// }));
/// ```
///
/// You may add a domain separation tag:
/// ```rust
/// let hash = udigest::hash::<sha2::Sha256, _>(
///     &udigest::inline_struct!("some tag" {
///         name: "Alice",
///         age: 24_u32,
///     })
/// );
/// ```
///
/// Several structs may be embedded in each other:
/// ```rust
/// let hash = udigest::hash::<sha2::Sha256, _>(&udigest::inline_struct!({
///     name: "Alice",
///     age: 24_u32,
///     preferences: udigest::inline_struct!({
///         display_email: false,
///         receive_newsletter: false,
///     }),
/// }));
/// ```
#[macro_export]
macro_rules! inline_struct {
    ({$($field_name:ident: $field_value:expr),*$(,)?}) => {{
        $crate::inline_struct::InlineStruct::new()
            $(.add_field(stringify!($field_name), &$field_value))*
    }};
    ($tag:tt {$($field_name:ident: $field_value:expr),*$(,)?}) => {{
        $crate::inline_struct::InlineStruct::new()
            .set_tag($tag)
            $(.add_field(stringify!($field_name), &$field_value))*

    }};
}

pub use crate::inline_struct;

mod sealed {
    pub trait Sealed {}
}

/// List of fields in inline struct
///
/// Normally, you don't need to use it directly. Use [`inline_struct!`] macro instead.
pub trait FieldsList: sealed::Sealed {
    /// Encodes all fields in order from the first to last
    fn encode<B: crate::Buffer>(&self, encoder: &mut crate::encoding::EncodeStruct<B>);
}

/// Empty list of fields
///
/// Normally, you don't need to use it directly. Use [`inline_struct!`] macro instead.
pub struct Nil;
impl sealed::Sealed for Nil {}
impl FieldsList for Nil {
    fn encode<B: crate::Buffer>(&self, _encoder: &mut crate::encoding::EncodeStruct<B>) {
        // Empty list - do nothing
    }
}

fn cons<'a, V, T>(field_name: &'a str, field_value: &'a V, tail: T) -> impl FieldsList + 'a
where
    V: crate::Digestable,
    T: FieldsList + 'a,
{
    struct Cons<'a, V, T: 'a> {
        field_name: &'a str,
        field_value: &'a V,
        tail: T,
    }

    impl<'a, V, T: 'a> sealed::Sealed for Cons<'a, V, T> {}

    impl<'a, V: crate::Digestable, T: FieldsList + 'a> FieldsList for Cons<'a, V, T> {
        fn encode<B: crate::Buffer>(&self, encoder: &mut crate::encoding::EncodeStruct<B>) {
            // Since we store fields from last to first, we need to encode the tail first
            // to reverse order of fields
            self.tail.encode(encoder);

            let value_encoder = encoder.add_field(self.field_name);
            self.field_value.unambiguously_encode(value_encoder);
        }
    }

    Cons {
        field_name,
        field_value,
        tail,
    }
}
