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

    pub bind_insert: char,
    pub bind_visual: char,
    pub bind_left: char,
    pub bind_right: char,
    pub bind_up: char,
    pub bind_down: char,
    pub bind_word_next: char,
    pub bind_word_prev: char,
    pub bind_line_start: char,
    pub bind_line_end: char,
    pub bind_select_all: char,
    pub bind_file_bounds: char,
    pub bind_delete: char,
    pub bind_copy: char,
    pub bind_paste: char,
    pub bind_undo: char,
    pub bind_redo: char,
    pub bind_save: char,
}

impl Default for Config {
    fn default() -> Self {
        let mut config = Self {
            test: false,
            border_test: String::new(),
            something: 0,
            primary_color: String::new(),

            bind_insert: '\0',
            bind_visual: '\0',
            bind_left: '\0',
            bind_right: '\0',
            bind_up: '\0',
            bind_down: '\0',
            bind_word_next: '\0',
            bind_word_prev: '\0',
            bind_line_start: '\0',
            bind_line_end: '\0',
            bind_select_all: '\0',
            bind_file_bounds: '\0',
            bind_delete: '\0',
            bind_copy: '\0',
            bind_paste: '\0',
            bind_undo: '\0',
            bind_redo: '\0',
            bind_save: '\0',
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

                    "bind_insert" => {
                        self.bind_insert = val
                            .chars()
                            .next()
                            .unwrap_or(self.bind_insert)
                            .to_ascii_lowercase()
                    }
                    "bind_visual" => {
                        self.bind_visual = val
                            .chars()
                            .next()
                            .unwrap_or(self.bind_visual)
                            .to_ascii_lowercase()
                    }
                    "bind_left" => {
                        self.bind_left = val
                            .chars()
                            .next()
                            .unwrap_or(self.bind_left)
                            .to_ascii_lowercase()
                    }
                    "bind_right" => {
                        self.bind_right = val
                            .chars()
                            .next()
                            .unwrap_or(self.bind_right)
                            .to_ascii_lowercase()
                    }
                    "bind_up" => {
                        self.bind_up = val
                            .chars()
                            .next()
                            .unwrap_or(self.bind_up)
                            .to_ascii_lowercase()
                    }
                    "bind_down" => {
                        self.bind_down = val
                            .chars()
                            .next()
                            .unwrap_or(self.bind_down)
                            .to_ascii_lowercase()
                    }
                    "bind_word_next" => {
                        self.bind_word_next = val
                            .chars()
                            .next()
                            .unwrap_or(self.bind_word_next)
                            .to_ascii_lowercase()
                    }
                    "bind_word_prev" => {
                        self.bind_word_prev = val
                            .chars()
                            .next()
                            .unwrap_or(self.bind_word_prev)
                            .to_ascii_lowercase()
                    }
                    "bind_line_start" => {
                        self.bind_line_start = val
                            .chars()
                            .next()
                            .unwrap_or(self.bind_line_start)
                            .to_ascii_lowercase()
                    }
                    "bind_line_end" => {
                        self.bind_line_end = val
                            .chars()
                            .next()
                            .unwrap_or(self.bind_line_end)
                            .to_ascii_lowercase()
                    }
                    "bind_select_all" => {
                        self.bind_select_all = val
                            .chars()
                            .next()
                            .unwrap_or(self.bind_select_all)
                            .to_ascii_lowercase()
                    }
                    "bind_file_bounds" => {
                        self.bind_file_bounds = val
                            .chars()
                            .next()
                            .unwrap_or(self.bind_file_bounds)
                            .to_ascii_lowercase()
                    }
                    "bind_delete" => {
                        self.bind_delete = val
                            .chars()
                            .next()
                            .unwrap_or(self.bind_delete)
                            .to_ascii_lowercase()
                    }
                    "bind_copy" => {
                        self.bind_copy = val
                            .chars()
                            .next()
                            .unwrap_or(self.bind_copy)
                            .to_ascii_lowercase()
                    }
                    "bind_paste" => {
                        self.bind_paste = val
                            .chars()
                            .next()
                            .unwrap_or(self.bind_paste)
                            .to_ascii_lowercase()
                    }
                    "bind_undo" => {
                        self.bind_undo = val
                            .chars()
                            .next()
                            .unwrap_or(self.bind_undo)
                            .to_ascii_lowercase()
                    }
                    "bind_redo" => {
                        self.bind_redo = val
                            .chars()
                            .next()
                            .unwrap_or(self.bind_redo)
                            .to_ascii_lowercase()
                    }
                    "bind_save" => {
                        self.bind_save = val
                            .chars()
                            .next()
                            .unwrap_or(self.bind_save)
                            .to_ascii_lowercase()
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn reset_general(&mut self) {
        let default = Config::default();
        self.test = default.test;
        self.border_test = default.border_test;
    }

    pub fn reset_keybinds(&mut self) {
        let default = Config::default();
        self.bind_insert = default.bind_insert;
        self.bind_visual = default.bind_visual;
        self.bind_left = default.bind_left;
        self.bind_right = default.bind_right;
        self.bind_up = default.bind_up;
        self.bind_down = default.bind_down;
        self.bind_word_next = default.bind_word_next;
        self.bind_word_prev = default.bind_word_prev;
        self.bind_line_start = default.bind_line_start;
        self.bind_line_end = default.bind_line_end;
        self.bind_select_all = default.bind_select_all;
        self.bind_file_bounds = default.bind_file_bounds;
        self.bind_delete = default.bind_delete;
        self.bind_copy = default.bind_copy;
        self.bind_paste = default.bind_paste;
        self.bind_undo = default.bind_undo;
        self.bind_redo = default.bind_redo;
        self.bind_save = default.bind_save;
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
                    "bind_insert" => {
                        writeln!(&mut output, "{} = {}{}", key_str, self.bind_insert, comment)
                            .unwrap()
                    }
                    "bind_visual" => {
                        writeln!(&mut output, "{} = {}{}", key_str, self.bind_visual, comment)
                            .unwrap()
                    }
                    "bind_left" => {
                        writeln!(&mut output, "{} = {}{}", key_str, self.bind_left, comment)
                            .unwrap()
                    }
                    "bind_right" => {
                        writeln!(&mut output, "{} = {}{}", key_str, self.bind_right, comment)
                            .unwrap()
                    }
                    "bind_up" => {
                        writeln!(&mut output, "{} = {}{}", key_str, self.bind_up, comment).unwrap()
                    }
                    "bind_down" => {
                        writeln!(&mut output, "{} = {}{}", key_str, self.bind_down, comment)
                            .unwrap()
                    }
                    "bind_word_next" => writeln!(
                        &mut output,
                        "{} = {}{}",
                        key_str, self.bind_word_next, comment
                    )
                    .unwrap(),
                    "bind_word_prev" => writeln!(
                        &mut output,
                        "{} = {}{}",
                        key_str, self.bind_word_prev, comment
                    )
                    .unwrap(),
                    "bind_line_start" => writeln!(
                        &mut output,
                        "{} = {}{}",
                        key_str, self.bind_line_start, comment
                    )
                    .unwrap(),
                    "bind_line_end" => writeln!(
                        &mut output,
                        "{} = {}{}",
                        key_str, self.bind_line_end, comment
                    )
                    .unwrap(),
                    "bind_select_all" => writeln!(
                        &mut output,
                        "{} = {}{}",
                        key_str, self.bind_select_all, comment
                    )
                    .unwrap(),
                    "bind_file_bounds" => writeln!(
                        &mut output,
                        "{} = {}{}",
                        key_str, self.bind_file_bounds, comment
                    )
                    .unwrap(),
                    "bind_delete" => {
                        writeln!(&mut output, "{} = {}{}", key_str, self.bind_delete, comment)
                            .unwrap()
                    }
                    "bind_copy" => {
                        writeln!(&mut output, "{} = {}{}", key_str, self.bind_copy, comment)
                            .unwrap()
                    }
                    "bind_paste" => {
                        writeln!(&mut output, "{} = {}{}", key_str, self.bind_paste, comment)
                            .unwrap()
                    }
                    "bind_undo" => {
                        writeln!(&mut output, "{} = {}{}", key_str, self.bind_undo, comment)
                            .unwrap()
                    }
                    "bind_redo" => {
                        writeln!(&mut output, "{} = {}{}", key_str, self.bind_redo, comment)
                            .unwrap()
                    }
                    "bind_save" => {
                        writeln!(&mut output, "{} = {}{}", key_str, self.bind_save, comment)
                            .unwrap()
                    }
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
