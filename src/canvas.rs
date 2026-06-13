use crate::style::{Color, Modifier};
use std::io::{self, Write};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cell {
    pub s: char,
    pub fg: Color,
    pub bg: Color,
    pub md: Modifier,
}

#[derive(Debug)]
pub struct Buffer {
    pub width: u16,
    pub height: u16,
    pub grid: Vec<Cell>,
}

#[derive(Debug)]
pub struct Canvas {
    pub width: u16,
    pub height: u16,
    pub old: Vec<Cell>,
    pub new: Vec<Cell>,
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

impl Buffer {
    pub fn new(width: u16, height: u16) -> Self {
        let size = (width as usize) * (height as usize);
        Self {
            width,
            height,
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
}

impl Canvas {
    pub fn new(width: u16, height: u16) -> Self {
        let size = (width as usize) * (height as usize);
        Self {
            width,
            height,
            old: vec![Cell::default(); size],
            new: vec![Cell::default(); size],
        }
    }

    pub fn clean(&mut self) {
        self.old.fill(Cell::default());
        self.new.fill(Cell::default());
    }

    pub fn put_buffer(&mut self, buffer: &Buffer, x: i16, y: i16) {
        let cw = self.width as i16;
        let ch = self.height as i16;
        let bw = buffer.width as i16;
        let bh = buffer.height as i16;

        if x >= cw || y >= ch || x + bw <= 0 || y + bh <= 0 {
            return;
        }

        let canvas_x = x.max(0) as usize;
        let canvas_y = y.max(0) as usize;
        let buffer_x = if x < 0 { -x as usize } else { 0 };
        let buffer_y = if y < 0 { -y as usize } else { 0 };

        let render_w = ((x + bw).min(cw) - x.max(0)) as usize;
        let render_h = ((y + bh).min(ch) - y.max(0)) as usize;

        let buffer_rows = buffer
            .grid
            .chunks_exact(buffer.width as usize)
            .skip(buffer_y)
            .take(render_h);

        let canvas_rows = self
            .new
            .chunks_exact_mut(self.width as usize)
            .skip(canvas_y)
            .take(render_h);

        for (b_row, c_row) in buffer_rows.zip(canvas_rows) {
            let src_slice = &b_row[buffer_x..buffer_x + render_w];

            let dest_slice = &mut c_row[canvas_x..canvas_x + render_w];

            dest_slice.copy_from_slice(src_slice);
        }
    }

    pub fn render(&self) {
        let mut stdout = io::BufWriter::new(io::stdout().lock());

        write!(stdout, "\x1b[H").unwrap();

        let mut cur_fg = Color::None;
        let mut cur_bg = Color::None;
        let mut cur_md = Modifier::None;

        for row in self.new.chunks_exact(self.width as usize) {
            for cell in row {
                if cell.fg != cur_fg || cell.bg != cur_bg || cell.md != cur_md {
                    if cell.fg == Color::None && cell.bg == Color::None && cell.md == Modifier::None
                    {
                        write!(stdout, "\x1b[0m").unwrap();
                    } else {
                        write!(
                            stdout,
                            "{}{}{}",
                            cell.md.to_ansi(),
                            cell.fg.fg_ansi(),
                            cell.bg.bg_ansi()
                        )
                        .unwrap();
                    }
                    cur_fg = cell.fg;
                    cur_bg = cell.bg;
                    cur_md = cell.md;
                }

                write!(stdout, "{}", cell.s).unwrap();
            }

            write!(stdout, "\n").unwrap();
        }

        write!(stdout, "\x1b[0m").unwrap();
        stdout.flush().unwrap();
    } // this render is temporary and will be replaced with a more efficient diffing render later (yes it was taken from the buffer lol)
}
