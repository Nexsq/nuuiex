use crate::{Border, Box, Canvas, Color, Key, Modifier, Style, Terminal};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Choice {
    Regenerate,
    Exit,
}

pub fn render_and_handle(terminal: &Terminal, err_msg: &str) -> Choice {
    let (mut width, mut height) = Terminal::size();
    let mut canvas = Canvas::new(width, height);
    let mut choice = Choice::Regenerate;

    loop {
        let (current_w, current_h) = Terminal::size();
        if current_w != canvas.width || current_h != canvas.height {
            canvas.resize(current_w, current_h);
            width = current_w;
            height = current_h;
        }

        canvas.clean();

        let mut error_box = Box::new(
            width,
            height,
            0,
            Border::Double,
            Style {
                fg: Color::Red,
                bg: Color::None,
                md: Modifier::Bold,
            },
        );

        error_box.insert_text(
            " CONFIG ERROR ",
            4,
            1,
            false,
            Style {
                fg: Color::Black,
                bg: Color::Red,
                md: Modifier::Bold,
            },
        );

        error_box.insert_text(
            err_msg,
            4,
            3,
            true,
            Style {
                fg: Color::White,
                bg: Color::None,
                md: Modifier::None,
            },
        );

        let hint_y = height.saturating_sub(5) as i16;
        error_box.insert_text(
            "Use Left/Right Arrows to navigate, Enter to select.",
            4,
            hint_y,
            true,
            Style {
                fg: Color::DarkGray,
                bg: Color::None,
                md: Modifier::Dim,
            },
        );

        let btn_y = height.saturating_sub(3) as i16;

        let reg_style = if choice == Choice::Regenerate {
            Style {
                fg: Color::Black,
                bg: Color::Yellow,
                md: Modifier::Bold,
            }
        } else {
            Style {
                fg: Color::Yellow,
                bg: Color::None,
                md: Modifier::None,
            }
        };

        let exit_style = if choice == Choice::Exit {
            Style {
                fg: Color::Black,
                bg: Color::Red,
                md: Modifier::Bold,
            }
        } else {
            Style {
                fg: Color::Red,
                bg: Color::None,
                md: Modifier::None,
            }
        };

        let start_x = (width.saturating_sub(26) / 2) as i16;

        error_box.insert_text("[ Regenerate ]", start_x, btn_y, false, reg_style);
        error_box.insert_text("[ Exit ]", start_x + 18, btn_y, false, exit_style);

        canvas.put_box(&error_box, 0, 0);
        canvas.render();

        match terminal.read_key() {
            Key::Left | Key::Char('h') | Key::Char('a') => {
                choice = Choice::Regenerate;
            }
            Key::Right | Key::Char('l') | Key::Char('d') => {
                choice = Choice::Exit;
            }
            Key::Enter => return choice,
            Key::Char('q') | Key::Esc => return Choice::Exit,
            _ => {}
        }

        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}
