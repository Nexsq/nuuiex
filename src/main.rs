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
            let msg = format!("Configuration Error:\n{}\n\nReset config?", e);
            let res = error::error_box(
                &terminal,
                &mut canvas,
                &msg,
                &["EXIT", "RESET CONFIG"],
                min_w,
                min_h,
                nuui::Border::Heavy,
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
                        nuui::Border::Heavy,
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
            let msg = format!("Theme Error:\n{}\n\nReset to default theme?", e);
            let res = error::error_box(
                &terminal,
                &mut canvas,
                &msg,
                &["EXIT", "RESET TO DEFAULT THEME"],
                0,
                0,
                config.get_border(),
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

    let library = match lib::init(&config.lib_sorting) {
        Ok(l) => l,
        Err(e) => {
            error::error_box(
                &terminal,
                &mut canvas,
                &format!("Library Error:\n{}\n\nCannot proceed", e),
                &["EXIT"],
                min_w,
                min_h,
                config.get_border(),
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
                if !toosmall::run(
                    &terminal,
                    &mut canvas,
                    main_view.min_w,
                    main_view.min_h,
                    config.get_border(),
                ) {
                    break;
                }
                dirty = true;
                continue;
            }

            canvas.clean();
            main_view.render(&mut canvas);
            canvas.render();
            dirty = false;
        }

        match terminal.read_key(Duration::from_millis(16)) {
            Key::None => continue,
            key => {
                if main_view.active == main::ActivePanel::Main
                    && main_view.editors[main_view.current_tab].is_editing
                {
                    if key == Key::Char('\x03') {
                        break;
                    }
                    if main_view.editors[main_view.current_tab].mode == nuui::editor::Mode::Command
                        && key == Key::Char('q')
                    {
                        break;
                    }
                    if main_view.editors[main_view.current_tab].mode == nuui::editor::Mode::Command
                        && (key == Key::Tab)
                    {
                        main_view.toggle_focus(&config);
                        dirty = true;
                        continue;
                    }
                    if main_view.editors[main_view.current_tab].mode == nuui::editor::Mode::Insert
                        && key == Key::Tab
                    {
                        main_view.editors[main_view.current_tab].handle_key(key, &config);
                        main_view.refresh_main(&config);
                        dirty = true;
                        continue;
                    }

                    main_view.editors[main_view.current_tab].handle_key(key, &config);
                    main_view.refresh_main(&config);
                    dirty = true;
                    continue;
                }

                if main_view.list_input != main::ListInputMode::None {
                    match main::handle_list_input(
                        &mut main_view,
                        &key,
                        &terminal,
                        &mut canvas,
                        &config,
                    ) {
                        Ok(true) => {
                            dirty = true;
                            continue;
                        }
                        Ok(false) => {}
                        Err(e) => {
                            error::error_box(
                                &terminal,
                                &mut canvas,
                                &format!("Library Error:\n{}\n\nCannot proceed", e),
                                &["EXIT"],
                                min_w,
                                min_h,
                                config.get_border(),
                            );
                            break;
                        }
                    }
                }

                if main_view.active == main::ActivePanel::List {
                    if let Key::Char(c) = key {
                        if c >= '1'
                            && c <= std::char::from_digit(config.tabs_num.clamp(2, 6) as u32, 10)
                                .unwrap()
                        {
                            let tab_idx = (c as u8 - b'1') as usize;
                            main_view.switch_tab(tab_idx, &config);
                            dirty = true;
                            continue;
                        }
                    }

                    match main::handle_list_action(
                        &mut main_view,
                        &key,
                        &terminal,
                        &mut canvas,
                        &config,
                    ) {
                        Ok(true) => {
                            dirty = true;
                            continue;
                        }
                        Ok(false) => {}
                        Err(e) => {
                            error::error_box(
                                &terminal,
                                &mut canvas,
                                &format!("Library Error:\n{}\n\nCannot proceed", e),
                                &["EXIT"],
                                min_w,
                                min_h,
                                config.get_border(),
                            );
                            break;
                        }
                    }
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

                        if let Ok(l) = lib::init(&config.lib_sorting) {
                            main_view.library_tree = l.tree;
                            main_view.library_root = l.root_path;
                        }
                        main_view.auto_load();
                        main_view.refresh_all(&config);

                        if should_quit {
                            break;
                        }
                    }

                    Key::Up => main_view.selection_up(&config),
                    Key::Down => main_view.selection_down(&config),
                    Key::Right => main_view.handle_right_arrow(&config),
                    Key::Left => main_view.handle_left_arrow(&config),
                    Key::Enter => main_view.trigger_selected(&config),

                    _ => continue,
                }

                dirty = true;
            }
        }
    }
}
