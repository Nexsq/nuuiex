pub struct Box {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub corner: u8,
}

impl Box {
    pub fn draw_in(&self, canvas: &mut crate::canvas::Canvas) {
        match self.corner {
            0 => {
                for row in 0..self.height {
                    for col in 0..self.width {
                        if row == 0 || row == self.height - 1 || col == 0 || col == self.width - 1 {
                            canvas.put(self.x + col, self.y + row, 'o');
                        }
                    }
                }
            }
            1 => {
                for row in 0..self.height {
                    for col in 0..self.width {
                        if row == 0 || row == self.height - 1 || col == 0 || col == self.width - 1 {
                            canvas.put(self.x + col, self.y + row, 'o');
                        }
                    }
                }
            }
            2 => {
                for row in 0..self.height {
                    for col in 0..self.width {
                        if row == 0 || row == self.height - 1 || col == 0 || col == self.width - 1 {
                            canvas.put(self.x + col, self.y + row, 'o');
                        }
                    }
                }
            }
            _ => {}
        }
    }
}