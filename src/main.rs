use std::thread;
use std::time::Duration;

use nuui::{Buffer, Canvas, Cell};
use nuui::{Color, Modifier};
use nuui::{Key, Terminal};

fn main() {
    let terminal = Terminal::init();

    let w = 20;
    let h = 4;
    let cw = 40;
    let ch = 8;

    let mut buffer1 = Buffer::new(w, h);
    let cell1 = Cell {
        s: 'x',
        fg: Color::None,
        bg: Color::None,
        md: Modifier::None,
    };

    buffer1.put_cell(cell1.clone(), 0, 0);
    buffer1.put_cell(cell1.clone(), w - 1, 0);
    buffer1.put_cell(cell1.clone(), 0, h - 1);
    buffer1.put_cell(cell1.clone(), w - 1, h - 1);

    let mut canvas = Canvas::new(cw, ch);
    let mut buffer_x: i16 = 0;

    loop {
        canvas.clean();
        canvas.put_buffer(&buffer1, buffer_x, 0);
        canvas.render();

        match terminal.read_key() {
            Key::Char('q') | Key::Esc => break,
            Key::Char('\x03') => break,
            Key::Right => buffer_x += 1,
            Key::Left => buffer_x -= 1,
            _ => {}
        }

        thread::sleep(Duration::from_millis(16));
    }
}
