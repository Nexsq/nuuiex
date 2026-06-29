use directories::ProjectDirs;
use std::fs;

const DEFAULT_CONFIG: &str = include_str!("template.conf");

pub fn init() {
    let proj_dirs = ProjectDirs::from("com", "Nexsq", "nuui")
        .expect("Failed to locate the system configuration directory.");

    let config_dir = proj_dirs.config_dir();
    let config_file = config_dir.join("config.conf");

    if !config_dir.exists() {
        if let Err(e) = fs::create_dir_all(config_dir) {
            panic!(
                "Failed to create config directory at {:?}\nDetails: {}",
                config_dir, e
            );
        }
    }

    if !config_file.exists() {
        if let Err(e) = fs::write(&config_file, DEFAULT_CONFIG) {
            panic!(
                "Failed to write config file at {:?}\nDetails: {}",
                config_file, e
            );
        }
    }
}
