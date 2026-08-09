#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    pub fg: Color,
    pub bg: Color,
    pub md: Modifier,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Gradient {
    Solid(Color),
    Linear([Color; 4], u8),
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
            Gradient::Linear(colors, len) => {
                let len = *len as usize;
                if len == 0 {
                    return Color::None;
                }
                if len == 1 || max_x <= 1 {
                    return colors[0];
                }
                if x >= max_x - 1 {
                    return colors[len - 1];
                }

                let t = x as f32 / (max_x - 1) as f32;
                let scaled_t = t * (len - 1) as f32;
                let index = scaled_t as usize;

                if index >= len - 1 {
                    return colors[len - 1];
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
    Default,
    None,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    Gray,
    BrightGray,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
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

fn push_num_u8(buf: &mut Vec<u8>, mut n: u8) {
    if n >= 100 {
        buf.push(b'0' + (n / 100));
        n %= 100;
        buf.push(b'0' + (n / 10));
        buf.push(b'0' + (n % 10));
    } else if n >= 10 {
        buf.push(b'0' + (n / 10));
        buf.push(b'0' + (n % 10));
    } else {
        buf.push(b'0' + n);
    }
}

impl Color {
    pub fn to_rgb(&self) -> Option<(u8, u8, u8)> {
        match self {
            Color::Default => None,
            Color::None => None,
            Color::Black => Some((0, 0, 0)),
            Color::Red => Some((205, 49, 49)),
            Color::Green => Some((13, 188, 121)),
            Color::Yellow => Some((229, 229, 16)),
            Color::Blue => Some((36, 114, 200)),
            Color::Magenta => Some((188, 63, 188)),
            Color::Cyan => Some((17, 168, 205)),
            Color::Gray => Some((102, 102, 102)),
            Color::BrightGray => Some((229, 229, 229)),
            Color::White => Some((255, 255, 255)),
            Color::BrightRed => Some((241, 76, 76)),
            Color::BrightGreen => Some((35, 209, 139)),
            Color::BrightYellow => Some((245, 245, 67)),
            Color::BrightBlue => Some((59, 142, 234)),
            Color::BrightMagenta => Some((214, 112, 214)),
            Color::BrightCyan => Some((41, 184, 219)),
            Color::Rgb(r, g, b) => Some((*r, *g, *b)),
        }
    }

    pub fn interpolate(&self, other: Color, t: f32) -> Color {
        if t <= 0.0 {
            return *self;
        }
        if t >= 1.0 {
            return other;
        }
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
            Color::Default | Color::None => buf.extend_from_slice(b"\x1b[39m"),
            Color::Black => buf.extend_from_slice(b"\x1b[30m"),
            Color::Red => buf.extend_from_slice(b"\x1b[31m"),
            Color::Green => buf.extend_from_slice(b"\x1b[32m"),
            Color::Yellow => buf.extend_from_slice(b"\x1b[33m"),
            Color::Blue => buf.extend_from_slice(b"\x1b[34m"),
            Color::Magenta => buf.extend_from_slice(b"\x1b[35m"),
            Color::Cyan => buf.extend_from_slice(b"\x1b[36m"),
            Color::Gray => buf.extend_from_slice(b"\x1b[90m"),
            Color::BrightGray => buf.extend_from_slice(b"\x1b[37m"),
            Color::White => buf.extend_from_slice(b"\x1b[97m"),
            Color::BrightRed => buf.extend_from_slice(b"\x1b[91m"),
            Color::BrightGreen => buf.extend_from_slice(b"\x1b[92m"),
            Color::BrightYellow => buf.extend_from_slice(b"\x1b[93m"),
            Color::BrightBlue => buf.extend_from_slice(b"\x1b[94m"),
            Color::BrightMagenta => buf.extend_from_slice(b"\x1b[95m"),
            Color::BrightCyan => buf.extend_from_slice(b"\x1b[96m"),
            Color::Rgb(r, g, b) => {
                buf.extend_from_slice(b"\x1b[38;2;");
                push_num_u8(buf, *r);
                buf.push(b';');
                push_num_u8(buf, *g);
                buf.push(b';');
                push_num_u8(buf, *b);
                buf.push(b'm');
            }
        }
    }

    pub fn bg_ansi(&self, buf: &mut Vec<u8>) {
        match self {
            Color::Default | Color::None => buf.extend_from_slice(b"\x1b[49m"),
            Color::Black => buf.extend_from_slice(b"\x1b[40m"),
            Color::Red => buf.extend_from_slice(b"\x1b[41m"),
            Color::Green => buf.extend_from_slice(b"\x1b[42m"),
            Color::Yellow => buf.extend_from_slice(b"\x1b[43m"),
            Color::Blue => buf.extend_from_slice(b"\x1b[44m"),
            Color::Magenta => buf.extend_from_slice(b"\x1b[45m"),
            Color::Cyan => buf.extend_from_slice(b"\x1b[46m"),
            Color::Gray => buf.extend_from_slice(b"\x1b[100m"),
            Color::BrightGray => buf.extend_from_slice(b"\x1b[47m"),
            Color::White => buf.extend_from_slice(b"\x1b[107m"),
            Color::BrightRed => buf.extend_from_slice(b"\x1b[101m"),
            Color::BrightGreen => buf.extend_from_slice(b"\x1b[102m"),
            Color::BrightYellow => buf.extend_from_slice(b"\x1b[103m"),
            Color::BrightBlue => buf.extend_from_slice(b"\x1b[104m"),
            Color::BrightMagenta => buf.extend_from_slice(b"\x1b[105m"),
            Color::BrightCyan => buf.extend_from_slice(b"\x1b[106m"),
            Color::Rgb(r, g, b) => {
                buf.extend_from_slice(b"\x1b[48;2;");
                push_num_u8(buf, *r);
                buf.push(b';');
                push_num_u8(buf, *g);
                buf.push(b';');
                push_num_u8(buf, *b);
                buf.push(b'm');
            }
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
