use std::thread;
use std::time::Duration;

use nuui::Canvas;
use nuui::{Key, Terminal};
use nuui::{conf, lib};
use nuui::{error, main, toosmall};

fn main() {
    let _config = conf::init();
    let mut library = lib::init();

    let terminal = Terminal::init();
    let (mut term_w, mut term_h) = Terminal::size();

    let mut main_view = main::MainView::new(
        term_w,
        term_h,
        main::ActivePanel::Main,
        String::new(),
        library.tree.clone(),
    );
    let mut canvas = Canvas::new(term_w, term_h);

    loop {
        let (current_w, current_h) = Terminal::size();

        if current_w != term_w || current_h != term_h {
            canvas.resize(current_w, current_h);
            term_w = current_w;
            term_h = current_h;

            if term_w >= main_view.min_w && term_h >= main_view.min_h {
                main_view.resize(term_w, term_h);
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
            Key::Char('q' | 'Q') | Key::Esc | Key::Char('\x03') => break,
            Key::Char('e' | 'E') => main_view.toggle_focus(),

            Key::Up => main_view.selection_up(),
            Key::Down => main_view.selection_down(),
            Key::Right | Key::Enter => main_view.trigger_selected(),

            Key::Char('r' | 'R') => {
                library = lib::init();
                main_view.library_tree = library.tree.clone();
                main_view.refresh_list();
            }

            Key::Char('t') => {
                error::error_box(
                    &terminal,
                    &mut canvas,
                    "This is a test warning\n\nDo you want to proceed",
                    &["CANCEL", "CONFIRM"],
                    0,
                    0,
                    main_view.min_w,
                    main_view.min_h,
                    |cvs, w, h| {
                        if w != term_w || h != term_h {
                            term_w = w;
                            term_h = h;
                            if term_w >= main_view.min_w && term_h >= main_view.min_h {
                                main_view.resize(term_w, term_h);
                            }
                        }
                        if term_w < main_view.min_w || term_h < main_view.min_h {
                            toosmall::render(cvs, term_w, term_h);
                        } else {
                            main_view.render(cvs);
                        }
                    },
                );
            }
            Key::Char('T') => {
                error::error_screen(
                    &terminal,
                    &mut canvas,
                    "This is a test warning\n\nDo you want to proceed",
                    &["CANCEL", "CONFIRM"],
                    main_view.min_w,
                    main_view.min_h,
                );
            }
            _ => {}
        }

        thread::sleep(Duration::from_millis(2));
    }
}
