use crate::{Box, Color, Gradient, Modifier, Style, conf::Config, theme::themecore::Theme};
use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug, Clone)]
pub enum MacroInfo {
    None,
    Library {
        name: String,
        path: PathBuf,
        is_running: bool,
    },
    Editing {
        name: String,
        path: Option<PathBuf>,
        lines: usize,
        loc: usize,
        errors: usize,
    },
    Running {
        name: String,
        start_time: Instant,
        cpu_usage: u8,
    },
}

pub struct MacrostatsState {
    pub last_hash: u64,
    pub last_sec: u64,
    pub cached_created: String,
    pub cached_size: String,
    pub last_path: Option<PathBuf>,
}

impl MacrostatsState {
    pub fn new() -> Self {
        Self {
            last_hash: 0,
            last_sec: 0,
            cached_created: String::new(),
            cached_size: String::new(),
            last_path: None,
        }
    }

    fn get_meta(&mut self, path: &std::path::Path) {
        if self.last_path.as_deref() == Some(path) && !self.cached_created.is_empty() {
            return;
        }
        if let Ok(meta) = std::fs::metadata(path) {
            let created = meta.created().or_else(|_| meta.modified()).ok();
            self.cached_created = if let Some(c) = created {
                let datetime: chrono::DateTime<chrono::Local> = c.into();
                datetime.format("%Y-%m-%d %H:%M").to_string()
            } else {
                "Unknown".to_string()
            };

            let size = meta.len();
            self.cached_size = if size < 1024 {
                format!("{} B", size)
            } else if size < 1024 * 1024 {
                format!("{:.1} KB", size as f64 / 1024.0)
            } else {
                format!("{:.2} MB", size as f64 / 1048576.0)
            };
        } else {
            self.cached_created = "Unknown".to_string();
            self.cached_size = "0 B".to_string();
        }
        self.last_path = Some(path.to_path_buf());
    }

    pub fn tick(&mut self, info: &MacroInfo) -> bool {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        let mut current_sec = 0;

        match info {
            MacroInfo::None => {
                0.hash(&mut hasher);
            }
            MacroInfo::Library {
                name,
                path,
                is_running,
            } => {
                1.hash(&mut hasher);
                name.hash(&mut hasher);
                path.hash(&mut hasher);
                is_running.hash(&mut hasher);
                self.get_meta(path);
            }
            MacroInfo::Editing {
                name,
                path,
                lines,
                loc,
                errors,
            } => {
                2.hash(&mut hasher);
                name.hash(&mut hasher);
                path.hash(&mut hasher);
                lines.hash(&mut hasher);
                loc.hash(&mut hasher);
                errors.hash(&mut hasher);
                if let Some(p) = path {
                    self.get_meta(p);
                }
            }
            MacroInfo::Running {
                name,
                start_time,
                cpu_usage,
            } => {
                3.hash(&mut hasher);
                name.hash(&mut hasher);
                current_sec = start_time.elapsed().as_secs();
                current_sec.hash(&mut hasher);
                cpu_usage.hash(&mut hasher);
            }
        }

        let new_hash = hasher.finish();
        if new_hash != self.last_hash || current_sec != self.last_sec {
            self.last_hash = new_hash;
            self.last_sec = current_sec;
            true
        } else {
            false
        }
    }
}

fn push_text(
    cells: &mut Vec<(char, Style)>,
    text: &str,
    fg: &Gradient,
    bg: &Gradient,
    md: Modifier,
) {
    let len = text.chars().count();
    for (i, c) in text.chars().enumerate() {
        cells.push((
            c,
            Style {
                fg: fg.color_at(i, len),
                bg: bg.color_at(i, len),
                md,
            },
        ));
    }
}

pub fn draw(
    state: &MacrostatsState,
    info: &MacroInfo,
    b: &mut Box,
    config: &Config,
    theme: &Theme,
) {
    if b.width == 0 || b.height == 0 {
        return;
    }

    let bg_none = Gradient::Solid(Color::None);
    let key_color = &theme.macrostats_key;
    let val_color = &theme.macrostats_val;

    let mut lines_to_draw: Vec<Vec<(char, Style)>> = Vec::new();

    let gap_str = if config.monitor_icons { "   " } else { "     " };
    let push_gap = |l: &mut Vec<(char, Style)>| {
        if !l.is_empty() {
            push_text(l, gap_str, &bg_none, &bg_none, Modifier::None);
        }
    };

    match info {
        MacroInfo::Editing {
            name,
            path: _,
            lines,
            loc,
            errors,
        } => {
            let mut line0 = Vec::new();
            let mut line1 = Vec::new();

            if config.macrostats_edit_name {
                let lbl = if config.monitor_icons {
                    "▪ "
                } else {
                    "File: "
                };
                push_text(&mut line0, lbl, key_color, &bg_none, Modifier::Bold);
                push_text(&mut line0, name, val_color, &bg_none, Modifier::None);
            }

            let show_err_text =
                config.macrostats_edit_err == "text" || config.macrostats_edit_err == "both";
            let show_err_chart =
                config.macrostats_edit_err == "chart" || config.macrostats_edit_err == "both";

            let push_chart = |target_line: &mut Vec<(char, Style)>| {
                let chart_len = config.macrostats_err_chart_len.clamp(4, 16);
                let err_str = errors.to_string();
                let num_len = err_str.chars().count();
                let display_len = num_len.min(chart_len);
                let active_idx = (*errors).min(chart_len.saturating_sub(1));

                let start_idx = if active_idx + num_len > chart_len {
                    chart_len.saturating_sub(num_len)
                } else {
                    active_idx
                };
                let end_idx = start_idx + display_len;

                let mut num_chars = err_str.chars();
                for i in 0..chart_len {
                    if i >= start_idx && i < end_idx && config.macrostats_err_chart_num {
                        target_line.push((
                            num_chars.next().unwrap_or(' '),
                            Style {
                                fg: Color::Black,
                                bg: theme.macrostats_err.color_at(i, chart_len),
                                md: Modifier::None,
                            },
                        ));
                    } else if i == active_idx && !config.macrostats_err_chart_num {
                        target_line.push((
                            '█',
                            Style {
                                fg: theme.macrostats_err.color_at(i, chart_len),
                                bg: Color::None,
                                md: Modifier::None,
                            },
                        ));
                    } else {
                        target_line.push((
                            '█',
                            Style {
                                fg: theme.macrostats_err.color_at(i, chart_len),
                                bg: Color::None,
                                md: Modifier::Dim,
                            },
                        ));
                    }
                }
            };

            let mut err_chart_align_idx = 0;

            if show_err_text || show_err_chart {
                push_gap(&mut line0);
                err_chart_align_idx = line0.len();

                let lbl = if config.monitor_icons {
                    "⚠ "
                } else {
                    "Errors: "
                };
                push_text(&mut line0, lbl, key_color, &bg_none, Modifier::Bold);

                if show_err_text {
                    push_text(
                        &mut line0,
                        &errors.to_string(),
                        val_color,
                        &bg_none,
                        Modifier::None,
                    );
                } else if show_err_chart {
                    push_chart(&mut line0);
                }
            }

            if config.macrostats_edit_created && state.last_path.is_some() {
                push_gap(&mut line0);
                let lbl = if config.monitor_icons {
                    "⌚ "
                } else {
                    "Created: "
                };
                push_text(&mut line0, lbl, key_color, &bg_none, Modifier::Bold);
                push_text(
                    &mut line0,
                    &state.cached_created,
                    val_color,
                    &bg_none,
                    Modifier::None,
                );
            }

            if config.macrostats_edit_lines {
                push_gap(&mut line0);
                let lbl = if config.monitor_icons {
                    "☰ "
                } else {
                    "Lines: "
                };
                push_text(&mut line0, lbl, key_color, &bg_none, Modifier::Bold);
                push_text(
                    &mut line0,
                    &lines.to_string(),
                    val_color,
                    &bg_none,
                    Modifier::None,
                );
            }

            if config.macrostats_edit_code {
                push_gap(&mut line0);
                let lbl = if config.monitor_icons {
                    "≣ "
                } else {
                    "Code: "
                };
                push_text(&mut line0, lbl, key_color, &bg_none, Modifier::Bold);
                push_text(
                    &mut line0,
                    &loc.to_string(),
                    val_color,
                    &bg_none,
                    Modifier::None,
                );
            }

            if config.macrostats_edit_err == "both" {
                for _ in 0..line0.len() {
                    line1.push((
                        ' ',
                        Style {
                            fg: Color::None,
                            bg: Color::None,
                            md: Modifier::None,
                        },
                    ));
                }

                let mut chart_cells = Vec::new();
                push_chart(&mut chart_cells);

                for (i, cell) in chart_cells.into_iter().enumerate() {
                    if err_chart_align_idx + i < line1.len() {
                        line1[err_chart_align_idx + i] = cell;
                    } else {
                        line1.push(cell);
                    }
                }

                while line0.len() < line1.len() {
                    line0.push((
                        ' ',
                        Style {
                            fg: Color::None,
                            bg: Color::None,
                            md: Modifier::None,
                        },
                    ));
                }
            }

            let l0_len = line0.len();
            if !line0.is_empty() {
                lines_to_draw.push(line0);
            }
            if !line1.is_empty() {
                while line1.len() < l0_len {
                    line1.push((
                        ' ',
                        Style {
                            fg: Color::None,
                            bg: Color::None,
                            md: Modifier::None,
                        },
                    ));
                }
                lines_to_draw.push(line1);
            }
        }
        _ => {
            let mut line0 = Vec::new();
            let mut add_item = |k: &str, v: &str, show: bool| {
                if !show {
                    return;
                }
                if !line0.is_empty() {
                    let sep = if config.monitor_divider == "show" {
                        "  |  "
                    } else {
                        "   "
                    };
                    push_text(
                        &mut line0,
                        sep,
                        &theme.monitor_divider,
                        &bg_none,
                        Modifier::None,
                    );
                }
                push_text(&mut line0, k, key_color, &bg_none, Modifier::Bold);
                push_text(&mut line0, v, val_color, &bg_none, Modifier::None);
            };

            match info {
                MacroInfo::None => {
                    add_item("Status: ", "Idle", true);
                }
                MacroInfo::Library {
                    name,
                    path: _,
                    is_running,
                } => {
                    add_item(
                        if config.monitor_icons {
                            "▪ "
                        } else {
                            "Name: "
                        },
                        name,
                        config.macrostats_lib_name,
                    );
                    add_item(
                        if config.monitor_icons {
                            "⌚ "
                        } else {
                            "Created: "
                        },
                        &state.cached_created,
                        config.macrostats_lib_created,
                    );
                    add_item(
                        if config.monitor_icons {
                            "▤ "
                        } else {
                            "Size: "
                        },
                        &state.cached_size,
                        config.macrostats_lib_size,
                    );
                    let status = if *is_running { "Running" } else { "Stopped" };
                    add_item(
                        if config.monitor_icons {
                            "▶ "
                        } else {
                            "Status: "
                        },
                        status,
                        config.macrostats_lib_status,
                    );
                }
                MacroInfo::Running {
                    name,
                    start_time: _,
                    cpu_usage,
                } => {
                    add_item(
                        if config.monitor_icons {
                            "▶ "
                        } else {
                            "Running: "
                        },
                        name,
                        config.macrostats_run_name,
                    );
                    let mins = state.last_sec / 60;
                    let secs = state.last_sec % 60;
                    let hours = mins / 60;
                    let time_str = if hours > 0 {
                        format!("{:02}:{:02}:{:02}", hours, mins % 60, secs)
                    } else {
                        format!("{:02}:{:02}", mins, secs)
                    };
                    add_item(
                        if config.monitor_icons {
                            "⌚ "
                        } else {
                            "Elapsed: "
                        },
                        &time_str,
                        config.macrostats_run_elapsed,
                    );
                    add_item(
                        if config.monitor_icons {
                            "◈ "
                        } else {
                            "CPU: "
                        },
                        &format!("{}%", cpu_usage),
                        config.macrostats_run_cpu,
                    );
                }
                _ => {}
            }
            if !line0.is_empty() {
                lines_to_draw.push(line0);
            }
        }
    }

    let inner_w = b.width.saturating_sub(b.padding * 2);
    let inner_h = b.height.saturating_sub(b.padding * 2);

    let total_lines = lines_to_draw.len() as u16;
    let start_y = (inner_h.saturating_sub(total_lines) + 1) / 2 + b.padding;

    for (line_idx, line) in lines_to_draw.into_iter().enumerate() {
        let line_len = line.len() as u16;
        let start_x = (inner_w.saturating_sub(line_len) / 2).max(0) + b.padding;
        let y = start_y + line_idx as u16;

        for (i, (c, style)) in line.into_iter().enumerate() {
            let cx = start_x + i as u16;
            if cx < b.width.saturating_sub(b.padding) && y < b.height.saturating_sub(b.padding) {
                b.put_cell(crate::Cell { c, s: style }, cx, y);
            }
        }
    }
}
