use crate::style::Color;

pub struct Cell {
    pub s: String,
    pub fg: Color,
    pub bg: Color,
}

pub struct Canvas {
    pub w: u16,
    pub h: u16,
    pub buf: Vec<Cell>,
}