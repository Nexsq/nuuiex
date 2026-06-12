use nuui::style::*; // rem this later (mr obvious)
use nuui::{Canvas, Cell}; // Cell will not be used in main later

fn main() {
    let w = 20;
    let h = 4;

    let mut canvas1 = Canvas::new(w, h);
    let cell1 = Cell {
        s: 'x',
        fg: Color::None,
        bg: Color::None,
        md: Modifier::None,
    };

    canvas1.put_cell(cell1.clone(), 0, 0);
    canvas1.put_cell(cell1.clone(), w - 1, 0);
    canvas1.put_cell(cell1.clone(), 0, h - 1);
    canvas1.put_cell(cell1.clone(), w - 1, h - 1);

    canvas1.render();
}
