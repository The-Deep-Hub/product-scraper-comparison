//! Integration tests for the rust_scraper application.
//! This module contains all the test suites for testing the application's functionality.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod api {
    pub mod middleware {
        pub mod auth;
    }
    pub mod routes {
        pub mod auth;
    }
} 