use std::time::Duration;

use crate::conf::Config;
use crate::{Border, Box, Canvas, Cell, Color, Key, Modifier, Style, Terminal};

#[derive(Debug, Clone, Copy, PartialEq)]
enum ActiveSettingsPanel {
    Categories,
    Details,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CustomType {
    Text,
    Char,
}

#[derive(Debug, Clone)]
pub enum SettingType {
    Choice(Vec<String>, usize),
    Custom {
        value: String,
        default: String,
        validation: CustomType,
    },
    Action,
}

pub fn available_themes() -> Vec<String> {
    let mut themes = Vec::new();
    for (name, _) in crate::theme::themecore::BUILTIN_THEMES {
        themes.push(name.to_string());
    }

    if let Some(proj_dirs) = directories::ProjectDirs::from("com", "Nexsq", "nuui") {
        let themes_dir = proj_dirs.config_dir().join("themes");
        if let Ok(entries) = std::fs::read_dir(themes_dir) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "conf") {
                    if let Some(stem) = path.file_stem() {
                        let name = stem.to_string_lossy().into_owned();
                        if !themes.contains(&name) {
                            themes.push(name);
                        }
                    }
                }
            }
        }
    }

    themes.sort();
    themes
}

fn build_categories(config: &Config, themes: &[String], theme_idx: usize) -> Vec<Category> {
    let def = Config::default();

    vec![
        Category {
            name: "Appearance",
            settings: vec![
                Setting {
                    name: "Theme",
                    key: "theme",
                    kind: SettingType::Choice(themes.to_vec(), theme_idx),
                },
                Setting {
                    name: "Selected Indicator",
                    key: "indicator_style",
                    kind: SettingType::Choice(
                        vec!["border".to_string(), "corner".to_string()],
                        if config.indicator_style == "corner" {
                            1
                        } else {
                            0
                        },
                    ),
                },
                Setting {
                    name: "Reset Appearance",
                    key: "reset_appearance",
                    kind: SettingType::Action,
                },
            ],
        },
        Category {
            name: "General",
            settings: vec![Setting {
                name: "Reset Config",
                key: "reset_config",
                kind: SettingType::Action,
            }],
        },
        Category {
            name: "Editor Keybinds",
            settings: vec![
                Setting {
                    name: "Insert Mode",
                    key: "bind_insert",
                    kind: SettingType::Custom {
                        value: config.bind_insert.to_string(),
                        default: def.bind_insert.to_string(),
                        validation: CustomType::Char,
                    },
                },
                Setting {
                    name: "Visual Mode",
                    key: "bind_visual",
                    kind: SettingType::Custom {
                        value: config.bind_visual.to_string(),
                        default: def.bind_visual.to_string(),
                        validation: CustomType::Char,
                    },
                },
                Setting {
                    name: "Move Left",
                    key: "bind_left",
                    kind: SettingType::Custom {
                        value: config.bind_left.to_string(),
                        default: def.bind_left.to_string(),
                        validation: CustomType::Char,
                    },
                },
                Setting {
                    name: "Move Right",
                    key: "bind_right",
                    kind: SettingType::Custom {
                        value: config.bind_right.to_string(),
                        default: def.bind_right.to_string(),
                        validation: CustomType::Char,
                    },
                },
                Setting {
                    name: "Move Up",
                    key: "bind_up",
                    kind: SettingType::Custom {
                        value: config.bind_up.to_string(),
                        default: def.bind_up.to_string(),
                        validation: CustomType::Char,
                    },
                },
                Setting {
                    name: "Move Down",
                    key: "bind_down",
                    kind: SettingType::Custom {
                        value: config.bind_down.to_string(),
                        default: def.bind_down.to_string(),
                        validation: CustomType::Char,
                    },
                },
                Setting {
                    name: "Next Word",
                    key: "bind_word_next",
                    kind: SettingType::Custom {
                        value: config.bind_word_next.to_string(),
                        default: def.bind_word_next.to_string(),
                        validation: CustomType::Char,
                    },
                },
                Setting {
                    name: "Prev Word",
                    key: "bind_word_prev",
                    kind: SettingType::Custom {
                        value: config.bind_word_prev.to_string(),
                        default: def.bind_word_prev.to_string(),
                        validation: CustomType::Char,
                    },
                },
                Setting {
                    name: "Line Start",
                    key: "bind_line_start",
                    kind: SettingType::Custom {
                        value: config.bind_line_start.to_string(),
                        default: def.bind_line_start.to_string(),
                        validation: CustomType::Char,
                    },
                },
                Setting {
                    name: "Line End",
                    key: "bind_line_end",
                    kind: SettingType::Custom {
                        value: config.bind_line_end.to_string(),
                        default: def.bind_line_end.to_string(),
                        validation: CustomType::Char,
                    },
                },
                Setting {
                    name: "Select All",
                    key: "bind_select_all",
                    kind: SettingType::Custom {
                        value: config.bind_select_all.to_string(),
                        default: def.bind_select_all.to_string(),
                        validation: CustomType::Char,
                    },
                },
                Setting {
                    name: "File Bounds",
                    key: "bind_file_bounds",
                    kind: SettingType::Custom {
                        value: config.bind_file_bounds.to_string(),
                        default: def.bind_file_bounds.to_string(),
                        validation: CustomType::Char,
                    },
                },
                Setting {
                    name: "Delete",
                    key: "bind_delete",
                    kind: SettingType::Custom {
                        value: config.bind_delete.to_string(),
                        default: def.bind_delete.to_string(),
                        validation: CustomType::Char,
                    },
                },
                Setting {
                    name: "Copy",
                    key: "bind_copy",
                    kind: SettingType::Custom {
                        value: config.bind_copy.to_string(),
                        default: def.bind_copy.to_string(),
                        validation: CustomType::Char,
                    },
                },
                Setting {
                    name: "Paste",
                    key: "bind_paste",
                    kind: SettingType::Custom {
                        value: config.bind_paste.to_string(),
                        default: def.bind_paste.to_string(),
                        validation: CustomType::Char,
                    },
                },
                Setting {
                    name: "Undo",
                    key: "bind_undo",
                    kind: SettingType::Custom {
                        value: config.bind_undo.to_string(),
                        default: def.bind_undo.to_string(),
                        validation: CustomType::Char,
                    },
                },
                Setting {
                    name: "Redo",
                    key: "bind_redo",
                    kind: SettingType::Custom {
                        value: config.bind_redo.to_string(),
                        default: def.bind_redo.to_string(),
                        validation: CustomType::Char,
                    },
                },
                Setting {
                    name: "Save",
                    key: "bind_save",
                    kind: SettingType::Custom {
                        value: config.bind_save.to_string(),
                        default: def.bind_save.to_string(),
                        validation: CustomType::Char,
                    },
                },
                Setting {
                    name: "Reset Keybinds",
                    key: "reset_keybinds",
                    kind: SettingType::Action,
                },
            ],
        },
    ]
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

pub fn settings_modal(
    terminal: &Terminal,
    canvas: &mut Canvas,
    config: &mut Config,
    main_view: &mut crate::main::MainView,
) -> bool {
    let mut active_panel = ActiveSettingsPanel::Categories;

    let mut edit_mode = false;
    let mut edit_buffer = String::new();

    let themes = available_themes();
    let current_theme_idx = themes.iter().position(|t| t == &config.theme).unwrap_or(0);

    let mut categories = build_categories(config, &themes, current_theme_idx);

    let mut cat_selected = 0;
    let mut cat_scroll = 0;

    let mut det_selected = vec![0; categories.len()];
    let mut det_scroll = vec![0; categories.len()];

    let mut prev_theme = config.theme.clone();
    let mut dirty = true;

    let version_str = format!(" v{} ", env!("CARGO_PKG_VERSION"));
    let version_len = version_str.chars().count() as u16;

    macro_rules! apply_setting {
        ($config:expr, $set:expr, $key:expr, $field:ident) => {
            if $set.key == $key {
                if let SettingType::Custom { value, .. } = &$set.kind {
                    $config.$field = value.chars().next().unwrap_or($config.$field);
                }
            }
        };
        ($config:expr, $set:expr, choice $key:expr, $field:ident) => {
            if $set.key == $key {
                if let SettingType::Choice(opts, idx) = &$set.kind {
                    $config.$field = opts[*idx].clone();
                }
            }
        };
    }

    let apply_settings = |categories: &[Category], config: &mut Config| {
        for cat in categories {
            for set in &cat.settings {
                apply_setting!(config, set, choice "theme", theme);
                apply_setting!(config, set, choice "indicator_style", indicator_style);

                apply_setting!(config, set, "bind_insert", bind_insert);
                apply_setting!(config, set, "bind_visual", bind_visual);
                apply_setting!(config, set, "bind_left", bind_left);
                apply_setting!(config, set, "bind_right", bind_right);
                apply_setting!(config, set, "bind_up", bind_up);
                apply_setting!(config, set, "bind_down", bind_down);
                apply_setting!(config, set, "bind_word_next", bind_word_next);
                apply_setting!(config, set, "bind_word_prev", bind_word_prev);
                apply_setting!(config, set, "bind_line_start", bind_line_start);
                apply_setting!(config, set, "bind_line_end", bind_line_end);
                apply_setting!(config, set, "bind_select_all", bind_select_all);
                apply_setting!(config, set, "bind_file_bounds", bind_file_bounds);
                apply_setting!(config, set, "bind_delete", bind_delete);
                apply_setting!(config, set, "bind_copy", bind_copy);
                apply_setting!(config, set, "bind_paste", bind_paste);
                apply_setting!(config, set, "bind_undo", bind_undo);
                apply_setting!(config, set, "bind_redo", bind_redo);
                apply_setting!(config, set, "bind_save", bind_save);
            }
        }
    };

    fn validate_custom(val: &str, kind: &CustomType) -> bool {
        match kind {
            CustomType::Text => true,
            CustomType::Char => {
                val.chars().count() == 1 && val.chars().next().unwrap().is_ascii_alphanumeric()
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

            let current_min_w = main_view.min_w;
            let current_min_h = main_view.min_h;

            if current_w < current_min_w || current_h < current_min_h {
                crate::toosmall::render(canvas, current_w, current_h);
                canvas.render();
            } else {
                if current_w != main_view.term_w || current_h != main_view.term_h {
                    main_view.resize(current_w, current_h, config);
                } else {
                    main_view.refresh_all(config);
                }
                main_view.render(canvas);
                canvas.apply_dim();

                let term_w = canvas.width;
                let term_h = canvas.height;

                let modal_w = 64.min(term_w.saturating_sub(4));
                let modal_h = 24.min(term_h.saturating_sub(4));

                let start_x = ((term_w.saturating_sub(modal_w)) / 2) as i16;
                let start_y = ((term_h.saturating_sub(modal_h)) / 2) as i16;

                let left_w = modal_w / 3;
                let right_w = modal_w.saturating_sub(left_w);

                let use_corner = config.indicator_style == "corner";

                let cat_active = active_panel == ActiveSettingsPanel::Categories;
                let cat_border = if cat_active && !use_corner {
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
                        fg: if cat_active && !use_corner {
                            main_view.theme.selected_box
                        } else {
                            main_view.theme.settings_category_box
                        },
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
                        fg: main_view.theme.main_label,
                        bg: Color::None,
                        md: Modifier::Bold,
                    },
                );

                if cat_active && use_corner {
                    if cat_box.width > 0 {
                        cat_box.put_cell(
                            Cell {
                                c: '■',
                                s: Style {
                                    fg: main_view.theme.selected_box,
                                    bg: Color::None,
                                    md: Modifier::None,
                                },
                            },
                            cat_box.width - 1,
                            0,
                        );
                    }
                }

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

                    let (fg_color, bg_color) = if is_selected && cat_active {
                        (Color::Black, main_view.theme.settings_selected)
                    } else if is_selected {
                        (main_view.theme.settings_selected, Color::None)
                    } else {
                        (main_view.theme.settings_entry, Color::None)
                    };

                    let style = Style {
                        fg: fg_color,
                        bg: bg_color,
                        md: if is_selected && cat_active {
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

                let det_active = active_panel == ActiveSettingsPanel::Details;
                let det_border = if det_active && !use_corner {
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
                        fg: if det_active && !use_corner {
                            main_view.theme.selected_box
                        } else {
                            main_view.theme.settings_options_box
                        },
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
                        fg: main_view.theme.main_label,
                        bg: Color::None,
                        md: Modifier::Bold,
                    },
                );

                if det_active && use_corner {
                    if det_box.width > 0 {
                        det_box.put_cell(
                            Cell {
                                c: '■',
                                s: Style {
                                    fg: main_view.theme.selected_box,
                                    bg: Color::None,
                                    md: Modifier::None,
                                },
                            },
                            det_box.width - 1,
                            0,
                        );
                    }
                }

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

                    let is_action = matches!(setting.kind, SettingType::Action);

                    let (mut fg_color, mut bg_color) = if is_selected && det_active {
                        (Color::Black, main_view.theme.settings_selected)
                    } else if is_selected {
                        (main_view.theme.settings_selected, Color::None)
                    } else {
                        (main_view.theme.settings_entry, Color::None)
                    };

                    if is_action {
                        if is_selected && det_active {
                            fg_color = Color::Black;
                            bg_color = main_view.theme.settings_special;
                        } else if is_selected {
                            fg_color = main_view.theme.settings_special;
                        } else {
                            fg_color = main_view.theme.settings_entry;
                        }
                    }

                    let style = Style {
                        fg: fg_color,
                        bg: bg_color,
                        md: if is_selected && det_active {
                            Modifier::Bold
                        } else {
                            Modifier::None
                        },
                    };

                    let mut text = match &setting.kind {
                        SettingType::Action => setting.name.to_string(),
                        _ => {
                            let val_str = match &setting.kind {
                                SettingType::Choice(opts, idx) => {
                                    if is_selected && det_active {
                                        format!("< {} >", opts[*idx])
                                    } else {
                                        format!("{}", opts[*idx])
                                    }
                                }
                                SettingType::Custom { value, .. } => {
                                    if edit_mode && is_selected && det_active {
                                        format!("[ {}_ ]", edit_buffer)
                                    } else if is_selected && det_active {
                                        format!("[ {} ]", value)
                                    } else {
                                        format!("{}", value)
                                    }
                                }
                                SettingType::Action => unreachable!(),
                            };
                            format!("{}: {}", setting.name, val_str)
                        }
                    };

                    let char_count = text.chars().count();

                    if char_count > max_det_len {
                        text = text.chars().take(max_det_len).collect();
                    } else {
                        for _ in 0..(max_det_len.saturating_sub(char_count + 2)) {
                            text.push(' ');
                        }
                    }

                    let display_y = (i - current_det_scroll) as i16;
                    det_box.insert_text(&text, 1, display_y, false, style);
                }

                let v_y = det_box.height.saturating_sub(1);
                let v_start_x = det_box.width.saturating_sub(3 + version_len);

                let det_style = Style {
                    fg: main_view.theme.settings_options_box,
                    bg: Color::None,
                    md: Modifier::None,
                };

                for (i, c) in version_str.chars().enumerate() {
                    let offset = i as u16;
                    if v_start_x + offset < det_box.width {
                        det_box.put_cell(Cell { c, s: det_style }, v_start_x + offset, v_y);
                    }
                }

                canvas.put_box_opaque(&cat_box, start_x, start_y);
                canvas.put_box_opaque(&det_box, start_x + (left_w as i16), start_y);
                canvas.render();
            }
            dirty = false;
        }

        let mut check_theme = false;

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
                            check_theme = true;
                            edit_mode = false;
                        }
                        Key::Esc => {
                            edit_mode = false;
                        }
                        Key::Backspace => {
                            edit_buffer.pop();
                        }
                        Key::Char('\x03') => {
                            config.save();
                            return true;
                        }
                        Key::Char(c) => {
                            if !c.is_control() {
                                let caps = Terminal::is_caps_lock_on();
                                let mut final_c = c;
                                if caps && c.is_ascii_alphabetic() {
                                    final_c = c.to_ascii_uppercase();
                                }
                                let is_char_type = {
                                    let cat = &categories[cat_selected];
                                    let setting = &cat.settings[det_selected[cat_selected]];
                                    matches!(
                                        setting.kind,
                                        SettingType::Custom {
                                            validation: CustomType::Char,
                                            ..
                                        }
                                    )
                                };
                                if is_char_type {
                                    edit_buffer = final_c.to_ascii_lowercase().to_string();
                                } else {
                                    edit_buffer.push(final_c);
                                }
                            }
                        }
                        Key::Shift(c) => {
                            if !c.is_control() {
                                let caps = Terminal::is_caps_lock_on();
                                let mut final_c = c.to_ascii_uppercase();
                                if caps && final_c.is_ascii_alphabetic() {
                                    final_c = final_c.to_ascii_lowercase();
                                }
                                let is_char_type = {
                                    let cat = &categories[cat_selected];
                                    let setting = &cat.settings[det_selected[cat_selected]];
                                    matches!(
                                        setting.kind,
                                        SettingType::Custom {
                                            validation: CustomType::Char,
                                            ..
                                        }
                                    )
                                };
                                if is_char_type {
                                    edit_buffer = final_c.to_ascii_lowercase().to_string();
                                } else {
                                    edit_buffer.push(final_c);
                                }
                            }
                        }
                        _ => {}
                    }
                    dirty = true;
                } else {
                    match key {
                        Key::Esc => {
                            config.save();
                            return false;
                        }
                        Key::Char('q') | Key::Char('\x03') => {
                            config.save();
                            return true;
                        }
                        Key::Tab => {
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
                                            check_theme = true;
                                        }
                                        SettingType::Custom { .. } | SettingType::Action => {}
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
                                            check_theme = true;
                                        }
                                        SettingType::Custom { .. } | SettingType::Action => {}
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
                                let max_det =
                                    categories[cat_selected].settings.len().saturating_sub(1);
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
                                    match &setting.kind {
                                        SettingType::Custom { value, .. } => {
                                            edit_mode = true;
                                            edit_buffer = value.clone();
                                        }
                                        SettingType::Action => {
                                            let (msg, is_config, is_appearance) = match setting.key
                                            {
                                                "reset_config" => (
                                                    "Reset all configurations to default?\n\nAre you sure?",
                                                    true,
                                                    false,
                                                ),
                                                "reset_keybinds" => (
                                                    "Reset all editor keybinds to default?\n\nAre you sure?",
                                                    false,
                                                    false,
                                                ),
                                                "reset_appearance" => (
                                                    "Reset appearance settings to default?\n\nAre you sure?",
                                                    false,
                                                    true,
                                                ),
                                                _ => ("", false, false),
                                            };

                                            let result = crate::error::warning_box(
                                                terminal,
                                                canvas,
                                                msg,
                                                &["CANCEL", "CONFIRM"],
                                                0,
                                                0,
                                                main_view.min_w,
                                                main_view.min_h,
                                                |cvs, w, h| {
                                                    if w != main_view.term_w
                                                        || h != main_view.term_h
                                                    {
                                                        main_view.resize(w, h, config);
                                                    } else {
                                                        main_view.refresh_all(config);
                                                    }
                                                    main_view.render(cvs);
                                                },
                                            );

                                            if result == crate::PanelResult::Ok(1) {
                                                if is_config {
                                                    *config = Config::default();
                                                } else if is_appearance {
                                                    config.reset_appearance();
                                                } else {
                                                    config.reset_keybinds();
                                                }
                                                config.save();
                                                let themes = available_themes();
                                                let current_theme_idx = themes
                                                    .iter()
                                                    .position(|t| t == &config.theme)
                                                    .unwrap_or(0);
                                                categories = build_categories(
                                                    config,
                                                    &themes,
                                                    current_theme_idx,
                                                );
                                                check_theme = true;
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            } else if active_panel == ActiveSettingsPanel::Categories {
                                active_panel = ActiveSettingsPanel::Details;
                            }
                        }
                        _ => {}
                    }
                    dirty = true;
                }
            }
        }

        if check_theme && config.theme != prev_theme {
            match crate::theme::themecore::init(&config.theme) {
                Ok(new_theme) => {
                    main_view.theme = new_theme;
                    let header_h = main_view.theme.title.len().max(1) as u16;
                    main_view.min_h = 13 + header_h;

                    let (term_w, term_h) = Terminal::size();
                    main_view.resize(term_w, term_h, config);

                    prev_theme = config.theme.clone();
                }
                Err(e) => {
                    let msg = format!("Failed to load theme '{}':\n{}", config.theme, e);
                    crate::error::warning_box(
                        terminal,
                        canvas,
                        &msg,
                        &["OK"],
                        0,
                        0,
                        main_view.min_w,
                        main_view.min_h,
                        |cvs, w, h| {
                            if w != main_view.term_w || h != main_view.term_h {
                                main_view.resize(w, h, config);
                            } else {
                                main_view.refresh_all(config);
                            }
                            main_view.render(cvs);
                        },
                    );
                    config.theme = prev_theme.clone();
                    let themes = available_themes();
                    let current_theme_idx =
                        themes.iter().position(|t| t == &config.theme).unwrap_or(0);
                    categories = build_categories(config, &themes, current_theme_idx);
                }
            }
            dirty = true;
        }
    }
}
