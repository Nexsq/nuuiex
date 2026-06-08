use nuui::Cell;
use nuui::style::*; // rem this later (mr obvious)

fn main() {
    let cell1: Cell = Cell {
        s: String::from("big aah test"),
        fg: Color::White,
        bg: Color::Black,
        md: Modifier::None,
    };

    cell1.print();
}