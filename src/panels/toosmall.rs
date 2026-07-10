use crate::{Border, Box, Canvas, Color, Key, Modifier, Style, Terminal};
use std::time::Duration;

pub fn run(
    terminal: &Terminal,
    canvas: &mut Canvas,
    min_w: u16,
    min_h: u16,
    border: Border,
) -> bool {
    let mut dirty = true;

    loop {
        let (current_w, current_h) = Terminal::size();

        if current_w >= min_w && current_h >= min_h {
            return true;
        }

        if current_w != canvas.width || current_h != canvas.height {
            canvas.resize(current_w, current_h);
            dirty = true;
        }

        if dirty {
            canvas.clean();

            let mut warning_box = Box::new(
                current_w,
                current_h,
                0,
                border,
                Style {
                    fg: Color::Red,
                    bg: Color::None,
                    md: Modifier::Bold,
                },
            );

            let title = "WINDOW TOO SMALL";
            let stats = format!(
                "Current: {}x{}\nNeeded:  {}x{}",
                current_w, current_h, min_w, min_h
            );

            let stats_width = stats.lines().map(|l| l.chars().count()).max().unwrap_or(16) as u16;
            let block_width = stats_width.max(16);
            let block_height = 4;

            let start_x = (current_w.saturating_sub(block_width) / 2) as i16;
            let start_y = (current_h.saturating_sub(block_height) / 2) as i16;
            let title_x = start_x + ((block_width.saturating_sub(16)) / 2) as i16;

            warning_box.insert_text(
                title,
                title_x,
                start_y,
                false,
                Style {
                    fg: Color::Red,
                    bg: Color::None,
                    md: Modifier::Bold,
                },
            );

            warning_box.insert_text(
                &stats,
                start_x,
                start_y + 2,
                false,
                Style {
                    fg: Color::White,
                    bg: Color::None,
                    md: Modifier::None,
                },
            );

            canvas.put_box(&warning_box, 0, 0);
            canvas.render();
            dirty = false;
        }

        match terminal.read_key(Duration::from_millis(16)) {
            Key::None => continue,
            Key::Char('q') | Key::Char('\x03') => return false,
            _ => {
                dirty = true;
            }
        }
    }
}
