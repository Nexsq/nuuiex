use super::layout::{LIST_W, TABS_W};
use crate::{Box, Color, Modifier, Style, conf::Config, theme::themecore::Theme};

pub fn refresh_tabs(
    theme: &Theme,
    config: &Config,
    current_tab: usize,
    running_macros: &[Option<std::path::PathBuf>; 6],
) -> Box {
    let tabs_num = config.tabs_num.clamp(1, 6);
    if tabs_num == 1 {
        return Box::new(
            0,
            0,
            0,
            crate::Border::None,
            Style {
                fg: Color::None,
                bg: Color::None,
                md: Modifier::None,
            },
        );
    }

    let height = 1 + (tabs_num as u16) * 2;

    let mut tabs_box = Box::new(
        TABS_W,
        height,
        0,
        crate::Border::None,
        Style {
            fg: theme.tabs_box,
            bg: Color::None,
            md: Modifier::None,
        },
    );

    let (tl, h, v, l_tee, bl) = match config.get_border() {
        crate::Border::Rounded => ('╭', '─', '│', '├', '╰'),
        crate::Border::Light => ('┌', '─', '│', '├', '└'),
        crate::Border::Heavy => ('┏', '━', '┃', '┣', '┗'),
        crate::Border::None => (' ', ' ', ' ', ' ', ' '),
    };

    let border_style = Style {
        fg: theme.tabs_box,
        bg: Color::None,
        md: Modifier::None,
    };

    tabs_box.put_cell(crate::Cell::new(tl, border_style), 0, 0);
    tabs_box.put_cell(crate::Cell::new(h, border_style), 1, 0);

    for i in 0..tabs_num {
        let y_num = 1 + i * 2;
        let y_sep = 2 + i * 2;

        let is_selected = i == current_tab;
        let is_running = running_macros[i].is_some();

        let (num_color, num_md) = if is_selected {
            (theme.tab_selected, Modifier::Bold)
        } else if is_running {
            (theme.tab_lazy, Modifier::Bold)
        } else {
            (Color::DarkGray, Modifier::Dim)
        };

        tabs_box.put_cell(crate::Cell::new(v, border_style), 0, y_num as u16);
        tabs_box.put_cell(
            crate::Cell::new(
                char::from_digit((i + 1) as u32, 10).unwrap(),
                Style {
                    fg: num_color,
                    bg: Color::None,
                    md: num_md,
                },
            ),
            1,
            y_num as u16,
        );

        if i < tabs_num - 1 {
            tabs_box.put_cell(crate::Cell::new(l_tee, border_style), 0, y_sep as u16);
            tabs_box.put_cell(crate::Cell::new(h, border_style), 1, y_sep as u16);
        }
    }

    tabs_box.put_cell(crate::Cell::new(bl, border_style), 0, height - 1);
    tabs_box.put_cell(crate::Cell::new(h, border_style), 1, height - 1);

    tabs_box
}

pub fn refresh_title(theme: &Theme) -> Box {
    let header_h = theme.title.len().max(1) as u16;
    let mut title_box = Box::new(
        TABS_W + LIST_W,
        header_h,
        0,
        crate::Border::None,
        Style {
            fg: theme.title_box,
            bg: Color::None,
            md: Modifier::None,
        },
    );

    let max_width = title_box.width;
    for (i, line) in theme.title.iter().enumerate() {
        let mut current_x = 0;
        for (text, color) in line {
            if current_x >= max_width {
                break;
            }
            let mut display_text = text.clone();
            let chars_count = display_text.chars().count() as u16;

            if current_x + chars_count > max_width {
                let allowed = max_width.saturating_sub(current_x);
                display_text = display_text.chars().take(allowed as usize).collect();
            }

            title_box.insert_text(
                &display_text,
                current_x as i16,
                i as i16,
                false,
                Style {
                    fg: *color,
                    bg: Color::None,
                    md: Modifier::None,
                },
            );
            current_x += display_text.chars().count() as u16;
        }
    }
    title_box
}

pub fn refresh_deck(term_w: u16, header_h: u16, theme: &Theme) -> Box {
    let mut deck_box = Box::new(
        term_w.saturating_sub(TABS_W + LIST_W - 1),
        header_h,
        0,
        crate::Border::Heavy,
        Style {
            fg: theme.deck_box,
            bg: Color::None,
            md: Modifier::None,
        },
    );
    deck_box.insert_text(
        "DECK",
        0,
        0,
        false,
        Style {
            fg: theme.main_label,
            bg: Color::None,
            md: Modifier::Bold,
        },
    );
    deck_box
}
