use std::time::Duration;

use nuui::Canvas;
use nuui::{Key, Terminal};
use nuui::{conf, lib};
use nuui::{error, main, settings, toosmall};

fn main() {
    let mut config = conf::init();
    let mut library = lib::init();

    // println!("Test: {}", config.test);
    // println!("Border test: {}", config.border_test);
    // println!("Something: {}", config.something);
    // println!("Primary color: {}", config.primary_color);

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

    let mut dirty = true;

    loop {
        let (current_w, current_h) = Terminal::size();

        if current_w != term_w || current_h != term_h {
            canvas.resize(current_w, current_h);
            term_w = current_w;
            term_h = current_h;

            if term_w >= main_view.min_w && term_h >= main_view.min_h {
                main_view.resize(term_w, term_h);
            }
            dirty = true;
        }

        if dirty {
            if term_w < main_view.min_w || term_h < main_view.min_h {
                toosmall::render(&mut canvas, term_w, term_h);
            } else {
                canvas.clean();
                main_view.render(&mut canvas);
                canvas.render();
            }
            dirty = false;
        }

        match terminal.read_key(Duration::from_millis(16)) {
            Key::None => continue,
            key => {
                match key {
                    Key::Char('q' | 'Q') | Key::Char('\x03') => break,

                    Key::Esc => {
                        let should_quit = settings::settings_modal(
                            &terminal,
                            &mut canvas,
                            &mut config,
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

                        if should_quit {
                            break;
                        }
                    }

                    Key::Char('e' | 'E') => main_view.toggle_focus(),

                    Key::Up => main_view.selection_up(),
                    Key::Down => main_view.selection_down(),
                    Key::Right => main_view.handle_right_arrow(),
                    Key::Left => main_view.handle_left_arrow(),
                    Key::Enter => main_view.trigger_selected(),

                    Key::Char('r' | 'R') => {
                        library = lib::init();
                        main_view.library_tree = library.tree.clone();
                        main_view.expanded_path.clear();
                        main_view.list_selected = 0;
                        main_view.list_scroll = 0;
                        main_view.refresh_list();
                    }

                    Key::Char('t') => {
                        let result = error::error_box(
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

                        if result == nuui::PanelResult::Quit {
                            break;
                        }
                    }
                    Key::Char('T') => {
                        let result = error::error_screen(
                            &terminal,
                            &mut canvas,
                            "This is a test warning\n\nDo you want to proceed",
                            &["CANCEL", "CONFIRM"],
                            main_view.min_w,
                            main_view.min_h,
                        );

                        if result == nuui::PanelResult::Quit {
                            break;
                        }
                    }
                    _ => continue,
                }

                dirty = true;
            }
        }
    }
}
