use crate::{Border, Box, Canvas, Cell, Color, Key, Modifier, Style, Terminal};
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
                    fg: Color::Yellow,
                    bg: Color::None,
                    md: Modifier::Bold,
                },
            );

            let w_len = current_w.to_string().len().max(min_w.to_string().len());
            let h_len = current_h.to_string().len().max(min_h.to_string().len());

            let cw_str = format!("{:>width$}", current_w, width = w_len);
            let mw_str = format!("{:>width$}", min_w, width = w_len);
            let ch_str = format!("{:<width$}", current_h, width = h_len);
            let mh_str = format!("{:<width$}", min_h, width = h_len);

            let title_len = 18;
            let stats_len = 9 + w_len + 1 + h_len;
            let block_width = (stats_len as u16).max(title_len);
            let block_height = 4;

            let start_x = current_w.saturating_sub(block_width) / 2;
            let start_y = current_h.saturating_sub(block_height) / 2;

            let title_x = start_x + (block_width.saturating_sub(title_len)) / 2;
            warning_box.insert_text(
                "TERMINAL TOO SMALL",
                title_x as i16,
                start_y as i16,
                false,
                Style {
                    fg: Color::Yellow,
                    bg: Color::None,
                    md: Modifier::Bold,
                },
            );

            let base_style = Style {
                fg: Color::White,
                bg: Color::None,
                md: Modifier::None,
            };
            let dark_gray_bg = Style {
                fg: Color::White,
                bg: Color::DarkGray,
                md: Modifier::None,
            };
            let green_bg = Style {
                fg: Color::White,
                bg: Color::Green,
                md: Modifier::None,
            };
            let yellow_bg = Style {
                fg: Color::White,
                bg: Color::Yellow,
                md: Modifier::None,
            };
            let arrow_style = Style {
                fg: Color::White,
                bg: Color::None,
                md: Modifier::Bold,
            };

            let mut cur_x = start_x;
            let line1_y = start_y + 2;
            warning_box.insert_text("Current: ", cur_x as i16, line1_y as i16, false, base_style);
            cur_x += 9;

            let cw_style = if current_w >= min_w {
                green_bg
            } else {
                yellow_bg
            };
            warning_box.insert_text(&cw_str, cur_x as i16, line1_y as i16, false, cw_style);
            cur_x += w_len as u16;

            warning_box.insert_text("x", cur_x as i16, line1_y as i16, false, dark_gray_bg);
            cur_x += 1;

            let ch_style = if current_h >= min_h {
                green_bg
            } else {
                yellow_bg
            };
            warning_box.insert_text(&ch_str, cur_x as i16, line1_y as i16, false, ch_style);

            let mut cur_x = start_x;
            let line2_y = start_y + 3;
            warning_box.insert_text("Needed:  ", cur_x as i16, line2_y as i16, false, base_style);
            cur_x += 9;

            warning_box.insert_text(&mw_str, cur_x as i16, line2_y as i16, false, dark_gray_bg);
            cur_x += w_len as u16;

            warning_box.insert_text("x", cur_x as i16, line2_y as i16, false, dark_gray_bg);
            cur_x += 1;

            warning_box.insert_text(&mh_str, cur_x as i16, line2_y as i16, false, dark_gray_bg);

            if current_w < min_w && start_x >= 3 {
                let left_arr_x = start_x / 2;
                let right_arr_x = current_w.saturating_sub(1).saturating_sub(left_arr_x);

                warning_box.put_cell(
                    Cell {
                        c: '◀',
                        s: arrow_style,
                    },
                    left_arr_x,
                    line1_y,
                );
                warning_box.put_cell(
                    Cell {
                        c: '▶',
                        s: arrow_style,
                    },
                    right_arr_x,
                    line1_y,
                );
            }

            if current_h < min_h && start_y >= 3 {
                let center_x = current_w / 2;
                let top_arr_y = start_y / 2;
                let bot_arr_y = current_h.saturating_sub(1).saturating_sub(top_arr_y);

                warning_box.put_cell(
                    Cell {
                        c: '▲',
                        s: arrow_style,
                    },
                    center_x,
                    top_arr_y,
                );
                warning_box.put_cell(
                    Cell {
                        c: '▼',
                        s: arrow_style,
                    },
                    center_x,
                    bot_arr_y,
                );
            }

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
