//! Digestable inline structs
//!
//! If you find yourself in situation in which you have to define a struct just
//! to use it only once for `udigest` hashing, [`inline_struct!`] macro can be
//! used instead:
//!
//! ```rust
//! let hash = udigest::hash::<sha2::Sha256>(&udigest::inline_struct!({
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
//! let hash = udigest::hash::<sha2::Sha256>(&Person {
//!     name: "Alice",
//!     age: 24,
//! });
//! ```
//!
//! See [`inline_struct!`] macro for more examples.

/// Inline structure
///
/// Normally, you don't need to use it directly. Use [`inline_struct!`] macro instead.
#[derive(Clone, Copy)]
pub struct InlineStruct<'a, F: FieldsList> {
    fields_list: F,
    tag: Option<&'a [u8]>,
}

impl InlineStruct<'static, Nil> {
    /// Creates an empty struct with no fields
    pub fn new() -> Self {
        Self {
            fields_list: Nil,
            tag: None,
        }
    }
}

impl Default for InlineStruct<'static, Nil> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'t, F: FieldsList> InlineStruct<'t, F> {
    /// Adds field to the struct
    ///
    /// Normally, you don't need to use it directly. Use [`inline_struct!`] macro instead.
    pub fn add_field<'f, V>(
        self,
        field_name: &'f str,
        field_value: V,
    ) -> InlineStruct<'t, Cons<'f, V, F>>
    where
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
    pub fn set_tag<T: ?Sized + AsRef<[u8]>>(self, tag: &T) -> InlineStruct<F> {
        InlineStruct {
            fields_list: self.fields_list,
            tag: Some(tag.as_ref()),
        }
    }
}

impl<'a, F: FieldsList> crate::Digestable for InlineStruct<'a, F> {
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
/// let hash = udigest::hash::<sha2::Sha256>(&udigest::inline_struct!({
///     name: "Alice",
///     age: 24_u32,
/// }));
/// ```
///
/// You may add a domain separation tag:
/// ```rust
/// let hash = udigest::hash::<sha2::Sha256>(
///     &udigest::inline_struct!("some tag" {
///         name: "Alice",
///         age: 24_u32,
///     })
/// );
/// ```
///
/// Several structs may be embedded in each other:
/// ```rust
/// let hash = udigest::hash::<sha2::Sha256>(&udigest::inline_struct!({
///     name: "Alice",
///     age: 24_u32,
///     preferences: udigest::inline_struct!({
///         display_email: false,
///         receive_newsletter: false,
///     }),
/// }));
/// ```
///
/// Similar to regular struct construction, you may omit field value, then macro will be looking
/// for a variable named the same as the field:
/// ```rust
/// let name = "Alice";
/// let age = 24_u32;
/// let hash = udigest::hash::<sha2::Sha256>(
///     &udigest::inline_struct!({
///         name,
///         age,
///     })
/// );
/// ```
///
/// You can also put an ampersand `&` before field name, then it will be taken by reference:
/// ```rust
/// let name: String = "Alice".into();
/// let hash = udigest::hash::<sha2::Sha256>(
///     &udigest::inline_struct!({
///         &name,
///         age: 24_u32,
///     })
/// );
///
/// // `name` is not consumed:
/// println!("{name}")
/// ```
#[macro_export]
macro_rules! inline_struct {
    ({$($fields:tt)*}) => {{
        let s = $crate::inline_struct::InlineStruct::new();
        $crate::inline_struct_helper!(s {$($fields)*})
    }};
    ($tag:tt {$($fields:tt)*}) => {{
        $crate::inline_struct!({$($fields)*}).set_tag($tag)
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! inline_struct_helper {
    ($s:ident {$field_name:ident: $field_value:expr $(, $($rest:tt)*)?}) => {{
        let s = $s.add_field(stringify!($field_name), $field_value);
        $crate::inline_struct_helper!(s {$($($rest)*)?})
    }};
    ($s:ident {$field_name:ident $(, $($rest:tt)*)?}) => {{
        let s = $s.add_field(stringify!($field_name), $field_name);
        $crate::inline_struct_helper!(s {$($($rest)*)?})
    }};
    ($s:ident {&$field_name:ident $(, $($rest:tt)*)?}) => {{
        let s = $s.add_field(stringify!($field_name), &$field_name);
        $crate::inline_struct_helper!(s {$($($rest)*)?})
    }};
    ($s:ident {,}) => {{
        $s
    }};
    ($s:ident {}) => {{
        $s
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
#[derive(Clone, Copy)]
pub struct Nil;
impl sealed::Sealed for Nil {}
impl FieldsList for Nil {
    fn encode<B: crate::Buffer>(&self, _encoder: &mut crate::encoding::EncodeStruct<B>) {
        // Empty list - do nothing
    }
}

/// Prepends a field to the list of fields
///
/// Normally, you don't need to use it directly. Use [`inline_struct!`] macro instead.
#[derive(Clone, Copy)]
pub struct Cons<'a, V, T> {
    field_name: &'a str,
    field_value: V,
    tail: T,
}

impl<'a, V, T> sealed::Sealed for Cons<'a, V, T> {}

impl<'a, V: crate::Digestable, T: FieldsList> FieldsList for Cons<'a, V, T> {
    fn encode<B: crate::Buffer>(&self, encoder: &mut crate::encoding::EncodeStruct<B>) {
        // Since we store fields from last to first, we need to encode the tail first
        // to reverse order of fields
        self.tail.encode(encoder);

        let value_encoder = encoder.add_field(self.field_name);
        self.field_value.unambiguously_encode(value_encoder);
    }
}

fn cons<V, T>(field_name: &str, field_value: V, tail: T) -> Cons<V, T>
where
    V: crate::Digestable,
    T: FieldsList,
{
    Cons {
        field_name,
        field_value,
        tail,
    }
}
