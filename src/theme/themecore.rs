use crate::render::style::Color;
use directories::ProjectDirs;
use std::fs;

const DEFAULT_THEME: &str = include_str!("themes/default.conf");

pub const BUILTIN_THEMES: &[(&str, &str)] = &[
    ("default", DEFAULT_THEME),
    ("fern", include_str!("themes/fern.conf")),
    ("ocean", include_str!("themes/ocean.conf")),
];

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub title: Vec<Vec<(String, Color)>>,
    pub main_label: Color,
    pub warning_color: Color,
    pub main_box: Color,
    pub list_box: Color,
    pub tabs_box: Color,
    pub title_box: Color,
    pub deck_box: Color,
    pub settings_category_box: Color,
    pub settings_options_box: Color,
    pub selected_box: Color,
    pub list_folder: Color,
    pub list_file: Color,
    pub tab_lazy: Color,
    pub tab_selected: Color,
    pub settings_entry: Color,
    pub settings_selected: Color,
    pub settings_special: Color,
    pub editor_ins: Color,
    pub editor_cmd: Color,
    pub editor_vis: Color,
    pub editor_src: Color,
    pub editor_src_bg: Color,

    pub editor_keywords: Color,
    pub editor_functions: Color,
    pub editor_strings: Color,
    pub editor_numbers: Color,
    pub editor_bool: Color,
    pub editor_comments: Color,
    pub editor_variables: Color,
    pub editor_operators: Color,
    pub editor_brackets: Color,
    pub editor_errors: Color,
}

impl Theme {
    fn new_empty() -> Self {
        Self {
            name: String::new(),
            title: Vec::new(),
            main_label: Color::None,
            warning_color: Color::None,
            main_box: Color::None,
            list_box: Color::None,
            tabs_box: Color::None,
            title_box: Color::None,
            deck_box: Color::None,
            settings_category_box: Color::None,
            settings_options_box: Color::None,
            selected_box: Color::None,
            list_folder: Color::None,
            list_file: Color::None,
            tab_lazy: Color::None,
            tab_selected: Color::None,
            settings_entry: Color::None,
            settings_selected: Color::None,
            settings_special: Color::None,
            editor_ins: Color::None,
            editor_cmd: Color::None,
            editor_vis: Color::None,
            editor_src: Color::None,
            editor_src_bg: Color::None,

            editor_keywords: Color::None,
            editor_functions: Color::None,
            editor_strings: Color::None,
            editor_numbers: Color::None,
            editor_bool: Color::None,
            editor_comments: Color::None,
            editor_variables: Color::None,
            editor_operators: Color::None,
            editor_brackets: Color::None,
            editor_errors: Color::None,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        let mut t = Theme::new_empty();
        t = parse_theme_base(DEFAULT_THEME, t).unwrap_or_else(|_| Theme::new_empty());
        t.name = "default".to_string();
        t
    }
}

pub fn init(name: &str) -> Result<Theme, String> {
    for (b_name, content) in BUILTIN_THEMES {
        if *b_name == name {
            let mut theme = parse_theme_base(content, Theme::default())?;
            theme.name = name.to_string();
            return Ok(theme);
        }
    }

    let proj_dirs =
        ProjectDirs::from("com", "Nexsq", "nuui").ok_or("Failed to locate config directory.")?;
    let themes_dir = proj_dirs.config_dir().join("themes");

    if !themes_dir.exists() {
        let _ = fs::create_dir_all(&themes_dir);
    }

    let theme_path = themes_dir.join(format!("{}.conf", name));
    if !theme_path.exists() {
        return Err(format!("Theme '{}' does not exist.", name));
    }

    let content =
        fs::read_to_string(&theme_path).map_err(|e| format!("Failed to read theme file: {}", e))?;

    let mut theme = parse_theme_base(&content, Theme::default())?;
    theme.name = name.to_string();

    Ok(theme)
}

fn parse_theme_base(content: &str, mut theme: Theme) -> Result<Theme, String> {
    let mut chars = content.char_indices().peekable();

    while let Some(&(_, c)) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
            continue;
        }
        if c == '#' {
            while let Some(&(_, ch)) = chars.peek() {
                chars.next();
                if ch == '\n' {
                    break;
                }
            }
            continue;
        }

        let mut key = String::new();
        while let Some(&(_, ch)) = chars.peek() {
            if ch == '=' || ch.is_whitespace() {
                break;
            }
            key.push(ch);
            chars.next();
        }

        while let Some(&(_, ch)) = chars.peek() {
            if ch == '=' {
                chars.next();
                break;
            }
            chars.next();
        }

        while let Some(&(_, ch)) = chars.peek() {
            if ch == '\n' {
                break;
            }
            if ch.is_whitespace() {
                chars.next();
                continue;
            }
            break;
        }

        if let Some(&(_, ch)) = chars.peek() {
            if ch == '\'' {
                chars.next();
                let mut val = String::new();
                while let Some(&(_, ch)) = chars.peek() {
                    chars.next();
                    if ch == '\'' {
                        break;
                    }
                    val.push(ch);
                }
                apply_theme_value(&mut theme, &key, &val)?;
            } else {
                let mut val = String::new();
                while let Some(&(_, ch)) = chars.peek() {
                    if ch == '\n' {
                        break;
                    }
                    if ch == '#' {
                        if val.ends_with(' ') || val.ends_with('\t') {
                            break;
                        }
                    }
                    val.push(ch);
                    chars.next();
                }
                apply_theme_value(
                    &mut theme,
                    &key,
                    val.trim().trim_matches('\'').trim_matches('"'),
                )?;
            }
        }
    }

    Ok(theme)
}

fn parse_color(val: &str) -> Result<Color, String> {
    let val = val.trim();

    if val.eq_ignore_ascii_case("none") {
        return Ok(Color::None);
    }
    if val.eq_ignore_ascii_case("black") {
        return Ok(Color::Black);
    }
    if val.eq_ignore_ascii_case("red") {
        return Ok(Color::Red);
    }
    if val.eq_ignore_ascii_case("green") {
        return Ok(Color::Green);
    }
    if val.eq_ignore_ascii_case("yellow") {
        return Ok(Color::Yellow);
    }
    if val.eq_ignore_ascii_case("blue") {
        return Ok(Color::Blue);
    }
    if val.eq_ignore_ascii_case("magenta") {
        return Ok(Color::Magenta);
    }
    if val.eq_ignore_ascii_case("cyan") {
        return Ok(Color::Cyan);
    }
    if val.eq_ignore_ascii_case("white") {
        return Ok(Color::White);
    }
    if val.eq_ignore_ascii_case("darkgray") {
        return Ok(Color::DarkGray);
    }
    if val.eq_ignore_ascii_case("brightred") {
        return Ok(Color::BrightRed);
    }
    if val.eq_ignore_ascii_case("brightgreen") {
        return Ok(Color::BrightGreen);
    }
    if val.eq_ignore_ascii_case("brightyellow") {
        return Ok(Color::BrightYellow);
    }
    if val.eq_ignore_ascii_case("brightblue") {
        return Ok(Color::BrightBlue);
    }
    if val.eq_ignore_ascii_case("brightmagenta") {
        return Ok(Color::BrightMagenta);
    }
    if val.eq_ignore_ascii_case("brightcyan") {
        return Ok(Color::BrightCyan);
    }
    if val.eq_ignore_ascii_case("brightwhite") {
        return Ok(Color::BrightWhite);
    }

    let hex_str = val.trim_start_matches('#');
    if hex_str.len() == 6 {
        if let Ok(rgb) = u32::from_str_radix(hex_str, 16) {
            let r = ((rgb >> 16) & 0xFF) as u8;
            let g = ((rgb >> 8) & 0xFF) as u8;
            let b = (rgb & 0xFF) as u8;
            return Ok(Color::Rgb(r, g, b));
        }
    }
    Err(format!("Invalid color: '{}'", val))
}

fn parse_title(val: &str) -> Result<Vec<Vec<(String, Color)>>, String> {
    let mut lines = Vec::new();
    let mut current_color = Color::White;

    let val = if val.starts_with('\n') {
        &val[1..]
    } else {
        val
    };
    let val = if val.ends_with('\n') {
        &val[..val.len() - 1]
    } else {
        val
    };

    for line in val.split('\n') {
        let mut segments = Vec::new();
        let mut chars = line.chars().peekable();
        let mut text = String::new();

        while let Some(c) = chars.next() {
            if c == '{' {
                if !text.is_empty() {
                    segments.push((text.clone(), current_color));
                    text.clear();
                }
                let mut color_str = String::new();
                while let Some(&nc) = chars.peek() {
                    if nc == '}' {
                        chars.next();
                        break;
                    }
                    color_str.push(chars.next().unwrap());
                }
                current_color = parse_color(&color_str)?;
            } else {
                text.push(c);
            }
        }
        if !text.is_empty() {
            segments.push((text, current_color));
        }
        if segments.is_empty() {
            segments.push((String::new(), current_color));
        }
        lines.push(segments);
    }

    Ok(lines)
}

fn apply_theme_value(theme: &mut Theme, key: &str, val: &str) -> Result<(), String> {
    match key {
        "title" => theme.title = parse_title(val)?,
        "main_label" => theme.main_label = parse_color(val)?,
        "warning_color" => theme.warning_color = parse_color(val)?,
        "main_box" => theme.main_box = parse_color(val)?,
        "list_box" => theme.list_box = parse_color(val)?,
        "tabs_box" => theme.tabs_box = parse_color(val)?,
        "title_box" => theme.title_box = parse_color(val)?,
        "deck_box" => theme.deck_box = parse_color(val)?,
        "settings_category_box" => theme.settings_category_box = parse_color(val)?,
        "settings_options_box" => theme.settings_options_box = parse_color(val)?,
        "selected_box" => theme.selected_box = parse_color(val)?,
        "list_folder" => theme.list_folder = parse_color(val)?,
        "list_file" => theme.list_file = parse_color(val)?,
        "tab_lazy" => theme.tab_lazy = parse_color(val)?,
        "tab_selected" => theme.tab_selected = parse_color(val)?,
        "settings_entry" => theme.settings_entry = parse_color(val)?,
        "settings_selected" => theme.settings_selected = parse_color(val)?,
        "settings_special" => theme.settings_special = parse_color(val)?,
        "editor_ins" => theme.editor_ins = parse_color(val)?,
        "editor_cmd" => theme.editor_cmd = parse_color(val)?,
        "editor_vis" => theme.editor_vis = parse_color(val)?,
        "editor_src" => theme.editor_src = parse_color(val)?,
        "editor_src_bg" => theme.editor_src_bg = parse_color(val)?,

        "editor_keywords" => theme.editor_keywords = parse_color(val)?,
        "editor_functions" => theme.editor_functions = parse_color(val)?,
        "editor_strings" => theme.editor_strings = parse_color(val)?,
        "editor_numbers" => theme.editor_numbers = parse_color(val)?,
        "editor_bool" => theme.editor_bool = parse_color(val)?,
        "editor_comments" => theme.editor_comments = parse_color(val)?,
        "editor_variables" => theme.editor_variables = parse_color(val)?,
        "editor_operators" => theme.editor_operators = parse_color(val)?,
        "editor_brackets" => theme.editor_brackets = parse_color(val)?,
        "editor_errors" => theme.editor_errors = parse_color(val)?,
        _ => {}
    }
    Ok(())
}
