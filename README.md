# rust-t1ha [![travis](https://travis-ci.org/flier/rust-t1ha.svg?branch=master)](https://travis-ci.org/flier/rust-t1ha) [![appveyor](https://ci.appveyor.com/api/projects/status/c5hli7g424r0g49n?svg=true)](https://ci.appveyor.com/project/flier/rust-t1ha) [![crate](https://img.shields.io/crates/v/t1ha.svg)](https://crates.io/crates/t1ha) [![docs](https://docs.rs/t1ha/badge.svg)](https://docs.rs/t1ha)

An implementation of the [T1HA (Fast Positive Hash)](https://github.com/leo-yuriev/t1ha) hash function.

### Briefly, it is a portable 64-bit hash function:

* Intended for 64-bit little-endian platforms, predominantly for Elbrus and x86_64, but portable and without penalties it can run on any 64-bit CPU.
* In most cases up to 15% faster than StadtX hash, xxHash, mum-hash, metro-hash, etc. and all others portable hash-functions (which do not use specific hardware tricks).
* Provides a set of terraced hash functions.
* Currently not suitable for cryptography.
* Licensed under [zlib License](https://en.wikipedia.org/wiki/Zlib_License).

## Usage

To include this crate in your program, add the following to your `Cargo.toml`:

```toml
[dependencies]
t1ha = "0.1"
```

### Using `t1ha` in a `HashMap`

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

### Using `t1ha` in a `HashSet`

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

## Performance

`t1ha` can use AES, AVX or AVX2 instructions as hardware acceleration.

 | Implementation          | Platform/CPU                           |
 | :---------------------- | :------------------------------------- |
 | `t1ha0_ia32aes_avx()`   | x86 with AES-NI and AVX extensions     |
 | `t1ha0_ia32aes_avx2()`  | x86 with AES-NI and AVX2 extensions    |
 | `t1ha0_ia32aes_noavx()` | x86 with AES-NI without AVX extensions |
 | `t1ha0_32le()`          | 32-bit little-endian                   |
 | `t1h0a_32be()`          | 32-bit big-endian                      |
 | `t1ha1_le()`            | 64-bit little-endian                   |
 | `t1ha1_be()`            | 64-bit big-endian                      |
 | `t1ha2_atonce()`        | 64-bit little-endian                   |

You could choose the right implementation base on your `target_cpu`.

 > $ RUSTFLAGS="-C target-cpu=native" cargo build

### Benchmark

`rust-t1ha` provide [a rough performance comparison](https://www.reddit.com/r/rust/comments/ayla9m/rust_implementation_for_t1ha_fast_positive_hash/) to other `Rust` implemenation of non-cryptographic hash functions, you can run the benchmark base on your envrionment and usage scenario.

```sh
$ RUSTFLAGS="-C target-cpu=native" cargo bench
```

### Native `t1ha` Library

`rust-t1ha` major focus `Rust` implementation, if you intent to use the origin native `t1ha` library, please check [rust-fasthash](https://github.com/flier/rust-fasthash) project and it's benchmark, which provides a suite of non-cryptographic hash functions from [SMHasher](https://github.com/rurban/smhasher/).
