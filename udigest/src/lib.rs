//! Unambiguously digest structured data.
//!
//! This crate re-exports all encoding APIs from `udigest-encoding` and provides
//! hash helper functions powered by the `digest` crate.

#![no_std]
#![forbid(missing_docs)]
#![cfg_attr(not(test), forbid(unused_crate_dependencies))]
#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub use udigest_encoding::*;

struct BufferDigest<D: digest::Digest>(D);

impl<D: digest::Digest> udigest_encoding::encoding::Buffer for BufferDigest<D> {
    fn write(&mut self, bytes: &[u8]) {
        self.0.update(bytes);
    }
}

struct BufferUpdate<D: digest::Update>(D);

impl<D: digest::Update> udigest_encoding::encoding::Buffer for BufferUpdate<D> {
    fn write(&mut self, bytes: &[u8]) {
        self.0.update(bytes);
    }
}

/// Digests a structured value using fixed-output hash function (like sha2-256).
pub fn hash<D: digest::Digest>(value: &dyn udigest_encoding::Digestable) -> digest::Output<D> {
    let mut hash = BufferDigest(D::new());
    value.unambiguously_encode(udigest_encoding::encoding::EncodeValue::new(&mut hash));
    hash.0.finalize()
}

/// Digests a list of structured data using fixed-output hash function (like sha2-256).
pub fn hash_iter<D: digest::Digest>(
    iter: impl IntoIterator<Item = impl udigest_encoding::Digestable>,
) -> digest::Output<D> {
    let mut hash = BufferDigest(D::new());
    let mut encoder =
        udigest_encoding::encoding::EncodeList::new(&mut hash).with_tag(b"udigest.list");
    for value in iter {
        let item_encoder = encoder.add_item();
        value.unambiguously_encode(item_encoder);
    }
    encoder.finish();
    hash.0.finalize()
}

/// Digests a structured value using extendable-output hash function (like shake-256).
pub fn hash_xof<D>(value: &dyn udigest_encoding::Digestable) -> D::Reader
where
    D: Default + digest::Update + digest::ExtendableOutput,
{
    let mut hash = BufferUpdate(D::default());
    value.unambiguously_encode(udigest_encoding::encoding::EncodeValue::new(&mut hash));
    hash.0.finalize_xof()
}

/// Digests a list of structured data using extendable-output hash function (like shake-256).
pub fn hash_xof_iter<D>(
    iter: impl IntoIterator<Item = impl udigest_encoding::Digestable>,
) -> D::Reader
where
    D: Default + digest::Update + digest::ExtendableOutput,
{
    let mut hash = BufferUpdate(D::default());
    let mut encoder =
        udigest_encoding::encoding::EncodeList::new(&mut hash).with_tag(b"udigest.list");
    for value in iter {
        let item_encoder = encoder.add_item();
        value.unambiguously_encode(item_encoder);
    }
    encoder.finish();
    hash.0.finalize_xof()
}

/// Digests a structured value using variable-output hash function (like blake2b).
pub fn hash_vof<D>(
    value: &dyn udigest_encoding::Digestable,
    out: &mut [u8],
) -> Result<(), digest::InvalidOutputSize>
where
    D: digest::VariableOutput + digest::Update,
{
    let mut hash = BufferUpdate(D::new(out.len())?);
    value.unambiguously_encode(udigest_encoding::encoding::EncodeValue::new(&mut hash));
    hash.0
        .finalize_variable(out)
        .map_err(|_| digest::InvalidOutputSize)
}

/// Digests a list of structured data using variable-output hash function (like blake2b).
pub fn hash_vof_iter<D>(
    iter: impl IntoIterator<Item = impl udigest_encoding::Digestable>,
    out: &mut [u8],
) -> Result<(), digest::InvalidOutputSize>
where
    D: digest::VariableOutput + digest::Update,
{
    let mut hash = BufferUpdate(D::new(out.len())?);
    let mut encoder =
        udigest_encoding::encoding::EncodeList::new(&mut hash).with_tag(b"udigest.list");
    for value in iter {
        let item_encoder = encoder.add_item();
        value.unambiguously_encode(item_encoder);
    }
    encoder.finish();
    hash.0
        .finalize_variable(out)
        .map_err(|_| digest::InvalidOutputSize)
}
