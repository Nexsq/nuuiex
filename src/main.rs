use std::thread;
use std::time::Duration;

use nuui::{Box, Canvas, Cell};
use nuui::{Color, Modifier};
use nuui::{Key, Terminal};

fn main() {
    let terminal = Terminal::init();

    let (cw, ch) = Terminal::size();

    let xw = 40;
    let xh = 8;

    let ow = 30;
    let oh = 6;

    let mut box1 = Box::new(xw, xh, 0);
    let mut box2 = Box::new(ow, oh, 1);
    let cell1 = Cell {
        s: 'x',
        fg: Color::None,
        bg: Color::None,
        md: Modifier::None,
    };
    let cell2 = Cell {
        s: 'o',
        fg: Color::None,
        bg: Color::Red,
        md: Modifier::Underline,
    };
    let text_style = Cell {
        s: ' ',
        fg: Color::Red,
        bg: Color::None,
        md: Modifier::Bold,
    };

    box1.put_cell(cell1.clone(), 0, 0);
    box1.put_cell(cell1.clone(), xw - 1, 0);
    box1.put_cell(cell1.clone(), 0, xh - 1);
    box1.put_cell(cell1.clone(), xw - 1, xh - 1);

    box2.put_cell(cell2.clone(), 0, 0);
    box2.put_cell(cell2.clone(), ow - 1, 0);
    box2.put_cell(cell2.clone(), 0, oh - 1);
    box2.put_cell(cell2.clone(), ow - 1, oh - 1);

    box2.insert_text(
        "Hello World!\n\nLorem ipsum dolor sit amet consectetur adipiscing elit",
        1,
        0,
        false,
        text_style,
    );

    box1.insert_box(&box2, ((xw - ow) / 2) as i16, ((xh - oh) / 2) as i16);

    let mut canvas = Canvas::new(cw, ch);
    let mut box_x: i16 = 0;
    let mut box_y: i16 = 0;

    loop {
        let (current_w, current_h) = Terminal::size();

        if current_w != canvas.width || current_h != canvas.height {
            canvas.resize(current_w, current_h);
        }

        canvas.clean();
        canvas.put_box(&box1, box_x, box_y);
        canvas.render();

        match terminal.read_key() {
            Key::Char('q') | Key::Esc => break,
            Key::Char('\x03') => break,
            Key::Right => box_x += 1,
            Key::Left => box_x -= 1,
            Key::Down => box_y += 1,
            Key::Up => box_y -= 1,
            _ => {}
        }

        thread::sleep(Duration::from_millis(16));
    }
}
