use std::thread;
use std::time::Duration;

use nuui::Canvas;
use nuui::conf::{self, ConfigError};
use nuui::{Key, Terminal};
use nuui::{configerr, main, toosmall};

fn main() {
    let terminal = Terminal::init();
    let (mut term_w, mut term_h);

    let mut config = loop {
        let (current_w, current_h) = Terminal::size();
        term_w = current_w;
        term_h = current_h;

        match conf::init(term_w, term_h) {
            Ok(cfg) => break cfg,
            Err(e) => {
                let display_msg = match &e {
                    ConfigError::Io(io_err) => {
                        panic!("Could not read/write config file: {}", io_err)
                    }
                    ConfigError::SystemPathNotFound => {
                        panic!("Could not locate system config directory.")
                    }
                    ConfigError::SyntaxError(msg) => format!("Syntax Error:\n{}", msg),
                    ConfigError::MissingBox(b) => format!(
                        "Missing layout component:\nRequired block '{}' not found.",
                        b
                    ),
                    ConfigError::MissingVar(v) => format!(
                        "Missing setup variable:\nRequired global key '{}' is absent.",
                        v
                    ),
                    ConfigError::TypeError(m) => format!("Data Type mismatch:\n{}", m),
                };

                match configerr::render_and_handle(&terminal, &display_msg) {
                    configerr::Choice::Regenerate => {
                        if let Err(regen_err) = conf::force_regenerate() {
                            panic!(
                                "Fatal: Failed to execute backup generation: {:?}",
                                regen_err
                            );
                        }
                    }
                    configerr::Choice::Exit => return,
                }
            }
        }
    };

    let mut main_view = main::MainView::new(&config);
    let mut canvas = Canvas::new(term_w, term_h);

    loop {
        let (current_w, current_h) = Terminal::size();

        if current_w != canvas.width || current_h != canvas.height {
            canvas.resize(current_w, current_h);
            term_w = current_w;
            term_h = current_h;

            if term_w >= main_view.min_w && term_h >= main_view.min_h {
                if let Ok(new_cfg) = conf::init(term_w, term_h) {
                    config = new_cfg;
                    main_view = main::MainView::new(&config);
                }
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
            _ => {}
        }

        thread::sleep(Duration::from_millis(16));
    }
}
