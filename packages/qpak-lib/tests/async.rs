#[cfg(test)]
mod tests {
    const FIXTURE_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../testing/fixtures");
    use std::path::{Path, PathBuf};
    use qpak_lib as qpak;

    // Optional test a PAK file provided by environment variable.
    #[tokio::test]
    async fn test_installed_pak() {
        let env_test_pak = match std::option_env!("QPAK_TEST_PAK") {
            Some(path) => path,
            None => {
                println!("QPAK_TEST_PAK environment variable not set. Skipping optional test");
                return;
            }
        };

        do_test_pak(&env_test_pak).await;
    }

    #[tokio::test]
    async fn test_fixture_pak() {
        let fixture_pak_file = PathBuf::from(FIXTURE_PATH).join("pak9.pak");
        do_test_pak(fixture_pak_file).await;
    }


    async fn do_test_pak<P: AsRef<Path>>(pak_file: P) {
        let test_pak_path = PathBuf::from(pak_file.as_ref());
        let dest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target/tests/qpak-lib/test_installed_pak/test_pak");
        let new_test_pak_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target/tests/qpak-lib/test_installed_pak/new_test_pak.pak");
        let new_dest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target/tests/qpak-lib/test_installed_pak/new_test_pak");

        if dest_dir.exists() {
            std::fs::remove_dir_all(&dest_dir).unwrap();
        }

        std::fs::create_dir_all(&dest_dir).unwrap();

        // list it
        let pak = qpak::PakFile::from_file(&test_pak_path).await.unwrap();
        println!("{:#?}", pak.manifest().header());
        println!("{:#?}", pak.manifest().table());

        // unpack it
        qpak_lib::PakFile::from_file(&test_pak_path).await.unwrap();
        pak.extract_sync(&dest_dir).unwrap();

        // repack it
        let manifest = qpak_lib::PakManifest::from_dir(&dest_dir).await.unwrap();
        qpak_lib::PakFile::create_from_dir(&dest_dir, manifest, &new_test_pak_path).await.unwrap();

        // unpack the repack
        let pak = qpak_lib::PakFile::from_file(&new_test_pak_path).await.unwrap();
        pak.extract_sync(&new_dest_dir).unwrap();

        // diff the two directories
        assert!(!dir_diff::is_different(&dest_dir, &new_dest_dir).unwrap());
    }
}
