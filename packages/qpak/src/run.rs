use crate::cli::{Cli, Command, PackCommand, UnpackCommand, ListCommand, InsertCommand};
use clap::Parser;
use qpak_lib::{Result, PakFile, PakManifest};
use std::{fs::{self, File}, io::{BufWriter, Write}, path::PathBuf, process::exit};

pub fn run() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Command::Pack(cmd) => run_pack(&cli, cmd),
        Command::Unpack(cmd) => run_unpack(&cli, cmd),
        Command::List(cmd) => run_list(&cli, cmd),
        //Command::Insert(cmd) => run_insert(&cli, cmd),
    };

    if let Err(e) = result {
        println!("{}", e);
        exit(1);
    }
}

fn _run_insert(_cli: &Cli, cmd: &InsertCommand) -> Result<()> {
    let pak = match PakFile::from_file_sync(&cmd.pak_file) {
        Ok(p) => p,
        Err(why) => {
            println!("Couldn't open {:#?}: {}", &cmd.pak_file, why);
            exit(1);
        }
    };

    if !&cmd.force && pak.manifest().table().contains(&cmd.source_path) {
        println!("Path already exists in pak. Use --force to overwrite.");
        exit(1);
    }

    todo!("not implemented")
}

fn run_list(_cli: &Cli, cmd: &ListCommand) -> Result<()> {
    let pak = match PakFile::from_file_sync(&cmd.pak_file) {
        Ok(p) => p,
        Err(why) => {
            println!("Couldn't open {:#?}: {}", &cmd.pak_file, why);
            exit(1);
        }
    };

    println!("{:#?}", pak.manifest().header());
    println!("{:#?}", pak.manifest().table());

    Ok(())
}

fn run_pack(_cli: &Cli, cmd: &PackCommand) -> Result<()> {
    let manifest = PakManifest::from_dir_sync(&cmd.source_dir)?;
    let _pak = PakFile::write_from_dir_sync(&cmd.source_dir, manifest, &cmd.pak_file)?;
    println!("Created pak file");
    Ok(())
}

fn run_unpack(_cli: &Cli, cmd: &UnpackCommand) -> Result<()> {
    let pak = match PakFile::from_file_sync(&cmd.pak_file) {
        Ok(p) => p,
        Err(why) => {
            println!("Couldn't open {:#?}: {}", &cmd.pak_file, why);
            exit(1);
        }
    };

    for pak_item in pak.read_items_sync()? {
        let pak_item = pak_item?;
        let mut path = PathBuf::new();

        // default: prefix with the file stem if output directory is not provided (pak0.pak -> pak0/)
        match &cmd.dest_dir {
            Some(dir) => path.push(dir),
            None => path.push(cmd.pak_file.file_stem().unwrap())
        }

        path.push(&pak_item.table_entry.path);

        if let Some(p) = path.parent() {
            if !p.exists() {
                if let Err(why) = fs::create_dir_all(p) {
                    println!("Couldn't create parent directories: {}", why);
                    exit(1);
                }
            }
        }

        let file = match File::create(&path) {
            Ok(f) => f,
            Err(why) => {
                println!("Couldn't open {}: {}", path.to_str().unwrap(), why);
                exit(1);
            }
        };

        let mut writer = BufWriter::new(file);
        match writer.write_all(pak_item.data.as_ref()) {
            Ok(_) => (),
            Err(why) => {
                println!("Couldn't write to {}: {}", path.to_str().unwrap(), why);
                exit(1);
            }
        }
    }

    Ok(())
}
