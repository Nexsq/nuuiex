use crate::{Box, Color, Gradient, Modifier, Style, conf::Config, theme::themecore::Theme};
use chrono::Local;

const DIGITS: [[&str; 3]; 11] = [
    ["▄▄▄", "█ █", "█▄█"],
    [" ▄▄", "  █", "  █"],
    ["▄▄▄", "▄▄█", "█▄▄"],
    ["▄▄▄", " ▄█", "▄▄█"],
    ["▄ ▄", "█▄█", "  █"],
    ["▄▄▄", "█▄▄", "▄▄█"],
    ["▄▄▄", "█▄▄", "█▄█"],
    ["▄▄▄", "  █", "  █"],
    ["▄▄▄", "█▄█", "█▄█"],
    ["▄▄▄", "█▄█", "▄▄█"],
    [" ", "▀", "▀"],
];

pub struct ClockState {
    pub last_time: String,
    pub last_date: String,
    pub last_term_w: u16,
    pub last_term_h: u16,
}

impl ClockState {
    pub fn new() -> Self {
        Self {
            last_time: String::new(),
            last_date: String::new(),
            last_term_w: 0,
            last_term_h: 0,
        }
    }

    pub fn tick(&mut self, term_w: u16, term_h: u16, config: &Config) -> bool {
        let now = Local::now();
        let is_12h = config.clock_format == "12h";

        let time_str = match (config.clock_seconds, is_12h) {
            (true, true) => now.format("%I:%M:%S %p").to_string(),
            (false, true) => now.format("%I:%M %p").to_string(),
            (true, false) => now.format("%H:%M:%S").to_string(),
            (false, false) => now.format("%H:%M").to_string(),
        };
        let date_str = if config.clock_date {
            match config.clock_date_style.as_str() {
                "eu" => now.format("%d/%m/%Y").to_string(),
                "us" => now.format("%m/%d/%Y").to_string(),
                "clean" => now.format("%d %m %Y").to_string(),
                "mon name" => now.format("%b %d, %Y").to_string(),
                "rfc 2822" => now.format("%a, %d %b %Y").to_string(),
                _ => String::new(),
            }
        } else {
            String::new()
        };

        if self.last_time != time_str
            || self.last_date != date_str
            || self.last_term_w != term_w
            || self.last_term_h != term_h
        {
            self.last_time = time_str;
            self.last_date = date_str;
            self.last_term_w = term_w;
            self.last_term_h = term_h;
            true
        } else {
            false
        }
    }
}

pub fn draw(state: &ClockState, b: &mut Box, config: &Config, theme: &Theme) {
    if b.width == 0 || b.height == 0 {
        return;
    }

    let time_str = &state.last_time;
    let date_str = &state.last_date;

    let is_big = config.clock_mode == "big";

    let mut time_w = 0;
    if is_big {
        let mut first = true;
        for c in time_str.chars() {
            if !first {
                time_w += 1;
            }
            first = false;
            match c {
                '0'..='9' => time_w += DIGITS[(c as u8 - b'0') as usize][0].chars().count(),
                ':' => time_w += DIGITS[10][0].chars().count(),
                ' ' => time_w += 0,
                _ => time_w += 1,
            }
        }
    } else {
        time_w = time_str.chars().count();
    }

    let date_w = date_str.chars().count();
    let spacing = if date_w > 0 { 2 } else { 0 };

    let total_w = time_w + spacing + date_w;
    let inner_w = b.width.saturating_sub(b.padding * 2);

    let start_x = match config.clock_position.as_str() {
        "mid" => inner_w.saturating_sub(total_w as u16) / 2,
        "right" => inner_w.saturating_sub(total_w as u16 + 2),
        _ => 2,
    };

    let inner_h = b.height.saturating_sub(b.padding * 2);
    let item_h = if is_big { 3 } else { 1 };
    let start_y = inner_h.saturating_sub(item_h) / 2;

    let (time_x, date_x) = if config.clock_position == "right" {
        (start_x + date_w as u16 + spacing as u16, start_x)
    } else {
        (start_x, start_x + time_w as u16 + spacing as u16)
    };

    if date_w > 0 {
        let date_y = if is_big { start_y + 2 } else { start_y };
        b.insert_text(
            date_str,
            date_x as i16,
            date_y as i16,
            false,
            theme.clock_date_color.clone(),
            Gradient::Solid(Color::None),
            Modifier::None,
        );
    }

    if is_big {
        let mut curr_x = time_x;
        for c in time_str.chars() {
            match c {
                '0'..='9' => {
                    let digit_idx = (c as u8 - b'0') as usize;
                    let lines = DIGITS[digit_idx];
                    let w = lines[0].chars().count() as u16;

                    for dy in 0..3 {
                        let part = lines[dy as usize];
                        for (dx, ch) in part.chars().enumerate() {
                            if ch == ' ' {
                                continue;
                            }
                            let local_x = curr_x + dx as u16;
                            let screen_x = local_x + b.padding;
                            let screen_y = start_y + dy + b.padding;

                            if screen_x < b.width && screen_y < b.height {
                                let fg = theme
                                    .clock_time_color
                                    .color_at((local_x - time_x) as usize, time_w);
                                b.put_cell(
                                    crate::Cell::new(
                                        ch,
                                        Style {
                                            fg,
                                            bg: Color::None,
                                            md: Modifier::None,
                                        },
                                    ),
                                    screen_x,
                                    screen_y,
                                );
                            }
                        }
                    }
                    curr_x += w + 1;
                }
                ':' => {
                    let lines = DIGITS[10];
                    let w = lines[0].chars().count() as u16;

                    for dy in 0..3 {
                        let part = lines[dy as usize];
                        for (dx, ch) in part.chars().enumerate() {
                            if ch == ' ' {
                                continue;
                            }
                            let local_x = curr_x + dx as u16;
                            let screen_x = local_x + b.padding;
                            let screen_y = start_y + dy + b.padding;

                            if screen_x < b.width && screen_y < b.height {
                                let fg = theme
                                    .clock_time_color
                                    .color_at((local_x - time_x) as usize, time_w);
                                b.put_cell(
                                    crate::Cell::new(
                                        ch,
                                        Style {
                                            fg,
                                            bg: Color::None,
                                            md: Modifier::None,
                                        },
                                    ),
                                    screen_x,
                                    screen_y,
                                );
                            }
                        }
                    }
                    curr_x += w + 1;
                }
                ' ' => {
                    curr_x += 1;
                }
                _ => {
                    let local_x = curr_x;
                    let screen_x = local_x + b.padding;
                    let screen_y = start_y + 2 + b.padding;

                    if screen_x < b.width && screen_y < b.height {
                        let fg = theme
                            .clock_time_color
                            .color_at((local_x - time_x) as usize, time_w);
                        b.put_cell(
                            crate::Cell::new(
                                c,
                                Style {
                                    fg,
                                    bg: Color::None,
                                    md: Modifier::None,
                                },
                            ),
                            screen_x,
                            screen_y,
                        );
                    }
                    curr_x += 2;
                }
            }
        }
    } else {
        b.insert_text(
            time_str,
            time_x as i16,
            start_y as i16,
            false,
            theme.clock_time_color.clone(),
            Gradient::Solid(Color::None),
            Modifier::None,
        );
    }
}
