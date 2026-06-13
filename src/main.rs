use nuui::style::*; // rem this later (mr obvious)
use nuui::{Buffer, Canvas, Cell};

fn main() {
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

    canvas.put_buffer(&buffer1, 0, 0);

    canvas.render();
}
