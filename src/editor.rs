use arboard::Clipboard;
use std::fs;
use std::path::PathBuf;

use crate::{Border, Box, Color, Key, Modifier, Style};

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Command,
    Insert,
}

#[derive(Clone)]
pub struct EditorState {
    pub lines: Vec<String>,
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub selection_start: Option<(usize, usize)>,
}

pub struct Editor {
    pub is_editing: bool,
    pub mode: Mode,
    pub state: EditorState,
    pub scroll_x: usize,
    pub scroll_y: usize,
    pub file_path: Option<PathBuf>,
    pub rel_path: String,
    pub clipboard: Option<Clipboard>,

    pub undo_stack: Vec<EditorState>,
    pub redo_stack: Vec<EditorState>,

    pub last_key_g: bool,
    pub last_key_d: bool,
    pub last_key_y: bool,
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
            mode: Mode::Command,
            state: EditorState {
                lines: vec![String::new()],
                cursor_x: 0,
                cursor_y: 0,
                selection_start: None,
            },
            scroll_x: 0,
            scroll_y: 0,
            file_path: None,
            rel_path: String::new(),
            clipboard: Clipboard::new().ok(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            last_key_g: false,
            last_key_d: false,
            last_key_y: false,
        }
    }

    pub fn load_file(&mut self, path: PathBuf, edit: bool, rel_path: String) {
        match fs::read_to_string(&path) {
            Ok(content) => {
                let lines: Vec<String> = if content.is_empty() {
                    vec![String::new()]
                } else {
                    content.lines().map(|s| s.to_string()).collect()
                };
                self.state = EditorState {
                    lines,
                    cursor_x: 0,
                    cursor_y: 0,
                    selection_start: None,
                };
                self.file_path = Some(path);
                self.rel_path = rel_path;
                self.is_editing = edit;
            }
            Err(e) => {
                self.state = EditorState {
                    lines: vec![format!("Error reading macro file: {}", e)],
                    cursor_x: 0,
                    cursor_y: 0,
                    selection_start: None,
                };
                self.file_path = None;
                self.rel_path.clear();
                self.is_editing = false;
            }
        }
        self.scroll_x = 0;
        self.scroll_y = 0;
        self.mode = Mode::Command;
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.reset_keys();
    }

    pub fn save(&self) {
        if let Some(path) = &self.file_path {
            let content = self.state.lines.join("\n");
            let _ = fs::write(path, content);
        }
    }

    fn reset_keys(&mut self) {
        self.last_key_g = false;
        self.last_key_d = false;
        self.last_key_y = false;
    }

    fn push_undo(&mut self) {
        self.undo_stack.push(self.state.clone());
        self.redo_stack.clear();
    }

    pub fn handle_key(&mut self, key: Key) {
        match self.mode {
            Mode::Command => self.handle_command_key(key),
            Mode::Insert => self.handle_insert_key(key),
        }
    }

    fn handle_command_key(&mut self, key: Key) {
        let mut reset_g = true;
        let mut reset_d = true;
        let mut reset_y = true;

        match key {
            Key::Char('i') => {
                self.state.selection_start = None;
                self.mode = Mode::Insert;
            }
            Key::Char('h') | Key::Left => self.move_cursor(-1, 0, false),
            Key::Char('j') | Key::Down => self.move_cursor(0, 1, false),
            Key::Char('k') | Key::Up => self.move_cursor(0, -1, false),
            Key::Char('l') | Key::Right => self.move_cursor(1, 0, false),
            Key::ShiftLeft => self.move_cursor(-1, 0, true),
            Key::ShiftDown => self.move_cursor(0, 1, true),
            Key::ShiftUp => self.move_cursor(0, -1, true),
            Key::ShiftRight => self.move_cursor(1, 0, true),
            Key::Char('1') => {
                self.state.selection_start = None;
                self.state.cursor_x = 0;
            }
            Key::Char('0') => {
                self.state.selection_start = None;
                let len = self.state.lines[self.state.cursor_y].chars().count();
                self.state.cursor_x = len.saturating_sub(1);
            }
            Key::Char('w') => self.jump_word_forward(),
            Key::Char('b') => self.jump_word_backward(),
            Key::Char('g') => {
                self.state.selection_start = None;
                if self.last_key_g {
                    self.state.cursor_y = 0;
                    self.state.cursor_x = 0;
                } else {
                    reset_g = false;
                    self.last_key_g = true;
                }
            }
            Key::Char('G') => {
                self.state.selection_start = None;
                self.state.cursor_y = self.state.lines.len().saturating_sub(1);
                let len = self.state.lines[self.state.cursor_y].chars().count();
                self.state.cursor_x = len.saturating_sub(1);
            }
            Key::Char('d') => {
                if self.state.selection_start.is_some() {
                    self.push_undo();
                    self.delete_selection();
                    reset_d = true;
                } else if self.last_key_d {
                    self.push_undo();
                    self.delete_current_line();
                } else {
                    reset_d = false;
                    self.last_key_d = true;
                }
            }
            Key::Char('y') => {
                if self.state.selection_start.is_some() {
                    self.copy_selection();
                    reset_y = true;
                } else if self.last_key_y {
                    self.copy_current_line();
                } else {
                    reset_y = false;
                    self.last_key_y = true;
                }
            }
            Key::Char('p') => {
                self.push_undo();
                self.paste_from_clipboard();
            }
            Key::Char('u') => {
                if let Some(state) = self.undo_stack.pop() {
                    self.redo_stack.push(self.state.clone());
                    self.state = state;
                }
            }
            Key::Ctrl('r') => {
                if let Some(state) = self.redo_stack.pop() {
                    self.undo_stack.push(self.state.clone());
                    self.state = state;
                }
            }
            Key::Char('s') => self.save(),
            Key::Delete => {
                self.push_undo();
                self.delete_char_after();
            }
            _ => {}
        }

        if reset_g {
            self.last_key_g = false;
        }
        if reset_d {
            self.last_key_d = false;
        }
        if reset_y {
            self.last_key_y = false;
        }

        self.clamp_cursor();
    }

    fn handle_insert_key(&mut self, key: Key) {
        match key {
            Key::Esc => {
                self.mode = Mode::Command;
                if self.state.cursor_x > 0 {
                    self.state.cursor_x -= 1;
                }
                self.state.selection_start = None;
                self.clamp_cursor();
            }
            Key::Char(c) => {
                self.push_undo();
                self.insert_char(c);
            }
            Key::Enter => {
                self.push_undo();
                self.insert_newline();
            }
            Key::Backspace => {
                self.push_undo();
                self.delete_char_before();
            }
            Key::Delete => {
                self.push_undo();
                self.delete_char_after();
            }
            Key::Tab => {
                self.push_undo();
                for _ in 0..4 {
                    self.insert_char(' ');
                }
            }
            Key::Up | Key::ShiftUp => self.move_cursor(0, -1, false),
            Key::Down | Key::ShiftDown => self.move_cursor(0, 1, false),
            Key::Left | Key::ShiftLeft => self.move_cursor(-1, 0, false),
            Key::Right | Key::ShiftRight => self.move_cursor(1, 0, false),
            _ => {}
        }
    }

    fn move_cursor(&mut self, dx: isize, dy: isize, shift: bool) {
        if shift {
            if self.state.selection_start.is_none() {
                self.state.selection_start = Some((self.state.cursor_x, self.state.cursor_y));
            }
        } else {
            self.state.selection_start = None;
        }

        let y = self.state.cursor_y as isize + dy;
        let y = y.clamp(0, self.state.lines.len().saturating_sub(1) as isize) as usize;
        self.state.cursor_y = y;

        let x = self.state.cursor_x as isize + dx;
        let max_x = self.state.lines[y].chars().count();
        let limit = if self.mode == Mode::Insert || shift {
            max_x
        } else {
            max_x.saturating_sub(1)
        };
        let x = x.clamp(0, limit as isize) as usize;
        self.state.cursor_x = x;
    }

    fn clamp_cursor(&mut self) {
        let max_y = self.state.lines.len().saturating_sub(1);
        if self.state.cursor_y > max_y {
            self.state.cursor_y = max_y;
        }
        let max_x = self.state.lines[self.state.cursor_y].chars().count();
        let limit = if self.mode == Mode::Insert || self.state.selection_start.is_some() {
            max_x
        } else {
            max_x.saturating_sub(1)
        };
        if self.state.cursor_x > limit {
            self.state.cursor_x = limit;
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

    fn insert_newline(&mut self) {
        let line = &mut self.state.lines[self.state.cursor_y];
        let byte_idx = line
            .char_indices()
            .nth(self.state.cursor_x)
            .map(|(i, _)| i)
            .unwrap_or(line.len());
        let new_line = line.split_off(byte_idx);
        self.state.lines.insert(self.state.cursor_y + 1, new_line);
        self.state.cursor_y += 1;
        self.state.cursor_x = 0;
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
            self.state.lines[self.state.cursor_y].push_str(&next_line);
        }
        self.clamp_cursor();
    }

    fn delete_current_line(&mut self) {
        let mut line = self.state.lines.remove(self.state.cursor_y);
        line.push('\n');
        if let Some(cb) = &mut self.clipboard {
            let _ = cb.set_text(line);
        }
        if self.state.lines.is_empty() {
            self.state.lines.push(String::new());
        }
        self.clamp_cursor();
    }

    fn copy_current_line(&mut self) {
        let mut line = self.state.lines[self.state.cursor_y].clone();
        line.push('\n');
        if let Some(cb) = &mut self.clipboard {
            let _ = cb.set_text(line);
        }
    }

    fn paste_from_clipboard(&mut self) {
        if let Some(cb) = &mut self.clipboard {
            if let Ok(text) = cb.get_text() {
                for c in text.chars() {
                    if c == '\n' {
                        self.insert_newline();
                    } else if c != '\r' {
                        self.insert_char(c);
                    }
                }
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

    fn delete_selection(&mut self) {
        if let Some((start, end)) = self.get_selection_bounds() {
            let mut text = String::new();
            let mut new_lines = Vec::new();

            for (i, line) in self.state.lines.iter().enumerate() {
                if i < start.1 || i > end.1 {
                    new_lines.push(line.clone());
                } else if start.1 == end.1 {
                    let mut new_line = String::new();
                    for (j, c) in line.chars().enumerate() {
                        if j < start.0 || j >= end.0 {
                            new_line.push(c);
                        } else {
                            text.push(c);
                        }
                    }
                    new_lines.push(new_line);
                } else {
                    if i == start.1 {
                        let mut new_line = String::new();
                        for (j, c) in line.chars().enumerate() {
                            if j < start.0 {
                                new_line.push(c);
                            } else {
                                text.push(c);
                            }
                        }
                        text.push('\n');
                        new_lines.push(new_line);
                    } else if i == end.1 {
                        let mut new_line = String::new();
                        for (j, c) in line.chars().enumerate() {
                            if j >= end.0 {
                                new_line.push(c);
                            } else {
                                text.push(c);
                            }
                        }
                        let last = new_lines.pop().unwrap();
                        new_lines.push(last + &new_line);
                    } else {
                        for c in line.chars() {
                            text.push(c);
                        }
                        text.push('\n');
                    }
                }
            }

            self.state.lines = new_lines;
            self.state.cursor_x = start.0;
            self.state.cursor_y = start.1;
            self.state.selection_start = None;

            if let Some(cb) = &mut self.clipboard {
                let _ = cb.set_text(text);
            }
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
                    for (j, c) in line.chars().enumerate() {
                        if j >= start.0 && j < end.0 {
                            text.push(c);
                        }
                    }
                } else {
                    if i == start.1 {
                        for (j, c) in line.chars().enumerate() {
                            if j >= start.0 {
                                text.push(c);
                            }
                        }
                        text.push('\n');
                    } else if i == end.1 {
                        for (j, c) in line.chars().enumerate() {
                            if j < end.0 {
                                text.push(c);
                            }
                        }
                    } else {
                        text.push_str(line);
                        text.push('\n');
                    }
                }
            }
            if let Some(cb) = &mut self.clipboard {
                let _ = cb.set_text(text);
            }
            self.state.selection_start = None;
        }
    }

    fn jump_word_forward(&mut self) {
        self.state.selection_start = None;
        let line = &self.state.lines[self.state.cursor_y];
        if self.state.cursor_x >= line.chars().count().saturating_sub(1) {
            if self.state.cursor_y < self.state.lines.len() - 1 {
                self.state.cursor_y += 1;
                self.state.cursor_x = 0;
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

    fn jump_word_backward(&mut self) {
        self.state.selection_start = None;
        if self.state.cursor_x == 0 {
            if self.state.cursor_y > 0 {
                self.state.cursor_y -= 1;
                self.state.cursor_x = self.state.lines[self.state.cursor_y]
                    .chars()
                    .count()
                    .saturating_sub(1);
            }
            return;
        }

        let line = &self.state.lines[self.state.cursor_y];
        let mut rev_chars = line.chars().take(self.state.cursor_x).collect::<Vec<_>>();
        rev_chars.reverse();

        let mut skipped = 0;
        for &c in &rev_chars {
            if !c.is_whitespace() {
                break;
            }
            skipped += 1;
        }
        for &c in rev_chars.iter().skip(skipped) {
            if c.is_whitespace() {
                break;
            }
            skipped += 1;
        }
        self.state.cursor_x = self.state.cursor_x.saturating_sub(skipped);
    }

    pub fn render(&mut self, width: u16, height: u16, is_active: bool) -> Box {
        let mut b = Box::new(
            width,
            height,
            1,
            if is_active {
                Border::Heavy
            } else {
                Border::Light
            },
            Style {
                fg: if is_active {
                    Color::White
                } else {
                    Color::Magenta
                },
                bg: Color::None,
                md: Modifier::None,
            },
        );

        if self.is_editing {
            let mode_str = match self.mode {
                Mode::Command => "[COMMAND]",
                Mode::Insert => "[INSERT]",
            };
            let title = if self.rel_path.is_empty() {
                format!(" {} ", mode_str)
            } else {
                format!(" {} {} ", mode_str, self.rel_path)
            };
            b.insert_text(
                &title,
                1,
                -1,
                false,
                Style {
                    fg: Color::Yellow,
                    bg: Color::None,
                    md: Modifier::Bold,
                },
            );
        }

        let inner_w = width.saturating_sub(2) as usize;
        let inner_h = height.saturating_sub(2) as usize;

        if self.state.cursor_y < self.scroll_y {
            self.scroll_y = self.state.cursor_y;
        } else if self.state.cursor_y >= self.scroll_y + inner_h && inner_h > 0 {
            self.scroll_y = self.state.cursor_y - inner_h + 1;
        }

        if self.state.cursor_x < self.scroll_x {
            self.scroll_x = self.state.cursor_x;
        } else if self.state.cursor_x >= self.scroll_x + inner_w && inner_w > 0 {
            self.scroll_x = self.state.cursor_x - inner_w + 1;
        }

        for (i, line) in self
            .state
            .lines
            .iter()
            .enumerate()
            .skip(self.scroll_y)
            .take(inner_h)
        {
            let display_y = (i - self.scroll_y) as i16;
            let chars: Vec<char> = line.chars().collect();

            for (j, &c) in chars.iter().enumerate().skip(self.scroll_x).take(inner_w) {
                let display_x = (j - self.scroll_x) as i16;

                let is_selected = if let Some((start, end)) = self.get_selection_bounds() {
                    let p_start = (start.0, start.1);
                    let p_end = (end.0, end.1);
                    let is_after_start = i > p_start.1 || (i == p_start.1 && j >= p_start.0);
                    let is_before_end = i < p_end.1 || (i == p_end.1 && j < p_end.0);
                    is_after_start && is_before_end
                } else {
                    false
                };

                let mut style = Style {
                    fg: Color::White,
                    bg: if is_selected {
                        Color::DarkGray
                    } else {
                        Color::None
                    },
                    md: if is_active {
                        Modifier::None
                    } else {
                        Modifier::Dim
                    },
                };

                if self.is_editing && i == self.state.cursor_y && j == self.state.cursor_x {
                    style.bg = Color::White;
                    style.fg = Color::Black;
                }

                b.put_cell(
                    crate::Cell { c, s: style },
                    display_x as u16 + 1,
                    display_y as u16 + 1,
                );
            }

            if self.is_editing && i == self.state.cursor_y && self.state.cursor_x == chars.len() {
                if self.state.cursor_x >= self.scroll_x {
                    let display_x = self.state.cursor_x - self.scroll_x;
                    if display_x < inner_w {
                        b.put_cell(
                            crate::Cell {
                                c: ' ',
                                s: Style {
                                    fg: Color::Black,
                                    bg: Color::White,
                                    md: if is_active {
                                        Modifier::None
                                    } else {
                                        Modifier::Dim
                                    },
                                },
                            },
                            display_x as u16 + 1,
                            display_y as u16 + 1,
                        );
                    }
                }
            }
        }
        b
    }
}
