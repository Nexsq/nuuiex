use super::layout::{LIST_W, TABS_W};
use crate::{Box, Color, Modifier, Style, conf::Config, theme::themecore::Theme};

pub fn refresh_tabs(term_h: u16, header_h: u16, theme: &Theme, config: &Config) -> Box {
    let mut tabs_box = Box::new(
        TABS_W,
        term_h.saturating_sub(header_h),
        1,
        config.get_border(),
        Style {
            fg: theme.tabs_box,
            bg: Color::None,
            md: Modifier::None,
        },
    );
    tabs_box.insert_text(
        "t a b s",
        0,
        0,
        false,
        Style {
            fg: Color::White,
            bg: Color::None,
            md: Modifier::None,
        },
    );
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
        term_w.saturating_sub(TABS_W + LIST_W),
        header_h,
        0,
        crate::Border::None,
        Style {
            fg: theme.deck_box,
            bg: Color::None,
            md: Modifier::None,
        },
    );
    deck_box.insert_text(
        "DECK",
        1,
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
