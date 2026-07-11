pub mod layout;
pub mod list;
pub mod mainbox;
pub mod r#static;

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::{
    Border, Box, Canvas, Color, Key, Modifier, Style, conf::Config, editor::Editor, lib::MacroNode,
    theme::themecore::Theme,
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
    pub list_selected: usize,
    pub list_scroll: usize,

    pub current_tab: usize,
    pub editors: [Editor; 6],
    pub running_macros: [Option<PathBuf>; 6],
    pub cancellation_tokens: [Option<Arc<AtomicBool>>; 6],
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

    pub theme: Theme,
    pub list_input: ListInputMode,
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
        let header_h = theme.title.len().max(1) as u16;

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

        let editors = std::array::from_fn(|_| Editor::new());
        let running_macros = std::array::from_fn(|_| None);
        let cancellation_tokens = std::array::from_fn(|_| None);

        let mut view = Self {
            term_w,
            term_h,
            min_w: 64,
            min_h: 13 + header_h,
            active,
            list_selected: 0,
            list_scroll: 0,
            library_tree,
            library_root,
            expanded_path: Vec::new(),
            current_tab: 0,
            editors,
            running_macros,
            cancellation_tokens,
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
            theme,
            list_input: ListInputMode::None,
        };

        view.auto_load();
        view.resize(term_w, term_h, config);
        view
    }

    pub fn get_selected_node(&self) -> Option<&MacroNode> {
        let mut current = &self.library_tree;
        for &idx in &self.expanded_path {
            if let Some(MacroNode::Folder { children, .. }) = current.get(idx) {
                current = children;
            } else {
                return None;
            }
        }
        current.get(self.list_selected)
    }

    pub fn auto_load(&mut self) {
        if self.running_macros[self.current_tab].is_some() {
            return;
        }

        let mut to_load = None;
        let mut is_folder = false;

        if let Some(node) = self.get_selected_node() {
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

        let should_edit = self.active == ActivePanel::Main;
        let editor = &mut self.editors[self.current_tab];

        if let Some((path, rp)) = to_load {
            if editor.file_path.as_deref() != Some(path.as_path()) {
                editor.load_file(path, should_edit, rp);
            } else if should_edit && !editor.is_editing {
                editor.is_editing = true;
                editor.refresh_analysis();
            }
        } else if is_folder {
            editor.file_path = None;
            editor.rel_path.clear();
            editor.state.lines = vec![String::new()];
            editor.is_editing = false;
            editor.error_count = 0;
            editor.error_lines.clear();

            editor.scroll_x = 0;
            editor.scroll_y = 0;
            editor.state.cursor_x = 0;
            editor.state.cursor_y = 0;
            editor.state.selection_start = None;
        }
    }

    pub fn switch_tab(&mut self, tab: usize, config: &Config) {
        if tab >= config.tabs_num.clamp(2, 6) || tab == self.current_tab {
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

        let header_h = self.theme.title.len().max(1) as u16;

        let (main_pos, list_pos, tabs_pos, title_pos, deck_pos) =
            layout::get_positions(term_w, term_h, header_h);

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
        self.refresh_static_boxes(config);
    }

    pub fn refresh_main(&mut self, config: &Config) {
        let header_h = self.theme.title.len().max(1) as u16;
        self.main_box = self.editors[self.current_tab].render(
            self.term_w
                .saturating_sub(layout::TABS_W + layout::LIST_W - 1),
            self.term_h.saturating_sub(header_h),
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

        self.list_box = list::refresh(
            self.term_h,
            header_h,
            self.active,
            &self.library_tree,
            &self.expanded_path,
            &mut self.list_selected,
            &mut self.list_scroll,
            active_path,
            config,
            &self.theme,
            &self.list_input,
        );
    }

    pub fn refresh_static_boxes(&mut self, config: &Config) {
        let header_h = self.theme.title.len().max(1) as u16;
        self.tabs_box =
            r#static::refresh_tabs(&self.theme, config, self.current_tab, &self.running_macros);
        self.title_box = r#static::refresh_title(&self.theme);
        self.deck_box = r#static::refresh_deck(self.term_w, header_h, &self.theme);
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

        let (_, empty_idx) = list::resolve_view(&self.expanded_path, &self.library_tree, false);

        if self.list_selected > 0 {
            self.list_selected -= 1;
            if empty_idx.is_some() {
                self.expanded_path.pop();
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

        let (view_path, empty_idx) =
            list::resolve_view(&self.expanded_path, &self.library_tree, false);

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
        self.refresh_list(config);
        self.refresh_main(config);
    }

    pub fn move_selected_up(&mut self, config: &Config) {
        if config.lib_sorting != "custom" {
            return;
        }
        if self.list_selected > 0 {
            let (view_path, _) = list::resolve_view(&self.expanded_path, &self.library_tree, false);
            self.swap_items(&view_path, self.list_selected, self.list_selected - 1);
            self.list_selected -= 1;
            self.save_custom_order();
            self.refresh_list(config);
        }
    }

    pub fn move_selected_down(&mut self, config: &Config) {
        if config.lib_sorting != "custom" {
            return;
        }
        let (view_path, _) = list::resolve_view(&self.expanded_path, &self.library_tree, false);

        let mut current = &self.library_tree;
        for &idx in &view_path {
            if let Some(MacroNode::Folder { children, .. }) = current.get(idx) {
                current = children;
            }
        }

        if self.list_selected + 1 < current.len() {
            self.swap_items(&view_path, self.list_selected, self.list_selected + 1);
            self.list_selected += 1;
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

        for &idx in &self.expanded_path {
            if let Some(MacroNode::Folder {
                path: p, children, ..
            }) = current.get(idx)
            {
                path = p.clone();
                current = children;
            }
        }
        path
    }

    pub fn handle_right_arrow(&mut self, config: &Config) {
        if self.active != ActivePanel::List {
            return;
        }

        let (view_path, empty_idx) =
            list::resolve_view(&self.expanded_path, &self.library_tree, false);

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
            self.refresh_list(config);
            self.refresh_main(config);
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
                    .unwrap_or(path)
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
            self.editors[self.current_tab].process_rx = None;

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
                let cancel_token = Arc::new(AtomicBool::new(false));
                let thread_cancel_token = Arc::clone(&cancel_token);
                std::thread::spawn(move || {
                    crate::engine::core::run_in_thread(&source, tx, thread_cancel_token);
                });

                self.editors[self.current_tab].process_rx = Some(rx);
                self.editors[self.current_tab].state.lines = vec![String::new()];
                self.cancellation_tokens[self.current_tab] = Some(cancel_token);
            } else {
                self.editors[self.current_tab].state.lines =
                    vec![format!("Failed to read script '{}'", name)];
            }

            self.editors[self.current_tab].scroll_x = 0;
            self.editors[self.current_tab].scroll_y = 0;
            self.editors[self.current_tab].state.cursor_x = 0;
            self.editors[self.current_tab].state.cursor_y = 0;
            self.editors[self.current_tab].state.selection_start = None;

            self.editors[self.current_tab].file_path = None;
            self.running_macros[self.current_tab] = Some(path);
            self.active = ActivePanel::Main;
            self.editors[self.current_tab].is_editing = false;
            self.editors[self.current_tab].error_count = 0;
            self.editors[self.current_tab].error_lines.clear();
            self.refresh_main(config);
            self.refresh_list(config);
            self.refresh_static_boxes(config);
        }
    }

    pub fn render(&self, canvas: &mut Canvas) {
        canvas.put_box(&self.title_box, self.title_x, self.title_y);
        canvas.put_box(&self.deck_box, self.deck_x, self.deck_y);

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
                if let Some(node) = view.get_selected_node() {
                    let old_path = node.path().to_path_buf();
                    let is_file = matches!(node, crate::lib::MacroNode::Script { .. });
                    let parent = old_path.parent().unwrap();
                    let new_path = if is_file {
                        parent.join(format!("{}.nuui", name))
                    } else {
                        parent.join(&name)
                    };

                    if new_path.exists() && new_path != old_path {
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
                            |cvs, w, h| {
                                if w != view.term_w || h != view.term_h {
                                    if w >= view.min_w && h >= view.min_h {
                                        view.resize(w, h, config);
                                    }
                                } else {
                                    if view.term_w >= view.min_w && view.term_h >= view.min_h {
                                        view.refresh_all(config);
                                    }
                                }
                                view.render(cvs);
                            },
                        );
                        return Ok(true);
                    }

                    if std::fs::rename(&old_path, &new_path).is_ok() {
                        if config.lib_sorting == "custom" {
                            let mut order = crate::lib::load_custom_order();
                            let old_rel = old_path
                                .strip_prefix(&view.library_root)
                                .unwrap()
                                .to_string_lossy()
                                .to_string();
                            let new_rel = new_path
                                .strip_prefix(&view.library_root)
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
                            if view.editors[i].file_path.as_ref() == Some(&old_path) {
                                view.editors[i].file_path = Some(new_path.clone());
                            }
                            if view.running_macros[i].as_ref() == Some(&old_path) {
                                view.running_macros[i] = Some(new_path.clone());
                            }
                        }
                    }

                    let l = crate::lib::init(&config.lib_sorting)?;
                    view.library_tree = l.tree;

                    let mut current = &view.library_tree;
                    let (view_path, _) = crate::panels::main::list::resolve_view(
                        &view.expanded_path,
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
                        view.list_selected = idx;
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
                        |cvs, w, h| {
                            if w != view.term_w || h != view.term_h {
                                if w >= view.min_w && h >= view.min_h {
                                    view.resize(w, h, config);
                                }
                            } else {
                                if view.term_w >= view.min_w && view.term_h >= view.min_h {
                                    view.refresh_all(config);
                                }
                            }
                            view.render(cvs);
                        },
                    );
                    return Ok(true);
                }

                if is_file {
                    let _ = std::fs::write(&target_path, "");
                } else {
                    let _ = std::fs::create_dir(&target_path);
                }

                let l = crate::lib::init(&config.lib_sorting)?;
                view.library_tree = l.tree;

                let mut current = &view.library_tree;
                let (view_path, _) = crate::panels::main::list::resolve_view(
                    &view.expanded_path,
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
                    view.list_selected = idx;
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
            match &mut view.list_input {
                ListInputMode::CreatingFile(n)
                | ListInputMode::CreatingFolder(n)
                | ListInputMode::Renaming(n) => {
                    n.pop();
                }
                _ => {}
            }
            view.refresh_list(config);
            return Ok(true);
        }
        Key::Up | Key::Down | Key::Left | Key::Right | Key::Tab => {
            return Ok(true);
        }
        Key::Char(c) | Key::Shift(c) => {
            if !c.is_control() {
                let mut final_c = *c;
                if let Key::Shift(_) = key {
                    let caps = crate::Terminal::is_caps_lock_on();
                    final_c = c.to_ascii_uppercase();
                    if caps && final_c.is_ascii_alphabetic() {
                        final_c = final_c.to_ascii_lowercase();
                    }
                } else {
                    let caps = crate::Terminal::is_caps_lock_on();
                    if caps && final_c.is_ascii_alphabetic() {
                        final_c = final_c.to_ascii_uppercase();
                    }
                }

                match &mut view.list_input {
                    ListInputMode::CreatingFile(n)
                    | ListInputMode::CreatingFolder(n)
                    | ListInputMode::Renaming(n) => {
                        if n.chars().count() < 64 {
                            if final_c != '/' && final_c != '\\' {
                                n.push(final_c);
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
    config: &Config,
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
            view.move_selected_up(config);
            handled = true;
        } else if *c == config.bind_lib_move_down {
            view.move_selected_down(config);
            handled = true;
        } else if *c == config.bind_lib_new_file {
            view.list_input = ListInputMode::CreatingFile(String::new());
            view.refresh_list(config);
            handled = true;
        } else if *c == config.bind_lib_new_folder {
            view.list_input = ListInputMode::CreatingFolder(String::new());
            view.refresh_list(config);
            handled = true;
        } else if *c == config.bind_lib_edit {
            view.edit_selected(config);
            handled = true;
        } else if *c == config.bind_lib_rename {
            if let Some(node) = view.get_selected_node() {
                view.list_input = ListInputMode::Renaming(node.name().to_string());
                view.refresh_list(config);
                handled = true;
            }
        } else if *c == config.bind_lib_delete {
            if let Some(node) = view.get_selected_node() {
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
                    |cvs, w, h| {
                        if w != view.term_w || h != view.term_h {
                            if w >= view.min_w && h >= view.min_h {
                                view.resize(w, h, config);
                            }
                        } else {
                            if view.term_w >= view.min_w && view.term_h >= view.min_h {
                                view.refresh_all(config);
                            }
                        }
                        view.render(cvs);
                    },
                );

                if res == crate::PanelResult::Ok(1) {
                    if is_folder {
                        let _ = std::fs::remove_dir_all(&path);
                    } else {
                        let _ = std::fs::remove_file(&path);
                    }

                    for i in 0..6 {
                        if view.editors[i].file_path.as_ref() == Some(&path)
                            || view.running_macros[i].as_ref() == Some(&path)
                        {
                            if let Some(token) = view.cancellation_tokens[i].take() {
                                token.store(true, std::sync::atomic::Ordering::SeqCst);
                            }
                            view.editors[i].file_path = None;
                            view.running_macros[i] = None;
                            view.editors[i].process_rx = None;
                            view.editors[i].state.lines = vec![String::new()];
                            view.editors[i].error_count = 0;
                            view.editors[i].error_lines.clear();
                        }
                    }

                    let l = crate::lib::init(&config.lib_sorting)?;
                    view.library_tree = l.tree;
                    view.auto_load();
                    view.refresh_list(config);
                    view.refresh_main(config);
                }
                handled = true;
            }
        }
    }

    if !handled && is_force_delete {
        if let Some(node) = view.get_selected_node() {
            let path = node.path().to_path_buf();
            let is_folder = matches!(node, crate::lib::MacroNode::Folder { .. });

            let mut confirm = true;

            if is_folder {
                let is_empty = std::fs::read_dir(&path)
                    .map(|mut i| i.next().is_none())
                    .unwrap_or(true);
                if !is_empty {
                    let msg = format!("Delete folder '{}' and all contents?", node.name());
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
                        |cvs, w, h| {
                            if w != view.term_w || h != view.term_h {
                                if w >= view.min_w && h >= view.min_h {
                                    view.resize(w, h, config);
                                }
                            } else {
                                if view.term_w >= view.min_w && view.term_h >= view.min_h {
                                    view.refresh_all(config);
                                }
                            }
                            view.render(cvs);
                        },
                    );
                    confirm = res == crate::PanelResult::Ok(1);
                }
            } else {
                let content = std::fs::read_to_string(&path).unwrap_or_default();
                if !content.trim().is_empty() {
                    let msg = format!("Delete file '{}'?", node.name());
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
                        |cvs, w, h| {
                            if w != view.term_w || h != view.term_h {
                                if w >= view.min_w && h >= view.min_h {
                                    view.resize(w, h, config);
                                }
                            } else {
                                if view.term_w >= view.min_w && view.term_h >= view.min_h {
                                    view.refresh_all(config);
                                }
                            }
                            view.render(cvs);
                        },
                    );
                    confirm = res == crate::PanelResult::Ok(1);
                }
            }

            if confirm {
                if is_folder {
                    let _ = std::fs::remove_dir_all(&path);
                } else {
                    let _ = std::fs::remove_file(&path);
                }

                for i in 0..6 {
                    if view.editors[i].file_path.as_ref() == Some(&path)
                        || view.running_macros[i].as_ref() == Some(&path)
                    {
                        if let Some(token) = view.cancellation_tokens[i].take() {
                            token.store(true, std::sync::atomic::Ordering::SeqCst);
                        }
                        view.editors[i].file_path = None;
                        view.running_macros[i] = None;
                        view.editors[i].process_rx = None;
                        view.editors[i].state.lines = vec![String::new()];
                        view.editors[i].error_count = 0;
                        view.editors[i].error_lines.clear();
                    }
                }

                let l = crate::lib::init(&config.lib_sorting)?;
                view.library_tree = l.tree;
                view.auto_load();
                view.refresh_list(config);
                view.refresh_main(config);
            }
            handled = true;
        }
    }

    Ok(handled)
}
