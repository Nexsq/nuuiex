use std::time::Duration;

use crate::{Border, Box, Canvas, Color, Key, Modifier, Style, Terminal};

#[derive(Debug, Clone, Copy, PartialEq)]
enum ActiveSettingsPanel {
    Categories,
    Details,
}

pub fn settings_modal<F>(
    terminal: &Terminal,
    canvas: &mut Canvas,
    min_w: u16,
    min_h: u16,
    mut draw_background: F,
) -> bool
where
    F: FnMut(&mut Canvas, u16, u16),
{
    let mut active_panel = ActiveSettingsPanel::Categories;

    let categories = vec!["Themes", "Controls", "Library", "Advanced", "About"];
    let settings_data = vec![
        vec![
            "Theme: Default",
            "ThemeSetting1: Something",
            "ThemeSetting2: OtherSomething",
        ],
        vec!["Keybindings", "Mouse Support: Off"],
        vec!["Path: ./lib", "Sort: Name"],
        vec!["Debug Mode: Off", "Reset Settings"],
        vec!["Version: 0.1.0"],
    ];

    let mut cat_selected = 0;
    let mut cat_scroll = 0;

    let mut det_selected = vec![0; categories.len()];
    let mut det_scroll = vec![0; categories.len()];

    let mut dirty = true;

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
                draw_background(canvas, current_w, current_h);
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

                    let mut text = cat.to_string();
                    let char_count = text.chars().count();
                    if char_count > max_cat_len {
                        text = text.chars().take(max_cat_len).collect();
                    } else {
                        text.push_str(&" ".repeat((max_cat_len - char_count).saturating_sub(2)));
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

                let current_settings = &settings_data[cat_selected];
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

                    let mut text = setting.to_string();
                    let char_count = text.chars().count();
                    if char_count > max_det_len {
                        text = text.chars().take(max_det_len).collect();
                    } else {
                        text.push_str(&" ".repeat((max_det_len - char_count).saturating_sub(2)));
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
                match key {
                    Key::Esc | Key::Char('q' | 'Q') => return false,
                    Key::Char('\x03') => return true,
                    Key::Char('e' | 'E') => {
                        active_panel = if active_panel == ActiveSettingsPanel::Categories {
                            ActiveSettingsPanel::Details
                        } else {
                            ActiveSettingsPanel::Categories
                        };
                    }
                    Key::Left => {
                        if active_panel == ActiveSettingsPanel::Details {
                            active_panel = ActiveSettingsPanel::Categories;
                        }
                    }
                    Key::Right => {
                        if active_panel == ActiveSettingsPanel::Categories {
                            active_panel = ActiveSettingsPanel::Details;
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
                            let max_det = settings_data[cat_selected].len().saturating_sub(1);
                            if det_selected[cat_selected] < max_det {
                                det_selected[cat_selected] += 1;
                            }
                        }
                    }
                    _ => continue,
                }
                dirty = true;
            }
        }
    }
}
