use std::time::Duration;

use nuui::Canvas;
use nuui::themecore;
use nuui::{Key, Terminal};
use nuui::{conf, lib};
use nuui::{error, main, settings, toosmall};

fn main() {
    let terminal = Terminal::init();
    let (mut term_w, mut term_h) = Terminal::size();
    let mut canvas = Canvas::new(term_w, term_h);

    let min_w = 64;
    let min_h = 16;

    let mut config = match conf::init() {
        Ok(c) => c,
        Err(e) => {
            let msg = format!("Configuration Error:\n{}\n\nWhat would you like to do?", e);
            let res = error::error_box(
                &terminal,
                &mut canvas,
                &msg,
                &["EXIT", "RESET CONFIG"],
                min_w,
                min_h,
            );
            if res == nuui::PanelResult::Ok(1) {
                if let Err(err) = conf::reset_to_default() {
                    error::error_box(
                        &terminal,
                        &mut canvas,
                        &format!("Failed to reset config:\n{}", err),
                        &["EXIT"],
                        min_w,
                        min_h,
                    );
                    return;
                }
                conf::init().unwrap()
            } else {
                return;
            }
        }
    };

    let theme = match themecore::init(&config.theme) {
        Ok(t) => t,
        Err(e) => {
            let msg = format!("Theme Warning:\n{}\n\nWhat would you like to do?", e);
            let res = error::warning_box(
                &terminal,
                &mut canvas,
                &msg,
                &["EXIT", "RESET TO DEFAULT THEME"],
                0,
                0,
                min_w,
                min_h,
                |_, _, _| {},
            );
            if res == nuui::PanelResult::Ok(1) {
                config.theme = "default".to_string();
                config.save();
                themecore::init("default").unwrap_or_default()
            } else {
                return;
            }
        }
    };

    let mut library = match lib::init() {
        Ok(l) => l,
        Err(e) => {
            error::error_box(
                &terminal,
                &mut canvas,
                &format!("Library Error:\n{}\n\nCannot proceed.", e),
                &["EXIT"],
                min_w,
                min_h,
            );
            return;
        }
    };

    let mut main_view = main::MainView::new(
        term_w,
        term_h,
        main::ActivePanel::List,
        library.tree.clone(),
        library.root_path.clone(),
        &config,
        theme,
    );

    let mut dirty = true;

    loop {
        let (current_w, current_h) = Terminal::size();

        if current_w != term_w || current_h != term_h {
            canvas.resize(current_w, current_h);
            term_w = current_w;
            term_h = current_h;

            if term_w >= main_view.min_w && term_h >= main_view.min_h {
                main_view.resize(term_w, term_h, &config);
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
                if main_view.active == main::ActivePanel::Main && main_view.editor.is_editing {
                    if key == Key::Char('\x03') {
                        break;
                    }
                    if main_view.editor.mode == nuui::editor::Mode::Command && key == Key::Char('q')
                    {
                        break;
                    }
                    if main_view.editor.mode == nuui::editor::Mode::Command && (key == Key::Tab) {
                        main_view.toggle_focus(&config);
                        dirty = true;
                        continue;
                    }
                    if main_view.editor.mode == nuui::editor::Mode::Insert && key == Key::Tab {
                        main_view.editor.handle_key(key, &config);
                        main_view.refresh_main(&config);
                        dirty = true;
                        continue;
                    }

                    main_view.editor.handle_key(key, &config);
                    main_view.refresh_main(&config);
                    dirty = true;
                    continue;
                }

                match key {
                    Key::Char('q') | Key::Char('\x03') => break,

                    Key::Tab => main_view.toggle_focus(&config),

                    Key::Esc => {
                        let should_quit = settings::settings_modal(
                            &terminal,
                            &mut canvas,
                            &mut config,
                            &mut main_view,
                        );

                        if should_quit {
                            break;
                        }
                    }

                    Key::Up => main_view.selection_up(&config),
                    Key::Down => main_view.selection_down(&config),
                    Key::Right => main_view.handle_right_arrow(&config),
                    Key::Left => main_view.handle_left_arrow(&config),
                    Key::Enter => main_view.trigger_selected(&config),

                    Key::Char('r') => {
                        library = match lib::init() {
                            Ok(l) => l,
                            Err(e) => {
                                error::error_box(
                                    &terminal,
                                    &mut canvas,
                                    &format!("Library Error:\n{}\n\nCannot proceed.", e),
                                    &["EXIT"],
                                    min_w,
                                    min_h,
                                );
                                break;
                            }
                        };
                        main_view.library_tree = library.tree.clone();
                        main_view.library_root = library.root_path.clone();
                        main_view.expanded_path.clear();
                        main_view.list_selected = 0;
                        main_view.list_scroll = 0;
                        main_view.auto_load();
                        main_view.refresh_main(&config);
                        main_view.refresh_list(&config);
                    }

                    Key::Char('t') => {
                        let result = error::warning_box(
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
                                        main_view.resize(term_w, term_h, &config);
                                    }
                                } else {
                                    if term_w >= main_view.min_w && term_h >= main_view.min_h {
                                        main_view.refresh_all(&config);
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
                    Key::Shift('t') => {
                        let result = error::error_box(
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
