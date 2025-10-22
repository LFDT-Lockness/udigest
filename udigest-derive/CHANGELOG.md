## v0.4.0
* `rename` attribute now works on enum variants as it always should have been
* Derive macro produces an error if two fields or two enum variants have the same
  name encoding
* Improve errors readability

See [PR #23](https://github.com/LFDT-Lockness/udigest/pull/23).

## v0.3.2
* Produce compile error if `rename` attr is used on enum variant [#22] \
  Refer to [issue #21] to get more details on this

[#22]: https://github.com/LFDT-Lockness/udigest/pull/22
[issue #21]: https://github.com/LFDT-Lockness/udigest/issues/21

## v0.3.1
* Update links in crate settings [#14]

[#14]: https://github.com/LFDT-Lockness/udigest/pull/14

## v0.3.0
* Add `#[udigest(as = ...)]` attribute support [#12]

[#12]: https://github.com/LFDT-Lockness/udigest/pull/12

## v0.2.0
* Fix proc macro causing clippy warnings in certain cases [#6]

[#6]: https://github.com/LFDT-Lockness/udigest/pull/6

## v0.1.0

The first release!
