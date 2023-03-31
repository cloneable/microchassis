#![deny(unsafe_code, rust_2018_idioms)]
#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    clippy::panic,
    clippy::unseparated_literal_suffix,
    clippy::unwrap_used,
    // clippy::expect_used, // TODO: revisit
    clippy::unwrap_in_result,
)]
#![allow(
    dead_code, // TODO: remove
    clippy::cargo_common_metadata, // TODO: revisit
    clippy::missing_const_for_fn,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    clippy::unnecessary_wraps,
    clippy::use_self,
    clippy::unwrap_in_result, // TODO: revisit
    clippy::multiple_crate_versions,
    clippy::needless_pass_by_value,
)]
#![cfg_attr(not(feature = "std"), no_std)]

mod allocator;
pub mod error;
mod jemalloc;
pub mod jeprof;
