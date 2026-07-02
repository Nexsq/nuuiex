pub mod layout;
pub mod list;
pub mod mainbox;
pub mod r#static;

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
    pub expanded_path: Vec<usize>,

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
                    fg: Color::None,
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
            expanded_path: Vec::new(),
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

        let (main_pos, list_pos, tabs_pos, title_pos, deck_pos) =
            layout::get_positions(term_w, term_h);

        self.main_x = main_pos.0;
        self.main_y = main_pos.1;
        self.list_x = list_pos.0;
        self.list_y = list_pos.1;
        self.tabs_x = tabs_pos.0;
        self.tabs_y = tabs_pos.1;
        self.title_x = title_pos.0;
        self.title_y = title_pos.1;
        self.deck_x = deck_pos.0;
        self.deck_y = deck_pos.1;

        self.refresh_all();
    }

    pub fn refresh_all(&mut self) {
        self.refresh_main();
        self.refresh_list();
        self.refresh_static_boxes();
    }

    pub fn refresh_main(&mut self) {
        self.main_box = mainbox::refresh(self.term_w, self.term_h, self.active, &self.main_buffer);
    }

    pub fn refresh_list(&mut self) {
        self.list_box = list::refresh(
            self.term_h,
            self.active,
            &self.library_tree,
            &self.expanded_path,
            &mut self.list_selected,
            &mut self.list_scroll,
        );
    }

    pub fn refresh_static_boxes(&mut self) {
        self.tabs_box = r#static::refresh_tabs(self.term_h);
        self.title_box = r#static::refresh_title();
        self.deck_box = r#static::refresh_deck(self.term_w);
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
        if self.active != ActivePanel::List {
            return;
        }

        let (_, empty_idx) = list::resolve_view(&self.expanded_path, &self.library_tree);

        if self.list_selected > 0 {
            self.list_selected -= 1;
            if empty_idx.is_some() {
                self.expanded_path.pop();
            }
        }
        self.refresh_list();
    }

    pub fn selection_down(&mut self) {
        if self.active != ActivePanel::List {
            return;
        }

        let (view_path, empty_idx) = list::resolve_view(&self.expanded_path, &self.library_tree);

        let mut parent_nodes = &self.library_tree;
        for &idx in &view_path {
            if let Some(MacroNode::Folder { children, .. }) = parent_nodes.get(idx) {
                parent_nodes = children;
            }
        }

        let max_idx = parent_nodes.len().saturating_sub(1);
        if self.list_selected < max_idx {
            self.list_selected += 1;
            if empty_idx.is_some() {
                self.expanded_path.pop();
            }
        }
        self.refresh_list();
    }

    pub fn handle_right_arrow(&mut self) {
        if self.active != ActivePanel::List {
            return;
        }

        let (view_path, empty_idx) = list::resolve_view(&self.expanded_path, &self.library_tree);

        if empty_idx.is_some() {
            return;
        }

        let mut current_nodes = &self.library_tree;
        for &idx in &view_path {
            if let Some(MacroNode::Folder { children, .. }) = current_nodes.get(idx) {
                current_nodes = children;
            }
        }

        if let Some(MacroNode::Folder { children, .. }) = current_nodes.get(self.list_selected) {
            self.expanded_path.push(self.list_selected);
            if !children.is_empty() {
                self.list_selected = 0;
                self.list_scroll = 0;
            }
            self.refresh_list();
        }
    }

    pub fn handle_left_arrow(&mut self) {
        if self.active != ActivePanel::List {
            return;
        }

        if let Some(previous_parent_idx) = self.expanded_path.pop() {
            self.list_selected = previous_parent_idx;
            self.list_scroll = 0;
            self.refresh_list();
        }
    }

    pub fn trigger_selected(&mut self) {
        if self.active != ActivePanel::List {
            return;
        }

        let (view_path, _) = list::resolve_view(&self.expanded_path, &self.library_tree);
        let mut parent_nodes = &self.library_tree;

        for &idx in &view_path {
            if let Some(MacroNode::Folder { children, .. }) = parent_nodes.get(idx) {
                parent_nodes = children;
            }
        }

        if let Some(node) = parent_nodes.get(self.list_selected) {
            match node {
                MacroNode::Script { path, .. } => {
                    self.main_buffer.clear();

                    match std::fs::read_to_string(path) {
                        Ok(contents) => {
                            self.main_buffer = contents;
                        }
                        Err(e) => {
                            self.main_buffer = format!("Error reading macro file:\n{}", e);
                        }
                    }

                    self.refresh_main();
                }
                _ => {}
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
