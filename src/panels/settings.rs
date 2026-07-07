use std::time::Duration;

use crate::conf::Config;
use crate::{Border, Box, Canvas, Color, Key, Modifier, Style, Terminal};

#[derive(Debug, Clone, Copy, PartialEq)]
enum ActiveSettingsPanel {
    Categories,
    Details,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CustomType {
    Text,
    Number,
    Color,
}

#[derive(Debug, Clone)]
pub enum SettingType {
    Choice(Vec<String>, usize),
    Custom {
        value: String,
        default: String,
        validation: CustomType,
    },
}

#[derive(Debug, Clone)]
pub struct Setting {
    pub name: &'static str,
    pub key: &'static str,
    pub kind: SettingType,
}

#[derive(Debug, Clone)]
pub struct Category {
    pub name: &'static str,
    pub settings: Vec<Setting>,
}

pub fn settings_modal<F>(
    terminal: &Terminal,
    canvas: &mut Canvas,
    config: &mut Config,
    min_w: u16,
    min_h: u16,
    mut draw_background: F,
) -> bool
where
    F: FnMut(&mut Canvas, &Config, u16, u16),
{
    let mut active_panel = ActiveSettingsPanel::Categories;

    let mut edit_mode = false;
    let mut edit_buffer = String::new();

    let mut categories = vec![
        Category {
            name: "General",
            settings: vec![
                Setting {
                    name: "Test Mode",
                    key: "test",
                    kind: SettingType::Choice(
                        vec!["false".to_string(), "true".to_string()],
                        if config.test { 1 } else { 0 },
                    ),
                },
                Setting {
                    name: "Border Type",
                    key: "border_test",
                    kind: SettingType::Choice(
                        vec![
                            "none".to_string(),
                            "light".to_string(),
                            "heavy".to_string(),
                            "double".to_string(),
                            "rounded".to_string(),
                        ],
                        match config.border_test.as_str() {
                            "none" => 0,
                            "light" => 1,
                            "heavy" => 2,
                            "double" => 3,
                            "rounded" => 4,
                            _ => 1,
                        },
                    ),
                },
            ],
        },
        Category {
            name: "Advanced",
            settings: vec![
                Setting {
                    name: "Something (Number)",
                    key: "something",
                    kind: SettingType::Custom {
                        value: config.something.to_string(),
                        default: "0".to_string(),
                        validation: CustomType::Number,
                    },
                },
                Setting {
                    name: "Primary Color",
                    key: "primary_color",
                    kind: SettingType::Custom {
                        value: config.primary_color.clone(),
                        default: "#FFFFFF".to_string(),
                        validation: CustomType::Color,
                    },
                },
            ],
        },
    ];

    let mut cat_selected = 0;
    let mut cat_scroll = 0;

    let mut det_selected = vec![0; categories.len()];
    let mut det_scroll = vec![0; categories.len()];

    let mut dirty = true;

    let apply_settings = |categories: &[Category], config: &mut Config| {
        for cat in categories {
            for set in &cat.settings {
                match set.key {
                    "test" => {
                        if let SettingType::Choice(_, idx) = &set.kind {
                            config.test = *idx == 1;
                        }
                    }
                    "border_test" => {
                        if let SettingType::Choice(opts, idx) = &set.kind {
                            config.border_test = opts[*idx].clone();
                        }
                    }
                    "something" => {
                        if let SettingType::Custom { value, .. } = &set.kind {
                            config.something = value.parse().unwrap_or(config.something);
                        }
                    }
                    "primary_color" => {
                        if let SettingType::Custom { value, .. } = &set.kind {
                            config.primary_color = value.clone();
                        }
                    }
                    _ => {}
                }
            }
        }
    };

    fn validate_custom(val: &str, kind: &CustomType) -> bool {
        match kind {
            CustomType::Text => true,
            CustomType::Number => val.parse::<i32>().is_ok(),
            CustomType::Color => {
                val.starts_with('#')
                    && val.len() == 7
                    && val.chars().skip(1).all(|c| c.is_ascii_hexdigit())
            }
        }
    }

    loop {
        let (current_w, current_h) = Terminal::size();

        if current_w != canvas.width || current_h != canvas.height {
            canvas.resize(current_w, current_h);
            dirty = true;
        }

        if dirty {
            canvas.clean();

            if current_w < min_w || current_h < min_h {
                crate::toosmall::render(canvas, current_w, current_h);
                canvas.render();
            } else {
                draw_background(canvas, config, current_w, current_h);
                canvas.apply_dim();

                let term_w = canvas.width;
                let term_h = canvas.height;

                let modal_w = 64.min(term_w.saturating_sub(4));
                let modal_h = 24.min(term_h.saturating_sub(4));

                let start_x = ((term_w.saturating_sub(modal_w)) / 2) as i16;
                let start_y = ((term_h.saturating_sub(modal_h)) / 2) as i16;

                let left_w = modal_w / 3;
                let right_w = modal_w.saturating_sub(left_w);

                let cat_color = if active_panel == ActiveSettingsPanel::Categories {
                    Color::White
                } else {
                    Color::Blue
                };
                let cat_border = if active_panel == ActiveSettingsPanel::Categories {
                    Border::Heavy
                } else {
                    Border::Light
                };
                let mut cat_box = Box::new(
                    left_w,
                    modal_h,
                    1,
                    cat_border,
                    Style {
                        fg: cat_color,
                        bg: Color::None,
                        md: Modifier::None,
                    },
                );
                cat_box.insert_text(
                    " Categories ",
                    1,
                    -1,
                    false,
                    Style {
                        fg: Color::Yellow,
                        bg: Color::None,
                        md: Modifier::Bold,
                    },
                );

                let visible_items = modal_h.saturating_sub(2) as usize;

                if cat_selected < cat_scroll {
                    cat_scroll = cat_selected;
                } else if cat_selected >= cat_scroll + visible_items && visible_items > 0 {
                    cat_scroll = cat_selected.saturating_sub(visible_items - 1);
                }

                let max_cat_len = left_w.saturating_sub(2) as usize;

                for (i, cat) in categories
                    .iter()
                    .enumerate()
                    .skip(cat_scroll)
                    .take(visible_items)
                {
                    let is_selected = i == cat_selected;
                    let is_active = active_panel == ActiveSettingsPanel::Categories;

                    let (fg_color, bg_color) = if is_selected && is_active {
                        (Color::Black, cat_color)
                    } else if is_selected {
                        (cat_color, Color::None)
                    } else {
                        (Color::DarkGray, Color::None)
                    };

                    let style = Style {
                        fg: fg_color,
                        bg: bg_color,
                        md: if is_selected && is_active {
                            Modifier::Bold
                        } else {
                            Modifier::None
                        },
                    };

                    let mut text = cat.name.to_string();
                    let char_count = text.chars().count();
                    if char_count > max_cat_len {
                        text = text.chars().take(max_cat_len).collect();
                    } else {
                        text.push_str(&" ".repeat(max_cat_len.saturating_sub(char_count + 2)));
                    }

                    let display_y = (i - cat_scroll) as i16;
                    cat_box.insert_text(&text, 1, display_y, false, style);
                }

                let det_color = if active_panel == ActiveSettingsPanel::Details {
                    Color::White
                } else {
                    Color::Blue
                };
                let det_border = if active_panel == ActiveSettingsPanel::Details {
                    Border::Heavy
                } else {
                    Border::Light
                };
                let mut det_box = Box::new(
                    right_w,
                    modal_h,
                    1,
                    det_border,
                    Style {
                        fg: det_color,
                        bg: Color::None,
                        md: Modifier::None,
                    },
                );
                det_box.insert_text(
                    " Settings ",
                    1,
                    -1,
                    false,
                    Style {
                        fg: Color::Yellow,
                        bg: Color::None,
                        md: Modifier::Bold,
                    },
                );

                let current_cat = &categories[cat_selected];
                let current_settings = &current_cat.settings;
                let current_det_sel = det_selected[cat_selected];
                let mut current_det_scroll = det_scroll[cat_selected];

                if current_det_sel < current_det_scroll {
                    current_det_scroll = current_det_sel;
                } else if current_det_sel >= current_det_scroll + visible_items && visible_items > 0
                {
                    current_det_scroll = current_det_sel.saturating_sub(visible_items - 1);
                }
                det_scroll[cat_selected] = current_det_scroll;

                let max_det_len = right_w.saturating_sub(2) as usize;

                for (i, setting) in current_settings
                    .iter()
                    .enumerate()
                    .skip(current_det_scroll)
                    .take(visible_items)
                {
                    let is_selected = i == current_det_sel;
                    let is_active = active_panel == ActiveSettingsPanel::Details;

                    let (fg_color, bg_color) = if is_selected && is_active {
                        (Color::Black, det_color)
                    } else if is_selected {
                        (det_color, Color::None)
                    } else {
                        (Color::DarkGray, Color::None)
                    };

                    let style = Style {
                        fg: fg_color,
                        bg: bg_color,
                        md: if is_selected && is_active {
                            Modifier::Bold
                        } else {
                            Modifier::None
                        },
                    };

                    let val_str = match &setting.kind {
                        SettingType::Choice(opts, idx) => {
                            if is_selected && is_active {
                                format!("< {} >", opts[*idx])
                            } else {
                                format!("{}", opts[*idx])
                            }
                        }
                        SettingType::Custom { value, .. } => {
                            if edit_mode && is_selected && is_active {
                                format!("[ {}_ ]", edit_buffer)
                            } else if is_selected && is_active {
                                format!("[ {} ]", value)
                            } else {
                                format!("{}", value)
                            }
                        }
                    };

                    let mut text = format!("{}: {}", setting.name, val_str);
                    let char_count = text.chars().count();

                    if char_count > max_det_len {
                        text = text.chars().take(max_det_len).collect();
                    } else {
                        text.push_str(&" ".repeat(max_det_len.saturating_sub(char_count + 2)));
                    }

                    let display_y = (i - current_det_scroll) as i16;
                    det_box.insert_text(&text, 1, display_y, false, style);
                }

                canvas.put_box_opaque(&cat_box, start_x, start_y);
                canvas.put_box_opaque(&det_box, start_x + (left_w as i16), start_y);
                canvas.render();
            }
            dirty = false;
        }

        match terminal.read_key(Duration::from_millis(16)) {
            Key::None => continue,
            key => {
                if edit_mode {
                    match key {
                        Key::Enter => {
                            let cat = &mut categories[cat_selected];
                            let setting = &mut cat.settings[det_selected[cat_selected]];
                            if let SettingType::Custom {
                                value,
                                default,
                                validation,
                            } = &mut setting.kind
                            {
                                if edit_buffer.is_empty() {
                                    *value = default.clone();
                                } else if validate_custom(&edit_buffer, validation) {
                                    *value = edit_buffer.clone();
                                }
                            }
                            apply_settings(&categories, config);
                            edit_mode = false;
                        }
                        Key::Esc => {
                            edit_mode = false;
                        }
                        Key::Char('\x03') => {
                            config.save();
                            return true;
                        }
                        Key::Char(c) => {
                            if c == '\x08' || c == '\x7F' {
                                edit_buffer.pop();
                            } else if !c.is_control() {
                                edit_buffer.push(c);
                            }
                        }
                        _ => {}
                    }
                    dirty = true;
                    continue;
                }

                match key {
                    Key::Esc => {
                        config.save();
                        return false;
                    }
                    Key::Char('q' | 'Q') | Key::Char('\x03') => {
                        config.save();
                        return true;
                    }
                    Key::Char('e' | 'E') => {
                        active_panel = if active_panel == ActiveSettingsPanel::Categories {
                            ActiveSettingsPanel::Details
                        } else {
                            ActiveSettingsPanel::Categories
                        };
                    }
                    Key::Left => {
                        if active_panel == ActiveSettingsPanel::Details {
                            let cat = &mut categories[cat_selected];
                            if !cat.settings.is_empty() {
                                let setting = &mut cat.settings[det_selected[cat_selected]];
                                match &mut setting.kind {
                                    SettingType::Choice(opts, idx) => {
                                        if *idx > 0 {
                                            *idx -= 1;
                                        } else {
                                            *idx = opts.len() - 1;
                                        }
                                        apply_settings(&categories, config);
                                    }
                                    SettingType::Custom { .. } => {}
                                }
                            }
                        }
                    }
                    Key::Right => {
                        if active_panel == ActiveSettingsPanel::Categories {
                            active_panel = ActiveSettingsPanel::Details;
                        } else if active_panel == ActiveSettingsPanel::Details {
                            let cat = &mut categories[cat_selected];
                            if !cat.settings.is_empty() {
                                let setting = &mut cat.settings[det_selected[cat_selected]];
                                match &mut setting.kind {
                                    SettingType::Choice(opts, idx) => {
                                        if *idx < opts.len() - 1 {
                                            *idx += 1;
                                        } else {
                                            *idx = 0;
                                        }
                                        apply_settings(&categories, config);
                                    }
                                    SettingType::Custom { .. } => {}
                                }
                            }
                        }
                    }
                    Key::Up => {
                        if active_panel == ActiveSettingsPanel::Categories {
                            if cat_selected > 0 {
                                cat_selected -= 1;
                            }
                        } else {
                            if det_selected[cat_selected] > 0 {
                                det_selected[cat_selected] -= 1;
                            }
                        }
                    }
                    Key::Down => {
                        if active_panel == ActiveSettingsPanel::Categories {
                            if cat_selected < categories.len().saturating_sub(1) {
                                cat_selected += 1;
                            }
                        } else {
                            let max_det = categories[cat_selected].settings.len().saturating_sub(1);
                            if det_selected[cat_selected] < max_det {
                                det_selected[cat_selected] += 1;
                            }
                        }
                    }
                    Key::Enter => {
                        if active_panel == ActiveSettingsPanel::Details {
                            let cat = &mut categories[cat_selected];
                            if !cat.settings.is_empty() {
                                let setting = &mut cat.settings[det_selected[cat_selected]];
                                if let SettingType::Custom { value, .. } = &setting.kind {
                                    edit_mode = true;
                                    edit_buffer = value.clone();
                                }
                            }
                        } else if active_panel == ActiveSettingsPanel::Categories {
                            active_panel = ActiveSettingsPanel::Details;
                        }
                    }
                    _ => continue,
                }
                dirty = true;
            }
        }
    }
}
