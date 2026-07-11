use directories::ProjectDirs;
use std::fmt::Write as _;
use std::fs;

const DEFAULT_CONFIG: &str = include_str!("template.conf");

#[derive(Debug, Clone)]
pub struct Config {
    pub indicator_style: String,
    pub border_style: String,
    pub theme: String,
    pub lib_sorting: String,
    pub tabs_num: usize,

    pub bind_edit_insert: char,
    pub bind_edit_visual: char,
    pub bind_edit_left: char,
    pub bind_edit_right: char,
    pub bind_edit_up: char,
    pub bind_edit_down: char,
    pub bind_edit_word_next: char,
    pub bind_edit_word_prev: char,
    pub bind_edit_line_start: char,
    pub bind_edit_line_end: char,
    pub bind_edit_select_all: char,
    pub bind_edit_file_bounds: char,
    pub bind_edit_delete: char,
    pub bind_edit_copy: char,
    pub bind_edit_paste: char,
    pub bind_edit_search: char,
    pub bind_edit_undo: char,
    pub bind_edit_redo: char,
    pub bind_edit_save: char,

    pub bind_lib_new_file: char,
    pub bind_lib_new_folder: char,
    pub bind_lib_edit: char,
    pub bind_lib_rename: char,
    pub bind_lib_delete: char,
    pub bind_lib_move_up: char,
    pub bind_lib_move_down: char,
}

impl Default for Config {
    fn default() -> Self {
        let mut config = Self {
            indicator_style: String::new(),
            border_style: String::new(),
            theme: String::new(),
            lib_sorting: String::new(),
            tabs_num: 6,

            bind_edit_insert: '\0',
            bind_edit_visual: '\0',
            bind_edit_left: '\0',
            bind_edit_right: '\0',
            bind_edit_up: '\0',
            bind_edit_down: '\0',
            bind_edit_word_next: '\0',
            bind_edit_word_prev: '\0',
            bind_edit_line_start: '\0',
            bind_edit_line_end: '\0',
            bind_edit_select_all: '\0',
            bind_edit_file_bounds: '\0',
            bind_edit_delete: '\0',
            bind_edit_copy: '\0',
            bind_edit_paste: '\0',
            bind_edit_search: '\0',
            bind_edit_undo: '\0',
            bind_edit_redo: '\0',
            bind_edit_save: '\0',

            bind_lib_new_file: '\0',
            bind_lib_new_folder: '\0',
            bind_lib_edit: '\0',
            bind_lib_rename: '\0',
            bind_lib_delete: '\0',
            bind_lib_move_up: '\0',
            bind_lib_move_down: '\0',
        };
        config.parse_str(DEFAULT_CONFIG).unwrap();
        config
    }
}

impl Config {
    pub fn get_border(&self) -> crate::Border {
        match self.border_style.as_str() {
            "round" => crate::Border::Rounded,
            "heavy" => crate::Border::Heavy,
            _ => crate::Border::Light,
        }
    }

    fn parse_str(&mut self, contents: &str) -> Result<(), String> {
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
                    "indicator_style" => self.indicator_style = val.to_string(),
                    "border_style" => self.border_style = val.to_string(),
                    "theme" => self.theme = val.to_string(),
                    "lib_sorting" => self.lib_sorting = val.to_string(),
                    "tabs_num" => self.tabs_num = val.parse().unwrap_or(self.tabs_num).clamp(2, 6),

                    "bind_edit_insert" => {
                        self.bind_edit_insert = val
                            .chars()
                            .next()
                            .unwrap_or(self.bind_edit_insert)
                            .to_ascii_lowercase()
                    }
                    "bind_edit_visual" => {
                        self.bind_edit_visual = val
                            .chars()
                            .next()
                            .unwrap_or(self.bind_edit_visual)
                            .to_ascii_lowercase()
                    }
                    "bind_edit_left" => {
                        self.bind_edit_left = val
                            .chars()
                            .next()
                            .unwrap_or(self.bind_edit_left)
                            .to_ascii_lowercase()
                    }
                    "bind_edit_right" => {
                        self.bind_edit_right = val
                            .chars()
                            .next()
                            .unwrap_or(self.bind_edit_right)
                            .to_ascii_lowercase()
                    }
                    "bind_edit_up" => {
                        self.bind_edit_up = val
                            .chars()
                            .next()
                            .unwrap_or(self.bind_edit_up)
                            .to_ascii_lowercase()
                    }
                    "bind_edit_down" => {
                        self.bind_edit_down = val
                            .chars()
                            .next()
                            .unwrap_or(self.bind_edit_down)
                            .to_ascii_lowercase()
                    }
                    "bind_edit_word_next" => {
                        self.bind_edit_word_next = val
                            .chars()
                            .next()
                            .unwrap_or(self.bind_edit_word_next)
                            .to_ascii_lowercase()
                    }
                    "bind_edit_word_prev" => {
                        self.bind_edit_word_prev = val
                            .chars()
                            .next()
                            .unwrap_or(self.bind_edit_word_prev)
                            .to_ascii_lowercase()
                    }
                    "bind_edit_line_start" => {
                        self.bind_edit_line_start = val
                            .chars()
                            .next()
                            .unwrap_or(self.bind_edit_line_start)
                            .to_ascii_lowercase()
                    }
                    "bind_edit_line_end" => {
                        self.bind_edit_line_end = val
                            .chars()
                            .next()
                            .unwrap_or(self.bind_edit_line_end)
                            .to_ascii_lowercase()
                    }
                    "bind_edit_select_all" => {
                        self.bind_edit_select_all = val
                            .chars()
                            .next()
                            .unwrap_or(self.bind_edit_select_all)
                            .to_ascii_lowercase()
                    }
                    "bind_edit_file_bounds" => {
                        self.bind_edit_file_bounds = val
                            .chars()
                            .next()
                            .unwrap_or(self.bind_edit_file_bounds)
                            .to_ascii_lowercase()
                    }
                    "bind_edit_delete" => {
                        self.bind_edit_delete = val
                            .chars()
                            .next()
                            .unwrap_or(self.bind_edit_delete)
                            .to_ascii_lowercase()
                    }
                    "bind_edit_copy" => {
                        self.bind_edit_copy = val
                            .chars()
                            .next()
                            .unwrap_or(self.bind_edit_copy)
                            .to_ascii_lowercase()
                    }
                    "bind_edit_paste" => {
                        self.bind_edit_paste = val
                            .chars()
                            .next()
                            .unwrap_or(self.bind_edit_paste)
                            .to_ascii_lowercase()
                    }
                    "bind_edit_search" => {
                        self.bind_edit_search = val
                            .chars()
                            .next()
                            .unwrap_or(self.bind_edit_search)
                            .to_ascii_lowercase()
                    }
                    "bind_edit_undo" => {
                        self.bind_edit_undo = val
                            .chars()
                            .next()
                            .unwrap_or(self.bind_edit_undo)
                            .to_ascii_lowercase()
                    }
                    "bind_edit_redo" => {
                        self.bind_edit_redo = val
                            .chars()
                            .next()
                            .unwrap_or(self.bind_edit_redo)
                            .to_ascii_lowercase()
                    }
                    "bind_edit_save" => {
                        self.bind_edit_save = val
                            .chars()
                            .next()
                            .unwrap_or(self.bind_edit_save)
                            .to_ascii_lowercase()
                    }
                    "bind_lib_new_file" => {
                        self.bind_lib_new_file = val
                            .chars()
                            .next()
                            .unwrap_or(self.bind_lib_new_file)
                            .to_ascii_lowercase()
                    }
                    "bind_lib_new_folder" => {
                        self.bind_lib_new_folder = val
                            .chars()
                            .next()
                            .unwrap_or(self.bind_lib_new_folder)
                            .to_ascii_lowercase()
                    }
                    "bind_lib_edit" => {
                        self.bind_lib_edit = val
                            .chars()
                            .next()
                            .unwrap_or(self.bind_lib_edit)
                            .to_ascii_lowercase()
                    }
                    "bind_lib_rename" => {
                        self.bind_lib_rename = val
                            .chars()
                            .next()
                            .unwrap_or(self.bind_lib_rename)
                            .to_ascii_lowercase()
                    }
                    "bind_lib_delete" => {
                        self.bind_lib_delete = val
                            .chars()
                            .next()
                            .unwrap_or(self.bind_lib_delete)
                            .to_ascii_lowercase()
                    }
                    "bind_lib_move_up" => {
                        self.bind_lib_move_up = val
                            .chars()
                            .next()
                            .unwrap_or(self.bind_lib_move_up)
                            .to_ascii_lowercase()
                    }
                    "bind_lib_move_down" => {
                        self.bind_lib_move_down = val
                            .chars()
                            .next()
                            .unwrap_or(self.bind_lib_move_down)
                            .to_ascii_lowercase()
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    pub fn reset_appearance(&mut self) {
        let default = Config::default();
        self.indicator_style = default.indicator_style;
        self.border_style = default.border_style;
        self.theme = default.theme;
        self.tabs_num = default.tabs_num;
    }

    pub fn reset_edit_keybinds(&mut self) {
        let default = Config::default();
        self.bind_edit_insert = default.bind_edit_insert;
        self.bind_edit_visual = default.bind_edit_visual;
        self.bind_edit_left = default.bind_edit_left;
        self.bind_edit_right = default.bind_edit_right;
        self.bind_edit_up = default.bind_edit_up;
        self.bind_edit_down = default.bind_edit_down;
        self.bind_edit_word_next = default.bind_edit_word_next;
        self.bind_edit_word_prev = default.bind_edit_word_prev;
        self.bind_edit_line_start = default.bind_edit_line_start;
        self.bind_edit_line_end = default.bind_edit_line_end;
        self.bind_edit_select_all = default.bind_edit_select_all;
        self.bind_edit_file_bounds = default.bind_edit_file_bounds;
        self.bind_edit_delete = default.bind_edit_delete;
        self.bind_edit_copy = default.bind_edit_copy;
        self.bind_edit_paste = default.bind_edit_paste;
        self.bind_edit_search = default.bind_edit_search;
        self.bind_edit_undo = default.bind_edit_undo;
        self.bind_edit_redo = default.bind_edit_redo;
        self.bind_edit_save = default.bind_edit_save;
    }

    pub fn reset_lib_keybinds(&mut self) {
        let default = Config::default();
        self.bind_lib_new_file = default.bind_lib_new_file;
        self.bind_lib_new_folder = default.bind_lib_new_folder;
        self.bind_lib_edit = default.bind_lib_edit;
        self.bind_lib_rename = default.bind_lib_rename;
        self.bind_lib_delete = default.bind_lib_delete;
        self.bind_lib_move_up = default.bind_lib_move_up;
        self.bind_lib_move_down = default.bind_lib_move_down;
    }

    pub fn save(&self) {
        let proj_dirs = ProjectDirs::from("com", "Nexsq", "nuui")
            .expect("Failed to locate the system configuration directory.");

        let config_dir = proj_dirs.config_dir().join("conf");
        let config_file = config_dir.join("config.conf");

        let mut output = String::with_capacity(DEFAULT_CONFIG.len() + 256);

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
                    "indicator_style" => writeln!(
                        &mut output,
                        "{} = {}{}",
                        key_str, self.indicator_style, comment
                    )
                    .unwrap(),
                    "border_style" => writeln!(
                        &mut output,
                        "{} = {}{}",
                        key_str, self.border_style, comment
                    )
                    .unwrap(),
                    "theme" => {
                        writeln!(&mut output, "{} = {}{}", key_str, self.theme, comment).unwrap()
                    }
                    "lib_sorting" => {
                        writeln!(&mut output, "{} = {}{}", key_str, self.lib_sorting, comment)
                            .unwrap()
                    }
                    "tabs_num" => {
                        writeln!(&mut output, "{} = {}{}", key_str, self.tabs_num, comment).unwrap()
                    }
                    "bind_edit_insert" => writeln!(
                        &mut output,
                        "{} = {}{}",
                        key_str, self.bind_edit_insert, comment
                    )
                    .unwrap(),
                    "bind_edit_visual" => writeln!(
                        &mut output,
                        "{} = {}{}",
                        key_str, self.bind_edit_visual, comment
                    )
                    .unwrap(),
                    "bind_edit_left" => writeln!(
                        &mut output,
                        "{} = {}{}",
                        key_str, self.bind_edit_left, comment
                    )
                    .unwrap(),
                    "bind_edit_right" => writeln!(
                        &mut output,
                        "{} = {}{}",
                        key_str, self.bind_edit_right, comment
                    )
                    .unwrap(),
                    "bind_edit_up" => writeln!(
                        &mut output,
                        "{} = {}{}",
                        key_str, self.bind_edit_up, comment
                    )
                    .unwrap(),
                    "bind_edit_down" => writeln!(
                        &mut output,
                        "{} = {}{}",
                        key_str, self.bind_edit_down, comment
                    )
                    .unwrap(),
                    "bind_edit_word_next" => writeln!(
                        &mut output,
                        "{} = {}{}",
                        key_str, self.bind_edit_word_next, comment
                    )
                    .unwrap(),
                    "bind_edit_word_prev" => writeln!(
                        &mut output,
                        "{} = {}{}",
                        key_str, self.bind_edit_word_prev, comment
                    )
                    .unwrap(),
                    "bind_edit_line_start" => writeln!(
                        &mut output,
                        "{} = {}{}",
                        key_str, self.bind_edit_line_start, comment
                    )
                    .unwrap(),
                    "bind_edit_line_end" => writeln!(
                        &mut output,
                        "{} = {}{}",
                        key_str, self.bind_edit_line_end, comment
                    )
                    .unwrap(),
                    "bind_edit_select_all" => writeln!(
                        &mut output,
                        "{} = {}{}",
                        key_str, self.bind_edit_select_all, comment
                    )
                    .unwrap(),
                    "bind_edit_file_bounds" => writeln!(
                        &mut output,
                        "{} = {}{}",
                        key_str, self.bind_edit_file_bounds, comment
                    )
                    .unwrap(),
                    "bind_edit_delete" => writeln!(
                        &mut output,
                        "{} = {}{}",
                        key_str, self.bind_edit_delete, comment
                    )
                    .unwrap(),
                    "bind_edit_copy" => writeln!(
                        &mut output,
                        "{} = {}{}",
                        key_str, self.bind_edit_copy, comment
                    )
                    .unwrap(),
                    "bind_edit_paste" => writeln!(
                        &mut output,
                        "{} = {}{}",
                        key_str, self.bind_edit_paste, comment
                    )
                    .unwrap(),
                    "bind_edit_search" => writeln!(
                        &mut output,
                        "{} = {}{}",
                        key_str, self.bind_edit_search, comment
                    )
                    .unwrap(),
                    "bind_edit_undo" => writeln!(
                        &mut output,
                        "{} = {}{}",
                        key_str, self.bind_edit_undo, comment
                    )
                    .unwrap(),
                    "bind_edit_redo" => writeln!(
                        &mut output,
                        "{} = {}{}",
                        key_str, self.bind_edit_redo, comment
                    )
                    .unwrap(),
                    "bind_edit_save" => writeln!(
                        &mut output,
                        "{} = {}{}",
                        key_str, self.bind_edit_save, comment
                    )
                    .unwrap(),

                    "bind_lib_new_file" => writeln!(
                        &mut output,
                        "{} = {}{}",
                        key_str, self.bind_lib_new_file, comment
                    )
                    .unwrap(),
                    "bind_lib_new_folder" => writeln!(
                        &mut output,
                        "{} = {}{}",
                        key_str, self.bind_lib_new_folder, comment
                    )
                    .unwrap(),
                    "bind_lib_edit" => writeln!(
                        &mut output,
                        "{} = {}{}",
                        key_str, self.bind_lib_edit, comment
                    )
                    .unwrap(),
                    "bind_lib_rename" => writeln!(
                        &mut output,
                        "{} = {}{}",
                        key_str, self.bind_lib_rename, comment
                    )
                    .unwrap(),
                    "bind_lib_delete" => writeln!(
                        &mut output,
                        "{} = {}{}",
                        key_str, self.bind_lib_delete, comment
                    )
                    .unwrap(),
                    "bind_lib_move_up" => writeln!(
                        &mut output,
                        "{} = {}{}",
                        key_str, self.bind_lib_move_up, comment
                    )
                    .unwrap(),
                    "bind_lib_move_down" => writeln!(
                        &mut output,
                        "{} = {}{}",
                        key_str, self.bind_lib_move_down, comment
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

pub fn init() -> Result<Config, String> {
    let proj_dirs = ProjectDirs::from("com", "Nexsq", "nuui")
        .ok_or("Failed to locate the system configuration directory.")?;

    let config_dir = proj_dirs.config_dir().join("conf");
    let config_file = config_dir.join("config.conf");

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    if !config_file.exists() {
        fs::write(&config_file, DEFAULT_CONFIG)
            .map_err(|e| format!("Failed to write default config file: {}", e))?;
    }

    let contents = fs::read_to_string(&config_file)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let mut config = Config::default();
    if let Err(e) = config.parse_str(&contents) {
        return Err(format!("Config file is corrupted: {}", e));
    }

    Ok(config)
}

pub fn reset_to_default() -> Result<(), String> {
    let proj_dirs = ProjectDirs::from("com", "Nexsq", "nuui")
        .ok_or("Failed to locate the system configuration directory.")?;

    let config_dir = proj_dirs.config_dir().join("conf");
    let config_file = config_dir.join("config.conf");

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    fs::write(&config_file, DEFAULT_CONFIG)
        .map_err(|e| format!("Failed to write default config file: {}", e))?;

    Ok(())
}
