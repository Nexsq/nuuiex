use crate::style::{Color, Modifier};

pub struct Cell { // later remove pub (I think lol)
    pub s: String,
    pub fg: Color,
    pub bg: Color,
    pub md: Modifier,
}

pub struct Canvas {
    pub w: u16,
    pub h: u16,
    pub buf: Vec<Cell>,
}

impl Cell {
    pub fn print(&self) {
        println!("{}", self.s);
    }
}

impl Canvas {
    pub fn new(w: u16, h: u16) -> Self {
        Self {
            w,
            h,
            buf: Vec::with_capacity((w * h) as usize), // capacity here might break things, test later
        }
    }

    pub fn add_cell(&mut self, cell: Cell) {
        self.buf.push(cell);
    }

    pub fn print(&self) {
        for (_, cell) in self.buf.iter().enumerate() {
            print!("{}", cell.s);
        }
        println!();
    }
}