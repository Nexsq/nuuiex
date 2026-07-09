pub mod layout;
pub mod list;
pub mod mainbox;
pub mod r#static;

use std::path::PathBuf;

use crate::{
    Border, Box, Canvas, Color, Modifier, Style, conf::Config, editor::Editor, lib::MacroNode,
};

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

    pub editor: Editor,
    pub running_macro: Option<PathBuf>,
    pub main_box: Box,
    pub main_x: i16,
    pub main_y: i16,

    pub list_box: Box,
    pub list_x: i16,
    pub list_y: i16,
    pub library_tree: Vec<MacroNode>,
    pub library_root: PathBuf,
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
        library_tree: Vec<MacroNode>,
        library_root: PathBuf,
        config: &Config,
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
            library_root,
            expanded_path: Vec::new(),
            editor: Editor::new(),
            running_macro: None,
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

        view.auto_load();
        view.resize(term_w, term_h, config);
        view
    }

    pub fn auto_load(&mut self) {
        let (view_path, _) = list::resolve_view(&self.expanded_path, &self.library_tree);
        let mut parent_nodes = &self.library_tree;

        for &idx in &view_path {
            if let Some(MacroNode::Folder { children, .. }) = parent_nodes.get(idx) {
                parent_nodes = children;
            }
        }

        let mut to_load = None;
        let mut is_folder = false;

        if let Some(node) = parent_nodes.get(self.list_selected) {
            match node {
                MacroNode::Script { path, .. } => {
                    let rp = path
                        .strip_prefix(&self.library_root)
                        .unwrap_or(path)
                        .to_string_lossy()
                        .into_owned();
                    to_load = Some((path.clone(), rp));
                }
                MacroNode::Folder { .. } => {
                    is_folder = true;
                }
            }
        } else {
            is_folder = true;
        }

        if let Some((path, rp)) = to_load {
            if self.editor.file_path.as_deref() != Some(path.as_path()) {
                self.editor.load_file(path, false, rp);
            }
        } else if is_folder {
            self.editor.file_path = None;
            self.editor.rel_path.clear();
            self.editor.state.lines = vec![String::new()];
            self.editor.is_editing = false;
        }
    }

    pub fn resize(&mut self, term_w: u16, term_h: u16, config: &Config) {
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

        self.refresh_all(config);
    }

    pub fn refresh_all(&mut self, config: &Config) {
        self.refresh_main(config);
        self.refresh_list(config);
        self.refresh_static_boxes();
    }

    pub fn refresh_main(&mut self, config: &Config) {
        self.main_box = self.editor.render(
            self.term_w.saturating_sub(layout::TABS_W + layout::LIST_W),
            self.term_h.saturating_sub(layout::DECK_H),
            self.active == ActivePanel::Main,
            config,
        );
    }

    pub fn refresh_list(&mut self, config: &Config) {
        let active_path = self
            .running_macro
            .as_deref()
            .or(self.editor.file_path.as_deref());

        self.list_box = list::refresh(
            self.term_h,
            self.active,
            &self.library_tree,
            &self.expanded_path,
            &mut self.list_selected,
            &mut self.list_scroll,
            active_path,
            config,
        );
    }

    pub fn refresh_static_boxes(&mut self) {
        self.tabs_box = r#static::refresh_tabs(self.term_h);
        self.title_box = r#static::refresh_title();
        self.deck_box = r#static::refresh_deck(self.term_w);
    }

    pub fn toggle_focus(&mut self, config: &Config) {
        if self.active == ActivePanel::Main {
            self.active = ActivePanel::List;
            self.running_macro = None;
            if self.editor.is_editing {
                self.editor.save();
                self.editor.is_editing = false;
            }
            self.auto_load();
        } else {
            self.active = ActivePanel::Main;
            if self.editor.file_path.is_some() {
                self.editor.is_editing = true;
            }
        }
        self.refresh_main(config);
        self.refresh_list(config);
    }

    pub fn selection_up(&mut self, config: &Config) {
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
        self.auto_load();
        self.refresh_main(config);
        self.refresh_list(config);
    }

    pub fn selection_down(&mut self, config: &Config) {
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
        self.auto_load();
        self.refresh_main(config);
        self.refresh_list(config);
    }

    pub fn handle_right_arrow(&mut self, config: &Config) {
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

        let mut do_expand = false;
        let mut reset_scroll = false;

        if let Some(MacroNode::Folder { children, .. }) = current_nodes.get(self.list_selected) {
            do_expand = true;
            reset_scroll = !children.is_empty();
        }

        if do_expand {
            self.expanded_path.push(self.list_selected);
            if reset_scroll {
                self.list_selected = 0;
                self.list_scroll = 0;
            }
            self.auto_load();
            self.refresh_main(config);
            self.refresh_list(config);
        }
    }

    pub fn handle_left_arrow(&mut self, config: &Config) {
        if self.active != ActivePanel::List {
            return;
        }

        if let Some(previous_parent_idx) = self.expanded_path.pop() {
            self.list_selected = previous_parent_idx;
            self.list_scroll = 0;
            self.auto_load();
            self.refresh_main(config);
            self.refresh_list(config);
        }
    }

    pub fn trigger_selected(&mut self, config: &Config) {
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

        let mut should_switch = false;
        if let Some(node) = parent_nodes.get(self.list_selected) {
            if let MacroNode::Script { name, path } = node {
                if let Ok(source) = std::fs::read_to_string(path) {
                    let output = crate::engine::run(&source);
                    self.editor.state.lines = output;
                } else {
                    self.editor.state.lines = vec![format!("Failed to read script '{}'", name)];
                }

                self.editor.file_path = None;
                self.running_macro = Some(path.clone());
                should_switch = true;
            }
        }

        if should_switch {
            self.active = ActivePanel::Main;
            self.editor.is_editing = false;
            self.refresh_main(config);
            self.refresh_list(config);
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
