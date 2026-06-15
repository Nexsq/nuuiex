use super::style::{Color, Modifier};
use std::io::{self, Write};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cell {
    pub s: char,
    pub fg: Color,
    pub bg: Color,
    pub md: Modifier,
}

#[derive(Debug)]
pub struct Box {
    pub width: u16,
    pub height: u16,
    pub padding: u16,
    // border and sth here
    pub grid: Vec<Cell>,
}

#[derive(Debug)]
pub struct Canvas {
    pub width: u16,
    pub height: u16,
    pub old: Vec<Cell>,
    pub new: Vec<Cell>,
    pub buffer: String,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            s: ' ',
            fg: Color::None,
            bg: Color::None,
            md: Modifier::None,
        }
    }
}

impl Box {
    pub fn new(width: u16, height: u16, padding: u16) -> Self {
        let size = (width as usize) * (height as usize);
        Self {
            width,
            height,
            padding,
            // border and sth here
            grid: vec![Cell::default(); size],
        }
    }

    pub fn clean(&mut self) {
        self.grid.fill(Cell::default());
    }

    pub fn put_cell(&mut self, cell: Cell, x: u16, y: u16) {
        if x < self.width && y < self.height {
            let idx = (y as usize) * (self.width as usize) + (x as usize);
            self.grid[idx] = cell;
        }
    }

    pub fn insert_box(&mut self, buffer: &Box, x: i16, y: i16) {
        self.draw_box(buffer, x, y, false);
    }

    pub fn insert_box_overlap(&mut self, buffer: &Box, x: i16, y: i16) {
        self.draw_box(buffer, x, y, true);
    }

    pub fn insert_text(
        &mut self,
        text: &str,
        offset_x: i16,
        offset_y: i16,
        word_wrap: bool,
        style: Cell,
    ) {
        let pad = self.padding as i16;

        let eff_left = (pad + offset_x).max(0) as u16;
        let eff_top = (pad + offset_y).max(0) as u16;

        let max_x = self.width.saturating_sub(self.padding);
        let max_y = self.height.saturating_sub(self.padding);

        if max_x <= eff_left || max_y <= eff_top {
            return;
        }

        let mut cx = eff_left;
        let mut cy = eff_top;

        let mut chars = text.chars().peekable();

        while let Some(&c) = chars.peek() {
            if cy >= max_y {
                break;
            }

            if c == '\n' {
                cy += 1;
                cx = eff_left;
                chars.next();
                continue;
            }

            if !word_wrap {
                let mut cell = style;
                cell.s = c;
                self.put_cell(cell, cx, cy);

                cx += 1;
                if cx >= max_x {
                    cx = eff_left;
                    cy += 1;
                }
                chars.next();
            } else {
                if c.is_whitespace() {
                    if cx > eff_left && cx < max_x {
                        let mut cell = style;
                        cell.s = c;
                        self.put_cell(cell, cx, cy);
                        cx += 1;
                    }
                    chars.next();
                    continue;
                }

                let mut lookahead = chars.clone();
                let mut word_len = 0;
                while let Some(&lc) = lookahead.peek() {
                    if lc.is_whitespace() || lc == '\n' {
                        break;
                    }
                    word_len += 1;
                    lookahead.next();
                }

                if cx + word_len > max_x && cx > eff_left {
                    cx = eff_left;
                    cy += 1;
                    if cy >= max_y {
                        break;
                    }
                }

                for _ in 0..word_len {
                    if cy >= max_y {
                        break;
                    }
                    let mut cell = style;
                    cell.s = chars.next().unwrap();
                    self.put_cell(cell, cx, cy);
                    cx += 1;
                    if cx >= max_x {
                        cx = eff_left;
                        cy += 1;
                    }
                }
            }
        }
    }
}

impl Canvas {
    pub fn new(width: u16, height: u16) -> Self {
        let size = (width as usize) * (height as usize);

        let imp_cell = Cell {
            s: '\0',
            fg: Color::None,
            bg: Color::None,
            md: Modifier::None,
        };

        Self {
            width,
            height,
            old: vec![imp_cell; size],
            new: vec![Cell::default(); size],
            buffer: String::with_capacity(size * 4),
        }
    }

    pub fn clean(&mut self) {
        self.new.fill(Cell::default());
    }

    pub fn render(&mut self) {
        use std::fmt::Write;

        self.buffer.clear();

        let mut cur_fg = Color::None;
        let mut cur_bg = Color::None;
        let mut cur_md = Modifier::None;

        let mut cursor_x: u16 = u16::MAX;
        let mut cursor_y: u16 = u16::MAX;

        let width = self.width;

        let mut x: u16 = 0;
        let mut y: u16 = 0;

        for (old_cell, new_cell) in self.old.iter_mut().zip(self.new.iter()) {
            if old_cell != new_cell {
                if cursor_x != x || cursor_y != y {
                    write!(&mut self.buffer, "\x1b[{};{}H", y + 1, x + 1).unwrap();
                }

                if new_cell.md != cur_md {
                    if cur_md != Modifier::None {
                        write!(&mut self.buffer, "\x1b[0m").unwrap();
                        cur_fg = Color::None;
                        cur_bg = Color::None;
                    }

                    if new_cell.md != Modifier::None {
                        write!(&mut self.buffer, "{}", new_cell.md.to_ansi()).unwrap();
                    }
                    cur_md = new_cell.md;
                }

                if new_cell.fg != cur_fg {
                    if new_cell.fg == Color::None {
                        write!(&mut self.buffer, "\x1b[39m").unwrap();
                    } else {
                        write!(&mut self.buffer, "{}", new_cell.fg.fg_ansi()).unwrap();
                    }
                    cur_fg = new_cell.fg;
                }

                if new_cell.bg != cur_bg {
                    if new_cell.bg == Color::None {
                        write!(&mut self.buffer, "\x1b[49m").unwrap();
                    } else {
                        write!(&mut self.buffer, "{}", new_cell.bg.bg_ansi()).unwrap();
                    }
                    cur_bg = new_cell.bg;
                }

                write!(&mut self.buffer, "{}", new_cell.s).unwrap();

                cursor_x = x + 1;
                cursor_y = y;

                if cursor_x >= width {
                    cursor_x = u16::MAX;
                }

                *old_cell = *new_cell;
            }

            x += 1;
            if x >= width {
                x = 0;
                y += 1;
            }
        }

        write!(&mut self.buffer, "\x1b[0m").unwrap();

        let mut stdout = io::stdout().lock();
        stdout.write_all(self.buffer.as_bytes()).unwrap();
        stdout.flush().unwrap();
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        let size = (width as usize) * (height as usize);
        self.width = width;
        self.height = height;

        self.old.clear();
        self.new.clear();

        self.old.resize(size, Cell::default());
        self.new.resize(size, Cell::default());

        print!("\x1b[2J\x1b[H");
        io::stdout().flush().unwrap();
    }

    pub fn put_box(&mut self, buffer: &Box, x: i16, y: i16) {
        self.draw_box(buffer, x, y, false);
    }

    pub fn put_box_overlap(&mut self, buffer: &Box, x: i16, y: i16) {
        self.draw_box(buffer, x, y, true);
    }
}

pub trait DrawTarget {
    fn width(&self) -> u16;
    fn height(&self) -> u16;
    fn buffer_mut(&mut self) -> &mut [Cell];

    fn draw_box(&mut self, buffer: &Box, x: i16, y: i16, ignore_spaces: bool) {
        let cw = self.width() as i16;
        let ch = self.height() as i16;
        let bw = buffer.width as i16;
        let bh = buffer.height as i16;

        if x >= cw || y >= ch || x + bw <= 0 || y + bh <= 0 {
            return;
        }

        let dest_x = x.max(0) as usize;
        let dest_y = y.max(0) as usize;
        let src_x = if x < 0 { -x as usize } else { 0 };
        let src_y = if y < 0 { -y as usize } else { 0 };

        let render_w = ((x + bw).min(cw) - x.max(0)) as usize;
        let render_h = ((y + bh).min(ch) - y.max(0)) as usize;

        let buffer_rows = buffer
            .grid
            .chunks_exact(buffer.width as usize)
            .skip(src_y)
            .take(render_h);

        let target_width = self.width() as usize;
        let self_rows = self
            .buffer_mut()
            .chunks_exact_mut(target_width)
            .skip(dest_y)
            .take(render_h);

        for (b_row, s_row) in buffer_rows.zip(self_rows) {
            let src_slice = &b_row[src_x..src_x + render_w];
            let dest_slice = &mut s_row[dest_x..dest_x + render_w];

            if ignore_spaces {
                for (src_cell, dest_cell) in src_slice.iter().zip(dest_slice.iter_mut()) {
                    if *src_cell != Cell::default() {
                        *dest_cell = *src_cell;
                    }
                }
            } else {
                dest_slice.copy_from_slice(src_slice);
            }
        }
    }
}

impl DrawTarget for Box {
    fn width(&self) -> u16 {
        self.width
    }
    fn height(&self) -> u16 {
        self.height
    }
    fn buffer_mut(&mut self) -> &mut [Cell] {
        &mut self.grid
    }
}

impl DrawTarget for Canvas {
    fn width(&self) -> u16 {
        self.width
    }
    fn height(&self) -> u16 {
        self.height
    }
    fn buffer_mut(&mut self) -> &mut [Cell] {
        &mut self.new
    }
}
