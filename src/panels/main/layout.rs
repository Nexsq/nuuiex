pub const TABS_W: u16 = 3;

pub fn get_positions(
    _term_w: u16,
    _term_h: u16,
    header_h: u16,
    deck_h: u16,
    tabs_num: usize,
    deck: bool,
    list_w: u16,
) -> ((i16, i16), (i16, i16), (i16, i16), (i16, i16), (i16, i16)) {
    let tabs_x = 0;
    let tabs_y = header_h as i16;
    let list_x = if tabs_num == 1 {
        0
    } else {
        (TABS_W - 1) as i16
    };
    let list_y = header_h as i16;
    let main_x = (TABS_W + list_w - 1) as i16;
    let main_y = if !deck { 0 } else { deck_h as i16 };
    let title_x = 0;
    let title_y = 0;
    let deck_x = (TABS_W + list_w - 1) as i16;
    let deck_y = 0;

    (
        (main_x, main_y),
        (list_x, list_y),
        (tabs_x, tabs_y),
        (title_x, title_y),
        (deck_x, deck_y),
    )
}
