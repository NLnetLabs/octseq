# Changelog

## Unreleased next version

Breaking changes

New

* Added an impl of `SerialzeOctets` for plain, unsized `[u8]`. ([#54])

Bug fixes

Other changes

[#54]: https://github.com/NLnetLabs/octseq/pull/54


## 0.5.0

Released 2024-01-30.

Breaking changes

* Removed the impls of `Borrow<[u8]>` for `Str<_>` and `StrBuilder<_>`
  (`Borrow<str>` is still there). ([#52])

New

* Added an impl of `OctetsFrom<_>` for `Smallvec<_>`. ([#52])
* Added an impl of `OctetsFrom<_>` for `Str<_>`. ([#51])

[#51]: https://github.com/NLnetLabs/octseq/pull/51
[#52]: https://github.com/NLnetLabs/octseq/pull/52


## 0.4.0

Released 2024-01-09

Breaking changes

* Update _heapless_ dependency to 0.8. ([#47] by
  [@reitermarkus])

[#47]: https://github.com/NLnetLabs/octseq/pull/47
[@reitermarkus]: https://github.com/reitermarkus


## 0.3.2

Released 2023-12-28.

Bug fixes

* Don’t enable _bytes’_ `std` feature by default. ([#45] by
  [@reitermarkus])

[#45]: https://github.com/NLnetLabs/octseq/pull/45
[@reitermarkus]: https://github.com/reitermarkus


## 0.3.1

Release 2023-11-16.

New

* Added `Parser::with_range` and `Parser::try_with_range` that allow
  creating a parser positioned and length-limited according to a given
  range. ([#43])

[#43]: https://github.com/NLnetLabs/octseq/pull/43


## 0.3.0

Release 2023-10-18.

Breaking changes

* Change the lifetime of the range for a reference to the lifetime of the
  reference. ([#41] by [@xofyarg])
* Explicitly re-export select items at crate level rather than wildcard
  export everything. ([#39])

New

* Adds a `BuilderAppendError<_>` type alias that simplifies trait bounds
  for complex `FromBuilder` trait bounds. ([#38])

Bug fixes

* Fix `Parser::peek_all` to only return data up until the parser's
  length rather than all data. ([#40])

[#38]: https://github.com/NLnetLabs/octseq/pull/38
[#39]: https://github.com/NLnetLabs/octseq/pull/39
[#40]: https://github.com/NLnetLabs/octseq/pull/40
[#41]: https://github.com/NLnetLabs/octseq/pull/41
[@xofyarg]: https://github.com/xofyarg


## 0.2.0

Released 2023-05-12.

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
* The integer parsing methods on `Parser` have been renamed to make it
  clear they parse big-endian integers and new methods for parsing
  little-endian integers have been added. ([#35])
* The optional traits `SerializeOctets` and `DeserializeOctets` have been
  redesigned for greater flexibility. ([#21])

New

* Added `Parser::parse_parser` that allows parsing a given number of octets
  and return them as a cloned parser. ([#10])
* Add methods to `Parser` to parse 64 and 128 bit integers. ([#11])
* Added support for the `heapless` crate. ([#19])
* Added missing `Octets` implementation for `Array<N>`. ([#29])
* Added `Octets` implementation for `Arc<[u8]>`. ([#28])
* Added blanket implementations for `OctetsBuilder` and `Truncate` for
  mutable references of types that are `OctetsBuilder` and `Truncate`,
  respectively. ([#30])
* Added conversions from `&str` and `&[u8]` to `Str<[u8]>`. ([#31])
* Added `Array::resize_raw`. ([#32], [#33])

[#10]: https://github.com/NLnetLabs/octseq/pull/10
[#11]: https://github.com/NLnetLabs/octseq/pull/11
[#12]: https://github.com/NLnetLabs/octseq/pull/12
[#13]: https://github.com/NLnetLabs/octseq/pull/13
[#18]: https://github.com/NLnetLabs/octseq/pull/18
[#19]: https://github.com/NLnetLabs/octseq/pull/19
[#21]: https://github.com/NLnetLabs/octseq/pull/21
[#25]: https://github.com/NLnetLabs/octseq/pull/25
[#28]: https://github.com/NLnetLabs/octseq/pull/28
[#29]: https://github.com/NLnetLabs/octseq/pull/29
[#30]: https://github.com/NLnetLabs/octseq/pull/30
[#31]: https://github.com/NLnetLabs/octseq/pull/31
[#32]: https://github.com/NLnetLabs/octseq/pull/32
[#33]: https://github.com/NLnetLabs/octseq/pull/33
[#35]: https://github.com/NLnetLabs/octseq/pull/35


## 0.1.0

Released 2022-08-18.

Initial release.

