use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Pack (PackCommand),
    Unpack (UnpackCommand),
    List (ListCommand),
    //Insert (InsertCommand),
}

#[derive(Parser)]
/// Creates a new PAK file from a directory
pub struct PackCommand {
    /// Directory to copy files from
    pub source_dir: PathBuf,
    /// PAK file to create
    pub pak_file: PathBuf,
}

/// Extracts a PAK file into a directory
#[derive(Parser)]
pub struct UnpackCommand {
    /// PAK file to unpack
    pub pak_file: PathBuf,
    /// Directory to unpack to. Default: The current directory
    pub dest_dir: Option<PathBuf>,
}

/// Describes a PAK file
#[derive(Parser)]
pub struct ListCommand {
    /// PAK file to list
    pub pak_file: PathBuf,
}

/// Inserts a file or directory (recursively) into a PAK file
#[derive(Parser)]
pub struct InsertCommand {
    /// Forces overwriting of existing files in PAK
    #[arg(short, long, default_value_t = false)]
    pub force: bool,

    /// File or directory to copy (recursively) into the PAK file
    pub source_path: PathBuf,

    /// PAK file to insert into
    pub pak_file: PathBuf,

    /// Path prefix prepended to all inserted files. Default: The root path of the PAK file
    pub pak_path_prefix: Option<PathBuf>,
}
