# Persistent key-value maps in Rust

I was reading the source code for [rpds][] and thought its implementation of
hash-maps might be more complex than necessary, so I did this quick experiment
to see what a minimal implementation might look like.

[rpds]: https://github.com/orium/rpds

I have neither tested nor benchmarked this code; all I can say is it compiles.
You probably should just be using [rpds][]. I did this primarily for my own
learning.

Like rpds, this repo implements the hash-array mapped trie (HAMT) data
structure, as described in "[Ideal Hash Trees][]" by Phil Bagwell.

[Ideal Hash Trees]: https://infoscience.epfl.ch/record/64398/files/idealhashtrees.pdf

This data structure is a classic hash-table except that it uses a sparse array
to hold the table, which means it behaves like a hash-table with 2^64 buckets.
That means it never needs resizing, but also means that if the hash function is
decent, collisions should approximately never happen.

One thing I discovered while doing this experiment is that a persistent sparse
array could be a useful data structure in its own right. It's like a `HashMap`
but specialized to integer keys, which means you can avoid storing an extra
copy of the key in each bucket.

The main difference I was experimenting with, compared to rpds, was the use of
[linear probing][] to deal with hash collisions. As of this writing, rpds uses
chaining via singly-linked lists at the leaves of its HAMT. At first glance
that seems to me to be more complex than necessary, although there definitely
could be factors I'm missing.

[linear probing]: https://en.wikipedia.org/wiki/Linear_probing

If collisions are very unlikely, then any implementation complexity or
performance cost for supporting them is hard to justify. So far with this
experiment I've convinced myself that linear probing is very simple to
implement for insert/get operations, so it remains possibly attractive as an
alternative to chaining.

However, I got tired of thinking about this while implementing a remove
operation, and have not tested or benchmarked any of this implementation.
So it's still easily possible that chaining is a better choice for reasons I
haven't identified yet.

The other reason I wanted to try writing this myself was to experiment with
different ways to minimize memory usage when implementing a HAMT in Rust.
However, in the end I nearly reinvented the same types that rpds uses. I can
see some opportunities to save space using unsafe code and possibly feature-
gated Rust nightly APIs, but I decided not to try those.

My representation does highlight an optimization that I think rustc could
apply automatically which would help data structures like this one. There's
an existing optimization where `Option<Box<T>>` is the same size as `Box<T>`
because the compiler knows that the pointer in `Box<T>` can never be null, so
the `None` case can be represented as a null pointer without a separate tag.
In this case I have an `enum` with two variants, which both hold pointers to
types with alignment constraints. That means the least-significant bit in both
pointers is always 0, so that bit could be used to store the tag indicating
which variant is stored there.
