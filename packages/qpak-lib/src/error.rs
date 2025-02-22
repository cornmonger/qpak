// SPDX-License-Identifier: MIT

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
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
    FileNameTooLong(String),
    #[error("Non-UTF-8 file name: {0}")]
    NonUtf8FileName(#[from] std::string::FromUtf8Error),
    #[error("No such file in PAK archive: {0}")]
    NoSuchFile(String),
    #[error("Not a directory: {0}")]
    NotDirectory(String),
}

pub type Result<T> = std::result::Result<T, Error>;
