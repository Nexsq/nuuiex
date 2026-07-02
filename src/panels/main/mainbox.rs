use super::ActivePanel;
use super::layout::{DECK_H, LIST_W, TABS_W};
use crate::{Border, Box, Color, Modifier, Style};

pub fn refresh(term_w: u16, term_h: u16, active: ActivePanel, main_buffer: &str) -> Box {
    let main_color = if active == ActivePanel::Main {
        Color::White
    } else {
        Color::Magenta
    };
    let main_border = if active == ActivePanel::Main {
        Border::Heavy
    } else {
        Border::Light
    };

    let mut main_box = Box::new(
        term_w.saturating_sub(TABS_W + LIST_W),
        term_h.saturating_sub(DECK_H),
        1,
        main_border,
        Style {
            fg: main_color,
            bg: Color::None,
            md: Modifier::None,
        },
    );

    main_box.insert_text(
        main_buffer,
        0,
        0,
        false,
        Style {
            fg: Color::White,
            bg: Color::None,
            md: Modifier::None,
        },
    );

    main_box
}
