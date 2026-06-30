use std::thread;
use std::time::Duration;

use nuui::Canvas;
use nuui::conf;
use nuui::{Key, Terminal};
use nuui::{main, toosmall};

fn main() {
    let _config = conf::init();

    // println!("Test: {}", _config.test);
    // println!("Border test: {}", _config.border_test);
    // println!("Something: {}", _config.something);

    let terminal = Terminal::init();
    let (mut term_w, mut term_h) = Terminal::size();

    let mut main_view = main::MainView::new(
        term_w,
        term_h,
        main::ActivePanel::Main,
        String::new(),
        String::new(),
    );
    let mut canvas = Canvas::new(term_w, term_h);

    loop {
        let (current_w, current_h) = Terminal::size();

        if current_w != canvas.width || current_h != canvas.height {
            canvas.resize(current_w, current_h);
            term_w = current_w;
            term_h = current_h;

            if term_w >= main_view.min_w && term_h >= main_view.min_h {
                main_view = main::MainView::new(
                    term_w,
                    term_h,
                    main_view.active,
                    main_view.main_buffer.clone(),
                    main_view.list_buffer.clone(),
                );
            }
        }

        if term_w < main_view.min_w || term_h < main_view.min_h {
            toosmall::render(&mut canvas, term_w, term_h);
        } else {
            canvas.clean();
            main_view.render(&mut canvas);
            canvas.render();
        }

        match terminal.read_key() {
            Key::Char('q') | Key::Esc | Key::Char('\x03') => break,
            Key::Char('e') => main_view.toggle_focus(),
            Key::Char('f') => main_view.insert_test_text(),
            _ => {}
        }

        thread::sleep(Duration::from_millis(16));
    }
}
