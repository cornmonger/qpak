// SPDX-License-Identifier: MIT
//! # qpak
//! Unofficial Quake PAK file modification tool
//!
//! Provides a multiplatform command-line tool for inspecting and modifying and PAK files.
//!
//! ## Usage
//! `qpak --help`
//!  - [qpak list](#qpak-list)
//!  - [qpak unpack](#qpak-unpack)
//!  - [qpak pack](#qpak-pack)
//!
//! ### qpak list
//! `qpak list <pak_file>`
//!
//! Describes the contents of a PAK file.
//!
//! - **pak_file**: The PAK file to inspect
//!
//! ### qpak unpack
//! `qpak unpack <pak_file> <dest_dir>`
//!
//! Extracts a pak file into a directory.
//!
//! - **pak_file**: The PAK file to extract from
//! - **dest_dir**: The directory to extract into
//!
//! ### qpak pack
//! `qpak pack <src_dir> <pak_file>`
//!
//! Creates a PAK file from a directory.
//!
//! - **src_dir**: The directory to copy files from
//! - **pak_file**: The PAK file to create
//!

pub mod cli;
pub mod run;

pub use crate::{
    run::*,
    cli::*
};
