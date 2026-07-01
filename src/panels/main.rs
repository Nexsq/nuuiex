use crate::{Border, Box, Canvas, Color, Modifier, Style, lib::MacroNode};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActivePanel {
    Main,
    List,
}

pub struct MainView {
    pub min_w: u16,
    pub min_h: u16,
    pub active: ActivePanel,
    pub main_box: Box,
    pub main_x: i16,
    pub main_y: i16,
    pub main_buffer: String,
    pub list_box: Box,
    pub list_x: i16,
    pub list_y: i16,
    pub library_tree: Vec<MacroNode>,
    pub tabs_box: Box,
    pub tabs_x: i16,
    pub tabs_y: i16,
    pub title_box: Box,
    pub title_x: i16,
    pub title_y: i16,
    pub deck_box: Box,
    pub deck_x: i16,
    pub deck_y: i16,
}

impl MainView {
    pub fn new(
        term_w: u16,
        term_h: u16,
        active: ActivePanel,
        main_buffer: String,
        library_tree: Vec<MacroNode>,
    ) -> Self {
        let min_w = 64;
        let min_h = 16;
        let list_w = 24;
        let tabs_w = 3;
        let title_h = 3;
        let deck_h = 3;

        let main_color = if active == ActivePanel::Main {
            Color::White
        } else {
            Color::Magenta
        };
        let list_color = if active == ActivePanel::List {
            Color::White
        } else {
            Color::Blue
        };
        let main_border = if active == ActivePanel::Main {
            Border::Heavy
        } else {
            Border::Light
        };
        let list_border = if active == ActivePanel::List {
            Border::Heavy
        } else {
            Border::Light
        };

        let mut main_box = Box::new(
            term_w.saturating_sub(tabs_w + list_w),
            term_h.saturating_sub(deck_h),
            1,
            main_border,
            Style {
                fg: main_color,
                bg: Color::None,
                md: Modifier::None,
            },
        );

        let mut list_box = Box::new(
            list_w,
            term_h.saturating_sub(title_h),
            1,
            list_border,
            Style {
                fg: list_color,
                bg: Color::None,
                md: Modifier::None,
            },
        );

        let mut tabs_box = Box::new(
            tabs_w,
            term_h.saturating_sub(title_h),
            1,
            Border::Rounded,
            Style {
                fg: Color::Cyan,
                bg: Color::None,
                md: Modifier::None,
            },
        );

        let mut title_box = Box::new(
            tabs_w + list_w,
            title_h,
            1,
            Border::Double,
            Style {
                fg: Color::Green,
                bg: Color::None,
                md: Modifier::None,
            },
        );

        let mut deck_box = Box::new(
            term_w.saturating_sub(tabs_w + list_w),
            deck_h,
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

        let text_style = Style {
            fg: Color::White,
            bg: Color::None,
            md: Modifier::None,
        };

        main_box.insert_text(&main_buffer, 0, 0, true, text_style);
        tabs_box.insert_text("t a b s", 0, 0, false, text_style);

        for (i, node) in library_tree.iter().enumerate() {
            if i as u16 >= list_box.height.saturating_sub(2) {
                break;
            }

            let (prefix, color) = match node {
                MacroNode::Folder { .. } => ("▪", Color::Blue),
                MacroNode::Script { .. } => ("▫", Color::Magenta),
            };

            let text = format!("{} {}", prefix, node.name());

            list_box.insert_text(
                &text,
                1,
                i as i16,
                false,
                Style {
                    fg: color,
                    bg: Color::None,
                    md: Modifier::None,
                },
            );
        }

        Self {
            min_w,
            min_h,
            active,
            main_box,
            main_x: (tabs_w + list_w) as i16,
            main_y: deck_h as i16,
            main_buffer,
            list_box,
            list_x: tabs_w as i16,
            list_y: title_h as i16,
            library_tree,
            tabs_box,
            tabs_x: 0,
            tabs_y: title_h as i16,
            title_box,
            title_x: 0,
            title_y: 0,
            deck_box,
            deck_x: (tabs_w + list_w) as i16,
            deck_y: 0,
        }
    }

    pub fn toggle_focus(&mut self) {
        match self.active {
            ActivePanel::Main => {
                self.active = ActivePanel::List;
                self.main_box.set_border_color(Color::Magenta);
                self.main_box.set_border_style(Border::Light);
                self.list_box.set_border_color(Color::White);
                self.list_box.set_border_style(Border::Heavy);
            }
            ActivePanel::List => {
                self.active = ActivePanel::Main;
                self.list_box.set_border_color(Color::Blue);
                self.list_box.set_border_style(Border::Light);
                self.main_box.set_border_color(Color::White);
                self.main_box.set_border_style(Border::Heavy);
            }
        }
    }

    pub fn render(&self, canvas: &mut Canvas) {
        canvas.put_box(&self.main_box, self.main_x, self.main_y);
        canvas.put_box(&self.list_box, self.list_x, self.list_y);
        canvas.put_box(&self.tabs_box, self.tabs_x, self.tabs_y);
        canvas.put_box(&self.title_box, self.title_x, self.title_y);
        canvas.put_box(&self.deck_box, self.deck_x, self.deck_y);
    }
}
