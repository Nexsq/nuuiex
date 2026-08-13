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
    IntRange(usize, usize),
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

    if let Ok(config_dir) = crate::get_config_dir() {
        let themes_dir = config_dir.join("themes");
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

    let mut categories = Vec::new();

    categories.push(Category {
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
                name: "Show Caret",
                key: "show_caret",
                kind: SettingType::Choice(
                    vec!["true".to_string(), "false".to_string()],
                    if config.show_caret { 0 } else { 1 },
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
                name: "Reset Appearance",
                key: "reset_appearance",
                kind: SettingType::Action,
            },
        ],
    });

    categories.push(Category {
        name: "Settings",
        settings: vec![
            Setting {
                name: "Settings Style",
                key: "settings_style",
                kind: SettingType::Choice(
                    vec![
                        "inline".to_string(),
                        "aligned".to_string(),
                        "right".to_string(),
                    ],
                    match config.settings_style.as_str() {
                        "aligned" => 1,
                        "right" => 2,
                        _ => 0,
                    },
                ),
            },
            Setting {
                name: "Choice Indicator",
                key: "setting_indicator_choice",
                kind: SettingType::Choice(
                    vec![
                        "modern".to_string(),
                        "classic".to_string(),
                        "none".to_string(),
                    ],
                    match config.setting_indicator_choice.as_str() {
                        "classic" => 1,
                        "none" => 2,
                        _ => 0,
                    },
                ),
            },
            Setting {
                name: "Custom Indicator",
                key: "setting_indicator_custom",
                kind: SettingType::Choice(
                    vec![
                        "modern".to_string(),
                        "classic".to_string(),
                        "none".to_string(),
                    ],
                    match config.setting_indicator_custom.as_str() {
                        "classic" => 1,
                        "none" => 2,
                        _ => 0,
                    },
                ),
            },
            Setting {
                name: "Fancy Bools",
                key: "fancy_bools",
                kind: SettingType::Choice(
                    vec!["true".to_string(), "false".to_string()],
                    if config.fancy_bools { 0 } else { 1 },
                ),
            },
            Setting {
                name: "Reset Settings Menu",
                key: "reset_settings_menu",
                kind: SettingType::Action,
            },
        ],
    });

    categories.push(Category {
        name: "Deck",
        settings: vec![
            Setting {
                name: "Deck",
                key: "deck",
                kind: SettingType::Choice(
                    vec!["true".to_string(), "false".to_string()],
                    if config.deck { 0 } else { 1 },
                ),
            },
            Setting {
                name: "Deck Mode",
                key: "deck_mode",
                kind: SettingType::Choice(
                    vec![
                        "title".to_string(),
                        "keyvis".to_string(),
                        "monitor".to_string(),
                        "clock".to_string(),
                        "macrostats".to_string(),
                        "matrix".to_string(),
                    ],
                    match config.deck_mode.as_str() {
                        "keyvis" => 1,
                        "monitor" => 2,
                        "clock" => 3,
                        "macrostats" => 4,
                        "matrix" => 5,
                        _ => 0,
                    },
                ),
            },
            Setting {
                name: "Reset Deck",
                key: "reset_deck",
                kind: SettingType::Action,
            },
        ],
    });

    categories.push(Category {
        name: "Keyvis",
        settings: vec![
            Setting {
                name: "Width",
                key: "keyvis_width",
                kind: SettingType::Custom {
                    value: config.keyvis_width.to_string(),
                    default: def.keyvis_width.to_string(),
                    validation: CustomType::IntRange(1, 1024),
                },
            },
            Setting {
                name: "Height",
                key: "keyvis_height",
                kind: SettingType::Custom {
                    value: config.keyvis_height.to_string(),
                    default: def.keyvis_height.to_string(),
                    validation: CustomType::IntRange(2, 32),
                },
            },
            Setting {
                name: "Steps",
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
            },
            Setting {
                name: "Spread",
                key: "keyvis_spread",
                kind: SettingType::Custom {
                    value: config.keyvis_spread.to_string(),
                    default: def.keyvis_spread.to_string(),
                    validation: CustomType::IntRange(2, 32),
                },
            },
            Setting {
                name: "Force",
                key: "keyvis_force",
                kind: SettingType::Custom {
                    value: config.keyvis_force.to_string(),
                    default: def.keyvis_force.to_string(),
                    validation: CustomType::FloatRange(0.1, 1.0),
                },
            },
            Setting {
                name: "Gravity",
                key: "keyvis_gravity",
                kind: SettingType::Custom {
                    value: config.keyvis_gravity.to_string(),
                    default: def.keyvis_gravity.to_string(),
                    validation: CustomType::FloatRange(0.1, 1.0),
                },
            },
            Setting {
                name: "Tension",
                key: "keyvis_tension",
                kind: SettingType::Custom {
                    value: config.keyvis_tension.to_string(),
                    default: def.keyvis_tension.to_string(),
                    validation: CustomType::FloatRange(0.1, 1.0),
                },
            },
            Setting {
                name: "Base",
                key: "keyvis_base",
                kind: SettingType::Choice(
                    vec!["true".to_string(), "false".to_string()],
                    if config.keyvis_base { 0 } else { 1 },
                ),
            },
            Setting {
                name: "Reset Keyvis",
                key: "reset_keyvis",
                kind: SettingType::Action,
            },
        ],
    });

    categories.push(Category {
        name: "Matrix",
        settings: vec![
            Setting {
                name: "Height",
                key: "matrix_height",
                kind: SettingType::Custom {
                    value: config.matrix_height.to_string(),
                    default: def.matrix_height.to_string(),
                    validation: CustomType::IntRange(2, 32),
                },
            },
            Setting {
                name: "Density",
                key: "matrix_density",
                kind: SettingType::Custom {
                    value: config.matrix_density.to_string(),
                    default: def.matrix_density.to_string(),
                    validation: CustomType::IntRange(1, 200),
                },
            },
            Setting {
                name: "Dim Ratio",
                key: "matrix_dim_ratio",
                kind: SettingType::Custom {
                    value: config.matrix_dim_ratio.to_string(),
                    default: def.matrix_dim_ratio.to_string(),
                    validation: CustomType::IntRange(0, 100),
                },
            },
            Setting {
                name: "Speed",
                key: "matrix_speed",
                kind: SettingType::Custom {
                    value: config.matrix_speed.to_string(),
                    default: def.matrix_speed.to_string(),
                    validation: CustomType::FloatRange(0.1, 5.0),
                },
            },
            Setting {
                name: "Direction",
                key: "matrix_direction",
                kind: SettingType::Choice(
                    vec![
                        "down".to_string(),
                        "up".to_string(),
                        "left".to_string(),
                        "right".to_string(),
                    ],
                    match config.matrix_direction.as_str() {
                        "up" => 1,
                        "left" => 2,
                        "right" => 3,
                        _ => 0,
                    },
                ),
            },
            Setting {
                name: "Min Length",
                key: "matrix_min_length",
                kind: SettingType::Custom {
                    value: config.matrix_min_length.to_string(),
                    default: def.matrix_min_length.to_string(),
                    validation: CustomType::IntRange(2, 64),
                },
            },
            Setting {
                name: "Max Length",
                key: "matrix_max_length",
                kind: SettingType::Custom {
                    value: config.matrix_max_length.to_string(),
                    default: def.matrix_max_length.to_string(),
                    validation: CustomType::IntRange(2, 64),
                },
            },
            Setting {
                name: "Reset Matrix",
                key: "reset_matrix",
                kind: SettingType::Action,
            },
        ],
    });

    categories.push(Category {
        name: "Monitor",
        settings: vec![
            Setting {
                name: "CPU",
                key: "monitor_cpu",
                kind: SettingType::Choice(
                    vec!["true".to_string(), "false".to_string()],
                    if config.monitor_cpu { 0 } else { 1 },
                ),
            },
            Setting {
                name: "GPU",
                key: "monitor_gpu",
                kind: SettingType::Choice(
                    vec!["true".to_string(), "false".to_string()],
                    if config.monitor_gpu { 0 } else { 1 },
                ),
            },
            Setting {
                name: "MEM",
                key: "monitor_mem",
                kind: SettingType::Choice(
                    vec!["true".to_string(), "false".to_string()],
                    if config.monitor_mem { 0 } else { 1 },
                ),
            },
            Setting {
                name: "CPU Style",
                key: "monitor_cpu_style",
                kind: SettingType::Choice(
                    vec![
                        "pct".to_string(),
                        "bar".to_string(),
                        "graph".to_string(),
                        "pctbar".to_string(),
                        "pctgraph".to_string(),
                    ],
                    match config.monitor_cpu_style.as_str() {
                        "bar" => 1,
                        "graph" => 2,
                        "pctbar" => 3,
                        "pctgraph" => 4,
                        _ => 0,
                    },
                ),
            },
            Setting {
                name: "GPU Style",
                key: "monitor_gpu_style",
                kind: SettingType::Choice(
                    vec![
                        "pct".to_string(),
                        "bar".to_string(),
                        "graph".to_string(),
                        "pctbar".to_string(),
                        "pctgraph".to_string(),
                    ],
                    match config.monitor_gpu_style.as_str() {
                        "bar" => 1,
                        "graph" => 2,
                        "pctbar" => 3,
                        "pctgraph" => 4,
                        _ => 0,
                    },
                ),
            },
            Setting {
                name: "MEM Style",
                key: "monitor_mem_style",
                kind: SettingType::Choice(
                    vec![
                        "pct".to_string(),
                        "used".to_string(),
                        "bar".to_string(),
                        "graph".to_string(),
                        "pctbar".to_string(),
                        "pctgraph".to_string(),
                    ],
                    match config.monitor_mem_style.as_str() {
                        "used" => 1,
                        "bar" => 2,
                        "graph" => 3,
                        "pctbar" => 4,
                        "pctgraph" => 5,
                        _ => 0,
                    },
                ),
            },
            Setting {
                name: "TERM",
                key: "monitor_term",
                kind: SettingType::Choice(
                    vec!["true".to_string(), "false".to_string()],
                    if config.monitor_term { 0 } else { 1 },
                ),
            },
            Setting {
                name: "Bar",
                key: "monitor_bar",
                kind: SettingType::Choice(
                    vec!["true".to_string(), "false".to_string()],
                    if config.monitor_bar { 0 } else { 1 },
                ),
            },
            Setting {
                name: "Bar Style",
                key: "monitor_bar_style",
                kind: SettingType::Choice(
                    vec!["background".to_string(), "caps".to_string()],
                    match config.monitor_bar_style.as_str() {
                        "caps" => 1,
                        _ => 0,
                    },
                ),
            },
            Setting {
                name: "Bar Width",
                key: "monitor_bar_width",
                kind: SettingType::Custom {
                    value: config.monitor_bar_width.to_string(),
                    default: def.monitor_bar_width.to_string(),
                    validation: CustomType::IntRange(4, 16),
                },
            },
            Setting {
                name: "Divider",
                key: "monitor_divider",
                kind: SettingType::Choice(
                    vec!["true".to_string(), "false".to_string()],
                    if config.monitor_divider { 0 } else { 1 },
                ),
            },
            Setting {
                name: "Icons",
                key: "monitor_icons",
                kind: SettingType::Choice(
                    vec!["true".to_string(), "false".to_string()],
                    if config.monitor_icons { 0 } else { 1 },
                ),
            },
            Setting {
                name: "Reset Monitor",
                key: "reset_monitor",
                kind: SettingType::Action,
            },
        ],
    });

    categories.push(Category {
        name: "Clock",
        settings: vec![
            Setting {
                name: "Date",
                key: "clock_date",
                kind: SettingType::Choice(
                    vec!["true".to_string(), "false".to_string()],
                    if config.clock_date { 0 } else { 1 },
                ),
            },
            Setting {
                name: "Date Style",
                key: "clock_date_style",
                kind: SettingType::Choice(
                    vec![
                        "eu".to_string(),
                        "us".to_string(),
                        "clean".to_string(),
                        "mon name".to_string(),
                        "rfc 2822".to_string(),
                    ],
                    match config.clock_date_style.as_str() {
                        "us" => 1,
                        "clean" => 2,
                        "mon name" => 3,
                        "rfc 2822" => 4,
                        _ => 0,
                    },
                ),
            },
            Setting {
                name: "Mode",
                key: "clock_mode",
                kind: SettingType::Choice(
                    vec!["small".to_string(), "big".to_string()],
                    match config.clock_mode.as_str() {
                        "big" => 1,
                        _ => 0,
                    },
                ),
            },
            Setting {
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
            },
            Setting {
                name: "Format",
                key: "clock_format",
                kind: SettingType::Choice(
                    vec!["12h".to_string(), "24h".to_string()],
                    match config.clock_format.as_str() {
                        "12h" => 0,
                        _ => 1,
                    },
                ),
            },
            Setting {
                name: "Show Seconds",
                key: "clock_seconds",
                kind: SettingType::Choice(
                    vec!["true".to_string(), "false".to_string()],
                    if config.clock_seconds { 0 } else { 1 },
                ),
            },
            Setting {
                name: "Reset Clock",
                key: "reset_clock",
                kind: SettingType::Action,
            },
        ],
    });

    categories.push(Category {
        name: "Macrostats",
        settings: {
            let mut s = Vec::new();
            macro_rules! add_bool {
                ($name:expr, $key:expr, $val:expr) => {
                    s.push(Setting {
                        name: $name,
                        key: $key,
                        kind: SettingType::Choice(
                            vec!["true".to_string(), "false".to_string()],
                            if $val { 0 } else { 1 },
                        ),
                    });
                };
            }
            add_bool!("Icons", "macrostats_icons", config.macrostats_icons);
            add_bool!(
                "Edit Show Name",
                "macrostats_edit_name",
                config.macrostats_edit_name
            );
            add_bool!(
                "Edit Errors",
                "macrostats_edit_err",
                config.macrostats_edit_err
            );
            s.push(Setting {
                name: "Edit Errors Style",
                key: "macrostats_edit_err_style",
                kind: SettingType::Choice(
                    vec!["text".to_string(), "chart".to_string(), "both".to_string()],
                    match config.macrostats_edit_err_style.as_str() {
                        "chart" => 1,
                        "both" => 2,
                        _ => 0,
                    },
                ),
            });
            add_bool!(
                "Edit Show Created",
                "macrostats_edit_created",
                config.macrostats_edit_created
            );
            add_bool!(
                "Edit Show Lines",
                "macrostats_edit_lines",
                config.macrostats_edit_lines
            );
            add_bool!(
                "Edit Show Code",
                "macrostats_edit_code",
                config.macrostats_edit_code
            );
            add_bool!(
                "Run Show Name",
                "macrostats_run_name",
                config.macrostats_run_name
            );
            add_bool!(
                "Run Show Elapsed",
                "macrostats_run_elapsed",
                config.macrostats_run_elapsed
            );
            add_bool!(
                "Run Show CPU",
                "macrostats_run_cpu",
                config.macrostats_run_cpu
            );
            add_bool!(
                "Lib Show Name",
                "macrostats_lib_name",
                config.macrostats_lib_name
            );
            add_bool!(
                "Lib Show Created",
                "macrostats_lib_created",
                config.macrostats_lib_created
            );
            add_bool!(
                "Lib Show Size",
                "macrostats_lib_size",
                config.macrostats_lib_size
            );
            add_bool!(
                "Lib Show Status",
                "macrostats_lib_status",
                config.macrostats_lib_status
            );
            s.push(Setting {
                name: "Err Chart Length",
                key: "macrostats_err_chart_len",
                kind: SettingType::Custom {
                    value: config.macrostats_err_chart_len.to_string(),
                    default: def.macrostats_err_chart_len.to_string(),
                    validation: CustomType::IntRange(4, 16),
                },
            });
            add_bool!(
                "Err Chart Numbers",
                "macrostats_err_chart_num",
                config.macrostats_err_chart_num
            );
            s.push(Setting {
                name: "Reset Macrostats",
                key: "reset_macrostats",
                kind: SettingType::Action,
            });
            s
        },
    });

    categories.push(Category {
        name: "Library",
        settings: vec![
            Setting {
                name: "Lib Width",
                key: "lib_width",
                kind: SettingType::Custom {
                    value: config.lib_width.to_string(),
                    default: def.lib_width.to_string(),
                    validation: CustomType::IntRange(16, 64),
                },
            },
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
                name: "Reset Library",
                key: "reset_library",
                kind: SettingType::Action,
            },
        ],
    });

    categories.push(Category {
        name: "Library Keybinds",
        settings: vec![
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
                name: "Edit/Run",
                key: "bind_lib_edit",
                kind: SettingType::Custom {
                    value: config.bind_lib_edit.to_string(),
                    default: def.bind_lib_edit.to_string(),
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
                name: "Move Out",
                key: "bind_lib_move_out",
                kind: SettingType::Custom {
                    value: config.bind_lib_move_out.to_string(),
                    default: def.bind_lib_move_out.to_string(),
                    validation: CustomType::Char,
                },
            },
            Setting {
                name: "Move In",
                key: "bind_lib_move_in",
                kind: SettingType::Custom {
                    value: config.bind_lib_move_in.to_string(),
                    default: def.bind_lib_move_in.to_string(),
                    validation: CustomType::Char,
                },
            },
            Setting {
                name: "Reset Library Keybinds",
                key: "reset_lib_keybinds",
                kind: SettingType::Action,
            },
        ],
    });

    categories.push(Category {
        name: "Editor",
        settings: vec![
            Setting {
                name: "Tab Backspace",
                key: "edit_tab_backspace",
                kind: SettingType::Choice(
                    vec!["true".to_string(), "false".to_string()],
                    if config.edit_tab_backspace { 0 } else { 1 },
                ),
            },
            Setting {
                name: "Auto Indent",
                key: "edit_auto_indent",
                kind: SettingType::Choice(
                    vec!["true".to_string(), "false".to_string()],
                    if config.edit_auto_indent { 0 } else { 1 },
                ),
            },
            Setting {
                name: "Auto Bracket",
                key: "edit_auto_bracket",
                kind: SettingType::Choice(
                    vec!["true".to_string(), "false".to_string()],
                    if config.edit_auto_bracket { 0 } else { 1 },
                ),
            },
            Setting {
                name: "Error Highlight",
                key: "edit_error_highlight",
                kind: SettingType::Choice(
                    vec!["background".to_string(), "underline".to_string()],
                    match config.edit_error_highlight.as_str() {
                        "underline" => 1,
                        _ => 0,
                    },
                ),
            },
            Setting {
                name: "Reset Editor",
                key: "reset_editor",
                kind: SettingType::Action,
            },
        ],
    });

    categories.push(Category {
        name: "Editor Keybinds",
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
                name: "Error Jump",
                key: "bind_edit_error_jump",
                kind: SettingType::Custom {
                    value: config.bind_edit_error_jump.to_string(),
                    default: def.bind_edit_error_jump.to_string(),
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
                name: "Reset Editor Keybinds",
                key: "reset_edit_keybinds",
                kind: SettingType::Action,
            },
        ],
    });

    categories.push(Category {
        name: "Advanced",
        settings: vec![
            Setting {
                name: "Double Q Exit",
                key: "double_q_exit",
                kind: SettingType::Choice(
                    vec!["true".to_string(), "false".to_string()],
                    if config.double_q_exit { 0 } else { 1 },
                ),
            },
            Setting {
                name: "Reset Config",
                key: "reset_config",
                kind: SettingType::Action,
            },
            Setting {
                name: "Reset Lib",
                key: "reset_lib",
                kind: SettingType::Action,
            },
            Setting {
                name: "Reset Macrodata",
                key: "reset_macrodata",
                kind: SettingType::Action,
            },
        ],
    });

    categories
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
    let mut edit_cursor = 0_usize;

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
    let mut prev_deck = config.deck;
    let mut prev_deck_mode = config.deck_mode.clone();
    let mut dirty = true;

    let mut q_pressed_once = false;

    let version_str = format!(" v{} ", env!("CARGO_PKG_VERSION"));
    let version_len = version_str.chars().count() as u16;

    let mut last_blink = false;

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
                apply_setting!(config, set, choice "settings_style", settings_style);
                apply_setting!(config, set, choice "setting_indicator_choice", setting_indicator_choice);
                apply_setting!(config, set, choice "setting_indicator_custom", setting_indicator_custom);
                apply_setting!(config, set, bool "fancy_bools", fancy_bools);
                apply_setting!(config, set, choice "theme", theme);
                apply_setting!(config, set, choice "border_style", border_style);
                apply_setting!(config, set, choice "indicator_style", indicator_style);
                apply_setting!(config, set, choice "lib_sorting", lib_sorting);
                apply_setting!(config, set, choice_offset "tabs_num", tabs_num, 1);
                apply_setting!(config, set, parse_clamp "lib_width", lib_width, 16, 64);
                apply_setting!(config, set, bool "deck", deck);
                apply_setting!(config, set, choice "deck_mode", deck_mode);
                apply_setting!(config, set, bool "show_caret", show_caret);

                apply_setting!(config, set, parse_clamp "keyvis_width", keyvis_width, 1, 1024);
                apply_setting!(config, set, parse_clamp "keyvis_height", keyvis_height, 2, 32);
                apply_setting!(config, set, choice_offset "keyvis_steps", keyvis_steps, 1);
                apply_setting!(config, set, parse_clamp "keyvis_spread", keyvis_spread, 2, 32);
                apply_setting!(config, set, parse_clamp "keyvis_force", keyvis_force, 0.1, 1.0);
                apply_setting!(config, set, parse_clamp "keyvis_gravity", keyvis_gravity, 0.1, 1.0);
                apply_setting!(config, set, parse_clamp "keyvis_tension", keyvis_tension, 0.1, 1.0);
                apply_setting!(config, set, bool "keyvis_base", keyvis_base);

                apply_setting!(config, set, parse_clamp "matrix_height", matrix_height, 2, 32);
                apply_setting!(config, set, parse_clamp "matrix_density", matrix_density, 1, 200);
                apply_setting!(config, set, parse_clamp "matrix_dim_ratio", matrix_dim_ratio, 0, 100);
                apply_setting!(config, set, parse_clamp "matrix_speed", matrix_speed, 0.1, 5.0);
                apply_setting!(config, set, choice "matrix_direction", matrix_direction);
                apply_setting!(config, set, parse_clamp "matrix_min_length", matrix_min_length, 2, 64);
                apply_setting!(config, set, parse_clamp "matrix_max_length", matrix_max_length, 2, 64);

                apply_setting!(config, set, bool "monitor_cpu", monitor_cpu);
                apply_setting!(config, set, bool "monitor_gpu", monitor_gpu);
                apply_setting!(config, set, bool "monitor_mem", monitor_mem);
                apply_setting!(config, set, bool "monitor_term", monitor_term);
                apply_setting!(config, set, bool "monitor_divider", monitor_divider);
                apply_setting!(config, set, bool "monitor_bar", monitor_bar);
                apply_setting!(config, set, bool "monitor_icons", monitor_icons);
                apply_setting!(config, set, choice "monitor_cpu_style", monitor_cpu_style);
                apply_setting!(config, set, choice "monitor_gpu_style", monitor_gpu_style);
                apply_setting!(config, set, choice "monitor_mem_style", monitor_mem_style);
                apply_setting!(config, set, choice "monitor_bar_style", monitor_bar_style);
                apply_setting!(config, set, parse_clamp "monitor_bar_width", monitor_bar_width, 4, 16);

                apply_setting!(config, set, bool "clock_date", clock_date);
                apply_setting!(config, set, choice "clock_date_style", clock_date_style);
                apply_setting!(config, set, choice "clock_mode", clock_mode);
                apply_setting!(config, set, choice "clock_position", clock_position);
                apply_setting!(config, set, choice "clock_format", clock_format);
                apply_setting!(config, set, bool "clock_seconds", clock_seconds);

                apply_setting!(config, set, bool "macrostats_icons", macrostats_icons);
                apply_setting!(config, set, bool "macrostats_edit_name", macrostats_edit_name);
                apply_setting!(config, set, bool "macrostats_edit_err", macrostats_edit_err);
                apply_setting!(config, set, choice "macrostats_edit_err_style", macrostats_edit_err_style);
                apply_setting!(config, set, bool "macrostats_edit_created", macrostats_edit_created);
                apply_setting!(config, set, bool "macrostats_edit_lines", macrostats_edit_lines);
                apply_setting!(config, set, bool "macrostats_edit_code", macrostats_edit_code);
                apply_setting!(config, set, bool "macrostats_run_name", macrostats_run_name);
                apply_setting!(config, set, bool "macrostats_run_elapsed", macrostats_run_elapsed);
                apply_setting!(config, set, bool "macrostats_run_cpu", macrostats_run_cpu);
                apply_setting!(config, set, bool "macrostats_lib_name", macrostats_lib_name);
                apply_setting!(config, set, bool "macrostats_lib_created", macrostats_lib_created);
                apply_setting!(config, set, bool "macrostats_lib_size", macrostats_lib_size);
                apply_setting!(config, set, bool "macrostats_lib_status", macrostats_lib_status);
                apply_setting!(config, set, parse_clamp "macrostats_err_chart_len", macrostats_err_chart_len, 4, 16);
                apply_setting!(config, set, bool "macrostats_err_chart_num", macrostats_err_chart_num);

                apply_setting!(config, set, bool "double_q_exit", double_q_exit);

                apply_setting!(config, set, bool "edit_tab_backspace", edit_tab_backspace);
                apply_setting!(config, set, bool "edit_auto_indent", edit_auto_indent);
                apply_setting!(config, set, bool "edit_auto_bracket", edit_auto_bracket);
                apply_setting!(config, set, choice "edit_error_highlight", edit_error_highlight);
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
                apply_setting!(config, set, char "bind_edit_error_jump", bind_edit_error_jump);
                apply_setting!(config, set, char "bind_edit_undo", bind_edit_undo);
                apply_setting!(config, set, char "bind_edit_redo", bind_edit_redo);
                apply_setting!(config, set, char "bind_edit_save", bind_edit_save);

                apply_setting!(config, set, char "bind_lib_new_file", bind_lib_new_file);
                apply_setting!(config, set, char "bind_lib_new_folder", bind_lib_new_folder);
                apply_setting!(config, set, char "bind_lib_edit", bind_lib_edit);
                apply_setting!(config, set, char "bind_lib_rename", bind_lib_rename);
                apply_setting!(config, set, char "bind_lib_delete", bind_lib_delete);
                apply_setting!(config, set, char "bind_lib_move_up", bind_lib_move_up);
                apply_setting!(config, set, char "bind_lib_move_down", bind_lib_move_down);
                apply_setting!(config, set, char "bind_lib_move_out", bind_lib_move_out);
                apply_setting!(config, set, char "bind_lib_move_in", bind_lib_move_in);
            }
        }
    };

    loop {
        let (current_w, current_h) = Terminal::size();

        if current_w != canvas.width || current_h != canvas.height {
            canvas.resize(current_w, current_h);
            dirty = true;
        }

        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let current_blink = time % 1000 < 500;
        if edit_mode && current_blink != last_blink {
            last_blink = current_blink;
            dirty = true;
        }

        let mut bg_dirty = false;
        if config.deck {
            if config.deck_mode == "keyvis" {
                if main_view.keyvis.tick(
                    config.keyvis_gravity,
                    config.keyvis_steps,
                    config.keyvis_tension,
                ) {
                    main_view.refresh_static_boxes(config);
                    bg_dirty = true;
                }
            } else if config.deck_mode == "monitor" {
                if main_view.monitor.tick(current_w, current_h) {
                    main_view.refresh_static_boxes(config);
                    bg_dirty = true;
                }
            } else if config.deck_mode == "clock" {
                if main_view.clock.tick(current_w, current_h, config) {
                    main_view.refresh_static_boxes(config);
                    bg_dirty = true;
                }
            } else if config.deck_mode == "macrostats" {
                let _ = main_view.monitor.tick(current_w, current_h);
                let info = main_view.get_macrostats_info();
                if main_view.macrostats.tick(&info) {
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

            let max_cat_len = left_w.saturating_sub(4) as usize;

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
                    text.push_str(&" ".repeat(max_cat_len.saturating_sub(char_count)));
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

            let max_det_len = right_w.saturating_sub(4) as usize;
            let target_width = max_det_len;
            let max_name_len = current_settings
                .iter()
                .map(|s| s.name.chars().count() + 1)
                .max()
                .unwrap_or(0);

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

                let (cho_l, cho_r) = match config.setting_indicator_choice.as_str() {
                    "classic" => ("› ", " ‹"),
                    "none" => ("  ", "  "),
                    _ => ("▶ ", " ◀"),
                };
                let pad_cho_l = " ".repeat(cho_l.chars().count());

                let (cus_l, cus_r) = match config.setting_indicator_custom.as_str() {
                    "classic" => ("» ", " «"),
                    "none" => ("  ", "  "),
                    _ => ("❯ ", " ❮"),
                };
                let pad_cus_l = " ".repeat(cus_l.chars().count());

                let mut cursor_char_idx: Option<usize> = None;
                let mut is_fancy_bool = false;
                let mut bool_val = false;

                let mut text = match &setting.kind {
                    SettingType::Action => setting.name.to_string(),
                    _ => {
                        let val_str = match &setting.kind {
                            SettingType::Choice(opts, idx) => {
                                let mut disp_val = opts[*idx].clone();
                                if config.fancy_bools
                                    && opts.len() == 2
                                    && opts[0] == "true"
                                    && opts[1] == "false"
                                {
                                    is_fancy_bool = true;
                                    bool_val = *idx == 0;
                                    disp_val = "[▪]".to_string();
                                }

                                if is_selected && det_active {
                                    if config.settings_style == "inline" {
                                        format!("{}{}", disp_val, cho_r)
                                    } else if config.settings_style == "aligned" {
                                        format!("{}{}{}", cho_l, disp_val, cho_r)
                                    } else {
                                        format!("{}{}", cho_l, disp_val)
                                    }
                                } else if config.settings_style == "inline" {
                                    format!("{}", disp_val)
                                } else {
                                    format!("{}{}", pad_cho_l, disp_val)
                                }
                            }
                            SettingType::Custom { value, .. } => {
                                if edit_mode && is_selected && det_active {
                                    let cursor_buffer = edit_buffer.clone();

                                    let suffix = if config.settings_style == "right" {
                                        if edit_cursor == cursor_buffer.chars().count() {
                                            " ".to_string()
                                        } else {
                                            String::new()
                                        }
                                    } else {
                                        cus_r.to_string()
                                    };
                                    let suffix_len = suffix.chars().count();

                                    let name_str = format!("{}:", setting.name);
                                    let name_len = name_str.chars().count();

                                    let val_prefix = if config.settings_style == "inline" {
                                        String::new()
                                    } else {
                                        cus_l.to_string()
                                    };
                                    let val_prefix_len = val_prefix.chars().count();

                                    let name_total_len = if config.settings_style == "aligned" {
                                        max_name_len.max(name_len)
                                    } else {
                                        name_len
                                    };

                                    let max_input_len = target_width
                                        .saturating_sub(
                                            name_total_len + 1 + val_prefix_len + suffix_len,
                                        )
                                        .max(1);

                                    let mut display_buffer = cursor_buffer.clone();
                                    let mut visual_cursor_pos = edit_cursor;

                                    if display_buffer.chars().count() > max_input_len {
                                        let skip = if edit_cursor == cursor_buffer.chars().count() {
                                            edit_cursor.saturating_sub(max_input_len)
                                        } else {
                                            edit_cursor
                                                .saturating_sub(max_input_len.saturating_sub(1))
                                        };
                                        display_buffer = display_buffer
                                            .chars()
                                            .skip(skip)
                                            .take(max_input_len)
                                            .collect();
                                        visual_cursor_pos = edit_cursor.saturating_sub(skip);
                                    }

                                    let full_val_len = val_prefix_len
                                        + display_buffer.chars().count()
                                        + suffix_len;

                                    let prefix_to_buffer = if config.settings_style == "right" {
                                        if name_len + 1 + full_val_len >= target_width {
                                            format!("{} {}", name_str, val_prefix)
                                        } else {
                                            let spaces = target_width
                                                .saturating_sub(name_len + full_val_len);
                                            format!(
                                                "{}{}{}",
                                                name_str,
                                                " ".repeat(spaces),
                                                val_prefix
                                            )
                                        }
                                    } else if config.settings_style == "aligned" {
                                        let name_pad = max_name_len.saturating_sub(name_len);
                                        format!(
                                            "{}{} {}",
                                            name_str,
                                            " ".repeat(name_pad),
                                            val_prefix
                                        )
                                    } else {
                                        format!("{} {}", name_str, val_prefix)
                                    };

                                    cursor_char_idx =
                                        Some(prefix_to_buffer.chars().count() + visual_cursor_pos);

                                    if config.settings_style == "inline" {
                                        format!("{}{}", display_buffer, suffix)
                                    } else if config.settings_style == "aligned" {
                                        format!("{}{}{}", val_prefix, display_buffer, suffix)
                                    } else {
                                        format!("{}{}{}", val_prefix, display_buffer, suffix)
                                    }
                                } else {
                                    if is_selected && det_active {
                                        if config.settings_style == "inline" {
                                            format!("{}{}", value, cus_r)
                                        } else if config.settings_style == "aligned" {
                                            format!("{}{}{}", cus_l, value, cus_r)
                                        } else {
                                            format!("{}{}", cus_l, value)
                                        }
                                    } else {
                                        if config.settings_style == "inline" {
                                            format!("{}", value)
                                        } else {
                                            format!("{}{}", pad_cus_l, value)
                                        }
                                    }
                                }
                            }
                            SettingType::Action => unreachable!(),
                        };

                        let name_str = format!("{}:", setting.name);
                        let name_len = name_str.chars().count();
                        let val_len = val_str.chars().count();

                        if config.settings_style == "right" {
                            if name_len + val_len >= target_width {
                                format!("{} {}", name_str, val_str)
                            } else {
                                let spaces = target_width.saturating_sub(name_len + val_len);
                                format!("{}{}{}", name_str, " ".repeat(spaces), val_str)
                            }
                        } else if config.settings_style == "aligned" {
                            let name_pad = max_name_len.saturating_sub(name_len);
                            format!("{}{} {}", name_str, " ".repeat(name_pad), val_str)
                        } else {
                            format!("{} {}", name_str, val_str)
                        }
                    }
                };

                let char_count = text.chars().count();

                if char_count > max_det_len {
                    text = text.chars().take(max_det_len).collect();
                } else {
                    text.push_str(&" ".repeat(max_det_len.saturating_sub(char_count)));
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

                if let Some(char_idx) = cursor_char_idx {
                    let cursor_y_idx = (det_box.padding as i16 + display_y) as usize;
                    let mut visual_cursor_offset = 0;
                    for c in text.chars().take(char_idx) {
                        visual_cursor_offset += crate::render::canvas::char_width(c) as usize;
                    }

                    let mut cursor_x_idx = det_box.padding as usize + 1 + visual_cursor_offset;

                    let max_x = det_box
                        .width
                        .saturating_sub(det_box.padding)
                        .saturating_sub(1) as usize;
                    cursor_x_idx = cursor_x_idx.min(max_x);

                    if cursor_x_idx < det_box.width as usize
                        && cursor_y_idx < det_box.height as usize
                    {
                        let cell_idx = cursor_y_idx * det_box.width as usize + cursor_x_idx;
                        det_box.grid[cell_idx].s.md = Modifier::Underline;
                    }
                }

                if is_fancy_bool {
                    if let Some(dot_byte_idx) = text.rfind('▪') {
                        let text_before = &text[..dot_byte_idx];
                        let mut x_offset = 0;
                        for c in text_before.chars() {
                            x_offset += crate::render::canvas::char_width(c) as u16;
                        }

                        let mut t_col = main_view.theme.fancy_bools_true.color_at(0, 1);
                        if t_col == Color::None {
                            t_col = Color::Green;
                        }
                        let mut f_col = main_view.theme.fancy_bools_false.color_at(0, 1);
                        if f_col == Color::None {
                            f_col = Color::Red;
                        }

                        let dot_color = if bool_val { t_col } else { f_col };
                        let dot_char_idx = text_before.chars().count();
                        let final_text_len = text.chars().count();
                        let style = Style {
                            fg: dot_color,
                            bg: bg_color.color_at(dot_char_idx, final_text_len),
                            md: Modifier::Bold,
                        };
                        det_box.put_cell(
                            Cell::new('▪', style),
                            2 + x_offset,
                            (1 + display_y) as u16,
                        );
                    }
                }
            }

            let v_y = det_box.height.saturating_sub(1);
            let v_start_x = det_box.width.saturating_sub(3 + version_len);

            for (i, c) in version_str.chars().enumerate() {
                let offset = i as u16;
                if v_start_x + offset < det_box.width {
                    det_box.put_cell(
                        Cell::new(
                            c,
                            Style {
                                fg: main_view
                                    .theme
                                    .settings_options_box
                                    .color_at(i, version_len as usize),
                                bg: Color::None,
                                md: Modifier::None,
                            },
                        ),
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
        if key != Key::None && config.deck && config.deck_mode == "keyvis" {
            main_view
                .keyvis
                .push_key(&key, config.keyvis_force, config.keyvis_spread);
        }

        if key != Key::None && key != Key::Char('q') {
            q_pressed_once = false;
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
                                CustomType::IntRange(min, max) => {
                                    if let Ok(v) = edit_buffer.parse::<usize>() {
                                        edit_buffer = v.clamp(*min, *max).to_string();
                                        do_apply = true;
                                    }
                                }
                                CustomType::FloatRange(min, max) => {
                                    if let Ok(v) = edit_buffer.parse::<f32>() {
                                        edit_buffer = v.clamp(*min, *max).to_string();
                                        do_apply = true;
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
                        if cat_selected >= categories.len() {
                            cat_selected = categories.len().saturating_sub(1);
                        }
                        det_selected.resize(categories.len(), 0);
                        det_scroll.resize(categories.len(), 0);
                        needs_ui_refresh = true;
                    }
                    dirty = true;
                }
                Key::Esc => {
                    edit_mode = false;
                    dirty = true;
                }
                Key::Backspace => {
                    if edit_cursor > 0 && !edit_buffer.is_empty() {
                        edit_cursor -= 1;
                        let byte_idx = edit_buffer
                            .char_indices()
                            .nth(edit_cursor)
                            .map(|(i, _)| i)
                            .unwrap();
                        edit_buffer.remove(byte_idx);
                    }
                    dirty = true;
                }
                Key::CtrlBackspace | Key::Ctrl('w') | Key::Ctrl('h') => {
                    if edit_cursor > 0 && !edit_buffer.is_empty() {
                        let initial_cursor = edit_cursor;
                        let chars: Vec<char> = edit_buffer.chars().collect();
                        let mut i = edit_cursor;

                        while i > 0 && chars[i - 1].is_whitespace() {
                            i -= 1;
                        }
                        if i > 0 {
                            let is_word = chars[i - 1].is_alphanumeric() || chars[i - 1] == '_';
                            while i > 0 {
                                let prev_is_word =
                                    chars[i - 1].is_alphanumeric() || chars[i - 1] == '_';
                                if chars[i - 1].is_whitespace() || prev_is_word != is_word {
                                    break;
                                }
                                i -= 1;
                            }
                        }

                        let delete_count = initial_cursor - i;
                        if delete_count > 0 {
                            let start_byte = edit_buffer
                                .char_indices()
                                .nth(i)
                                .map(|(idx, _)| idx)
                                .unwrap_or(edit_buffer.len());
                            let end_byte = edit_buffer
                                .char_indices()
                                .nth(initial_cursor)
                                .map(|(idx, _)| idx)
                                .unwrap_or(edit_buffer.len());
                            edit_buffer.drain(start_byte..end_byte);
                            edit_cursor = i;
                        }
                    }
                    dirty = true;
                }
                Key::Left => {
                    if edit_cursor > 0 {
                        edit_cursor -= 1;
                    }
                    dirty = true;
                }
                Key::CtrlLeft => {
                    let chars: Vec<char> = edit_buffer.chars().collect();
                    let mut i = edit_cursor;
                    while i > 0 && chars[i - 1].is_whitespace() {
                        i -= 1;
                    }
                    if i > 0 {
                        let is_word = chars[i - 1].is_alphanumeric() || chars[i - 1] == '_';
                        while i > 0 {
                            let prev_is_word =
                                chars[i - 1].is_alphanumeric() || chars[i - 1] == '_';
                            if chars[i - 1].is_whitespace() || prev_is_word != is_word {
                                break;
                            }
                            i -= 1;
                        }
                    }
                    edit_cursor = i;
                    dirty = true;
                }
                Key::Right => {
                    if edit_cursor < edit_buffer.chars().count() {
                        edit_cursor += 1;
                    }
                    dirty = true;
                }
                Key::CtrlRight => {
                    let chars: Vec<char> = edit_buffer.chars().collect();
                    let mut i = edit_cursor;
                    let len = chars.len();

                    if i < len && chars[i].is_whitespace() {
                        while i < len && chars[i].is_whitespace() {
                            i += 1;
                        }
                    } else if i < len {
                        let is_word = chars[i].is_alphanumeric() || chars[i] == '_';
                        while i < len {
                            let curr_is_word = chars[i].is_alphanumeric() || chars[i] == '_';
                            if chars[i].is_whitespace() || curr_is_word != is_word {
                                break;
                            }
                            i += 1;
                        }
                        while i < len && chars[i].is_whitespace() {
                            i += 1;
                        }
                    }
                    edit_cursor = i;
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
                            edit_cursor = 1;
                        } else {
                            let byte_idx = edit_buffer
                                .char_indices()
                                .nth(edit_cursor)
                                .map(|(i, _)| i)
                                .unwrap_or(edit_buffer.len());
                            edit_buffer.insert(byte_idx, c);
                            edit_cursor += 1;
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
                            edit_cursor = 1;
                        } else {
                            let byte_idx = edit_buffer
                                .char_indices()
                                .nth(edit_cursor)
                                .map(|(i, _)| i)
                                .unwrap_or(edit_buffer.len());
                            edit_buffer.insert(byte_idx, final_c);
                            edit_cursor += 1;
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
                    if key == Key::Char('q') && config.double_q_exit && !q_pressed_once {
                        q_pressed_once = true;
                        continue;
                    }
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
                                if cat_selected >= categories.len() {
                                    cat_selected = categories.len().saturating_sub(1);
                                }
                                det_selected.resize(categories.len(), 0);
                                det_scroll.resize(categories.len(), 0);
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
                                if cat_selected >= categories.len() {
                                    cat_selected = categories.len().saturating_sub(1);
                                }
                                det_selected.resize(categories.len(), 0);
                                det_scroll.resize(categories.len(), 0);
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
                                    edit_cursor = edit_buffer.chars().count();
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
                                        "reset_lib" => {
                                            ("Delete all files in the library\n\nAre you sure?", 5)
                                        }
                                        "reset_deck" => {
                                            ("Reset deck settings to default\n\nAre you sure?", 6)
                                        }
                                        "reset_macrodata" => {
                                            ("Delete all saved macro data\n\nAre you sure?", 7)
                                        }
                                        "reset_editor" => {
                                            ("Reset editor settings to default\n\nAre you sure?", 8)
                                        }
                                        "reset_keyvis" => {
                                            ("Reset Keyvis settings to default\n\nAre you sure?", 9)
                                        }
                                        "reset_monitor" => (
                                            "Reset Monitor settings to default\n\nAre you sure?",
                                            10,
                                        ),
                                        "reset_clock" => {
                                            ("Reset Clock settings to default\n\nAre you sure?", 11)
                                        }
                                        "reset_macrostats" => (
                                            "Reset Macrostats settings to default\n\nAre you sure?",
                                            12,
                                        ),
                                        "reset_library" => (
                                            "Reset library settings to default\n\nAre you sure?",
                                            13,
                                        ),
                                        "reset_settings_menu" => (
                                            "Reset settings menu configuration to default\n\nAre you sure?",
                                            14,
                                        ),
                                        "reset_matrix" => (
                                            "Reset Matrix settings to default\n\nAre you sure?",
                                            15,
                                        ),
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
                                        } else if action_type == 7 {
                                            crate::lib::reset_macrodata();
                                        } else if action_type == 8 {
                                            config.reset_editor();
                                        } else if action_type == 9 {
                                            config.reset_keyvis();
                                        } else if action_type == 10 {
                                            config.reset_monitor();
                                        } else if action_type == 11 {
                                            config.reset_clock();
                                        } else if action_type == 12 {
                                            config.reset_macrostats();
                                        } else if action_type == 13 {
                                            config.reset_library();
                                        } else if action_type == 14 {
                                            config.reset_settings_menu();
                                        } else if action_type == 15 {
                                            config.reset_matrix();
                                        }
                                        config.save();

                                        if action_type == 4 || action_type == 5 || action_type == 0
                                        {
                                            main_view.reload_library_tree(config);

                                            if action_type == 5 || action_type == 0 {
                                                for i in 0..6 {
                                                    main_view.expanded_path[i].clear();
                                                    main_view.list_selected[i] = 0;
                                                    main_view.list_scroll[i] = 0;
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
                                                    main_view.macro_start_times[i] = None;
                                                    main_view.editors[i].error_count = 0;
                                                    main_view.editors[i].error_lines.clear();
                                                }
                                            }
                                            main_view.auto_load();
                                        }

                                        let current_theme_idx = themes
                                            .iter()
                                            .position(|t| t == &config.theme)
                                            .unwrap_or(0);
                                        categories =
                                            build_categories(config, &themes, current_theme_idx);
                                        if cat_selected >= categories.len() {
                                            cat_selected = categories.len().saturating_sub(1);
                                        }
                                        det_selected.resize(categories.len(), 0);
                                        det_scroll.resize(categories.len(), 0);
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
                        main_view.update_min_sizes(config);

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
                main_view.reload_library_tree(config);
                main_view.auto_load();
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
                        main_view.macro_start_times[i] = None;
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

            if config.deck != prev_deck || config.deck_mode != prev_deck_mode {
                let (term_w, term_h) = Terminal::size();
                main_view.resize(term_w, term_h, config);
                prev_deck = config.deck;
                prev_deck_mode = config.deck_mode.clone();

                if cat_selected >= categories.len() {
                    cat_selected = categories.len().saturating_sub(1);
                }
                det_selected.resize(categories.len(), 0);
                det_scroll.resize(categories.len(), 0);
            } else {
                main_view.update_min_sizes(config);
                let (term_w, term_h) = Terminal::size();
                main_view.resize(term_w, term_h, config);
            }

            dirty = true;
        }
    }
}
