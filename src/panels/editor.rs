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
    pub visual_mode: bool,
    pub state: EditorState,
    pub scroll_x: usize,
    pub scroll_y: usize,
    pub file_path: Option<PathBuf>,
    pub rel_path: String,
    pub clipboard: Option<Clipboard>,

    pub undo_stack: Vec<EditorState>,
    pub redo_stack: Vec<EditorState>,

    pub last_key_a: bool,
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
            rel_path: String::new(),
            clipboard: Clipboard::new().ok(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            last_key_a: false,
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
                    content
                        .split('\n')
                        .map(|s| s.strip_suffix('\r').unwrap_or(s).to_string())
                        .collect()
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
        self.visual_mode = false;
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
        self.last_key_a = false;
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
        let mut reset_a = true;
        let mut reset_g = true;
        let mut reset_d = true;
        let mut reset_y = true;

        match key {
            Key::Char('i') | Key::Esc => {
                self.visual_mode = false;
                self.state.selection_start = None;
                self.mode = Mode::Insert;
            }
            Key::Char('v') => {
                if self.visual_mode {
                    self.visual_mode = false;
                    self.state.selection_start = None;
                } else {
                    self.visual_mode = true;
                    self.state.selection_start = Some((self.state.cursor_x, self.state.cursor_y));
                }
            }
            Key::Left | Key::Char('h') => self.move_cursor(-1, 0, self.visual_mode),
            Key::Right | Key::Char('l') => self.move_cursor(1, 0, self.visual_mode),
            Key::Up | Key::Char('k') => self.move_cursor(0, -1, self.visual_mode),
            Key::Down | Key::Char('j') => self.move_cursor(0, 1, self.visual_mode),
            Key::CtrlLeft | Key::Ctrl('h') | Key::Char('b') | Key::CtrlBackspace => {
                self.jump_word_backward(self.visual_mode)
            }
            Key::CtrlRight | Key::Ctrl('l') | Key::Char('w') => {
                self.jump_word_forward(self.visual_mode)
            }
            Key::CtrlUp | Key::Ctrl('k') => self.jump_block_backward(self.visual_mode),
            Key::CtrlDown | Key::Ctrl('j') => self.jump_block_forward(self.visual_mode),

            Key::Char('1') => {
                if !self.visual_mode {
                    self.state.selection_start = None;
                }
                self.state.cursor_x = 0;
            }
            Key::Char('0') => {
                if !self.visual_mode {
                    self.state.selection_start = None;
                }
                self.state.cursor_x = self.state.lines[self.state.cursor_y].chars().count();
            }
            Key::Char('a') => {
                if self.last_key_a {
                    self.visual_mode = true;
                    self.state.selection_start = Some((0, 0));
                    self.state.cursor_y = self.state.lines.len().saturating_sub(1);
                    self.state.cursor_x = self.state.lines[self.state.cursor_y].chars().count();
                } else {
                    reset_a = false;
                    self.last_key_a = true;
                }
            }
            Key::Delete => {
                self.push_undo();
                if self.state.selection_start.is_some() {
                    self.delete_selection();
                } else {
                    self.delete_char_after();
                }
            }
            Key::Char('g') => {
                if !self.visual_mode {
                    self.state.selection_start = None;
                }
                if self.last_key_g {
                    self.state.cursor_y = 0;
                    self.state.cursor_x = 0;
                } else {
                    reset_g = false;
                    self.last_key_g = true;
                }
            }
            Key::Shift('g') => {
                if !self.visual_mode {
                    self.state.selection_start = None;
                }
                self.state.cursor_y = self.state.lines.len().saturating_sub(1);
                self.state.cursor_x = self.state.lines[self.state.cursor_y].chars().count();
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
            _ => {}
        }

        if reset_a {
            self.last_key_a = false;
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
                self.visual_mode = false;
                self.state.selection_start = None;
                self.clamp_cursor();
            }
            Key::Char(c) => {
                if c.is_control() {
                    return;
                }
                self.push_undo();
                if self.state.selection_start.is_some() {
                    self.delete_selection();
                }
                let caps = crate::Terminal::is_caps_lock_on();
                let mut final_c = c;
                if caps && c.is_ascii_alphabetic() {
                    final_c = c.to_ascii_uppercase();
                }
                self.insert_char(final_c);
            }
            Key::Shift(c) => {
                if c.is_control() {
                    return;
                }
                self.push_undo();
                if self.state.selection_start.is_some() {
                    self.delete_selection();
                }
                let caps = crate::Terminal::is_caps_lock_on();
                let mut final_c = c.to_ascii_uppercase();
                if caps && final_c.is_ascii_alphabetic() {
                    final_c = final_c.to_ascii_lowercase();
                }
                self.insert_char(final_c);
            }
            Key::Enter => {
                self.push_undo();
                if self.state.selection_start.is_some() {
                    self.delete_selection();
                }
                self.insert_newline();
            }
            Key::CtrlLeft => self.jump_word_backward(false),
            Key::CtrlRight => self.jump_word_forward(false),
            Key::CtrlUp => self.jump_block_backward(false),
            Key::CtrlDown => self.jump_block_forward(false),
            Key::Backspace => {
                self.push_undo();
                if self.state.selection_start.is_some() {
                    self.delete_selection();
                } else {
                    self.delete_char_before();
                }
            }
            Key::Delete => {
                self.push_undo();
                if self.state.selection_start.is_some() {
                    self.delete_selection();
                } else {
                    self.delete_char_after();
                }
            }
            Key::CtrlBackspace | Key::Ctrl('w') | Key::Ctrl('h') => {
                self.push_undo();
                if self.state.selection_start.is_some() {
                    self.delete_selection();
                } else {
                    self.delete_word_before();
                }
            }
            Key::CtrlDelete => {
                self.push_undo();
                if self.state.selection_start.is_some() {
                    self.delete_selection();
                } else {
                    self.delete_word_after();
                }
            }
            Key::Tab => {
                self.push_undo();
                if self.state.selection_start.is_some() {
                    self.delete_selection();
                }
                for _ in 0..4 {
                    self.insert_char(' ');
                }
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

        let y = self.state.cursor_y as isize + dy;
        let y = y.clamp(0, self.state.lines.len().saturating_sub(1) as isize) as usize;
        self.state.cursor_y = y;

        let x = self.state.cursor_x as isize + dx;
        let max_x = self.state.lines[y].chars().count();
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
                    self.state.lines.splice(insert_idx..insert_idx, new_lines);
                    self.state.cursor_y += lines_to_insert.len() - 1;
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

            if start.1 == end.1 {
                let line = &self.state.lines[start.1];
                let mut new_line = String::with_capacity(line.len());
                for (j, c) in line.chars().enumerate() {
                    if j < start.0 || j >= end.0 {
                        new_line.push(c);
                    } else {
                        text.push(c);
                    }
                }
                self.state.lines[start.1] = new_line;
            } else {
                let start_line = &self.state.lines[start.1];
                let mut new_start_line = String::new();
                for (j, c) in start_line.chars().enumerate() {
                    if j < start.0 {
                        new_start_line.push(c);
                    } else {
                        text.push(c);
                    }
                }
                text.push('\n');

                for i in (start.1 + 1)..end.1 {
                    text.push_str(&self.state.lines[i]);
                    text.push('\n');
                }

                let end_line = &self.state.lines[end.1];
                let mut new_end_line = String::new();
                for (j, c) in end_line.chars().enumerate() {
                    if j >= end.0 {
                        new_end_line.push(c);
                    } else {
                        text.push(c);
                    }
                }

                new_start_line.push_str(&new_end_line);

                if end.1 > start.1 {
                    self.state.lines.drain((start.1 + 1)..=end.1);
                }

                self.state.lines[start.1] = new_start_line;
            }

            self.state.cursor_x = start.0;
            self.state.cursor_y = start.1;
            self.state.selection_start = None;
            self.visual_mode = false;

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
            self.visual_mode = false;
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

    fn jump_word_backward(&mut self, select: bool) {
        if select {
            if self.state.selection_start.is_none() {
                self.state.selection_start = Some((self.state.cursor_x, self.state.cursor_y));
            }
        } else {
            self.state.selection_start = None;
        }

        if self.state.cursor_x == 0 {
            if self.state.cursor_y > 0 {
                self.state.cursor_y -= 1;
                self.state.cursor_x = self.state.lines[self.state.cursor_y].chars().count();
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

        let mut y = self.state.cursor_y;
        let lines = &self.state.lines;
        let max_y = lines.len().saturating_sub(1);

        if y < max_y {
            let start_is_empty = lines[y].trim().is_empty();
            y += 1;
            while y < max_y && lines[y].trim().is_empty() == start_is_empty {
                y += 1;
            }
        }

        self.state.cursor_y = y;
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

        let mut y = self.state.cursor_y;
        let lines = &self.state.lines;

        if y > 0 {
            let start_is_empty = lines[y].trim().is_empty();
            y -= 1;
            while y > 0 && lines[y].trim().is_empty() == start_is_empty {
                y -= 1;
            }
        }

        self.state.cursor_y = y;
        self.clamp_cursor();
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
                Mode::Command => {
                    if self.visual_mode {
                        "[VIS]"
                    } else {
                        "[CMD]"
                    }
                }
                Mode::Insert => "[INS]",
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
            max_num_width + 3
        } else {
            0
        };

        let text_inner_w = inner_w.saturating_sub(prefix_width);

        if self.state.cursor_y < self.scroll_y {
            self.scroll_y = self.state.cursor_y;
        } else if self.state.cursor_y >= self.scroll_y + inner_h && inner_h > 0 {
            self.scroll_y = self.state.cursor_y - inner_h + 1;
        }

        if self.state.cursor_x < self.scroll_x {
            self.scroll_x = self.state.cursor_x;
        } else if self.state.cursor_x >= self.scroll_x + text_inner_w && text_inner_w > 0 {
            self.scroll_x = self.state.cursor_x - text_inner_w + 1;
        }

        let selection = self.get_selection_bounds();

        for (i, line) in self
            .state
            .lines
            .iter()
            .enumerate()
            .skip(self.scroll_y)
            .take(inner_h)
        {
            let display_y = (i - self.scroll_y) as i16;

            if show_line_numbers {
                let prefix_str = format!("{:<w$} | ", i + 1, w = max_num_width);
                let prefix_style = Style {
                    fg: Color::DarkGray,
                    bg: Color::None,
                    md: Modifier::Dim,
                };

                for (idx, c) in prefix_str.chars().enumerate() {
                    if idx < inner_w {
                        b.put_cell(
                            crate::Cell { c, s: prefix_style },
                            idx as u16 + 1,
                            display_y as u16 + 1,
                        );
                    }
                }
            }

            for (j, c) in line
                .chars()
                .enumerate()
                .skip(self.scroll_x)
                .take(text_inner_w)
            {
                let display_x = (j - self.scroll_x + prefix_width) as i16;

                let is_selected = if let Some((start, end)) = selection {
                    let is_after_start = i > start.1 || (i == start.1 && j >= start.0);
                    let is_before_end = i < end.1 || (i == end.1 && j < end.0);
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

                let display_c = if c.is_control() { ' ' } else { c };

                if (display_x as usize) < inner_w {
                    b.put_cell(
                        crate::Cell {
                            c: display_c,
                            s: style,
                        },
                        display_x as u16 + 1,
                        display_y as u16 + 1,
                    );
                }
            }

            if self.is_editing && i == self.state.cursor_y {
                let line_len = line.chars().count();
                if self.state.cursor_x == line_len {
                    if self.state.cursor_x >= self.scroll_x {
                        let display_x = self.state.cursor_x - self.scroll_x + prefix_width;
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
        }
        b
    }
}
