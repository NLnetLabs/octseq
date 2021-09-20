# octets

This crate provides a set of basic traits that allow defining types that
are generic over a variable length sequence of octets (or, vulgo: bytes).
It implements these traits for most commonly used types of such sequences
and provides a array-backed type for use in a no-std environment.

## Types of Octet Sequences

There are two fundamental types of octet sequences. If a sequence
contains content of a constant size, we call it simply ‘octets.’ If the
sequence is actually a buffer into which octets can be placed, it is
called an `octets builder.`


## Octets and Octets References

There is no special trait for octets, we simply use `AsRef<[u8]>` for
imutable octets or `AsMut<[u8]>` if the octets of the sequence can be
manipulated (but the length is still fixed). This way, any type
implementing these traits can be used already. The trait [`OctetsExt`]
has been defined to collect additional methods that aren’t available via
plain `AsRef<[u8]>`.

A reference to an octets type implements [`OctetsRef`]. The main purpose
of this trait is to allow taking a sub-sequence, called a ‘range’,
out of the octets in the cheapest way possible. For most types, ranges
will be octet slices `&[u8]` but some shareable types (most notably
`bytes::Bytes`) allow ranges to be owned values, thus avoiding the
lifetime limitations a slice would bring.

One type is special in that it is its own octets reference: `&[u8]`,
referred to as an _octets slice_ here. This means that you
always use an octets slice irregardless whether a type is generic over
an octets sequence or an octets reference.

The [`OctetsRef`] trait is separate because of limitations of lifetimes
in traits. It has an associated type `OctetsRef::Range` that defines the
type of a range. When using the trait as a trait bound for a generic type,
you will typically bound a reference to this type. For instance, a generic
function taking part out of some octets and returning a reference to it
could be defined like so:

```
## use octets::OctetsRef;

fn take_part<'a, Octets>(
    src: &'a Octets
) -> <&'a Octets as OctetsRef>::Range
where &'a Octets: OctetsRef {
    unimplemented!()
}
```

The where clause demands that whatever octets type is being used, a
reference to it must be an octets ref. The return value refers to the
range type defined for this octets ref. The lifetime argument is
necessary to tie all these references together.


## Octets Builders

Octets builders and their [`OctetsBuilder`] trait are comparatively
straightforward. They represent a buffer to which octets can be appended.
Whether the buffer can grow to accommodate appended data depends on the
underlying type. Because it may not, all such operations may fail with a
[`ShortBuf`] error.

The [`EmptyBuilder`] trait marks a type as being able to create a new,
empty builder.


## Conversion Traits

A series of special traits allows converting octets into octets builder
and vice versa. They pair octets with their natural builders via
associated types. These conversions are always cyclic, i.e., if an
octets value is converted into a builder and then that builder is
converted back into an octets value, the initial and final octets value
have the same type.


## Contributing

If you have comments, proposed changes, or would like to contribute,
please open an issue in the [Github repository]. In particular, if you
would like to use the crate but it is missing functionality for your use
case, we would love to hear from you!

[Github repository]: (https://github.com/NLnetLabs/octets)

## License

The _octets_ crate is distributed under the terms of the BSD-3-clause license.
See LICENSE for details.


