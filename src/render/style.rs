use std::io::Write;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    pub fg: Color,
    pub bg: Color,
    pub md: Modifier,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Gradient {
    Solid(Color),
    Linear(Vec<Color>),
}

impl Default for Gradient {
    fn default() -> Self {
        Gradient::Solid(Color::None)
    }
}

impl Gradient {
    pub fn color_at(&self, x: usize, max_x: usize) -> Color {
        match self {
            Gradient::Solid(c) => *c,
            Gradient::Linear(colors) => {
                if colors.is_empty() {
                    return Color::None;
                }
                if colors.len() == 1 || max_x <= 1 {
                    return colors[0];
                }
                let t = (x as f32 / (max_x - 1) as f32).clamp(0.0, 1.0);
                let segments = (colors.len() - 1) as f32;
                let scaled_t = t * segments;
                let index = scaled_t.floor() as usize;

                if index >= colors.len() - 1 {
                    return colors.last().copied().unwrap_or(Color::None);
                }

                let local_t = scaled_t - index as f32;
                let c1 = colors[index];
                let c2 = colors[index + 1];

                c1.interpolate(c2, local_t)
            }
        }
    }
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
    pub fn to_rgb(&self) -> Option<(u8, u8, u8)> {
        match self {
            Color::None => None,
            Color::Black => Some((0, 0, 0)),
            Color::Red => Some((205, 49, 49)),
            Color::Green => Some((13, 188, 121)),
            Color::Yellow => Some((229, 229, 16)),
            Color::Blue => Some((36, 114, 200)),
            Color::Magenta => Some((188, 63, 188)),
            Color::Cyan => Some((17, 168, 205)),
            Color::White => Some((229, 229, 229)),
            Color::DarkGray => Some((102, 102, 102)),
            Color::BrightRed => Some((241, 76, 76)),
            Color::BrightGreen => Some((35, 209, 139)),
            Color::BrightYellow => Some((245, 245, 67)),
            Color::BrightBlue => Some((59, 142, 234)),
            Color::BrightMagenta => Some((214, 112, 214)),
            Color::BrightCyan => Some((41, 184, 219)),
            Color::BrightWhite => Some((255, 255, 255)),
            Color::Rgb(r, g, b) => Some((*r, *g, *b)),
        }
    }

    pub fn interpolate(&self, other: Color, t: f32) -> Color {
        if let (Some((r1, g1, b1)), Some((r2, g2, b2))) = (self.to_rgb(), other.to_rgb()) {
            let inv_t = 1.0 - t;
            let r = (r1 as f32 * inv_t + r2 as f32 * t) as u8;
            let g = (g1 as f32 * inv_t + g2 as f32 * t) as u8;
            let b = (b1 as f32 * inv_t + b2 as f32 * t) as u8;

            Color::Rgb(r, g, b)
        } else {
            *self
        }
    }

    pub fn fg_ansi(&self, buf: &mut Vec<u8>) {
        match self {
            Color::None => buf.write_all(b"\x1b[39m").unwrap(),
            Color::Black => buf.write_all(b"\x1b[30m").unwrap(),
            Color::Red => buf.write_all(b"\x1b[31m").unwrap(),
            Color::Green => buf.write_all(b"\x1b[32m").unwrap(),
            Color::Yellow => buf.write_all(b"\x1b[33m").unwrap(),
            Color::Blue => buf.write_all(b"\x1b[34m").unwrap(),
            Color::Magenta => buf.write_all(b"\x1b[35m").unwrap(),
            Color::Cyan => buf.write_all(b"\x1b[36m").unwrap(),
            Color::White => buf.write_all(b"\x1b[37m").unwrap(),
            Color::DarkGray => buf.write_all(b"\x1b[90m").unwrap(),
            Color::BrightRed => buf.write_all(b"\x1b[91m").unwrap(),
            Color::BrightGreen => buf.write_all(b"\x1b[92m").unwrap(),
            Color::BrightYellow => buf.write_all(b"\x1b[93m").unwrap(),
            Color::BrightBlue => buf.write_all(b"\x1b[94m").unwrap(),
            Color::BrightMagenta => buf.write_all(b"\x1b[95m").unwrap(),
            Color::BrightCyan => buf.write_all(b"\x1b[96m").unwrap(),
            Color::BrightWhite => buf.write_all(b"\x1b[97m").unwrap(),
            Color::Rgb(r, g, b) => write!(buf, "\x1b[38;2;{};{};{}m", r, g, b).unwrap(),
        }
    }

    pub fn bg_ansi(&self, buf: &mut Vec<u8>) {
        match self {
            Color::None => buf.write_all(b"\x1b[49m").unwrap(),
            Color::Black => buf.write_all(b"\x1b[40m").unwrap(),
            Color::Red => buf.write_all(b"\x1b[41m").unwrap(),
            Color::Green => buf.write_all(b"\x1b[42m").unwrap(),
            Color::Yellow => buf.write_all(b"\x1b[43m").unwrap(),
            Color::Blue => buf.write_all(b"\x1b[44m").unwrap(),
            Color::Magenta => buf.write_all(b"\x1b[45m").unwrap(),
            Color::Cyan => buf.write_all(b"\x1b[46m").unwrap(),
            Color::White => buf.write_all(b"\x1b[47m").unwrap(),
            Color::DarkGray => buf.write_all(b"\x1b[100m").unwrap(),
            Color::BrightRed => buf.write_all(b"\x1b[101m").unwrap(),
            Color::BrightGreen => buf.write_all(b"\x1b[102m").unwrap(),
            Color::BrightYellow => buf.write_all(b"\x1b[103m").unwrap(),
            Color::BrightBlue => buf.write_all(b"\x1b[104m").unwrap(),
            Color::BrightMagenta => buf.write_all(b"\x1b[105m").unwrap(),
            Color::BrightCyan => buf.write_all(b"\x1b[106m").unwrap(),
            Color::BrightWhite => buf.write_all(b"\x1b[107m").unwrap(),
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
