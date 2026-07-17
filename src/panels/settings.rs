use std::time::Duration;

use crate::conf::Config;
use crate::{Box, Canvas, Cell, Color, Gradient, Key, Modifier, Style, Terminal};

#[derive(Debug, Clone, Copy, PartialEq)]
enum ActiveSettingsPanel {
    Categories,
    Details,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CustomType {
    Text,
    Char,
    Int,
    FloatRange(f32, f32),
    Gradient,
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
                    name: "Border",
                    key: "border_style",
                    kind: SettingType::Choice(
                        vec![
                            "round".to_string(),
                            "squared".to_string(),
                            "heavy".to_string(),
                        ],
                        match config.border_style.as_str() {
                            "round" => 0,
                            "heavy" => 2,
                            _ => 1,
                        },
                    ),
                },
                Setting {
                    name: "Selected Indicator",
                    key: "indicator_style",
                    kind: SettingType::Choice(
                        vec![
                            "border".to_string(),
                            "corner".to_string(),
                            "corners".to_string(),
                        ],
                        match config.indicator_style.as_str() {
                            "corner" => 1,
                            "corners" => 2,
                            _ => 0,
                        },
                    ),
                },
                Setting {
                    name: "Tabs",
                    key: "tabs_num",
                    kind: SettingType::Choice(
                        vec![
                            "1".to_string(),
                            "2".to_string(),
                            "3".to_string(),
                            "4".to_string(),
                            "5".to_string(),
                            "6".to_string(),
                        ],
                        config.tabs_num.clamp(1, 6).saturating_sub(1),
                    ),
                },
                Setting {
                    name: "Lib Width",
                    key: "lib_width",
                    kind: SettingType::Custom {
                        value: config.lib_width.to_string(),
                        default: def.lib_width.to_string(),
                        validation: CustomType::Int,
                    },
                },
                Setting {
                    name: "Reset Appearance",
                    key: "reset_appearance",
                    kind: SettingType::Action,
                },
            ],
        },
        Category {
            name: "Deck",
            settings: {
                let mut s = vec![Setting {
                    name: "Deck Mode",
                    key: "deck_mode",
                    kind: SettingType::Choice(
                        vec![
                            "none".to_string(),
                            "title".to_string(),
                            "widget".to_string(),
                        ],
                        match config.deck_mode.as_str() {
                            "none" => 0,
                            "title" => 1,
                            _ => 2,
                        },
                    ),
                }];
                if config.deck_mode == "widget" {
                    s.push(Setting {
                        name: "Deck Widget",
                        key: "deck_widget",
                        kind: SettingType::Choice(
                            vec![
                                "keyvis".to_string(),
                                "monitor".to_string(),
                                "clock".to_string(),
                            ],
                            match config.deck_widget.as_str() {
                                "monitor" => 1,
                                "clock" => 2,
                                _ => 0,
                            },
                        ),
                    });

                    if config.deck_widget == "monitor" {
                        s.push(Setting {
                            name: "Monitor CPU",
                            key: "monitor_cpu",
                            kind: SettingType::Choice(
                                vec![
                                    "off".to_string(),
                                    "pct".to_string(),
                                    "bar".to_string(),
                                    "graph".to_string(),
                                    "pctbar".to_string(),
                                    "pctgraph".to_string(),
                                ],
                                match config.monitor_cpu.as_str() {
                                    "pct" => 1,
                                    "bar" => 2,
                                    "graph" => 3,
                                    "pctbar" => 4,
                                    "pctgraph" => 5,
                                    _ => 0,
                                },
                            ),
                        });
                        s.push(Setting {
                            name: "Monitor GPU",
                            key: "monitor_gpu",
                            kind: SettingType::Choice(
                                vec![
                                    "off".to_string(),
                                    "pct".to_string(),
                                    "bar".to_string(),
                                    "graph".to_string(),
                                    "pctbar".to_string(),
                                    "pctgraph".to_string(),
                                ],
                                match config.monitor_gpu.as_str() {
                                    "pct" => 1,
                                    "bar" => 2,
                                    "graph" => 3,
                                    "pctbar" => 4,
                                    "pctgraph" => 5,
                                    _ => 0,
                                },
                            ),
                        });
                        s.push(Setting {
                            name: "Monitor MEM",
                            key: "monitor_mem",
                            kind: SettingType::Choice(
                                vec![
                                    "off".to_string(),
                                    "pct".to_string(),
                                    "used".to_string(),
                                    "bar".to_string(),
                                    "graph".to_string(),
                                    "pctbar".to_string(),
                                    "pctgraph".to_string(),
                                ],
                                match config.monitor_mem.as_str() {
                                    "pct" => 1,
                                    "used" => 2,
                                    "bar" => 3,
                                    "graph" => 4,
                                    "pctbar" => 5,
                                    "pctgraph" => 6,
                                    _ => 0,
                                },
                            ),
                        });
                        s.push(Setting {
                            name: "Monitor TERM",
                            key: "monitor_term",
                            kind: SettingType::Choice(
                                vec!["off".to_string(), "on".to_string()],
                                match config.monitor_term.as_str() {
                                    "on" => 1,
                                    _ => 0,
                                },
                            ),
                        });
                        s.push(Setting {
                            name: "Monitor Divider",
                            key: "monitor_divider",
                            kind: SettingType::Choice(
                                vec!["show".to_string(), "hide".to_string()],
                                match config.monitor_divider.as_str() {
                                    "hide" => 1,
                                    _ => 0,
                                },
                            ),
                        });
                        s.push(Setting {
                            name: "Monitor Bar",
                            key: "monitor_bar_mode",
                            kind: SettingType::Choice(
                                vec![
                                    "background".to_string(),
                                    "caps".to_string(),
                                    "clean".to_string(),
                                ],
                                match config.monitor_bar_mode.as_str() {
                                    "caps" => 1,
                                    "clean" => 2,
                                    _ => 0,
                                },
                            ),
                        });
                        s.push(Setting {
                            name: "Monitor Bar Width",
                            key: "monitor_bar_width",
                            kind: SettingType::Custom {
                                value: config.monitor_bar_width.to_string(),
                                default: def.monitor_bar_width.to_string(),
                                validation: CustomType::Int,
                            },
                        });
                        s.push(Setting {
                            name: "Monitor Icons",
                            key: "monitor_icons",
                            kind: SettingType::Choice(
                                vec!["true".to_string(), "false".to_string()],
                                if config.monitor_icons { 0 } else { 1 },
                            ),
                        });
                    } else if config.deck_widget == "keyvis" {
                        s.push(Setting {
                            name: "Keyvis Width",
                            key: "keyvis_width",
                            kind: SettingType::Custom {
                                value: config.keyvis_width.to_string(),
                                default: def.keyvis_width.to_string(),
                                validation: CustomType::Int,
                            },
                        });
                        s.push(Setting {
                            name: "Keyvis Height",
                            key: "keyvis_height",
                            kind: SettingType::Custom {
                                value: config.keyvis_height.to_string(),
                                default: def.keyvis_height.to_string(),
                                validation: CustomType::Int,
                            },
                        });
                        s.push(Setting {
                            name: "Keyvis Steps",
                            key: "keyvis_steps",
                            kind: SettingType::Choice(
                                vec![
                                    "1".to_string(),
                                    "2".to_string(),
                                    "3".to_string(),
                                    "4".to_string(),
                                ],
                                config.keyvis_steps.clamp(1, 4).saturating_sub(1),
                            ),
                        });
                        s.push(Setting {
                            name: "Keyvis Spread",
                            key: "keyvis_spread",
                            kind: SettingType::Custom {
                                value: config.keyvis_spread.to_string(),
                                default: def.keyvis_spread.to_string(),
                                validation: CustomType::Int,
                            },
                        });
                        s.push(Setting {
                            name: "Keyvis Force",
                            key: "keyvis_force",
                            kind: SettingType::Custom {
                                value: config.keyvis_force.to_string(),
                                default: def.keyvis_force.to_string(),
                                validation: CustomType::FloatRange(0.1, 1.0),
                            },
                        });
                        s.push(Setting {
                            name: "Keyvis Gravity",
                            key: "keyvis_gravity",
                            kind: SettingType::Custom {
                                value: config.keyvis_gravity.to_string(),
                                default: def.keyvis_gravity.to_string(),
                                validation: CustomType::FloatRange(0.1, 1.0),
                            },
                        });
                        s.push(Setting {
                            name: "Keyvis Tension",
                            key: "keyvis_tension",
                            kind: SettingType::Custom {
                                value: config.keyvis_tension.to_string(),
                                default: def.keyvis_tension.to_string(),
                                validation: CustomType::FloatRange(0.1, 1.0),
                            },
                        });
                        s.push(Setting {
                            name: "Keyvis Base",
                            key: "keyvis_base",
                            kind: SettingType::Choice(
                                vec!["true".to_string(), "false".to_string()],
                                if config.keyvis_base { 0 } else { 1 },
                            ),
                        });
                    } else if config.deck_widget == "clock" {
                        s.push(Setting {
                            name: "Date",
                            key: "clock_date",
                            kind: SettingType::Choice(
                                vec![
                                    "off".to_string(),
                                    "eu".to_string(),
                                    "us".to_string(),
                                    "clean".to_string(),
                                    "mon name".to_string(),
                                    "rfc 2822".to_string(),
                                ],
                                match config.clock_date.as_str() {
                                    "eu" => 1,
                                    "us" => 2,
                                    "clean" => 3,
                                    "mon name" => 4,
                                    "rfc 2822" => 5,
                                    _ => 0,
                                },
                            ),
                        });
                        s.push(Setting {
                            name: "Mode",
                            key: "clock_mode",
                            kind: SettingType::Choice(
                                vec!["small".to_string(), "big".to_string()],
                                match config.clock_mode.as_str() {
                                    "big" => 1,
                                    _ => 0,
                                },
                            ),
                        });
                        s.push(Setting {
                            name: "Position",
                            key: "clock_position",
                            kind: SettingType::Choice(
                                vec!["left".to_string(), "mid".to_string(), "right".to_string()],
                                match config.clock_position.as_str() {
                                    "mid" => 1,
                                    "right" => 2,
                                    _ => 0,
                                },
                            ),
                        });
                        s.push(Setting {
                            name: "Format",
                            key: "clock_format",
                            kind: SettingType::Choice(
                                vec!["12h".to_string(), "24h".to_string()],
                                match config.clock_format.as_str() {
                                    "12h" => 0,
                                    _ => 1,
                                },
                            ),
                        });
                        s.push(Setting {
                            name: "Show Seconds",
                            key: "clock_seconds",
                            kind: SettingType::Choice(
                                vec!["true".to_string(), "false".to_string()],
                                if config.clock_seconds { 0 } else { 1 },
                            ),
                        });
                    }
                }
                s.push(Setting {
                    name: "Reset Deck",
                    key: "reset_deck",
                    kind: SettingType::Action,
                });
                s
            },
        },
        Category {
            name: "Library",
            settings: vec![
                Setting {
                    name: "Sorting",
                    key: "lib_sorting",
                    kind: SettingType::Choice(
                        vec![
                            "ascending".to_string(),
                            "descending".to_string(),
                            "custom".to_string(),
                        ],
                        match config.lib_sorting.as_str() {
                            "descending" => 1,
                            "custom" => 2,
                            _ => 0,
                        },
                    ),
                },
                Setting {
                    name: "Reset Order",
                    key: "reset_order",
                    kind: SettingType::Action,
                },
                Setting {
                    name: "New File",
                    key: "bind_lib_new_file",
                    kind: SettingType::Custom {
                        value: config.bind_lib_new_file.to_string(),
                        default: def.bind_lib_new_file.to_string(),
                        validation: CustomType::Char,
                    },
                },
                Setting {
                    name: "New Folder",
                    key: "bind_lib_new_folder",
                    kind: SettingType::Custom {
                        value: config.bind_lib_new_folder.to_string(),
                        default: def.bind_lib_new_folder.to_string(),
                        validation: CustomType::Char,
                    },
                },
                Setting {
                    name: "Rename",
                    key: "bind_lib_rename",
                    kind: SettingType::Custom {
                        value: config.bind_lib_rename.to_string(),
                        default: def.bind_lib_rename.to_string(),
                        validation: CustomType::Char,
                    },
                },
                Setting {
                    name: "Delete",
                    key: "bind_lib_delete",
                    kind: SettingType::Custom {
                        value: config.bind_lib_delete.to_string(),
                        default: def.bind_lib_delete.to_string(),
                        validation: CustomType::Char,
                    },
                },
                Setting {
                    name: "Move Up",
                    key: "bind_lib_move_up",
                    kind: SettingType::Custom {
                        value: config.bind_lib_move_up.to_string(),
                        default: def.bind_lib_move_up.to_string(),
                        validation: CustomType::Char,
                    },
                },
                Setting {
                    name: "Move Down",
                    key: "bind_lib_move_down",
                    kind: SettingType::Custom {
                        value: config.bind_lib_move_down.to_string(),
                        default: def.bind_lib_move_down.to_string(),
                        validation: CustomType::Char,
                    },
                },
                Setting {
                    name: "Reset Keybinds",
                    key: "reset_lib_keybinds",
                    kind: SettingType::Action,
                },
            ],
        },
        Category {
            name: "Editor",
            settings: vec![
                Setting {
                    name: "Insert Mode",
                    key: "bind_edit_insert",
                    kind: SettingType::Custom {
                        value: config.bind_edit_insert.to_string(),
                        default: def.bind_edit_insert.to_string(),
                        validation: CustomType::Char,
                    },
                },
                Setting {
                    name: "Visual Mode",
                    key: "bind_edit_visual",
                    kind: SettingType::Custom {
                        value: config.bind_edit_visual.to_string(),
                        default: def.bind_edit_visual.to_string(),
                        validation: CustomType::Char,
                    },
                },
                Setting {
                    name: "Fold",
                    key: "bind_edit_fold",
                    kind: SettingType::Custom {
                        value: config.bind_edit_fold.to_string(),
                        default: def.bind_edit_fold.to_string(),
                        validation: CustomType::Char,
                    },
                },
                Setting {
                    name: "Move Left",
                    key: "bind_edit_left",
                    kind: SettingType::Custom {
                        value: config.bind_edit_left.to_string(),
                        default: def.bind_edit_left.to_string(),
                        validation: CustomType::Char,
                    },
                },
                Setting {
                    name: "Move Right",
                    key: "bind_edit_right",
                    kind: SettingType::Custom {
                        value: config.bind_edit_right.to_string(),
                        default: def.bind_edit_right.to_string(),
                        validation: CustomType::Char,
                    },
                },
                Setting {
                    name: "Move Up",
                    key: "bind_edit_up",
                    kind: SettingType::Custom {
                        value: config.bind_edit_up.to_string(),
                        default: def.bind_edit_up.to_string(),
                        validation: CustomType::Char,
                    },
                },
                Setting {
                    name: "Move Down",
                    key: "bind_edit_down",
                    kind: SettingType::Custom {
                        value: config.bind_edit_down.to_string(),
                        default: def.bind_edit_down.to_string(),
                        validation: CustomType::Char,
                    },
                },
                Setting {
                    name: "Next Word",
                    key: "bind_edit_word_next",
                    kind: SettingType::Custom {
                        value: config.bind_edit_word_next.to_string(),
                        default: def.bind_edit_word_next.to_string(),
                        validation: CustomType::Char,
                    },
                },
                Setting {
                    name: "Prev Word",
                    key: "bind_edit_word_prev",
                    kind: SettingType::Custom {
                        value: config.bind_edit_word_prev.to_string(),
                        default: def.bind_edit_word_prev.to_string(),
                        validation: CustomType::Char,
                    },
                },
                Setting {
                    name: "Line Start",
                    key: "bind_edit_line_start",
                    kind: SettingType::Custom {
                        value: config.bind_edit_line_start.to_string(),
                        default: def.bind_edit_line_start.to_string(),
                        validation: CustomType::Char,
                    },
                },
                Setting {
                    name: "Line End",
                    key: "bind_edit_line_end",
                    kind: SettingType::Custom {
                        value: config.bind_edit_line_end.to_string(),
                        default: def.bind_edit_line_end.to_string(),
                        validation: CustomType::Char,
                    },
                },
                Setting {
                    name: "Select All",
                    key: "bind_edit_select_all",
                    kind: SettingType::Custom {
                        value: config.bind_edit_select_all.to_string(),
                        default: def.bind_edit_select_all.to_string(),
                        validation: CustomType::Char,
                    },
                },
                Setting {
                    name: "File Bounds",
                    key: "bind_edit_file_bounds",
                    kind: SettingType::Custom {
                        value: config.bind_edit_file_bounds.to_string(),
                        default: def.bind_edit_file_bounds.to_string(),
                        validation: CustomType::Char,
                    },
                },
                Setting {
                    name: "Delete",
                    key: "bind_edit_delete",
                    kind: SettingType::Custom {
                        value: config.bind_edit_delete.to_string(),
                        default: def.bind_edit_delete.to_string(),
                        validation: CustomType::Char,
                    },
                },
                Setting {
                    name: "Copy",
                    key: "bind_edit_copy",
                    kind: SettingType::Custom {
                        value: config.bind_edit_copy.to_string(),
                        default: def.bind_edit_copy.to_string(),
                        validation: CustomType::Char,
                    },
                },
                Setting {
                    name: "Paste",
                    key: "bind_edit_paste",
                    kind: SettingType::Custom {
                        value: config.bind_edit_paste.to_string(),
                        default: def.bind_edit_paste.to_string(),
                        validation: CustomType::Char,
                    },
                },
                Setting {
                    name: "Search",
                    key: "bind_edit_search",
                    kind: SettingType::Custom {
                        value: config.bind_edit_search.to_string(),
                        default: def.bind_edit_search.to_string(),
                        validation: CustomType::Char,
                    },
                },
                Setting {
                    name: "Undo",
                    key: "bind_edit_undo",
                    kind: SettingType::Custom {
                        value: config.bind_edit_undo.to_string(),
                        default: def.bind_edit_undo.to_string(),
                        validation: CustomType::Char,
                    },
                },
                Setting {
                    name: "Redo",
                    key: "bind_edit_redo",
                    kind: SettingType::Custom {
                        value: config.bind_edit_redo.to_string(),
                        default: def.bind_edit_redo.to_string(),
                        validation: CustomType::Char,
                    },
                },
                Setting {
                    name: "Save",
                    key: "bind_edit_save",
                    kind: SettingType::Custom {
                        value: config.bind_edit_save.to_string(),
                        default: def.bind_edit_save.to_string(),
                        validation: CustomType::Char,
                    },
                },
                Setting {
                    name: "Reset Keybinds",
                    key: "reset_edit_keybinds",
                    kind: SettingType::Action,
                },
            ],
        },
        Category {
            name: "Advanced",
            settings: vec![
                Setting {
                    name: "Reset Lib",
                    key: "reset_lib",
                    kind: SettingType::Action,
                },
                Setting {
                    name: "Reset Config",
                    key: "reset_config",
                    kind: SettingType::Action,
                },
            ],
        },
    ]
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
    let mut prev_sorting = config.lib_sorting.clone();
    let mut prev_tabs_num = config.tabs_num;
    let mut prev_lib_width = config.lib_width;
    let mut prev_deck_mode = config.deck_mode.clone();
    let mut prev_deck_widget = config.deck_widget.clone();
    let mut dirty = true;

    let version_str = format!(" v{} ", env!("CARGO_PKG_VERSION"));
    let version_len = version_str.chars().count() as u16;

    macro_rules! apply_setting {
        ($config:expr, $set:expr, char $key:expr, $field:ident) => {
            if $set.key == $key {
                if let SettingType::Custom { value, .. } = &$set.kind {
                    $config.$field = value.chars().next().unwrap_or($config.$field);
                }
            }
        };
        ($config:expr, $set:expr, text $key:expr, $field:ident) => {
            if $set.key == $key {
                if let SettingType::Custom { value, .. } = &$set.kind {
                    $config.$field = value.clone();
                }
            }
        };
        ($config:expr, $set:expr, parse $key:expr, $field:ident) => {
            if $set.key == $key {
                if let SettingType::Custom { value, .. } = &$set.kind {
                    if let Ok(v) = value.parse() {
                        $config.$field = v;
                    }
                }
            }
        };
        ($config:expr, $set:expr, parse_clamp $key:expr, $field:ident, $min:expr, $max:expr) => {
            if $set.key == $key {
                if let SettingType::Custom { value, .. } = &$set.kind {
                    if let Ok(v) = value.parse() {
                        $config.$field = v;
                        $config.$field = $config.$field.clamp($min, $max);
                    }
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
        ($config:expr, $set:expr, choice_offset $key:expr, $field:ident, $offset:expr) => {
            if $set.key == $key {
                if let SettingType::Choice(_, idx) = &$set.kind {
                    $config.$field = *idx + $offset;
                }
            }
        };
        ($config:expr, $set:expr, bool $key:expr, $field:ident) => {
            if $set.key == $key {
                if let SettingType::Choice(opts, idx) = &$set.kind {
                    $config.$field = opts[*idx] == "true";
                }
            }
        };
    }

    let apply_settings = |categories: &[Category], config: &mut Config| {
        for cat in categories {
            for set in &cat.settings {
                apply_setting!(config, set, choice "theme", theme);
                apply_setting!(config, set, choice "border_style", border_style);
                apply_setting!(config, set, choice "indicator_style", indicator_style);
                apply_setting!(config, set, choice "lib_sorting", lib_sorting);
                apply_setting!(config, set, choice_offset "tabs_num", tabs_num, 1);
                apply_setting!(config, set, parse_clamp "lib_width", lib_width, 16, 64);
                apply_setting!(config, set, choice "deck_mode", deck_mode);
                apply_setting!(config, set, choice "deck_widget", deck_widget);

                apply_setting!(config, set, choice "clock_date", clock_date);
                apply_setting!(config, set, choice "clock_mode", clock_mode);
                apply_setting!(config, set, choice "clock_position", clock_position);
                apply_setting!(config, set, choice "clock_format", clock_format);
                apply_setting!(config, set, bool "clock_seconds", clock_seconds);

                apply_setting!(config, set, parse "keyvis_width", keyvis_width);
                apply_setting!(config, set, parse_clamp "keyvis_height", keyvis_height, 2, 32);
                apply_setting!(config, set, choice_offset "keyvis_steps", keyvis_steps, 1);
                apply_setting!(config, set, parse_clamp "keyvis_spread", keyvis_spread, 2, 32);
                apply_setting!(config, set, parse_clamp "keyvis_force", keyvis_force, 0.1, 1.0);
                apply_setting!(config, set, parse_clamp "keyvis_gravity", keyvis_gravity, 0.1, 1.0);
                apply_setting!(config, set, parse_clamp "keyvis_tension", keyvis_tension, 0.1, 1.0);
                apply_setting!(config, set, bool "keyvis_base", keyvis_base);

                apply_setting!(config, set, choice "monitor_cpu", monitor_cpu);
                apply_setting!(config, set, choice "monitor_gpu", monitor_gpu);
                apply_setting!(config, set, choice "monitor_mem", monitor_mem);
                apply_setting!(config, set, choice "monitor_term", monitor_term);
                apply_setting!(config, set, choice "monitor_divider", monitor_divider);
                apply_setting!(config, set, choice "monitor_bar_mode", monitor_bar_mode);
                apply_setting!(config, set, parse_clamp "monitor_bar_width", monitor_bar_width, 4, 16);
                apply_setting!(config, set, bool "monitor_icons", monitor_icons);

                apply_setting!(config, set, char "bind_edit_insert", bind_edit_insert);
                apply_setting!(config, set, char "bind_edit_visual", bind_edit_visual);
                apply_setting!(config, set, char "bind_edit_fold", bind_edit_fold);
                apply_setting!(config, set, char "bind_edit_left", bind_edit_left);
                apply_setting!(config, set, char "bind_edit_right", bind_edit_right);
                apply_setting!(config, set, char "bind_edit_up", bind_edit_up);
                apply_setting!(config, set, char "bind_edit_down", bind_edit_down);
                apply_setting!(config, set, char "bind_edit_word_next", bind_edit_word_next);
                apply_setting!(config, set, char "bind_edit_word_prev", bind_edit_word_prev);
                apply_setting!(config, set, char "bind_edit_line_start", bind_edit_line_start);
                apply_setting!(config, set, char "bind_edit_line_end", bind_edit_line_end);
                apply_setting!(config, set, char "bind_edit_select_all", bind_edit_select_all);
                apply_setting!(config, set, char "bind_edit_file_bounds", bind_edit_file_bounds);
                apply_setting!(config, set, char "bind_edit_delete", bind_edit_delete);
                apply_setting!(config, set, char "bind_edit_copy", bind_edit_copy);
                apply_setting!(config, set, char "bind_edit_paste", bind_edit_paste);
                apply_setting!(config, set, char "bind_edit_search", bind_edit_search);
                apply_setting!(config, set, char "bind_edit_undo", bind_edit_undo);
                apply_setting!(config, set, char "bind_edit_redo", bind_edit_redo);
                apply_setting!(config, set, char "bind_edit_save", bind_edit_save);

                apply_setting!(config, set, char "bind_lib_new_file", bind_lib_new_file);
                apply_setting!(config, set, char "bind_lib_new_folder", bind_lib_new_folder);
                apply_setting!(config, set, char "bind_lib_rename", bind_lib_rename);
                apply_setting!(config, set, char "bind_lib_delete", bind_lib_delete);
                apply_setting!(config, set, char "bind_lib_move_up", bind_lib_move_up);
                apply_setting!(config, set, char "bind_lib_move_down", bind_lib_move_down);
            }
        }
    };

    loop {
        let (current_w, current_h) = Terminal::size();

        if current_w != canvas.width || current_h != canvas.height {
            canvas.resize(current_w, current_h);
            dirty = true;
        }

        let mut bg_dirty = false;
        if config.deck_mode == "widget" {
            if config.deck_widget == "keyvis" {
                if main_view.keyvis.tick(
                    config.keyvis_gravity,
                    config.keyvis_steps,
                    config.keyvis_tension,
                ) {
                    main_view.refresh_static_boxes(config);
                    bg_dirty = true;
                }
            } else if config.deck_widget == "monitor" {
                if main_view.monitor.tick(current_w, current_h) {
                    main_view.refresh_static_boxes(config);
                    bg_dirty = true;
                }
            } else if config.deck_widget == "clock" {
                if main_view.clock.tick(current_w, current_h, config) {
                    main_view.refresh_static_boxes(config);
                    bg_dirty = true;
                }
            }
        }

        if dirty || bg_dirty {
            let current_min_w = main_view.min_w;
            let current_min_h = main_view.min_h;

            if current_w < current_min_w || current_h < current_min_h {
                if !crate::toosmall::run(
                    terminal,
                    canvas,
                    current_min_w,
                    current_min_h,
                    config.get_border(),
                    main_view.theme.warning_color.clone(),
                ) {
                    return true;
                }
                dirty = true;
                continue;
            }

            canvas.clean();

            if current_w != main_view.term_w || current_h != main_view.term_h {
                main_view.resize(current_w, current_h, config);
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

            let use_border_color = config.indicator_style == "border";
            let cat_active = active_panel == ActiveSettingsPanel::Categories;

            let mut cat_box = Box::new(
                left_w,
                modal_h,
                1,
                config.get_border(),
                if cat_active && use_border_color {
                    main_view.theme.selected_box.clone()
                } else {
                    main_view.theme.settings_category_box.clone()
                },
                Gradient::Solid(Color::None),
                Modifier::None,
            );

            crate::panels::apply_indicator(&mut cat_box, config, &main_view.theme, cat_active);

            cat_box.insert_text(
                " Categories ",
                1,
                -1,
                false,
                main_view.theme.main_label.clone(),
                Gradient::Solid(Color::None),
                Modifier::Bold,
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

                let (fg_color, bg_color) = if is_selected && cat_active {
                    (
                        Gradient::Solid(Color::Black),
                        main_view.theme.settings_selected.clone(),
                    )
                } else if is_selected {
                    (
                        main_view.theme.settings_selected.clone(),
                        Gradient::Solid(Color::None),
                    )
                } else {
                    (
                        main_view.theme.settings_entry.clone(),
                        Gradient::Solid(Color::None),
                    )
                };

                let mut text = cat.name.to_string();
                let char_count = text.chars().count();
                if char_count > max_cat_len {
                    text = text.chars().take(max_cat_len).collect();
                } else {
                    text.push_str(&" ".repeat(max_cat_len.saturating_sub(char_count + 2)));
                }

                let display_y = (i - cat_scroll) as i16;
                cat_box.insert_text(
                    &text,
                    1,
                    display_y,
                    false,
                    fg_color,
                    bg_color,
                    Modifier::None,
                );
            }

            let det_active = active_panel == ActiveSettingsPanel::Details;

            let mut det_box = Box::new(
                right_w,
                modal_h,
                1,
                config.get_border(),
                if det_active && use_border_color {
                    main_view.theme.selected_box.clone()
                } else {
                    main_view.theme.settings_options_box.clone()
                },
                Gradient::Solid(Color::None),
                Modifier::None,
            );

            crate::panels::apply_indicator(&mut det_box, config, &main_view.theme, det_active);

            det_box.insert_text(
                " Settings ",
                1,
                -1,
                false,
                main_view.theme.main_label.clone(),
                Gradient::Solid(Color::None),
                Modifier::Bold,
            );

            let current_cat = &categories[cat_selected];
            let current_settings = &current_cat.settings;
            let current_det_sel = det_selected[cat_selected];
            let mut current_det_scroll = det_scroll[cat_selected];

            if current_det_sel < current_det_scroll {
                current_det_scroll = current_det_sel;
            } else if current_det_sel >= current_det_scroll + visible_items && visible_items > 0 {
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
                    (
                        Gradient::Solid(Color::Black),
                        main_view.theme.settings_selected.clone(),
                    )
                } else if is_selected {
                    (
                        main_view.theme.settings_selected.clone(),
                        Gradient::Solid(Color::None),
                    )
                } else {
                    (
                        main_view.theme.settings_entry.clone(),
                        Gradient::Solid(Color::None),
                    )
                };

                if is_action {
                    if is_selected && det_active {
                        fg_color = Gradient::Solid(Color::Black);
                        bg_color = main_view.theme.settings_special.clone();
                    } else if is_selected {
                        fg_color = main_view.theme.settings_special.clone();
                    } else {
                        fg_color = main_view.theme.settings_entry.clone();
                    }
                }

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
                det_box.insert_text(
                    &text,
                    1,
                    display_y,
                    false,
                    fg_color,
                    bg_color,
                    Modifier::None,
                );
            }

            let v_y = det_box.height.saturating_sub(1);
            let v_start_x = det_box.width.saturating_sub(3 + version_len);

            for (i, c) in version_str.chars().enumerate() {
                let offset = i as u16;
                if v_start_x + offset < det_box.width {
                    det_box.put_cell(
                        Cell {
                            c,
                            s: Style {
                                fg: main_view
                                    .theme
                                    .settings_options_box
                                    .color_at(i, version_len as usize),
                                bg: Color::None,
                                md: Modifier::None,
                            },
                        },
                        v_start_x + offset,
                        v_y,
                    );
                }
            }

            canvas.put_box_opaque(&cat_box, start_x, start_y);
            canvas.put_box_opaque(&det_box, start_x + (left_w as i16), start_y);
            canvas.render();
            dirty = false;
        }

        let mut needs_ui_refresh = false;

        let key = terminal.read_key(Duration::from_millis(16));
        if key != Key::None && config.deck_mode == "widget" && config.deck_widget == "keyvis" {
            main_view
                .keyvis
                .push_key(&key, config.keyvis_force, config.keyvis_spread);
        }

        if key == Key::None {
            continue;
        }

        if edit_mode {
            match key {
                Key::Enter => {
                    let cat = &mut categories[cat_selected];
                    let setting = &mut cat.settings[det_selected[cat_selected]];

                    let mut do_apply = false;

                    if let SettingType::Custom {
                        value,
                        default,
                        validation,
                    } = &mut setting.kind
                    {
                        if edit_buffer.is_empty() {
                            *value = default.clone();
                            do_apply = true;
                            edit_mode = false;
                        } else {
                            match validation {
                                CustomType::Text => do_apply = true,
                                CustomType::Char => {
                                    if edit_buffer.chars().count() == 1 {
                                        do_apply = true;
                                    }
                                }
                                CustomType::Int => {
                                    if edit_buffer.parse::<usize>().is_ok() {
                                        do_apply = true;
                                    }
                                }
                                CustomType::FloatRange(min, max) => {
                                    if let Ok(v) = edit_buffer.parse::<f32>() {
                                        if v >= *min && v <= *max {
                                            do_apply = true;
                                        }
                                    }
                                }
                                CustomType::Gradient => {
                                    if crate::theme::themecore::parse_gradient(&edit_buffer).is_ok()
                                    {
                                        do_apply = true;
                                    }
                                }
                            }

                            if do_apply {
                                *value = edit_buffer.clone();
                                edit_mode = false;
                            }
                        }
                    }

                    if do_apply {
                        apply_settings(&categories, config);

                        let current_theme_idx =
                            themes.iter().position(|t| t == &config.theme).unwrap_or(0);
                        categories = build_categories(config, &themes, current_theme_idx);

                        needs_ui_refresh = true;
                    }
                    dirty = true;
                }
                Key::Esc => {
                    edit_mode = false;
                    dirty = true;
                }
                Key::Backspace => {
                    edit_buffer.pop();
                    dirty = true;
                }
                Key::Char('\x03') => {
                    config.save();
                    return true;
                }
                Key::Char(c) => {
                    if !c.is_control() {
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
                            edit_buffer = c.to_ascii_lowercase().to_string();
                        } else {
                            edit_buffer.push(c);
                        }
                        dirty = true;
                    }
                }
                Key::Shift(c) => {
                    if !c.is_control() {
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
                        let final_c = if c.is_ascii_lowercase() {
                            c.to_ascii_uppercase()
                        } else {
                            c
                        };
                        if is_char_type {
                            edit_buffer = final_c.to_ascii_lowercase().to_string();
                        } else {
                            edit_buffer.push(final_c);
                        }
                        dirty = true;
                    }
                }
                _ => {}
            }
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
                            if let SettingType::Choice(opts, idx) = &mut setting.kind {
                                *idx = if *idx > 0 { *idx - 1 } else { opts.len() - 1 };
                                apply_settings(&categories, config);
                                let current_theme_idx =
                                    themes.iter().position(|t| t == &config.theme).unwrap_or(0);
                                categories = build_categories(config, &themes, current_theme_idx);
                                needs_ui_refresh = true;
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
                            if let SettingType::Choice(opts, idx) = &mut setting.kind {
                                *idx = (*idx + 1) % opts.len();
                                apply_settings(&categories, config);
                                let current_theme_idx =
                                    themes.iter().position(|t| t == &config.theme).unwrap_or(0);
                                categories = build_categories(config, &themes, current_theme_idx);
                                needs_ui_refresh = true;
                            }
                        }
                    }
                }
                Key::Up => {
                    if active_panel == ActiveSettingsPanel::Categories && cat_selected > 0 {
                        cat_selected -= 1;
                    } else if active_panel == ActiveSettingsPanel::Details
                        && det_selected[cat_selected] > 0
                    {
                        det_selected[cat_selected] -= 1;
                    }
                }
                Key::Down => {
                    if active_panel == ActiveSettingsPanel::Categories
                        && cat_selected < categories.len().saturating_sub(1)
                    {
                        cat_selected += 1;
                    } else if active_panel == ActiveSettingsPanel::Details {
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
                            match &setting.kind {
                                SettingType::Custom { value, .. } => {
                                    edit_mode = true;
                                    edit_buffer = value.clone();
                                }
                                SettingType::Action => {
                                    let (msg, action_type) = match setting.key {
                                        "reset_config" => {
                                            ("Reset all settings to default\n\nAre you sure?", 0)
                                        }
                                        "reset_edit_keybinds" => (
                                            "Reset all editor keybinds to default\n\nAre you sure?",
                                            1,
                                        ),
                                        "reset_appearance" => (
                                            "Reset appearance settings to default\n\nAre you sure?",
                                            2,
                                        ),
                                        "reset_lib_keybinds" => (
                                            "Reset library keybinds to default\n\nAre you sure?",
                                            3,
                                        ),
                                        "reset_order" => {
                                            ("Reset custom sorting order\n\nAre you sure?", 4)
                                        }
                                        "reset_lib" => (
                                            "Delete all files in the library\nThis cannot be undone!\n\nAre you sure?",
                                            5,
                                        ),
                                        "reset_deck" => {
                                            ("Reset deck settings to default\n\nAre you sure?", 6)
                                        }
                                        _ => ("", 99),
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
                                        config.get_border(),
                                        main_view.theme.warning_color.clone(),
                                        |cvs, w, h, k| {
                                            main_view.draw_background(cvs, w, h, k, config)
                                        },
                                    );

                                    if result == crate::PanelResult::Ok(1) {
                                        if action_type == 0 {
                                            *config = Config::default();
                                        } else if action_type == 1 {
                                            config.reset_edit_keybinds();
                                        } else if action_type == 2 {
                                            config.reset_appearance();
                                        } else if action_type == 3 {
                                            config.reset_lib_keybinds();
                                        } else if action_type == 4 {
                                            crate::lib::reset_custom_order();
                                        } else if action_type == 5 {
                                            crate::lib::reset_library();
                                        } else if action_type == 6 {
                                            config.reset_deck();
                                        }
                                        config.save();

                                        if action_type == 4 || action_type == 5 || action_type == 1
                                        {
                                            if let Ok(l) = crate::lib::init(&config.lib_sorting) {
                                                main_view.library_tree = l.tree;
                                                main_view.library_root = l.root_path;
                                                if action_type == 5 || action_type == 1 {
                                                    main_view.expanded_path.clear();
                                                    main_view.list_selected = 0;
                                                    main_view.list_scroll = 0;
                                                    for i in 0..6 {
                                                        if let Some(token) =
                                                            main_view.cancellation_tokens[i].take()
                                                        {
                                                            token.store(
                                                                true,
                                                                std::sync::atomic::Ordering::SeqCst,
                                                            );
                                                        }
                                                        main_view.editors[i].file_path = None;
                                                        main_view.editors[i].rel_path.clear();
                                                        main_view.editors[i].state.lines =
                                                            vec![String::new()];
                                                        main_view.editors[i].process_rx = None;
                                                        main_view.running_macros[i] = None;
                                                        main_view.editors[i].error_count = 0;
                                                        main_view.editors[i].error_lines.clear();
                                                    }
                                                }
                                                main_view.auto_load();
                                            }
                                        }

                                        let current_theme_idx = themes
                                            .iter()
                                            .position(|t| t == &config.theme)
                                            .unwrap_or(0);
                                        categories =
                                            build_categories(config, &themes, current_theme_idx);
                                        needs_ui_refresh = true;
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

        if needs_ui_refresh {
            for (i, cat) in categories.iter().enumerate() {
                if det_selected[i] >= cat.settings.len() {
                    det_selected[i] = cat.settings.len().saturating_sub(1);
                }
                if det_scroll[i] >= cat.settings.len() {
                    det_scroll[i] = cat.settings.len().saturating_sub(1);
                }
            }

            if config.theme != prev_theme {
                match crate::theme::themecore::init(&config.theme) {
                    Ok(new_theme) => {
                        main_view.theme = new_theme;
                        main_view.update_min_h(config);

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
                            config.get_border(),
                            main_view.theme.warning_color.clone(),
                            |cvs, w, h, k| main_view.draw_background(cvs, w, h, k, config),
                        );
                        config.theme = prev_theme.clone();
                        let current_theme_idx =
                            themes.iter().position(|t| t == &config.theme).unwrap_or(0);
                        categories = build_categories(config, &themes, current_theme_idx);
                    }
                }
            }
            if config.lib_sorting != prev_sorting {
                if let Ok(l) = crate::lib::init(&config.lib_sorting) {
                    main_view.library_tree = l.tree;
                    main_view.library_root = l.root_path;
                    main_view.auto_load();
                }
                prev_sorting = config.lib_sorting.clone();
            }
            if config.tabs_num != prev_tabs_num || config.lib_width != prev_lib_width {
                if config.tabs_num != prev_tabs_num {
                    for i in config.tabs_num..6 {
                        if let Some(token) = main_view.cancellation_tokens[i].take() {
                            token.store(true, std::sync::atomic::Ordering::SeqCst);
                        }
                        main_view.editors[i].process_rx = None;
                        main_view.running_macros[i] = None;
                        main_view.editors[i].file_path = None;
                        main_view.editors[i].rel_path.clear();
                        main_view.editors[i].state.lines = vec![String::new()];
                        main_view.editors[i].is_editing = false;
                        main_view.editors[i].error_count = 0;
                        main_view.editors[i].error_lines.clear();
                    }
                }
                main_view.current_tab =
                    main_view.current_tab.min(config.tabs_num.saturating_sub(1));
                main_view.auto_load();

                let (term_w, term_h) = Terminal::size();
                main_view.resize(term_w, term_h, config);

                prev_tabs_num = config.tabs_num;
                prev_lib_width = config.lib_width;
            }

            if config.deck_mode != prev_deck_mode || config.deck_widget != prev_deck_widget {
                let (term_w, term_h) = Terminal::size();
                main_view.resize(term_w, term_h, config);
                prev_deck_mode = config.deck_mode.clone();
                prev_deck_widget = config.deck_widget.clone();
            } else {
                main_view.update_min_h(config);
                let (term_w, term_h) = Terminal::size();
                main_view.resize(term_w, term_h, config);
            }

            dirty = true;
        }
    }
}
