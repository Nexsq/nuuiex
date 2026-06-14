use std::thread;
use std::time::Duration;

use nuui::{Buffer, Canvas, Cell};
use nuui::{Color, Modifier};
use nuui::{Key, Terminal};

fn main() {
    let terminal = Terminal::init();

    let (cw, ch) = Terminal::size();

    let bw = 40;
    let bh = 8;

    let mut buffer1 = Buffer::new(bw, bh);
    let cell1 = Cell {
        s: 'x',
        fg: Color::None,
        bg: Color::None,
        md: Modifier::None,
    };

    buffer1.put_cell(cell1.clone(), 0, 0);
    buffer1.put_cell(cell1.clone(), bw - 1, 0);
    buffer1.put_cell(cell1.clone(), 0, bh - 1);
    buffer1.put_cell(cell1.clone(), bw - 1, bh - 1);

    let mut canvas = Canvas::new(cw, ch);
    let mut buffer_x: i16 = 0;
    let mut buffer_y: i16 = 0;

    loop {
        canvas.clean();
        canvas.put_buffer(&buffer1, buffer_x, buffer_y);
        canvas.render();

        match terminal.read_key() {
            Key::Char('q') | Key::Esc => break,
            Key::Char('\x03') => break,
            Key::Right => buffer_x += 1,
            Key::Left => buffer_x -= 1,
            Key::Down => buffer_y += 1,
            Key::Up => buffer_y -= 1,
            _ => {}
        }

        thread::sleep(Duration::from_millis(16));
    }
}
