# Changelog

## Unreleased future version

Breaking Changes

* Drop the `OctetsRef` trait and introduce a new `Octets` trait that takes
  over its purpose. This requires Rust 1.65.0. ([#12])
* Change the signature of `Octets::range` to use a range and drop all the
  convenience methods. ([#13])
* Split conversion from an octets builder to an immutable octets sequence
  off of `OctetsBuilder` to the new `FreezeBuilder` trait. ([#25])
* Dropped the `len` and `is_empty` methods from the `OctetsBuilder` trait.
  These can be requested via `AsRef<[u8]>` if necessary. ([#20])
* Rearranged module structure:
  * broke up `traits` into `octets` and `builder`,
  * renamed `types` to `array`, and
  * moved `SmallOctets` to `octets`. ([#18])
* The optional traits `SerializeOctets` and `DeserializeOctets` have been
  redesigned for greater flexibility. ([#21])

New

* Added `Parser::parse_parser` that allows parsing a given number of octets
  and return them as a cloned parser. ([#10])
* Add methods to `Parser` to parse 64 and 128 bit integers. ([#11])
* Added support for the `heapless` crate. ([#19])

Bug Fixes

Other Changes

[#10]: https://github.com/NLnetLabs/octseq/pull/10
[#11]: https://github.com/NLnetLabs/octseq/pull/11
[#12]: https://github.com/NLnetLabs/octseq/pull/12
[#13]: https://github.com/NLnetLabs/octseq/pull/13
[#18]: https://github.com/NLnetLabs/octseq/pull/18
[#19]: https://github.com/NLnetLabs/octseq/pull/19
[#21]: https://github.com/NLnetLabs/octseq/pull/21
[#25]: https://github.com/NLnetLabs/octseq/pull/25


## 0.1.0

Released 2022-08-18.

Initial release.

