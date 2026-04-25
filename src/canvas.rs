use crossterm::{
    cursor::MoveTo,
    style::Print,
    terminal::{Clear, ClearType, size},
    ExecutableCommand, QueueableCommand,
};
use std::io::{stdout, Write};

const UP: u8 = 1;
const DOWN: u8 = 2;
const LEFT: u8 = 4;
const RIGHT: u8 = 8;

pub struct Canvas {
    pub width: u16,
    pub height: u16,
    buffer: Vec<u8>,
    prev_buffer: Vec<u8>,
}

impl Canvas {
    pub fn new() -> Self {
        let (cols, rows) = size().unwrap_or((80, 24));
        let mut stdout = stdout();

        stdout.execute(Clear(ClearType::All)).unwrap();

        let capacity = (cols * rows) as usize;
        Self {
            width: cols,
            height: rows,
            buffer: vec![0; capacity],
            prev_buffer: vec![0; capacity],
        }
    }

    fn apply_mask(&mut self, x: i32, y: i32, mask: u8) {
        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
            self.buffer[(y * self.width as i32 + x) as usize] |= mask;
        }
    }

    fn add_h_line(&mut self, x_start: i32, x_end: i32, y: i32) {
        let start = x_start.min(x_end);
        let end = x_start.max(x_end);
        for x in start..end {
            self.apply_mask(x, y, RIGHT);
            self.apply_mask(x + 1, y, LEFT);
        }
    }

    fn add_v_line(&mut self, x: i32, y_start: i32, y_end: i32) {
        let start = y_start.min(y_end);
        let end = y_start.max(y_end);
        for y in start..end {
            self.apply_mask(x, y, DOWN);
            self.apply_mask(x, y + 1, UP);
        }
    }

    pub fn draw_box(&mut self, pos: (i32, i32), size: (i32, i32), corner: u8) {
        let (x, y) = pos;
        let (w, h) = size;

        if w < 2 || h < 2 { return; }

        let (start_x, start_y) = match corner {
            0 => (x, y),
            1 => (x - w + 1, y),
            2 => (x, y - h + 1),
            _ => (x - w + 1, y - h + 1),
        };

        let end_x = start_x + w - 1;
        let end_y = start_y + h - 1;

        self.add_h_line(start_x, end_x, start_y);
        self.add_h_line(start_x, end_x, end_y);
        self.add_v_line(start_x, start_y, end_y);
        self.add_v_line(end_x, start_y, end_y);
    }

    pub fn clear(&mut self) {
        self.buffer.fill(0);
    }

    pub fn resize(&mut self, cols: u16, rows: u16) {
        self.width = cols;
        self.height = rows;
        let capacity = (cols as usize) * (rows as usize);

        self.buffer = vec![0; capacity];
        self.prev_buffer = vec![0; capacity];

        let mut stdout = stdout();
        stdout.execute(Clear(ClearType::All)).unwrap();
    }

    pub fn render(&mut self) {
        let mut stdout = stdout();
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = (y * self.width + x) as usize;
                let current = self.buffer[idx];

                if current != self.prev_buffer[idx] {
                    let c = match current {
                        1 => '╵', 2 => '╷', 3 => '│', 4 => '╴',
                        5 => '╯', 6 => '╮', 7 => '┤', 8 => '╶',
                        9 => '╰', 10 => '╭', 11 => '├', 12 => '─',
                        13 => '┴', 14 => '┬', 15 => '┼',
                        _ => ' ',
                    };
                    stdout.queue(MoveTo(x, y)).unwrap();
                    stdout.queue(Print(c)).unwrap();
                    self.prev_buffer[idx] = current;
                }
            }
        }
        stdout.flush().unwrap();
    }
}