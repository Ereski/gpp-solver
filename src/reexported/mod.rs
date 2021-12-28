//! This module contains reexported symbols that are imported from different places depending on
//! how the crate was compiled.
//!
//! Modules:
//!
//! - [`iter`]: rust's `iter` module. Can come from `std` or the `core` crate.
//! - [`mem`]: rust's `mem` module. Can come from `std` or the `alloc` crate.
//!
//! Macros:
//!
//! - [`format`]: rust's `format` macro. Can come from `std` or the `alloc` crate.
//! - [`test`]: test macro for asynchronous code. Can come from `futures-test`, `tokio`, or the
//!   `async-std` crates. Only available during testing.
//!
//! Structs:
//!
//! - [`Arc`]: rust's `Arc` struct. Can come from `std` or the `alloc` crate.
//! - [`Box`]: rust's `Box` struct. Can come from `std` or the `alloc` crate.
//! - [`Map`]: one of rust's map types, either `HashMap` from `std` or `BTreeMap` from the `alloc`
//!   crate.
//! - [`Mutex`]: a futures-aware mutex. Can come from `futures`, `tokio`, or the `async-lock`
//!   crates.
//! - [`NonZeroUsize`]: rust's `NonZeroUsize` struct. Can come from `std` or the `core` crate.
//! - [`Pin`]: rust's `Pin` struct. Can come from `std` or the `core` crate.
//! - [`Set`]: one of rust's set types, either `HashSet` from `std` or `BTreeSet` from the `alloc`
//!   crate.
//! - [`Vec`]: rust's `Vec` struct. Can come from `std` or the `alloc` crate.
//!
//! Traits:
//!
//! - [`Future`]: rust's `Future` trait. Can come from `std` or the `core` crate.
//! - [`IntoIterator`]: rust's `IntoIterator` trait. Can come from `std` or the `core` crate.
//! - [`Iterator`]: rust's `IntoIterator` trait. Can come from `std` or the `core` crate.

#[cfg(not(any(
    feature = "futures-lock",
    feature = "tokio-lock",
    feature = "async-std-lock"
)))]
compile_error!(
    "Must enable one of: futures-lock, tokio-lock, or async-std-lock"
);

#[cfg(test)]
pub use crate::reexported::test::*;

#[cfg(test)]
mod test;

feature_cfg! {
    for "std";

    use std::collections::{HashMap, HashSet};

    pub use std::{
        boxed::Box,
        format,
        future::Future,
        iter::{self, IntoIterator, Iterator},
        mem,
        num::NonZeroUsize,
        pin::Pin,
        sync::Arc,
        vec::Vec,
    };

    pub type Map<K, V> = HashMap<K, V>;
    pub type Set<T> = HashSet<T>;
}

feature_cfg! {
    for !"std";

    extern crate alloc;

    use alloc::collections::{BTreeMap, BTreeSet};

    pub use alloc::{
        boxed::Box,
        format,
        sync::Arc,
        vec::Vec,
    };
    pub use core::{
        future::Future,
        iter::{self, IntoIterator, Iterator},
        mem,
        num::NonZeroUsize,
        pin::Pin,
    };

    pub type Map<K, V> = BTreeMap<K, V>;
    pub type Set<T> = BTreeSet<T>;
}

feature_cfg! {
    for "futures-lock";

    pub use futures::lock::Mutex;
}

feature_cfg! {
    for "tokio-lock";

    pub use tokio::sync::Mutex;
}

feature_cfg! {
    for "async-std-lock";

    pub use async_lock::Mutex;
}
