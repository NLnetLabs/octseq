# _octseq_ - Generic Octets Sequences

Such sequences require a varying amount of memory and different use cases
suggest different strategies to manage that memory: references to `u8`
slices, `Vec<u8>`, `Arc<[u8]>` are examples provided by the standard library.

In many cases, underlying memory management strategies don’t matter for
composite types storing such octets sequences or for code manipulating
them. Instead of insisting on a specific representation, these types and
functions can be generic over the representation and describe the
necessary properties through trait bounds.

This crate provides a set of such traits that describe basic functionality
of octets sequences as well as buffers to construct such sequences,
termed _octets builders,_ conversions between sequences and builders.

It also provides some helper types that simplify common tasks such as
parsing data from an octets sequence.

For details, please see the [crate documentation on
docs.rs](https://docs.rs/octseq).

## Contributing

If you have comments, proposed changes, or would like to contribute,
please open an issue in the [Github repository]. In particular, if you
would like to use the crate but it is missing functionality for your use
case, we would love to hear from you!

[Github repository]: (https://github.com/NLnetLabs/octseq)

## License

The _octseq_ crate is distributed under the terms of the BSD-3-clause license.
See LICENSE for details.


