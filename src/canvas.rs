use crossterm::{cursor, queue, style::Print};

pub struct Canvas {
    pub width: u16,
    pub height: u16,
    grid: Vec<Vec<char>>,
}

impl Canvas {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            grid: vec![vec![' '; width as usize]; height as usize],
        }
    }

    pub fn put(&mut self, x: u16, y: u16, c: char) {
        if x < self.width && y < self.height {
            self.grid[y as usize][x as usize] = c;
        }
    }

    pub fn clear(&mut self) {
        for row in &mut self.grid {
            row.fill(' ');
        }
    }

    pub fn render(&self, stdout: &mut std::io::Stdout) -> std::io::Result<()> {
        for (y, row) in self.grid.iter().enumerate() {
            let line: String = row.iter().collect();
            queue!(stdout, cursor::MoveTo(0, y as u16), Print(line))?;
        }
        Ok(())
    }
}