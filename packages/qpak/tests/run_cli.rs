#[cfg(test)]
mod tests {
    const FIXTURE_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../testing/fixtures");
    const TARGET_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../target");
    use std::path::{Path, PathBuf};
    use qpak;

    // Optional test a PAK file provided by environment variable.
    #[test]
    fn test_installed_pak() {
        let env_test_pak = match std::option_env!("QPAK_TEST_PAK") {
            Some(path) => path,
            None => {
                println!("QPAK_TEST_PAK environment variable not set. Skipping optional test");
                return;
            }
        };

        do_test_pak(env_test_pak, "test_installed_pak");
    }

    // Test a PAK file provided by environment variable.
    #[test]
    fn test_fixture_pak() {
        let fixture_pak_file = PathBuf::from(FIXTURE_PATH).join("pak9.pak");
        do_test_pak(fixture_pak_file, "test_fixture_pak");
    }

    fn do_test_pak<P: AsRef<Path>>(pak_file: P, test_name: &str) {
        let test_pak_path = PathBuf::from(pak_file.as_ref());
        let dest_dir = PathBuf::from(TARGET_PATH)
            .join("tests/qpak").join(test_name).join("test_pak");
        let new_test_pak_path = PathBuf::from(TARGET_PATH)
            .join("tests/qpak").join(test_name).join("new_test_pak.pak");
        let new_dest_dir = PathBuf::from(TARGET_PATH)
            .join("tests/qpak").join(test_name).join("new_test_pak");


        // list it
        qpak::run_cli(qpak::Cli {
            command: qpak::Command::List(qpak::ListCommand {
                pak_file: test_pak_path.clone(),
            }),
        }).unwrap();

        if dest_dir.exists() {
            std::fs::remove_dir_all(&dest_dir).unwrap();
        }
        if new_dest_dir.exists() {
            std::fs::remove_dir_all(&new_dest_dir).unwrap();
        }

        std::fs::create_dir_all(&dest_dir).unwrap();

        // unpack it
        qpak::run_cli(qpak::Cli {
            command: qpak::Command::Unpack(qpak::UnpackCommand {
                pak_file: test_pak_path.clone(),
                dest_dir: Some(dest_dir.clone()),
            }),
        }).unwrap();

        // pack it
        qpak::run_cli(qpak::Cli {
            command: qpak::Command::Pack(qpak::PackCommand {
                source_dir: dest_dir.clone(),
                pak_file: new_test_pak_path.clone(),
            }),
        }).unwrap();

        // unpack the packed file
        qpak::run_cli(qpak::Cli {
            command: qpak::Command::Unpack(qpak::UnpackCommand {
                pak_file: new_test_pak_path.clone(),
                dest_dir: Some(new_dest_dir.clone()),
            }),
        }).unwrap();

        // diff the two directories
        assert!(!dir_diff::is_different(&dest_dir, &new_dest_dir).unwrap());
    }
}
