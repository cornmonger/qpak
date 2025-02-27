// SPDX-License-Identifier: MIT

/// Pak library errors
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("I/O error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Invalid magic number: {0:?}")]
    InvalidMagicNumber([u8; 4]),
    #[error("Invalid file table offset: {0}")]
    InvalidTableOffset(i32),
    #[error("Invalid file table size: {0}")]
    InvalidTableSize(i32),
    #[error("Invalid file offset: {0}")]
    InvalidFileOffset(i32),
    #[error("Invalid file size: {0}")]
    InvalidFileSize(i32),
    #[error("File name too long: {0}")]
    FilenameTooLong(String),
    #[error("Non-UTF-8 file name: {0}")]
    NonUtf8Filename(#[from] std::string::FromUtf8Error),
    #[error("Non-UTF-8 file name: {0}")]
    NonUtf8Path(std::path::PathBuf),
     #[error("No such file in PAK archive: {0}")]
    NoSuchFile(String),
    #[error("Not a directory: {0}")]
    NotDirectory(String),
    #[error("Pak path already exists: {0}")]
    PakPathExists(String),
    #[error("Failed to create directory: {0}")]
    CreateDirectory(std::io::Error),
    #[error("Failed to read directory: {0} :: {1}")]
    ReadDirectory(std::path::PathBuf, String),
    #[error("Failed to open PAK file: {0}")]
    OpenPak(std::io::Error),
    #[error("Failed to write to PAK file: {0}")]
    WritePak(std::io::Error),
    #[error("Failed to read from PAK file: {0} :: {1}")]
    ReadPak(std::path::PathBuf, String),
}

/// Pak library result with Pak [Error]
pub type Result<T> = std::result::Result<T, Error>;
