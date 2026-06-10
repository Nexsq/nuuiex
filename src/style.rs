#[derive(Debug, Clone, Copy)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn fg_ansi(&self) -> &'static str {
        match self {
            Color::White => "\x1b[97m",
            Color::Black => "\x1b[30m",
        }
    }

    pub fn bg_ansi(&self) -> &'static str {
        match self {
            Color::White => "\x1b[107m",
            Color::Black => "\x1b[40m",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Modifier {
    None,
    Bold,
    Italic,
}

impl Modifier {
    pub fn to_ansi(&self) -> &'static str {
        match self {
            Modifier::None => "",
            Modifier::Bold => "\x1b[1m",
            Modifier::Italic => "\x1b[3m",
        }
    }
}