//! Runs all web tests
//! We use a parent test crate like this so that wasm-pack runs all the tests at once.
#![expect(warnings, reason = "tests")]
#![expect(clippy::arithmetic_side_effects, reason = "tests")]
#![expect(clippy::indexing_slicing, reason = "tests")]

#[cfg(feature = "test_utils")]
mod actual_tests;

#[cfg(feature = "test_utils")]
use natrix::test_utils::*;
