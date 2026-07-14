use super::style::{Border, Color, Gradient, Modifier, Style};
use std::io::{self, Write};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cell {
    pub c: char,
    pub s: Style,
}

#[derive(Debug)]
pub struct Box {
    pub width: u16,
    pub height: u16,
    pub padding: u16,
    pub border: Border,
    pub grid: Vec<Cell>,
}

#[derive(Debug)]
pub struct Canvas {
    pub width: u16,
    pub height: u16,
    pub old: Vec<Cell>,
    pub new: Vec<Cell>,
    pub buffer: Vec<u8>,
    pub needs_clear: bool,
}

impl Cell {
    pub fn new(c: char, s: Style) -> Self {
        Self { c, s }
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            c: ' ',
            s: Style::default(),
        }
    }
}

impl Box {
    pub fn new(
        width: u16,
        height: u16,
        padding: u16,
        border: Border,
        fg: Gradient,
        bg: Gradient,
        md: Modifier,
    ) -> Self {
        let w = width as usize;
        let h = height as usize;
        let size = w * h;
        let mut grid = vec![Cell::default(); size];

        if w > 0 && h > 0 {
            for x in 0..w {
                grid[x] = Cell {
                    c: ' ',
                    s: Style {
                        fg: fg.color_at(x, w),
                        bg: bg.color_at(x, w),
                        md,
                    },
                };
            }

            for y in 1..h {
                let (first, rest) = grid.split_at_mut(y * w);
                rest[..w].copy_from_slice(&first[..w]);
            }
        }

        if let Some(chars) = border.chars() {
            if width >= 2 && height >= 2 {
                for x in 1..(w - 1) {
                    grid[x].c = chars.h;
                    grid[(h - 1) * w + x].c = chars.h;
                }

                for y in 1..(h - 1) {
                    grid[y * w].c = chars.v;
                    grid[y * w + (w - 1)].c = chars.v;
                }

                grid[0].c = chars.tl;
                grid[w - 1].c = chars.tr;
                grid[(h - 1) * w].c = chars.bl;
                grid[(h - 1) * w + (w - 1)].c = chars.br;
            }
        }

        Self {
            width,
            height,
            padding,
            border,
            grid,
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

    pub fn insert_text(
        &mut self,
        text: &str,
        offset_x: i16,
        offset_y: i16,
        word_wrap: bool,
        fg: Gradient,
        bg: Gradient,
        md: Modifier,
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

        let text_len = text.chars().count().max(1);
        let mut text_idx = 0;

        while let Some(&c) = chars.peek() {
            if cy >= max_y {
                break;
            }

            let current_fg = fg.color_at(text_idx, text_len);
            let current_bg = bg.color_at(text_idx, text_len);
            let style = Style {
                fg: current_fg,
                bg: current_bg,
                md,
            };

            if c == '\n' {
                cy += 1;
                cx = eff_left;
                chars.next();
                text_idx += 1;
                continue;
            }

            if c == '\t' {
                let tab_spaces = 4 - ((cx - eff_left) % 4);
                for _ in 0..tab_spaces {
                    if cx < max_x {
                        let cell = Cell { c: ' ', s: style };
                        self.put_cell(cell, cx, cy);
                        cx += 1;
                    }
                }
                if cx >= max_x {
                    cx = eff_left;
                    cy += 1;
                }
                chars.next();
                text_idx += 1;
                continue;
            }

            if c.is_control() {
                chars.next();
                text_idx += 1;
                continue;
            }

            if !word_wrap {
                let cell = Cell { c, s: style };
                self.put_cell(cell, cx, cy);

                cx += 1;
                if cx >= max_x {
                    cx = eff_left;
                    cy += 1;
                }
                chars.next();
                text_idx += 1;
            } else {
                if c.is_whitespace() {
                    if cx > eff_left && cx < max_x {
                        let cell = Cell { c, s: style };
                        self.put_cell(cell, cx, cy);
                        cx += 1;
                    }
                    chars.next();
                    text_idx += 1;
                    continue;
                }

                let mut lookahead = chars.clone();
                let mut word_len = 0;
                while let Some(&lc) = lookahead.peek() {
                    if lc.is_whitespace() || lc.is_control() {
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
                    let cfg = fg.color_at(text_idx, text_len);
                    let cbg = bg.color_at(text_idx, text_len);
                    let cell = Cell {
                        c: chars.next().unwrap(),
                        s: Style {
                            fg: cfg,
                            bg: cbg,
                            md,
                        },
                    };
                    self.put_cell(cell, cx, cy);
                    cx += 1;
                    if cx >= max_x {
                        cx = eff_left;
                        cy += 1;
                    }
                    text_idx += 1;
                }
            }
        }
    }

    pub fn set_border_style(&mut self, border: Border) {
        self.border = border;

        if self.width < 2 || self.height < 2 {
            return;
        }

        if let Some(chars) = border.chars() {
            let w = self.width as usize;
            let h = self.height as usize;

            self.grid[0].c = chars.tl;
            self.grid[w - 1].c = chars.tr;
            self.grid[(h - 1) * w].c = chars.bl;
            self.grid[(h - 1) * w + (w - 1)].c = chars.br;

            for x in 1..(w - 1) {
                self.grid[x].c = chars.h;
                self.grid[(h - 1) * w + x].c = chars.h;
            }

            for y in 1..(h - 1) {
                self.grid[y * w].c = chars.v;
                self.grid[y * w + (w - 1)].c = chars.v;
            }
        }
    }

    pub fn set_style(&mut self, style: Style) {
        for cell in self.grid.iter_mut() {
            cell.s = style;
        }
    }
}

fn push_num_u16(buf: &mut Vec<u8>, mut n: u16) {
    if n >= 10000 {
        buf.push(b'0' + (n / 10000) as u8);
        n %= 10000;
        buf.push(b'0' + (n / 1000) as u8);
        n %= 1000;
        buf.push(b'0' + (n / 100) as u8);
        n %= 100;
        buf.push(b'0' + (n / 10) as u8);
        buf.push(b'0' + (n % 10) as u8);
    } else if n >= 1000 {
        buf.push(b'0' + (n / 1000) as u8);
        n %= 1000;
        buf.push(b'0' + (n / 100) as u8);
        n %= 100;
        buf.push(b'0' + (n / 10) as u8);
        buf.push(b'0' + (n % 10) as u8);
    } else if n >= 100 {
        buf.push(b'0' + (n / 100) as u8);
        n %= 100;
        buf.push(b'0' + (n / 10) as u8);
        buf.push(b'0' + (n % 10) as u8);
    } else if n >= 10 {
        buf.push(b'0' + (n / 10) as u8);
        buf.push(b'0' + (n % 10) as u8);
    } else {
        buf.push(b'0' + n as u8);
    }
}

impl Canvas {
    pub fn new(width: u16, height: u16) -> Self {
        let size = (width as usize) * (height as usize);
        Self {
            width,
            height,
            old: vec![
                Cell {
                    c: '\0',
                    s: Style::default()
                };
                size
            ],
            new: vec![Cell::default(); size],
            buffer: Vec::with_capacity(size * 16),
            needs_clear: false,
        }
    }

    pub fn clean(&mut self) {
        self.new.fill(Cell::default());
    }

    pub fn render(&mut self) {
        self.buffer.clear();

        if self.needs_clear {
            self.buffer.extend_from_slice(b"\x1b[2J\x1b[H");
            self.needs_clear = false;
        }

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
                    self.buffer.extend_from_slice(b"\x1b[");
                    push_num_u16(&mut self.buffer, y + 1);
                    self.buffer.push(b';');
                    push_num_u16(&mut self.buffer, x + 1);
                    self.buffer.push(b'H');
                }

                if new_cell.s.md != cur_md {
                    if cur_md != Modifier::None {
                        self.buffer.extend_from_slice(b"\x1b[0m");
                        cur_fg = Color::None;
                        cur_bg = Color::None;
                    }
                    if new_cell.s.md != Modifier::None {
                        self.buffer
                            .extend_from_slice(new_cell.s.md.to_ansi().as_bytes());
                    }
                    cur_md = new_cell.s.md;
                }

                if new_cell.s.fg != cur_fg {
                    new_cell.s.fg.fg_ansi(&mut self.buffer);
                    cur_fg = new_cell.s.fg;
                }

                if new_cell.s.bg != cur_bg {
                    new_cell.s.bg.bg_ansi(&mut self.buffer);
                    cur_bg = new_cell.s.bg;
                }

                let mut char_buf = [0; 4];
                self.buffer
                    .extend_from_slice(new_cell.c.encode_utf8(&mut char_buf).as_bytes());

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

        self.buffer.extend_from_slice(b"\x1b[0m");

        let mut stdout = io::stdout().lock();
        stdout.write_all(&self.buffer).unwrap();
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

        self.needs_clear = true;
    }

    pub fn apply_dim(&mut self) {
        for cell in self.new.iter_mut() {
            if cell.c != ' ' || cell.s.bg != Color::None {
                cell.s.md = Modifier::Dim;
            }
        }
    }

    pub fn put_box(&mut self, buffer: &Box, x: i16, y: i16) {
        self.draw_box(buffer, x, y, true);
    }

    pub fn put_box_opaque(&mut self, buffer: &Box, x: i16, y: i16) {
        self.draw_box(buffer, x, y, false);
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
                    if src_cell.c != ' ' || src_cell.s.bg != Color::None {
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
