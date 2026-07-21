use std::time::Duration;

use nuui::Canvas;
use nuui::themecore;
use nuui::{Gradient, Key, Terminal};
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
                Gradient::Solid(nuui::Color::BrightYellow),
                |_, _, _, _| false,
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
                        Gradient::Solid(nuui::Color::BrightYellow),
                        |_, _, _, _| false,
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
                Gradient::Solid(nuui::Color::BrightYellow),
                |_, _, _, _| false,
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
                theme.warning_color.clone(),
                |_, _, _, _| false,
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
    let mut q_pressed_once = false;

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

        let mut tabs_updated = [false; 6];
        for i in 0..6 {
            if let Some(rx) = &main_view.editors[i].process_rx {
                let mut disconnected = false;
                let mut processed = 0;
                loop {
                    match rx.try_recv() {
                        Ok(nuui::EngineMessage::Output(lines, caret_x, caret_y)) => {
                            main_view.editors[i].state.lines = lines;
                            main_view.editors[i].state.cursor_x = caret_x;
                            main_view.editors[i].state.cursor_y = caret_y;
                            main_view.editors[i].error_count = 0;
                            main_view.editors[i].error_lines.clear();
                            tabs_updated[i] = true;
                            processed += 1;
                            if processed > 500 {
                                break;
                            }
                        }
                        Ok(nuui::EngineMessage::InputRequest) => {
                            main_view.editors[i].is_waiting_for_input = true;
                            main_view.editors[i].input_buffer.clear();
                            tabs_updated[i] = true;
                            break;
                        }
                        Err(std::sync::mpsc::TryRecvError::Empty) => break,
                        Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                            disconnected = true;
                            break;
                        }
                    }
                }
                if disconnected {
                    main_view.editors[i].process_rx = None;
                    main_view.editors[i].process_input_tx = None;
                    main_view.editors[i].is_waiting_for_input = false;
                    main_view.running_macros[i] = None;
                    main_view.macro_focus_tokens[i] = None;
                    tabs_updated[i] = true;
                }
            }
        }

        let mut any_tab_updated = false;
        for i in 0..6 {
            if tabs_updated[i] {
                any_tab_updated = true;
                if main_view.current_tab == i {
                    main_view.refresh_main(&config);
                }
            }
        }
        if any_tab_updated {
            main_view.refresh_static_boxes(&config);
            dirty = true;
        }

        if dirty {
            if term_w < main_view.min_w || term_h < main_view.min_h {
                main_view.update_macro_focus(false);
                let small_res = toosmall::run(
                    &terminal,
                    &mut canvas,
                    main_view.min_w,
                    main_view.min_h,
                    config.get_border(),
                    main_view.theme.warning_color.clone(),
                );
                main_view.update_macro_focus(true);
                if !small_res {
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

        let key = terminal.read_key(Duration::from_millis(16));

        if key != Key::None && key != Key::Char('q') {
            q_pressed_once = false;
        }

        main_view.update_macro_focus(true);

        let mut anim_dirty = false;
        if config.deck_mode == "widget" {
            if config.deck_widget == "keyvis" {
                if key != Key::None {
                    main_view
                        .keyvis
                        .push_key(&key, config.keyvis_force, config.keyvis_spread);
                }
                if main_view.keyvis.tick(
                    config.keyvis_gravity,
                    config.keyvis_steps,
                    config.keyvis_tension,
                ) {
                    main_view.refresh_static_boxes(&config);
                    anim_dirty = true;
                }
            } else if config.deck_widget == "monitor" {
                if main_view.monitor.tick(term_w, term_h) {
                    main_view.refresh_static_boxes(&config);
                    anim_dirty = true;
                }
            } else if config.deck_widget == "clock" {
                if main_view.clock.tick(term_w, term_h, &config) {
                    main_view.refresh_static_boxes(&config);
                    anim_dirty = true;
                }
            }
        }

        for i in 0..6 {
            if !main_view.editors[i].is_editing && main_view.editors[i].is_waiting_for_input {
                let time = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis();
                let blink = time % 1000 < 500;
                if blink != main_view.editors[i].last_blink_state {
                    main_view.editors[i].last_blink_state = blink;
                    if main_view.current_tab == i {
                        main_view.refresh_main(&config);
                        anim_dirty = true;
                    }
                }
            }
        }

        if anim_dirty {
            dirty = true;
        }

        match key {
            Key::None => continue,
            key => {
                if main_view.active == main::ActivePanel::Main {
                    let editor = &mut main_view.editors[main_view.current_tab];

                    if editor.is_waiting_for_input && !editor.is_editing {
                        if !matches!(
                            key,
                            Key::Tab | Key::Esc | Key::Up | Key::Down | Key::Left | Key::Right
                        ) {
                            match key {
                                Key::Char('\x03') => {
                                    if let Some(token) =
                                        main_view.cancellation_tokens[main_view.current_tab].take()
                                    {
                                        token.store(true, std::sync::atomic::Ordering::SeqCst);
                                    }
                                }
                                Key::Enter => {
                                    if let Some(tx) = &editor.process_input_tx {
                                        let _ = tx.send(editor.input_buffer.clone());
                                    }
                                    editor.is_waiting_for_input = false;
                                    editor.input_buffer.clear();
                                }
                                Key::Backspace | Key::CtrlBackspace => {
                                    editor.input_buffer.pop();
                                }
                                Key::Char(c) if !c.is_control() => {
                                    editor.input_buffer.push(c);
                                }
                                Key::Shift(c) if !c.is_control() => {
                                    editor.input_buffer.push(c.to_ascii_uppercase());
                                }
                                _ => {}
                            }

                            main_view.refresh_main(&config);
                            dirty = true;
                            continue;
                        }
                    }

                    if editor.is_editing {
                        if key == Key::Char('\x03') {
                            break;
                        }
                        if editor.mode == nuui::editor::Mode::Command && key == Key::Char('q') {
                            if config.double_q_exit && !q_pressed_once {
                                q_pressed_once = true;
                                continue;
                            }
                            break;
                        }

                        let is_ignorable_esc = editor.mode == nuui::editor::Mode::Command
                            && key == Key::Esc
                            && !editor.visual_mode
                            && editor.state.selection_start.is_none();
                        if !is_ignorable_esc {
                            if editor.mode == nuui::editor::Mode::Command
                                && key == Key::Tab
                                && !editor.visual_mode
                            {
                                main_view.toggle_focus(&config);
                                dirty = true;
                                continue;
                            }

                            if editor.mode == nuui::editor::Mode::Insert && key == Key::Tab {
                                editor.handle_key(key, &config);
                                main_view.refresh_main(&config);
                                dirty = true;
                                continue;
                            }

                            let saved = editor.handle_key(key, &config);
                            if saved {
                                if let Some(path) = editor.file_path.clone() {
                                    for i in 0..6 {
                                        if i != main_view.current_tab
                                            && main_view.editors[i].file_path == Some(path.clone())
                                        {
                                            main_view.editors[i].reload_file();
                                        }
                                    }
                                }
                            }

                            main_view.refresh_main(&config);
                            dirty = true;
                            continue;
                        }
                    }
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
                            main_view.update_macro_focus(false);
                            error::error_box(
                                &terminal,
                                &mut canvas,
                                &format!("Library Error:\n{}\n\nCannot proceed", e),
                                &["EXIT"],
                                min_w,
                                min_h,
                                config.get_border(),
                                main_view.theme.warning_color.clone(),
                                |cvs, w, h, k| main_view.draw_background(cvs, w, h, k, &config),
                            );
                            break;
                        }
                    }
                }

                if main_view.active == main::ActivePanel::List {
                    if let Key::Char(c) = key {
                        if c >= '1'
                            && c <= std::char::from_digit(config.tabs_num.clamp(1, 6) as u32, 10)
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
                            main_view.update_macro_focus(false);
                            error::error_box(
                                &terminal,
                                &mut canvas,
                                &format!("Library Error:\n{}\n\nCannot proceed", e),
                                &["EXIT"],
                                min_w,
                                min_h,
                                config.get_border(),
                                main_view.theme.warning_color.clone(),
                                |cvs, w, h, k| main_view.draw_background(cvs, w, h, k, &config),
                            );
                            break;
                        }
                    }
                }

                match key {
                    Key::Char('q') | Key::Char('\x03') => {
                        if key == Key::Char('q') && config.double_q_exit && !q_pressed_once {
                            q_pressed_once = true;
                            continue;
                        }
                        break;
                    }

                    Key::Tab => main_view.toggle_focus(&config),

                    Key::Esc => {
                        main_view.update_macro_focus(false);
                        let should_quit = settings::settings_modal(
                            &terminal,
                            &mut canvas,
                            &mut config,
                            &mut main_view,
                        );
                        main_view.update_macro_focus(true);

                        if let Ok(l) = lib::init(&config.lib_sorting) {
                            main_view.library_tree = l.tree;
                            main_view.library_root = l.root_path;
                        }
                        main_view.auto_load();
                        main_view.update_min_h(&config);
                        main_view.resize(term_w, term_h, &config);

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
