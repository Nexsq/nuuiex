pub const TABS_W: u16 = 3;
pub const LIST_W: u16 = 24;
pub const TITLE_H: u16 = 3;
pub const DECK_H: u16 = 3;

pub fn get_positions(
    _term_w: u16,
    _term_h: u16,
) -> ((i16, i16), (i16, i16), (i16, i16), (i16, i16), (i16, i16)) {
    let main_x = (TABS_W + LIST_W) as i16;
    let main_y = DECK_H as i16;
    let list_x = TABS_W as i16;
    let list_y = TITLE_H as i16;
    let tabs_x = 0;
    let tabs_y = TITLE_H as i16;
    let title_x = 0;
    let title_y = 0;
    let deck_x = (TABS_W + LIST_W) as i16;
    let deck_y = 0;

    (
        (main_x, main_y),
        (list_x, list_y),
        (tabs_x, tabs_y),
        (title_x, title_y),
        (deck_x, deck_y),
    )
}
