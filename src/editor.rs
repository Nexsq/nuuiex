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
}

pub struct Editor {
    pub is_editing: bool,
    pub mode: Mode,
    pub state: EditorState,
    pub scroll_x: usize,
    pub scroll_y: usize,
    pub file_path: Option<PathBuf>,
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
            },
            scroll_x: 0,
            scroll_y: 0,
            file_path: None,
            clipboard: Clipboard::new().ok(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            last_key_g: false,
            last_key_d: false,
            last_key_y: false,
        }
    }

    pub fn load_file(&mut self, path: PathBuf, edit: bool) {
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
                };
                self.file_path = Some(path);
                self.is_editing = edit;
            }
            Err(e) => {
                self.state = EditorState {
                    lines: vec![format!("Error reading macro file: {}", e)],
                    cursor_x: 0,
                    cursor_y: 0,
                };
                self.file_path = None;
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
            Key::Char('i') => self.mode = Mode::Insert,
            Key::Char('h') | Key::Left => self.move_cursor(-1, 0),
            Key::Char('j') | Key::Down => self.move_cursor(0, 1),
            Key::Char('k') | Key::Up => self.move_cursor(0, -1),
            Key::Char('l') | Key::Right => self.move_cursor(1, 0),
            Key::Char('0') => self.state.cursor_x = 0,
            Key::Char('$') => {
                let len = self.state.lines[self.state.cursor_y].chars().count();
                self.state.cursor_x = len.saturating_sub(1);
            }
            Key::Char('w') => self.jump_word_forward(),
            Key::Char('b') => self.jump_word_backward(),
            Key::Char('g') => {
                if self.last_key_g {
                    self.state.cursor_y = 0;
                    self.state.cursor_x = 0;
                } else {
                    reset_g = false;
                    self.last_key_g = true;
                }
            }
            Key::Char('G') => {
                self.state.cursor_y = self.state.lines.len().saturating_sub(1);
                self.state.cursor_x = 0;
            }
            Key::Char('d') => {
                if self.last_key_d {
                    self.push_undo();
                    self.delete_current_line();
                } else {
                    reset_d = false;
                    self.last_key_d = true;
                }
            }
            Key::Char('y') => {
                if self.last_key_y {
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
            Key::Tab => {
                self.push_undo();
                for _ in 0..4 {
                    self.insert_char(' ');
                }
            }
            Key::Up => self.move_cursor(0, -1),
            Key::Down => self.move_cursor(0, 1),
            Key::Left => self.move_cursor(-1, 0),
            Key::Right => self.move_cursor(1, 0),
            _ => {}
        }
    }

    fn move_cursor(&mut self, dx: isize, dy: isize) {
        let y = self.state.cursor_y as isize + dy;
        let y = y.clamp(0, self.state.lines.len().saturating_sub(1) as isize) as usize;
        self.state.cursor_y = y;

        let x = self.state.cursor_x as isize + dx;
        let max_x = self.state.lines[y].chars().count();
        let limit = if self.mode == Mode::Insert {
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
        let limit = if self.mode == Mode::Insert {
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

    fn jump_word_forward(&mut self) {
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
            let title = format!(" {} ", mode_str);
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
                let mut style = Style {
                    fg: Color::White,
                    bg: Color::None,
                    md: Modifier::None,
                };

                if self.is_editing && i == self.state.cursor_y && j == self.state.cursor_x {
                    style.bg = if is_active {
                        Color::White
                    } else {
                        Color::DarkGray
                    };
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
                                    bg: if is_active {
                                        Color::White
                                    } else {
                                        Color::DarkGray
                                    },
                                    md: Modifier::None,
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
