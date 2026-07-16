use crate::{Box, Cell, Key, Modifier, Style, conf::Config, theme::themecore::Theme};

const NUM_BARS: usize = 256;
const SUBCHARS: [char; 9] = [
    ' ', '\u{2581}', '\u{2582}', '\u{2583}', '\u{2584}', '\u{2585}', '\u{2586}', '\u{2587}',
    '\u{2588}',
];

pub struct KeyvisState {
    pub heights: [f32; NUM_BARS],
    pub velocities: [f32; NUM_BARS],
    pub rand_seed: u32,
}

impl KeyvisState {
    pub fn new() -> Self {
        Self {
            heights: [0.0; NUM_BARS],
            velocities: [0.0; NUM_BARS],
            rand_seed: 1337,
        }
    }

    fn next_rand(&mut self) -> f32 {
        self.rand_seed = self
            .rand_seed
            .wrapping_mul(1664525)
            .wrapping_add(1013904223);
        (self.rand_seed >> 8) as f32 / 16777216.0
    }

    pub fn push_key(&mut self, key: &Key, config_force: f32, config_spread: usize) {
        let k_val = match key {
            Key::Char(c) | Key::Shift(c) | Key::Ctrl(c) => (*c as u32) as usize,
            Key::Enter => 13,
            Key::Backspace | Key::CtrlBackspace => 8,
            Key::Esc => 27,
            Key::Up => 200,
            Key::Down => 201,
            Key::Left => 202,
            Key::Right => 203,
            Key::Tab => 9,
            Key::Delete | Key::CtrlDelete => 127,
            _ => 0,
        };

        if k_val == 0 {
            return;
        }

        self.rand_seed = self.rand_seed.wrapping_add(k_val as u32);

        let center = (k_val * 11) % NUM_BARS;
        let force = 1.0 + config_force + self.next_rand() * 0.4;

        let spread = config_spread.clamp(2, 32);
        for i in (center.saturating_sub(spread))..=(center + spread).min(NUM_BARS - 1) {
            let dist = (center as i32 - i as i32).abs() as f32;
            let t = dist / (spread as f32 + 1.0);

            let intensity = (1.0 - t * t).max(0.0);
            self.velocities[i] += force * intensity;
        }
    }

    pub fn tick(&mut self, gravity: f32, steps: usize, tension: f32) -> bool {
        let stiffness = (gravity * 0.08).max(0.008);
        let damping = 0.88;

        let sim_tension = tension;

        for _ in 0..steps {
            let lap_left = self.heights[1] - self.heights[0];
            self.velocities[0] = (self.velocities[0] + lap_left * sim_tension
                - self.heights[0] * stiffness)
                * damping;

            for i in 1..(NUM_BARS - 1) {
                let lap = self.heights[i - 1] + self.heights[i + 1] - 2.0 * self.heights[i];
                self.velocities[i] = (self.velocities[i] + lap * sim_tension
                    - self.heights[i] * stiffness)
                    * damping;
            }

            let lap_right = self.heights[NUM_BARS - 2] - self.heights[NUM_BARS - 1];
            self.velocities[NUM_BARS - 1] = (self.velocities[NUM_BARS - 1]
                + lap_right * sim_tension
                - self.heights[NUM_BARS - 1] * stiffness)
                * damping;

            for i in 0..NUM_BARS {
                self.heights[i] += self.velocities[i];
            }

            let mut prev_h = self.heights[0];
            self.heights[0] = prev_h * 0.82 + (prev_h + self.heights[1]) * 0.09;

            for i in 1..(NUM_BARS - 1) {
                let curr_h = self.heights[i];
                self.heights[i] = curr_h * 0.82 + (prev_h + self.heights[i + 1]) * 0.09;
                prev_h = curr_h;
            }

            let last_h = self.heights[NUM_BARS - 1];
            self.heights[NUM_BARS - 1] = last_h * 0.82 + (prev_h + last_h) * 0.09;
        }

        let mut max_energy = 0.0_f32;
        for i in 0..NUM_BARS {
            let energy = self.heights[i].abs() + self.velocities[i].abs();
            if energy > max_energy {
                max_energy = energy;
            }
        }

        if max_energy > 0.005 {
            true
        } else {
            self.heights.fill(0.0);
            self.velocities.fill(0.0);
            false
        }
    }
}

pub fn draw(state: &KeyvisState, b: &mut Box, config: &Config, theme: &Theme) {
    let w = b.width as usize;
    let h = b.height as usize;

    if w == 0 || h == 0 {
        return;
    }

    let gradient = theme.keyview_color.clone();

    let bar_width = config.keyvis_width.max(1);
    let visible_bars = (w.saturating_add(bar_width - 1)) / bar_width;
    let visible_bars = visible_bars.max(1);

    let scale_factor = (h as f32) / 6.0;

    for i in 0..visible_bars {
        let start_phys = ((i * NUM_BARS) / visible_bars).min(NUM_BARS - 1);
        let mut end_phys = (((i + 1) * NUM_BARS) / visible_bars).min(NUM_BARS);
        end_phys = end_phys.max(start_phys + 1).min(NUM_BARS);

        let mut max_h = 0.0_f32;
        let mut sum_h = 0.0_f32;
        let count = (end_phys - start_phys) as f32;

        for p in start_phys..end_phys {
            let h_val = state.heights[p].max(0.0);
            if h_val > max_h {
                max_h = h_val;
            }
            sum_h += h_val;
        }

        let avg_h = sum_h / count;
        let blended_h = max_h * 0.7 + avg_h * 0.3;
        let mut bar_h = blended_h * scale_factor;
        let color = gradient.color_at(i, visible_bars);

        for y in (0..h as u16).rev() {
            let cell_h = bar_h.min(1.0).max(0.0);
            bar_h -= 1.0;

            let ch = if cell_h <= 0.01 {
                if y == h as u16 - 1 && config.keyvis_base {
                    SUBCHARS[1]
                } else {
                    break;
                }
            } else if cell_h >= 0.99 {
                SUBCHARS[8]
            } else {
                let idx = (cell_h * 8.0).round() as usize;
                SUBCHARS[idx.clamp(1, 8)]
            };

            if ch != ' ' {
                for dw in 0..bar_width {
                    let x = (i * bar_width + dw) as u16;
                    if x < w as u16 {
                        let target_idx = (y as usize) * w + (x as usize);
                        let original_bg = b.grid[target_idx].s.bg;

                        let style = Style {
                            fg: color,
                            bg: original_bg,
                            md: Modifier::None,
                        };
                        b.grid[target_idx] = Cell { c: ch, s: style };
                    }
                }
            }
        }
    }
}
