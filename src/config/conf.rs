use directories::ProjectDirs;
use std::fs;

const DEFAULT_CONFIG: &str = include_str!("template.conf");

#[derive(Debug, Clone)]
pub struct Config {
    pub test: bool,
    pub border_test: String,
    pub something: i32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            test: false,
            border_test: String::from("light"),
            something: 0,
        }
    }
}

pub fn init() -> Config {
    let proj_dirs = ProjectDirs::from("com", "Nexsq", "nuui")
        .expect("Failed to locate the system configuration directory.");

    let config_dir = proj_dirs.config_dir().join("conf");
    let config_file = config_dir.join("config.conf");

    if !config_dir.exists() {
        if let Err(e) = fs::create_dir_all(&config_dir) {
            panic!(
                "Failed to create config directory at {:?}\nDetails: {}",
                config_dir, e
            );
        }
    }

    if !config_file.exists() {
        if let Err(e) = fs::write(&config_file, DEFAULT_CONFIG) {
            panic!(
                "Failed to write default config file at {:?}\nDetails: {}",
                config_file, e
            );
        }
    }

    let contents = match fs::read_to_string(&config_file) {
        Ok(c) => c,
        Err(e) => {
            panic!(
                "Failed to read config file at {:?}\nDetails: {}",
                config_file, e
            );
        }
    };

    parse_config(&contents)
}

fn parse_config(contents: &str) -> Config {
    let mut config = Config::default();

    for line in contents.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if let Some((key, val)) = trimmed.split_once('=') {
            let key = key.trim();
            let mut val = val.trim();

            val = val.split_once('#').map_or(val, |(v, _)| v).trim();

            match key {
                "test" => config.test = val.parse().unwrap_or(config.test),
                "border_test" => config.border_test = val.to_string(),
                "something" => config.something = val.parse().unwrap_or(config.something),
                _ => {}
            }
        }
    }

    config
}
