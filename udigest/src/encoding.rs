//! # Unambiguous encoding
//!
//! The core of the crate is functionality to unambiguously encode any structured data
//! into bytes. It's then used to digest any data into a hash. Note that this module
//! provides low-level implementation details which you normally don't need to know
//! unless you manually implemenet [Digestable](crate::Digestable) trait.
//!
//! Any structured `value` is encoded either as a bytestring or as a list (each element
//! within the list is either a bytestring or a list). The simplified grammar can be seen as
//! (the full grammar is presented and explained below):
//!
//! ```text
//! value ::= leaf | list
//! leaf  ::= bytestring
//! list  ::= [value]
//! ```
//!
//! Encoding goal is to distinguish `["12", "3"]` from `["1", "23"]`, `["1", [], "2"]`
//! from `["1", "2"]` and so on. Now, we only need to map any structured data onto
//! that grammar to have an unambiguous encoding. Below, we will show how Rust structures
//! can be mapped onto the lists, and then we descrive how exactly encoding works.
//!
//! # Mapping Rust types onto lists
//!
//! ### Structure
//! Structures can be encoded as a list of field names followed by field values. For instance, a structure
//!
//! ```rust
//! struct Person {
//!     name: String,
//!     job_title: String,
//! }
//! let alice = Person {
//!     name: "Alice".into(),
//!     job_title: "cryptographer".into(),
//! };
//! ```
//!
//! will be encoded as a list below:
//!
//! ```text
//! ["name", "Alice", "job_title", "cryptographer"]
//! ```
//!
//! [EncodeStruct] can be used to encode a struct.
//!
//! ### Enum
//! Enums are encoded as a list that contains a variant name, and then, same as for a structure, it contains
//! fields names followed by field values associated with the variant (if any). For instance, an enum
//!
//! ```rust
//! enum Shape {
//!     Circle { radius: u8 },
//!     Square { width: u8 },
//! }
//! let circle = Shape::Circle { radius: 5 };
//! ```
//!
//! will be encoded as a list:
//!
//! ```text
//! ["variant", "Circle", "radius", 5_u32]
//! ```
//!
//! [EncodeEnum] can be used to encode an enum.
//!
//! ### Primitive types
//! Primitive values can be encoded as bytestrings as long as they can be unambiguously converted to bytes.
//! For instance, any integer can be converted into bytes using [to_be_bytes](u32::to_be_bytes). Strings can
//! be [converted to bytes](str::as_bytes) as well, and so on.
//!
//! ### Domain separation
//! When value is encoded into bytes, it loses its type. For instance, "abcd" bytestring may correspond to
//! `Vec<u8>`, `String`, `u32` and so on. When it's required to distinguish one type from another, domain
//! separation tag can be used.
//!
//! Domain separation tag can be specified for any value using [`.with_tag()`](EncodeLeaf::with_tag) method.
//!
//! It's recommended to specify the tag only for high-level structures.
//!
//! # Encoding lists into bytes
//! Generally, when a value is encoded, firstly we write its byte representation and then append its metadata
//! like length and type (leaf of list). Writing a length after the value makes it possible to encode byte strings
//! and lists the length of which is not known in advance.
//!
//! Any `value` is encoded according to this grammar specification:
//!
//! ```text
//! value    ::= leaf | leaf_ctx | list | list_ctx
//!
//! leaf     ::= bytestring len(bytestring) LEAF
//! leaf_ctx ::= bytestring len(bytestring) tag len(tag) LEAF_CTX
//!
//! list     ::= [value] len([value]) LIST
//! list_ctx ::= [value] len([value]) ctx len(ctx) LIST_CTX
//!
//! len(n) ::=
//!   if n.len() <= u32::MAX {
//!     (n.len() as u32) LEN_32
//!   } else {
//!     let len_n = n.len().to_be_bytes().strip();
//!     assert!(len_n.len() < 256);
//!     len_n (len_n.len() as u8) BIGLEN
//!   }
//!
//! LIST     ::= 1
//! LIST_CTX ::= 2
//! LEAF     ::= 3
//! LEAF_CTX ::= 4
//! LEN_32   ::= 5
//! BIGLEN   ::= 6
//! ```
//!
//! # Example
//!
//! A structured data below
//! ```rust
//! struct Person {
//!     name: String,
//!     skills: Vec<String>,
//!     job_title: String,
//! }
//! let alice = Person {
//!     name: "Alice".into(),
//!     skills: vec!["math".into(), "crypto".into()],
//!     job_title: "cryptographer".into(),
//! };
//! ```
//!
//! will be encoded as a list:
//! ```text
//! ["name", "Alice", "skills", ["math", "crypto"], "job_title", "cryptographer"]
//! ```
//!
//! which will be translated into the bytes:
//! ```text
//! // Writes "name", "Alice" onto the stack
//! "name"  4_u32 LEN_32 LEAF
//! "Alice" 5_u32 LEN_32 LEAF
//! // Writes her skills onto the stack
//! "skills" 6_u32 LEN_32 LEAF
//! "math"   4_u32 LEN_32 LEAF
//! "crypto" 6_u32 LEN_32 LEAF
//! // Merges the last 2 elements from the stack into a list
//! 2_u32 LEN_32 LIST
//! // Writes her job title
//! "job_title"     9_u32  LEN_32 LEAF
//! "cryptographer" 13_u32 LEN_32 LEAF
//! // Merges the last 6 elements from the stack into a list
//! 6_u32 LEN_32 LIST
//! ```
//!
//! where `LEAF`, `LIST`, and `LEN_32` are constants [defined above](#encoding-lists-into-bytes).

/// Control symbol
///
/// See [module level](self) docs
pub const LIST: u8 = 1;
/// Control symbol
///
/// See [module level](self) docs
pub const LIST_CTX: u8 = 2;
/// Control symbol
///
/// See [module level](self) docs
pub const LEAF: u8 = 3;
/// Control symbol
///
/// See [module level](self) docs
pub const LEAF_CTX: u8 = 4;
/// Control symbol
///
/// See [module level](self) docs
pub const LEN_32: u8 = 5;
/// Control symbol
///
/// See [module level](self) docs
pub const BIGLEN: u8 = 6;

/// A buffer that exposes append-only access
///
/// Out of box, it's implemented for any hashing algorithm that implements
/// [`digest::Update`]
pub trait Buffer {
    /// Appends `bytes` to the buffer
    ///
    /// Method must never panic
    fn write(&mut self, bytes: &[u8]);
}

impl<D: digest::Digest> Buffer for D {
    fn write(&mut self, bytes: &[u8]) {
        self.update(bytes)
    }
}

/// Encodes a value
///
/// Can be used to encode (only) a single value. Value can be a leaf (bytestring) or a list of values.
#[must_use = "encoder must be used to encode a value"]
pub struct EncodeValue<'b, B: Buffer> {
    buffer: &'b mut B,
}

impl<'b, B: Buffer> EncodeValue<'b, B> {
    /// Constructs an encoder
    pub fn new(buffer: &'b mut B) -> Self {
        Self { buffer }
    }

    /// Encodes a list
    pub fn encode_list(self) -> EncodeList<'b, B> {
        EncodeList::new(self.buffer)
    }

    /// Encodes a leaf (bytestring)
    pub fn encode_leaf(self) -> EncodeLeaf<'b, B> {
        EncodeLeaf::new(self.buffer)
    }

    /// Encodes a struct
    ///
    /// Struct is represented as a list: `[field_name1, field_value1, ...]`
    pub fn encode_struct(self) -> EncodeStruct<'b, B> {
        EncodeStruct::new(self.buffer)
    }

    /// Encodes an enum
    ///
    /// Enum is represented as a list: `["variant", variant_name, field_name1, field_value1, ...]`
    pub fn encode_enum(self) -> EncodeEnum<'b, B> {
        EncodeEnum::new(self.buffer)
    }
}

/// Encodes an enum
///
/// Enum variant is encoded as a list: `["variant", variant_name]`. If variant contains any fields,
/// they are encoded in the same way as [structure](EncodeStruct) fields are encoded.
#[must_use = "encoder must be used to encode a value"]
pub struct EncodeEnum<'b, B: Buffer> {
    buffer: &'b mut B,
}

impl<'b, B: Buffer> EncodeEnum<'b, B> {
    /// Constructs an encoder
    pub fn new(buffer: &'b mut B) -> Self {
        Self { buffer }
    }

    /// Encodes a variant name
    ///
    /// Returns a structure encoder that can be used to encode any fields the variant may have
    pub fn with_variant(self, variant_name: impl AsRef<[u8]>) -> EncodeStruct<'b, B> {
        let mut s = EncodeStruct::new(self.buffer);
        s.add_field("variant").encode_leaf().chain(variant_name);
        s
    }
}

/// Encodes a structure
pub struct EncodeStruct<'b, B: Buffer> {
    list: EncodeList<'b, B>,
}

impl<'b, B: Buffer> EncodeStruct<'b, B> {
    /// Constructs an encoder
    pub fn new(buffer: &'b mut B) -> Self {
        Self {
            list: EncodeList::new(buffer),
        }
    }

    /// Specifies a domain separation tag
    ///
    /// Tag will be unambiguously encoded
    pub fn set_tag(&mut self, tag: &'b [u8]) {
        self.list.set_tag(tag);
    }

    /// Specifies a domain separation tag
    ///
    /// Tag will be unambiguously encoded
    pub fn with_tag(mut self, tag: &'b [u8]) -> Self {
        self.set_tag(tag);
        self
    }

    /// Adds a fiels to the structure
    ///
    /// Returns an encoder that shall be used to encode the fiels value
    pub fn add_field(&mut self, field_name: impl AsRef<[u8]>) -> EncodeValue<B> {
        self.list.add_leaf().chain(field_name);
        self.list.add_item()
    }

    /// Finilizes the encoding, puts the necessary metadata to the buffer
    ///
    /// It's an alias to dropping the encoder
    pub fn finish(self) {}
}

/// Encodes a leaf (bytestring)
pub struct EncodeLeaf<'b, B: Buffer> {
    buffer: &'b mut B,
    len: usize,
    tag: Option<&'b [u8]>,
}

impl<'b, B: Buffer> EncodeLeaf<'b, B> {
    /// Constructs a leaf
    pub fn new(buffer: &'b mut B) -> Self {
        Self {
            buffer,
            len: 0,
            tag: None,
        }
    }

    /// Specifies a domain separation tag
    ///
    /// Tag will be unambiguously encoded
    pub fn set_tag(&mut self, tag: &'b [u8]) {
        self.tag = Some(tag)
    }

    /// Specifies a domain separation tag
    ///
    /// Tag will be unambiguously encoded
    pub fn with_tag(mut self, tag: &'b [u8]) -> Self {
        self.set_tag(tag);
        self
    }

    /// Chains a bytestring
    ///
    /// Encoded value will correspond to concatenation of all the chained bytestrings
    pub fn chain(mut self, bytes: impl AsRef<[u8]>) -> Self {
        self.update(bytes.as_ref());
        self
    }

    /// Appends a bytestring
    ///
    /// Encoded value will correspond to concatenation of all the chained bytestrings
    pub fn update(&mut self, bytes: &[u8]) {
        self.buffer.write(bytes);
        self.len = self
            .len
            .checked_add(bytes.len())
            .expect("leaf length overflows `usize`")
    }

    /// Finilizes the encoding, puts the necessary metadata to the buffer
    ///
    /// It's an alias to dropping the encoder
    pub fn finish(self) {}
}

impl<'b, B: Buffer> Drop for EncodeLeaf<'b, B> {
    fn drop(&mut self) {
        encode_len(self.buffer, self.len);

        if let Some(tag) = self.tag {
            self.buffer.write(tag);
            encode_len(self.buffer, tag.len());

            self.buffer.write(&[LEAF_CTX]);
        } else {
            self.buffer.write(&[LEAF]);
        }
    }
}

/// Encodes a list of values
pub struct EncodeList<'b, B: Buffer> {
    buffer: &'b mut B,
    len: usize,
    tag: Option<&'b [u8]>,
}

impl<'b, B: Buffer> EncodeList<'b, B> {
    /// Constructs an encoder
    pub fn new(buffer: &'b mut B) -> Self {
        Self {
            buffer,
            len: 0,
            tag: None,
        }
    }

    /// Specifies a domain separation tag
    ///
    /// Tag will be unambiguously encoded
    pub fn set_tag(&mut self, tag: &'b [u8]) {
        self.tag = Some(tag)
    }

    /// Specifies a domain separation tag
    ///
    /// Tag will be unambiguously encoded
    pub fn with_tag(mut self, tag: &'b [u8]) -> Self {
        self.set_tag(tag);
        self
    }

    /// Adds an item to the list
    ///
    /// Returns an encoder that shall be used to encode a value of the item
    pub fn add_item(&mut self) -> EncodeValue<B> {
        self.len = self.len.checked_add(1).expect("list len overflows usize");
        EncodeValue::new(self.buffer)
    }

    /// Adds a leaf (bytestring) to the list
    ///
    /// Alias to `.add_item().encode_leaf()`
    pub fn add_leaf(&mut self) -> EncodeLeaf<B> {
        self.add_item().encode_leaf()
    }

    /// Adds a sublist to the list
    ///
    /// Alias to `.add_item().encode_list()`
    pub fn add_list(&mut self) -> EncodeList<B> {
        self.add_item().encode_list()
    }

    /// Finilizes the encoding, puts the necessary metadata to the buffer
    ///
    /// It's an alias to dropping the encoder
    pub fn finish(self) {}
}

impl<'b, B: Buffer> Drop for EncodeList<'b, B> {
    fn drop(&mut self) {
        encode_len(self.buffer, self.len);

        if let Some(tag) = self.tag {
            self.buffer.write(tag);
            encode_len(self.buffer, tag.len());

            self.buffer.write(&[LIST_CTX]);
        } else {
            self.buffer.write(&[LIST])
        }
    }
}

/// Encodes length of list or leaf
///
/// Altough we expose how the length is encoded, normally you should use [EncodeList]
/// and [EncodeLeaf] which use this function internally
pub fn encode_len(buffer: &mut impl Buffer, len: usize) {
    match u32::try_from(len) {
        Ok(len_32) => {
            buffer.write(&len_32.to_be_bytes());
            buffer.write(&[LEN_32]);
        }
        Err(_) => {
            let len = len.to_be_bytes();
            let leading_zeroes = len.iter().take_while(|b| **b == 0).count();
            let len = &len[leading_zeroes..];
            let len_of_len = u8::try_from(len.len()).expect("usize is more than 256 bytes long");
            buffer.write(len);
            buffer.write(&[len_of_len]);
            buffer.write(&[BIGLEN]);
        }
    }
}
