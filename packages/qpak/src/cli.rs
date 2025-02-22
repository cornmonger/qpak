use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Pack (PackCommand),
    Unpack (UnpackCommand),
    List (ListCommand),
}

#[derive(Parser)]
pub struct PackCommand {
    #[arg()]
    pub input_dir: PathBuf,
    #[arg()]
    pub pak_file: PathBuf,
}

#[derive(Parser)]
pub struct UnpackCommand {
    #[arg()]
    pub pak_file: PathBuf,
    #[arg()]
    pub output_dir: Option<PathBuf>,
}

#[derive(Parser)]
pub struct ListCommand {
    #[arg()]
    pub pak_file: PathBuf,
}
