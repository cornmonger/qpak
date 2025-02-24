// SPDX-License-Identifier: MIT
//! # qpak
//! *An unofficial Quake PAK file manipulation tool*
//!
//! ## Usage:
//!   qpak [options] <command> ...
//!
//! ### Commands:
//! - pack: Create a new PAK file
//! - unpack: Extract files from a PAK file
//! - list: Modify files in a PAK file
//!
//! ### Options:
//! - -h, --help      Print help information

pub mod cli;
pub mod run;

pub use run::run;
