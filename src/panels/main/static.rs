use super::layout::{DECK_H, LIST_W, TABS_W, TITLE_H};
use crate::{Border, Box, Color, Modifier, Style};

pub fn refresh_tabs(term_h: u16) -> Box {
    let mut tabs_box = Box::new(
        TABS_W,
        term_h.saturating_sub(TITLE_H),
        1,
        Border::Rounded,
        Style {
            fg: Color::Cyan,
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

pub fn refresh_title() -> Box {
    let mut title_box = Box::new(
        TABS_W + LIST_W,
        TITLE_H,
        1,
        Border::Double,
        Style {
            fg: Color::Green,
            bg: Color::None,
            md: Modifier::None,
        },
    );
    title_box.insert_text(
        "TITLE",
        0,
        0,
        false,
        Style {
            fg: Color::Yellow,
            bg: Color::None,
            md: Modifier::Bold,
        },
    );
    title_box
}

pub fn refresh_deck(term_w: u16) -> Box {
    let mut deck_box = Box::new(
        term_w.saturating_sub(TABS_W + LIST_W),
        DECK_H,
        1,
        Border::Double,
        Style {
            fg: Color::Green,
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
            fg: Color::Yellow,
            bg: Color::None,
            md: Modifier::Bold,
        },
    );
    deck_box
}
