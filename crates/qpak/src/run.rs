use crate::cli::{Cli, Command, PackCommand, UnpackCommand, ListCommand};
use clap::Parser;
use qpak_lib::{Result, PakFile, PakManifest};
use std::path::PathBuf;

pub fn run() {
    let cli = Cli::parse();
    if let Err(e) = run_cli(cli) {
        println!("{}", e);
        std::process::exit(1);
    }
}

pub fn run_cli(cli: Cli) -> Result<()> {
    match &cli.command {
        Command::Pack(cmd) => run_pack(&cli, cmd),
        Command::Unpack(cmd) => run_unpack(&cli, cmd),
        Command::List(cmd) => run_list(&cli, cmd)
    }
}

fn run_list(_cli: &Cli, cmd: &ListCommand) -> Result<()> {
    let pak = PakFile::from_file_sync(&cmd.pak_file)?;
    println!("{:#?}", pak.manifest().header());
    println!("{:#?}", pak.manifest().table());
    Ok(())
}

fn run_pack(_cli: &Cli, cmd: &PackCommand) -> Result<()> {
    let manifest = PakManifest::from_dir_sync(&cmd.source_dir)?;
    let _pak = PakFile::create_from_dir_sync(&cmd.source_dir, manifest, &cmd.pak_file)?;
    println!("packed: {}", cmd.pak_file.to_str().unwrap());
    Ok(())
}

fn run_unpack(_cli: &Cli, cmd: &UnpackCommand) -> Result<()> {
    let pak = PakFile::from_file_sync(&cmd.pak_file)?;

    // default: prefix with the file stem if output directory is not provided (pak0.pak -> pak0/)
    let dest_dir = match &cmd.dest_dir {
        Some(dir) => PathBuf::from(dir),
        None => PathBuf::from(cmd.pak_file.file_stem().unwrap())
    };

    pak.extract_sync(&dest_dir)?;
    println!("unpacked: {}", dest_dir.to_str().unwrap());
    Ok(())
}
