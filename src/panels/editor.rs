use crate::conf::Config;
use crate::{Box, Color, Gradient, Key, Modifier, Style, theme::themecore::Theme};
use arboard::Clipboard;
use std::collections::HashSet;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

fn calculate_hash(lines: &[String]) -> u64 {
    let mut hasher = DefaultHasher::new();
    for line in lines {
        line.hash(&mut hasher);
        '\n'.hash(&mut hasher);
    }
    hasher.finish()
}

pub fn is_wayland_session() -> bool {
    if std::env::var("WAYLAND_DISPLAY").is_ok()
        || std::env::var("XDG_SESSION_TYPE").unwrap_or_default() == "wayland"
    {
        return true;
    }
    if let Ok(sudo_user) = std::env::var("SUDO_USER") {
        if let Ok(output) = std::process::Command::new("id")
            .arg("-u")
            .arg(&sudo_user)
            .output()
        {
            let uid = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let xdg_dir = format!("/run/user/{}", uid);
            if let Ok(entries) = std::fs::read_dir(&xdg_dir) {
                for entry in entries.filter_map(Result::ok) {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.starts_with("wayland-") && !name.ends_with(".lock") {
                        return true;
                    }
                }
            }
        }
    }
    false
}

#[cfg(target_os = "linux")]
pub fn fix_x11_sudo_env() {
    unsafe {
        if libc::geteuid() == 0 {
            if let Ok(sudo_user) = std::env::var("SUDO_USER") {
                if std::env::var("DISPLAY").is_err()
                    || std::env::var("DISPLAY").unwrap_or_default().is_empty()
                {
                    std::env::set_var("DISPLAY", ":0");
                }
                if std::env::var("XAUTHORITY").is_err()
                    || std::env::var("XAUTHORITY").unwrap_or_default().is_empty()
                {
                    if let Ok(output) = std::process::Command::new("getent")
                        .args(["passwd", &sudo_user])
                        .output()
                    {
                        let out = String::from_utf8_lossy(&output.stdout);
                        let parts: Vec<&str> = out.split(':').collect();
                        if parts.len() >= 6 {
                            let home = parts[5];
                            std::env::set_var("XAUTHORITY", format!("{}/.Xauthority", home));
                        } else {
                            std::env::set_var(
                                "XAUTHORITY",
                                format!("/home/{}/.Xauthority", sudo_user),
                            );
                        }
                    } else {
                        std::env::set_var("XAUTHORITY", format!("/home/{}/.Xauthority", sudo_user));
                    }
                }
            }
        }
    }
}

pub fn build_clipboard_cmd(program: &str, args: &[&str]) -> std::process::Command {
    if let Ok(sudo_user) = std::env::var("SUDO_USER") {
        let mut cmd = std::process::Command::new("sudo");
        cmd.arg("-u").arg(&sudo_user).arg("env");

        if let Ok(output) = std::process::Command::new("id")
            .arg("-u")
            .arg(&sudo_user)
            .output()
        {
            let uid = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let xdg_dir = format!("/run/user/{}", uid);
            cmd.arg(format!("XDG_RUNTIME_DIR={}", xdg_dir));

            let mut wd = std::env::var("WAYLAND_DISPLAY").unwrap_or_default();
            if wd.is_empty() {
                if let Ok(entries) = std::fs::read_dir(&xdg_dir) {
                    for entry in entries.filter_map(Result::ok) {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if name.starts_with("wayland-") && !name.ends_with(".lock") {
                            wd = name;
                            break;
                        }
                    }
                }
            }
            if !wd.is_empty() {
                cmd.arg(format!("WAYLAND_DISPLAY={}", wd));
            }
        }

        let display = std::env::var("DISPLAY").unwrap_or_else(|_| ":0".to_string());
        cmd.arg(format!("DISPLAY={}", display));

        cmd.arg(program);
        for arg in args {
            cmd.arg(arg);
        }
        cmd
    } else {
        let mut cmd = std::process::Command::new(program);
        cmd.args(args);
        cmd
    }
}

pub fn set_clipboard(text: String) {
    let is_wayland = is_wayland_session();

    if is_wayland {
        use std::io::Write;
        if let Ok(mut child) = build_clipboard_cmd("wl-copy", &[])
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(text.as_bytes());
            }
            let _ = child.wait();
            return;
        }
    }

    if let Ok(mut cb) = Clipboard::new() {
        if cb.set_text(text.clone()).is_ok() {
            return;
        }
    }

    use std::io::Write;
    if let Ok(mut child) = build_clipboard_cmd("xclip", &["-selection", "clipboard", "-i"])
        .stdin(std::process::Stdio::piped())
        .spawn()
    {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(text.as_bytes());
        }
        let _ = child.wait();
    }
}

pub fn get_clipboard() -> Option<String> {
    let is_wayland = is_wayland_session();

    if is_wayland {
        if let Ok(output) = build_clipboard_cmd("wl-paste", &["-n"]).output() {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout).into_owned();
                if !text.is_empty() {
                    return Some(text);
                }
            }
        }
    }

    if let Ok(mut cb) = Clipboard::new() {
        if let Ok(text) = cb.get_text() {
            if !text.is_empty() || !is_wayland {
                return Some(text);
            }
        }
    }

    if let Ok(output) = build_clipboard_cmd("xclip", &["-selection", "clipboard", "-o"]).output() {
        if output.status.success() {
            return Some(String::from_utf8_lossy(&output.stdout).into_owned());
        }
    }

    None
}

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Command,
    Insert,
    Search,
    LineSearch,
}

#[derive(Clone)]
pub struct EditorState {
    pub lines: Vec<String>,
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub selection_start: Option<(usize, usize)>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum EditAction {
    Char(bool),
    Bulk,
}

pub struct Editor {
    pub is_editing: bool,
    pub is_output: bool,
    pub is_waiting_for_input: bool,
    pub input_buffer: String,
    pub last_blink_state: bool,

    pub mode: Mode,
    pub visual_mode: bool,
    pub state: EditorState,
    pub scroll_x: usize,
    pub scroll_y: usize,
    pub file_path: Option<PathBuf>,
    pub last_file_path: Option<PathBuf>,
    pub saved_state: Option<EditorState>,
    pub saved_scroll_x: usize,
    pub saved_scroll_y: usize,
    pub saved_folded_lines: HashSet<usize>,
    pub rel_path: String,
    pub saved_hash: u64,
    pub is_dirty_flag: bool,

    pub folded_lines: HashSet<usize>,

    pub undo_stack: Vec<(EditorState, HashSet<usize>, EditAction)>,
    pub redo_stack: Vec<(EditorState, HashSet<usize>, EditAction)>,
    pub last_edit_pos: Option<(usize, usize)>,
    pub last_edited_path: Option<PathBuf>,

    pub last_key_select_all: bool,
    pub last_key_file_bounds: bool,
    pub last_key_delete: bool,
    pub last_key_copy: bool,

    pub search_query: String,
    pub last_search: String,
    pub line_search_query: String,

    pub error_count: usize,
    pub error_lines: HashSet<usize>,
    pub defined_functions: HashSet<String>,

    pub process_input_tx: Option<std::sync::mpsc::Sender<String>>,
    pub process_rx: Option<std::sync::mpsc::Receiver<crate::EngineMessage>>,
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}

impl Editor {
    pub fn new() -> Self {
        Self {
            is_editing: false,
            is_output: false,
            is_waiting_for_input: false,
            input_buffer: String::new(),
            last_blink_state: false,
            mode: Mode::Command,
            visual_mode: false,
            state: EditorState {
                lines: vec![String::new()],
                cursor_x: 0,
                cursor_y: 0,
                selection_start: None,
            },
            scroll_x: 0,
            scroll_y: 0,
            file_path: None,
            last_file_path: None,
            saved_state: None,
            saved_scroll_x: 0,
            saved_scroll_y: 0,
            saved_folded_lines: HashSet::new(),
            rel_path: String::new(),
            saved_hash: 0,
            is_dirty_flag: false,
            folded_lines: HashSet::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            last_edit_pos: None,
            last_edited_path: None,
            last_key_select_all: false,
            last_key_file_bounds: false,
            last_key_delete: false,
            last_key_copy: false,
            search_query: String::new(),
            last_search: String::new(),
            line_search_query: String::new(),
            error_count: 0,
            error_lines: HashSet::new(),
            defined_functions: HashSet::new(),
            process_input_tx: None,
            process_rx: None,
        }
    }

    pub fn load_file(&mut self, path: PathBuf, edit: bool, rel_path: String) {
        let is_last_edited = self.last_edited_path.as_deref() == Some(path.as_path());

        match fs::read_to_string(&path) {
            Ok(content) => {
                let lines: Vec<String> = if content.is_empty() {
                    vec![String::new()]
                } else {
                    content
                        .split('\n')
                        .map(|s| s.strip_suffix('\r').unwrap_or(s).to_string())
                        .collect()
                };

                if is_last_edited {
                    if let Some(mut restored) = self.saved_state.take() {
                        restored.lines = lines;
                        self.state = restored;
                        self.scroll_x = self.saved_scroll_x;
                        self.scroll_y = self.saved_scroll_y;
                        self.folded_lines = self.saved_folded_lines.clone();
                    } else {
                        self.state.lines = lines;
                    }
                } else {
                    self.state = EditorState {
                        lines,
                        cursor_x: 0,
                        cursor_y: 0,
                        selection_start: None,
                    };
                    self.scroll_x = 0;
                    self.scroll_y = 0;
                    self.folded_lines.clear();
                }

                self.file_path = Some(path.clone());
                self.last_file_path = Some(path.clone());
                self.rel_path = rel_path;
                self.is_editing = edit;
                self.is_output = false;

                self.saved_hash = calculate_hash(&self.state.lines);
                self.is_dirty_flag = false;

                if edit {
                    if self.last_edited_path.as_deref() != Some(path.as_path()) {
                        self.undo_stack.clear();
                        self.redo_stack.clear();
                        self.last_edit_pos = None;
                        self.search_query.clear();
                        self.last_search.clear();
                        self.line_search_query.clear();
                        self.saved_state = None;
                        self.saved_folded_lines.clear();
                        self.last_edited_path = Some(path.clone());
                    }
                }
            }
            Err(e) => {
                self.state = EditorState {
                    lines: vec![format!("Error reading macro file: {}", e)],
                    cursor_x: 0,
                    cursor_y: 0,
                    selection_start: None,
                };
                self.file_path = None;
                self.last_file_path = None;
                self.rel_path.clear();
                self.is_editing = false;
                self.is_output = false;
                self.folded_lines.clear();

                self.saved_hash = 0;
                self.is_dirty_flag = false;

                if edit {
                    self.undo_stack.clear();
                    self.redo_stack.clear();
                    self.last_edit_pos = None;
                    self.search_query.clear();
                    self.last_search.clear();
                    self.line_search_query.clear();
                    self.saved_state = None;
                    self.saved_folded_lines.clear();
                    self.last_edited_path = None;
                }
            }
        }
        self.mode = Mode::Command;
        self.visual_mode = false;
        self.clamp_cursor();
        self.reset_keys();
        self.refresh_analysis(self.is_editing);
    }

    pub fn reload_file(&mut self) {
        if let Some(path) = &self.file_path {
            if let Ok(content) = fs::read_to_string(path) {
                let lines: Vec<String> = if content.is_empty() {
                    vec![String::new()]
                } else {
                    content
                        .split('\n')
                        .map(|s| s.strip_suffix('\r').unwrap_or(s).to_string())
                        .collect()
                };
                self.state.lines = lines;
                self.saved_hash = calculate_hash(&self.state.lines);
                self.is_dirty_flag = false;
                self.folded_lines.clear();
                self.clamp_cursor();
                self.refresh_analysis(self.is_editing);
            }
        }
    }

    pub fn save(&mut self) {
        if let Some(path) = &self.file_path {
            let content = self.state.lines.join("\n");
            if fs::write(path, content).is_ok() {
                self.saved_hash = calculate_hash(&self.state.lines);
                self.is_dirty_flag = false;
            }
            self.refresh_analysis(true);
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.is_dirty_flag
    }

    fn reset_keys(&mut self) {
        self.last_key_select_all = false;
        self.last_key_file_bounds = false;
        self.last_key_delete = false;
        self.last_key_copy = false;
    }

    pub fn get_block_end(&self, start_idx: usize) -> usize {
        let lines = &self.state.lines;
        if start_idx >= lines.len() {
            return start_idx;
        }

        let start_line = &lines[start_idx];
        let base_indent = start_line.chars().take_while(|c| c.is_whitespace()).count();

        let mut end_idx = start_idx;

        for i in (start_idx + 1)..lines.len() {
            let line = &lines[i];
            if line.trim().is_empty() {
                continue;
            }
            let indent = line.chars().take_while(|c| c.is_whitespace()).count();
            if indent <= base_indent {
                break;
            }
            end_idx = i;
        }
        end_idx
    }

    pub fn get_display_lines(&self) -> Vec<isize> {
        let mut result = Vec::with_capacity(self.state.lines.len());
        let mut i = 0;
        while i < self.state.lines.len() {
            result.push(i as isize);
            if self.folded_lines.contains(&i) {
                let end = self.get_block_end(i);
                if end > i {
                    result.push(-1);
                    i = end + 1;
                    continue;
                }
            }
            i += 1;
        }
        result
    }

    fn shift_folds(&mut self, after_y: usize, delta: isize) {
        if self.folded_lines.is_empty() {
            return;
        }
        let mut new_folds = HashSet::new();
        for &y in &self.folded_lines {
            if y < after_y {
                new_folds.insert(y);
            } else if delta >= 0 || y >= (after_y as isize - delta) as usize {
                new_folds.insert((y as isize + delta) as usize);
            }
        }
        self.folded_lines = new_folds;
    }

    fn push_undo(&mut self, action: EditAction) {
        let current_pos = (self.state.cursor_x, self.state.cursor_y);

        let mut contiguous = false;
        if let Some(pos) = self.last_edit_pos {
            if pos.1 == current_pos.1 && (pos.0 as isize - current_pos.0 as isize).abs() <= 1 {
                contiguous = true;
            } else if current_pos.1 == pos.1 + 1 && current_pos.0 == 0 {
                contiguous = true;
            }
        }

        let last_action = self.undo_stack.last().map(|(_, _, a)| *a);

        let should_push = match (last_action, action) {
            (_, EditAction::Bulk) => true,
            (Some(EditAction::Char(last_ws)), EditAction::Char(curr_ws)) => {
                last_ws != curr_ws || !contiguous
            }
            _ => true,
        };

        if should_push {
            self.undo_stack
                .push((self.state.clone(), self.folded_lines.clone(), action));

            if self.undo_stack.len() > 1024 {
                self.undo_stack.remove(0);
            }
        }

        self.last_edit_pos = Some(current_pos);
        self.redo_stack.clear();
    }

    fn prepare_edit(&mut self, is_whitespace: bool) {
        let action = if self.state.selection_start.is_some() {
            EditAction::Bulk
        } else {
            EditAction::Char(is_whitespace)
        };
        self.push_undo(action);
        if self.state.selection_start.is_some() {
            self.delete_selection();
        }
    }

    fn char_before_cursor(&self) -> Option<char> {
        if self.state.cursor_x > 0 {
            let line = &self.state.lines[self.state.cursor_y];
            line.chars().nth(self.state.cursor_x - 1)
        } else if self.state.cursor_y > 0 {
            Some('\n')
        } else {
            None
        }
    }

    fn char_after_cursor(&self) -> Option<char> {
        let line = &self.state.lines[self.state.cursor_y];
        if self.state.cursor_x < line.chars().count() {
            line.chars().nth(self.state.cursor_x)
        } else if self.state.cursor_y < self.state.lines.len() - 1 {
            Some('\n')
        } else {
            None
        }
    }

    pub fn refresh_analysis(&mut self, force_errors: bool) {
        if !force_errors {
            let mut funcs = HashSet::new();
            for f in crate::engine::core::BUILTIN_FUNCS {
                funcs.insert(f.to_string());
            }

            for line in &self.state.lines {
                let trimmed = line.trim_start();
                if trimmed.starts_with("fn") {
                    let after_fn = &trimmed[2..];
                    if after_fn.starts_with(|c: char| c.is_whitespace()) {
                        let rest = after_fn.trim_start();
                        let end = rest
                            .find(|c: char| !c.is_alphanumeric() && c != '_')
                            .unwrap_or(rest.len());
                        let name = &rest[..end];
                        if !name.is_empty() {
                            funcs.insert(name.to_string());
                        }
                    }
                }
            }
            self.defined_functions = funcs;
            return;
        }

        let source = self.state.lines.join("\n");
        let (count, lines, funcs) = crate::engine::core::analyze_code(&source);

        self.defined_functions = funcs;
        self.error_count = count;
        self.error_lines = lines;
    }

    pub fn handle_key(&mut self, key: Key, config: &Config) -> bool {
        let mut saved = false;
        let mut needs_analysis = false;
        let mut is_undo_redo = false;

        match self.mode {
            Mode::LineSearch => match key {
                Key::Esc | Key::Enter => {
                    self.mode = Mode::Command;
                }
                Key::Backspace => {
                    self.line_search_query.pop();
                    self.jump_to_line_search();
                }
                Key::Char(c) if c.is_ascii_digit() => {
                    let new_query = format!("{}{}", self.line_search_query, c);
                    if let Ok(line_num) = new_query.parse::<usize>() {
                        if line_num > 0 && line_num <= self.state.lines.len() {
                            self.line_search_query = new_query;
                            self.jump_to_line_search();
                        }
                    }
                }
                _ => {
                    self.mode = Mode::Command;
                }
            },
            Mode::Search => match key {
                Key::Esc => {
                    self.mode = Mode::Command;
                }
                Key::Enter => {
                    if !self.search_query.is_empty() {
                        self.last_search = self.search_query.clone();
                        self.find_next();
                    }
                }
                Key::Backspace => {
                    self.search_query.pop();
                }
                Key::Char(c) | Key::Shift(c) => {
                    if !c.is_control() {
                        let final_c = if matches!(key, Key::Shift(_)) && c.is_ascii_lowercase() {
                            c.to_ascii_uppercase()
                        } else {
                            c
                        };
                        self.search_query.push(final_c);
                    }
                }
                _ => {}
            },
            Mode::Command => {
                let mut reset_select_all = true;
                let mut reset_file_bounds = true;
                let mut reset_delete = true;
                let mut reset_copy = true;

                match key {
                    Key::Esc => {
                        self.visual_mode = false;
                        self.state.selection_start = None;
                        self.last_search.clear();
                    }
                    Key::Enter => {
                        if !self.last_search.is_empty() {
                            self.find_next();
                        }
                    }
                    k if k == Key::Char(config.bind_edit_search) => {
                        self.mode = Mode::Search;
                        self.visual_mode = false;
                        self.state.selection_start = None;
                    }
                    k if k == Key::Char(config.bind_edit_error_jump) => {
                        if !self.error_lines.is_empty() {
                            let current_line = self.state.cursor_y + 1;
                            let mut next_line = None;
                            let mut first_line = None;
                            for &line in &self.error_lines {
                                if first_line.map_or(true, |f| line < f) {
                                    first_line = Some(line);
                                }
                                if line > current_line {
                                    if next_line.map_or(true, |n| line < n) {
                                        next_line = Some(line);
                                    }
                                }
                            }
                            if let Some(target_line) = next_line.or(first_line) {
                                self.state.cursor_y = target_line.saturating_sub(1);
                                self.state.cursor_x = 0;
                                if !self.visual_mode {
                                    self.state.selection_start = None;
                                }

                                let target = self.state.cursor_y;
                                let mut to_remove = Vec::new();
                                for &fold_start in &self.folded_lines {
                                    if target > fold_start
                                        && target <= self.get_block_end(fold_start)
                                    {
                                        to_remove.push(fold_start);
                                    }
                                }
                                for r in to_remove {
                                    self.folded_lines.remove(&r);
                                }
                            }
                        }
                    }
                    k if k == Key::Char(config.bind_edit_insert) => {
                        self.visual_mode = false;
                        self.state.selection_start = None;
                        self.mode = Mode::Insert;
                    }
                    k if k == Key::Char(config.bind_edit_visual) => {
                        if self.visual_mode {
                            self.visual_mode = false;
                            self.state.selection_start = None;
                        } else {
                            self.visual_mode = true;
                            self.state.selection_start =
                                Some((self.state.cursor_x, self.state.cursor_y));
                        }
                    }
                    k if k == Key::Char(config.bind_edit_fold) => {
                        let y = self.state.cursor_y;
                        if self.folded_lines.contains(&y) {
                            self.folded_lines.remove(&y);
                        } else {
                            let line = &self.state.lines[y];
                            if line.trim_start().starts_with("fn ") {
                                let end = self.get_block_end(y);
                                if end > y {
                                    self.folded_lines.insert(y);
                                }
                            }
                        }
                    }
                    k if k == Key::Left || k == Key::Char(config.bind_edit_left) => {
                        self.move_cursor(-1, 0, self.visual_mode)
                    }
                    k if k == Key::Right || k == Key::Char(config.bind_edit_right) => {
                        self.move_cursor(1, 0, self.visual_mode)
                    }
                    k if k == Key::Up || k == Key::Char(config.bind_edit_up) => {
                        self.move_cursor(0, -1, self.visual_mode)
                    }
                    k if k == Key::Down || k == Key::Char(config.bind_edit_down) => {
                        self.move_cursor(0, 1, self.visual_mode)
                    }
                    k if k == Key::CtrlLeft
                        || k == Key::Ctrl(config.bind_edit_left)
                        || k == Key::Char(config.bind_edit_word_prev)
                        || k == Key::CtrlBackspace =>
                    {
                        self.jump_word_backward(self.visual_mode)
                    }
                    k if k == Key::CtrlRight
                        || k == Key::Ctrl(config.bind_edit_right)
                        || k == Key::Char(config.bind_edit_word_next) =>
                    {
                        self.jump_word_forward(self.visual_mode)
                    }
                    k if k == Key::CtrlUp || k == Key::Ctrl(config.bind_edit_up) => {
                        self.jump_block_backward(self.visual_mode)
                    }
                    k if k == Key::CtrlDown || k == Key::Ctrl(config.bind_edit_down) => {
                        self.jump_block_forward(self.visual_mode)
                    }
                    k if k == Key::Char(config.bind_edit_line_start) => {
                        if !self.visual_mode {
                            self.state.selection_start = None;
                        }
                        self.state.cursor_x = 0;
                    }
                    k if k == Key::Char(config.bind_edit_line_end) => {
                        if !self.visual_mode {
                            self.state.selection_start = None;
                        }
                        self.state.cursor_x = self.state.lines[self.state.cursor_y].chars().count();
                    }
                    k if k == Key::Char(config.bind_edit_select_all) => {
                        if self.last_key_select_all || self.state.selection_start.is_some() {
                            self.visual_mode = true;
                            self.state.selection_start = Some((0, 0));
                            self.state.cursor_y = self.state.lines.len().saturating_sub(1);
                            self.state.cursor_x =
                                self.state.lines[self.state.cursor_y].chars().count();
                        } else {
                            reset_select_all = false;
                            self.last_key_select_all = true;
                        }
                    }
                    Key::Delete => {
                        let is_ws = self.char_after_cursor().map_or(true, |c| c.is_whitespace());
                        if self.state.selection_start.is_some() {
                            self.prepare_edit(is_ws);
                        } else {
                            self.push_undo(EditAction::Char(is_ws));
                            self.delete_char_after();
                        }
                        needs_analysis = true;
                    }
                    k if k == Key::Char(config.bind_edit_file_bounds) => {
                        if !self.visual_mode {
                            self.state.selection_start = None;
                        }
                        if self.last_key_file_bounds {
                            self.state.cursor_y = 0;
                            self.state.cursor_x = 0;
                        } else {
                            reset_file_bounds = false;
                            self.last_key_file_bounds = true;
                        }
                    }
                    k if k == Key::Shift(config.bind_edit_file_bounds) => {
                        if !self.visual_mode {
                            self.state.selection_start = None;
                        }
                        self.state.cursor_y = self.state.lines.len().saturating_sub(1);
                        self.state.cursor_x = self.state.lines[self.state.cursor_y].chars().count();
                    }
                    k if k == Key::Ctrl(config.bind_edit_file_bounds) => {
                        self.mode = Mode::LineSearch;
                        self.visual_mode = false;
                        self.state.selection_start = None;
                        self.line_search_query.clear();
                    }
                    k if k == Key::Char(config.bind_edit_delete) => {
                        if self.state.selection_start.is_some() {
                            self.prepare_edit(true);
                            reset_delete = true;
                            needs_analysis = true;
                        } else if self.last_key_delete {
                            self.push_undo(EditAction::Bulk);
                            self.delete_current_line();
                            needs_analysis = true;
                        } else {
                            reset_delete = false;
                            self.last_key_delete = true;
                        }
                    }
                    k if k == Key::Char(config.bind_edit_copy) => {
                        if self.state.selection_start.is_some() {
                            self.copy_selection();
                            reset_copy = true;
                        } else if self.last_key_copy {
                            self.copy_current_line();
                        } else {
                            reset_copy = false;
                            self.last_key_copy = true;
                        }
                    }
                    k if k == Key::Char(config.bind_edit_paste) => {
                        if self.state.selection_start.is_some() {
                            self.prepare_edit(true);
                        } else {
                            self.push_undo(EditAction::Bulk);
                        }
                        self.paste_from_clipboard();
                        needs_analysis = true;
                    }
                    k if k == Key::Char(config.bind_edit_undo) => {
                        if let Some((state, folds, a)) = self.undo_stack.pop() {
                            let old_state = std::mem::replace(&mut self.state, state);
                            let old_folds = std::mem::replace(&mut self.folded_lines, folds);
                            self.redo_stack.push((old_state, old_folds, a));
                            self.last_edit_pos = None;
                            needs_analysis = true;
                            is_undo_redo = true;
                        }
                    }
                    k if k == Key::Char(config.bind_edit_redo) => {
                        if let Some((state, folds, a)) = self.redo_stack.pop() {
                            let old_state = std::mem::replace(&mut self.state, state);
                            let old_folds = std::mem::replace(&mut self.folded_lines, folds);
                            self.undo_stack.push((old_state, old_folds, a));
                            self.last_edit_pos = None;
                            needs_analysis = true;
                            is_undo_redo = true;
                        }
                    }
                    k if k == Key::Char(config.bind_edit_save) => {
                        self.save();
                        saved = true;
                    }
                    Key::Tab => {
                        if self.visual_mode {
                            self.indent_selection();
                            needs_analysis = true;
                        }
                    }
                    Key::ShiftTab => {
                        if self.visual_mode {
                            self.unindent_selection();
                            needs_analysis = true;
                        }
                    }
                    k if k == Key::Char('#') => {
                        if self.visual_mode {
                            self.toggle_comment_selection();
                            needs_analysis = true;
                        }
                    }
                    _ => {}
                }

                if self.mode != Mode::Search {
                    if reset_select_all {
                        self.last_key_select_all = false;
                    }
                    if reset_file_bounds {
                        self.last_key_file_bounds = false;
                    }
                    if reset_delete {
                        self.last_key_delete = false;
                    }
                    if reset_copy {
                        self.last_key_copy = false;
                    }
                    self.clamp_cursor();
                }
            }
            Mode::Insert => match key {
                Key::Esc => {
                    self.mode = Mode::Command;
                    self.visual_mode = false;
                    self.state.selection_start = None;
                    self.clamp_cursor();
                }
                Key::Char(c) => {
                    if !c.is_control() {
                        self.handle_insert_char(c, config);
                        needs_analysis = true;
                    }
                }
                Key::Shift(c) => {
                    if !c.is_control() {
                        let final_c = if c.is_ascii_lowercase() {
                            c.to_ascii_uppercase()
                        } else {
                            c
                        };
                        self.handle_insert_char(final_c, config);
                        needs_analysis = true;
                    }
                }
                Key::Enter => {
                    self.prepare_edit(true);
                    self.insert_newline(config);
                    needs_analysis = true;
                }
                Key::CtrlLeft => self.jump_word_backward(false),
                Key::CtrlRight => self.jump_word_forward(false),
                Key::CtrlUp => self.jump_block_backward(false),
                Key::CtrlDown => self.jump_block_forward(false),
                Key::Backspace => {
                    let is_ws = self
                        .char_before_cursor()
                        .map_or(true, |c| c.is_whitespace());
                    if self.state.selection_start.is_some() {
                        self.prepare_edit(is_ws);
                    } else {
                        self.push_undo(EditAction::Char(is_ws));

                        let mut spaces_to_delete = 1;
                        if config.edit_tab_backspace && self.state.cursor_x > 0 {
                            let line = &self.state.lines[self.state.cursor_y];
                            let char_idx = self.state.cursor_x;
                            let before_cursor: String = line.chars().take(char_idx).collect();

                            if before_cursor.trim().is_empty() {
                                let rem = char_idx % 4;
                                spaces_to_delete = if rem == 0 { 4 } else { rem };
                            } else if before_cursor.ends_with("    ") {
                                spaces_to_delete = 4;
                            }
                        }

                        if config.edit_auto_bracket && spaces_to_delete == 1 {
                            let before = self.char_before_cursor();
                            let after = self.char_after_cursor();
                            let is_pair = match (before, after) {
                                (Some('('), Some(')')) => true,
                                (Some('['), Some(']')) => true,
                                (Some('{'), Some('}')) => true,
                                (Some('"'), Some('"')) => true,
                                (Some('\''), Some('\'')) => true,
                                (Some('`'), Some('`')) => true,
                                _ => false,
                            };
                            if is_pair {
                                self.delete_char_after();
                            }
                        }

                        for _ in 0..spaces_to_delete {
                            self.delete_char_before();
                        }
                    }
                    needs_analysis = true;
                }
                Key::Delete => {
                    let is_ws = self.char_after_cursor().map_or(true, |c| c.is_whitespace());
                    if self.state.selection_start.is_some() {
                        self.prepare_edit(is_ws);
                    } else {
                        self.push_undo(EditAction::Char(is_ws));
                        self.delete_char_after();
                    }
                    needs_analysis = true;
                }
                Key::CtrlBackspace | Key::Ctrl('w') | Key::Ctrl('h') => {
                    if self.state.selection_start.is_some() {
                        self.prepare_edit(true);
                    } else {
                        self.push_undo(EditAction::Bulk);
                        self.delete_word_before();
                    }
                    needs_analysis = true;
                }
                Key::CtrlDelete => {
                    if self.state.selection_start.is_some() {
                        self.prepare_edit(true);
                    } else {
                        self.push_undo(EditAction::Bulk);
                        self.delete_word_after();
                    }
                    needs_analysis = true;
                }
                Key::Tab => {
                    self.prepare_edit(true);
                    for _ in 0..4 {
                        self.insert_char(' ');
                    }
                    needs_analysis = true;
                }
                Key::ShiftTab => {
                    self.unindent_current_line();
                    needs_analysis = true;
                }
                Key::Left => self.move_cursor(-1, 0, false),
                Key::Right => self.move_cursor(1, 0, false),
                Key::Up => self.move_cursor(0, -1, false),
                Key::Down => self.move_cursor(0, 1, false),
                Key::ShiftLeft => self.move_cursor(-1, 0, true),
                Key::ShiftRight => self.move_cursor(1, 0, true),
                Key::ShiftUp => self.move_cursor(0, -1, true),
                Key::ShiftDown => self.move_cursor(0, 1, true),
                _ => {}
            },
        }

        if needs_analysis {
            if is_undo_redo {
                self.is_dirty_flag = calculate_hash(&self.state.lines) != self.saved_hash;
            } else if !self.is_dirty_flag {
                self.is_dirty_flag = calculate_hash(&self.state.lines) != self.saved_hash;
            }
            self.refresh_analysis(saved);
        }

        saved
    }

    fn jump_to_line_search(&mut self) {
        if let Ok(line_num) = self.line_search_query.parse::<usize>() {
            if line_num > 0 && line_num <= self.state.lines.len() {
                self.state.cursor_y = line_num - 1;
                self.state.cursor_x = 0;

                let target = self.state.cursor_y;
                let mut to_remove = Vec::new();
                for &fold_start in &self.folded_lines {
                    if target > fold_start && target <= self.get_block_end(fold_start) {
                        to_remove.push(fold_start);
                    }
                }
                for r in to_remove {
                    self.folded_lines.remove(&r);
                }

                self.clamp_cursor();
            }
        }
    }

    fn find_next(&mut self) {
        if self.last_search.is_empty() {
            return;
        }
        let query = &self.last_search;
        let query_char_count = query.chars().count();

        let num_lines = self.state.lines.len();
        let start_y = self.state.cursor_y;
        let mut start_x = self.state.cursor_x;

        for i in 0..=num_lines {
            let y = (start_y + i) % num_lines;
            let line = &self.state.lines[y];

            let offset = if i == 0 { start_x } else { 0 };

            let byte_offset = line
                .char_indices()
                .nth(offset)
                .map(|(b, _)| b)
                .unwrap_or(line.len());

            if let Some(match_byte_idx) = line[byte_offset..].find(query) {
                let absolute_byte_idx = byte_offset + match_byte_idx;
                let match_char_idx = line[..absolute_byte_idx].chars().count();

                self.state.cursor_y = y;
                self.state.cursor_x = match_char_idx + query_char_count;
                self.state.selection_start = Some((match_char_idx, y));
                self.visual_mode = true;
                self.clamp_cursor();
                return;
            }
            start_x = 0;
        }
    }

    fn move_cursor(&mut self, dx: isize, dy: isize, select: bool) {
        if select {
            if self.state.selection_start.is_none() {
                self.state.selection_start = Some((self.state.cursor_x, self.state.cursor_y));
            }
        } else {
            self.state.selection_start = None;
        }

        let d_lines = self.get_display_lines();
        let current_d_idx = d_lines
            .iter()
            .position(|&x| x == self.state.cursor_y as isize)
            .unwrap_or(0);

        let mut target_d_idx = current_d_idx as isize + dy;
        target_d_idx = target_d_idx.clamp(0, d_lines.len().saturating_sub(1) as isize);

        let mut new_y = d_lines[target_d_idx as usize];
        if new_y == -1 {
            if dy > 0 {
                target_d_idx = (target_d_idx + 1).min(d_lines.len() as isize - 1);
            } else if dy < 0 {
                target_d_idx = (target_d_idx - 1).max(0);
            }
            new_y = d_lines[target_d_idx as usize];
        }

        if new_y != -1 {
            self.state.cursor_y = new_y as usize;
        }

        let x = self.state.cursor_x as isize + dx;
        let max_x = self.state.lines[self.state.cursor_y].chars().count();
        let x = x.clamp(0, max_x as isize) as usize;
        self.state.cursor_x = x;
    }

    fn clamp_cursor(&mut self) {
        let max_y = self.state.lines.len().saturating_sub(1);
        if self.state.cursor_y > max_y {
            self.state.cursor_y = max_y;
        }
        let max_x = self.state.lines[self.state.cursor_y].chars().count();
        if self.state.cursor_x > max_x {
            self.state.cursor_x = max_x;
        }
    }

    fn handle_insert_char(&mut self, c: char, config: &Config) {
        let is_step_over = config.edit_auto_bracket
            && ")]}\"'`".contains(c)
            && self.char_after_cursor() == Some(c);

        if is_step_over {
            self.state.cursor_x += 1;
        } else {
            self.prepare_edit(c.is_whitespace());
            self.insert_char(c);

            if config.edit_auto_bracket {
                let closing = match c {
                    '(' => Some(')'),
                    '[' => Some(']'),
                    '{' => Some('}'),
                    '"' => Some('"'),
                    '\'' => Some('\''),
                    '`' => Some('`'),
                    _ => None,
                };
                if let Some(cc) = closing {
                    let next_c = self.char_after_cursor();
                    let should_close = match next_c {
                        Some(nc) => nc.is_whitespace() || ")]},:;".contains(nc),
                        _ => true,
                    };
                    if should_close {
                        self.insert_char(cc);
                        self.state.cursor_x -= 1;
                    }
                }
            }
        }
    }

    fn insert_char(&mut self, c: char) {
        let line = &mut self.state.lines[self.state.cursor_y];
        let byte_idx = line
            .char_indices()
            .nth(self.state.cursor_x)
            .map(|(i, _)| i)
            .unwrap_or(line.len());
        line.insert(byte_idx, c);
        self.state.cursor_x += 1;
    }

    fn insert_newline(&mut self, config: &Config) {
        let line = &mut self.state.lines[self.state.cursor_y];
        let byte_idx = line
            .char_indices()
            .nth(self.state.cursor_x)
            .map(|(i, _)| i)
            .unwrap_or(line.len());
        let new_line = line.split_off(byte_idx);

        let mut indent_spaces = 0;
        if config.edit_auto_indent {
            let before_cursor = &line[..];
            indent_spaces = before_cursor
                .chars()
                .take_while(|c| c.is_whitespace())
                .count();
            if before_cursor.trim_end().ends_with(':') {
                indent_spaces += 4;
            }
        }

        let mut final_new_line = " ".repeat(indent_spaces);
        final_new_line.push_str(&new_line);

        self.shift_folds(self.state.cursor_y + 1, 1);
        self.state
            .lines
            .insert(self.state.cursor_y + 1, final_new_line);
        self.state.cursor_y += 1;
        self.state.cursor_x = indent_spaces;
    }

    fn delete_word_before(&mut self) {
        if self.state.cursor_x == 0 {
            if self.state.cursor_y > 0 {
                self.delete_char_before();
            }
            return;
        }

        let line = &self.state.lines[self.state.cursor_y];
        let byte_idx = line
            .char_indices()
            .nth(self.state.cursor_x)
            .map(|(i, _)| i)
            .unwrap_or(line.len());

        let (before, after) = line.split_at(byte_idx);

        let mut rev_chars = before.chars().rev().peekable();
        let mut deleted_count = 0;

        while let Some(&c) = rev_chars.peek() {
            if c.is_whitespace() {
                deleted_count += 1;
                rev_chars.next();
            } else {
                break;
            }
        }

        if let Some(&c) = rev_chars.peek() {
            let is_word = c.is_alphanumeric() || c == '_';
            while let Some(&next_c) = rev_chars.peek() {
                if next_c.is_whitespace() {
                    break;
                }
                let next_is_word = next_c.is_alphanumeric() || next_c == '_';
                if next_is_word == is_word {
                    deleted_count += 1;
                    rev_chars.next();
                } else {
                    break;
                }
            }
        }

        if deleted_count > 0 {
            let new_cursor_x = self.state.cursor_x - deleted_count;
            let new_byte_idx = line
                .char_indices()
                .nth(new_cursor_x)
                .map(|(i, _)| i)
                .unwrap_or(line.len());

            let mut new_line = String::new();
            new_line.push_str(&line[..new_byte_idx]);
            new_line.push_str(after);

            self.state.lines[self.state.cursor_y] = new_line;
            self.state.cursor_x = new_cursor_x;
        }
    }

    fn delete_word_after(&mut self) {
        let line_len = self.state.lines[self.state.cursor_y].chars().count();
        if self.state.cursor_x >= line_len {
            if self.state.cursor_y < self.state.lines.len().saturating_sub(1) {
                let next_line = self.state.lines.remove(self.state.cursor_y + 1);
                self.shift_folds(self.state.cursor_y + 1, -1);
                self.state.lines[self.state.cursor_y].push_str(&next_line);
            }
            return;
        }

        let line = &self.state.lines[self.state.cursor_y];
        let byte_idx = line
            .char_indices()
            .nth(self.state.cursor_x)
            .map(|(i, _)| i)
            .unwrap_or(line.len());

        let (before, after) = line.split_at(byte_idx);

        let mut chars = after.chars().peekable();
        let mut deleted_count = 0;

        while let Some(&c) = chars.peek() {
            if c.is_whitespace() {
                deleted_count += 1;
                chars.next();
            } else {
                break;
            }
        }

        if let Some(&c) = chars.peek() {
            let is_word = c.is_alphanumeric() || c == '_';
            while let Some(&next_c) = chars.peek() {
                if next_c.is_whitespace() {
                    break;
                }
                let next_is_word = next_c.is_alphanumeric() || next_c == '_';
                if next_is_word == is_word {
                    deleted_count += 1;
                    chars.next();
                } else {
                    break;
                }
            }
        }

        if deleted_count > 0 {
            let delete_end_byte = after
                .char_indices()
                .nth(deleted_count)
                .map(|(i, _)| i)
                .unwrap_or(after.len());

            let mut new_line = String::new();
            new_line.push_str(before);
            new_line.push_str(&after[delete_end_byte..]);

            self.state.lines[self.state.cursor_y] = new_line;
        }
    }

    fn delete_char_before(&mut self) {
        if self.state.cursor_x > 0 {
            let line = &mut self.state.lines[self.state.cursor_y];
            let byte_idx = line
                .char_indices()
                .nth(self.state.cursor_x - 1)
                .map(|(i, _)| i)
                .unwrap();
            line.remove(byte_idx);
            self.state.cursor_x -= 1;
        } else if self.state.cursor_y > 0 {
            self.shift_folds(self.state.cursor_y, -1);
            let current_line = self.state.lines.remove(self.state.cursor_y);
            self.state.cursor_y -= 1;
            let prev_line = &mut self.state.lines[self.state.cursor_y];
            self.state.cursor_x = prev_line.chars().count();
            prev_line.push_str(&current_line);
        }
    }

    fn delete_char_after(&mut self) {
        let line_len = self.state.lines[self.state.cursor_y].chars().count();
        if self.state.cursor_x < line_len {
            let line = &mut self.state.lines[self.state.cursor_y];
            let byte_idx = line
                .char_indices()
                .nth(self.state.cursor_x)
                .map(|(i, _)| i)
                .unwrap();
            line.remove(byte_idx);
        } else if self.state.cursor_y < self.state.lines.len().saturating_sub(1) {
            let next_line = self.state.lines.remove(self.state.cursor_y + 1);
            self.shift_folds(self.state.cursor_y + 1, -1);
            self.state.lines[self.state.cursor_y].push_str(&next_line);
        }
        self.clamp_cursor();
    }

    fn delete_current_line(&mut self) {
        self.state.lines.remove(self.state.cursor_y);
        self.shift_folds(self.state.cursor_y, -1);
        if self.state.lines.is_empty() {
            self.state.lines.push(String::new());
        }
        self.clamp_cursor();
    }

    fn copy_current_line(&mut self) {
        let mut line = self.state.lines[self.state.cursor_y].clone();
        line.push('\n');
        set_clipboard(line);
    }

    fn paste_from_clipboard(&mut self) {
        if let Some(text) = get_clipboard() {
            let lines_to_insert: Vec<&str> = text.split('\n').collect();
            if lines_to_insert.is_empty() {
                return;
            }

            let line = &mut self.state.lines[self.state.cursor_y];
            let byte_idx = line
                .char_indices()
                .nth(self.state.cursor_x)
                .map(|(i, _)| i)
                .unwrap_or(line.len());
            let after = line.split_off(byte_idx);

            if lines_to_insert.len() == 1 {
                let cleaned = lines_to_insert[0].replace('\r', "");
                self.state.lines[self.state.cursor_y].push_str(&cleaned);
                self.state.cursor_x += cleaned.chars().count();
                self.state.lines[self.state.cursor_y].push_str(&after);
            } else {
                let first_cleaned = lines_to_insert[0].replace('\r', "");
                self.state.lines[self.state.cursor_y].push_str(&first_cleaned);

                let mut new_lines = Vec::new();
                for i in 1..lines_to_insert.len() - 1 {
                    new_lines.push(lines_to_insert[i].replace('\r', ""));
                }

                let last_cleaned = lines_to_insert.last().unwrap().replace('\r', "");
                self.state.cursor_x = last_cleaned.chars().count();

                let mut final_line = String::new();
                final_line.push_str(&last_cleaned);
                final_line.push_str(&after);
                new_lines.push(final_line);

                let insert_idx = self.state.cursor_y + 1;
                self.shift_folds(insert_idx, (lines_to_insert.len() - 1) as isize);
                self.state.lines.splice(insert_idx..insert_idx, new_lines);
                self.state.cursor_y += lines_to_insert.len() - 1;
            }
        }
    }

    fn get_selection_bounds(&self) -> Option<((usize, usize), (usize, usize))> {
        if let Some(start) = self.state.selection_start {
            let p1 = (start.0, start.1);
            let p2 = (self.state.cursor_x, self.state.cursor_y);

            let (p_start, p_end) = if (p1.1, p1.0) < (p2.1, p2.0) {
                (p1, p2)
            } else {
                (p2, p1)
            };

            if p_start != p_end {
                Some((p_start, p_end))
            } else {
                None
            }
        } else {
            None
        }
    }

    fn indent_selection(&mut self) {
        if let Some((start, end)) = self.get_selection_bounds() {
            self.push_undo(EditAction::Bulk);
            for y in start.1..=end.1 {
                let mut new_line = String::from("    ");
                new_line.push_str(&self.state.lines[y]);
                self.state.lines[y] = new_line;
            }

            self.state.cursor_x += 4;
            if let Some(sel) = self.state.selection_start.as_mut() {
                sel.0 += 4;
            }

            self.clamp_cursor();
        }
    }

    fn unindent_selection(&mut self) {
        if let Some((start, end)) = self.get_selection_bounds() {
            self.push_undo(EditAction::Bulk);
            for y in start.1..=end.1 {
                let (spaces, byte_idx) = {
                    let line = &self.state.lines[y];
                    let spaces = line.chars().take_while(|c| *c == ' ').count().min(4);
                    let byte_idx = if spaces > 0 {
                        line.char_indices()
                            .nth(spaces)
                            .map(|(i, _)| i)
                            .unwrap_or(line.len())
                    } else {
                        0
                    };
                    (spaces, byte_idx)
                };

                if spaces > 0 {
                    let new_line = self.state.lines[y][byte_idx..].to_string();
                    self.state.lines[y] = new_line;
                }
            }

            self.state.cursor_x = self.state.cursor_x.saturating_sub(4);
            if let Some(sel) = self.state.selection_start.as_mut() {
                sel.0 = sel.0.saturating_sub(4);
            }
            self.clamp_cursor();
        }
    }

    fn unindent_current_line(&mut self) {
        let y = self.state.cursor_y;
        let (spaces, byte_idx) = {
            let line = &self.state.lines[y];
            let spaces = line.chars().take_while(|c| *c == ' ').count().min(4);
            let byte_idx = if spaces > 0 {
                line.char_indices()
                    .nth(spaces)
                    .map(|(i, _)| i)
                    .unwrap_or(line.len())
            } else {
                0
            };
            (spaces, byte_idx)
        };

        if spaces > 0 {
            self.push_undo(EditAction::Char(true));
            let new_line = self.state.lines[y][byte_idx..].to_string();
            self.state.lines[y] = new_line;
            self.state.cursor_x = self.state.cursor_x.saturating_sub(spaces);
            self.clamp_cursor();
        }
    }

    fn delete_selection(&mut self) {
        if let Some((start, end)) = self.get_selection_bounds() {
            if start.1 == end.1 {
                let line = &self.state.lines[start.1];
                let byte_start = line
                    .char_indices()
                    .nth(start.0)
                    .map(|(i, _)| i)
                    .unwrap_or(line.len());
                let byte_end = line
                    .char_indices()
                    .nth(end.0)
                    .map(|(i, _)| i)
                    .unwrap_or(line.len());

                let mut new_line = String::with_capacity(line.len() - (byte_end - byte_start));
                new_line.push_str(&line[..byte_start]);
                new_line.push_str(&line[byte_end..]);
                self.state.lines[start.1] = new_line;
            } else {
                let start_line = &self.state.lines[start.1];
                let byte_start = start_line
                    .char_indices()
                    .nth(start.0)
                    .map(|(i, _)| i)
                    .unwrap_or(start_line.len());
                let mut new_start_line = start_line[..byte_start].to_string();

                let end_line = &self.state.lines[end.1];
                let byte_end = end_line
                    .char_indices()
                    .nth(end.0)
                    .map(|(i, _)| i)
                    .unwrap_or(end_line.len());
                new_start_line.push_str(&end_line[byte_end..]);

                if end.1 > start.1 {
                    self.shift_folds(start.1 + 1, -((end.1 - start.1) as isize));
                    self.state.lines.drain((start.1 + 1)..=end.1);
                }

                self.state.lines[start.1] = new_start_line;
            }

            self.state.cursor_x = start.0;
            self.state.cursor_y = start.1;
            self.state.selection_start = None;
            self.visual_mode = false;

            if self.state.lines.is_empty() {
                self.state.lines.push(String::new());
            }
            self.clamp_cursor();
        }
    }

    fn copy_selection(&mut self) {
        if let Some((start, end)) = self.get_selection_bounds() {
            let mut text = String::new();
            for i in start.1..=end.1 {
                let line = &self.state.lines[i];
                if start.1 == end.1 {
                    let byte_start = line
                        .char_indices()
                        .nth(start.0)
                        .map(|(i, _)| i)
                        .unwrap_or(line.len());
                    let byte_end = line
                        .char_indices()
                        .nth(end.0)
                        .map(|(i, _)| i)
                        .unwrap_or(line.len());
                    text.push_str(&line[byte_start..byte_end]);
                } else {
                    if i == start.1 {
                        let byte_start = line
                            .char_indices()
                            .nth(start.0)
                            .map(|(i, _)| i)
                            .unwrap_or(line.len());
                        text.push_str(&line[byte_start..]);
                        text.push('\n');
                    } else if i == end.1 {
                        let byte_end = line
                            .char_indices()
                            .nth(end.0)
                            .map(|(i, _)| i)
                            .unwrap_or(line.len());
                        text.push_str(&line[..byte_end]);
                    } else {
                        text.push_str(line);
                        text.push('\n');
                    }
                }
            }
            set_clipboard(text);
        }
    }

    fn toggle_comment_selection(&mut self) {
        if let Some((start, end)) = self.get_selection_bounds() {
            self.push_undo(EditAction::Bulk);

            let mut all_commented = true;
            let mut min_indent = usize::MAX;

            for y in start.1..=end.1 {
                let line = &self.state.lines[y];
                if line.trim().is_empty() {
                    continue;
                }
                let indent = line.chars().take_while(|c| c.is_whitespace()).count();
                if indent < min_indent {
                    min_indent = indent;
                }
                if !line[indent..].starts_with('#') {
                    all_commented = false;
                }
            }

            if min_indent == usize::MAX {
                min_indent = 0;
            }

            for y in start.1..=end.1 {
                let line = &mut self.state.lines[y];
                if line.trim().is_empty() && !all_commented {
                    continue;
                }

                let mut removed_len = 0;
                let mut added_len = 0;

                let indent = line.chars().take_while(|c| c.is_whitespace()).count();
                let is_commented = line[indent..].starts_with('#');

                if all_commented {
                    if line[indent..].starts_with("# ") {
                        let byte_idx_hash = line.char_indices().nth(indent).unwrap().0;
                        let byte_idx_after = line
                            .char_indices()
                            .nth(indent + 2)
                            .map(|x| x.0)
                            .unwrap_or(line.len());
                        line.replace_range(byte_idx_hash..byte_idx_after, "");
                        removed_len = 2;
                    } else if line[indent..].starts_with('#') {
                        let byte_idx_hash = line.char_indices().nth(indent).unwrap().0;
                        let byte_idx_after = line
                            .char_indices()
                            .nth(indent + 1)
                            .map(|x| x.0)
                            .unwrap_or(line.len());
                        line.replace_range(byte_idx_hash..byte_idx_after, "");
                        removed_len = 1;
                    }
                } else {
                    if !is_commented {
                        let byte_idx = line
                            .char_indices()
                            .nth(min_indent)
                            .map(|x| x.0)
                            .unwrap_or(line.len());
                        line.insert_str(byte_idx, "# ");
                        added_len = 2;
                    }
                }

                if self.state.cursor_y == y && self.state.cursor_x >= min_indent {
                    self.state.cursor_x = (self.state.cursor_x + added_len)
                        .saturating_sub(removed_len)
                        .max(min_indent);
                }
                if let Some(sel) = self.state.selection_start.as_mut() {
                    if sel.1 == y && sel.0 >= min_indent {
                        sel.0 = (sel.0 + added_len)
                            .saturating_sub(removed_len)
                            .max(min_indent);
                    }
                }
            }
            self.clamp_cursor();
        }
    }

    fn jump_word_forward(&mut self, select: bool) {
        if select {
            if self.state.selection_start.is_none() {
                self.state.selection_start = Some((self.state.cursor_x, self.state.cursor_y));
            }
        } else {
            self.state.selection_start = None;
        }

        let line = &self.state.lines[self.state.cursor_y];
        if self.state.cursor_x >= line.chars().count() {
            let d_lines = self.get_display_lines();
            let current_d_idx = d_lines
                .iter()
                .position(|&x| x == self.state.cursor_y as isize)
                .unwrap_or(0);
            for i in (current_d_idx + 1)..d_lines.len() {
                if d_lines[i] != -1 {
                    self.state.cursor_y = d_lines[i] as usize;
                    self.state.cursor_x = 0;
                    break;
                }
            }
            return;
        }

        let mut chars = line.chars().skip(self.state.cursor_x);
        let mut skipped = 0;
        while let Some(c) = chars.next() {
            if c.is_whitespace() {
                skipped += 1;
                break;
            }
            skipped += 1;
        }
        while let Some(c) = chars.next() {
            if !c.is_whitespace() {
                break;
            }
            skipped += 1;
        }
        self.state.cursor_x += skipped;
        self.clamp_cursor();
    }

    fn jump_word_backward(&mut self, select: bool) {
        if select {
            if self.state.selection_start.is_none() {
                self.state.selection_start = Some((self.state.cursor_x, self.state.cursor_y));
            }
        } else {
            self.state.selection_start = None;
        }

        if self.state.cursor_x == 0 {
            let d_lines = self.get_display_lines();
            let current_d_idx = d_lines
                .iter()
                .position(|&x| x == self.state.cursor_y as isize)
                .unwrap_or(0);
            for i in (0..current_d_idx).rev() {
                if d_lines[i] != -1 {
                    self.state.cursor_y = d_lines[i] as usize;
                    self.state.cursor_x = self.state.lines[self.state.cursor_y].chars().count();
                    break;
                }
            }
            return;
        }

        let line = &self.state.lines[self.state.cursor_y];
        let byte_idx = line
            .char_indices()
            .nth(self.state.cursor_x)
            .map(|(i, _)| i)
            .unwrap_or(line.len());

        let before_cursor = &line[..byte_idx];

        let mut skipped = 0;
        let mut chars_rev = before_cursor.chars().rev();

        while let Some(c) = chars_rev.next() {
            if !c.is_whitespace() {
                skipped += 1;
                break;
            }
            skipped += 1;
        }
        for c in chars_rev {
            if c.is_whitespace() {
                break;
            }
            skipped += 1;
        }

        self.state.cursor_x = self.state.cursor_x.saturating_sub(skipped);
    }

    fn jump_block_forward(&mut self, select: bool) {
        if select {
            if self.state.selection_start.is_none() {
                self.state.selection_start = Some((self.state.cursor_x, self.state.cursor_y));
            }
        } else {
            self.state.selection_start = None;
        }

        let d_lines = self.get_display_lines();
        let current_d_idx = d_lines
            .iter()
            .position(|&x| x == self.state.cursor_y as isize)
            .unwrap_or(0);
        let max_idx = d_lines.len().saturating_sub(1);

        if current_d_idx < max_idx {
            let start_y = d_lines[current_d_idx] as usize;
            let start_is_empty = self.state.lines[start_y].trim().is_empty();

            let mut target_d_idx = current_d_idx + 1;
            while target_d_idx < max_idx {
                let y = d_lines[target_d_idx];
                if y != -1 && self.state.lines[y as usize].trim().is_empty() != start_is_empty {
                    break;
                }
                target_d_idx += 1;
            }

            let mut final_y = d_lines[target_d_idx];
            if final_y == -1 {
                for i in (target_d_idx + 1)..=max_idx {
                    if d_lines[i] != -1 {
                        final_y = d_lines[i];
                        break;
                    }
                }
            }
            if final_y != -1 {
                self.state.cursor_y = final_y as usize;
            }
        }

        self.clamp_cursor();
    }

    fn jump_block_backward(&mut self, select: bool) {
        if select {
            if self.state.selection_start.is_none() {
                self.state.selection_start = Some((self.state.cursor_x, self.state.cursor_y));
            }
        } else {
            self.state.selection_start = None;
        }

        let d_lines = self.get_display_lines();
        let current_d_idx = d_lines
            .iter()
            .position(|&x| x == self.state.cursor_y as isize)
            .unwrap_or(0);

        if current_d_idx > 0 {
            let start_y = d_lines[current_d_idx] as usize;
            let start_is_empty = self.state.lines[start_y].trim().is_empty();

            let mut target_d_idx = current_d_idx - 1;
            while target_d_idx > 0 {
                let y = d_lines[target_d_idx];
                if y != -1 && self.state.lines[y as usize].trim().is_empty() != start_is_empty {
                    break;
                }
                target_d_idx -= 1;
            }

            let mut final_y = d_lines[target_d_idx];
            if final_y == -1 {
                for i in (0..target_d_idx).rev() {
                    if d_lines[i] != -1 {
                        final_y = d_lines[i];
                        break;
                    }
                }
            }
            if final_y != -1 {
                self.state.cursor_y = final_y as usize;
            } else {
                self.state.cursor_y = 0;
            }
        }

        self.clamp_cursor();
    }

    pub fn render(
        &mut self,
        width: u16,
        height: u16,
        is_active: bool,
        config: &Config,
        theme: &Theme,
    ) -> Box {
        let use_border_color = config.indicator_style == "border";

        let mut b = Box::new(
            width,
            height,
            1,
            config.get_border(),
            if is_active && use_border_color {
                theme.selected_box.clone()
            } else {
                theme.main_box.clone()
            },
            Gradient::Solid(Color::None),
            Modifier::None,
        );

        crate::panels::apply_indicator(&mut b, config, theme, is_active);

        if self.is_editing {
            let (mode_str, mode_color) = match self.mode {
                Mode::Command => {
                    if self.visual_mode {
                        ("[VIS]", &theme.editor_vis)
                    } else {
                        ("[CMD]", &theme.editor_cmd)
                    }
                }
                Mode::Search => ("[FND]", &theme.editor_fnd),
                Mode::LineSearch => ("[LNE]", &theme.editor_lne),
                Mode::Insert => ("[INS]", &theme.editor_ins),
            };

            b.insert_text(
                " ",
                1,
                -1,
                false,
                Gradient::Solid(Color::White),
                Gradient::Solid(Color::None),
                Modifier::Bold,
            );
            b.insert_text(
                mode_str,
                2,
                -1,
                false,
                mode_color.clone(),
                Gradient::Solid(Color::None),
                Modifier::Bold,
            );

            if !self.rel_path.is_empty() {
                let start_x = 2 + mode_str.len() as u16;
                let max_insert_len = (width.saturating_sub(start_x + 3)) as usize;

                let mut display_path = self.rel_path.clone();
                let path_chars = display_path.chars().count();

                if path_chars + 2 > max_insert_len {
                    if max_insert_len >= 4 {
                        let keep = max_insert_len - 4;
                        display_path = display_path.chars().take(keep).collect();
                        display_path.push_str("..");
                    } else {
                        display_path.clear();
                    }
                }

                if !display_path.is_empty() {
                    b.insert_text(
                        &format!(" {} ", display_path),
                        start_x as i16,
                        -1,
                        false,
                        theme.main_label.clone(),
                        Gradient::Solid(Color::None),
                        Modifier::Bold,
                    );
                } else {
                    b.insert_text(
                        " ",
                        start_x as i16,
                        -1,
                        false,
                        theme.main_label.clone(),
                        Gradient::Solid(Color::None),
                        Modifier::Bold,
                    );
                }
            } else {
                b.insert_text(
                    " ",
                    2 + mode_str.len() as i16,
                    -1,
                    false,
                    theme.main_label.clone(),
                    Gradient::Solid(Color::None),
                    Modifier::Bold,
                );
            }
        }

        let inner_w = width.saturating_sub(2) as usize;
        let inner_h = height.saturating_sub(2) as usize;

        let text_inner_h = if self.mode == Mode::Search {
            inner_h.saturating_sub(1)
        } else {
            inner_h
        };

        let show_line_numbers = self.file_path.is_some();
        let line_count = self.state.lines.len();
        let max_num_width = if line_count < 10 {
            1
        } else if line_count < 100 {
            2
        } else if line_count < 1000 {
            3
        } else if line_count < 10000 {
            4
        } else if line_count < 100000 {
            5
        } else {
            line_count.to_string().len()
        };
        let prefix_width = if show_line_numbers {
            max_num_width + 1
        } else {
            0
        };

        let text_inner_w = inner_w.saturating_sub(prefix_width);

        let d_lines = self.get_display_lines();

        let mut target_y = self.state.cursor_y;
        let mut target_x = self.state.cursor_x;

        if self.is_output && !config.show_caret && !self.is_waiting_for_input {
            if target_y > 0 && target_y == self.state.lines.len().saturating_sub(1) {
                if self.state.lines[target_y].is_empty() {
                    target_y -= 1;
                    target_x = self.state.lines[target_y].chars().count();
                }
            }
            if target_y < self.state.lines.len() {
                let current_line_len = self.state.lines[target_y].chars().count();
                if target_x > 0 && target_x >= current_line_len {
                    target_x = current_line_len.saturating_sub(1);
                }
            }
        }

        let mut visual_target_x = 0;
        if !self.is_output && target_y < self.state.lines.len() {
            let chars_vec: Vec<char> = self.state.lines[target_y].chars().collect();
            let mut i_char = 0;
            while i_char < chars_vec.len() && i_char < target_x {
                if i_char + 6 <= chars_vec.len()
                    && chars_vec[i_char..i_char + 6].iter().collect::<String>() == "Image:"
                {
                    let mut end_img = i_char + 6;
                    while end_img < chars_vec.len()
                        && (chars_vec[end_img].is_ascii_alphanumeric()
                            || chars_vec[end_img] == '+'
                            || chars_vec[end_img] == '/'
                            || chars_vec[end_img] == '=')
                    {
                        end_img += 1;
                    }
                    if end_img - (i_char + 6) > 10 {
                        visual_target_x += 12;
                        i_char = end_img;
                        continue;
                    }
                }

                let mut cluster = crate::render::canvas::CharCluster::new(chars_vec[i_char]);
                let mut k = i_char + 1;
                while k < chars_vec.len() && crate::render::canvas::is_combining(chars_vec[k]) {
                    cluster.push(chars_vec[k]);
                    k += 1;
                }
                visual_target_x += cluster.width as usize;
                i_char = k;
            }
        } else {
            visual_target_x = target_x;
        }

        let cursor_d_idx = d_lines
            .iter()
            .position(|&x| x == target_y as isize)
            .unwrap_or(0);

        if cursor_d_idx < self.scroll_y {
            self.scroll_y = cursor_d_idx;
        } else if cursor_d_idx >= self.scroll_y + text_inner_h && text_inner_h > 0 {
            self.scroll_y = cursor_d_idx - text_inner_h + 1;
        }

        let mut eff_len = d_lines.len();
        if self.is_output && !config.show_caret && !self.is_waiting_for_input {
            if eff_len > 0 && self.state.lines.last().map_or(false, |l| l.is_empty()) {
                eff_len -= 1;
            }
        }
        let max_scroll_y = eff_len.saturating_sub(text_inner_h);
        if self.scroll_y > max_scroll_y {
            self.scroll_y = max_scroll_y;
        }

        if visual_target_x < self.scroll_x {
            self.scroll_x = visual_target_x;
        } else if visual_target_x >= self.scroll_x + text_inner_w && text_inner_w > 0 {
            self.scroll_x = visual_target_x - text_inner_w + 1;
        }

        let mut max_line_len = if self.is_output {
            self.state
                .lines
                .iter()
                .map(|l| {
                    let mut len = 0;
                    let mut chars = l.chars().peekable();
                    while let Some(c) = chars.next() {
                        if c == '{' {
                            let mut valid = false;
                            let mut lookahead = chars.clone();
                            let mut tag = String::new();
                            while let Some(nc) = lookahead.next() {
                                if nc == '}' {
                                    valid = true;
                                    break;
                                }
                                tag.push(nc);
                            }
                            if valid
                                && (tag.starts_with("Color:")
                                    || tag.starts_with("Modifier:")
                                    || tag.starts_with("Background:"))
                            {
                                for _ in 0..=tag.len() {
                                    chars.next();
                                }
                                continue;
                            }
                        }
                        len += 1;
                    }
                    len
                })
                .max()
                .unwrap_or(0)
        } else {
            self.state
                .lines
                .iter()
                .map(|l| {
                    let mut len = 0;
                    let chars_vec: Vec<char> = l.chars().collect();
                    let mut i_char = 0;
                    while i_char < chars_vec.len() {
                        if i_char + 6 <= chars_vec.len()
                            && chars_vec[i_char..i_char + 6].iter().collect::<String>() == "Image:"
                        {
                            let mut end_img = i_char + 6;
                            while end_img < chars_vec.len()
                                && (chars_vec[end_img].is_ascii_alphanumeric()
                                    || chars_vec[end_img] == '+'
                                    || chars_vec[end_img] == '/'
                                    || chars_vec[end_img] == '=')
                            {
                                end_img += 1;
                            }
                            if end_img - (i_char + 6) > 10 {
                                len += 12;
                                i_char = end_img;
                                continue;
                            }
                        }

                        let mut cluster =
                            crate::render::canvas::CharCluster::new(chars_vec[i_char]);
                        let mut k = i_char + 1;
                        while k < chars_vec.len()
                            && crate::render::canvas::is_combining(chars_vec[k])
                        {
                            cluster.push(chars_vec[k]);
                            k += 1;
                        }
                        len += cluster.width as usize;
                        i_char = k;
                    }
                    len
                })
                .max()
                .unwrap_or(0)
        };

        if self.is_output && self.is_waiting_for_input {
            let input_len = self.input_buffer.chars().count() + 1;
            let cursor_line_len = self.state.cursor_x + input_len;
            if cursor_line_len > max_line_len {
                max_line_len = cursor_line_len;
            }
        }

        let max_scroll_x = max_line_len.saturating_sub(text_inner_w);
        if self.scroll_x > max_scroll_x {
            self.scroll_x = max_scroll_x;
        }

        let selection = self.get_selection_bounds();

        let mut line_chars: Vec<_> = Vec::with_capacity(128);
        let mut syntax_colors = Vec::with_capacity(128);
        let mut syntax_bg_colors = Vec::with_capacity(128);

        for (display_idx, &actual_y) in d_lines
            .iter()
            .enumerate()
            .skip(self.scroll_y)
            .take(text_inner_h)
        {
            let display_y = (display_idx - self.scroll_y) as i16;

            if actual_y == -1 {
                if show_line_numbers {
                    let dot_str = ".".repeat(max_num_width);
                    let prefix_style = Style {
                        fg: Color::DarkGray,
                        bg: Color::None,
                        md: Modifier::Dim,
                    };
                    for (idx, c) in dot_str.chars().enumerate() {
                        if idx < inner_w {
                            b.put_cell(
                                crate::Cell::new(c, prefix_style),
                                idx as u16 + 1,
                                display_y as u16 + 1,
                            );
                        }
                    }
                }
                continue;
            }

            let i = actual_y as usize;

            let mut is_output_err_header = false;
            let mut is_output_err_line = false;

            if self.is_output {
                let text = self.state.lines[i].as_str();
                if text == "Syntax Errors:"
                    || text == "Analysis Errors:"
                    || text == "Runtime Errors:"
                {
                    is_output_err_header = true;
                } else if text.starts_with("Line ") && text.contains(':') {
                    if let Some(colon_idx) = text.find(':') {
                        if text[5..colon_idx].chars().all(|c| c.is_ascii_digit()) {
                            is_output_err_line = true;
                        }
                    }
                }
            }

            let is_lne_highlight = self.mode == Mode::LineSearch
                && !self.line_search_query.is_empty()
                && (i + 1).to_string() == self.line_search_query;

            if show_line_numbers {
                let prefix_str = format!("{:>w$}", i + 1, w = max_num_width);

                let prefix_style = Style {
                    fg: Color::DarkGray,
                    bg: Color::None,
                    md: Modifier::Dim,
                };

                for (idx, c) in prefix_str.chars().enumerate() {
                    if idx < inner_w {
                        let mut s = prefix_style;
                        if is_lne_highlight {
                            s.fg = theme.editor_lne.color_at(idx, max_num_width);
                            s.md = Modifier::Bold;
                        }
                        b.put_cell(crate::Cell::new(c, s), idx as u16 + 1, display_y as u16 + 1);
                    }
                }
            }

            let is_error_line = self.is_editing && self.error_lines.contains(&(i + 1));
            let error_underline = is_error_line && config.edit_error_highlight == "underline";
            let error_bg = is_error_line && !error_underline;

            line_chars.clear();
            syntax_colors.clear();
            syntax_bg_colors.clear();
            let mut syntax_modifiers = Vec::with_capacity(128);

            let mut display_line = String::new();

            if !self.is_output {
                let chars_vec: Vec<char> = self.state.lines[i].chars().collect();
                let mut i_char = 0;
                while i_char < chars_vec.len() {
                    if i_char + 6 <= chars_vec.len()
                        && chars_vec[i_char..i_char + 6].iter().collect::<String>() == "Image:"
                    {
                        let start_img = i_char;
                        let mut end_img = i_char + 6;
                        while end_img < chars_vec.len()
                            && (chars_vec[end_img].is_ascii_alphanumeric()
                                || chars_vec[end_img] == '+'
                                || chars_vec[end_img] == '/'
                                || chars_vec[end_img] == '=')
                        {
                            end_img += 1;
                        }

                        let b64_len = end_img - (start_img + 6);
                        if b64_len > 10 {
                            let collapsed = format!(
                                "Image:{}..{}",
                                chars_vec[start_img + 6..start_img + 8]
                                    .iter()
                                    .collect::<String>(),
                                chars_vec[end_img - 2..end_img].iter().collect::<String>()
                            );
                            display_line.push_str(&collapsed);
                            i_char = end_img;
                            continue;
                        }
                    }
                    display_line.push(chars_vec[i_char]);
                    i_char += 1;
                }
            } else {
                display_line = self.state.lines[i].clone();
            }

            let mut line_str = display_line.as_str();
            if self.folded_lines.contains(&i) {
                if let Some(pos) = line_str.rfind(':') {
                    line_str = &line_str[..pos];
                }
            }

            if self.is_output {
                let mut chars = line_str.chars().peekable();
                let mut current_color = Color::Default;
                let mut current_bg_color = Color::None;
                let mut current_modifier = Modifier::None;

                while let Some(c) = chars.next() {
                    if c == '{' {
                        let mut valid_tag = String::new();
                        let mut is_valid = false;
                        let mut lookahead = chars.clone();
                        while let Some(nc) = lookahead.next() {
                            if nc == '}' {
                                is_valid = true;
                                break;
                            }
                            valid_tag.push(nc);
                        }

                        if is_valid {
                            if valid_tag.starts_with("Color:") {
                                let color_name = &valid_tag[6..];
                                if let Ok(parsed_color) =
                                    crate::theme::themecore::parse_color(color_name)
                                {
                                    current_color = parsed_color;
                                    for _ in 0..=valid_tag.len() {
                                        chars.next();
                                    }
                                    continue;
                                }
                            } else if valid_tag.starts_with("Background:") {
                                let color_name = &valid_tag[11..];
                                if let Ok(parsed_color) =
                                    crate::theme::themecore::parse_color(color_name)
                                {
                                    current_bg_color = parsed_color;
                                    for _ in 0..=valid_tag.len() {
                                        chars.next();
                                    }
                                    continue;
                                }
                            } else if valid_tag.starts_with("Modifier:") {
                                let mod_name = &valid_tag[9..];
                                let parsed_mod = match mod_name {
                                    "None" => Some(Modifier::None),
                                    "Bold" => Some(Modifier::Bold),
                                    "Dim" => Some(Modifier::Dim),
                                    "Italic" => Some(Modifier::Italic),
                                    "Underline" => Some(Modifier::Underline),
                                    "Reverse" => Some(Modifier::Reverse),
                                    "Strikethrough" => Some(Modifier::Strikethrough),
                                    _ => None,
                                };
                                if let Some(m) = parsed_mod {
                                    current_modifier = m;
                                    for _ in 0..=valid_tag.len() {
                                        chars.next();
                                    }
                                    continue;
                                }
                            }
                        }
                    }
                    line_chars.push(c);
                    syntax_colors.push(current_color);
                    syntax_bg_colors.push(current_bg_color);
                    syntax_modifiers.push(current_modifier);
                }
            } else {
                line_chars.extend(line_str.chars());
            }

            if self.is_output && self.is_waiting_for_input && i == self.state.cursor_y {
                let last_color = syntax_colors.last().copied().unwrap_or(Color::Default);
                let last_bg = syntax_bg_colors.last().copied().unwrap_or(Color::None);
                let last_modifier = syntax_modifiers.last().copied().unwrap_or(Modifier::None);
                let pad_spaces = self.state.cursor_x.saturating_sub(line_chars.len());
                for _ in 0..pad_spaces {
                    line_chars.push(' ');
                    syntax_colors.push(last_color);
                    syntax_bg_colors.push(last_bg);
                    syntax_modifiers.push(last_modifier);
                }

                let insert_idx = self.state.cursor_x;

                let mut input_chars = Vec::new();
                for c in self.input_buffer.chars() {
                    input_chars.push(c);
                }
                if self.last_blink_state {
                    input_chars.push('_');
                } else {
                    input_chars.push(' ');
                }

                for (offset, c) in input_chars.into_iter().enumerate() {
                    line_chars.insert(insert_idx + offset, c);
                    syntax_colors.insert(insert_idx + offset, last_color);
                    syntax_bg_colors.insert(insert_idx + offset, last_bg);
                    syntax_modifiers.insert(insert_idx + offset, last_modifier);
                }
            }

            if !self.is_output {
                let mut idx = 0;
                while idx < line_chars.len() {
                    let c = line_chars[idx];
                    if c == '"' || c == '\'' || c == '`' {
                        let quote_char = c;
                        let start = idx;
                        idx += 1;
                        while idx < line_chars.len() {
                            let sc = line_chars[idx];
                            idx += 1;
                            if sc == '\\' && idx < line_chars.len() {
                                idx += 1;
                            } else if sc == quote_char {
                                break;
                            }
                        }

                        let mut k = start;
                        while k < idx {
                            if line_chars[k] == '{' && (k == start || line_chars[k - 1] != '\\') {
                                syntax_colors.push(theme.editor_brackets.color_at(0, 1));
                                syntax_bg_colors.push(Color::None);
                                syntax_modifiers.push(Modifier::None);
                                k += 1;

                                let interp_start = k;
                                let mut interp_end = k;
                                let mut depth = 1;
                                while interp_end < idx {
                                    if line_chars[interp_end] == '{' {
                                        depth += 1;
                                    } else if line_chars[interp_end] == '}' {
                                        depth -= 1;
                                        if depth == 0 {
                                            break;
                                        }
                                    }
                                    interp_end += 1;
                                }

                                let mut p = interp_start;
                                while p < interp_end {
                                    let is_color = p + 6 <= interp_end
                                        && line_chars[p..p + 6].iter().collect::<String>()
                                            == "Color:";
                                    let is_bg = p + 11 <= interp_end
                                        && line_chars[p..p + 11].iter().collect::<String>()
                                            == "Background:";

                                    if is_color || is_bg {
                                        let color_word_start = p;
                                        let var_start = if is_color { p + 6 } else { p + 11 };
                                        let mut var_end = var_start;
                                        while var_end < interp_end
                                            && (line_chars[var_end].is_ascii_alphanumeric()
                                                || line_chars[var_end] == '_')
                                        {
                                            var_end += 1;
                                        }
                                        let variant_str: String =
                                            line_chars[var_start..var_end].iter().collect();

                                        let custom_c = if let Ok(num) = variant_str.parse::<f64>() {
                                            if num.fract() == 0.0 && num >= 0.0 && num <= 999999.0 {
                                                crate::theme::themecore::parse_color(&format!(
                                                    "{:06}",
                                                    num as u64
                                                ))
                                                .ok()
                                            } else {
                                                None
                                            }
                                        } else {
                                            crate::theme::themecore::parse_color(&variant_str).ok()
                                        };

                                        if let Some(cc) = custom_c {
                                            for _ in color_word_start..var_end {
                                                if is_bg {
                                                    syntax_colors.push(Color::Black);
                                                    syntax_bg_colors.push(cc);
                                                } else {
                                                    syntax_colors.push(cc);
                                                    syntax_bg_colors.push(Color::None);
                                                }
                                                syntax_modifiers.push(Modifier::None);
                                            }
                                        } else {
                                            let prefix_len = if is_color { 5 } else { 10 };
                                            for _ in 0..prefix_len {
                                                syntax_colors
                                                    .push(theme.editor_keywords.color_at(0, 1));
                                                syntax_bg_colors.push(Color::None);
                                                syntax_modifiers.push(Modifier::None);
                                            }
                                            syntax_colors
                                                .push(theme.editor_operators.color_at(0, 1));
                                            syntax_bg_colors.push(Color::None);
                                            syntax_modifiers.push(Modifier::None);
                                            for _ in var_start..var_end {
                                                syntax_colors
                                                    .push(theme.editor_variables.color_at(0, 1));
                                                syntax_bg_colors.push(Color::None);
                                                syntax_modifiers.push(Modifier::None);
                                            }
                                        }
                                        p = var_end;
                                        continue;
                                    }

                                    if p + 9 <= interp_end {
                                        let possible_mod: String =
                                            line_chars[p..p + 9].iter().collect();
                                        if possible_mod == "Modifier:" {
                                            let var_start = p + 9;
                                            let mut var_end = var_start;
                                            while var_end < interp_end
                                                && (line_chars[var_end].is_ascii_alphanumeric()
                                                    || line_chars[var_end] == '_')
                                            {
                                                var_end += 1;
                                            }
                                            let variant_str: String =
                                                line_chars[var_start..var_end].iter().collect();
                                            let custom_mod = match variant_str.as_str() {
                                                "None" => Some(Modifier::None),
                                                "Bold" => Some(Modifier::Bold),
                                                "Dim" => Some(Modifier::Dim),
                                                "Italic" => Some(Modifier::Italic),
                                                "Underline" => Some(Modifier::Underline),
                                                "Reverse" => Some(Modifier::Reverse),
                                                "Strikethrough" => Some(Modifier::Strikethrough),
                                                _ => None,
                                            };
                                            if let Some(m) = custom_mod {
                                                for k in p..(p + 8) {
                                                    syntax_colors.push(
                                                        theme.editor_keywords.color_at(k - p, 8),
                                                    );
                                                    syntax_bg_colors.push(Color::None);
                                                    syntax_modifiers.push(m);
                                                }
                                                syntax_colors
                                                    .push(theme.editor_operators.color_at(0, 1));
                                                syntax_bg_colors.push(Color::None);
                                                syntax_modifiers.push(m);
                                                for k in var_start..var_end {
                                                    syntax_colors.push(
                                                        theme.editor_keywords.color_at(
                                                            k - var_start,
                                                            var_end - var_start,
                                                        ),
                                                    );
                                                    syntax_bg_colors.push(Color::None);
                                                    syntax_modifiers.push(m);
                                                }
                                                p = var_end;
                                                continue;
                                            } else {
                                                for _ in 0..8 {
                                                    syntax_colors
                                                        .push(theme.editor_keywords.color_at(0, 1));
                                                    syntax_bg_colors.push(Color::None);
                                                    syntax_modifiers.push(Modifier::None);
                                                }
                                                syntax_colors
                                                    .push(theme.editor_operators.color_at(0, 1));
                                                syntax_bg_colors.push(Color::None);
                                                syntax_modifiers.push(Modifier::None);
                                                for _ in var_start..var_end {
                                                    syntax_colors.push(
                                                        theme.editor_variables.color_at(0, 1),
                                                    );
                                                    syntax_bg_colors.push(Color::None);
                                                    syntax_modifiers.push(Modifier::None);
                                                }
                                                p = var_end;
                                                continue;
                                            }
                                        }
                                    }

                                    syntax_colors.push(theme.editor_variables.color_at(0, 1));
                                    syntax_bg_colors.push(Color::None);
                                    syntax_modifiers.push(Modifier::None);
                                    p += 1;
                                }

                                if interp_end < idx && line_chars[interp_end] == '}' {
                                    syntax_colors.push(theme.editor_brackets.color_at(0, 1));
                                    syntax_bg_colors.push(Color::None);
                                    syntax_modifiers.push(Modifier::None);
                                    k = interp_end + 1;
                                } else {
                                    k = interp_end;
                                }
                            } else {
                                syntax_colors
                                    .push(theme.editor_strings.color_at(k - start, idx - start));
                                syntax_bg_colors.push(Color::None);
                                syntax_modifiers.push(Modifier::None);
                                k += 1;
                            }
                        }
                    } else if c == '#' {
                        let start = idx;
                        let end = line_chars.len();
                        for k in start..end {
                            syntax_colors
                                .push(theme.editor_comments.color_at(k - start, end - start));
                            syntax_bg_colors.push(Color::None);
                            syntax_modifiers.push(Modifier::None);
                        }
                        break;
                    } else if c.is_ascii_digit() || c.is_ascii_alphabetic() || c == '_' {
                        let start = idx;
                        let mut is_number = c.is_ascii_digit();

                        let mut is_enum_prefix = false;
                        if start >= 1 {
                            let mut k = start;
                            while k > 0 && line_chars[k - 1].is_whitespace() {
                                k -= 1;
                            }
                            if k > 0 && line_chars[k - 1] == ':' {
                                k -= 1;
                                while k > 0 && line_chars[k - 1].is_whitespace() {
                                    k -= 1;
                                }
                                let end_prev = k;
                                while k > 0
                                    && (line_chars[k - 1].is_ascii_alphanumeric()
                                        || line_chars[k - 1] == '_')
                                {
                                    k -= 1;
                                }
                                let prev_word: String = line_chars[k..end_prev].iter().collect();
                                if prev_word == "Key"
                                    || prev_word == "Modifier"
                                    || prev_word == "Color"
                                    || prev_word == "Background"
                                    || prev_word == "Image"
                                {
                                    is_enum_prefix = true;
                                }
                            }
                        }

                        if is_number && is_enum_prefix {
                            is_number = false;
                        }

                        if is_number {
                            let mut temp_idx = idx;
                            while temp_idx < line_chars.len()
                                && line_chars[temp_idx].is_ascii_digit()
                            {
                                temp_idx += 1;
                            }
                            let mut has_dot = false;
                            if temp_idx < line_chars.len() && line_chars[temp_idx] == '.' {
                                has_dot = true;
                                temp_idx += 1;
                                while temp_idx < line_chars.len()
                                    && line_chars[temp_idx].is_ascii_digit()
                                {
                                    temp_idx += 1;
                                }
                            }
                            if !has_dot
                                && temp_idx < line_chars.len()
                                && (line_chars[temp_idx].is_ascii_alphabetic()
                                    || line_chars[temp_idx] == '_')
                            {
                                is_number = false;
                            } else {
                                idx = temp_idx;
                            }
                        }

                        if is_number {
                            for k in start..idx {
                                syntax_colors
                                    .push(theme.editor_numbers.color_at(k - start, idx - start));
                                syntax_bg_colors.push(Color::None);
                                syntax_modifiers.push(Modifier::None);
                            }
                        } else {
                            while idx < line_chars.len()
                                && (line_chars[idx].is_ascii_alphanumeric()
                                    || line_chars[idx] == '_'
                                    || (is_enum_prefix
                                        && (line_chars[idx] == '+'
                                            || line_chars[idx] == '/'
                                            || line_chars[idx] == '='
                                            || line_chars[idx] == '.')))
                            {
                                idx += 1;
                            }
                            let word_chars = &line_chars[start..idx];
                            let word_str: String = word_chars.iter().collect();

                            if word_str == "Color" || word_str == "Background" {
                                let is_bg = word_str == "Background";
                                let mut temp_idx = idx;
                                while temp_idx < line_chars.len()
                                    && line_chars[temp_idx].is_whitespace()
                                {
                                    temp_idx += 1;
                                }
                                if temp_idx < line_chars.len() && line_chars[temp_idx] == ':' {
                                    temp_idx += 1;
                                    while temp_idx < line_chars.len()
                                        && line_chars[temp_idx].is_whitespace()
                                    {
                                        temp_idx += 1;
                                    }
                                    let variant_start = temp_idx;
                                    while temp_idx < line_chars.len()
                                        && (line_chars[temp_idx].is_ascii_alphanumeric()
                                            || line_chars[temp_idx] == '_')
                                    {
                                        temp_idx += 1;
                                    }
                                    if temp_idx > variant_start {
                                        let variant_str: String =
                                            line_chars[variant_start..temp_idx].iter().collect();
                                        let custom_color = if let Ok(num) =
                                            variant_str.parse::<f64>()
                                        {
                                            if num.fract() == 0.0 && num >= 0.0 && num <= 999999.0 {
                                                crate::theme::themecore::parse_color(&format!(
                                                    "{:06}",
                                                    num as u64
                                                ))
                                                .ok()
                                            } else {
                                                None
                                            }
                                        } else {
                                            crate::theme::themecore::parse_color(&variant_str).ok()
                                        };

                                        if let Some(cc) = custom_color {
                                            for _ in start..temp_idx {
                                                if is_bg {
                                                    syntax_colors.push(Color::Black);
                                                    syntax_bg_colors.push(cc);
                                                } else {
                                                    syntax_colors.push(cc);
                                                    syntax_bg_colors.push(Color::None);
                                                }
                                                syntax_modifiers.push(Modifier::None);
                                            }
                                            idx = temp_idx;
                                            continue;
                                        }
                                    }
                                }
                            } else if word_str == "Modifier" {
                                let mut temp_idx = idx;
                                while temp_idx < line_chars.len()
                                    && line_chars[temp_idx].is_whitespace()
                                {
                                    temp_idx += 1;
                                }
                                if temp_idx < line_chars.len() && line_chars[temp_idx] == ':' {
                                    temp_idx += 1;
                                    while temp_idx < line_chars.len()
                                        && line_chars[temp_idx].is_whitespace()
                                    {
                                        temp_idx += 1;
                                    }
                                    let variant_start = temp_idx;
                                    while temp_idx < line_chars.len()
                                        && (line_chars[temp_idx].is_ascii_alphanumeric()
                                            || line_chars[temp_idx] == '_')
                                    {
                                        temp_idx += 1;
                                    }
                                    if temp_idx > variant_start {
                                        let variant_str: String =
                                            line_chars[variant_start..temp_idx].iter().collect();
                                        let custom_mod = match variant_str.as_str() {
                                            "None" => Some(Modifier::None),
                                            "Bold" => Some(Modifier::Bold),
                                            "Dim" => Some(Modifier::Dim),
                                            "Italic" => Some(Modifier::Italic),
                                            "Underline" => Some(Modifier::Underline),
                                            "Reverse" => Some(Modifier::Reverse),
                                            "Strikethrough" => Some(Modifier::Strikethrough),
                                            _ => None,
                                        };
                                        if let Some(m) = custom_mod {
                                            for k in start..idx {
                                                syntax_colors.push(
                                                    theme
                                                        .editor_keywords
                                                        .color_at(k - start, idx - start),
                                                );
                                                syntax_bg_colors.push(Color::None);
                                                syntax_modifiers.push(m);
                                            }
                                            for _ in idx..variant_start {
                                                syntax_colors
                                                    .push(theme.editor_operators.color_at(0, 1));
                                                syntax_bg_colors.push(Color::None);
                                                syntax_modifiers.push(m);
                                            }
                                            for k in variant_start..temp_idx {
                                                syntax_colors.push(theme.editor_keywords.color_at(
                                                    k - variant_start,
                                                    temp_idx - variant_start,
                                                ));
                                                syntax_bg_colors.push(Color::None);
                                                syntax_modifiers.push(m);
                                            }
                                            idx = temp_idx;
                                            continue;
                                        }
                                    }
                                }
                            }

                            let is_kw = matches!(
                                word_str.as_str(),
                                "let"
                                    | "const"
                                    | "fn"
                                    | "return"
                                    | "loop"
                                    | "while"
                                    | "for"
                                    | "in"
                                    | "if"
                                    | "elif"
                                    | "else"
                                    | "break"
                                    | "continue"
                                    | "async"
                            );
                            let is_op_word = matches!(word_str.as_str(), "and" | "or" | "not");
                            let is_bool = matches!(word_str.as_str(), "True" | "False");

                            let is_enum_base = word_str == "Color"
                                || word_str == "Key"
                                || word_str == "Modifier"
                                || word_str == "Background"
                                || word_str == "Image";

                            let is_enum_variant = is_enum_prefix;

                            let mut skip_idx = idx;
                            while skip_idx < line_chars.len()
                                && line_chars[skip_idx].is_whitespace()
                            {
                                skip_idx += 1;
                            }

                            let is_builtin_method = matches!(
                                word_str.as_str(),
                                "len"
                                    | "append"
                                    | "clear"
                                    | "count"
                                    | "extend"
                                    | "index"
                                    | "insert"
                                    | "pop"
                                    | "remove"
                                    | "get"
                                    | "keys"
                                    | "values"
                                    | "update"
                                    | "set"
                                    | "capitalize"
                                    | "lower"
                                    | "upper"
                                    | "swapcase"
                                    | "trim"
                                    | "join"
                                    | "split"
                                    | "replace"
                                    | "startswith"
                                    | "endswith"
                                    | "asnum"
                                    | "abs"
                                    | "neg"
                                    | "floor"
                                    | "trunc"
                                    | "ceil"
                                    | "fract"
                                    | "clamp"
                                    | "round"
                                    | "pow"
                                    | "sqrt"
                                    | "tostring"
                            );

                            let is_func = skip_idx < line_chars.len()
                                && line_chars[skip_idx] == '('
                                && (self.defined_functions.contains(&word_str)
                                    || is_builtin_method);

                            let color = if is_kw || is_enum_base || is_enum_variant {
                                &theme.editor_keywords
                            } else if is_op_word {
                                &theme.editor_operators
                            } else if is_bool {
                                &theme.editor_bool
                            } else if is_func {
                                &theme.editor_functions
                            } else {
                                &theme.editor_variables
                            };

                            for k in start..idx {
                                syntax_colors.push(color.color_at(k - start, idx - start));
                                syntax_bg_colors.push(Color::None);
                                syntax_modifiers.push(Modifier::None);
                            }
                        }
                    } else if "+-=*/<>!:".contains(c) {
                        let start = idx;
                        idx += 1;
                        while idx < line_chars.len() && "+-=*/<>!:".contains(line_chars[idx]) {
                            idx += 1;
                        }
                        for k in start..idx {
                            syntax_colors
                                .push(theme.editor_operators.color_at(k - start, idx - start));
                            syntax_bg_colors.push(Color::None);
                            syntax_modifiers.push(Modifier::None);
                        }
                    } else if "()[]{}".contains(c) {
                        syntax_colors.push(theme.editor_brackets.color_at(0, 1));
                        syntax_bg_colors.push(Color::None);
                        syntax_modifiers.push(Modifier::None);
                        idx += 1;
                    } else {
                        syntax_colors.push(theme.main_label.color_at(0, 1));
                        syntax_bg_colors.push(Color::None);
                        syntax_modifiers.push(Modifier::None);
                        idx += 1;
                    }
                }
            }

            let mut current_x = 0;
            let mut j = 0;
            while j < line_chars.len() {
                let mut cluster = crate::render::canvas::CharCluster::new(line_chars[j]);
                let mut k = j + 1;
                while k < line_chars.len() && crate::render::canvas::is_combining(line_chars[k]) {
                    cluster.push(line_chars[k]);
                    k += 1;
                }

                let char_w = cluster.width as usize;

                if current_x + char_w > self.scroll_x && current_x < self.scroll_x + text_inner_w {
                    let display_x = (current_x.saturating_sub(self.scroll_x) + prefix_width) as i16;

                    let is_selected = if let Some((start, end)) = selection {
                        let is_after_start = i > start.1 || (i == start.1 && j >= start.0);
                        let is_before_end = i < end.1 || (i == end.1 && j < end.0);
                        is_after_start && is_before_end
                    } else {
                        false
                    };

                    let mut style = Style {
                        fg: if is_output_err_header {
                            theme.editor_errors.color_at(j, line_chars.len())
                        } else if is_output_err_line {
                            let colon_idx = line_chars.iter().position(|&x| x == ':').unwrap_or(0);
                            if j <= colon_idx {
                                theme.editor_errors.color_at(j, colon_idx + 1)
                            } else if j < syntax_colors.len() {
                                let c = syntax_colors[j];
                                if self.is_output && c == Color::Default {
                                    theme.macro_output.color_at(0, 1)
                                } else {
                                    c
                                }
                            } else {
                                if self.is_output {
                                    theme.macro_output.color_at(0, 1)
                                } else {
                                    Color::White
                                }
                            }
                        } else if error_underline {
                            theme.editor_errors.color_at(display_x as usize, inner_w)
                        } else if j < syntax_colors.len() {
                            let c = syntax_colors[j];
                            if c == Color::Default {
                                theme.macro_output.color_at(0, 1)
                            } else {
                                c
                            }
                        } else {
                            if self.is_output {
                                theme.macro_output.color_at(0, 1)
                            } else {
                                Color::White
                            }
                        },
                        bg: if is_selected {
                            Color::DarkGray
                        } else if error_bg {
                            theme.editor_errors.color_at(display_x as usize, inner_w)
                        } else if j < syntax_bg_colors.len() && syntax_bg_colors[j] != Color::None {
                            let c = syntax_bg_colors[j];
                            if c == Color::Default { Color::None } else { c }
                        } else {
                            Color::None
                        },
                        md: if !is_active {
                            Modifier::Dim
                        } else if is_output_err_header {
                            Modifier::Bold
                        } else if error_underline {
                            Modifier::Underline
                        } else if j < syntax_modifiers.len()
                            && syntax_modifiers[j] != Modifier::None
                        {
                            syntax_modifiers[j]
                        } else {
                            Modifier::None
                        },
                    };

                    let mut is_cursor = false;
                    if self.mode != Mode::Search && self.mode != Mode::LineSearch {
                        let show_caret = self.is_editing
                            || (self.is_output && config.show_caret && !self.is_waiting_for_input);
                        if show_caret && i == target_y && target_x >= j && target_x < k {
                            is_cursor = true;
                        }
                    }

                    if is_cursor {
                        style.bg = theme.caret_color.color_at(0, 1);
                        style.fg = Color::Black;
                    }

                    let display_c = if cluster.c.is_control() {
                        ' '
                    } else {
                        cluster.c
                    };

                    if (display_x as usize) < inner_w {
                        b.put_cell(
                            crate::Cell {
                                c: display_c,
                                ext: cluster.ext,
                                ext_len: cluster.ext_len,
                                width: cluster.width,
                                s: style,
                            },
                            display_x as u16 + 1,
                            display_y as u16 + 1,
                        );
                        if cluster.width == 2 && (display_x as usize) + 1 < inner_w {
                            b.put_cell(
                                crate::Cell::dummy(),
                                display_x as u16 + 2,
                                display_y as u16 + 1,
                            );
                        }
                    }
                }

                current_x += char_w;
                j = k;
            }

            if self.mode != Mode::Search && self.mode != Mode::LineSearch {
                let show_caret = self.is_editing
                    || (self.is_output && config.show_caret && !self.is_waiting_for_input);
                if show_caret && i == target_y && target_x >= line_chars.len() {
                    if current_x >= self.scroll_x {
                        let display_x = current_x - self.scroll_x + prefix_width;
                        if display_x < inner_w {
                            b.put_cell(
                                crate::Cell::new(
                                    ' ',
                                    Style {
                                        fg: Color::Black,
                                        bg: theme.caret_color.color_at(0, 1),
                                        md: if is_active {
                                            Modifier::None
                                        } else {
                                            Modifier::Dim
                                        },
                                    },
                                ),
                                display_x as u16 + 1,
                                display_y as u16 + 1,
                            );
                        }
                    }
                }
            }
        }

        if self.mode == Mode::Search {
            let bar_y = text_inner_h as u16 + 1;
            let mut bar_text = format!(":{}", self.search_query);

            let char_count = bar_text.chars().count();
            if char_count < inner_w {
                bar_text.push('_');
                let spaces = inner_w.saturating_sub(char_count + 1);
                bar_text.push_str(&" ".repeat(spaces));
            } else {
                let skip = char_count.saturating_sub(inner_w) + 1;
                bar_text = bar_text.chars().skip(skip).collect();
                bar_text.push('_');
            }

            let bar_bg = &theme.editor_fnd_bg;

            for (x, c) in bar_text.chars().enumerate().take(inner_w) {
                b.put_cell(
                    crate::Cell::new(
                        c,
                        Style {
                            fg: Color::Black,
                            bg: bar_bg.color_at(x, inner_w),
                            md: Modifier::None,
                        },
                    ),
                    x as u16 + 1,
                    bar_y,
                );
            }
        }

        if self.is_editing && self.error_count > 0 {
            let err_str = format!(" ERR {} ", self.error_count);
            let err_len = err_str.chars().count();
            let mut x = 2;
            for (i, c) in err_str.chars().enumerate() {
                if x < width.saturating_sub(1) {
                    b.put_cell(
                        crate::Cell::new(
                            c,
                            Style {
                                fg: theme.editor_errors.color_at(i, err_len),
                                bg: Color::None,
                                md: Modifier::Bold,
                            },
                        ),
                        x,
                        height.saturating_sub(1),
                    );
                    x += 1;
                }
            }
        }

        b
    }
}
