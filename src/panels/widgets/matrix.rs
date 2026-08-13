use crate::{Box, Color, Gradient, Modifier, conf::Config, theme::themecore::Theme};

const CHARS: &[char] = &[
    'ﾊ', 'ﾐ', 'ﾋ', 'ｰ', 'ｳ', 'ｼ', 'ﾅ', 'ﾓ', 'ﾆ', 'ｻ', 'ﾜ', 'ﾂ', 'ｵ', 'ﾘ', 'ｱ', 'ﾎ', 'ﾃ', 'ﾏ', 'ｹ',
    'ﾒ', 'ｴ', 'ｶ', 'ｷ', 'ﾑ', 'ﾕ', 'ﾗ', 'ｾ', 'ﾈ', 'ｽ', 'ﾀ', 'ﾇ', 'ﾍ', '0', '1', '2', '3', '4', '5',
    '6', '7', '8', '9', 'Z', 'X', 'C', 'V', 'B', 'N', 'M', '=', '*', '+', '-', '<', '>',
];

pub struct MatrixState {
    pub rand_seed: u32,
    pub drops: Vec<Drop>,
    pub last_w: u16,
    pub last_h: u16,
    pub last_density: usize,
    pub last_dir: String,
}

pub struct Drop {
    pub x: f32,
    pub y: f32,
    pub speed: f32,
    pub length: usize,
    pub chars: Vec<char>,
    pub dimmed: bool,
}

impl MatrixState {
    pub fn new() -> Self {
        Self {
            rand_seed: 1337,
            drops: Vec::new(),
            last_w: 0,
            last_h: 0,
            last_density: 0,
            last_dir: String::new(),
        }
    }

    #[inline(always)]
    fn next_rand(&mut self) -> f32 {
        self.rand_seed = self
            .rand_seed
            .wrapping_mul(1664525)
            .wrapping_add(1013904223);
        (self.rand_seed >> 8) as f32 / 16777216.0
    }

    #[inline(always)]
    fn next_rand_idx(&mut self, max: usize) -> usize {
        self.rand_seed = self
            .rand_seed
            .wrapping_mul(1664525)
            .wrapping_add(1013904223);
        ((self.rand_seed >> 8) as usize) % max
    }

    fn init_drop(&mut self, w: u16, h: u16, config: &Config) -> Drop {
        let mut drop = Drop {
            x: 0.0,
            y: 0.0,
            speed: 0.0,
            length: 0,
            chars: Vec::with_capacity(64),
            dimmed: false,
        };
        self.reset_drop(&mut drop, w, h, config);

        let dir = config.matrix_direction.as_str();
        match dir {
            "left" | "right" => {
                drop.x = self.next_rand() * w as f32;
            }
            _ => {
                drop.y = self.next_rand() * h as f32;
            }
        }
        drop
    }

    fn reset_drop(&mut self, drop: &mut Drop, w: u16, h: u16, config: &Config) {
        let dir = config.matrix_direction.as_str();

        match dir {
            "up" => {
                drop.x = self.next_rand_idx(w as usize) as f32;
                drop.y = h as f32 + self.next_rand() * h as f32 + 5.0;
            }
            "left" => {
                drop.x = w as f32 + self.next_rand() * w as f32 + 5.0;
                drop.y = self.next_rand_idx(h as usize) as f32;
            }
            "right" => {
                drop.x = -(self.next_rand() * w as f32) - 5.0;
                drop.y = self.next_rand_idx(h as usize) as f32;
            }
            _ => {
                drop.x = self.next_rand_idx(w as usize) as f32;
                drop.y = -(self.next_rand() * h as f32) - 5.0;
            }
        }

        let base_speed = 0.5 + self.next_rand() * 1.5;
        drop.speed = base_speed * config.matrix_speed;

        let min_l = config.matrix_min_length;
        let max_l = config.matrix_max_length.max(min_l);
        drop.length = min_l + self.next_rand_idx(max_l - min_l + 1);

        drop.dimmed = self.next_rand_idx(100) < config.matrix_dim_ratio;

        drop.chars.clear();
        for _ in 0..drop.length {
            drop.chars.push(CHARS[self.next_rand_idx(CHARS.len())]);
        }
    }

    pub fn tick(&mut self, w: u16, h: u16, config: &Config) -> bool {
        if w == 0 || h == 0 {
            return false;
        }

        let area = (w as usize) * (h as usize);
        let target_drops = ((area * config.matrix_density) / 1000).clamp(1, 2048);

        if self.last_w != w
            || self.last_h != h
            || self.last_density != config.matrix_density
            || self.last_dir != config.matrix_direction
        {
            self.last_w = w;
            self.last_h = h;
            self.last_density = config.matrix_density;
            self.last_dir = config.matrix_direction.clone();

            self.drops.clear();
            self.drops.reserve(target_drops);
            for _ in 0..target_drops {
                let drop = self.init_drop(w, h, config);
                self.drops.push(drop);
            }
        }

        let w_f32 = w as f32;
        let h_f32 = h as f32;
        let dir = config.matrix_direction.as_str();

        let mut current_drops = std::mem::take(&mut self.drops);

        for drop in &mut current_drops {
            match dir {
                "up" => drop.y -= drop.speed,
                "left" => drop.x -= drop.speed,
                "right" => drop.x += drop.speed,
                _ => drop.y += drop.speed,
            }

            if self.next_rand_idx(5) == 0 && !drop.chars.is_empty() {
                let idx = self.next_rand_idx(drop.chars.len());
                drop.chars[idx] = CHARS[self.next_rand_idx(CHARS.len())];
            }

            let respawn = match dir {
                "up" => drop.y + (drop.length as f32) < 0.0,
                "left" => drop.x + (drop.length as f32) < 0.0,
                "right" => drop.x - (drop.length as f32) > w_f32,
                _ => drop.y - (drop.length as f32) > h_f32,
            };

            if respawn {
                self.reset_drop(drop, w, h, config);
            }
        }

        self.drops = current_drops;
        true
    }
}

pub fn draw(state: &MatrixState, b: &mut Box, config: &Config, theme: &Theme) {
    if b.width == 0 || b.height == 0 {
        return;
    }

    let head_grad = if theme.matrix_head_color == Gradient::Solid(Color::None) {
        Gradient::Solid(Color::White)
    } else {
        theme.matrix_head_color
    };

    let body_grad = if theme.matrix_body_color == Gradient::Solid(Color::None) {
        theme.keyview_color
    } else {
        theme.matrix_body_color
    };

    let b_width = b.width as i32;
    let b_height = b.height as i32;
    let head_color = head_grad.color_at(0, 1);
    let dir = config.matrix_direction.as_str();

    match dir {
        "up" => {
            for drop in &state.drops {
                let head_x = drop.x as i32;
                let head_y = drop.y as i32;
                let length = drop.length;
                let head_md = if drop.dimmed {
                    Modifier::None
                } else {
                    Modifier::Bold
                };
                let body_md = if drop.dimmed {
                    Modifier::Dim
                } else {
                    Modifier::None
                };

                for dy in 0..length {
                    let sy = head_y + dy as i32;
                    if head_x >= 0 && head_x < b_width && sy >= 0 && sy < b_height {
                        let target_idx = (sy as usize) * (b_width as usize) + (head_x as usize);
                        let (fg, md) = if dy == 0 {
                            (head_color, head_md)
                        } else {
                            (body_grad.color_at(dy, length), body_md)
                        };
                        let cell = &mut b.grid[target_idx];
                        cell.c = drop.chars[dy];
                        cell.s.fg = fg;
                        cell.s.md = md;
                        cell.ext_len = 0;
                        cell.width = 1;
                    }
                }
            }
        }
        "left" => {
            for drop in &state.drops {
                let head_x = drop.x as i32;
                let head_y = drop.y as i32;
                let length = drop.length;
                let head_md = if drop.dimmed {
                    Modifier::None
                } else {
                    Modifier::Bold
                };
                let body_md = if drop.dimmed {
                    Modifier::Dim
                } else {
                    Modifier::None
                };

                for dy in 0..length {
                    let sx = head_x + dy as i32;
                    if sx >= 0 && sx < b_width && head_y >= 0 && head_y < b_height {
                        let target_idx = (head_y as usize) * (b_width as usize) + (sx as usize);
                        let (fg, md) = if dy == 0 {
                            (head_color, head_md)
                        } else {
                            (body_grad.color_at(dy, length), body_md)
                        };
                        let cell = &mut b.grid[target_idx];
                        cell.c = drop.chars[dy];
                        cell.s.fg = fg;
                        cell.s.md = md;
                        cell.ext_len = 0;
                        cell.width = 1;
                    }
                }
            }
        }
        "right" => {
            for drop in &state.drops {
                let head_x = drop.x as i32;
                let head_y = drop.y as i32;
                let length = drop.length;
                let head_md = if drop.dimmed {
                    Modifier::None
                } else {
                    Modifier::Bold
                };
                let body_md = if drop.dimmed {
                    Modifier::Dim
                } else {
                    Modifier::None
                };

                for dy in 0..length {
                    let sx = head_x - dy as i32;
                    if sx >= 0 && sx < b_width && head_y >= 0 && head_y < b_height {
                        let target_idx = (head_y as usize) * (b_width as usize) + (sx as usize);
                        let (fg, md) = if dy == 0 {
                            (head_color, head_md)
                        } else {
                            (body_grad.color_at(dy, length), body_md)
                        };
                        let cell = &mut b.grid[target_idx];
                        cell.c = drop.chars[dy];
                        cell.s.fg = fg;
                        cell.s.md = md;
                        cell.ext_len = 0;
                        cell.width = 1;
                    }
                }
            }
        }
        _ => {
            for drop in &state.drops {
                let head_x = drop.x as i32;
                let head_y = drop.y as i32;
                let length = drop.length;
                let head_md = if drop.dimmed {
                    Modifier::None
                } else {
                    Modifier::Bold
                };
                let body_md = if drop.dimmed {
                    Modifier::Dim
                } else {
                    Modifier::None
                };

                for dy in 0..length {
                    let sy = head_y - dy as i32;
                    if head_x >= 0 && head_x < b_width && sy >= 0 && sy < b_height {
                        let target_idx = (sy as usize) * (b_width as usize) + (head_x as usize);
                        let (fg, md) = if dy == 0 {
                            (head_color, head_md)
                        } else {
                            (body_grad.color_at(dy, length), body_md)
                        };
                        let cell = &mut b.grid[target_idx];
                        cell.c = drop.chars[dy];
                        cell.s.fg = fg;
                        cell.s.md = md;
                        cell.ext_len = 0;
                        cell.width = 1;
                    }
                }
            }
        }
    }
}
