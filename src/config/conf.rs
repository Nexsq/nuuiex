use crate::render::style::{Border, Color, Modifier, Style};
use directories::ProjectDirs;
use std::collections::HashMap;
use std::{fs, io};

#[derive(Debug, Clone)]
pub struct NuuiConfig {
    pub vars: HashMap<String, ConfigVar>,
    pub boxes: Vec<ConfigBox>,
}

#[derive(Debug, Clone)]
pub enum ConfigVar {
    Int(i32),
    Text(String),
}

#[derive(Debug, Clone)]
pub struct ConfigBox {
    pub name: String,
    pub width: u16,
    pub height: u16,
    pub x: i16,
    pub y: i16,
    pub padding: u16,
    pub border: Border,
    pub style: Style,
}

#[derive(Debug)]
pub enum ConfigError {
    Io(io::Error),
    SystemPathNotFound,
    SyntaxError(String),
    MissingBox(String),
    MissingVar(String),
    TypeError(String),
}

impl From<io::Error> for ConfigError {
    fn from(err: io::Error) -> Self {
        ConfigError::Io(err)
    }
}

const DEFAULT_CONFIG: &str = include_str!("template.conf");

pub fn init(term_w: u16, term_h: u16) -> Result<NuuiConfig, ConfigError> {
    let proj_dirs =
        ProjectDirs::from("com", "Nexsq", "nuui").ok_or(ConfigError::SystemPathNotFound)?;

    let config_dir = proj_dirs.config_dir();
    let config_file = config_dir.join("config.conf");

    if !config_dir.exists() {
        fs::create_dir_all(config_dir)?;
    }

    if !config_file.exists() {
        fs::write(&config_file, DEFAULT_CONFIG)?;
    }

    let content = fs::read_to_string(&config_file)?;

    parse_and_validate(&content, term_w, term_h)
}

fn parse_and_validate(
    content: &str,
    term_w: u16,
    term_h: u16,
) -> Result<NuuiConfig, ConfigError> {
    let mut boxes = Vec::new();
    let mut vars = HashMap::new();

    let mut has_main = false;
    let mut has_tabs = false;
    let mut has_title = false;

    let mut in_block = false;
    let mut current_name = String::new();

    let mut in_multiline = false;
    let mut current_var_name = String::new();
    let mut current_multiline_content = String::new();

    let mut tmp_w: Option<i16> = None;
    let mut tmp_h: Option<i16> = None;
    let mut tmp_x: Option<i16> = None;
    let mut tmp_y: Option<i16> = None;
    let mut tmp_pd: Option<i16> = None;
    let mut tmp_bd: Option<Border> = None;
    let mut tmp_fg: Option<Color> = None;
    let mut tmp_bg: Option<Color> = None;
    let mut tmp_md: Option<Modifier> = None;

    for (line_num, raw_line) in content.lines().enumerate() {
        let line = raw_line.trim();
        let human_line = line_num + 1;

        if in_multiline {
            if line.ends_with('"') {
                let end_idx = raw_line.rfind('"').unwrap();
                current_multiline_content.push_str(&raw_line[..end_idx]);

                let final_text = current_multiline_content
                    .trim_matches(|c| c == '\n' || c == '\r')
                    .replace("\\n", "\n")
                    .to_string();

                vars.insert(current_var_name.clone(), ConfigVar::Text(final_text));

                in_multiline = false;
                current_multiline_content.clear();
            } else {
                current_multiline_content.push_str(raw_line);
                current_multiline_content.push('\n');
            }
            continue;
        }

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.ends_with('[') {
            if in_block { return Err(ConfigError::SyntaxError(format!("Line {}: Nested blocks are not allowed.", human_line))); }
            in_block = true;
            current_name = line.trim_end_matches('[').trim().to_string();

            if current_name == "main" { has_main = true; }
            if current_name == "tabs" { has_tabs = true; }
            if current_name == "title" { has_title = true; }

            tmp_w = None; tmp_h = None; tmp_x = None; tmp_y = None;
            tmp_pd = None; tmp_bd = None; tmp_fg = None; tmp_bg = None; tmp_md = None;

        } else if line == "]" {
            if !in_block { return Err(ConfigError::SyntaxError(format!("Line {}: Found ']' without an opening '['.", human_line))); }

            let w = tmp_w.ok_or_else(|| ConfigError::SyntaxError(format!("Block '{}' missing 'w' property.", current_name)))?;
            let h = tmp_h.ok_or_else(|| ConfigError::SyntaxError(format!("Block '{}' missing 'h' property.", current_name)))?;
            let x = tmp_x.ok_or_else(|| ConfigError::SyntaxError(format!("Block '{}' missing 'x' property.", current_name)))?;
            let y = tmp_y.ok_or_else(|| ConfigError::SyntaxError(format!("Block '{}' missing 'y' property.", current_name)))?;
            let pd = tmp_pd.ok_or_else(|| ConfigError::SyntaxError(format!("Block '{}' missing 'pd' property.", current_name)))?;
            let bd = tmp_bd.ok_or_else(|| ConfigError::SyntaxError(format!("Block '{}' missing 'bd' property.", current_name)))?;
            let fg = tmp_fg.ok_or_else(|| ConfigError::SyntaxError(format!("Block '{}' missing 'fg' property.", current_name)))?;
            let bg = tmp_bg.ok_or_else(|| ConfigError::SyntaxError(format!("Block '{}' missing 'bg' property.", current_name)))?;
            let md = tmp_md.ok_or_else(|| ConfigError::SyntaxError(format!("Block '{}' missing 'md' property.", current_name)))?;

            if w < 0 || h < 0 { return Err(ConfigError::SyntaxError(format!("Block '{}' width and height cannot evaluate to a negative number.", current_name))); }
            if pd < 0 { return Err(ConfigError::SyntaxError(format!("Block '{}' padding cannot be negative.", current_name))); }

            boxes.push(ConfigBox {
                name: current_name.clone(),
                width: w as u16,
                height: h as u16,
                x, y,
                padding: pd as u16,
                border: bd,
                style: Style { fg, bg, md },
            });

            in_block = false;

        } else {
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() != 2 { return Err(ConfigError::SyntaxError(format!("Line {}: Invalid syntax. Expected 'key: value'.", human_line))); }

            let key = parts[0].trim();
            let val = parts[1].trim();

            if !in_block {
                if val.starts_with('"') {
                    if val.ends_with('"') && val.len() > 1 {
                        let start_idx = raw_line.find('"').unwrap() + 1;
                        let end_idx = raw_line.rfind('"').unwrap();
                        let s = raw_line[start_idx..end_idx].replace("\\n", "\n").to_string();
                        vars.insert(key.to_string(), ConfigVar::Text(s));
                    } else {
                        in_multiline = true;
                        current_var_name = key.to_string();
                        let start_idx = raw_line.find('"').unwrap() + 1;
                        current_multiline_content.push_str(&raw_line[start_idx..]); 
                        current_multiline_content.push('\n');
                    }
                } else if let Ok(num) = val.parse::<i32>() {
                    vars.insert(key.to_string(), ConfigVar::Int(num));
                } else {
                    return Err(ConfigError::SyntaxError(format!("Line {}: Invalid global variable. Strings must be wrapped in quotes.", human_line)));
                }
            } else {
                match key {
                    "w" => tmp_w = Some(eval_expr(val, term_w, term_h).map_err(|e| ConfigError::SyntaxError(format!("Line {}: {}", human_line, e)))?),
                    "h" => tmp_h = Some(eval_expr(val, term_w, term_h).map_err(|e| ConfigError::SyntaxError(format!("Line {}: {}", human_line, e)))?),
                    "x" => tmp_x = Some(eval_expr(val, term_w, term_h).map_err(|e| ConfigError::SyntaxError(format!("Line {}: {}", human_line, e)))?),
                    "y" => tmp_y = Some(eval_expr(val, term_w, term_h).map_err(|e| ConfigError::SyntaxError(format!("Line {}: {}", human_line, e)))?),
                    "pd" => tmp_pd = Some(val.parse::<i16>().map_err(|_| ConfigError::SyntaxError(format!("Line {}: Invalid integer value for padding.", human_line)))?),
                    "bd" => tmp_bd = Some(parse_border(val).ok_or_else(|| ConfigError::SyntaxError(format!("Line {}: Invalid Border style.", human_line)))?),
                    "fg" => tmp_fg = Some(parse_color(val).ok_or_else(|| ConfigError::SyntaxError(format!("Line {}: Invalid Color.", human_line)))?),
                    "bg" => tmp_bg = Some(parse_color(val).ok_or_else(|| ConfigError::SyntaxError(format!("Line {}: Invalid Color.", human_line)))?),
                    "md" => tmp_md = Some(parse_modifier(val).ok_or_else(|| ConfigError::SyntaxError(format!("Line {}: Invalid Modifier.", human_line)))?),
                    _ => return Err(ConfigError::SyntaxError(format!("Line {}: Unknown property '{}'.", human_line, key))),
                }
            }
        }
    }

    if in_multiline { return Err(ConfigError::SyntaxError("Unexpected end of file. Missing closing '\"' for multi-line string.".to_string())); }
    if in_block { return Err(ConfigError::SyntaxError("Unexpected end of file. Missing closing ']' for block.".to_string())); }

    if !has_main { return Err(ConfigError::MissingBox(String::from("main"))); }
    if !has_tabs { return Err(ConfigError::MissingBox(String::from("tabs"))); }
    if !has_title { return Err(ConfigError::MissingBox(String::from("title"))); }

    if !vars.contains_key("min_w") { return Err(ConfigError::MissingVar("min_w".to_string())); }
    if !vars.contains_key("min_h") { return Err(ConfigError::MissingVar("min_h".to_string())); }
    if !vars.contains_key("title_s") { return Err(ConfigError::MissingVar("title_s".to_string())); }

    if let Some(ConfigVar::Text(_)) = vars.get("min_w") { return Err(ConfigError::TypeError("min_w must be an integer".to_string())); }
    if let Some(ConfigVar::Text(_)) = vars.get("min_h") { return Err(ConfigError::TypeError("min_h must be an integer".to_string())); }
    if let Some(ConfigVar::Int(_)) = vars.get("title_s") { return Err(ConfigError::TypeError("title_s must be text".to_string())); }

    Ok(NuuiConfig { vars, boxes })
}

fn eval_expr(expr: &str, term_w: u16, term_h: u16) -> Result<i16, String> {
    let clean_expr = expr
        .replace("width", &term_w.to_string())
        .replace("height", &term_h.to_string())
        .replace(' ', "");

    if let Some((a, b)) = clean_expr.split_once('+') {
        let val_a: i16 = a
            .parse()
            .map_err(|_| "Invalid math expression".to_string())?;
        let val_b: i16 = b
            .parse()
            .map_err(|_| "Invalid math expression".to_string())?;
        Ok(val_a + val_b)
    } else if let Some((a, b)) = clean_expr.split_once('-') {
        let val_a: i16 = a
            .parse()
            .map_err(|_| "Invalid math expression".to_string())?;
        let val_b: i16 = b
            .parse()
            .map_err(|_| "Invalid math expression".to_string())?;
        Ok(val_a - val_b)
    } else {
        clean_expr
            .parse()
            .map_err(|_| "Cannot parse numerical value".to_string())
    }
}

fn parse_color(s: &str) -> Option<Color> {
    let trimmed = s.trim();
    let lower = trimmed.to_lowercase();

    if lower.starts_with("rgb(") && lower.ends_with(')') {
        let inner = &trimmed[4..trimmed.len() - 1];
        let parts: Vec<&str> = inner.split(',').collect();

        if parts.len() == 3 {
            let r = parts[0].trim().parse::<u8>().ok()?;
            let g = parts[1].trim().parse::<u8>().ok()?;
            let b = parts[2].trim().parse::<u8>().ok()?;
            return Some(Color::Rgb(r, g, b));
        }
        return None;
    }

    match lower.as_str() {
        "none" => Some(Color::None),
        "black" => Some(Color::Black),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "yellow" => Some(Color::Yellow),
        "blue" => Some(Color::Blue),
        "magenta" => Some(Color::Magenta),
        "cyan" => Some(Color::Cyan),
        "white" => Some(Color::White),
        "darkgray" => Some(Color::DarkGray),
        "brightred" => Some(Color::BrightRed),
        "brightgreen" => Some(Color::BrightGreen),
        "brightyellow" => Some(Color::BrightYellow),
        "brightblue" => Some(Color::BrightBlue),
        "brightmagenta" => Some(Color::BrightMagenta),
        "brightcyan" => Some(Color::BrightCyan),
        "brightwhite" => Some(Color::BrightWhite),
        _ => None,
    }
}

fn parse_modifier(s: &str) -> Option<Modifier> {
    match s.trim().to_lowercase().as_str() {
        "none" => Some(Modifier::None),
        "bold" => Some(Modifier::Bold),
        "dim" => Some(Modifier::Dim),
        "italic" => Some(Modifier::Italic),
        "underline" => Some(Modifier::Underline),
        "reverse" => Some(Modifier::Reverse),
        "hidden" => Some(Modifier::Hidden),
        "strikethrough" => Some(Modifier::Strikethrough),
        _ => None,
    }
}

fn parse_border(s: &str) -> Option<Border> {
    match s.trim().to_lowercase().as_str() {
        "none" => Some(Border::None),
        "light" => Some(Border::Light),
        "heavy" => Some(Border::Heavy),
        "double" => Some(Border::Double),
        "rounded" => Some(Border::Rounded),
        _ => None,
    }
}
