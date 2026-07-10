pub mod editor;
pub mod error;
pub mod main;
pub mod result;
pub mod settings;
pub mod toosmall;

use crate::conf::Config;
use crate::theme::themecore::Theme;
use crate::{Border, Box, Cell, Color, Modifier, Style};

pub fn apply_indicator(b: &mut Box, config: &Config, theme: &Theme, is_active: bool) {
    if !is_active {
        return;
    }

    match config.indicator_style.as_str() {
        "corner" => {
            if b.width > 0 {
                b.put_cell(
                    Cell {
                        c: '■',
                        s: Style {
                            fg: theme.selected_box,
                            bg: Color::None,
                            md: Modifier::None,
                        },
                    },
                    b.width - 1,
                    0,
                );
            }
        }
        "corners" => {
            let w = b.width;
            let h = b.height;

            if w < 8 || h < 4 {
                if w > 0 {
                    b.put_cell(
                        Cell {
                            c: '■',
                            s: Style {
                                fg: theme.selected_box,
                                bg: Color::None,
                                md: Modifier::None,
                            },
                        },
                        w - 1,
                        0,
                    );
                }
                return;
            }

            let chars = Border::Heavy.chars().unwrap();
            let s = Style {
                fg: theme.selected_box,
                bg: Color::None,
                md: Modifier::None,
            };

            b.put_cell(Cell { c: chars.tl, s }, 0, 0);
            b.put_cell(Cell { c: chars.v, s }, 0, 1);
            for i in 1..=3 {
                b.put_cell(Cell { c: chars.h, s }, i, 0);
            }

            b.put_cell(Cell { c: chars.tr, s }, w - 1, 0);
            b.put_cell(Cell { c: chars.v, s }, w - 1, 1);
            for i in 1..=3 {
                b.put_cell(Cell { c: chars.h, s }, w - 1 - i, 0);
            }

            b.put_cell(Cell { c: chars.bl, s }, 0, h - 1);
            b.put_cell(Cell { c: chars.v, s }, 0, h - 2);
            for i in 1..=3 {
                b.put_cell(Cell { c: chars.h, s }, i, h - 1);
            }

            b.put_cell(Cell { c: chars.br, s }, w - 1, h - 1);
            b.put_cell(Cell { c: chars.v, s }, w - 1, h - 2);
            for i in 1..=3 {
                b.put_cell(Cell { c: chars.h, s }, w - 1 - i, h - 1);
            }
        }
        _ => {}
    }
}
