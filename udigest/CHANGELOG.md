## v0.4.0
* Make `Digestable` trait `dyn`-compatible [#25]
* Update hash functions to accept `&dyn Digestable` [#25]

[#25]: https://github.com/LFDT-Lockness/udigest/pull/25

## v0.3.1
* Fix docs.rs build [#24]

[#24]: https://github.com/LFDT-Lockness/udigest/pull/24

## v0.3.0
* `rename` attribute is not ignored anymore on enum variants by `derive(udigest::Digestable)`

  **Possibly breaking change**: `#[udigest(rename = ..)]` attribute on enum variant was silently
  ignored by previous versions of the library (except `v0.2.4` which throws an error if this attr
  is used). If you used this attr on enum variant, encoding of your enum **will be changed** once you
  upgrade to this version.

  If you need a stable encoding:
  1. Upgrade to `v0.2.4` first and compile your project
  2. If anywhere in the project you're using `rename` attribute on enum variant, derive macro will produce an error
     1. For each such error, remove `rename` attribute as it has been silently ignored by the library anyways
  3. Now you can upgrade to `v0.3.0` while preserving encoding format compatible with previous version of your application

  This does not affect you if you haven't been using `rename` attribute on enum variant or you don't
  care about stability of encoding between library versions.

  Example of affected code:
  ```rust
  #[derive(udigest::Digestable)]
  enum Shape {
      #[udigest(rename = "Rectangular")] // this attr was silently ignored
      Rect {
          #[udigest(rename = "width")] // this attr IS NOT affected, it has always been taken into account
          w: u16,
          h: u16,
      },
  }
  ```
* Derive macro statically asserts that all fields and enum variant have unique encoding

  For instance, this code:
  ```rust
  #[derive(udigest::Digestable)]
  struct Person {
      name: String,
      #[udigest(rename = "name")]
      surname: String,
  }
  ```

  will now produce an error as two different fields have the same encoding.

  **Possibly breaking change**: in previous versions of the library, `rename` attribute used
  to accept any expression that implements `AsRef<[u8]>`. Now, since `AsRef` is not available
  in `const` context, the attribute only accepts any const expressions producing a value of type:
  `&str`, `&[u8]`, `[u8; N]`, or (if `alloc` feature is enabled) `String`, `Vec<u8>`, `Cow<str>`,
  `Cow<[u8]>`.
* Improve errors readability

See [PR #23](https://github.com/LFDT-Lockness/udigest/pull/23).

## v0.2.4
* Produce compile error if `rename` attr is used on enum variant [#22] \
  Refer to [issue #21] to get more details on this

[#22]: https://github.com/LFDT-Lockness/udigest/pull/22
[issue #21]: https://github.com/LFDT-Lockness/udigest/issues/21

## v0.2.3
* Relax bounds in `DigestAs` implementations: allow `?Sized` types [#18]
* Implement `Digestable` for `core::convert::Infallible` (a.k.a. Never type) [#19]

[#18]: https://github.com/LFDT-Lockness/udigest/pull/18
[#19]: https://github.com/LFDT-Lockness/udigest/pull/19

## v0.2.2
* Update links in crate settings [#14]

[#14]: https://github.com/LFDT-Lockness/udigest/pull/14

## v0.2.1
* Add `#[udigest(as = ...)]` attribute and `DeriveAs` trait [#12]

[#12]: https://github.com/LFDT-Lockness/udigest/pull/12

## v0.2.0
* Breaking change: remove `udigest::Tag` [#4]
* Breaking change: rename `udigest::udigest` function to `udigest::hash` [#4]
* Breaking change: change format of integers encoding [#5]
* Add support of all hash functions compatible with `digest` crate:
  hash functions with fixed output, with extendable output, and with
  variable output [#4]
* Add `udigest::inline_struct!` macro [#4]
* Add support for digesting `usize`/`isize` [#5]
* fix: handle cases when `EncodeValue` is dropped without being used [#4]
* fix: proc macro used to cause clippy warnings in certain cases [#6]

[#4]: https://github.com/LFDT-Lockness/udigest/pull/4
[#5]: https://github.com/LFDT-Lockness/udigest/pull/5
[#6]: https://github.com/LFDT-Lockness/udigest/pull/6

## v0.1.0

The first release!
