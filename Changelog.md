# Changelog

## Unreleased future version

Breaking Changes

* Drop the `OctetsRef` trait and introduce a new `Octets` trait that takes
  over its purpose. This requires Rust 1.65.0. ([#12])
* Change the signature of `Octets::range` to use a range and drop all the
  convenience methods. ([#13])

New

* Added `Parser::parse_parser` that allows parsing a given number of octets
  and return them as a cloned parser. ([#10])
* Add methods to `Parser` to parse 64 and 128 bit integers. ([#11])

Bug Fixes

Other Changes

[#10]: https://github.com/NLnetLabs/octseq/pull/10
[#11]: https://github.com/NLnetLabs/octseq/pull/11
[#12]: https://github.com/NLnetLabs/octseq/pull/12
[#13]: https://github.com/NLnetLabs/octseq/pull/13


## 0.1.0

Released 2022-08-18.

Initial release.

