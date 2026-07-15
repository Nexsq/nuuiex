use super::layout::{LIST_W, TABS_W};
use crate::{Box, Color, Gradient, Modifier, Style, conf::Config, theme::themecore::Theme};

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
            Gradient::Solid(Color::None),
            Gradient::Solid(Color::None),
            Modifier::None,
        );
    }

    let height = 1 + (tabs_num as u16) * 2;

    let mut tabs_box = Box::new(
        TABS_W,
        height,
        0,
        crate::Border::None,
        theme.tabs_box.clone(),
        Gradient::Solid(Color::None),
        Modifier::None,
    );

    let (tl, h, v, l_tee, bl) = match config.get_border() {
        crate::Border::Rounded => ('╭', '─', '│', '├', '╰'),
        crate::Border::Light => ('┌', '─', '│', '├', '└'),
        crate::Border::Heavy => ('┏', '━', '┃', '┣', '┗'),
        crate::Border::None => (' ', ' ', ' ', ' ', ' '),
    };

    let get_border_style = |x: usize| Style {
        fg: theme.tabs_box.color_at(x, TABS_W as usize),
        bg: Color::None,
        md: Modifier::None,
    };

    tabs_box.put_cell(crate::Cell::new(tl, get_border_style(0)), 0, 0);
    tabs_box.put_cell(crate::Cell::new(h, get_border_style(1)), 1, 0);

    let dark_gray = Gradient::Solid(Color::DarkGray);

    for i in 0..tabs_num {
        let y_num = 1 + i * 2;
        let y_sep = 2 + i * 2;

        let is_selected = i == current_tab;
        let is_running = running_macros[i].is_some();

        let (num_color, num_md) = if is_selected {
            (&theme.tab_selected, Modifier::Bold)
        } else if is_running {
            (&theme.tab_lazy, Modifier::Bold)
        } else {
            (&dark_gray, Modifier::Dim)
        };

        tabs_box.put_cell(crate::Cell::new(v, get_border_style(0)), 0, y_num as u16);
        tabs_box.put_cell(
            crate::Cell::new(
                char::from_digit((i + 1) as u32, 10).unwrap(),
                Style {
                    fg: num_color.color_at(1, TABS_W as usize),
                    bg: Color::None,
                    md: num_md,
                },
            ),
            1,
            y_num as u16,
        );

        if i < tabs_num - 1 {
            tabs_box.put_cell(
                crate::Cell::new(l_tee, get_border_style(0)),
                0,
                y_sep as u16,
            );
            tabs_box.put_cell(crate::Cell::new(h, get_border_style(1)), 1, y_sep as u16);
        }
    }

    tabs_box.put_cell(crate::Cell::new(bl, get_border_style(0)), 0, height - 1);
    tabs_box.put_cell(crate::Cell::new(h, get_border_style(1)), 1, height - 1);

    tabs_box
}

pub fn refresh_title(theme: &Theme, config: &Config, term_w: u16) -> Box {
    let header_h = theme.title.len().max(1) as u16;
    let width = if config.deck_mode == "title" {
        term_w
    } else {
        TABS_W + LIST_W
    };
    let mut title_box = Box::new(
        width,
        header_h,
        0,
        crate::Border::None,
        theme.title_box.clone(),
        Gradient::Solid(Color::None),
        Modifier::None,
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
                Gradient::Solid(*color),
                Gradient::Solid(Color::None),
                Modifier::None,
            );
            current_x += display_text.chars().count() as u16;
        }
    }
    title_box
}

pub fn refresh_deck(
    term_w: u16,
    deck_h: u16,
    theme: &Theme,
    config: &Config,
    keyvis: &crate::panels::widgets::keyvis::KeyvisState,
) -> Box {
    if config.deck_mode != "widget" {
        return Box::new(
            0,
            0,
            0,
            crate::Border::None,
            Gradient::Solid(Color::None),
            Gradient::Solid(Color::None),
            Modifier::None,
        );
    }

    let border = if config.deck_mode == "widget" && config.deck_widget == "keyvis" {
        crate::Border::None
    } else {
        config.get_border()
    };

    let mut deck_box = Box::new(
        term_w.saturating_sub(TABS_W + LIST_W - 1),
        deck_h,
        if border == crate::Border::None { 0 } else { 1 },
        border,
        theme.deck_box.clone(),
        Gradient::Solid(Color::None),
        Modifier::None,
    );

    if config.deck_widget == "monitor" {
        deck_box.insert_text(
            "monitor",
            0,
            0,
            false,
            theme.main_label.clone(),
            Gradient::Solid(Color::None),
            Modifier::Bold,
        );
    } else {
        crate::panels::widgets::keyvis::draw(keyvis, &mut deck_box, config, theme);
    }

    deck_box
}
