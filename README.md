![License](https://img.shields.io/crates/l/udigest.svg)
[![Docs](https://docs.rs/udigest/badge.svg)](https://docs.rs/udigest)
[![Crates io](https://img.shields.io/crates/v/udigest.svg)](https://crates.io/crates/udigest)

## Unambiguously digest structured data

`udigest` provides utilities for unambiguous hashing the structured data. Structured
data can be anything that implements `Digestable` trait:

* `str`, `String`, `CStr`, `CString`
* Integers:
  `i8`, `i16`, `i32`, `i64`, `i128`,
  `u8`, `u16`, `u32`, `u64`, `u128`,
  `char`
* Containers: `Box`, `Arc`, `Rc`, `Cow`, `Option`, `Result`
* Collections: arrays, slices, `Vec`, `LinkedList`, `VecDeque`, `BTreeSet`, `BTreeMap`

The trait is intentionally not implemented for certain types:

* `HashMap`, `HashSet` as they can not be traversed in deterministic order
* `usize`, `isize` as their byte size varies on different platforms

The `Digestable` trait can be implemented for the struct using a macro:
```rust
#[derive(udigest::Digestable)]
struct Person {
    name: String,
    job_title: String,
}
let alice = Person {
    name: "Alice".into(),
    job_title: "cryptographer".into(),
};

let hash = udigest::hash::<sha2::Sha256, _>(&alice);
```

The crate intentionally does not try to follow any existing standards for unambiguous
encoding. The format for encoding was designed specifically for `udigest` to provide
a better usage experience in Rust. The details of encoding format can be found in
`encoding` module.

### Features
* `digest` enables support of hash functions that implement `digest` traits \
   If feature is not enabled, the crate is still usable via `Digestable` trait that
   generically implements unambiguous encoding
* `std` implements `Digestable` trait for types in standard library
* `alloc` implements `Digestable` trait for type in `alloc` crate
* `derive` enables `Digestable` proc macro
