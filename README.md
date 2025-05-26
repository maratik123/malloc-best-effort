# Best effort memory allocator

[![Latest Version]][crates.io] [![Documentation]][docs.rs]

GlobalAllocator implementation best suited for target platform

It uses [tcmalloc-better] on linux (x86_64, aarch64) and [mimalloc] on other platforms.
Both wrappers are based on general-purpose, performance-oriented allocators built by Google and Microsoft respectively.

## Usage

* Put to your `src/main.rs`:
```rust
use malloc_best_effort::BEMalloc;

#[global_allocator]
static GLOBAL: BEMalloc = BEMalloc::new();

fn main() {
    BEMalloc::init();
    
    // Rest of main
}
```

* Put to `build.rs` to workaround mimalloc build dependencies:
```rust
use std::borrow::Cow;
use std::env;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("target_os not defined!");
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").expect("target_arch not defined!"); // on armv6 we need to link with libatomic

    if target_os == "linux" && target_arch == "arm" {
        // Embrace the atomic capability library across various platforms.
        // For instance, on certain platforms, llvm has relocated the atomic of the arm32 architecture to libclang_rt.builtins.a
        // while some use libatomic.a, and others use libatomic_ops.a.
        let atomic_name = match env::var("DEP_ATOMIC") {
            Ok(atomic_name) => Cow::Owned(atomic_name),
            Err(_) => Cow::Borrowed("atomic"),
        };
        println!("cargo:rustc-link-lib={atomic_name}");
    }

    // Link with libs needed on Windows
    if target_os == "windows" {
        // https://github.com/microsoft/mimalloc/blob/af21001f7a65eafb8fb16460b018ebf9d75e2ad8/CMakeLists.txt#L487

        for lib in ["psapi", "shell32", "user32", "advapi32", "bcrypt"] {
            println!("cargo:rustc-link-lib={lib}");
        }
    }
}
```

## Requirements

A __C__/__C++__ compilers are required for building allocator with cargo.

[tcmalloc-better]: https://crates.io/crates/tcmalloc-better
[mimalloc]: https://crates.io/crates/mimalloc
[crates.io]: https://crates.io/crates/malloc-best-effort
[Latest Version]: https://img.shields.io/crates/v/malloc-best-effort.svg
[Documentation]: https://docs.rs/malloc-best-effort/badge.svg
[docs.rs]: https://docs.rs/malloc-best-effort
