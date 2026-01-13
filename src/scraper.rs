use std::env;
use std::error::Error;
use std::fs;

const DIRS: &[&str] = &["/var/lib/flatpak/exports/share/applications"];

pub fn get_flatpaks() -> Result<(), Box<dyn Error>> {
    let data_dirs = env::var("XDG_DATA_DIRS").unwrap();
    let mut clean_dirs: Vec<&str> = data_dirs.split(":").collect();
    clean_dirs.extend_from_slice(DIRS);
    clean_dirs.retain(|&s| !s.starts_with("/nix/store/"));
    println!("{:?}", clean_dirs);

    for dir in clean_dirs {
        println!("Current dir: {:?}", dir);
        for file in fs::read_dir(dir)? {
            let entry = file?;
            let entry_path = entry.path();
            println!("{:?}", entry_path);
        }
    }

    Ok(())
}
