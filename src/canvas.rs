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
        Self {
            width,
            height,
            buffer: vec![
                Cell {
                    s: ' ',
                    fg: Color::White,
                    bg: Color::Black,
                    md: Modifier::None,
                };
                (width * height) as usize
            ],
        }
    }

    fn index_of(&self, x: u16, y: u16) -> usize {
        (y * self.width + x) as usize
    }

    pub fn put_cell(&mut self, cell: Cell, x: u16, y: u16) {
        if x < self.width && y < self.height {
            let idx = self.index_of(x, y);
            self.buffer[idx] = cell;
        }
    }

    pub fn clean(&mut self) {
        for cell in self.buffer.iter_mut() {
            cell.s = ' ';
            // reset colors and modifiers here too
        }
    }

    pub fn render(&self) {
        let mut stdout = io::BufWriter::new(io::stdout().lock());

        write!(stdout, "\x1b[H").unwrap(); // reset cursor to 0,0

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = self.index_of(x, y);
                let cell = &self.buffer[idx];

                // optimize to not inclue unnecessary ansi escape codes for repeating patterns later
                write!(
                    stdout,
                    "{}{}{}{}\x1b[0m",
                    cell.md.to_ansi(),
                    cell.fg.fg_ansi(),
                    cell.bg.bg_ansi(),
                    cell.s
                ).unwrap();
            }
            write!(stdout, "\n").unwrap();
        }

        stdout.flush().unwrap();
    }
}
