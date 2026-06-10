use nuui::style::*;
use nuui::{Canvas, Cell}; // Cell will not be used in main later // rem this later (mr obvious)

fn main() {
    let mut canvas1 = Canvas::new(4, 2);
    let cell1 = Cell {
        s: 'x',
        fg: Color::White,
        bg: Color::Black,
        md: Modifier::None,
    };
    canvas1.put_cell(cell1, 0, 0);

    canvas1.render();
}
