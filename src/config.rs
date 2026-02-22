#[cfg(debug_assertions)]
pub(crate) fn get_dir_path() -> String {
    ::std::env::current_exe()
        .unwrap()
        .with_file_name("test.sqlite3")
        .to_string_lossy()
        .to_string()
}

#[cfg(not(debug_assertions))]
pub(crate) fn get_dir_path() -> String {
    use directories::BaseDirs;
    use std::fs::{self, File};

    if let Some(base_dirs) = BaseDirs::new() {
        let dir = base_dirs.data_dir();
        let mut dir = dir.to_path_buf();
        dir.push("SeriousOrganizer");
        println!("Got base dir!");
        fs::create_dir_all(&dir).expect(&format!(
            "Failed to create data dir: {}",
            dir.to_string_lossy()
        ));

        dir.push("Database.sqlite3");

        if !dir.exists() {
            File::create(&dir).expect(&format!(
                "Failed to create db_file: {:?}",
                dir.to_string_lossy()
            ));
        }

        dir.to_string_lossy().to_string()
    } else {
        println!("No base dir! :'(");
        ::std::env::current_exe()
            .unwrap()
            .with_file_name("test.sqlite3")
            .to_string_lossy()
            .to_string()
    }
}
