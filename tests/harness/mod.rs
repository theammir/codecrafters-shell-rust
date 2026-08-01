//! Shared test harness. Stage files use only what is re-exported here.
//!
//! Rules of the road: no knowledge of the shell's internal types, no PTY
//! plumbing outside this module, and a hard timeout on every read.

#![allow(dead_code, unused_imports)] // Helpers land here before the stage that needs them.

pub mod sandbox;
pub mod session;

pub use sandbox::Sandbox;
pub use session::{PROMPT, Session};
