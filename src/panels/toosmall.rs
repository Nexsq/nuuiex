use crate::{Border, Box, Canvas, Color, Modifier, Style};

pub fn render(canvas: &mut Canvas, width: u16, height: u16) {
    canvas.clean();

    let mut warning_box = Box::new(
        width,
        height,
        0,
        Border::Light,
        Style {
            fg: Color::Red,
            bg: Color::None,
            md: Modifier::Bold,
        },
    );

    let text_style = Style {
        fg: Color::Yellow,
        bg: Color::None,
        md: Modifier::None,
    };

    let msg = "WINDOW TOO SMALL\n\nExpand your terminal";

    let offset_x = (width.saturating_sub(26) / 2) as i16;
    let offset_y = (height.saturating_sub(3) / 2) as i16;

    warning_box.insert_text(msg, offset_x, offset_y, true, text_style);

    canvas.put_box(&warning_box, 0, 0);
    canvas.render();
}
