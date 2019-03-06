rust-t1ha
===

An implementation of the [T1AH (Fast Positive Hash)](https://github.com/leo-yuriev/t1ha) hash function.

## Briefly, it is a portable 64-bit hash function:

* Intended for 64-bit little-endian platforms, predominantly for Elbrus and x86_64, but portable and without penalties it can run on any 64-bit CPU.
* In most cases up to 15% faster than StadtX hash, xxHash, mum-hash, metro-hash, etc. and all others portable hash-functions (which do not use specific hardware tricks).
* Provides a set of terraced hash functions.
* Currently not suitable for cryptography.
* Licensed under [zlib License](https://en.wikipedia.org/wiki/Zlib_License).

# Usage

To include this crate in your program, add the following to your `Cargo.toml`:

```toml
[dependencies]
t1ha = "0.1"
```

## Using t1ha in a HashMap

The `T1haHashMap` type alias is the easiest way to use the standard library’s `HashMap` with `t1ha`.

```rust
use t1ha::T1haHashMap;

let mut map = T1haHashMap::default();
map.insert(1, "one");
map.insert(2, "two");

map = T1haHashMap::with_capacity_and_hasher(10, Default::default());
map.insert(1, "one");
map.insert(2, "two");
```

**Note:** the standard library’s `HashMap::new` and `HashMap::with_capacity` are only implemented for the `RandomState` hasher, so using `Default` to get the hasher is the next best option.

## Using t1ha in a HashSet

Similarly, `T1haHashSet` is a type alias for the standard library’s `HashSet` with `t1ha.

```rust
use t1ha::T1haHashSet;

let mut set = T1haHashSet::default();
set.insert(1);
set.insert(2);

set = T1haHashSet::with_capacity_and_hasher(10, Default::default());
set.insert(1);
set.insert(2);
```
