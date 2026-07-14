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
                            fg: theme.selected_box.color_at(0, 1),
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
                                fg: theme.selected_box.color_at(0, 1),
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
            let get_style = |x: u16| Style {
                fg: theme.selected_box.color_at(x as usize, w as usize),
                bg: Color::None,
                md: Modifier::None,
            };

            b.put_cell(
                Cell {
                    c: chars.tl,
                    s: get_style(0),
                },
                0,
                0,
            );
            b.put_cell(
                Cell {
                    c: chars.v,
                    s: get_style(0),
                },
                0,
                1,
            );
            for i in 1..=3 {
                b.put_cell(
                    Cell {
                        c: chars.h,
                        s: get_style(i),
                    },
                    i,
                    0,
                );
            }

            b.put_cell(
                Cell {
                    c: chars.tr,
                    s: get_style(w - 1),
                },
                w - 1,
                0,
            );
            b.put_cell(
                Cell {
                    c: chars.v,
                    s: get_style(w - 1),
                },
                w - 1,
                1,
            );
            for i in 1..=3 {
                b.put_cell(
                    Cell {
                        c: chars.h,
                        s: get_style(w - 1 - i),
                    },
                    w - 1 - i,
                    0,
                );
            }

            b.put_cell(
                Cell {
                    c: chars.bl,
                    s: get_style(0),
                },
                0,
                h - 1,
            );
            b.put_cell(
                Cell {
                    c: chars.v,
                    s: get_style(0),
                },
                0,
                h - 2,
            );
            for i in 1..=3 {
                b.put_cell(
                    Cell {
                        c: chars.h,
                        s: get_style(i),
                    },
                    i,
                    h - 1,
                );
            }

            b.put_cell(
                Cell {
                    c: chars.br,
                    s: get_style(w - 1),
                },
                w - 1,
                h - 1,
            );
            b.put_cell(
                Cell {
                    c: chars.v,
                    s: get_style(w - 1),
                },
                w - 1,
                h - 2,
            );
            for i in 1..=3 {
                b.put_cell(
                    Cell {
                        c: chars.h,
                        s: get_style(w - 1 - i),
                    },
                    w - 1 - i,
                    h - 1,
                );
            }
        }
        _ => {}
    }
}
