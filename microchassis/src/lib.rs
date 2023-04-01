// Copyright 2023 Folke Behrens
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
pub mod jeprof;
pub(crate) mod mallctl;
