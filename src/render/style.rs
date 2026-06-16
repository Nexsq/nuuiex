#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    pub fg: Color,
    pub bg: Color,
    pub md: Modifier,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    None,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    DarkGray,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
    Rgb(u8, u8, u8),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Modifier {
    None,
    Bold,
    Dim,
    Italic,
    Underline,
    Reverse,
    Hidden,
    Strikethrough,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Border {
    None,
    Light,
    Heavy,
    Double,
    Rounded,
}

pub struct BorderChars {
    pub tl: char,
    pub tr: char,
    pub bl: char,
    pub br: char,
    pub h: char,
    pub v: char,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            fg: Color::None,
            bg: Color::None,
            md: Modifier::None,
        }
    }
}

impl Color {
    pub fn fg_ansi(&self, buf: &mut String) {
        use std::fmt::Write;
        match self {
            Color::None => buf.write_str("\x1b[39m").unwrap(),
            Color::Black => buf.write_str("\x1b[30m").unwrap(),
            Color::Red => buf.write_str("\x1b[31m").unwrap(),
            Color::Green => buf.write_str("\x1b[32m").unwrap(),
            Color::Yellow => buf.write_str("\x1b[33m").unwrap(),
            Color::Blue => buf.write_str("\x1b[34m").unwrap(),
            Color::Magenta => buf.write_str("\x1b[35m").unwrap(),
            Color::Cyan => buf.write_str("\x1b[36m").unwrap(),
            Color::White => buf.write_str("\x1b[37m").unwrap(),
            Color::DarkGray => buf.write_str("\x1b[90m").unwrap(),
            Color::BrightRed => buf.write_str("\x1b[91m").unwrap(),
            Color::BrightGreen => buf.write_str("\x1b[92m").unwrap(),
            Color::BrightYellow => buf.write_str("\x1b[93m").unwrap(),
            Color::BrightBlue => buf.write_str("\x1b[94m").unwrap(),
            Color::BrightMagenta => buf.write_str("\x1b[95m").unwrap(),
            Color::BrightCyan => buf.write_str("\x1b[96m").unwrap(),
            Color::BrightWhite => buf.write_str("\x1b[97m").unwrap(),
            Color::Rgb(r, g, b) => write!(buf, "\x1b[38;2;{};{};{}m", r, g, b).unwrap(),
        }
    }

    pub fn bg_ansi(&self, buf: &mut String) {
        use std::fmt::Write;
        match self {
            Color::None => buf.write_str("\x1b[49m").unwrap(),
            Color::Black => buf.write_str("\x1b[40m").unwrap(),
            Color::Red => buf.write_str("\x1b[41m").unwrap(),
            Color::Green => buf.write_str("\x1b[42m").unwrap(),
            Color::Yellow => buf.write_str("\x1b[43m").unwrap(),
            Color::Blue => buf.write_str("\x1b[44m").unwrap(),
            Color::Magenta => buf.write_str("\x1b[45m").unwrap(),
            Color::Cyan => buf.write_str("\x1b[46m").unwrap(),
            Color::White => buf.write_str("\x1b[47m").unwrap(),
            Color::DarkGray => buf.write_str("\x1b[100m").unwrap(),
            Color::BrightRed => buf.write_str("\x1b[101m").unwrap(),
            Color::BrightGreen => buf.write_str("\x1b[102m").unwrap(),
            Color::BrightYellow => buf.write_str("\x1b[103m").unwrap(),
            Color::BrightBlue => buf.write_str("\x1b[104m").unwrap(),
            Color::BrightMagenta => buf.write_str("\x1b[105m").unwrap(),
            Color::BrightCyan => buf.write_str("\x1b[106m").unwrap(),
            Color::BrightWhite => buf.write_str("\x1b[107m").unwrap(),
            Color::Rgb(r, g, b) => write!(buf, "\x1b[48;2;{};{};{}m", r, g, b).unwrap(),
        }
    }
}

impl Modifier {
    pub fn to_ansi(&self) -> &'static str {
        match self {
            Modifier::None => "",
            Modifier::Bold => "\x1b[1m",
            Modifier::Dim => "\x1b[2m",
            Modifier::Italic => "\x1b[3m",
            Modifier::Underline => "\x1b[4m",
            Modifier::Reverse => "\x1b[7m",
            Modifier::Hidden => "\x1b[8m",
            Modifier::Strikethrough => "\x1b[9m",
        }
    }
}

impl Border {
    pub fn chars(&self) -> Option<BorderChars> {
        match self {
            Border::None => None,
            Border::Light => Some(BorderChars {
                tl: '┌',
                tr: '┐',
                bl: '└',
                br: '┘',
                h: '─',
                v: '│',
            }),
            Border::Heavy => Some(BorderChars {
                tl: '┏',
                tr: '┓',
                bl: '┗',
                br: '┛',
                h: '━',
                v: '┃',
            }),
            Border::Double => Some(BorderChars {
                tl: '╔',
                tr: '╗',
                bl: '╚',
                br: '╝',
                h: '═',
                v: '║',
            }),
            Border::Rounded => Some(BorderChars {
                tl: '╭',
                tr: '╮',
                bl: '╰',
                br: '╯',
                h: '─',
                v: '│',
            }),
        }
    }
}
