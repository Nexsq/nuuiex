use std::thread;
use std::time::Duration;

use nuui::conf::{self, ConfigError, ConfigVar};
use nuui::{toosmall, configerr};
use nuui::{Border, Color, Modifier, Style};
use nuui::{Box, Canvas};
use nuui::{Key, Terminal};

fn main() {
    let terminal = Terminal::init();
    let (mut term_w, mut term_h) = Terminal::size();

    let mut config = loop {
        match conf::init(term_w, term_h) {
            Ok(cfg) => break cfg,
            Err(e) => {
                let display_msg = match &e {
                    ConfigError::Io(io_err) => panic!("Could not read/write config file: {}", io_err),
                    ConfigError::SystemPathNotFound => panic!("Could not locate system config directory."),
                    ConfigError::SyntaxError(msg) => format!("Syntax Error:\n{}", msg),
                    ConfigError::MissingBox(b)    => format!("Missing layout component:\nRequired block '{}' not found.", b),
                    ConfigError::MissingVar(v)    => format!("Missing setup variable:\nRequired global key '{}' is absent.", v),
                    ConfigError::TypeError(m)     => format!("Data Type mismatch:\n{}", m),
                };

                match configerr::render_and_handle(&terminal, &display_msg) {
                    configerr::Choice::Regenerate => {
                        if let Err(regen_err) = conf::force_regenerate() {
                            panic!("Fatal: Failed to execute backup generation: {:?}", regen_err);
                        }
                    }
                    configerr::Choice::Exit => {
                        return;
                    }
                }
            }
        }
    };

    let mut min_w = match config.vars.get("min_w").unwrap() {
        ConfigVar::Int(w) => *w as u16,
        _ => unreachable!(),
    };
    let mut min_h = match config.vars.get("min_h").unwrap() {
        ConfigVar::Int(h) => *h as u16,
        _ => unreachable!(),
    };
    let mut title_text = match config.vars.get("title_s").unwrap() {
        ConfigVar::Text(s) => s.clone(),
        _ => unreachable!(),
    };

    let mut main_cfg = config
        .boxes
        .iter()
        .find(|b| b.name == "main")
        .unwrap()
        .clone();
    let mut tabs_cfg = config
        .boxes
        .iter()
        .find(|b| b.name == "tabs")
        .unwrap()
        .clone();
    let mut title_cfg = config
        .boxes
        .iter()
        .find(|b| b.name == "title")
        .unwrap()
        .clone();

    let mut main = Box::new(
        main_cfg.width,
        main_cfg.height,
        main_cfg.padding,
        main_cfg.border,
        main_cfg.style,
    );
    let mut tabs = Box::new(
        tabs_cfg.width,
        tabs_cfg.height,
        tabs_cfg.padding,
        tabs_cfg.border,
        tabs_cfg.style,
    );
    let mut title = Box::new(
        title_cfg.width,
        title_cfg.height,
        title_cfg.padding,
        title_cfg.border,
        title_cfg.style,
    );

    let mut text_box = Box::new(
        main_cfg.width,
        title_cfg.height,
        0,
        Border::None,
        Style::default(),
    );
    let text_style = Style {
        fg: Color::Red,
        bg: Color::None,
        md: Modifier::Bold,
    };

    text_box.insert_text(&title_text, 1, 0, true, text_style);
    main.insert_box(&text_box, main_cfg.padding as i16, main_cfg.padding as i16);

    let mut canvas = Canvas::new(term_w, term_h);

    loop {
        let (current_w, current_h) = Terminal::size();

        if current_w != canvas.width || current_h != canvas.height {
            canvas.resize(current_w, current_h);
            term_w = current_w;
            term_h = current_h;

            if term_w >= min_w && term_h >= min_h {
                if let Ok(new_cfg) = conf::init(term_w, term_h) {
                    config = new_cfg;

                    min_w = match config.vars.get("min_w").unwrap() {
                        ConfigVar::Int(w) => *w as u16,
                        _ => unreachable!(),
                    };
                    min_h = match config.vars.get("min_h").unwrap() {
                        ConfigVar::Int(h) => *h as u16,
                        _ => unreachable!(),
                    };
                    title_text = match config.vars.get("title_s").unwrap() {
                        ConfigVar::Text(s) => s.clone(),
                        _ => unreachable!(),
                    };

                    main_cfg = config
                        .boxes
                        .iter()
                        .find(|b| b.name == "main")
                        .unwrap()
                        .clone();
                    tabs_cfg = config
                        .boxes
                        .iter()
                        .find(|b| b.name == "tabs")
                        .unwrap()
                        .clone();
                    title_cfg = config
                        .boxes
                        .iter()
                        .find(|b| b.name == "title")
                        .unwrap()
                        .clone();

                    main = Box::new(
                        main_cfg.width,
                        main_cfg.height,
                        main_cfg.padding,
                        main_cfg.border,
                        main_cfg.style,
                    );
                    tabs = Box::new(
                        tabs_cfg.width,
                        tabs_cfg.height,
                        tabs_cfg.padding,
                        tabs_cfg.border,
                        tabs_cfg.style,
                    );
                    title = Box::new(
                        title_cfg.width,
                        title_cfg.height,
                        title_cfg.padding,
                        title_cfg.border,
                        title_cfg.style,
                    );

                    text_box = Box::new(
                        main_cfg.width,
                        title_cfg.height,
                        0,
                        Border::None,
                        Style::default(),
                    );
                    text_box.insert_text(&title_text, 1, 0, true, text_style);
                    title.insert_box(&text_box, main_cfg.padding as i16, main_cfg.padding as i16);
                }
            }
        }

        if term_w < min_w || term_h < min_h {
            toosmall::render(&mut canvas, term_w, term_h);
        } else {
            canvas.clean();
            canvas.put_box(&main, main_cfg.x, main_cfg.y);
            canvas.put_box(&tabs, tabs_cfg.x, tabs_cfg.y);
            canvas.put_box(&title, title_cfg.x, title_cfg.y);
            canvas.render();
        }

        match terminal.read_key() {
            Key::Char('q') | Key::Esc => break,
            Key::Char('\x03') => break,
            _ => {}
        }

        thread::sleep(Duration::from_millis(16));
    }
}
