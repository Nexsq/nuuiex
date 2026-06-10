use crate::style::{Color, Modifier};
use std::io::{self, Write};

#[derive(Debug, Clone)]
pub struct Cell {
    // later remove pub (I think lol)
    pub s: char,
    pub fg: Color,
    pub bg: Color,
    pub md: Modifier,
}

#[derive(Debug)]
pub struct Canvas {
    pub width: u16,
    pub height: u16,
    pub buffer: Vec<Cell>,
}

impl Canvas {
    pub fn new(width: u16, height: u16) -> Self {
        let size = (width as usize) * (height as usize);
        Self {
            width,
            height,
            buffer: vec![
                Cell {
                    s: ' ',
                    fg: Color::None,
                    bg: Color::None,
                    md: Modifier::None,
                };
                size
            ],
        }
    }

    fn index_of(&self, x: u16, y: u16) -> usize {
        (y as usize) * (self.width as usize) + (x as usize)
    }

    pub fn put_cell(&mut self, cell: Cell, x: u16, y: u16) {
        if x < self.width && y < self.height {
            let idx = self.index_of(x, y);
            self.buffer[idx] = cell;
        }
    }

    pub fn clean(&mut self) {
        self.buffer.fill(Cell {
            s: ' ',
            fg: Color::None,
            bg: Color::None,
            md: Modifier::None,
        });
    }

    pub fn render(&self) {
        let mut stdout = io::BufWriter::new(io::stdout().lock());

        write!(stdout, "\x1b[H").unwrap();

        let mut cur_fg = Color::None;
        let mut cur_bg = Color::None;
        let mut cur_md = Modifier::None;

        for row in self.buffer.chunks_exact(self.width as usize) {
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
    }
}
