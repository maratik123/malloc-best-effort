#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! Best effort memory allocator for Rust
//!
//! Selects suitable allocator depending on target OS and architecture.
//!
//! ## Usage
//! * Put to your `src/main.rs`:
//! ```rust,ignore
//! use malloc_best_effort::BEMalloc;
//!
//! #[global_allocator]
//! static GLOBAL: BEMalloc = BEMalloc::new();
//!
//! fn main() {
//!     BEMalloc::init();
//!
//!     // Rest of main
//! }
//! ```
//!
//! * Put to `build.rs` to workaround mimalloc build dependencies:
//! ```rust,ignore
//! use std::borrow::Cow;
//! use std::env;
//!
//! fn main() {
//!     let target_os = env::var("CARGO_CFG_TARGET_OS").expect("target_os not defined!");
//!     let target_arch = env::var("CARGO_CFG_TARGET_ARCH").expect("target_arch not defined!"); // on armv6 we need to link with libatomic
//!
//!     if target_os == "linux" && target_arch == "arm" {
//!         // Embrace the atomic capability library across various platforms.
//!         // For instance, on certain platforms, llvm has relocated the atomic of the arm32 architecture to libclang_rt.builtins.a
//!         // while some use libatomic.a, and others use libatomic_ops.a.
//!         let atomic_name = match env::var("DEP_ATOMIC") {
//!             Ok(atomic_name) => Cow::Owned(atomic_name),
//!             Err(_) => Cow::Borrowed("atomic"),
//!         };
//!         println!("cargo:rustc-link-lib={atomic_name}");
//!     }
//!
//!     // Link with libs needed on Windows
//!     if target_os == "windows" {
//!         // https://github.com/microsoft/mimalloc/blob/af21001f7a65eafb8fb16460b018ebf9d75e2ad8/CMakeLists.txt#L487
//!
//!         for lib in ["psapi", "shell32", "user32", "advapi32", "bcrypt"] {
//!             println!("cargo:rustc-link-lib={lib}");
//!         }
//!     }
//! }
//! ```
//!
//! ## Feature flags
#![doc = document_features::document_features!()]

#[cfg(not(all(
    target_os = "linux",
    any(target_arch = "x86_64", target_arch = "aarch64")
)))]
mod mimalloc {
    pub(crate) use mimalloc::MiMalloc as BEMallocImpl;

    #[inline]
    pub(crate) fn init_impl() {}
}
#[cfg(all(
    target_os = "linux",
    any(target_arch = "x86_64", target_arch = "aarch64")
))]
mod tcmalloc {
    pub(crate) use tcmalloc_better::TCMalloc as BEMallocImpl;

    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    #[inline]
    pub(crate) fn init_impl() {
        BEMallocImpl::process_background_actions_thread();
    }

    #[cfg(not(feature = "std"))]
    #[cfg_attr(docsrs, doc(cfg(not(feature = "std"))))]
    #[inline]
    pub(crate) fn init_impl() {}
}

#[cfg(not(all(
    target_os = "linux",
    any(target_arch = "x86_64", target_arch = "aarch64")
)))]
use crate::mimalloc::BEMallocImpl;
#[cfg(not(all(
    target_os = "linux",
    any(target_arch = "x86_64", target_arch = "aarch64")
)))]
use crate::mimalloc::init_impl;
#[cfg(all(
    target_os = "linux",
    any(target_arch = "x86_64", target_arch = "aarch64")
))]
use crate::tcmalloc::BEMallocImpl;
#[cfg(all(
    target_os = "linux",
    any(target_arch = "x86_64", target_arch = "aarch64")
))]
use crate::tcmalloc::init_impl;
use core::alloc::{GlobalAlloc, Layout};

/// A memory allocator that can be registered as the standard library’s default
/// through the `#[global_allocator]` attribute.
pub struct BEMalloc {
    alloc_impl: BEMallocImpl,
}

unsafe impl GlobalAlloc for BEMalloc {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { self.alloc_impl.alloc(layout) }
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { self.alloc_impl.dealloc(ptr, layout) }
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        unsafe { self.alloc_impl.alloc_zeroed(layout) }
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        unsafe { self.alloc_impl.realloc(ptr, layout, new_size) }
    }
}

impl Default for BEMalloc {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl BEMalloc {
    /// Create new instance of allocator.
    #[inline]
    pub const fn new() -> Self {
        Self {
            alloc_impl: BEMallocImpl,
        }
    }
    /// Start allocator background job. Should be called in `main()` function.
    #[inline]
    pub fn init() {
        init_impl();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_frees_allocated_memory() {
        unsafe {
            let layout = Layout::from_size_align(8, 8).unwrap();
            let alloc = BEMalloc::new();

            let ptr = alloc.alloc(layout);
            alloc.dealloc(ptr, layout);
        }
    }

    #[test]
    fn it_frees_allocated_memory_with_init() {
        unsafe {
            let layout = Layout::from_size_align(8, 8).unwrap();
            let alloc = BEMalloc::new();
            BEMalloc::init();

            let ptr = alloc.alloc(layout);
            alloc.dealloc(ptr, layout);
        }
    }

    #[test]
    fn it_frees_allocated_big_memory() {
        unsafe {
            let layout = Layout::from_size_align(1 << 20, 32).unwrap();
            let alloc = BEMalloc::new();

            let ptr = alloc.alloc(layout);
            alloc.dealloc(ptr, layout);
        }
    }

    #[test]
    fn it_frees_zero_allocated_memory() {
        unsafe {
            let layout = Layout::from_size_align(8, 8).unwrap();
            let alloc = BEMalloc::new();

            let ptr = alloc.alloc_zeroed(layout);
            alloc.dealloc(ptr, layout);
        }
    }

    #[test]
    fn it_frees_zero_allocated_big_memory() {
        unsafe {
            let layout = Layout::from_size_align(1 << 20, 32).unwrap();
            let alloc = BEMalloc::new();

            let ptr = alloc.alloc_zeroed(layout);
            alloc.dealloc(ptr, layout);
        }
    }

    #[test]
    fn it_frees_reallocated_memory() {
        unsafe {
            let layout = Layout::from_size_align(8, 8).unwrap();
            let new_size = 16;
            let new_layout = Layout::from_size_align(new_size, layout.align()).unwrap();
            let alloc = BEMalloc::new();

            let ptr = alloc.alloc(layout);
            let ptr = alloc.realloc(ptr, layout, new_size);
            alloc.dealloc(ptr, new_layout);
        }
    }

    #[test]
    fn it_frees_reallocated_big_memory() {
        unsafe {
            let layout = Layout::from_size_align(1 << 20, 32).unwrap();
            let new_size = 2 << 20;
            let new_layout = Layout::from_size_align(new_size, layout.align()).unwrap();
            let alloc = BEMalloc::new();

            let ptr = alloc.alloc(layout);
            let ptr = alloc.realloc(ptr, layout, new_size);
            alloc.dealloc(ptr, new_layout);
        }
    }
}
