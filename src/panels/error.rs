use std::time::Duration;

use crate::{Border, Box, Canvas, Color, Gradient, Key, Modifier, PanelResult, Terminal};

pub fn warning_box<F>(
    terminal: &Terminal,
    canvas: &mut Canvas,
    msg: &str,
    options: &[&str],
    width: u16,
    height: u16,
    min_w: u16,
    min_h: u16,
    border: Border,
    warning_color: Gradient,
    mut draw_background: F,
) -> PanelResult
where
    F: FnMut(&mut Canvas, u16, u16),
{
    let max_msg_len = msg.lines().map(|l| l.chars().count()).max().unwrap_or(0);
    let msg_height = msg.lines().count();

    let total_opts_len = options
        .iter()
        .map(|opt| opt.chars().count() + 4)
        .sum::<usize>()
        + options.len().saturating_sub(1);

    let selected_opts: Vec<String> = options.iter().map(|o| format!("> {} <", o)).collect();
    let unselected_opts: Vec<String> = options.iter().map(|o| format!("  {}  ", o)).collect();

    let mut selected_idx: usize = 0;

    let mut dirty = true;

    loop {
        let (current_w, current_h) = Terminal::size();

        if current_w != canvas.width || current_h != canvas.height {
            canvas.resize(current_w, current_h);
            dirty = true;
        }

        if dirty {
            if current_w < min_w || current_h < min_h {
                if !crate::toosmall::run(
                    terminal,
                    canvas,
                    min_w,
                    min_h,
                    border,
                    warning_color.clone(),
                ) {
                    return PanelResult::Quit;
                }
                dirty = true;
                continue;
            }

            canvas.clean();
            draw_background(canvas, current_w, current_h);
            canvas.apply_dim();

            let term_w = canvas.width;
            let term_h = canvas.height;
            let pad: u16 = 1;

            let box_w = if width == 0 {
                (max_msg_len.max(total_opts_len) + (pad as usize * 2) + 4).max(24) as u16
            } else {
                width
            }
            .min(term_w);

            let box_h = if height == 0 {
                (msg_height + (pad as usize * 2) + 3).max(7) as u16
            } else {
                height
            }
            .min(term_h);

            let box_x = (term_w.saturating_sub(box_w) / 2) as i16;
            let box_y = (term_h.saturating_sub(box_h) / 2) as i16;

            let mut err_box = Box::new(
                box_w,
                box_h,
                pad,
                border,
                warning_color.clone(),
                Gradient::Solid(Color::None),
                Modifier::Bold,
            );
            let inner_w = box_w.saturating_sub(pad * 2);
            let inner_h = box_h.saturating_sub(pad * 2);
            let msg_x = (inner_w.saturating_sub(max_msg_len as u16) / 2) as i16;

            err_box.insert_text(
                msg,
                msg_x,
                0,
                true,
                Gradient::Solid(Color::White),
                Gradient::Solid(Color::None),
                Modifier::None,
            );

            if !options.is_empty() {
                let options_y = inner_h.saturating_sub(1) as i16;
                let mut current_x = (inner_w.saturating_sub(total_opts_len as u16) / 2) as i16;

                for i in 0..options.len() {
                    let is_selected = i == selected_idx;
                    let text = if is_selected {
                        &selected_opts[i]
                    } else {
                        &unselected_opts[i]
                    };

                    let (fg, bg, md) = if is_selected {
                        (
                            Gradient::Solid(Color::Black),
                            Gradient::Solid(Color::White),
                            Modifier::Bold,
                        )
                    } else {
                        (
                            Gradient::Solid(Color::Black),
                            warning_color.clone(),
                            Modifier::None,
                        )
                    };

                    err_box.insert_text(text, current_x, options_y, false, fg, bg, md);
                    current_x += (text.chars().count() + 1) as i16;
                }
            }

            canvas.put_box_opaque(&err_box, box_x, box_y);
            canvas.render();
            dirty = false;
        }

        let key = terminal.read_key(Duration::from_millis(16));
        if key == Key::None {
            continue;
        }

        match key {
            Key::Left | Key::Up => {
                if selected_idx > 0 {
                    selected_idx -= 1;
                } else if !options.is_empty() {
                    selected_idx = options.len() - 1;
                }
                dirty = true;
            }
            Key::Right | Key::Down => {
                if selected_idx < options.len().saturating_sub(1) {
                    selected_idx += 1;
                } else {
                    selected_idx = 0;
                }
                dirty = true;
            }
            Key::Enter => return PanelResult::Ok(selected_idx),
            Key::Esc => return PanelResult::Cancel,
            Key::Char('q') | Key::Char('\x03') => return PanelResult::Quit,
            _ => {
                dirty = true;
            }
        }
    }
}

pub fn error_box(
    terminal: &Terminal,
    canvas: &mut Canvas,
    msg: &str,
    options: &[&str],
    min_w: u16,
    min_h: u16,
    border: Border,
    warning_color: Gradient,
) -> PanelResult {
    let total_opts_len = options
        .iter()
        .map(|opt| opt.chars().count() + 4)
        .sum::<usize>()
        + options.len().saturating_sub(1);

    let selected_opts: Vec<String> = options.iter().map(|o| format!("> {} <", o)).collect();
    let unselected_opts: Vec<String> = options.iter().map(|o| format!("  {}  ", o)).collect();

    let mut selected_idx: usize = 0;

    let mut dirty = true;

    loop {
        let (current_w, current_h) = Terminal::size();
        if current_w != canvas.width || current_h != canvas.height {
            canvas.resize(current_w, current_h);
            dirty = true;
        }

        if dirty {
            if current_w < min_w || current_h < min_h {
                if !crate::toosmall::run(
                    terminal,
                    canvas,
                    min_w,
                    min_h,
                    border,
                    warning_color.clone(),
                ) {
                    return PanelResult::Quit;
                }
                dirty = true;
                continue;
            }

            canvas.clean();

            let term_w = canvas.width;
            let term_h = canvas.height;

            let pad: u16 = 2;
            let mut err_box = Box::new(
                term_w,
                term_h,
                pad,
                Border::Heavy,
                Gradient::Solid(Color::Red),
                Gradient::Solid(Color::None),
                Modifier::Bold,
            );

            let inner_w = term_w.saturating_sub(pad * 2);
            let inner_h = term_h.saturating_sub(pad * 2);

            let msg_x = 2;
            let msg_y = 0;

            err_box.insert_text(
                msg,
                msg_x,
                msg_y,
                true,
                Gradient::Solid(Color::White),
                Gradient::Solid(Color::None),
                Modifier::None,
            );

            if !options.is_empty() {
                let options_y = inner_h.saturating_sub(1) as i16;
                let mut current_x = (inner_w.saturating_sub(total_opts_len as u16 + 2)) as i16;

                for i in 0..options.len() {
                    let is_selected = i == selected_idx;
                    let text = if is_selected {
                        &selected_opts[i]
                    } else {
                        &unselected_opts[i]
                    };

                    let (fg, bg, md) = if is_selected {
                        (
                            Gradient::Solid(Color::Black),
                            Gradient::Solid(Color::White),
                            Modifier::Bold,
                        )
                    } else {
                        (
                            Gradient::Solid(Color::White),
                            Gradient::Solid(Color::Red),
                            Modifier::None,
                        )
                    };

                    err_box.insert_text(text, current_x, options_y, false, fg, bg, md);
                    current_x += (text.chars().count() + 1) as i16;
                }
            }

            canvas.put_box(&err_box, 0, 0);
            canvas.render();
            dirty = false;
        }

        let key = terminal.read_key(Duration::from_millis(16));
        if key == Key::None {
            continue;
        }

        match key {
            Key::Left | Key::Up => {
                if selected_idx > 0 {
                    selected_idx -= 1;
                } else if !options.is_empty() {
                    selected_idx = options.len() - 1;
                }
                dirty = true;
            }
            Key::Right | Key::Down => {
                if selected_idx < options.len().saturating_sub(1) {
                    selected_idx += 1;
                } else {
                    selected_idx = 0;
                }
                dirty = true;
            }
            Key::Enter => return PanelResult::Ok(selected_idx),
            Key::Esc => return PanelResult::Cancel,
            Key::Char('q') | Key::Char('\x03') => return PanelResult::Quit,
            _ => {
                dirty = true;
            }
        }
    }
}
