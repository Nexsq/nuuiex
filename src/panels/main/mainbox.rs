use super::ActivePanel;
use super::layout::{LIST_W, TABS_W};
use crate::{Border, Box, Color, Modifier, Style, conf::Config, theme::themecore::Theme};

pub fn refresh(
    term_w: u16,
    term_h: u16,
    header_h: u16,
    active: ActivePanel,
    main_buffer: &str,
    config: &Config,
    theme: &Theme,
) -> Box {
    let is_active = active == ActivePanel::Main;
    let use_corner = config.indicator_style == "corner";

    let main_color = if is_active && !use_corner {
        theme.selected_box
    } else {
        theme.main_box
    };
    let main_border = if is_active && !use_corner {
        Border::Heavy
    } else {
        Border::Light
    };

    let mut main_box = Box::new(
        term_w.saturating_sub(TABS_W + LIST_W),
        term_h.saturating_sub(header_h),
        1,
        main_border,
        Style {
            fg: main_color,
            bg: Color::None,
            md: Modifier::None,
        },
    );

    if is_active && use_corner {
        if main_box.width > 0 {
            main_box.put_cell(
                crate::Cell {
                    c: '■',
                    s: Style {
                        fg: theme.selected_box,
                        bg: Color::None,
                        md: Modifier::None,
                    },
                },
                main_box.width - 1,
                0,
            );
        }
    }

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
