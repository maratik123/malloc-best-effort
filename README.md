# Best effort memory allocator

[![Latest Version]][crates.io] [![Documentation]][docs.rs]

GlobalAllocator implementation best suited for target platform

It uses [tcmalloc-better] on linux (x86_64, aarch64) and [mimalloc] on other platforms.
Both wrappers are based on general-purpose, performance-oriented allocators built by Google and Microsoft respectively.

## Usage

```rust
use malloc_best_effort::BEMalloc;

#[global_allocator]
static GLOBAL: BEMalloc = BEMalloc::new();
```

## Requirements

A __C__/__C++__ compilers are required for building allocator with cargo.

[tcmalloc-better]: https://crates.io/crates/tcmalloc-better
[mimalloc]: https://crates.io/crates/mimalloc
[crates.io]: https://crates.io/crates/malloc-best-effort
[Latest Version]: https://img.shields.io/crates/v/malloc-best-effort.svg
[Documentation]: https://docs.rs/malloc-best-effort/badge.svg
[docs.rs]: https://docs.rs/malloc-best-effort
