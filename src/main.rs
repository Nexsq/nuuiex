use std::io::{self, stdout, Write};
use std::time::Duration;
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute, cursor,
};

use nuui::canvas::Canvas;
use nuui::widgets::Box;

fn main() -> io::Result<()> {
    let mut stdout = stdout();
    enable_raw_mode()?; 
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;

    let (width, height) = crossterm::terminal::size()?;
    let mut canvas = Canvas::new(width, height);

    let box1 = Box { x: 5, y: 2, width: 20, height: 10, corner: 0 };
    let box2 = Box { x: 30, y: 5, width: 15, height: 8, corner: 0 };

    loop {
        canvas.clear();

        box1.draw_in(&mut canvas);
        box2.draw_in(&mut canvas);

        canvas.render(&mut stdout)?;
        stdout.flush()?;

        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    execute!(stdout, LeaveAlternateScreen, cursor::Show)?;
    disable_raw_mode()?;
    Ok(())
}