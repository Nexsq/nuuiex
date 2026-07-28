use super::style::{Border, Color, Gradient, Modifier, Style};
use std::io::{self, Write};

pub fn char_width(c: char) -> u8 {
    let u = c as u32;
    if u < 0x7F {
        return 1;
    }
    if u >= 0x1100 && u <= 0x115F {
        return 2;
    }
    if u >= 0x2329 && u <= 0x232A {
        return 2;
    }
    if u >= 0x2E80 && u <= 0x303E {
        return 2;
    }
    if u >= 0x3040 && u <= 0xA4CF {
        return 2;
    }
    if u >= 0xAC00 && u <= 0xD7A3 {
        return 2;
    }
    if u >= 0xF900 && u <= 0xFAFF {
        return 2;
    }
    if u >= 0xFE10 && u <= 0xFE19 {
        return 2;
    }
    if u >= 0xFE30 && u <= 0xFE6F {
        return 2;
    }
    if u >= 0xFF00 && u <= 0xFF60 {
        return 2;
    }
    if u >= 0xFFE0 && u <= 0xFFE6 {
        return 2;
    }
    if u >= 0x1F000 && u <= 0x1FAFF {
        return 2;
    }
    if u >= 0x20000 && u <= 0x2FFFD {
        return 2;
    }
    if u >= 0x30000 && u <= 0x3FFFD {
        return 2;
    }
    1
}

pub fn is_combining(c: char) -> bool {
    let u = c as u32;
    u == 0x200D
        || u == 0xFE0F
        || u == 0x20E3
        || (u >= 0x0300 && u <= 0x036F)
        || (u >= 0x1F3FB && u <= 0x1F3FF)
        || (u >= 0x1F1E6 && u <= 0x1F1FF)
        || (u >= 0xE0020 && u <= 0xE007F)
}

#[derive(Clone, Copy)]
pub struct CharCluster {
    pub c: char,
    pub ext: [char; 8],
    pub ext_len: u8,
    pub width: u8,
}

impl CharCluster {
    pub fn new(c: char) -> Self {
        Self {
            c,
            ext: ['\0'; 8],
            ext_len: 0,
            width: char_width(c),
        }
    }
    pub fn push(&mut self, c: char) {
        if (self.ext_len as usize) < self.ext.len() {
            self.ext[self.ext_len as usize] = c;
            self.ext_len += 1;
            if c == '\u{FE0F}' && self.width == 1 {
                self.width = 2;
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cell {
    pub c: char,
    pub ext: [char; 8],
    pub ext_len: u8,
    pub width: u8,
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
        Self {
            c,
            ext: ['\0'; 8],
            ext_len: 0,
            width: char_width(c),
            s,
        }
    }

    pub fn dummy() -> Self {
        Self {
            c: '\0',
            ext: ['\0'; 8],
            ext_len: 0,
            width: 0,
            s: Style::default(),
        }
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self::new(' ', Style::default())
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
                grid[x] = Cell::new(
                    ' ',
                    Style {
                        fg: fg.color_at(x, w),
                        bg: bg.color_at(x, w),
                        md,
                    },
                );
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

        let mut clusters = Vec::new();
        let mut raw_chars = text.chars().peekable();
        while let Some(c) = raw_chars.next() {
            let mut cluster = CharCluster::new(c);
            while let Some(&nc) = raw_chars.peek() {
                if is_combining(nc) {
                    cluster.push(nc);
                    raw_chars.next();
                } else {
                    break;
                }
            }
            clusters.push(cluster);
        }

        let text_len = clusters.len().max(1);
        let mut text_idx = 0;
        let mut iter = clusters.into_iter().peekable();

        while let Some(cluster) = iter.peek().cloned() {
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

            if cluster.c == '\n' {
                cy += 1;
                cx = eff_left;
                iter.next();
                text_idx += 1;
                continue;
            }

            if cluster.c == '\t' {
                let tab_spaces = 4 - ((cx - eff_left) % 4);
                for _ in 0..tab_spaces {
                    if cx < max_x {
                        self.put_cell(Cell::new(' ', style), cx, cy);
                        cx += 1;
                    }
                }
                if cx >= max_x {
                    cx = eff_left;
                    cy += 1;
                }
                iter.next();
                text_idx += 1;
                continue;
            }

            if cluster.c.is_control() {
                iter.next();
                text_idx += 1;
                continue;
            }

            if !word_wrap {
                if cluster.width == 2 && cx + 1 >= max_x {
                    self.put_cell(Cell::new(' ', style), cx, cy);
                    cx = eff_left;
                    cy += 1;
                } else {
                    let cell = Cell {
                        c: cluster.c,
                        ext: cluster.ext,
                        ext_len: cluster.ext_len,
                        width: cluster.width,
                        s: style,
                    };
                    self.put_cell(cell, cx, cy);
                    cx += 1;
                    if cluster.width == 2 {
                        self.put_cell(Cell::dummy(), cx, cy);
                        cx += 1;
                    }
                    if cx >= max_x {
                        cx = eff_left;
                        cy += 1;
                    }
                }
                iter.next();
                text_idx += 1;
            } else {
                if cluster.c.is_whitespace() {
                    if cx > eff_left && cx < max_x {
                        self.put_cell(Cell::new(cluster.c, style), cx, cy);
                        cx += 1;
                    }
                    iter.next();
                    text_idx += 1;
                    continue;
                }

                let mut lookahead = iter.clone();
                let mut word_width = 0;
                let mut word_len = 0;
                while let Some(lc) = lookahead.peek() {
                    if lc.c.is_whitespace() || lc.c.is_control() {
                        break;
                    }
                    word_width += lc.width as u16;
                    word_len += 1;
                    lookahead.next();
                }

                if cx + word_width > max_x && cx > eff_left {
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
                    let next_cluster = iter.next().unwrap();
                    let cfg = fg.color_at(text_idx, text_len);
                    let cbg = bg.color_at(text_idx, text_len);

                    if next_cluster.width == 2 && cx + 1 >= max_x {
                        self.put_cell(
                            Cell::new(
                                ' ',
                                Style {
                                    fg: cfg,
                                    bg: cbg,
                                    md,
                                },
                            ),
                            cx,
                            cy,
                        );
                        cx = eff_left;
                        cy += 1;
                    } else {
                        let cell = Cell {
                            c: next_cluster.c,
                            ext: next_cluster.ext,
                            ext_len: next_cluster.ext_len,
                            width: next_cluster.width,
                            s: Style {
                                fg: cfg,
                                bg: cbg,
                                md,
                            },
                        };
                        self.put_cell(cell, cx, cy);
                        cx += 1;
                        if next_cluster.width == 2 {
                            self.put_cell(Cell::dummy(), cx, cy);
                            cx += 1;
                        }
                        if cx >= max_x {
                            cx = eff_left;
                            cy += 1;
                        }
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
            old: vec![Cell::dummy(); size],
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

        while (y as usize) < (self.height as usize) {
            let idx = (y as usize) * (width as usize) + (x as usize);
            let old_cell = &self.old[idx];
            let new_cell = &self.new[idx];

            if old_cell != new_cell {
                if new_cell.width == 0 {
                    let is_orphaned = x == 0 || self.new[idx - 1].width != 2;
                    if is_orphaned {
                        if cursor_x != x || cursor_y != y {
                            self.buffer.extend_from_slice(b"\x1b[");
                            push_num_u16(&mut self.buffer, y + 1);
                            self.buffer.push(b';');
                            push_num_u16(&mut self.buffer, x + 1);
                            self.buffer.push(b'H');
                        }
                        if cur_md != Modifier::None
                            || cur_bg != Color::None
                            || cur_fg != Color::None
                        {
                            self.buffer.extend_from_slice(b"\x1b[0m");
                            cur_md = Modifier::None;
                            cur_fg = Color::None;
                            cur_bg = Color::None;
                        }
                        self.buffer.extend_from_slice(b" ");
                        cursor_x = x + 1;
                        cursor_y = y;
                        if cursor_x >= width {
                            cursor_x = u16::MAX;
                        }
                    }
                    self.old[idx] = *new_cell;
                    x += 1;
                    if x >= width {
                        x = 0;
                        y += 1;
                    }
                    continue;
                }

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
                for i in 0..new_cell.ext_len as usize {
                    self.buffer
                        .extend_from_slice(new_cell.ext[i].encode_utf8(&mut char_buf).as_bytes());
                }

                cursor_x = x + new_cell.width as u16;
                cursor_y = y;

                self.old[idx] = *new_cell;
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

        self.old.resize(size, Cell::dummy());
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
