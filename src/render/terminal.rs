use std::io::{self, Read, Write};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

#[derive(Debug, PartialEq)]
pub enum Key {
    Up,
    Down,
    Left,
    Right,
    ShiftUp,
    ShiftDown,
    ShiftLeft,
    ShiftRight,
    Delete,
    Char(char),
    Ctrl(char),
    Esc,
    Enter,
    Tab,
    Backspace,
    CtrlBackspace,
    None,
}

pub struct Terminal {
    _raw_guard: sys::RawModeGuard,
    key_rx: Receiver<u8>,
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let mut stdout = io::stdout().lock();

        write!(stdout, "\x1b[?25h\x1b[?1049l").unwrap();
        let _ = stdout.flush();
    }
}

impl Terminal {
    pub fn init() -> Self {
        let mut stdout = io::stdout().lock();

        write!(stdout, "\x1b[?1049h\x1b[?25l").unwrap();
        stdout.flush().unwrap();

        let _raw_guard = sys::RawModeGuard::enable();

        let (tx, key_rx) = mpsc::channel();
        thread::spawn(move || {
            let mut buf = [0u8; 32];
            let mut stdin = io::stdin();

            while let Ok(n) = stdin.read(&mut buf) {
                for &byte in &buf[..n] {
                    if tx.send(byte).is_err() {
                        break;
                    }
                }
            }
        });

        Self { _raw_guard, key_rx }
    }

    pub fn read_key(&self, timeout: Duration) -> Key {
        let b = match self.key_rx.recv_timeout(timeout) {
            Ok(b) => b,
            Err(_) => return Key::None,
        };

        match b {
            9 => Key::Tab,
            10 | 13 => Key::Enter,
            8 => Key::CtrlBackspace,
            127 => Key::Backspace,
            27 => {
                match self.key_rx.recv_timeout(Duration::from_millis(16)) {
                    Ok(b'[') => {
                        let mut seq = Vec::new();
                        loop {
                            match self.key_rx.recv_timeout(Duration::from_millis(16)) {
                                Ok(b) => {
                                    seq.push(b);
                                    if b.is_ascii_alphabetic() || b == b'~' {
                                        break;
                                    }
                                }
                                Err(_) => break,
                            }
                        }
                        return match seq.as_slice() {
                            b"A" => Key::Up,
                            b"B" => Key::Down,
                            b"C" => Key::Right,
                            b"D" => Key::Left,
                            b"1;2A" => Key::ShiftUp,
                            b"1;2B" => Key::ShiftDown,
                            b"1;2C" => Key::ShiftRight,
                            b"1;2D" => Key::ShiftLeft,
                            b"3~" => Key::Delete,
                            _ => Key::Esc,
                        };
                    }
                    _ => {}
                }
                Key::Esc
            }
            b if b < 32 => {
                if b >= 1 && b <= 26 {
                    Key::Ctrl((b + 96) as char)
                } else {
                    Key::Char(b as char)
                }
            }
            b => Key::Char(b as char),
        }
    }

    pub fn size() -> (u16, u16) {
        sys::get_terminal_size()
    }
}

#[cfg(unix)]
mod sys {
    use libc::{
        ECHO, ICANON, ICRNL, IEXTEN, ISIG, IXON, OPOST, STDIN_FILENO, STDOUT_FILENO, TCSAFLUSH,
        TIOCGWINSZ, ioctl, tcgetattr, tcsetattr, termios, winsize,
    };
    use std::mem::{self, zeroed};

    pub struct RawModeGuard {
        orig_termios: termios,
    }

    impl Drop for RawModeGuard {
        fn drop(&mut self) {
            unsafe {
                tcsetattr(STDIN_FILENO, TCSAFLUSH, &self.orig_termios);
            }
        }
    }

    impl RawModeGuard {
        pub fn enable() -> Self {
            unsafe {
                let mut termios = mem::zeroed();
                tcgetattr(STDIN_FILENO, &mut termios);
                let orig_termios = termios;

                termios.c_lflag &= !(ECHO | ICANON | IEXTEN | ISIG);
                termios.c_iflag &= !(IXON | ICRNL);
                termios.c_oflag &= !(OPOST);

                tcsetattr(STDIN_FILENO, TCSAFLUSH, &termios);
                Self { orig_termios }
            }
        }
    }

    pub fn get_terminal_size() -> (u16, u16) {
        unsafe {
            let mut ws: winsize = zeroed();
            if ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut ws) == 0 {
                (ws.ws_col, ws.ws_row)
            } else {
                (80, 24)
            }
        }
    }
}

#[cfg(windows)]
mod sys {
    use windows_sys::Win32::System::Console::*;

    pub struct RawModeGuard {
        out_handle: *mut std::ffi::c_void,
        in_handle: *mut std::ffi::c_void,
        orig_out: u32,
        orig_in: u32,
    }

    impl Drop for RawModeGuard {
        fn drop(&mut self) {
            unsafe {
                SetConsoleMode(self.out_handle, self.orig_out);
                SetConsoleMode(self.in_handle, self.orig_in);
            }
        }
    }

    impl RawModeGuard {
        pub fn enable() -> Self {
            unsafe {
                let out_handle = GetStdHandle(STD_OUTPUT_HANDLE);
                let in_handle = GetStdHandle(STD_INPUT_HANDLE);

                let mut orig_out = 0;
                let mut orig_in = 0;
                GetConsoleMode(out_handle, &mut orig_out);
                GetConsoleMode(in_handle, &mut orig_in);

                SetConsoleMode(out_handle, orig_out | ENABLE_VIRTUAL_TERMINAL_PROCESSING);

                let mut in_mode = orig_in;
                in_mode &= !(ENABLE_ECHO_INPUT | ENABLE_LINE_INPUT | ENABLE_PROCESSED_INPUT);
                in_mode |= ENABLE_VIRTUAL_TERMINAL_INPUT;
                SetConsoleMode(in_handle, in_mode);

                Self {
                    out_handle,
                    in_handle,
                    orig_out,
                    orig_in,
                }
            }
        }
    }

    pub fn get_terminal_size() -> (u16, u16) {
        unsafe {
            let out_handle = GetStdHandle(STD_OUTPUT_HANDLE);
            let mut csbi: CONSOLE_SCREEN_BUFFER_INFO = std::mem::zeroed();

            if GetConsoleScreenBufferInfo(out_handle, &mut csbi) != 0 {
                let w = (csbi.srWindow.Right - csbi.srWindow.Left + 1) as u16;
                let h = (csbi.srWindow.Bottom - csbi.srWindow.Top + 1) as u16;
                (w, h)
            } else {
                (80, 24)
            }
        }
    }
}
