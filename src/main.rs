use nuui::{Cell, Canvas}; // Cell will not be used in main later
use nuui::style::*; // rem this later (mr obvious)

fn main() {
    let mut canvas1 = Canvas::new(2, 4);

    let cell1: Cell = Cell {
        s: String::from("big aah test "),
        fg: Color::White,
        bg: Color::Black,
        md: Modifier::None,
    };

    let cell2: Cell = Cell {
        s: String::from("and cell 2"),
        fg: Color::White,
        bg: Color::Black,
        md: Modifier::None,
    };

    canvas1.add_cell(cell1);
    canvas1.add_cell(cell2);
    canvas1.print();
}