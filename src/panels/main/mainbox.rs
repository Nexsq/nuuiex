use super::ActivePanel;
use super::layout::{LIST_W, TABS_W};
use crate::{Box, Color, Modifier, Style, conf::Config, theme::themecore::Theme};

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
    let use_border_color = config.indicator_style == "border";

    let main_color = if is_active && use_border_color {
        theme.selected_box
    } else {
        theme.main_box
    };

    let mut main_box = Box::new(
        term_w.saturating_sub(TABS_W + LIST_W),
        term_h.saturating_sub(header_h),
        1,
        config.get_border(),
        Style {
            fg: main_color,
            bg: Color::None,
            md: Modifier::None,
        },
    );

    crate::panels::apply_indicator(&mut main_box, config, theme, is_active);

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
