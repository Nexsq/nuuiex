use crate::{Border, Box, Canvas, Color, Modifier, Style, lib::MacroNode};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActivePanel {
    Main,
    List,
}

pub struct MainView {
    pub term_w: u16,
    pub term_h: u16,
    pub min_w: u16,
    pub min_h: u16,
    pub active: ActivePanel,
    pub list_selected: usize,
    pub list_scroll: usize,

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
        let dummy = || {
            Box::new(
                1,
                1,
                1,
                Border::None,
                Style {
                    fg: Color::White,
                    bg: Color::None,
                    md: Modifier::None,
                },
            )
        };

        let mut view = Self {
            term_w,
            term_h,
            min_w: 64,
            min_h: 16,
            active,
            list_selected: 0,
            list_scroll: 0,
            library_tree,
            main_buffer,
            main_box: dummy(),
            main_x: 0,
            main_y: 0,
            list_box: dummy(),
            list_x: 0,
            list_y: 0,
            tabs_box: dummy(),
            tabs_x: 0,
            tabs_y: 0,
            title_box: dummy(),
            title_x: 0,
            title_y: 0,
            deck_box: dummy(),
            deck_x: 0,
            deck_y: 0,
        };

        view.resize(term_w, term_h);
        view
    }

    pub fn resize(&mut self, term_w: u16, term_h: u16) {
        self.term_w = term_w;
        self.term_h = term_h;

        let tabs_w = 3;
        let list_w = 24;
        let title_h = 3;
        let deck_h = 3;

        self.main_x = (tabs_w + list_w) as i16;
        self.main_y = deck_h as i16;
        self.list_x = tabs_w as i16;
        self.list_y = title_h as i16;
        self.tabs_x = 0;
        self.tabs_y = title_h as i16;
        self.title_x = 0;
        self.title_y = 0;
        self.deck_x = (tabs_w + list_w) as i16;
        self.deck_y = 0;

        self.refresh_all();
    }

    pub fn refresh_all(&mut self) {
        self.refresh_main();
        self.refresh_list();
        self.refresh_static_boxes();
    }

    pub fn refresh_main(&mut self) {
        let deck_h = 3;
        let tabs_w = 3;
        let list_w = 24;

        let main_color = if self.active == ActivePanel::Main {
            Color::White
        } else {
            Color::Magenta
        };
        let main_border = if self.active == ActivePanel::Main {
            Border::Heavy
        } else {
            Border::Light
        };

        let mut main_box = Box::new(
            self.term_w.saturating_sub(tabs_w + list_w),
            self.term_h.saturating_sub(deck_h),
            1,
            main_border,
            Style {
                fg: main_color,
                bg: Color::None,
                md: Modifier::None,
            },
        );

        main_box.insert_text(
            &self.main_buffer,
            0,
            0,
            true,
            Style {
                fg: Color::White,
                bg: Color::None,
                md: Modifier::None,
            },
        );
        self.main_box = main_box;
    }

    pub fn refresh_list(&mut self) {
        let title_h = 3;
        let list_w = 24;
        let list_h = self.term_h.saturating_sub(title_h);

        let list_color = if self.active == ActivePanel::List {
            Color::White
        } else {
            Color::Blue
        };
        let list_border = if self.active == ActivePanel::List {
            Border::Heavy
        } else {
            Border::Light
        };

        let mut list_box = Box::new(
            list_w,
            list_h,
            1,
            list_border,
            Style {
                fg: list_color,
                bg: Color::None,
                md: Modifier::None,
            },
        );

        let visible_items = list_h.saturating_sub(2) as usize;

        let max_idx = self.library_tree.len().saturating_sub(1);
        self.list_selected = self.list_selected.min(max_idx);

        if self.list_selected < self.list_scroll {
            self.list_scroll = self.list_selected;
        } else if self.list_selected >= self.list_scroll + visible_items && visible_items > 0 {
            self.list_scroll = self.list_selected.saturating_sub(visible_items - 1);
        }

        let max_text_len = list_w.saturating_sub(2) as usize;

        for (i, node) in self
            .library_tree
            .iter()
            .enumerate()
            .skip(self.list_scroll)
            .take(visible_items)
        {
            let (prefix, normal_color) = match node {
                MacroNode::Folder { .. } => ("▪", Color::Blue),
                MacroNode::Script { .. } => ("▫", Color::Magenta),
            };

            let mut text = format!("{} {}", prefix, node.name());

            let char_count = text.chars().count();
            if char_count > max_text_len {
                text = text.chars().take(max_text_len).collect();
            } else {
                let spaces = " ".repeat(max_text_len - char_count - 2);
                text.push_str(&spaces);
            }

            let is_selected = (i == self.list_selected) && (self.active == ActivePanel::List);
            let (fg_color, bg_color) = if is_selected {
                (Color::White, normal_color)
            } else {
                (normal_color, Color::None)
            };

            list_box.insert_text(
                &text,
                1,
                (i - self.list_scroll) as i16,
                false,
                Style {
                    fg: fg_color,
                    bg: bg_color,
                    md: Modifier::None,
                },
            );
        }

        self.list_box = list_box;
    }

    pub fn refresh_static_boxes(&mut self) {
        let tabs_w = 3;
        let list_w = 24;
        let title_h = 3;
        let deck_h = 3;

        let mut tabs_box = Box::new(
            tabs_w,
            self.term_h.saturating_sub(title_h),
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
        self.tabs_box = tabs_box;

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
        self.title_box = title_box;

        let mut deck_box = Box::new(
            self.term_w.saturating_sub(tabs_w + list_w),
            deck_h,
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
        self.deck_box = deck_box;
    }

    pub fn toggle_focus(&mut self) {
        self.active = if self.active == ActivePanel::Main {
            ActivePanel::List
        } else {
            ActivePanel::Main
        };
        self.refresh_main();
        self.refresh_list();
    }

    pub fn selection_up(&mut self) {
        if self.active != ActivePanel::List || self.library_tree.is_empty() {
            return;
        }
        self.list_selected = self.list_selected.saturating_sub(1);
        self.refresh_list();
    }

    pub fn selection_down(&mut self) {
        if self.active != ActivePanel::List || self.library_tree.is_empty() {
            return;
        }
        let max_idx = self.library_tree.len().saturating_sub(1);
        if self.list_selected < max_idx {
            self.list_selected += 1;
        }
        self.refresh_list();
    }

    pub fn trigger_selected(&mut self) {
        if self.active != ActivePanel::List || self.library_tree.is_empty() {
            return;
        }

        let node_name = self.library_tree[self.list_selected].name().to_string();

        if !self.main_buffer.is_empty() {
            self.main_buffer.push('\n');
        }
        self.main_buffer.push_str(&node_name);

        self.refresh_main();
    }

    pub fn render(&self, canvas: &mut Canvas) {
        canvas.put_box(&self.main_box, self.main_x, self.main_y);
        canvas.put_box(&self.list_box, self.list_x, self.list_y);
        canvas.put_box(&self.tabs_box, self.tabs_x, self.tabs_y);
        canvas.put_box(&self.title_box, self.title_x, self.title_y);
        canvas.put_box(&self.deck_box, self.deck_x, self.deck_y);
    }
}
