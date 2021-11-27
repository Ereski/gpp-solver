//! This module contains reexported symbols that are imported from different places depending on
//! how the crate was compiled.
//!
//! Modules:
//!
//! - [`iter`]: rust's `iter`. Can come from `std` or the `core` crate.
//!
//! Macros:
//!
//! - [`test`]: test macro for asynchronous code. Can come from `futures-test`, `tokio`, or the
//!   `async-std` crates. Only available during testing.
//!
//! Structs:
//!
//! - [`Box`]: rust's `Box` struct. Can come from `std` or the `alloc` crate.
//! - [`Map`]: one of rust's map types, either `HashMap` from `std` or `BTreeMap` from the `alloc`
//!   crate.
//! - [`Mutex`]: a futures-aware mutex. Can come from `futures`, `tokio`, or the `async-lock`
//!   crates.
//! - [`NonZeroUsize`]: rust's `NonZeroUsize` struct. Can come from `std` or the `core` crate.
//! - [`Set`]: one of rust's set types, either `HashSet` from `std` or `BTreeSet` from the `alloc`
//!   crate.
//! - [`Vec`]: rust's `Vec` struct. Can come from `std` or the `alloc` crate.
//!
//! Traits:
//!
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
        iter::{self, IntoIterator, Iterator},
        num::NonZeroUsize,
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
        vec::Vec,
    };
    pub use core::{
        iter::{self, IntoIterator, Iterator},
        num::NonZeroUsize,
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
