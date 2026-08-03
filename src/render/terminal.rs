use std::io::{self, Read, Write};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

#[derive(Debug, PartialEq, Clone)]
pub enum Key {
    Up,
    Down,
    Left,
    Right,
    ShiftUp,
    ShiftDown,
    ShiftLeft,
    ShiftRight,
    CtrlUp,
    CtrlDown,
    CtrlLeft,
    CtrlRight,
    CtrlShiftUp,
    CtrlShiftDown,
    CtrlShiftLeft,
    CtrlShiftRight,
    Delete,
    CtrlDelete,
    Char(char),
    Shift(char),
    Ctrl(char),
    Alt(char),
    Esc,
    Enter,
    Tab,
    ShiftTab,
    Backspace,
    CtrlBackspace,
    None,
    F(u8),
}

pub static HAS_FOCUS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

pub struct Terminal {
    _raw_guard: sys::RawModeGuard,
    key_rx: Receiver<u8>,
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let mut stdout = io::stdout().lock();

        write!(stdout, "\x1b[?1004l\x1b[?25h\x1b[?1049l").unwrap();
        let _ = stdout.flush();
    }
}

impl Terminal {
    pub fn is_caps_lock_on() -> bool {
        sys::is_caps_lock_on()
    }

    pub fn init() -> Self {
        let mut stdout = io::stdout().lock();

        write!(stdout, "\x1b[?1049h\x1b[?25l\x1b[?1004h").unwrap();
        stdout.flush().unwrap();

        let _raw_guard = sys::RawModeGuard::enable();

        let (tx, key_rx) = mpsc::sync_channel(1024);
        thread::spawn(move || {
            let mut buf = [0u8; 32];
            let mut stdin = io::stdin();

            while let Ok(n) = stdin.read(&mut buf) {
                if n == 0 {
                    break;
                }
                for &byte in &buf[..n] {
                    if tx.send(byte).is_err() {
                        return;
                    }
                }
            }
        });

        Self { _raw_guard, key_rx }
    }

    pub fn read_key(&self, timeout: Duration) -> Key {
        let b = match self.key_rx.recv_timeout(timeout) {
            Ok(b) => b,
            Err(mpsc::RecvTimeoutError::Timeout) => return Key::None,
            Err(mpsc::RecvTimeoutError::Disconnected) => return Key::Char('\x03'),
        };

        match b {
            9 => Key::Tab,
            13 => Key::Enter,
            127 => Key::Backspace,
            27 => match self.key_rx.recv_timeout(Duration::from_millis(16)) {
                Ok(127) | Ok(8) => return Key::CtrlBackspace,
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
                        b"I" => {
                            crate::render::terminal::HAS_FOCUS
                                .store(true, std::sync::atomic::Ordering::Relaxed);
                            Key::None
                        }
                        b"O" => {
                            crate::render::terminal::HAS_FOCUS
                                .store(false, std::sync::atomic::Ordering::Relaxed);
                            Key::None
                        }
                        b"A" => Key::Up,
                        b"B" => Key::Down,
                        b"C" => Key::Right,
                        b"D" => Key::Left,
                        b"Z" => Key::ShiftTab,
                        b"1;2A" => Key::ShiftUp,
                        b"1;2B" => Key::ShiftDown,
                        b"1;2C" => Key::ShiftRight,
                        b"1;2D" => Key::ShiftLeft,
                        b"1;5A" => Key::CtrlUp,
                        b"1;5B" => Key::CtrlDown,
                        b"1;5C" => Key::CtrlRight,
                        b"1;5D" => Key::CtrlLeft,
                        b"1;6A" => Key::CtrlShiftUp,
                        b"1;6B" => Key::CtrlShiftDown,
                        b"1;6C" => Key::CtrlShiftRight,
                        b"1;6D" => Key::CtrlShiftLeft,
                        b"3~" => Key::Delete,
                        b"3;5~" => Key::CtrlDelete,
                        b"127;5u" => Key::CtrlBackspace,
                        b"11~" => Key::F(1),
                        b"12~" => Key::F(2),
                        b"13~" => Key::F(3),
                        b"14~" => Key::F(4),
                        b"15~" => Key::F(5),
                        b"17~" => Key::F(6),
                        b"18~" => Key::F(7),
                        b"19~" => Key::F(8),
                        b"20~" => Key::F(9),
                        b"21~" => Key::F(10),
                        b"23~" => Key::F(11),
                        b"24~" => Key::F(12),
                        _ => Key::None,
                    };
                }
                Ok(b'O') => match self.key_rx.recv_timeout(Duration::from_millis(16)) {
                    Ok(b'P') => return Key::F(1),
                    Ok(b'Q') => return Key::F(2),
                    Ok(b'R') => return Key::F(3),
                    Ok(b'S') => return Key::F(4),
                    _ => return Key::Esc,
                },
                Ok(b) => {
                    let c = b as char;
                    if c.is_ascii_uppercase() && !sys::is_caps_lock_on() {
                        return Key::Alt(c.to_ascii_lowercase());
                    }
                    return Key::Alt(c);
                }
                Err(_) => return Key::Esc,
            },
            b if b < 32 => {
                if b >= 1 && b <= 26 {
                    Key::Ctrl((b + 96) as char)
                } else {
                    Key::Char(b as char)
                }
            }
            b => {
                let c = b as char;
                let caps = sys::is_caps_lock_on();
                if c.is_ascii_uppercase() {
                    if caps {
                        Key::Char(c.to_ascii_lowercase())
                    } else {
                        Key::Shift(c.to_ascii_lowercase())
                    }
                } else if c.is_ascii_lowercase() {
                    if caps { Key::Shift(c) } else { Key::Char(c) }
                } else {
                    Key::Char(c)
                }
            }
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

    #[cfg(target_os = "linux")]
    pub fn is_caps_lock_on() -> bool {
        if let Ok(entries) = std::fs::read_dir("/sys/class/leds/") {
            for entry in entries.filter_map(Result::ok) {
                let name = entry.file_name().to_string_lossy().to_lowercase();
                if name.contains("capslock") {
                    if let Ok(content) = std::fs::read_to_string(entry.path().join("brightness")) {
                        if content.trim() == "1" {
                            return true;
                        }
                    }
                }
            }
        }

        use libc::{O_RDONLY, close, ioctl, open};
        const KDGKBLED: libc::c_ulong = 0x4B64;
        unsafe {
            let fd = open(b"/dev/tty\0".as_ptr() as *const libc::c_char, O_RDONLY);
            if fd >= 0 {
                let mut flags: libc::c_char = 0;
                let res = ioctl(fd, KDGKBLED as _, &mut flags);
                close(fd);
                if res == 0 {
                    return (flags & 0x04) != 0;
                }
            }
        }
        false
    }

    #[cfg(not(target_os = "linux"))]
    pub fn is_caps_lock_on() -> bool {
        false
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
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetKeyState, VK_CAPITAL};

    pub fn is_caps_lock_on() -> bool {
        unsafe { (GetKeyState(VK_CAPITAL as i32) & 0x0001) != 0 }
    }

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
