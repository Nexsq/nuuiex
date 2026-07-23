use crate::render::style::{Color, Gradient};
use std::fs;

const DEFAULT_THEME: &str = include_str!("themes/default.conf");

pub const BUILTIN_THEMES: &[(&str, &str)] = &[
    ("default", DEFAULT_THEME),
    ("carbon", include_str!("themes/carbon.conf")),
    ("ocean", include_str!("themes/ocean.conf")),
    ("abyss", include_str!("themes/abyss.conf")),
    ("fern", include_str!("themes/fern.conf")),
    ("sunset", include_str!("themes/sunset.conf")),
    ("borealis", include_str!("themes/borealis.conf")),
    ("solstice", include_str!("themes/solstice.conf")),
    ("sanguine", include_str!("themes/sanguine.conf")),
    ("sakura", include_str!("themes/sakura.conf")),
    ("nebula", include_str!("themes/nebula.conf")),
    ("neon", include_str!("themes/neon.conf")),
];

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub title: Vec<Vec<(String, Color)>>,
    pub main_label: Gradient,
    pub warning_color: Gradient,
    pub keyview_color: Gradient,
    pub main_box: Gradient,
    pub list_box: Gradient,
    pub tabs_box: Gradient,
    pub title_box: Gradient,
    pub deck_box: Gradient,
    pub settings_category_box: Gradient,
    pub settings_options_box: Gradient,
    pub selected_box: Gradient,
    pub list_folder: Gradient,
    pub list_file: Gradient,
    pub tab_lazy: Gradient,
    pub tab_selected: Gradient,
    pub settings_entry: Gradient,
    pub settings_selected: Gradient,
    pub settings_special: Gradient,
    pub editor_ins: Gradient,
    pub editor_cmd: Gradient,
    pub editor_vis: Gradient,
    pub editor_fnd: Gradient,
    pub editor_lne: Gradient,
    pub editor_fnd_bg: Gradient,

    pub editor_keywords: Gradient,
    pub editor_functions: Gradient,
    pub editor_strings: Gradient,
    pub editor_numbers: Gradient,
    pub editor_bool: Gradient,
    pub editor_comments: Gradient,
    pub editor_variables: Gradient,
    pub editor_operators: Gradient,
    pub editor_brackets: Gradient,
    pub editor_errors: Gradient,

    pub monitor_cpu_key: Gradient,
    pub monitor_cpu_val: Gradient,
    pub monitor_gpu_key: Gradient,
    pub monitor_gpu_val: Gradient,
    pub monitor_mem_key: Gradient,
    pub monitor_mem_val: Gradient,
    pub monitor_term_key: Gradient,
    pub monitor_term_val: Gradient,
    pub monitor_divider: Gradient,
    pub monitor_bar_bounds: Gradient,

    pub clock_time_color: Gradient,
    pub clock_date_color: Gradient,

    pub macrostats_key: Gradient,
    pub macrostats_val: Gradient,
    pub macrostats_err: Gradient,
}

impl Theme {
    fn new_empty() -> Self {
        Self {
            name: String::new(),
            title: Vec::new(),
            main_label: Gradient::default(),
            warning_color: Gradient::default(),
            keyview_color: Gradient::default(),
            main_box: Gradient::default(),
            list_box: Gradient::default(),
            tabs_box: Gradient::default(),
            title_box: Gradient::default(),
            deck_box: Gradient::default(),
            settings_category_box: Gradient::default(),
            settings_options_box: Gradient::default(),
            selected_box: Gradient::default(),
            list_folder: Gradient::default(),
            list_file: Gradient::default(),
            tab_lazy: Gradient::default(),
            tab_selected: Gradient::default(),
            settings_entry: Gradient::default(),
            settings_selected: Gradient::default(),
            settings_special: Gradient::default(),
            editor_ins: Gradient::default(),
            editor_cmd: Gradient::default(),
            editor_vis: Gradient::default(),
            editor_fnd: Gradient::default(),
            editor_lne: Gradient::default(),
            editor_fnd_bg: Gradient::default(),

            editor_keywords: Gradient::default(),
            editor_functions: Gradient::default(),
            editor_strings: Gradient::default(),
            editor_numbers: Gradient::default(),
            editor_bool: Gradient::default(),
            editor_comments: Gradient::default(),
            editor_variables: Gradient::default(),
            editor_operators: Gradient::default(),
            editor_brackets: Gradient::default(),
            editor_errors: Gradient::default(),

            monitor_cpu_key: Gradient::default(),
            monitor_cpu_val: Gradient::default(),
            monitor_gpu_key: Gradient::default(),
            monitor_gpu_val: Gradient::default(),
            monitor_mem_key: Gradient::default(),
            monitor_mem_val: Gradient::default(),
            monitor_term_key: Gradient::default(),
            monitor_term_val: Gradient::default(),
            monitor_divider: Gradient::default(),
            monitor_bar_bounds: Gradient::default(),

            clock_time_color: Gradient::default(),
            clock_date_color: Gradient::default(),

            macrostats_key: Gradient::default(),
            macrostats_val: Gradient::default(),
            macrostats_err: Gradient::default(),
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
    let config_dir = crate::get_config_dir()?;
    let themes_dir = config_dir.join("themes");

    if !themes_dir.exists() {
        let _ = fs::create_dir_all(&themes_dir);
    }

    for (b_name, content) in BUILTIN_THEMES {
        if *b_name == name {
            let mut theme = parse_theme_base(content, Theme::default())?;
            theme.name = name.to_string();
            return Ok(theme);
        }
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

pub fn parse_color(val: &str) -> Result<Color, String> {
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

pub fn parse_gradient(val: &str) -> Result<Gradient, String> {
    let parts: Vec<&str> = val.split_whitespace().collect();
    if parts.is_empty() {
        return Err("Empty color".into());
    }
    if parts.len() > 4 {
        return Err("Maximum of 4 colors allowed in a gradient".into());
    }
    if parts.len() == 1 {
        Ok(Gradient::Solid(parse_color(parts[0])?))
    } else {
        let mut colors = [Color::None; 4];
        for (i, p) in parts.iter().enumerate() {
            colors[i] = parse_color(p)?;
        }
        Ok(Gradient::Linear(colors, parts.len() as u8))
    }
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
        "main_label" => theme.main_label = parse_gradient(val)?,
        "warning_color" => theme.warning_color = parse_gradient(val)?,
        "keyview_color" => theme.keyview_color = parse_gradient(val)?,
        "main_box" => theme.main_box = parse_gradient(val)?,
        "list_box" => theme.list_box = parse_gradient(val)?,
        "tabs_box" => theme.tabs_box = parse_gradient(val)?,
        "title_box" => theme.title_box = parse_gradient(val)?,
        "deck_box" => theme.deck_box = parse_gradient(val)?,
        "settings_category_box" => theme.settings_category_box = parse_gradient(val)?,
        "settings_options_box" => theme.settings_options_box = parse_gradient(val)?,
        "selected_box" => theme.selected_box = parse_gradient(val)?,
        "list_folder" => theme.list_folder = parse_gradient(val)?,
        "list_file" => theme.list_file = parse_gradient(val)?,
        "tab_lazy" => theme.tab_lazy = parse_gradient(val)?,
        "tab_selected" => theme.tab_selected = parse_gradient(val)?,
        "settings_entry" => theme.settings_entry = parse_gradient(val)?,
        "settings_selected" => theme.settings_selected = parse_gradient(val)?,
        "settings_special" => theme.settings_special = parse_gradient(val)?,
        "editor_ins" => theme.editor_ins = parse_gradient(val)?,
        "editor_cmd" => theme.editor_cmd = parse_gradient(val)?,
        "editor_vis" => theme.editor_vis = parse_gradient(val)?,
        "editor_fnd" => theme.editor_fnd = parse_gradient(val)?,
        "editor_lne" => theme.editor_lne = parse_gradient(val)?,
        "editor_fnd_bg" => theme.editor_fnd_bg = parse_gradient(val)?,

        "editor_keywords" => theme.editor_keywords = parse_gradient(val)?,
        "editor_functions" => theme.editor_functions = parse_gradient(val)?,
        "editor_strings" => theme.editor_strings = parse_gradient(val)?,
        "editor_numbers" => theme.editor_numbers = parse_gradient(val)?,
        "editor_bool" => theme.editor_bool = parse_gradient(val)?,
        "editor_comments" => theme.editor_comments = parse_gradient(val)?,
        "editor_variables" => theme.editor_variables = parse_gradient(val)?,
        "editor_operators" => theme.editor_operators = parse_gradient(val)?,
        "editor_brackets" => theme.editor_brackets = parse_gradient(val)?,
        "editor_errors" => theme.editor_errors = parse_gradient(val)?,

        "monitor_cpu_key" => theme.monitor_cpu_key = parse_gradient(val)?,
        "monitor_cpu_val" => theme.monitor_cpu_val = parse_gradient(val)?,
        "monitor_gpu_key" => theme.monitor_gpu_key = parse_gradient(val)?,
        "monitor_gpu_val" => theme.monitor_gpu_val = parse_gradient(val)?,
        "monitor_mem_key" => theme.monitor_mem_key = parse_gradient(val)?,
        "monitor_mem_val" => theme.monitor_mem_val = parse_gradient(val)?,
        "monitor_term_key" => theme.monitor_term_key = parse_gradient(val)?,
        "monitor_term_val" => theme.monitor_term_val = parse_gradient(val)?,
        "monitor_divider" => theme.monitor_divider = parse_gradient(val)?,
        "monitor_bar_bounds" => theme.monitor_bar_bounds = parse_gradient(val)?,

        "clock_time_color" => theme.clock_time_color = parse_gradient(val)?,
        "clock_date_color" => theme.clock_date_color = parse_gradient(val)?,

        "macrostats_key" => theme.macrostats_key = parse_gradient(val)?,
        "macrostats_val" => theme.macrostats_val = parse_gradient(val)?,
        "macrostats_err" => theme.macrostats_err = parse_gradient(val)?,
        _ => {}
    }
    Ok(())
}
