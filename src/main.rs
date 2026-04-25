mod canvas;
use canvas::Canvas;
use crossterm::{
    cursor::MoveTo,
    event::{poll, read, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
    ExecutableCommand,
};
use std::{io::stdout, time::Duration};

fn main() {
    enable_raw_mode().unwrap();

    let mut canvas = Canvas::new();

    loop {
        canvas.clear();
        canvas.draw_box((10, 5), (20, 10), 1);
        canvas.draw_box((20, 8), (15, 8), 1);
        canvas.render();

        if poll(Duration::from_millis(50)).unwrap() {
            match read().unwrap() {
                Event::Resize(cols, rows) => {
                    canvas.resize(cols, rows);
                }
                Event::Key(key_event) => {
                    match key_event.code {
                        KeyCode::Esc => break,
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    disable_raw_mode().unwrap();
    let mut stdout = stdout();
    stdout.execute(MoveTo(0, canvas.height)).unwrap();
}