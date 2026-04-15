# udigest-encoding

Core crate for unambiguous encoding used by the udigest ecosystem.

This crate provides:
- Digestable trait
- encoding module
- as_ module
- inline_struct macro (feature: inline-struct)
- derive re-export (feature: derive)

It is dependency-free by default and does not depend on the digest crate.
For hashing helpers like hash and hash_vof, use the udigest crate.
