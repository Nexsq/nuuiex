use directories::ProjectDirs;
use std::fs;
use std::io;

#[derive(Debug)]
pub enum ConfigError {
    Io(io::Error),
    SystemPathNotFound,
    SyntaxError(String),
    MissingBox(String),
}

impl From<io::Error> for ConfigError {
    fn from(err: io::Error) -> Self {
        ConfigError::Io(err)
    }
}

const DEFAULT_CONFIG: &str = include_str!("template.conf");

pub fn init() -> Result<String, ConfigError> {
    let proj_dirs = ProjectDirs::from("com", "Nexsq", "nuui")
        .ok_or(ConfigError::SystemPathNotFound)?;

    let config_dir = proj_dirs.config_dir();
    let config_file = config_dir.join("config.conf");

    if !config_dir.exists() {
        fs::create_dir_all(config_dir)?;
    }

    if !config_file.exists() {
        fs::write(&config_file, DEFAULT_CONFIG)?;
    }

    let content = fs::read_to_string(&config_file)?;

    validate_syntax(&content)?;

    Ok(content) // later change content to a struct
}

fn validate_syntax(content: &str) -> Result<(), ConfigError> {
    let mut has_main = false;
    let mut has_tabs = false;
    let mut in_block = false;

    for (line_num, raw_line) in content.lines().enumerate() {
        let line = raw_line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.ends_with('[') {
            if in_block {
                return Err(ConfigError::SyntaxError(format!(
                    "Line {}: Nested blocks are not allowed.", line_num + 1
                )));
            }
            in_block = true;

            if line.starts_with("main") {
                has_main = true;
            }
            if line.starts_with("tabs") {
                has_tabs = true;
            }
        } else if line == "]" {
            if !in_block {
                return Err(ConfigError::SyntaxError(format!(
                    "Line {}: Found ']' without an opening '['.", line_num + 1
                )));
            }
            in_block = false;
        } else {
            if in_block && !line.contains(':') {
                return Err(ConfigError::SyntaxError(format!(
                    "Line {}: Invalid property syntax. Expected 'key: value'.", line_num + 1
                )));
            }
        }
    }

    if in_block {
        return Err(ConfigError::SyntaxError(
            "Unexpected end of file. Missing closing ']'.".to_string(),
        ));
    }

    if !has_main {
        return Err(ConfigError::MissingBox(String::from("main")));
    }

    if !has_tabs {
        return Err(ConfigError::MissingBox(String::from("tabs")));
    }

    Ok(())
}