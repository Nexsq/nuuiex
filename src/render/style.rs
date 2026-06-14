#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    None,
    White,
    Black,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modifier {
    None,
    Bold,
    Italic,
}

impl Color {
    pub fn fg_ansi(&self) -> &'static str {
        match self {
            Color::None => "",
            Color::White => "\x1b[97m",
            Color::Black => "\x1b[30m",
        }
    }

    pub fn bg_ansi(&self) -> &'static str {
        match self {
            Color::None => "",
            Color::White => "\x1b[107m",
            Color::Black => "\x1b[40m",
        }
    }
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
