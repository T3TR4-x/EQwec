//! EQwec Hub developer-preview launcher.
//!
//! The library owns the window-independent launcher logic so it can be tested
//! without opening a window. `main` adds the eframe entry point and the
//! `--self-check` mode.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod actions;
pub mod app;
pub mod error;
pub mod launch;
pub mod paths;
pub mod state;
pub mod trust;
pub mod worker;
