use std::thread;
use std::time::Duration;

use crate::{Border, Box, Canvas, Color, Key, Modifier, Style, Terminal};

pub fn error_box<F>(
    terminal: &Terminal,
    canvas: &mut Canvas,
    msg: &str,
    options: &[&str],
    width: u16,
    height: u16,
    min_w: u16,
    min_h: u16,
    mut draw_background: F,
) -> usize
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

    let selected_style = Style {
        fg: Color::Black,
        bg: Color::White,
        md: Modifier::Bold,
    };
    let unselected_style = Style {
        fg: Color::White,
        bg: Color::Red,
        md: Modifier::None,
    };
    let msg_style = Style {
        fg: Color::White,
        bg: Color::None,
        md: Modifier::None,
    };
    let border_style = Style {
        fg: Color::Red,
        bg: Color::None,
        md: Modifier::Bold,
    };

    let mut selected_idx: usize = 0;

    loop {
        let (current_w, current_h) = Terminal::size();

        if current_w != canvas.width || current_h != canvas.height {
            canvas.resize(current_w, current_h);
        }

        canvas.clean();
        draw_background(canvas, current_w, current_h);

        if current_w < min_w || current_h < min_h {
            canvas.render();
            match terminal.read_key() {
                Key::Char('q' | 'Q') | Key::Esc | Key::Char('\x03') => return 0,
                _ => {}
            }
            thread::sleep(Duration::from_millis(4));
            continue;
        }

        apply_dim(canvas);

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

        let mut err_box = Box::new(box_w, box_h, pad, Border::Heavy, border_style);

        let inner_w = box_w.saturating_sub(pad * 2);
        let inner_h = box_h.saturating_sub(pad * 2);

        let msg_x = (inner_w.saturating_sub(max_msg_len as u16) / 2) as i16;
        let msg_y = 0;

        err_box.insert_text(msg, msg_x, msg_y, true, msg_style);

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
                let style = if is_selected {
                    selected_style
                } else {
                    unselected_style
                };

                err_box.insert_text(text, current_x, options_y, false, style);
                current_x += (text.chars().count() + 1) as i16;
            }
        }

        canvas.put_box(&err_box, box_x, box_y);
        canvas.render();

        match terminal.read_key() {
            Key::Left | Key::Up => {
                if selected_idx > 0 {
                    selected_idx -= 1;
                } else if !options.is_empty() {
                    selected_idx = options.len() - 1;
                }
            }
            Key::Right | Key::Down => {
                if selected_idx < options.len().saturating_sub(1) {
                    selected_idx += 1;
                } else {
                    selected_idx = 0;
                }
            }
            Key::Enter => return selected_idx,
            Key::Char('q' | 'Q') | Key::Esc | Key::Char('\x03') => return 0,
            _ => {}
        }

        thread::sleep(Duration::from_millis(4));
    }
}

pub fn error_screen(
    terminal: &Terminal,
    canvas: &mut Canvas,
    msg: &str,
    options: &[&str],
    min_w: u16,
    min_h: u16,
) -> usize {
    let total_opts_len = options
        .iter()
        .map(|opt| opt.chars().count() + 4)
        .sum::<usize>()
        + options.len().saturating_sub(1);

    let selected_opts: Vec<String> = options.iter().map(|o| format!("> {} <", o)).collect();
    let unselected_opts: Vec<String> = options.iter().map(|o| format!("  {}  ", o)).collect();

    let selected_style = Style {
        fg: Color::Black,
        bg: Color::White,
        md: Modifier::Bold,
    };
    let unselected_style = Style {
        fg: Color::White,
        bg: Color::Red,
        md: Modifier::None,
    };
    let msg_style = Style {
        fg: Color::White,
        bg: Color::None,
        md: Modifier::None,
    };
    let border_style = Style {
        fg: Color::Red,
        bg: Color::None,
        md: Modifier::Bold,
    };

    let mut selected_idx: usize = 0;

    loop {
        let (current_w, current_h) = Terminal::size();
        if current_w != canvas.width || current_h != canvas.height {
            canvas.resize(current_w, current_h);
        }

        canvas.clean();

        if current_w < min_w || current_h < min_h {
            crate::toosmall::render(canvas, current_w, current_h);
            canvas.render();
            match terminal.read_key() {
                Key::Char('q' | 'Q') | Key::Esc | Key::Char('\x03') => return 0,
                _ => {}
            }
            thread::sleep(Duration::from_millis(4));
            continue;
        }

        let term_w = canvas.width;
        let term_h = canvas.height;

        let pad: u16 = 2;
        let mut err_box = Box::new(term_w, term_h, pad, Border::Heavy, border_style);

        let inner_w = term_w.saturating_sub(pad * 2);
        let inner_h = term_h.saturating_sub(pad * 2);

        let msg_x = 2;
        let msg_y = 0;

        err_box.insert_text(msg, msg_x, msg_y, true, msg_style);

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
                let style = if is_selected {
                    selected_style
                } else {
                    unselected_style
                };

                err_box.insert_text(text, current_x, options_y, false, style);
                current_x += (text.chars().count() + 1) as i16;
            }
        }

        canvas.put_box(&err_box, 0, 0);
        canvas.render();

        match terminal.read_key() {
            Key::Left | Key::Up => {
                if selected_idx > 0 {
                    selected_idx -= 1;
                } else if !options.is_empty() {
                    selected_idx = options.len() - 1;
                }
            }
            Key::Right | Key::Down => {
                if selected_idx < options.len().saturating_sub(1) {
                    selected_idx += 1;
                } else {
                    selected_idx = 0;
                }
            }
            Key::Enter => return selected_idx,
            Key::Char('q' | 'Q') | Key::Esc | Key::Char('\x03') => return 0,
            _ => {}
        }

        thread::sleep(Duration::from_millis(4));
    }
}

#[inline(always)]
fn apply_dim(canvas: &mut Canvas) {
    for cell in canvas.new.iter_mut() {
        cell.s.md = Modifier::Dim;
    }
}
