// SPDX-License-Identifier: MIT
//! # qpak-lib
//! Unoffical Quake PAK archive inspecton and modification library
//!
//! Provides an sync/async library for accessing and manipulating files in the PAK file format.
//!
//! ## Usage
//! Create a [PakManifest] using one of its constructors and then use it to iterator through or write the contents of a [PakFile].
//!
//! IO functions have both sync and async implementations. Sync functions are suffixed as such.

pub mod error;
pub mod pak;

pub use crate::{
    pak::{PakFile, PakManifest},
    error::{Error, Result},
};
