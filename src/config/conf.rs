use directories::ProjectDirs;
use std::fmt::Write as _;
use std::fs;

const DEFAULT_CONFIG: &str = include_str!("template.conf");

#[derive(Debug, Clone)]
pub struct Config {
    pub test: bool,
    pub border_test: String,
    pub something: i32,
    pub primary_color: String,
}

impl Default for Config {
    fn default() -> Self {
        let mut config = Self {
            test: false,
            border_test: String::new(),
            something: 0,
            primary_color: String::new(),
        };
        config.parse_str(DEFAULT_CONFIG);
        config
    }
}

impl Config {
    fn parse_str(&mut self, contents: &str) {
        for line in contents.lines() {
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, val)) = trimmed.split_once('=') {
                let key = key.trim();
                let mut val = val.trim();

                val = val.split_once(" #").map_or(val, |(v, _)| v).trim();

                match key {
                    "test" => self.test = val.parse().unwrap_or(self.test),
                    "border_test" => self.border_test = val.to_string(),
                    "something" => self.something = val.parse().unwrap_or(self.something),
                    "primary_color" => self.primary_color = val.to_string(),
                    _ => {}
                }
            }
        }
    }

    pub fn save(&self) {
        let proj_dirs = ProjectDirs::from("com", "Nexsq", "nuui")
            .expect("Failed to locate the system configuration directory.");

        let config_dir = proj_dirs.config_dir().join("conf");
        let config_file = config_dir.join("config.conf");

        let mut output = String::with_capacity(DEFAULT_CONFIG.len() + 128);

        for line in DEFAULT_CONFIG.lines() {
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                output.push_str(line);
                output.push('\n');
                continue;
            }

            if let Some((key, comment_part)) = line.split_once('=') {
                let key_str = key.trim();

                let comment = if let Some((_, c)) = comment_part.split_once(" #") {
                    format!(" #{}", c)
                } else {
                    String::new()
                };

                match key_str {
                    "test" => {
                        writeln!(&mut output, "{} = {}{}", key_str, self.test, comment).unwrap()
                    }
                    "border_test" => {
                        writeln!(&mut output, "{} = {}{}", key_str, self.border_test, comment)
                            .unwrap()
                    }
                    "something" => {
                        writeln!(&mut output, "{} = {}{}", key_str, self.something, comment)
                            .unwrap()
                    }
                    "primary_color" => writeln!(
                        &mut output,
                        "{} = {}{}",
                        key_str, self.primary_color, comment
                    )
                    .unwrap(),
                    _ => {
                        output.push_str(line);
                        output.push('\n');
                    }
                }
            } else {
                output.push_str(line);
                output.push('\n');
            }
        }

        if !config_dir.exists() {
            let _ = fs::create_dir_all(&config_dir);
        }

        let _ = fs::write(config_file, output);
    }
}

pub fn init() -> Config {
    let proj_dirs = ProjectDirs::from("com", "Nexsq", "nuui")
        .expect("Failed to locate the system configuration directory.");

    let config_dir = proj_dirs.config_dir().join("conf");
    let config_file = config_dir.join("config.conf");

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).expect("Failed to create config directory");
    }

    if !config_file.exists() {
        fs::write(&config_file, DEFAULT_CONFIG).expect("Failed to write default config file");
    }

    let contents = fs::read_to_string(&config_file).expect("Failed to read config file");

    let mut config = Config::default();
    config.parse_str(&contents);

    config
}
