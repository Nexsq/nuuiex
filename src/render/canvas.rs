use super::style::{Border, Color, Modifier, Style};
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
    pub buffer: String,
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
    pub fn new(width: u16, height: u16, padding: u16, border: Border, style: Style) -> Self {
        let size = (width as usize) * (height as usize);
        let mut grid = vec![Cell::default(); size];

        if let Some(chars) = border.chars() {
            if width >= 2 && height >= 2 {
                let w = width as usize;
                let h = height as usize;

                let h_cell = Cell {
                    c: chars.h,
                    s: style,
                };
                let v_cell = Cell {
                    c: chars.v,
                    s: style,
                };

                for x in 1..(w - 1) {
                    grid[x] = h_cell;
                    grid[(h - 1) * w + x] = h_cell;
                }

                for y in 1..(h - 1) {
                    grid[y * w] = v_cell;
                    grid[y * w + (w - 1)] = v_cell;
                }

                grid[0] = Cell {
                    c: chars.tl,
                    s: style,
                };
                grid[w - 1] = Cell {
                    c: chars.tr,
                    s: style,
                };
                grid[(h - 1) * w] = Cell {
                    c: chars.bl,
                    s: style,
                };
                grid[(h - 1) * w + (w - 1)] = Cell {
                    c: chars.br,
                    s: style,
                };
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
        style: Style,
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

            if c == '\t' {
                let tab_spaces = 4 - ((cx - eff_left) % 4);
                for _ in 0..tab_spaces {
                    if cx < max_x {
                        let mut cell = Cell { c: '\0', s: style };
                        cell.c = ' ';
                        self.put_cell(cell, cx, cy);
                        cx += 1;
                    }
                }
                if cx >= max_x {
                    cx = eff_left;
                    cy += 1;
                }
                chars.next();
                continue;
            }

            if c.is_control() {
                chars.next();
                continue;
            }

            if !word_wrap {
                let mut cell = Cell { c: '\0', s: style };
                cell.c = c;
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
                        let mut cell = Cell { c: '\0', s: style };
                        cell.c = c;
                        self.put_cell(cell, cx, cy);
                        cx += 1;
                    }
                    chars.next();
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
                    let mut cell = Cell { c: '\0', s: style };
                    cell.c = chars.next().unwrap();
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

    pub fn set_border_color(&mut self, color: Color) {
        if self.width < 2 || self.height < 2 {
            return;
        }

        let w = self.width as usize;
        let h = self.height as usize;

        for x in 0..w {
            self.grid[x].s.fg = color;
            self.grid[(h - 1) * w + x].s.fg = color;
        }
        for y in 0..h {
            self.grid[y * w].s.fg = color;
            self.grid[y * w + (w - 1)].s.fg = color;
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

impl Canvas {
    pub fn new(width: u16, height: u16) -> Self {
        let size = (width as usize) * (height as usize);

        let imp_cell = Cell {
            c: '\0',
            s: Style::default(),
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

                if new_cell.s.md != cur_md {
                    if cur_md != Modifier::None {
                        write!(&mut self.buffer, "\x1b[0m").unwrap();
                        cur_fg = Color::None;
                        cur_bg = Color::None;
                    }

                    if new_cell.s.md != Modifier::None {
                        write!(&mut self.buffer, "{}", new_cell.s.md.to_ansi()).unwrap();
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

                write!(&mut self.buffer, "{}", new_cell.c).unwrap();

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

        #[inline(always)]
        fn process_cell(src_cell: &Cell, dest_cell: &mut Cell, ignore_spaces: bool) {
            if ignore_spaces && *src_cell == Cell::default() {
                return;
            }

            let src_info = get_border_info(src_cell.c);
            let dest_info = get_border_info(dest_cell.c);

            let src_mask = src_info & 0x0F;
            let dest_mask = dest_info & 0x0F;

            if src_mask > 0 && dest_mask > 0 {
                let combined_mask = src_mask | dest_mask;
                let dominant_fam = (src_info & 0xF0).max(dest_info & 0xF0);

                if let Some(merged_char) = mask_to_char(combined_mask, dominant_fam) {
                    dest_cell.c = merged_char;
                    dest_cell.s = src_cell.s;
                    return;
                }
            }
            *dest_cell = *src_cell;
        }

        let mut row_iter = buffer_rows.zip(self_rows);

        if src_y == 0 {
            if let Some((b_row, s_row)) = row_iter.next() {
                let src_slice = &b_row[src_x..src_x + render_w];
                let dest_slice = &mut s_row[dest_x..dest_x + render_w];
                for (src_cell, dest_cell) in src_slice.iter().zip(dest_slice.iter_mut()) {
                    process_cell(src_cell, dest_cell, ignore_spaces);
                }
            }
        }

        let mut bottom_row = None;
        if (src_y + render_h) == (bh as usize) && render_h > (if src_y == 0 { 1 } else { 0 }) {
            bottom_row = row_iter.next_back();
        }

        for (b_row, s_row) in row_iter {
            let src_slice = &b_row[src_x..src_x + render_w];
            let dest_slice = &mut s_row[dest_x..dest_x + render_w];

            let mut start_idx = 0;
            let mut end_idx = render_w;

            if src_x == 0 && render_w > 0 {
                process_cell(&src_slice[0], &mut dest_slice[0], ignore_spaces);
                start_idx += 1;
            }

            if src_x + render_w == (bw as usize) && render_w > start_idx {
                process_cell(
                    &src_slice[render_w - 1],
                    &mut dest_slice[render_w - 1],
                    ignore_spaces,
                );
                end_idx -= 1;
            }

            if start_idx < end_idx {
                let src_interior = &src_slice[start_idx..end_idx];
                let dest_interior = &mut dest_slice[start_idx..end_idx];

                if ignore_spaces {
                    for (src_cell, dest_cell) in src_interior.iter().zip(dest_interior.iter_mut()) {
                        if *src_cell != Cell::default() {
                            *dest_cell = *src_cell;
                        }
                    }
                } else {
                    dest_interior.copy_from_slice(src_interior);
                }
            }
        }

        if let Some((b_row, s_row)) = bottom_row {
            let src_slice = &b_row[src_x..src_x + render_w];
            let dest_slice = &mut s_row[dest_x..dest_x + render_w];
            for (src_cell, dest_cell) in src_slice.iter().zip(dest_slice.iter_mut()) {
                process_cell(src_cell, dest_cell, ignore_spaces);
            }
        }
    }
}

#[inline(always)]
fn get_border_info(c: char) -> u8 {
    match c {
        '─' => 0x00 | (2 | 8),
        '│' => 0x00 | (1 | 4),
        '┌' | '╭' => 0x00 | (4 | 2),
        '┐' | '╮' => 0x00 | (4 | 8),
        '└' | '╰' => 0x00 | (1 | 2),
        '┘' | '╯' => 0x00 | (1 | 8),
        '├' => 0x00 | (1 | 4 | 2),
        '┤' => 0x00 | (1 | 4 | 8),
        '┬' => 0x00 | (4 | 2 | 8),
        '┴' => 0x00 | (1 | 2 | 8),
        '┼' => 0x00 | (1 | 2 | 4 | 8),

        '═' => 0x10 | (2 | 8),
        '║' => 0x10 | (1 | 4),
        '╔' => 0x10 | (4 | 2),
        '╗' => 0x10 | (4 | 8),
        '╚' => 0x10 | (1 | 2),
        '╝' => 0x10 | (1 | 8),
        '╠' => 0x10 | (1 | 4 | 2),
        '╣' => 0x10 | (1 | 4 | 8),
        '╦' => 0x10 | (4 | 2 | 8),
        '╩' => 0x10 | (1 | 2 | 8),
        '╬' => 0x10 | (1 | 2 | 4 | 8),

        '━' => 0x20 | (2 | 8),
        '┃' => 0x20 | (1 | 4),
        '┏' => 0x20 | (4 | 2),
        '┓' => 0x20 | (4 | 8),
        '┗' => 0x20 | (1 | 2),
        '┛' => 0x20 | (1 | 8),
        '┣' => 0x20 | (1 | 4 | 2),
        '┫' => 0x20 | (1 | 4 | 8),
        '┳' => 0x20 | (4 | 2 | 8),
        '┻' => 0x20 | (1 | 2 | 8),
        '╋' => 0x20 | (1 | 2 | 4 | 8),

        _ => 0,
    }
}

#[inline(always)]
fn mask_to_char(mask: u8, family_score: u8) -> Option<char> {
    match family_score {
        0x00 => match mask {
            3 => Some('└'),
            5 => Some('│'),
            6 => Some('┌'),
            7 => Some('├'),
            9 => Some('┘'),
            10 => Some('─'),
            11 => Some('┴'),
            12 => Some('┐'),
            13 => Some('┤'),
            14 => Some('┬'),
            15 => Some('┼'),
            _ => None,
        },
        0x10 => match mask {
            3 => Some('╚'),
            5 => Some('║'),
            6 => Some('╔'),
            7 => Some('╠'),
            9 => Some('╝'),
            10 => Some('═'),
            11 => Some('╩'),
            12 => Some('╗'),
            13 => Some('╣'),
            14 => Some('╦'),
            15 => Some('╬'),
            _ => None,
        },
        0x20 => match mask {
            3 => Some('┗'),
            5 => Some('┃'),
            6 => Some('┏'),
            7 => Some('┣'),
            9 => Some('┛'),
            10 => Some('━'),
            11 => Some('┻'),
            12 => Some('┓'),
            13 => Some('┫'),
            14 => Some('┳'),
            15 => Some('╋'),
            _ => None,
        },
        _ => None,
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
