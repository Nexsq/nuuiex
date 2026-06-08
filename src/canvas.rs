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