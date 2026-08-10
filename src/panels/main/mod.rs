pub mod layout;
pub mod list;
pub mod r#static;

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use crate::{
    Border, Box, Canvas, Color, Gradient, Key, Modifier, conf::Config, editor::Editor,
    lib::MacroNode, theme::themecore::Theme,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActivePanel {
    Main,
    List,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ListInputMode {
    None,
    CreatingFile(String),
    CreatingFolder(String),
    Renaming(String),
}

pub struct MainView {
    pub term_w: u16,
    pub term_h: u16,
    pub min_w: u16,
    pub min_h: u16,
    pub active: ActivePanel,
    pub list_selected: [usize; 6],
    pub list_scroll: [usize; 6],

    pub current_tab: usize,
    pub editors: [Editor; 6],
    pub running_macros: [Option<PathBuf>; 6],
    pub cancellation_tokens: [Option<Arc<AtomicBool>>; 6],
    pub macro_focus_tokens: [Option<Arc<AtomicBool>>; 6],
    pub display_sizes: [Option<Arc<AtomicU32>>; 6],
    pub macro_start_times: [Option<std::time::Instant>; 6],
    pub main_box: Box,
    pub main_x: i16,
    pub main_y: i16,

    pub list_box: Box,
    pub list_x: i16,
    pub list_y: i16,
    pub library_tree: Vec<MacroNode>,
    pub library_root: PathBuf,
    pub expanded_path: [Vec<usize>; 6],

    pub tabs_box: Box,
    pub tabs_x: i16,
    pub tabs_y: i16,

    pub title_box: Box,
    pub title_x: i16,
    pub title_y: i16,

    pub deck_box: Box,
    pub deck_x: i16,
    pub deck_y: i16,

    pub keyvis: crate::panels::widgets::keyvis::KeyvisState,
    pub monitor: crate::panels::widgets::monitor::MonitorState,
    pub clock: crate::panels::widgets::clock::ClockState,
    pub macrostats: crate::panels::widgets::macrostats::MacrostatsState,

    pub theme: Theme,
    pub list_input: ListInputMode,
    pub list_input_cursor: usize,
}

impl MainView {
    pub fn new(
        term_w: u16,
        term_h: u16,
        active: ActivePanel,
        library_tree: Vec<MacroNode>,
        library_root: PathBuf,
        config: &Config,
        theme: Theme,
    ) -> Self {
        let dummy = || {
            Box::new(
                1,
                1,
                1,
                Border::None,
                Gradient::Solid(Color::None),
                Gradient::Solid(Color::None),
                Modifier::None,
            )
        };

        let editors = std::array::from_fn(|_| Editor::new());
        let running_macros = std::array::from_fn(|_| None);
        let cancellation_tokens = std::array::from_fn(|_| None);
        let macro_focus_tokens = std::array::from_fn(|_| None);
        let display_sizes = std::array::from_fn(|_| None);
        let macro_start_times = std::array::from_fn(|_| None);

        let mut view = Self {
            term_w,
            term_h,
            min_w: 64,
            min_h: 16,
            active,
            list_selected: [0; 6],
            list_scroll: [0; 6],
            library_tree,
            library_root,
            expanded_path: std::array::from_fn(|_| Vec::new()),
            current_tab: 0,
            editors,
            running_macros,
            cancellation_tokens,
            macro_focus_tokens,
            display_sizes,
            macro_start_times,
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
            keyvis: crate::panels::widgets::keyvis::KeyvisState::new(),
            monitor: crate::panels::widgets::monitor::MonitorState::new(),
            clock: crate::panels::widgets::clock::ClockState::new(),
            macrostats: crate::panels::widgets::macrostats::MacrostatsState::new(),
            theme,
            list_input: ListInputMode::None,
            list_input_cursor: 0,
        };

        view.update_min_sizes(config);
        view.auto_load();
        view.resize(term_w, term_h, config);
        view
    }

    pub fn get_layout_heights(&self, config: &Config) -> (u16, u16) {
        let header_h = self.theme.title.len().max(1) as u16;
        let deck_h = if config.deck_mode == "widget" {
            match config.deck_widget.as_str() {
                "keyvis" => config.keyvis_height as u16,
                "monitor" | "clock" | "macrostats" => 3,
                _ => header_h,
            }
        } else if config.deck_mode == "none" {
            0
        } else {
            header_h
        };
        (header_h, deck_h)
    }

    pub fn draw_background(
        &mut self,
        cvs: &mut Canvas,
        w: u16,
        h: u16,
        k: &Key,
        config: &Config,
    ) -> bool {
        cvs.clean();
        let mut anim = false;
        if config.deck_mode == "widget" {
            if config.deck_widget == "keyvis" {
                if *k != Key::None {
                    self.keyvis
                        .push_key(k, config.keyvis_force, config.keyvis_spread);
                }
                if self.keyvis.tick(
                    config.keyvis_gravity,
                    config.keyvis_steps,
                    config.keyvis_tension,
                ) {
                    self.refresh_static_boxes(config);
                    anim = true;
                }
            } else if config.deck_widget == "monitor" {
                if self.monitor.tick(w, h) {
                    self.refresh_static_boxes(config);
                    anim = true;
                }
            } else if config.deck_widget == "clock" {
                if self.clock.tick(w, h, config) {
                    self.refresh_static_boxes(config);
                    anim = true;
                }
            }
        }
        if w != self.term_w || h != self.term_h {
            if w >= self.min_w && h >= self.min_h {
                self.resize(w, h, config);
            }
        }
        self.render(cvs);
        anim
    }

    pub fn update_min_sizes(&mut self, config: &Config) {
        let (_, deck_h) = self.get_layout_heights(config);
        self.min_h = 13 + deck_h;
        self.min_w = (config.lib_width as u16 + layout::TABS_W + 36).max(64);
    }

    pub fn reload_library_tree(&mut self, config: &Config) {
        let mut saved_expanded_paths: [Vec<std::path::PathBuf>; 6] =
            std::array::from_fn(|_| Vec::new());
        let mut saved_selected_paths: [Option<std::path::PathBuf>; 6] =
            std::array::from_fn(|_| None);

        for i in 0..6 {
            let mut current = &self.library_tree;
            for &idx in &self.expanded_path[i] {
                if let Some(crate::lib::MacroNode::Folder { path, children, .. }) = current.get(idx)
                {
                    saved_expanded_paths[i].push(path.clone());
                    current = children;
                } else {
                    break;
                }
            }
            if let Some(node) = current.get(self.list_selected[i]) {
                saved_selected_paths[i] = Some(node.path().to_path_buf());
            }
        }

        if let Ok(l) = crate::lib::init(&config.lib_sorting) {
            self.library_tree = l.tree;
            self.library_root = l.root_path;

            for i in 0..6 {
                let old_selected = self.list_selected[i];
                self.expanded_path[i].clear();
                let mut current = &self.library_tree;

                for target_path in &saved_expanded_paths[i] {
                    if let Some(idx) = current.iter().position(|n| n.path() == target_path) {
                        self.expanded_path[i].push(idx);
                        if let crate::lib::MacroNode::Folder { children, .. } = &current[idx] {
                            current = children;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }

                if let Some(target_sel) = &saved_selected_paths[i] {
                    if let Some(idx) = current.iter().position(|n| n.path() == target_sel) {
                        self.list_selected[i] = idx;
                    } else {
                        self.list_selected[i] = old_selected;
                    }
                } else {
                    self.list_selected[i] = old_selected;
                }
            }
        }
    }

    pub fn get_selected_node(&self) -> Option<&MacroNode> {
        let mut current = &self.library_tree;
        for &idx in &self.expanded_path[self.current_tab] {
            if let Some(MacroNode::Folder { children, .. }) = current.get(idx) {
                current = children;
            } else {
                return None;
            }
        }
        current.get(self.list_selected[self.current_tab])
    }

    pub fn clear_editor_for_path(&mut self, path: &std::path::Path) {
        for i in 0..6 {
            if self.editors[i].file_path.as_deref() == Some(path)
                || self.running_macros[i].as_deref() == Some(path)
                || self.editors[i].last_edited_path.as_deref() == Some(path)
            {
                if let Some(token) = self.cancellation_tokens[i].take() {
                    token.store(true, Ordering::SeqCst);
                }
                self.macro_focus_tokens[i] = None;
                self.display_sizes[i] = None;
                self.macro_start_times[i] = None;
                self.editors[i].file_path = None;
                self.editors[i].last_file_path = None;
                self.editors[i].saved_state = None;
                self.editors[i].saved_folded_lines.clear();
                self.editors[i].last_edited_path = None;
                self.running_macros[i] = None;
                self.editors[i].process_rx = None;
                self.editors[i].is_output = false;
                self.editors[i].state.lines = vec![String::new()];
                self.editors[i].folded_lines.clear();
                self.editors[i].error_count = 0;
                self.editors[i].error_lines.clear();
                self.editors[i].defined_functions.clear();

                self.editors[i].search_query.clear();
                self.editors[i].last_search.clear();
                self.editors[i].undo_stack.clear();
                self.editors[i].redo_stack.clear();
                self.editors[i].last_edit_pos = None;
            }
        }
    }

    pub fn auto_load(&mut self) {
        if self.running_macros[self.current_tab].is_some() {
            return;
        }

        if self.editors[self.current_tab].is_output && self.active == ActivePanel::Main {
            return;
        }

        let mut to_load = None;
        let mut is_folder = false;

        if let Some(node) = self.get_selected_node() {
            match node {
                MacroNode::Script { path, .. } => {
                    let rp = path
                        .strip_prefix(&self.library_root)
                        .unwrap_or(path.as_path())
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

        let should_edit = self.active == ActivePanel::Main;
        let editor = &mut self.editors[self.current_tab];

        let changing_view = match &to_load {
            Some((path, _)) => editor.file_path.as_deref() != Some(path.as_path()),
            _ => editor.file_path.is_some(),
        };

        if changing_view {
            if let (Some(fp), Some(lep)) = (&editor.file_path, &editor.last_edited_path) {
                if fp == lep {
                    editor.saved_state = Some(editor.state.clone());
                    editor.saved_scroll_x = editor.scroll_x;
                    editor.saved_scroll_y = editor.scroll_y;
                    editor.saved_folded_lines = editor.folded_lines.clone();
                }
            }
        }

        if let Some((path, rp)) = to_load {
            if changing_view {
                editor.load_file(path, should_edit, rp);
            } else if should_edit && !editor.is_editing {
                editor.is_editing = true;
                if editor.last_edited_path.as_deref() != Some(path.as_path()) {
                    editor.undo_stack.clear();
                    editor.redo_stack.clear();
                    editor.last_edit_pos = None;
                    editor.search_query.clear();
                    editor.last_search.clear();
                    editor.saved_state = None;
                    editor.saved_folded_lines.clear();
                    editor.last_edited_path = Some(path.clone());
                }

                editor.refresh_analysis(true);
            }
        } else if is_folder {
            if changing_view {
                editor.file_path = None;
                editor.rel_path.clear();
                editor.state.lines = vec![String::new()];
                editor.folded_lines.clear();
                editor.is_editing = false;
                editor.error_count = 0;
                editor.error_lines.clear();
                editor.defined_functions.clear();

                editor.scroll_x = 0;
                editor.scroll_y = 0;
                editor.state.cursor_x = 0;
                editor.state.cursor_y = 0;
                editor.state.selection_start = None;
            }
        }
    }

    pub fn switch_tab(&mut self, tab: usize, config: &Config) {
        if tab >= config.tabs_num.clamp(1, 6) || tab == self.current_tab {
            return;
        }
        if self.editors[self.current_tab].is_editing {
            self.editors[self.current_tab].save();
        }
        self.current_tab = tab;
        self.auto_load();
        self.refresh_all(config);
    }

    pub fn resize(&mut self, term_w: u16, term_h: u16, config: &Config) {
        self.term_w = term_w;
        self.term_h = term_h;
        self.update_min_sizes(config);

        let (header_h, deck_h) = self.get_layout_heights(config);

        let (main_pos, list_pos, tabs_pos, title_pos, deck_pos) = layout::get_positions(
            term_w,
            term_h,
            header_h,
            deck_h,
            config.tabs_num,
            &config.deck_mode,
            config.lib_width as u16,
        );

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

        let main_w = term_w.saturating_sub(layout::TABS_W + config.lib_width as u16 - 1);
        let main_h = term_h.saturating_sub(self.main_y as u16);
        let inner_w = main_w.saturating_sub(2) as u32;
        let inner_h = main_h.saturating_sub(2) as u32;
        let size_val = (inner_w << 16) | inner_h;

        for i in 0..6 {
            if let Some(size_arc) = &self.display_sizes[i] {
                size_arc.store(size_val, Ordering::Relaxed);
            }
        }

        self.refresh_all(config);
    }

    pub fn refresh_all(&mut self, config: &Config) {
        self.refresh_main(config);
        self.refresh_list(config);
        self.refresh_static_boxes(config);
    }

    pub fn refresh_main(&mut self, config: &Config) {
        let main_h = self.term_h.saturating_sub(self.main_y as u16);
        self.main_box = self.editors[self.current_tab].render(
            self.term_w
                .saturating_sub(layout::TABS_W + config.lib_width as u16 - 1),
            main_h,
            self.active == ActivePanel::Main,
            config,
            &self.theme,
        );
    }

    pub fn refresh_list(&mut self, config: &Config) {
        let header_h = self.theme.title.len().max(1) as u16;
        let active_path = self.running_macros[self.current_tab]
            .as_deref()
            .or(self.editors[self.current_tab].file_path.as_deref());

        let is_dirty = self.editors[self.current_tab].is_dirty();

        self.list_box = list::refresh(
            self.term_h,
            header_h,
            self.active,
            &self.library_tree,
            &self.expanded_path[self.current_tab],
            &mut self.list_selected[self.current_tab],
            &mut self.list_scroll[self.current_tab],
            active_path,
            is_dirty,
            config,
            &self.theme,
            &self.list_input,
            self.list_input_cursor,
        );
    }

    pub fn get_macrostats_info(&self) -> crate::panels::widgets::macrostats::MacroInfo {
        use crate::panels::widgets::macrostats::MacroInfo;
        if self.active == ActivePanel::List {
            if let Some(node) = self.get_selected_node() {
                if let MacroNode::Script { name, path } = node {
                    let is_running = self
                        .running_macros
                        .iter()
                        .any(|m| m.as_deref() == Some(path.as_path()));
                    return MacroInfo::Library {
                        name: name.clone(),
                        path: path.clone(),
                        is_running,
                    };
                }
            }
            MacroInfo::None
        } else {
            let tab = self.current_tab;
            if let Some(path) = &self.running_macros[tab] {
                let name = path
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                let start_time =
                    self.macro_start_times[tab].unwrap_or_else(std::time::Instant::now);
                MacroInfo::Running {
                    name,
                    start_time,
                    cpu_usage: f32::from_bits(
                        self.monitor
                            .process_cpu
                            .load(std::sync::atomic::Ordering::Relaxed),
                    )
                    .min(100.0),
                }
            } else if self.editors[tab].is_editing {
                let editor = &self.editors[tab];
                let name = editor
                    .file_path
                    .as_ref()
                    .and_then(|p| p.file_stem())
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Unsaved".to_string());
                let lines = editor.state.lines.len();
                let loc = editor
                    .state
                    .lines
                    .iter()
                    .filter(|l| !l.trim().is_empty())
                    .count();
                let errors = editor.error_count;
                MacroInfo::Editing {
                    name,
                    path: editor.file_path.clone(),
                    lines,
                    loc,
                    errors,
                }
            } else {
                MacroInfo::None
            }
        }
    }

    pub fn refresh_static_boxes(&mut self, config: &Config) {
        let (_, deck_h) = self.get_layout_heights(config);

        self.monitor.set_active(
            config.deck_mode == "widget"
                && (config.deck_widget == "monitor" || config.deck_widget == "macrostats"),
        );

        let macro_info = self.get_macrostats_info();

        self.tabs_box =
            r#static::refresh_tabs(&self.theme, config, self.current_tab, &self.running_macros);
        self.title_box = r#static::refresh_title(&self.theme, config, self.term_w);
        self.deck_box = r#static::refresh_deck(
            self.term_w,
            self.term_h,
            deck_h,
            &self.theme,
            config,
            &self.keyvis,
            &self.monitor,
            &self.clock,
            &self.macrostats,
            &macro_info,
        );
    }

    pub fn toggle_focus(&mut self, config: &Config) {
        let editor = &mut self.editors[self.current_tab];
        if self.active == ActivePanel::Main {
            self.active = ActivePanel::List;
            if editor.is_editing {
                editor.save();
                editor.is_editing = false;
            }
            self.auto_load();
        } else {
            self.active = ActivePanel::Main;
            self.list_input = ListInputMode::None;
            self.auto_load();
        }
        self.refresh_main(config);
        self.refresh_list(config);
    }

    pub fn selection_up(&mut self, config: &Config) {
        if self.active != ActivePanel::List {
            return;
        }

        let (_, empty_idx) = list::resolve_view(
            &self.expanded_path[self.current_tab],
            &self.library_tree,
            false,
        );

        if self.list_selected[self.current_tab] > 0 {
            self.list_selected[self.current_tab] -= 1;
            if empty_idx.is_some() {
                self.expanded_path[self.current_tab].pop();
            }
        }
        self.auto_load();
        self.refresh_list(config);
        self.refresh_main(config);
    }

    pub fn selection_down(&mut self, config: &Config) {
        if self.active != ActivePanel::List {
            return;
        }

        let (view_path, empty_idx) = list::resolve_view(
            &self.expanded_path[self.current_tab],
            &self.library_tree,
            false,
        );

        let mut parent_nodes = &self.library_tree;
        for &idx in &view_path {
            if let Some(MacroNode::Folder { children, .. }) = parent_nodes.get(idx) {
                parent_nodes = children;
            }
        }

        let max_idx = parent_nodes.len().saturating_sub(1);
        if self.list_selected[self.current_tab] < max_idx {
            self.list_selected[self.current_tab] += 1;
            if empty_idx.is_some() {
                self.expanded_path[self.current_tab].pop();
            }
        }
        self.auto_load();
        self.refresh_list(config);
        self.refresh_main(config);
    }

    pub fn move_selected_up(&mut self, config: &Config) {
        if config.lib_sorting != "custom" {
            return;
        }
        if self.list_selected[self.current_tab] > 0 {
            let (view_path, _) = list::resolve_view(
                &self.expanded_path[self.current_tab],
                &self.library_tree,
                false,
            );
            self.swap_items(
                &view_path,
                self.list_selected[self.current_tab],
                self.list_selected[self.current_tab] - 1,
            );
            self.list_selected[self.current_tab] -= 1;
            self.save_custom_order();
            self.refresh_list(config);
        }
    }

    pub fn move_selected_down(&mut self, config: &Config) {
        if config.lib_sorting != "custom" {
            return;
        }
        let (view_path, _) = list::resolve_view(
            &self.expanded_path[self.current_tab],
            &self.library_tree,
            false,
        );

        let mut current = &self.library_tree;
        for &idx in &view_path {
            if let Some(MacroNode::Folder { children, .. }) = current.get(idx) {
                current = children;
            }
        }

        if self.list_selected[self.current_tab] + 1 < current.len() {
            self.swap_items(
                &view_path,
                self.list_selected[self.current_tab],
                self.list_selected[self.current_tab] + 1,
            );
            self.list_selected[self.current_tab] += 1;
            self.save_custom_order();
            self.refresh_list(config);
        }
    }

    fn swap_items(&mut self, view_path: &[usize], idx1: usize, idx2: usize) {
        let mut current = &mut self.library_tree;
        for &idx in view_path {
            let temp = current;
            if idx < temp.len() {
                if let MacroNode::Folder { children, .. } = &mut temp[idx] {
                    current = children;
                    continue;
                }
            }
            return;
        }

        if idx1 < current.len() && idx2 < current.len() {
            current.swap(idx1, idx2);
        }
    }

    pub fn save_custom_order(&self) {
        let mut order = Vec::new();
        self.collect_paths(&self.library_tree, &mut order);
        crate::lib::save_custom_order(&order);
    }

    pub fn update_paths_after_move(
        &mut self,
        old_path: &std::path::Path,
        new_path: &std::path::Path,
        config: &Config,
    ) {
        crate::lib::rename_macrodata(old_path, new_path);

        if config.lib_sorting == "custom" {
            let mut order = crate::lib::load_custom_order();
            let old_rel = old_path
                .strip_prefix(&self.library_root)
                .unwrap()
                .to_string_lossy()
                .to_string();
            let new_rel = new_path
                .strip_prefix(&self.library_root)
                .unwrap()
                .to_string_lossy()
                .to_string();
            for item in &mut order {
                if item == &old_rel {
                    *item = new_rel.clone();
                } else if item.starts_with(&(old_rel.clone() + "/"))
                    || item.starts_with(&(old_rel.clone() + "\\"))
                {
                    *item = item.replacen(&old_rel, &new_rel, 1);
                }
            }
            crate::lib::save_custom_order(&order);
        }

        for i in 0..6 {
            if let Some(fp) = self.editors[i].file_path.clone() {
                if fp == old_path || fp.starts_with(old_path) {
                    let suffix = fp.strip_prefix(old_path).unwrap();
                    let new_fp = new_path.join(suffix);
                    self.editors[i].file_path = Some(new_fp.clone());
                    self.editors[i].rel_path = new_fp
                        .strip_prefix(&self.library_root)
                        .unwrap_or(new_fp.as_path())
                        .to_string_lossy()
                        .into_owned();
                }
            }
            if let Some(rm) = self.running_macros[i].clone() {
                if rm == old_path || rm.starts_with(old_path) {
                    let suffix = rm.strip_prefix(old_path).unwrap();
                    let new_rm = new_path.join(suffix);
                    self.running_macros[i] = Some(new_rm);
                }
            }
        }
    }

    pub fn move_selected_out(&mut self, config: &Config) {
        if self.active != ActivePanel::List {
            return;
        }

        if self.expanded_path[self.current_tab].is_empty() {
            return;
        }

        let node = match self.get_selected_node().cloned() {
            Some(n) => n,
            _ => return,
        };

        let old_path = node.path().to_path_buf();
        let parent_dir = match old_path.parent() {
            Some(p) => p,
            _ => return,
        };
        let grand_parent_dir = match parent_dir.parent() {
            Some(p) => p,
            _ => return,
        };

        let name = match old_path.file_name() {
            Some(n) => n,
            _ => return,
        };
        let new_path = grand_parent_dir.join(name);

        if new_path.exists() {
            return;
        }

        if std::fs::rename(&old_path, &new_path).is_ok() {
            self.update_paths_after_move(&old_path, &new_path, config);
            self.reload_library_tree(config);

            self.expanded_path[self.current_tab].pop();

            let mut current = &self.library_tree;
            let (view_path, _) = crate::panels::main::list::resolve_view(
                &self.expanded_path[self.current_tab],
                &self.library_tree,
                false,
            );
            for &idx in &view_path {
                if let Some(crate::lib::MacroNode::Folder { children, .. }) = current.get(idx) {
                    current = children;
                }
            }

            let is_folder = matches!(node, crate::lib::MacroNode::Folder { .. });
            if let Some(idx) = current.iter().position(|n| {
                n.name() == node.name()
                    && matches!(n, crate::lib::MacroNode::Folder { .. }) == is_folder
            }) {
                self.list_selected[self.current_tab] = idx;
            }

            self.auto_load();
            self.refresh_list(config);
            self.refresh_main(config);
        }
    }

    pub fn move_selected_in(&mut self, config: &Config) {
        if self.active != ActivePanel::List {
            return;
        }

        let sel_idx = self.list_selected[self.current_tab];
        if sel_idx == 0 {
            return;
        }

        let (view_path, _) = crate::panels::main::list::resolve_view(
            &self.expanded_path[self.current_tab],
            &self.library_tree,
            false,
        );

        let mut current = &self.library_tree;
        for &idx in &view_path {
            if let Some(crate::lib::MacroNode::Folder { children, .. }) = current.get(idx) {
                current = children;
            }
        }

        let target_folder_path = match &current[sel_idx - 1] {
            crate::lib::MacroNode::Folder { path, .. } => path.clone(),
            _ => return,
        };

        let node = current[sel_idx].clone();
        let old_path = node.path().to_path_buf();
        let name = match old_path.file_name() {
            Some(n) => n,
            _ => return,
        };
        let new_path = target_folder_path.join(name);

        if new_path.exists() {
            return;
        }

        if std::fs::rename(&old_path, &new_path).is_ok() {
            self.update_paths_after_move(&old_path, &new_path, config);
            self.reload_library_tree(config);

            self.expanded_path[self.current_tab].push(sel_idx - 1);

            let mut new_current = &self.library_tree;
            let (new_view_path, _) = crate::panels::main::list::resolve_view(
                &self.expanded_path[self.current_tab],
                &self.library_tree,
                false,
            );
            for &idx in &new_view_path {
                if let Some(crate::lib::MacroNode::Folder { children, .. }) = new_current.get(idx) {
                    new_current = children;
                }
            }

            let is_folder = matches!(node, crate::lib::MacroNode::Folder { .. });
            if let Some(idx) = new_current.iter().position(|n| {
                n.name() == node.name()
                    && matches!(n, crate::lib::MacroNode::Folder { .. }) == is_folder
            }) {
                self.list_selected[self.current_tab] = idx;
            } else {
                self.list_selected[self.current_tab] = 0;
            }

            self.auto_load();
            self.refresh_list(config);
            self.refresh_main(config);
        }
    }

    fn collect_paths(&self, nodes: &[MacroNode], order: &mut Vec<String>) {
        for node in nodes {
            if let Ok(rel) = node.path().strip_prefix(&self.library_root) {
                order.push(rel.to_string_lossy().to_string());
            }
            if let MacroNode::Folder { children, .. } = node {
                self.collect_paths(children, order);
            }
        }
    }

    pub fn current_parent_path(&self) -> PathBuf {
        let mut current = &self.library_tree;
        let mut path = self.library_root.clone();

        for &idx in &self.expanded_path[self.current_tab] {
            if let Some(MacroNode::Folder {
                path: p, children, ..
            }) = current.get(idx)
            {
                path = p.clone();
                current = children;
            } else {
                break;
            }
        }
        path
    }

    pub fn handle_right_arrow(&mut self, config: &Config) {
        if self.active != ActivePanel::List {
            return;
        }

        let (view_path, empty_idx) = list::resolve_view(
            &self.expanded_path[self.current_tab],
            &self.library_tree,
            false,
        );

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

        if let Some(MacroNode::Folder { children, .. }) =
            current_nodes.get(self.list_selected[self.current_tab])
        {
            do_expand = true;
            reset_scroll = !children.is_empty();
        }

        if do_expand {
            self.expanded_path[self.current_tab].push(self.list_selected[self.current_tab]);
            if reset_scroll {
                self.list_selected[self.current_tab] = 0;
                self.list_scroll[self.current_tab] = 0;
            }
            self.auto_load();
            self.refresh_list(config);
            self.refresh_main(config);
        }
    }

    pub fn handle_left_arrow(&mut self, config: &Config) {
        if self.active != ActivePanel::List {
            return;
        }

        if let Some(previous_parent_idx) = self.expanded_path[self.current_tab].pop() {
            self.list_selected[self.current_tab] = previous_parent_idx;
            self.list_scroll[self.current_tab] = 0;
            self.auto_load();
            self.refresh_list(config);
            self.refresh_main(config);
        }
    }

    pub fn edit_selected(&mut self, config: &Config) {
        if self.active != ActivePanel::List {
            return;
        }

        let mut target = None;
        if let Some(node) = self.get_selected_node() {
            if let MacroNode::Script { path, .. } = node {
                let rp = path
                    .strip_prefix(&self.library_root)
                    .unwrap_or(path.as_path())
                    .to_string_lossy()
                    .into_owned();
                target = Some((path.clone(), rp));
            }
        }

        if let Some((path, rp)) = target {
            if let Some(token) = self.cancellation_tokens[self.current_tab].take() {
                token.store(true, Ordering::SeqCst);
            }
            self.running_macros[self.current_tab] = None;
            self.macro_start_times[self.current_tab] = None;
            self.editors[self.current_tab].process_rx = None;
            self.editors[self.current_tab].process_input_tx = None;
            self.editors[self.current_tab].is_waiting_for_input = false;

            self.editors[self.current_tab].load_file(path, true, rp);
            self.active = ActivePanel::Main;
            self.list_input = ListInputMode::None;
            self.refresh_main(config);
            self.refresh_list(config);
            self.refresh_static_boxes(config);
        }
    }

    pub fn trigger_selected(&mut self, config: &Config) {
        if self.active != ActivePanel::List {
            return;
        }

        let mut target = None;
        if let Some(node) = self.get_selected_node() {
            if let MacroNode::Script { name, path } = node {
                target = Some((name.clone(), path.clone()));
            }
        }

        if let Some((name, path)) = target {
            if let Some(token) = self.cancellation_tokens[self.current_tab].take() {
                token.store(true, Ordering::SeqCst);
            }

            if let Ok(source) = std::fs::read_to_string(&path) {
                let (tx, rx) = std::sync::mpsc::sync_channel(1024);
                let (input_tx, input_rx) = std::sync::mpsc::channel();

                let cancel_token = Arc::new(AtomicBool::new(false));
                let focus_token = Arc::new(AtomicBool::new(true));

                let main_w = self
                    .term_w
                    .saturating_sub(layout::TABS_W + config.lib_width as u16 - 1);
                let main_h = self.term_h.saturating_sub(self.main_y as u16);
                let inner_w = main_w.saturating_sub(2) as u32;
                let inner_h = main_h.saturating_sub(2) as u32;
                let display_size = Arc::new(AtomicU32::new((inner_w << 16) | inner_h));

                let thread_cancel_token = Arc::clone(&cancel_token);
                let thread_focus_token = Arc::clone(&focus_token);
                let thread_display_size = Arc::clone(&display_size);
                let rp = path
                    .strip_prefix(&self.library_root)
                    .unwrap_or(path.as_path())
                    .to_string_lossy()
                    .into_owned();
                let macro_rel_path = rp;

                std::thread::spawn(move || {
                    crate::engine::core::run_in_thread(
                        &source,
                        tx,
                        input_rx,
                        thread_cancel_token,
                        thread_focus_token,
                        thread_display_size,
                        macro_rel_path,
                    );
                });

                let editor = &mut self.editors[self.current_tab];
                if !editor.is_output {
                    if editor.file_path.is_some() && editor.file_path == editor.last_edited_path {
                        editor.saved_state = Some(editor.state.clone());
                        editor.saved_scroll_x = editor.scroll_x;
                        editor.saved_scroll_y = editor.scroll_y;
                        editor.saved_folded_lines = editor.folded_lines.clone();
                    }
                    if editor.file_path.is_some() {
                        editor.last_file_path = editor.file_path.clone();
                    }
                }

                editor.process_rx = Some(rx);
                editor.process_input_tx = Some(input_tx);
                editor.state.lines = vec![String::new()];
                self.cancellation_tokens[self.current_tab] = Some(cancel_token);
                self.macro_focus_tokens[self.current_tab] = Some(focus_token);
                self.display_sizes[self.current_tab] = Some(display_size);
            } else {
                let editor = &mut self.editors[self.current_tab];
                if !editor.is_output {
                    if editor.file_path.is_some() && editor.file_path == editor.last_edited_path {
                        editor.saved_state = Some(editor.state.clone());
                        editor.saved_scroll_x = editor.scroll_x;
                        editor.saved_scroll_y = editor.scroll_y;
                        editor.saved_folded_lines = editor.folded_lines.clone();
                    }
                    if editor.file_path.is_some() {
                        editor.last_file_path = editor.file_path.clone();
                    }
                }
                editor.state.lines = vec![format!("Failed to read script '{}'", name)];
            }

            let editor = &mut self.editors[self.current_tab];
            editor.scroll_x = 0;
            editor.scroll_y = 0;
            self.editors[self.current_tab].state.cursor_x = 0;
            self.editors[self.current_tab].state.cursor_y = 0;
            self.editors[self.current_tab].state.selection_start = None;
            self.editors[self.current_tab].is_waiting_for_input = false;
            self.editors[self.current_tab].input_buffer.clear();

            self.editors[self.current_tab].file_path = None;
            self.running_macros[self.current_tab] = Some(path);
            self.macro_start_times[self.current_tab] = Some(std::time::Instant::now());
            self.active = ActivePanel::Main;
            self.editors[self.current_tab].is_editing = false;
            self.editors[self.current_tab].is_output = true;
            self.editors[self.current_tab].error_count = 0;
            self.editors[self.current_tab].error_lines.clear();
            self.refresh_main(config);
            self.refresh_list(config);
            self.refresh_static_boxes(config);
        }
    }

    pub fn update_macro_focus(&self, global_focus: bool) {
        for i in 0..6 {
            if let Some(token) = &self.macro_focus_tokens[i] {
                let is_focused = global_focus
                    && self.active == ActivePanel::Main
                    && self.current_tab == i
                    && self.list_input == ListInputMode::None;
                token.store(is_focused, Ordering::Relaxed);
            }
        }
    }

    pub fn render(&self, canvas: &mut Canvas) {
        canvas.put_box(&self.title_box, self.title_x, self.title_y);
        canvas.put_box_opaque(&self.deck_box, self.deck_x, self.deck_y);

        if self.active == ActivePanel::List {
            canvas.put_box(&self.main_box, self.main_x, self.main_y);
            canvas.put_box(&self.list_box, self.list_x, self.list_y);
        } else {
            canvas.put_box(&self.list_box, self.list_x, self.list_y);
            canvas.put_box(&self.main_box, self.main_x, self.main_y);
        }

        canvas.put_box(&self.tabs_box, self.tabs_x, self.tabs_y);
    }
}

pub fn handle_list_input(
    view: &mut MainView,
    key: &Key,
    terminal: &crate::Terminal,
    canvas: &mut crate::Canvas,
    config: &Config,
) -> Result<bool, String> {
    if view.list_input == ListInputMode::None {
        return Ok(false);
    }

    match key {
        Key::Esc => {
            view.list_input = ListInputMode::None;
            view.refresh_list(config);
            return Ok(true);
        }
        Key::Enter => {
            let mut name = match &view.list_input {
                ListInputMode::CreatingFile(n) => n.trim().to_string(),
                ListInputMode::CreatingFolder(n) => n.trim().to_string(),
                ListInputMode::Renaming(n) => n.trim().to_string(),
                _ => String::new(),
            };

            if name.ends_with(".nuui") {
                name = name.strip_suffix(".nuui").unwrap().to_string();
            }

            if name.is_empty() {
                view.list_input = ListInputMode::None;
                view.refresh_list(config);
                return Ok(true);
            }

            if let ListInputMode::Renaming(_) = view.list_input {
                if let Some(node) = view.get_selected_node().cloned() {
                    let old_path = node.path().to_path_buf();
                    let is_file = matches!(node, crate::lib::MacroNode::Script { .. });
                    let parent = old_path.parent().unwrap();
                    let new_path = if is_file {
                        parent.join(format!("{}.nuui", name))
                    } else {
                        parent.join(&name)
                    };

                    if new_path.exists() && new_path != old_path {
                        view.update_macro_focus(false);
                        crate::error::warning_box(
                            terminal,
                            canvas,
                            &format!("'{}' already exists!", name),
                            &["OK"],
                            0,
                            0,
                            view.min_w,
                            view.min_h,
                            config.get_border(),
                            view.theme.warning_color.clone(),
                            |cvs, w, h, k| view.draw_background(cvs, w, h, k, config),
                        );
                        view.update_macro_focus(true);
                        return Ok(true);
                    }

                    if std::fs::rename(&old_path, &new_path).is_ok() {
                        view.update_paths_after_move(&old_path, &new_path, config);
                    }

                    view.reload_library_tree(config);

                    let mut current = &view.library_tree;
                    let (view_path, _) = crate::panels::main::list::resolve_view(
                        &view.expanded_path[view.current_tab],
                        &view.library_tree,
                        false,
                    );
                    for &idx in &view_path {
                        if let Some(crate::lib::MacroNode::Folder { children, .. }) =
                            current.get(idx)
                        {
                            current = children;
                        }
                    }

                    let is_folder = !is_file;
                    if let Some(idx) = current.iter().position(|n| {
                        n.name() == name
                            && matches!(n, crate::lib::MacroNode::Folder { .. }) == is_folder
                    }) {
                        view.list_selected[view.current_tab] = idx;
                    }

                    view.list_input = ListInputMode::None;
                    view.auto_load();
                    view.refresh_list(config);
                    view.refresh_main(config);
                }
                return Ok(true);
            } else {
                let parent = view.current_parent_path();
                let is_file = matches!(view.list_input, ListInputMode::CreatingFile(_));
                let is_folder = !is_file;

                let target_path = if is_file {
                    parent.join(format!("{}.nuui", name))
                } else {
                    parent.join(&name)
                };

                if target_path.exists() {
                    view.update_macro_focus(false);
                    crate::error::warning_box(
                        terminal,
                        canvas,
                        &format!("'{}' already exists!", name),
                        &["OK"],
                        0,
                        0,
                        view.min_w,
                        view.min_h,
                        config.get_border(),
                        view.theme.warning_color.clone(),
                        |cvs, w, h, k| view.draw_background(cvs, w, h, k, config),
                    );
                    view.update_macro_focus(true);
                    return Ok(true);
                }

                if is_file {
                    let _ = std::fs::write(&target_path, "");
                } else {
                    let _ = std::fs::create_dir(&target_path);
                }

                view.reload_library_tree(config);

                let mut current = &view.library_tree;
                let (view_path, _) = crate::panels::main::list::resolve_view(
                    &view.expanded_path[view.current_tab],
                    &view.library_tree,
                    false,
                );
                for &idx in &view_path {
                    if let Some(crate::lib::MacroNode::Folder { children, .. }) = current.get(idx) {
                        current = children;
                    }
                }

                if let Some(idx) = current.iter().position(|n| {
                    n.name() == name
                        && matches!(n, crate::lib::MacroNode::Folder { .. }) == is_folder
                }) {
                    view.list_selected[view.current_tab] = idx;
                }

                if config.lib_sorting == "custom" {
                    view.save_custom_order();
                }

                view.list_input = ListInputMode::None;
                view.auto_load();
                view.refresh_list(config);
                view.refresh_main(config);
                return Ok(true);
            }
        }
        Key::Backspace => {
            let cursor = &mut view.list_input_cursor;
            match &mut view.list_input {
                ListInputMode::CreatingFile(n)
                | ListInputMode::CreatingFolder(n)
                | ListInputMode::Renaming(n) => {
                    if *cursor > 0 && !n.is_empty() {
                        *cursor -= 1;
                        let byte_idx = n.char_indices().nth(*cursor).map(|(i, _)| i).unwrap();
                        n.remove(byte_idx);
                    }
                }
                _ => {}
            }
            view.refresh_list(config);
            return Ok(true);
        }
        Key::CtrlBackspace | Key::Ctrl('w') | Key::Ctrl('h') => {
            let cursor = &mut view.list_input_cursor;
            match &mut view.list_input {
                ListInputMode::CreatingFile(n)
                | ListInputMode::CreatingFolder(n)
                | ListInputMode::Renaming(n) => {
                    if *cursor > 0 && !n.is_empty() {
                        let initial_cursor = *cursor;
                        let chars: Vec<char> = n.chars().collect();
                        let mut i = *cursor;

                        while i > 0 && chars[i - 1].is_whitespace() {
                            i -= 1;
                        }
                        if i > 0 {
                            let is_word = chars[i - 1].is_alphanumeric() || chars[i - 1] == '_';
                            while i > 0 {
                                let prev_is_word =
                                    chars[i - 1].is_alphanumeric() || chars[i - 1] == '_';
                                if chars[i - 1].is_whitespace() || prev_is_word != is_word {
                                    break;
                                }
                                i -= 1;
                            }
                        }

                        let delete_count = initial_cursor - i;
                        if delete_count > 0 {
                            let start_byte = n
                                .char_indices()
                                .nth(i)
                                .map(|(idx, _)| idx)
                                .unwrap_or(n.len());
                            let end_byte = n
                                .char_indices()
                                .nth(initial_cursor)
                                .map(|(idx, _)| idx)
                                .unwrap_or(n.len());
                            n.drain(start_byte..end_byte);
                            *cursor = i;
                        }
                    }
                }
                _ => {}
            }
            view.refresh_list(config);
            return Ok(true);
        }
        Key::Left => {
            if view.list_input_cursor > 0 {
                view.list_input_cursor -= 1;
            }
            view.refresh_list(config);
            return Ok(true);
        }
        Key::CtrlLeft => {
            let cursor = &mut view.list_input_cursor;
            match &view.list_input {
                ListInputMode::CreatingFile(n)
                | ListInputMode::CreatingFolder(n)
                | ListInputMode::Renaming(n) => {
                    let chars: Vec<char> = n.chars().collect();
                    let mut i = *cursor;
                    while i > 0 && chars[i - 1].is_whitespace() {
                        i -= 1;
                    }
                    if i > 0 {
                        let is_word = chars[i - 1].is_alphanumeric() || chars[i - 1] == '_';
                        while i > 0 {
                            let prev_is_word =
                                chars[i - 1].is_alphanumeric() || chars[i - 1] == '_';
                            if chars[i - 1].is_whitespace() || prev_is_word != is_word {
                                break;
                            }
                            i -= 1;
                        }
                    }
                    *cursor = i;
                }
                _ => {}
            }
            view.refresh_list(config);
            return Ok(true);
        }
        Key::Right => {
            let max_len = match &view.list_input {
                ListInputMode::CreatingFile(s) => s.chars().count(),
                ListInputMode::CreatingFolder(s) => s.chars().count(),
                ListInputMode::Renaming(s) => s.chars().count(),
                _ => 0,
            };
            if view.list_input_cursor < max_len {
                view.list_input_cursor += 1;
            }
            view.refresh_list(config);
            return Ok(true);
        }
        Key::CtrlRight => {
            let cursor = &mut view.list_input_cursor;
            match &view.list_input {
                ListInputMode::CreatingFile(n)
                | ListInputMode::CreatingFolder(n)
                | ListInputMode::Renaming(n) => {
                    let chars: Vec<char> = n.chars().collect();
                    let mut i = *cursor;
                    let len = chars.len();

                    if i < len && chars[i].is_whitespace() {
                        while i < len && chars[i].is_whitespace() {
                            i += 1;
                        }
                    } else if i < len {
                        let is_word = chars[i].is_alphanumeric() || chars[i] == '_';
                        while i < len {
                            let curr_is_word = chars[i].is_alphanumeric() || chars[i] == '_';
                            if chars[i].is_whitespace() || curr_is_word != is_word {
                                break;
                            }
                            i += 1;
                        }
                        while i < len && chars[i].is_whitespace() {
                            i += 1;
                        }
                    }
                    *cursor = i;
                }
                _ => {}
            }
            view.refresh_list(config);
            return Ok(true);
        }
        Key::Up | Key::Down | Key::Tab => {
            return Ok(true);
        }
        Key::Char(c) | Key::Shift(c) => {
            if !c.is_control() {
                let final_c = if matches!(key, Key::Shift(_)) && c.is_ascii_lowercase() {
                    c.to_ascii_uppercase()
                } else {
                    *c
                };

                let cursor = &mut view.list_input_cursor;
                match &mut view.list_input {
                    ListInputMode::CreatingFile(n)
                    | ListInputMode::CreatingFolder(n)
                    | ListInputMode::Renaming(n) => {
                        if n.chars().count() < 64 {
                            if final_c != '/' && final_c != '\\' {
                                let byte_idx = n
                                    .char_indices()
                                    .nth(*cursor)
                                    .map(|(i, _)| i)
                                    .unwrap_or(n.len());
                                n.insert(byte_idx, final_c);
                                *cursor += 1;
                            }
                        }
                    }
                    _ => {}
                }
                view.refresh_list(config);
                return Ok(true);
            }
        }
        _ => {}
    }
    Ok(false)
}

pub fn handle_list_action(
    view: &mut MainView,
    key: &Key,
    terminal: &crate::Terminal,
    canvas: &mut crate::Canvas,
    config: &mut Config,
) -> Result<bool, String> {
    if view.active != ActivePanel::List {
        return Ok(false);
    }

    let mut handled = false;

    let is_force_delete = if let Key::Shift(c) = key {
        c.to_ascii_lowercase() == config.bind_lib_delete
    } else {
        false
    };

    if let Key::Char(c) = key {
        if *c == config.bind_lib_move_up {
            if config.lib_sorting != "custom" {
                config.lib_sorting = "custom".to_string();
                config.save();
                view.reload_library_tree(config);
            }
            view.move_selected_up(config);
            handled = true;
        } else if *c == config.bind_lib_move_down {
            if config.lib_sorting != "custom" {
                config.lib_sorting = "custom".to_string();
                config.save();
                view.reload_library_tree(config);
            }
            view.move_selected_down(config);
            handled = true;
        } else if *c == config.bind_lib_move_out {
            view.move_selected_out(config);
            handled = true;
        } else if *c == config.bind_lib_move_in {
            view.move_selected_in(config);
            handled = true;
        } else if *c == config.bind_lib_new_file {
            view.list_input = ListInputMode::CreatingFile(String::new());
            view.list_input_cursor = 0;
            view.refresh_list(config);
            handled = true;
        } else if *c == config.bind_lib_new_folder {
            view.list_input = ListInputMode::CreatingFolder(String::new());
            view.list_input_cursor = 0;
            view.refresh_list(config);
            handled = true;
        } else if *c == config.bind_lib_edit {
            view.edit_selected(config);
            handled = true;
        } else if *c == config.bind_lib_rename {
            if let Some(node) = view.get_selected_node().cloned() {
                view.list_input_cursor = node.name().chars().count();
                view.list_input = ListInputMode::Renaming(node.name().to_string());
                view.refresh_list(config);
                handled = true;
            }
        } else if *c == config.bind_lib_delete {
            if let Some(node) = view.get_selected_node().cloned() {
                let path = node.path().to_path_buf();
                let is_folder = matches!(node, crate::lib::MacroNode::Folder { .. });

                let msg = if is_folder {
                    let is_empty = std::fs::read_dir(&path)
                        .map(|mut i| i.next().is_none())
                        .unwrap_or(true);
                    if is_empty {
                        format!("Delete folder '{}'?", node.name())
                    } else {
                        format!("Delete folder '{}' and all contents?", node.name())
                    }
                } else {
                    format!("Delete file '{}'?", node.name())
                };

                view.update_macro_focus(false);
                let res = crate::error::warning_box(
                    terminal,
                    canvas,
                    &msg,
                    &["CANCEL", "CONFIRM"],
                    0,
                    0,
                    view.min_w,
                    view.min_h,
                    config.get_border(),
                    view.theme.warning_color.clone(),
                    |cvs, w, h, k| view.draw_background(cvs, w, h, k, config),
                );
                view.update_macro_focus(true);

                if res == crate::PanelResult::Ok(1) {
                    crate::lib::delete_macrodata(&path, is_folder);
                    if is_folder {
                        let _ = std::fs::remove_dir_all(&path);
                    } else {
                        let _ = std::fs::remove_file(&path);
                    }

                    view.clear_editor_for_path(&path);

                    view.reload_library_tree(config);
                    view.auto_load();
                    view.refresh_list(config);
                    view.refresh_main(config);
                }
                handled = true;
            }
        }
    }

    if !handled && is_force_delete {
        if let Some(node) = view.get_selected_node().cloned() {
            let path = node.path().to_path_buf();
            let is_folder = matches!(node, crate::lib::MacroNode::Folder { .. });

            let mut confirm = true;

            if is_folder {
                let is_empty = std::fs::read_dir(&path)
                    .map(|mut i| i.next().is_none())
                    .unwrap_or(true);
                if !is_empty {
                    let msg = format!("Delete folder '{}' and all contents?", node.name());
                    view.update_macro_focus(false);
                    let res = crate::error::warning_box(
                        terminal,
                        canvas,
                        &msg,
                        &["CANCEL", "CONFIRM"],
                        0,
                        0,
                        view.min_w,
                        view.min_h,
                        config.get_border(),
                        view.theme.warning_color.clone(),
                        |cvs, w, h, k| view.draw_background(cvs, w, h, k, config),
                    );
                    view.update_macro_focus(true);
                    confirm = res == crate::PanelResult::Ok(1);
                }
            } else {
                let content = std::fs::read_to_string(&path).unwrap_or_default();
                if !content.trim().is_empty() {
                    let msg = format!("Delete file '{}'?", node.name());
                    view.update_macro_focus(false);
                    let res = crate::error::warning_box(
                        terminal,
                        canvas,
                        &msg,
                        &["CANCEL", "CONFIRM"],
                        0,
                        0,
                        view.min_w,
                        view.min_h,
                        config.get_border(),
                        view.theme.warning_color.clone(),
                        |cvs, w, h, k| view.draw_background(cvs, w, h, k, config),
                    );
                    view.update_macro_focus(true);
                    confirm = res == crate::PanelResult::Ok(1);
                }
            }

            if confirm {
                crate::lib::delete_macrodata(&path, is_folder);
                if is_folder {
                    let _ = std::fs::remove_dir_all(&path);
                } else {
                    let _ = std::fs::remove_file(&path);
                }

                view.clear_editor_for_path(&path);

                view.reload_library_tree(config);
                view.auto_load();
                view.refresh_list(config);
                view.refresh_main(config);
            }
            handled = true;
        }
    }

    Ok(handled)
}
