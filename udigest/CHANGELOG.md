## v0.3.0
* Add `#[udigest(as = ...)]` attribute and `DeriveAs` trait [#12]

[#12]: https://github.com/dfns/udigest/pull/12

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

[#4]: https://github.com/dfns/udigest/pull/4
[#5]: https://github.com/dfns/udigest/pull/5
[#6]: https://github.com/dfns/udigest/pull/6

## v0.1.0

The first release!
