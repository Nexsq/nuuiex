use super::ast::{BinaryOp, Expr, FunctionDef, Stmt, StringPart};
use super::value::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, SyncSender};

#[cfg(windows)]
struct WinKeyInfo {
    vk: u16,
    req_shift: bool,
    req_ctrl: bool,
    req_alt: bool,
    is_mouse: bool,
    mouse_down: u32,
    mouse_up: u32,
    mouse_data: u32,
}

#[cfg(windows)]
fn parse_win_key(key: &str) -> Result<WinKeyInfo, String> {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;

    let mut info = WinKeyInfo {
        vk: 0,
        req_shift: false,
        req_ctrl: false,
        req_alt: false,
        is_mouse: false,
        mouse_down: 0,
        mouse_up: 0,
        mouse_data: 0,
    };

    match key.to_lowercase().as_str() {
        "lmb" => {
            info.is_mouse = true;
            info.vk = VK_LBUTTON;
            info.mouse_down = MOUSEEVENTF_LEFTDOWN;
            info.mouse_up = MOUSEEVENTF_LEFTUP;
        }
        "rmb" => {
            info.is_mouse = true;
            info.vk = VK_RBUTTON;
            info.mouse_down = MOUSEEVENTF_RIGHTDOWN;
            info.mouse_up = MOUSEEVENTF_RIGHTUP;
        }
        "mmb" => {
            info.is_mouse = true;
            info.vk = VK_MBUTTON;
            info.mouse_down = MOUSEEVENTF_MIDDLEDOWN;
            info.mouse_up = MOUSEEVENTF_MIDDLEUP;
        }
        "sb1" => {
            info.is_mouse = true;
            info.vk = VK_XBUTTON1;
            info.mouse_down = MOUSEEVENTF_XDOWN;
            info.mouse_up = MOUSEEVENTF_XUP;
            info.mouse_data = 0x0001;
        }
        "sb2" => {
            info.is_mouse = true;
            info.vk = VK_XBUTTON2;
            info.mouse_down = MOUSEEVENTF_XDOWN;
            info.mouse_up = MOUSEEVENTF_XUP;
            info.mouse_data = 0x0002;
        }

        "backspace" | "back" => info.vk = VK_BACK,
        "tab" | "\t" => info.vk = VK_TAB,
        "enter" | "return" | "\n" => info.vk = VK_RETURN,
        "shift" => info.vk = VK_SHIFT,
        "ctrl" | "control" => info.vk = VK_CONTROL,
        "alt" | "menu" => info.vk = VK_MENU,
        "pause" => info.vk = VK_PAUSE,
        "capslock" | "caps" => info.vk = VK_CAPITAL,
        "esc" | "escape" => info.vk = VK_ESCAPE,
        "space" => info.vk = VK_SPACE,
        "pgup" | "pageup" => info.vk = VK_PRIOR,
        "pgdn" | "pagedown" => info.vk = VK_NEXT,
        "end" => info.vk = VK_END,
        "home" => info.vk = VK_HOME,
        "left" => info.vk = VK_LEFT,
        "up" => info.vk = VK_UP,
        "right" => info.vk = VK_RIGHT,
        "down" => info.vk = VK_DOWN,
        "prtscr" | "printscreen" => info.vk = VK_SNAPSHOT,
        "ins" | "insert" => info.vk = VK_INSERT,
        "del" | "delete" => info.vk = VK_DELETE,
        "lmeta" | "cmd" | "super" | "win" => info.vk = VK_LWIN,
        "rmeta" => info.vk = VK_RWIN,
        "f1" => info.vk = VK_F1,
        "f2" => info.vk = VK_F2,
        "f3" => info.vk = VK_F3,
        "f4" => info.vk = VK_F4,
        "f5" => info.vk = VK_F5,
        "f6" => info.vk = VK_F6,
        "f7" => info.vk = VK_F7,
        "f8" => info.vk = VK_F8,
        "f9" => info.vk = VK_F9,
        "f10" => info.vk = VK_F10,
        "f11" => info.vk = VK_F11,
        "f12" => info.vk = VK_F12,
        "f13" => info.vk = VK_F13,
        "f14" => info.vk = VK_F14,
        "f15" => info.vk = VK_F15,
        "f16" => info.vk = VK_F16,
        "f17" => info.vk = VK_F17,
        "f18" => info.vk = VK_F18,
        "f19" => info.vk = VK_F19,
        "f20" => info.vk = VK_F20,
        "f21" => info.vk = VK_F21,
        "f22" => info.vk = VK_F22,
        "f23" => info.vk = VK_F23,
        "f24" => info.vk = VK_F24,

        "shiftup" => {
            info.req_shift = true;
            info.vk = VK_UP;
        }
        "shiftdown" => {
            info.req_shift = true;
            info.vk = VK_DOWN;
        }
        "shiftleft" => {
            info.req_shift = true;
            info.vk = VK_LEFT;
        }
        "shiftright" => {
            info.req_shift = true;
            info.vk = VK_RIGHT;
        }
        "ctrlup" => {
            info.req_ctrl = true;
            info.vk = VK_UP;
        }
        "ctrldown" => {
            info.req_ctrl = true;
            info.vk = VK_DOWN;
        }
        "ctrlleft" => {
            info.req_ctrl = true;
            info.vk = VK_LEFT;
        }
        "ctrlright" => {
            info.req_ctrl = true;
            info.vk = VK_RIGHT;
        }
        "ctrlshiftup" => {
            info.req_ctrl = true;
            info.req_shift = true;
            info.vk = VK_UP;
        }
        "ctrlshiftdown" => {
            info.req_ctrl = true;
            info.req_shift = true;
            info.vk = VK_DOWN;
        }
        "ctrlshiftleft" => {
            info.req_ctrl = true;
            info.req_shift = true;
            info.vk = VK_LEFT;
        }
        "ctrlshiftright" => {
            info.req_ctrl = true;
            info.req_shift = true;
            info.vk = VK_RIGHT;
        }
        "ctrldelete" => {
            info.req_ctrl = true;
            info.vk = VK_DELETE;
        }
        "ctrlbackspace" => {
            info.req_ctrl = true;
            info.vk = VK_BACK;
        }

        s if s.chars().count() == 1 => {
            let c = key.chars().next().unwrap();
            unsafe {
                let res = VkKeyScanW(c as u16);
                if res == -1 {
                    return Err(format!("Unrecognized key: '{}'", key));
                }
                let state = (res >> 8) & 0xFF;

                if state & 1 != 0 {
                    info.req_shift = true;
                }
                if state & 2 != 0 {
                    info.req_ctrl = true;
                }
                if state & 4 != 0 {
                    info.req_alt = true;
                }

                info.vk = (res & 0xFF) as u16;
            }
        }
        _ => return Err(format!("Unrecognized key: '{}'", key)),
    }
    Ok(info)
}

fn check_key_down_focus(key_str: &str) -> Result<bool, String> {
    if !crate::render::terminal::HAS_FOCUS.load(std::sync::atomic::Ordering::Relaxed) {
        return Ok(false);
    }
    check_key_down(key_str)
}

#[cfg(windows)]
fn check_key_down(key: &str) -> Result<bool, String> {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;
    let info = parse_win_key(key)?;

    unsafe {
        let is_down = (GetAsyncKeyState(info.vk as i32) as u16 & 0x8000) != 0;
        if !is_down {
            return Ok(false);
        }
        if info.req_shift && (GetAsyncKeyState(VK_SHIFT as i32) as u16 & 0x8000) == 0 {
            return Ok(false);
        }
        if info.req_ctrl && (GetAsyncKeyState(VK_CONTROL as i32) as u16 & 0x8000) == 0 {
            return Ok(false);
        }
        if info.req_alt && (GetAsyncKeyState(VK_MENU as i32) as u16 & 0x8000) == 0 {
            return Ok(false);
        }
        Ok(true)
    }
}

#[cfg(windows)]
fn simulate_key(key: &str, down: bool) -> Result<(), String> {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;
    let info = parse_win_key(key)?;
    unsafe {
        let mut inputs: [INPUT; 4] = std::mem::zeroed();
        let mut count = 0;

        if info.is_mouse {
            inputs[count].r#type = INPUT_MOUSE;
            inputs[count].Anonymous.mi.dwFlags = if down { info.mouse_down } else { info.mouse_up };
            inputs[count].Anonymous.mi.mouseData = info.mouse_data;
            count += 1;
        } else {
            if down {
                if info.req_shift {
                    inputs[count].r#type = INPUT_KEYBOARD;
                    inputs[count].Anonymous.ki.wVk = VK_SHIFT;
                    count += 1;
                }
                if info.req_ctrl {
                    inputs[count].r#type = INPUT_KEYBOARD;
                    inputs[count].Anonymous.ki.wVk = VK_CONTROL;
                    count += 1;
                }
                if info.req_alt {
                    inputs[count].r#type = INPUT_KEYBOARD;
                    inputs[count].Anonymous.ki.wVk = VK_MENU;
                    count += 1;
                }
            }

            inputs[count].r#type = INPUT_KEYBOARD;
            inputs[count].Anonymous.ki.wVk = info.vk;
            inputs[count].Anonymous.ki.dwFlags = if down { 0 } else { KEYEVENTF_KEYUP };
            count += 1;

            if !down {
                if info.req_alt {
                    inputs[count].r#type = INPUT_KEYBOARD;
                    inputs[count].Anonymous.ki.wVk = VK_MENU;
                    inputs[count].Anonymous.ki.dwFlags = KEYEVENTF_KEYUP;
                    count += 1;
                }
                if info.req_ctrl {
                    inputs[count].r#type = INPUT_KEYBOARD;
                    inputs[count].Anonymous.ki.wVk = VK_CONTROL;
                    inputs[count].Anonymous.ki.dwFlags = KEYEVENTF_KEYUP;
                    count += 1;
                }
                if info.req_shift {
                    inputs[count].r#type = INPUT_KEYBOARD;
                    inputs[count].Anonymous.ki.wVk = VK_SHIFT;
                    inputs[count].Anonymous.ki.dwFlags = KEYEVENTF_KEYUP;
                    count += 1;
                }
            }
        }
        SendInput(
            count as u32,
            inputs.as_ptr(),
            std::mem::size_of::<INPUT>() as i32,
        );
    }
    Ok(())
}

#[cfg(windows)]
fn simulate_write(text: &str) -> Result<(), String> {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;

    for c in text.chars() {
        let key_str = match c {
            '\n' => "enter".to_string(),
            '\t' => "tab".to_string(),
            _ => c.to_string(),
        };

        if let Ok(info) = parse_win_key(&key_str) {
            unsafe {
                let mut inputs_down: [INPUT; 4] = std::mem::zeroed();
                let mut down_count = 0;

                if info.req_shift {
                    inputs_down[down_count].r#type = INPUT_KEYBOARD;
                    inputs_down[down_count].Anonymous.ki.wVk = VK_SHIFT;
                    down_count += 1;
                }
                if info.req_ctrl {
                    inputs_down[down_count].r#type = INPUT_KEYBOARD;
                    inputs_down[down_count].Anonymous.ki.wVk = VK_CONTROL;
                    down_count += 1;
                }
                if info.req_alt {
                    inputs_down[down_count].r#type = INPUT_KEYBOARD;
                    inputs_down[down_count].Anonymous.ki.wVk = VK_MENU;
                    down_count += 1;
                }

                inputs_down[down_count].r#type = INPUT_KEYBOARD;
                inputs_down[down_count].Anonymous.ki.wVk = info.vk;
                down_count += 1;

                SendInput(
                    down_count as u32,
                    inputs_down.as_ptr(),
                    std::mem::size_of::<INPUT>() as i32,
                );

                std::thread::sleep(std::time::Duration::from_millis(2));

                let mut inputs_up: [INPUT; 4] = std::mem::zeroed();
                let mut up_count = 0;

                inputs_up[up_count].r#type = INPUT_KEYBOARD;
                inputs_up[up_count].Anonymous.ki.wVk = info.vk;
                inputs_up[up_count].Anonymous.ki.dwFlags = KEYEVENTF_KEYUP;
                up_count += 1;

                if info.req_alt {
                    inputs_up[up_count].r#type = INPUT_KEYBOARD;
                    inputs_up[up_count].Anonymous.ki.wVk = VK_MENU;
                    inputs_up[up_count].Anonymous.ki.dwFlags = KEYEVENTF_KEYUP;
                    up_count += 1;
                }
                if info.req_ctrl {
                    inputs_up[up_count].r#type = INPUT_KEYBOARD;
                    inputs_up[up_count].Anonymous.ki.wVk = VK_CONTROL;
                    inputs_up[up_count].Anonymous.ki.dwFlags = KEYEVENTF_KEYUP;
                    up_count += 1;
                }
                if info.req_shift {
                    inputs_up[up_count].r#type = INPUT_KEYBOARD;
                    inputs_up[up_count].Anonymous.ki.wVk = VK_SHIFT;
                    inputs_up[up_count].Anonymous.ki.dwFlags = KEYEVENTF_KEYUP;
                    up_count += 1;
                }

                SendInput(
                    up_count as u32,
                    inputs_up.as_ptr(),
                    std::mem::size_of::<INPUT>() as i32,
                );
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    Ok(())
}

#[cfg(target_os = "linux")]
struct LinuxKeyInfo {
    code: u16,
    req_shift: bool,
    req_ctrl: bool,
    req_alt: bool,
}

#[cfg(target_os = "linux")]
fn parse_linux_key(key: &str) -> Result<LinuxKeyInfo, String> {
    let mut info = LinuxKeyInfo {
        code: 0,
        req_shift: false,
        req_ctrl: false,
        req_alt: false,
    };
    let mut search_str = key.to_string();

    if key.chars().count() == 1 {
        let c = key.chars().next().unwrap();
        if c.is_ascii_alphabetic() {
            if c.is_ascii_uppercase() {
                info.req_shift = true;
            }
            search_str = c.to_ascii_lowercase().to_string();
        } else {
            let shift_map = [
                ('~', "`"),
                ('!', "1"),
                ('@', "2"),
                ('#', "3"),
                ('$', "4"),
                ('%', "5"),
                ('^', "6"),
                ('&', "7"),
                ('*', "8"),
                ('(', "9"),
                (')', "0"),
                ('_', "-"),
                ('+', "="),
                ('{', "["),
                ('}', "]"),
                ('|', "\\"),
                (':', ";"),
                ('"', "'"),
                ('<', ","),
                ('>', "."),
                ('?', "/"),
            ];
            for &(shifted, unshifted) in &shift_map {
                if c == shifted {
                    info.req_shift = true;
                    search_str = unshifted.to_string();
                    break;
                }
            }
        }
    }

    info.code = match search_str.to_lowercase().as_str() {
        "lmb" => 272,
        "rmb" => 273,
        "mmb" => 274,
        "sb1" => 275,
        "sb2" => 276,
        "esc" | "escape" => 1,
        "1" => 2,
        "2" => 3,
        "3" => 4,
        "4" => 5,
        "5" => 6,
        "6" => 7,
        "7" => 8,
        "8" => 9,
        "9" => 10,
        "0" => 11,
        "minus" | "-" => 12,
        "equal" | "=" => 13,
        "backspace" | "back" => 14,
        "tab" | "\t" => 15,
        "q" => 16,
        "w" => 17,
        "e" => 18,
        "r" => 19,
        "t" => 20,
        "y" => 21,
        "u" => 22,
        "i" => 23,
        "o" => 24,
        "p" => 25,
        "[" => 26,
        "]" => 27,
        "enter" | "return" | "\n" => 28,
        "ctrl" | "control" => 29,
        "a" => 30,
        "s" => 31,
        "d" => 32,
        "f" => 33,
        "g" => 34,
        "h" => 35,
        "j" => 36,
        "k" => 37,
        "l" => 38,
        ";" => 39,
        "'" => 40,
        "`" => 41,
        "shift" => 42,
        "\\" => 43,
        "z" => 44,
        "x" => 45,
        "c" => 46,
        "v" => 47,
        "b" => 48,
        "n" => 49,
        "m" => 50,
        "," => 51,
        "." => 52,
        "/" => 53,
        "alt" | "menu" => 56,
        "space" | " " => 57,
        "capslock" | "caps" => 58,
        "f1" => 59,
        "f2" => 60,
        "f3" => 61,
        "f4" => 62,
        "f5" => 63,
        "f6" => 64,
        "f7" => 65,
        "f8" => 66,
        "f9" => 67,
        "f10" => 68,
        "f11" => 87,
        "f12" => 88,
        "f13" => 183,
        "f14" => 184,
        "f15" => 185,
        "f16" => 186,
        "f17" => 187,
        "f18" => 188,
        "f19" => 189,
        "f20" => 190,
        "f21" => 191,
        "f22" => 192,
        "f23" => 193,
        "f24" => 194,
        "up" => 103,
        "left" => 105,
        "right" => 106,
        "down" => 108,
        "home" => 102,
        "end" => 107,
        "pgup" | "pageup" => 104,
        "pgdn" | "pagedown" => 109,
        "ins" | "insert" => 110,
        "del" | "delete" => 111,
        "prtscr" | "printscreen" => 99,
        "lmeta" | "cmd" | "super" | "win" => 125,
        "rmeta" => 126,
        "shiftup" => {
            info.req_shift = true;
            103
        }
        "shiftdown" => {
            info.req_shift = true;
            108
        }
        "shiftleft" => {
            info.req_shift = true;
            105
        }
        "shiftright" => {
            info.req_shift = true;
            106
        }
        "ctrlup" => {
            info.req_ctrl = true;
            103
        }
        "ctrldown" => {
            info.req_ctrl = true;
            108
        }
        "ctrlleft" => {
            info.req_ctrl = true;
            105
        }
        "ctrlright" => {
            info.req_ctrl = true;
            106
        }
        "ctrlshiftup" => {
            info.req_ctrl = true;
            info.req_shift = true;
            103
        }
        "ctrlshiftdown" => {
            info.req_ctrl = true;
            info.req_shift = true;
            108
        }
        "ctrlshiftleft" => {
            info.req_ctrl = true;
            info.req_shift = true;
            105
        }
        "ctrlshiftright" => {
            info.req_ctrl = true;
            info.req_shift = true;
            106
        }
        "ctrldelete" => {
            info.req_ctrl = true;
            111
        }
        "ctrlbackspace" => {
            info.req_ctrl = true;
            14
        }
        _ => return Err(format!("Unrecognized key: '{}'", key)),
    };
    Ok(info)
}

#[cfg(target_os = "linux")]
fn check_x11_key_down(key_str: &str) -> Option<bool> {
    let info = parse_linux_key(key_str).ok()?;

    use libc::{RTLD_LAZY, c_void, dlclose, dlopen, dlsym};

    unsafe {
        let libx11 = dlopen(b"libX11.so.6\0".as_ptr() as *const i8, RTLD_LAZY);
        if libx11.is_null() {
            return None;
        }

        let xopen_sym = dlsym(libx11, b"XOpenDisplay\0".as_ptr() as *const i8);
        let xquery_sym = dlsym(libx11, b"XQueryKeymap\0".as_ptr() as *const i8);
        let xclose_sym = dlsym(libx11, b"XCloseDisplay\0".as_ptr() as *const i8);

        if xopen_sym.is_null() || xquery_sym.is_null() || xclose_sym.is_null() {
            dlclose(libx11);
            return None;
        }

        let xopendisplay: extern "C" fn(*const i8) -> *mut c_void = std::mem::transmute(xopen_sym);
        let xquerykeymap: extern "C" fn(*mut c_void, *mut u8) -> i32 =
            std::mem::transmute(xquery_sym);
        let xclosedisplay: extern "C" fn(*mut c_void) -> i32 = std::mem::transmute(xclose_sym);

        let display = xopendisplay(std::ptr::null());
        if display.is_null() {
            dlclose(libx11);
            return None;
        }

        let mut keys = [0u8; 32];
        xquerykeymap(display, keys.as_mut_ptr());
        xclosedisplay(display);
        dlclose(libx11);

        if info.code >= 272 {
            return None;
        }

        let is_key_pressed = |linux_code: u16| -> bool {
            let x11_code = linux_code + 8;
            if x11_code > 255 {
                return false;
            }
            let byte = (x11_code / 8) as usize;
            let bit = x11_code % 8;
            (keys[byte] & (1 << bit)) != 0
        };

        let mut is_down = is_key_pressed(info.code);

        let lower = key_str.to_lowercase();
        if lower == "shift" {
            is_down = is_key_pressed(42) || is_key_pressed(54);
        } else if lower == "ctrl" || lower == "control" {
            is_down = is_key_pressed(29) || is_key_pressed(97);
        } else if lower == "alt" || lower == "menu" {
            is_down = is_key_pressed(56) || is_key_pressed(100);
        } else {
            if info.req_shift && !(is_key_pressed(42) || is_key_pressed(54)) {
                return Some(false);
            }
            if info.req_ctrl && !(is_key_pressed(29) || is_key_pressed(97)) {
                return Some(false);
            }
            if info.req_alt && !(is_key_pressed(56) || is_key_pressed(100)) {
                return Some(false);
            }
        }

        Some(is_down)
    }
}

#[cfg(target_os = "linux")]
fn check_key_down(key: &str) -> Result<bool, String> {
    if let Some(pressed) = check_x11_key_down(key) {
        return Ok(pressed);
    }

    let info = parse_linux_key(key)?;

    use libc::{O_NONBLOCK, O_RDONLY, close, ioctl, open};

    struct FdList(Vec<i32>);
    impl Drop for FdList {
        fn drop(&mut self) {
            for &fd in &self.0 {
                unsafe {
                    close(fd);
                }
            }
        }
    }

    thread_local! {
        static KBD_FDS: std::cell::RefCell<Option<FdList>> = std::cell::RefCell::new(None);
    }

    let mut is_down = false;
    let mut shift_down = false;
    let mut ctrl_down = false;
    let mut alt_down = false;
    let mut any_opened = false;

    KBD_FDS.with(|f| {
        if f.borrow().is_none() {
            let mut fds = Vec::new();
            if let Ok(entries) = std::fs::read_dir("/dev/input") {
                for entry in entries.filter_map(Result::ok) {
                    let path = entry.path();
                    if path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .starts_with("event")
                    {
                        unsafe {
                            let mut path_bytes = path.to_str().unwrap_or("").as_bytes().to_vec();
                            path_bytes.push(0);
                            let fd = open(
                                path_bytes.as_ptr() as *const libc::c_char,
                                O_RDONLY | O_NONBLOCK,
                            );
                            if fd >= 0 {
                                let mut ev_bits = [0u8; 4];
                                let eviogbit = 0x80204520;
                                if ioctl(fd, eviogbit, ev_bits.as_mut_ptr()) >= 0 {
                                    if (ev_bits[0] & (1 << 1)) != 0 {
                                        let mut key_bits = [0u8; 96];
                                        let eviogbit_key = 0x80604521;
                                        if ioctl(fd, eviogbit_key, key_bits.as_mut_ptr()) >= 0 {
                                            if (key_bits[57 / 8] & (1 << (57 % 8))) != 0
                                                || (key_bits[1 / 8] & (1 << (1 % 8))) != 0
                                                || (key_bits[272 / 8] & (1 << (272 % 8))) != 0
                                            {
                                                fds.push(fd);
                                            } else {
                                                close(fd);
                                            }
                                        } else {
                                            close(fd);
                                        }
                                    } else {
                                        close(fd);
                                    }
                                } else {
                                    close(fd);
                                }
                            }
                        }
                    }
                }
            }
            *f.borrow_mut() = Some(FdList(fds));
        }

        if let Some(fd_list) = f.borrow().as_ref() {
            if !fd_list.0.is_empty() {
                any_opened = true;
            }
            for &fd in &fd_list.0 {
                unsafe {
                    let mut key_bits = [0u8; 96];
                    let evioca: libc::c_ulong = 0x80604518;
                    if ioctl(fd, evioca as _, key_bits.as_mut_ptr()) >= 0 {
                        let byte = (info.code / 8) as usize;
                        let bit = info.code % 8;
                        if (key_bits[byte] & (1 << bit)) != 0 {
                            is_down = true;
                        }

                        if info.req_shift {
                            for &c in &[42, 54] {
                                if (key_bits[(c / 8) as usize] & (1 << (c % 8))) != 0 {
                                    shift_down = true;
                                }
                            }
                        }
                        if info.req_ctrl {
                            for &c in &[29, 97] {
                                if (key_bits[(c / 8) as usize] & (1 << (c % 8))) != 0 {
                                    ctrl_down = true;
                                }
                            }
                        }
                        if info.req_alt {
                            for &c in &[56, 100] {
                                if (key_bits[(c / 8) as usize] & (1 << (c % 8))) != 0 {
                                    alt_down = true;
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    if !any_opened {
        return Err("Permission denied.\nRun with sudo or add user to 'input' group.".to_string());
    }

    if info.req_shift && !shift_down {
        return Ok(false);
    }
    if info.req_ctrl && !ctrl_down {
        return Ok(false);
    }
    if info.req_alt && !alt_down {
        return Ok(false);
    }

    Ok(is_down)
}

#[cfg(target_os = "linux")]
fn get_or_create_uinput() -> Result<i32, String> {
    thread_local! {
        static UINPUT_FD: std::cell::RefCell<Option<i32>> = std::cell::RefCell::new(None);
    }

    UINPUT_FD.with(|f| {
        if let Some(fd) = *f.borrow() {
            return Ok(fd);
        }

        unsafe {
            let fd = libc::open(
                b"/dev/uinput\0".as_ptr() as *const libc::c_char,
                libc::O_WRONLY | libc::O_NONBLOCK,
            );
            if fd < 0 {
                return Err("Failed to open /dev/uinput. Are you root?".to_string());
            }

            libc::ioctl(fd, 0x40045564, 1);
            libc::ioctl(fd, 0x40045564, 2);

            for i in 1..=276 {
                libc::ioctl(fd, 0x40045565, i);
            }

            libc::ioctl(fd, 0x40045566, 0);
            libc::ioctl(fd, 0x40045566, 1);

            #[repr(C)]
            struct input_id {
                bustype: u16,
                vendor: u16,
                product: u16,
                version: u16,
            }
            #[repr(C)]
            struct uinput_user_dev {
                name: [u8; 80],
                id: input_id,
                ff_effects_max: u32,
                absmax: [i32; 64],
                absmin: [i32; 64],
                absfuzz: [i32; 64],
                absflat: [i32; 64],
            }

            let mut dev: uinput_user_dev = std::mem::zeroed();
            let name = b"nuui_virtual_keyboard\0";
            dev.name[..name.len()].copy_from_slice(name);
            dev.id.bustype = 3;
            dev.id.vendor = 0x1234;
            dev.id.product = 0x5678;
            dev.id.version = 1;

            libc::write(
                fd,
                &dev as *const _ as *const libc::c_void,
                std::mem::size_of::<uinput_user_dev>(),
            );

            if libc::ioctl(fd, 0x5501) < 0 {
                libc::close(fd);
                return Err("Failed to create uinput device".to_string());
            }

            std::thread::sleep(std::time::Duration::from_millis(50));
            *f.borrow_mut() = Some(fd);
            Ok(fd)
        }
    })
}

#[cfg(target_os = "linux")]
fn simulate_key(key: &str, down: bool) -> Result<(), String> {
    let info = parse_linux_key(key)?;
    let fd = get_or_create_uinput()?;

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct input_event {
        time: libc::timeval,
        type_: u16,
        code: u16,
        value: i32,
    }

    let mut evs: Vec<input_event> = Vec::with_capacity(8);

    macro_rules! add_ev {
        ($code:expr, $val:expr) => {
            let mut ev: input_event = unsafe { std::mem::zeroed() };
            ev.type_ = 1;
            ev.code = $code;
            ev.value = $val;
            evs.push(ev);
        };
    }

    if down {
        if info.req_shift {
            add_ev!(42, 1);
        }
        if info.req_ctrl {
            add_ev!(29, 1);
        }
        if info.req_alt {
            add_ev!(56, 1);
        }
    }

    add_ev!(info.code, if down { 1 } else { 0 });

    if !down {
        if info.req_alt {
            add_ev!(56, 0);
        }
        if info.req_ctrl {
            add_ev!(29, 0);
        }
        if info.req_shift {
            add_ev!(42, 0);
        }
    }

    let mut syn: input_event = unsafe { std::mem::zeroed() };
    syn.type_ = 0;
    syn.code = 0;
    syn.value = 0;
    evs.push(syn);

    unsafe {
        libc::write(
            fd,
            evs.as_ptr() as *const libc::c_void,
            evs.len() * std::mem::size_of::<input_event>(),
        );
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn simulate_write(text: &str) -> Result<(), String> {
    let fd = get_or_create_uinput()?;

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct input_event {
        time: libc::timeval,
        type_: u16,
        code: u16,
        value: i32,
    }

    for c in text.chars() {
        let key_str = match c {
            '\n' => "enter".to_string(),
            '\t' => "tab".to_string(),
            _ => c.to_string(),
        };

        if let Ok(info) = parse_linux_key(&key_str) {
            let mut evs: Vec<input_event> = Vec::with_capacity(8);

            macro_rules! add_ev {
                ($code:expr, $val:expr) => {
                    let mut ev: input_event = unsafe { std::mem::zeroed() };
                    ev.type_ = 1;
                    ev.code = $code;
                    ev.value = $val;
                    evs.push(ev);
                };
            }

            if info.req_shift {
                add_ev!(42, 1);
            }
            if info.req_ctrl {
                add_ev!(29, 1);
            }
            if info.req_alt {
                add_ev!(56, 1);
            }

            add_ev!(info.code, 1);

            let mut syn: input_event = unsafe { std::mem::zeroed() };
            syn.type_ = 0;
            syn.code = 0;
            syn.value = 0;
            evs.push(syn);

            unsafe {
                libc::write(
                    fd,
                    evs.as_ptr() as *const libc::c_void,
                    evs.len() * std::mem::size_of::<input_event>(),
                );
            }

            std::thread::sleep(std::time::Duration::from_millis(2));

            evs.clear();
            add_ev!(info.code, 0);

            if info.req_alt {
                add_ev!(56, 0);
            }
            if info.req_ctrl {
                add_ev!(29, 0);
            }
            if info.req_shift {
                add_ev!(42, 0);
            }

            evs.push(syn);

            unsafe {
                libc::write(
                    fd,
                    evs.as_ptr() as *const libc::c_void,
                    evs.len() * std::mem::size_of::<input_event>(),
                );
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    Ok(())
}

fn variant_to_key_str(v: &Value) -> Result<String, String> {
    if let Value::EnumVariant(e, variant, inner) = v {
        if e == "Key" {
            match variant.as_str() {
                "Char" => {
                    if let Some(inner_val) = inner {
                        if let Value::String(s) = &**inner_val {
                            if s.chars().count() == 1 {
                                return Ok(s.clone());
                            }
                            return Err(format!(
                                "Variant '{}' expects a single character string argument",
                                variant
                            ));
                        }
                    }
                    return Err(format!("Variant '{}' expects a string argument", variant));
                }
                "Shift" => {
                    if let Some(inner_val) = inner {
                        if let Value::String(s) = &**inner_val {
                            if s.chars().count() == 1 {
                                return Ok(s.to_uppercase());
                            }
                            return Err(format!(
                                "Variant '{}' expects a single character string argument",
                                variant
                            ));
                        }
                        return Err(format!("Variant '{}' expects a string argument", variant));
                    }
                    return Ok("shift".to_string());
                }
                "Ctrl" => {
                    if inner.is_some() {
                        return Err("Variant 'Ctrl' with argument is not supported. Use Key::CtrlLeft or Key::CtrlRight instead.".to_string());
                    }
                    return Ok("ctrl".to_string());
                }
                "Alt" | "Space" | "CapsLock" | "PgUp" | "PgDn" | "Home" | "End" | "PrtScr"
                | "Insert" | "LMeta" | "RMeta" | "LMB" | "RMB" | "MMB" | "SB1" | "SB2" => {
                    if inner.is_some() {
                        return Err(format!("Variant '{}' does not take arguments", variant));
                    }
                    return Ok(variant.to_lowercase());
                }
                "F" => {
                    if let Some(inner_val) = inner {
                        if let Value::Number(n) = &**inner_val {
                            return Ok(format!("f{}", n));
                        }
                    }
                    return Err(format!("Variant '{}' expects a number argument", variant));
                }
                _ => {
                    if inner.is_some() {
                        return Err(format!("Variant '{}' does not take arguments", variant));
                    }
                    return Ok(variant.to_lowercase());
                }
            }
        }
    }
    Err("Expected a Key".to_string())
}

#[cfg(windows)]
fn get_cursor_pos() -> Result<(i32, i32), String> {
    use windows_sys::Win32::Foundation::POINT;
    use windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos;
    unsafe {
        let mut pt: POINT = std::mem::zeroed();
        if GetCursorPos(&mut pt) != 0 {
            Ok((pt.x, pt.y))
        } else {
            Err("Failed to get cursor position".to_string())
        }
    }
}

#[cfg(target_os = "linux")]
fn get_cursor_pos() -> Result<(i32, i32), String> {
    use libc::{RTLD_LAZY, c_void, dlclose, dlopen, dlsym};
    unsafe {
        let libx11 = dlopen(b"libX11.so.6\0".as_ptr() as *const i8, RTLD_LAZY);
        if !libx11.is_null() {
            let xopen_sym = dlsym(libx11, b"XOpenDisplay\0".as_ptr() as *const i8);
            let xquery_sym = dlsym(libx11, b"XQueryPointer\0".as_ptr() as *const i8);
            let xclose_sym = dlsym(libx11, b"XCloseDisplay\0".as_ptr() as *const i8);
            let xroot_sym = dlsym(libx11, b"XDefaultRootWindow\0".as_ptr() as *const i8);

            if !xopen_sym.is_null()
                && !xquery_sym.is_null()
                && !xclose_sym.is_null()
                && !xroot_sym.is_null()
            {
                let xopendisplay: extern "C" fn(*const i8) -> *mut c_void =
                    std::mem::transmute(xopen_sym);
                let xquerypointer: extern "C" fn(
                    *mut c_void,
                    libc::c_ulong,
                    *mut libc::c_ulong,
                    *mut libc::c_ulong,
                    *mut i32,
                    *mut i32,
                    *mut i32,
                    *mut i32,
                    *mut u32,
                ) -> i32 = std::mem::transmute(xquery_sym);
                let xclosedisplay: extern "C" fn(*mut c_void) -> i32 =
                    std::mem::transmute(xclose_sym);
                let xdefaultrootwindow: extern "C" fn(*mut c_void) -> libc::c_ulong =
                    std::mem::transmute(xroot_sym);

                let display = xopendisplay(std::ptr::null());
                if !display.is_null() {
                    let root = xdefaultrootwindow(display);
                    let mut root_return = 0;
                    let mut child_return = 0;
                    let mut root_x = 0;
                    let mut root_y = 0;
                    let mut win_x = 0;
                    let mut win_y = 0;
                    let mut mask = 0;

                    xquerypointer(
                        display,
                        root,
                        &mut root_return,
                        &mut child_return,
                        &mut root_x,
                        &mut root_y,
                        &mut win_x,
                        &mut win_y,
                        &mut mask,
                    );
                    xclosedisplay(display);
                    dlclose(libx11);
                    return Ok((root_x, root_y));
                }
            }
            dlclose(libx11);
        }
    }

    let output = std::process::Command::new("xdotool")
        .arg("getmouselocation")
        .output()
        .map_err(|e| format!("Failed to run xdotool (is it installed?): {}", e))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut x = 0;
    let mut y = 0;
    for part in stdout.split_whitespace() {
        if let Some(stripped) = part.strip_prefix("x:") {
            x = stripped.parse().unwrap_or(0);
        } else if let Some(stripped) = part.strip_prefix("y:") {
            y = stripped.parse().unwrap_or(0);
        }
    }
    Ok((x, y))
}

#[cfg(not(any(windows, target_os = "linux")))]
fn check_key_down(key: &str) -> Result<bool, String> {
    Err("Key checking is not supported on this OS".to_string())
}

#[cfg(not(any(windows, target_os = "linux")))]
fn get_cursor_pos() -> Result<(i32, i32), String> {
    Err("Cursor position is not supported on this OS".to_string())
}

#[cfg(windows)]
fn set_cursor_pos(x: i32, y: i32, relative: bool) -> Result<(), String> {
    if relative {
        use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
            INPUT, INPUT_MOUSE, MOUSEEVENTF_MOVE, SendInput,
        };
        unsafe {
            let mut input: INPUT = std::mem::zeroed();
            input.r#type = INPUT_MOUSE;
            input.Anonymous.mi.dx = x;
            input.Anonymous.mi.dy = y;
            input.Anonymous.mi.dwFlags = MOUSEEVENTF_MOVE;
            SendInput(1, &input, std::mem::size_of::<INPUT>() as i32);
        }
        Ok(())
    } else {
        use windows_sys::Win32::UI::WindowsAndMessaging::SetCursorPos;
        unsafe {
            if SetCursorPos(x, y) != 0 {
                Ok(())
            } else {
                Err("Failed to set absolute cursor position".to_string())
            }
        }
    }
}

#[cfg(target_os = "linux")]
fn set_cursor_pos(x: i32, y: i32, relative: bool) -> Result<(), String> {
    if relative {
        let fd = get_or_create_uinput()?;
        #[repr(C)]
        #[derive(Clone, Copy)]
        struct input_event {
            time: libc::timeval,
            type_: u16,
            code: u16,
            value: i32,
        }
        let mut evs: Vec<input_event> = Vec::with_capacity(3);
        macro_rules! add_ev {
            ($type:expr, $code:expr, $val:expr) => {
                let mut ev: input_event = unsafe { std::mem::zeroed() };
                ev.type_ = $type;
                ev.code = $code;
                ev.value = $val;
                evs.push(ev);
            };
        }
        if x != 0 {
            add_ev!(2, 0, x);
        }
        if y != 0 {
            add_ev!(2, 1, y);
        }
        add_ev!(0, 0, 0);
        unsafe {
            libc::write(
                fd,
                evs.as_ptr() as *const libc::c_void,
                evs.len() * std::mem::size_of::<input_event>(),
            );
        }
        Ok(())
    } else {
        use libc::{RTLD_LAZY, c_void, dlclose, dlopen, dlsym};
        unsafe {
            let libx11 = dlopen(b"libX11.so.6\0".as_ptr() as *const i8, RTLD_LAZY);
            if !libx11.is_null() {
                let xopen_sym = dlsym(libx11, b"XOpenDisplay\0".as_ptr() as *const i8);
                let xwarp_sym = dlsym(libx11, b"XWarpPointer\0".as_ptr() as *const i8);
                let xflush_sym = dlsym(libx11, b"XFlush\0".as_ptr() as *const i8);
                let xclose_sym = dlsym(libx11, b"XCloseDisplay\0".as_ptr() as *const i8);
                let xroot_sym = dlsym(libx11, b"XDefaultRootWindow\0".as_ptr() as *const i8);

                if !xopen_sym.is_null()
                    && !xwarp_sym.is_null()
                    && !xflush_sym.is_null()
                    && !xclose_sym.is_null()
                    && !xroot_sym.is_null()
                {
                    let xopendisplay: extern "C" fn(*const i8) -> *mut c_void =
                        std::mem::transmute(xopen_sym);
                    let xwarppointer: extern "C" fn(
                        *mut c_void,
                        libc::c_ulong,
                        libc::c_ulong,
                        i32,
                        i32,
                        u32,
                        u32,
                        i32,
                        i32,
                    ) -> i32 = std::mem::transmute(xwarp_sym);
                    let xflush: extern "C" fn(*mut c_void) -> i32 = std::mem::transmute(xflush_sym);
                    let xclosedisplay: extern "C" fn(*mut c_void) -> i32 =
                        std::mem::transmute(xclose_sym);
                    let xdefaultrootwindow: extern "C" fn(*mut c_void) -> libc::c_ulong =
                        std::mem::transmute(xroot_sym);

                    let display = xopendisplay(std::ptr::null());
                    if !display.is_null() {
                        let root = xdefaultrootwindow(display);
                        xwarppointer(display, 0, root, 0, 0, 0, 0, x, y);
                        xflush(display);
                        xclosedisplay(display);
                        dlclose(libx11);
                        return Ok(());
                    }
                }
                dlclose(libx11);
            }
        }

        std::process::Command::new("xdotool")
            .arg("mousemove")
            .arg(x.to_string())
            .arg(y.to_string())
            .output()
            .map_err(|e| format!("Failed to run xdotool (is it installed?): {}", e))?;
        Ok(())
    }
}

#[cfg(not(any(windows, target_os = "linux")))]
fn set_cursor_pos(_x: i32, _y: i32, _relative: bool) -> Result<(), String> {
    Err("Setting cursor position is not supported on this OS".to_string())
}

pub struct Environment {
    pub scopes: Vec<std::sync::Arc<std::sync::Mutex<HashMap<String, Value>>>>,
    pub constants: Vec<std::sync::Arc<std::sync::Mutex<HashSet<String>>>>,
}

impl Environment {
    pub fn new() -> Self {
        let mut env = Self {
            scopes: vec![std::sync::Arc::new(std::sync::Mutex::new(HashMap::new()))],
            constants: vec![std::sync::Arc::new(std::sync::Mutex::new(HashSet::new()))],
        };
        env.define(
            "Key".to_string(),
            Value::BuiltinEnum("Key".to_string()),
            true,
        )
        .unwrap();
        env.define(
            "Color".to_string(),
            Value::BuiltinEnum("Color".to_string()),
            true,
        )
        .unwrap();
        env
    }

    pub fn push(&mut self) {
        self.scopes
            .push(std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())));
        self.constants
            .push(std::sync::Arc::new(std::sync::Mutex::new(HashSet::new())));
    }

    pub fn pop(&mut self) {
        self.scopes.pop();
        self.constants.pop();
    }

    pub fn define(&mut self, name: String, val: Value, is_const: bool) -> Result<(), String> {
        let mut last_scope = self.scopes.last().unwrap().lock().unwrap();
        let mut last_consts = self.constants.last().unwrap().lock().unwrap();

        if last_scope.contains_key(&name) {
            return Err(format!(
                "Variable '{}' is already defined in this scope",
                name
            ));
        }

        last_scope.insert(name.clone(), val);
        if is_const {
            last_consts.insert(name);
        }
        Ok(())
    }

    pub fn assign(&mut self, name: &str, val: Value) -> Result<(), String> {
        for (scope_arc, consts_arc) in self.scopes.iter().zip(self.constants.iter()).rev() {
            let mut scope = scope_arc.lock().unwrap();
            if scope.contains_key(name) {
                let consts = consts_arc.lock().unwrap();
                if consts.contains(name) {
                    return Err(format!("Cannot modify constant '{}'", name));
                }
                scope.insert(name.to_string(), val);
                return Ok(());
            }
        }
        Err(format!("Undefined variable '{}'", name))
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        for scope_arc in self.scopes.iter().rev() {
            let scope = scope_arc.lock().unwrap();
            if let Some(val) = scope.get(name) {
                return Some(val.clone());
            }
        }
        None
    }
}

pub struct Interpreter {
    pub output: Arc<std::sync::Mutex<Vec<String>>>,
    pub errors: Arc<std::sync::Mutex<Vec<String>>>,
    pub current_line: Arc<std::sync::Mutex<String>>,
    tx: SyncSender<crate::EngineMessage>,
    input_rx: Receiver<String>,
    pub should_exit: bool,
    pub env: Environment,
    cancel_token: Arc<AtomicBool>,
    focus_token: Arc<AtomicBool>,
}

#[derive(PartialEq)]
pub enum Signal {
    Empty,
    Break,
    Continue,
    Return(Value),
}

impl Interpreter {
    pub fn new(
        tx: SyncSender<crate::EngineMessage>,
        input_rx: Receiver<String>,
        cancel_token: Arc<AtomicBool>,
        focus_token: Arc<AtomicBool>,
    ) -> Self {
        Self {
            output: Arc::new(std::sync::Mutex::new(Vec::new())),
            errors: Arc::new(std::sync::Mutex::new(Vec::new())),
            current_line: Arc::new(std::sync::Mutex::new(String::new())),
            tx,
            input_rx,
            should_exit: false,
            env: Environment::new(),
            cancel_token,
            focus_token,
        }
    }

    fn send_output(&mut self, is_finished: bool) {
        let mut out_lock = self.output.lock().unwrap();
        let cur_lock = self.current_line.lock().unwrap();
        let err_lock = self.errors.lock().unwrap();

        if out_lock.len() > 1000 {
            let excess = out_lock.len() - 1000;
            out_lock.drain(0..excess);
        }

        let mut res = out_lock.clone();

        if !cur_lock.is_empty() {
            res.push(cur_lock.clone());
        }

        if !err_lock.is_empty() {
            if !res.is_empty() && !res.last().unwrap().is_empty() {
                res.push("".to_string());
            }
            res.push("Runtime Errors:".to_string());
            res.extend(err_lock.clone());
        }

        if is_finished && res.is_empty() {
            res.push("Finished with no output.".to_string());
        }

        drop(out_lock);
        drop(cur_lock);
        drop(err_lock);

        if self.tx.send(crate::EngineMessage::Output(res)).is_err() {
            self.should_exit = true;
        }
    }

    pub fn exec(&mut self, stmts: &[Stmt]) {
        let _ = self.exec_block(stmts);
        self.send_output(true);
    }

    fn exec_block(&mut self, stmts: &[Stmt]) -> Result<Signal, String> {
        self.env.push();
        let mut res = Ok(Signal::Empty);

        for stmt in stmts {
            if self.should_exit || self.cancel_token.load(Ordering::Relaxed) {
                self.should_exit = true;
                break;
            }
            match self.execute_stmt(stmt) {
                Ok(Signal::Break) => {
                    res = Ok(Signal::Break);
                    break;
                }
                Ok(Signal::Continue) => {
                    res = Ok(Signal::Continue);
                    break;
                }
                Ok(Signal::Return(val)) => {
                    res = Ok(Signal::Return(val));
                    break;
                }
                Ok(Signal::Empty) => continue,
                Err(err) => {
                    self.errors.lock().unwrap().push(err);
                    self.should_exit = true;
                    res = Ok(Signal::Empty);
                    break;
                }
            }
        }

        self.env.pop();
        res
    }

    fn assign_expr(&mut self, target: &Expr, value: Value) -> Result<(), String> {
        match target {
            Expr::Ident(name, line) => self
                .env
                .assign(name, value)
                .map_err(|e| format!("Line {}: {}", line, e)),
            Expr::Index(left, index_expr, line) => {
                let mut left_val = self.eval_expr(left)?;
                let index_val = self.eval_expr(index_expr)?;
                if let Value::List(vec) = &mut left_val {
                    if let Value::Number(n) = index_val {
                        if n < 0.0 {
                            return Err(format!("Line {}: List index cannot be negative", line));
                        }
                        let idx = n as usize;
                        if idx < vec.len() {
                            vec[idx] = value;
                            self.assign_expr(left, left_val)
                        } else {
                            Err(format!("Line {}: Index out of bounds", line))
                        }
                    } else {
                        Err(format!("Line {}: List index must be a number", line))
                    }
                } else if let Value::Dict(map) = &mut left_val {
                    map.insert(index_val, value);
                    self.assign_expr(left, left_val)
                } else {
                    Err(format!("Line {}: Cannot index into non-list/dict", line))
                }
            }
            _ => Err("Invalid assignment target".to_string()),
        }
    }

    fn try_assign_expr(&mut self, target: &Expr, value: Value) -> Result<(), String> {
        match target {
            Expr::Ident(name, line) => self
                .env
                .assign(name, value)
                .map_err(|e| format!("Line {}: {}", line, e)),
            Expr::Index(left, index_expr, line) => {
                let mut left_val = self.eval_expr(left)?;
                let index_val = self.eval_expr(index_expr)?;
                if let Value::List(vec) = &mut left_val {
                    if let Value::Number(n) = index_val {
                        if n < 0.0 {
                            return Err(format!("Line {}: List index cannot be negative", line));
                        }
                        let idx = n as usize;
                        if idx < vec.len() {
                            vec[idx] = value;
                            self.try_assign_expr(left, left_val)
                        } else {
                            Err(format!("Line {}: Index out of bounds", line))
                        }
                    } else {
                        Err(format!("Line {}: List index must be a number", line))
                    }
                } else if let Value::Dict(map) = &mut left_val {
                    map.insert(index_val, value);
                    self.try_assign_expr(left, left_val)
                } else {
                    Err(format!("Line {}: Cannot index into non-list/dict", line))
                }
            }
            Expr::MethodCall(left, _, _, _) => self.try_assign_expr(left, value),
            _ => Ok(()),
        }
    }

    fn apply_method(
        &mut self,
        val: &mut Value,
        method: &str,
        args: Vec<Value>,
        line: usize,
    ) -> Result<Value, String> {
        if let Value::List(vec) = val {
            match method {
                "append" => {
                    if args.len() != 1 {
                        return Err(format!("Line {}: 'append' expects 1 argument", line));
                    }
                    vec.push(args[0].clone());
                    Ok(Value::List(vec.clone()))
                }
                "clear" => {
                    if args.len() != 0 {
                        return Err(format!("Line {}: 'clear' expects 0 arguments", line));
                    }
                    vec.clear();
                    Ok(Value::List(vec.clone()))
                }
                "count" => {
                    if args.len() != 0 {
                        return Err(format!("Line {}: 'count' expects 0 arguments", line));
                    }
                    Ok(Value::Number(vec.len() as f64))
                }
                "extend" => {
                    if args.len() != 1 {
                        return Err(format!("Line {}: 'extend' expects 1 argument", line));
                    }
                    if let Value::List(other) = &args[0] {
                        vec.extend(other.clone());
                        Ok(Value::List(vec.clone()))
                    } else {
                        Err(format!("Line {}: 'extend' expects a list", line))
                    }
                }
                "index" => {
                    if args.len() != 1 {
                        return Err(format!("Line {}: 'index' expects 1 argument", line));
                    }
                    if let Some(pos) = vec.iter().position(|x| x == &args[0]) {
                        Ok(Value::Number(pos as f64))
                    } else {
                        Ok(Value::Nil)
                    }
                }
                "insert" => {
                    if args.len() != 2 {
                        return Err(format!("Line {}: 'insert' expects 2 arguments", line));
                    }
                    let element = args[0].clone();
                    if let Value::Number(pos) = args[1] {
                        if pos < 0.0 {
                            return Err(format!("Line {}: Position cannot be negative", line));
                        }
                        let idx = pos as usize;
                        if idx <= vec.len() {
                            vec.insert(idx, element);
                            Ok(Value::List(vec.clone()))
                        } else {
                            Err(format!("Line {}: Index out of bounds", line))
                        }
                    } else {
                        Err(format!("Line {}: Position must be a number", line))
                    }
                }
                "pop" => {
                    if args.len() == 0 {
                        if let Some(popped) = vec.pop() {
                            Ok(popped)
                        } else {
                            Err(format!("Line {}: pop from empty list", line))
                        }
                    } else if args.len() == 1 {
                        if let Value::Number(pos) = args[0] {
                            if pos < 0.0 {
                                return Err(format!("Line {}: Position cannot be negative", line));
                            }
                            let idx = pos as usize;
                            if idx < vec.len() {
                                Ok(vec.remove(idx))
                            } else {
                                Err(format!("Line {}: Index out of bounds", line))
                            }
                        } else {
                            Err(format!("Line {}: Position must be a number", line))
                        }
                    } else {
                        Err(format!("Line {}: 'pop' expects 0 or 1 argument", line))
                    }
                }
                "remove" => {
                    if args.len() != 1 {
                        return Err(format!("Line {}: 'remove' expects 1 argument", line));
                    }
                    if let Some(pos) = vec.iter().position(|x| x == &args[0]) {
                        vec.remove(pos);
                    }
                    Ok(Value::List(vec.clone()))
                }
                _ => Err(format!("Line {}: Undefined list method '{}'", line, method)),
            }
        } else if let Value::Dict(map) = val {
            match method {
                "clear" => {
                    if args.len() != 0 {
                        return Err(format!("Line {}: 'clear' expects 0 arguments", line));
                    }
                    map.clear();
                    Ok(Value::Dict(map.clone()))
                }
                "get" => {
                    if args.len() != 1 && args.len() != 2 {
                        return Err(format!("Line {}: 'get' expects 1 or 2 arguments", line));
                    }
                    if let Some(v) = map.get(&args[0]) {
                        Ok(v.clone())
                    } else if args.len() == 2 {
                        Ok(args[1].clone())
                    } else {
                        Ok(Value::Nil)
                    }
                }
                "keys" => {
                    if args.len() != 0 {
                        return Err(format!("Line {}: 'keys' expects 0 arguments", line));
                    }
                    let keys: Vec<Value> = map.keys().cloned().collect();
                    Ok(Value::List(keys))
                }
                "values" => {
                    if args.len() != 0 {
                        return Err(format!("Line {}: 'values' expects 0 arguments", line));
                    }
                    let vals: Vec<Value> = map.values().cloned().collect();
                    Ok(Value::List(vals))
                }
                "pop" => {
                    if args.len() == 0 {
                        let key = map.keys().next().cloned();
                        if let Some(k) = key {
                            let v = map.remove(&k).unwrap();
                            Ok(Value::List(vec![k, v]))
                        } else {
                            Err(format!("Line {}: pop from empty dict", line))
                        }
                    } else if args.len() == 1 {
                        if let Some(v) = map.remove(&args[0]) {
                            Ok(v)
                        } else {
                            Err(format!("Line {}: KeyError", line))
                        }
                    } else {
                        Err(format!("Line {}: 'pop' expects 0 or 1 argument", line))
                    }
                }
                "update" => {
                    if args.len() != 1 {
                        return Err(format!("Line {}: 'update' expects 1 argument", line));
                    }
                    if let Value::Dict(other) = &args[0] {
                        for (k, v) in other {
                            map.insert(k.clone(), v.clone());
                        }
                        Ok(Value::Dict(map.clone()))
                    } else {
                        Err(format!("Line {}: 'update' expects a dict", line))
                    }
                }
                "set" => {
                    if args.len() != 2 {
                        return Err(format!("Line {}: 'set' expects 2 arguments", line));
                    }
                    let res = map
                        .entry(args[0].clone())
                        .or_insert(args[1].clone())
                        .clone();
                    Ok(res)
                }
                _ => Err(format!("Line {}: Undefined dict method '{}'", line, method)),
            }
        } else if let Value::String(s) = val {
            match method {
                "capitalize" => {
                    if args.len() != 0 {
                        return Err(format!("Line {}: 'capitalize' expects 0 arguments", line));
                    }
                    let res = if let Some(f) = s.chars().next() {
                        f.to_uppercase().collect::<String>() + &s[f.len_utf8()..]
                    } else {
                        String::new()
                    };
                    Ok(Value::String(res))
                }
                "lower" => {
                    if args.len() != 0 {
                        return Err(format!("Line {}: 'lower' expects 0 arguments", line));
                    }
                    Ok(Value::String(s.to_lowercase()))
                }
                "upper" => {
                    if args.len() != 0 {
                        return Err(format!("Line {}: 'upper' expects 0 arguments", line));
                    }
                    Ok(Value::String(s.to_uppercase()))
                }
                "swapcase" => {
                    if args.len() != 0 {
                        return Err(format!("Line {}: 'swapcase' expects 0 arguments", line));
                    }
                    let res: String = s
                        .chars()
                        .map(|c| {
                            if c.is_lowercase() {
                                c.to_uppercase().to_string()
                            } else {
                                c.to_lowercase().to_string()
                            }
                        })
                        .collect();
                    Ok(Value::String(res))
                }
                "count" => {
                    if args.len() != 1 {
                        return Err(format!("Line {}: 'count' expects 1 argument", line));
                    }
                    if let Value::String(sub) = &args[0] {
                        Ok(Value::Number(s.matches(sub).count() as f64))
                    } else {
                        Err(format!("Line {}: 'count' expects a string", line))
                    }
                }
                "index" => {
                    if args.len() != 1 {
                        return Err(format!("Line {}: 'index' expects 1 argument", line));
                    }
                    if let Value::String(sub) = &args[0] {
                        if let Some(idx) = s.find(sub) {
                            let char_idx = s[..idx].chars().count();
                            Ok(Value::Number(char_idx as f64))
                        } else {
                            Ok(Value::Nil)
                        }
                    } else {
                        Err(format!("Line {}: 'index' expects a string", line))
                    }
                }
                "trim" => {
                    if args.len() != 0 {
                        return Err(format!("Line {}: 'trim' expects 0 arguments", line));
                    }
                    Ok(Value::String(s.trim().to_string()))
                }
                "join" => {
                    if args.len() != 1 {
                        return Err(format!("Line {}: 'join' expects 1 argument", line));
                    }
                    if let Value::List(l) = &args[0] {
                        let strings: Vec<String> = l.iter().map(|v| v.to_string()).collect();
                        Ok(Value::String(strings.join(s)))
                    } else {
                        Err(format!("Line {}: 'join' expects a list", line))
                    }
                }
                "split" => {
                    if args.len() == 0 {
                        let parts: Vec<Value> = s
                            .split_whitespace()
                            .map(|p| Value::String(p.to_string()))
                            .collect();
                        Ok(Value::List(parts))
                    } else if args.len() == 1 {
                        if let Value::String(sep) = &args[0] {
                            let parts: Vec<Value> =
                                s.split(sep).map(|p| Value::String(p.to_string())).collect();
                            Ok(Value::List(parts))
                        } else {
                            Err(format!("Line {}: 'split' expects a string separator", line))
                        }
                    } else {
                        Err(format!("Line {}: 'split' expects 0 or 1 argument", line))
                    }
                }
                "replace" => {
                    if args.len() != 2 {
                        return Err(format!("Line {}: 'replace' expects 2 arguments", line));
                    }
                    if let (Value::String(old), Value::String(new)) = (&args[0], &args[1]) {
                        Ok(Value::String(s.replace(old, new)))
                    } else {
                        Err(format!("Line {}: 'replace' expects string arguments", line))
                    }
                }
                "startswith" => {
                    if args.len() != 1 {
                        return Err(format!("Line {}: 'startswith' expects 1 argument", line));
                    }
                    if let Value::String(sub) = &args[0] {
                        Ok(Value::Bool(s.starts_with(sub)))
                    } else {
                        Err(format!("Line {}: 'startswith' expects a string", line))
                    }
                }
                "endswith" => {
                    if args.len() != 1 {
                        return Err(format!("Line {}: 'endswith' expects 1 argument", line));
                    }
                    if let Value::String(sub) = &args[0] {
                        Ok(Value::Bool(s.ends_with(sub)))
                    } else {
                        Err(format!("Line {}: 'endswith' expects a string", line))
                    }
                }
                "asnum" => {
                    if args.len() != 0 {
                        return Err(format!("Line {}: 'asnum' expects 0 arguments", line));
                    }
                    match s.trim().parse::<f64>() {
                        Ok(num) => Ok(Value::Number(num)),
                        Err(_) => Ok(Value::Nil),
                    }
                }
                _ => Err(format!(
                    "Line {}: Undefined string method '{}'",
                    line, method
                )),
            }
        } else if let Value::Number(n_ref) = val {
            let n = *n_ref;
            match method {
                "abs" => {
                    if args.len() != 0 {
                        return Err(format!("Line {}: 'abs' expects 0 arguments", line));
                    }
                    Ok(Value::Number(n.abs()))
                }
                "neg" => {
                    if args.len() != 0 {
                        return Err(format!("Line {}: 'neg' expects 0 arguments", line));
                    }
                    Ok(Value::Number(-n))
                }
                "floor" => {
                    if args.len() != 0 {
                        return Err(format!("Line {}: 'floor' expects 0 arguments", line));
                    }
                    Ok(Value::Number(n.floor()))
                }
                "trunc" => {
                    if args.len() != 0 {
                        return Err(format!("Line {}: 'trunc' expects 0 arguments", line));
                    }
                    Ok(Value::Number(n.trunc()))
                }
                "ceil" => {
                    if args.len() != 0 {
                        return Err(format!("Line {}: 'ceil' expects 0 arguments", line));
                    }
                    Ok(Value::Number(n.ceil()))
                }
                "fract" => {
                    if args.len() != 0 {
                        return Err(format!("Line {}: 'fract' expects 0 arguments", line));
                    }
                    Ok(Value::Number(n.fract()))
                }
                "clamp" => {
                    if args.len() != 2 {
                        return Err(format!("Line {}: 'clamp' expects 2 arguments", line));
                    }
                    if let (Value::Number(min), Value::Number(max)) = (&args[0], &args[1]) {
                        Ok(Value::Number(n.clamp(*min, *max)))
                    } else {
                        Err(format!("Line {}: 'clamp' expects numbers", line))
                    }
                }
                "round" => {
                    if args.len() == 0 {
                        Ok(Value::Number(n.round()))
                    } else if args.len() == 1 {
                        if let Value::Number(places) = &args[0] {
                            let factor = 10.0_f64.powf(*places);
                            Ok(Value::Number((n * factor).round() / factor))
                        } else {
                            Err(format!("Line {}: 'round' expects a number", line))
                        }
                    } else {
                        Err(format!("Line {}: 'round' expects 0 or 1 argument", line))
                    }
                }
                "pow" => {
                    if args.len() != 1 {
                        return Err(format!("Line {}: 'pow' expects 1 argument", line));
                    }
                    if let Value::Number(exp) = &args[0] {
                        Ok(Value::Number(n.powf(*exp)))
                    } else {
                        Err(format!("Line {}: 'pow' expects a number", line))
                    }
                }
                "sqrt" => {
                    if args.len() != 0 {
                        return Err(format!("Line {}: 'sqrt' expects 0 arguments", line));
                    }
                    Ok(Value::Number(n.sqrt()))
                }
                _ => Err(format!(
                    "Line {}: Undefined number method '{}'",
                    line, method
                )),
            }
        } else {
            Err(format!(
                "Line {}: Methods can only be called on lists, dicts, strings, and numbers",
                line
            ))
        }
    }

    fn execute_stmt(&mut self, stmt: &Stmt) -> Result<Signal, String> {
        match stmt {
            Stmt::Expr(expr) => {
                self.eval_expr(expr)?;
                Ok(Signal::Empty)
            }
            Stmt::Let(name, expr, line) => {
                let val = self.eval_expr(expr)?;
                if let Err(e) = self.env.define(name.clone(), val, false) {
                    return Err(format!("Line {}: {}", line, e));
                }
                Ok(Signal::Empty)
            }
            Stmt::Const(name, expr, line) => {
                let val = self.eval_expr(expr)?;
                if let Err(e) = self.env.define(name.clone(), val, true) {
                    return Err(format!("Line {}: {}", line, e));
                }
                Ok(Signal::Empty)
            }
            Stmt::Assign(target, expr, _) => {
                let val = self.eval_expr(expr)?;
                self.assign_expr(target, val)?;
                Ok(Signal::Empty)
            }
            Stmt::AssignOp(target, op, expr, line) => {
                let right_val = self.eval_expr(expr)?;
                let left_val = self.eval_expr(target)?;
                let new_val = self.eval_binary_op(&left_val, op, &right_val, *line)?;
                self.assign_expr(target, new_val)?;
                Ok(Signal::Empty)
            }
            Stmt::If(cond, then_b, elifs, else_b) => {
                let cond_val = self.eval_expr(cond)?;
                if cond_val.is_truthy() {
                    return self.exec_block(then_b);
                }
                for (elif_cond, elif_b) in elifs {
                    let elif_val = self.eval_expr(elif_cond)?;
                    if elif_val.is_truthy() {
                        return self.exec_block(elif_b);
                    }
                }
                if let Some(e_b) = else_b {
                    return self.exec_block(e_b);
                }
                Ok(Signal::Empty)
            }
            Stmt::Loop(body) => {
                loop {
                    if self.should_exit || self.cancel_token.load(Ordering::Relaxed) {
                        self.should_exit = true;
                        break;
                    }
                    match self.exec_block(body)? {
                        Signal::Break => break,
                        Signal::Continue => continue,
                        Signal::Return(v) => return Ok(Signal::Return(v)),
                        Signal::Empty => continue,
                    }
                }
                Ok(Signal::Empty)
            }
            Stmt::While(cond, body) => {
                loop {
                    if self.should_exit || self.cancel_token.load(Ordering::Relaxed) {
                        self.should_exit = true;
                        break;
                    }

                    let cond_val = self.eval_expr(cond)?;
                    if !cond_val.is_truthy() {
                        break;
                    }

                    match self.exec_block(body)? {
                        Signal::Break => break,
                        Signal::Continue => continue,
                        Signal::Return(v) => return Ok(Signal::Return(v)),
                        Signal::Empty => continue,
                    }
                }
                Ok(Signal::Empty)
            }
            Stmt::For(name, expr, body, line) => {
                let iterable_val = self.eval_expr(expr)?;
                let items: Vec<Value> = match iterable_val {
                    Value::List(l) => l,
                    Value::String(s) => s.chars().map(|c| Value::String(c.to_string())).collect(),
                    Value::Dict(d) => d.keys().cloned().collect(),
                    _ => return Err(format!("Line {}: TypeError: value is not iterable", line)),
                };

                self.env.push();
                let _ = self.env.define(name.clone(), Value::Nil, false);
                for item in items {
                    if self.should_exit || self.cancel_token.load(Ordering::Relaxed) {
                        self.should_exit = true;
                        break;
                    }
                    let _ = self.env.assign(name, item);
                    match self.exec_block(body)? {
                        Signal::Break => break,
                        Signal::Continue => continue,
                        Signal::Return(v) => {
                            self.env.pop();
                            return Ok(Signal::Return(v));
                        }
                        Signal::Empty => continue,
                    }
                }
                self.env.pop();
                Ok(Signal::Empty)
            }
            Stmt::Async(body, _) => {
                let async_env = Environment {
                    scopes: self.env.scopes.clone(),
                    constants: self.env.constants.clone(),
                };

                let tx_clone = self.tx.clone();
                let cancel_token_clone = Arc::clone(&self.cancel_token);
                let focus_token_clone = Arc::clone(&self.focus_token);
                let body_clone = body.clone();

                let output_clone = Arc::clone(&self.output);
                let errors_clone = Arc::clone(&self.errors);
                let current_line_clone = Arc::clone(&self.current_line);

                std::thread::spawn(move || {
                    let (_, dummy_rx) = std::sync::mpsc::channel();
                    let mut async_interp = Interpreter {
                        output: output_clone,
                        errors: errors_clone,
                        current_line: current_line_clone,
                        tx: tx_clone,
                        input_rx: dummy_rx,
                        should_exit: false,
                        env: async_env,
                        cancel_token: cancel_token_clone,
                        focus_token: focus_token_clone,
                    };

                    let _ = async_interp.exec_block(&body_clone);
                    async_interp.send_output(false);
                });

                Ok(Signal::Empty)
            }
            Stmt::Break(_) => Ok(Signal::Break),
            Stmt::Continue(_) => Ok(Signal::Continue),
            Stmt::Fn(name, params, body, line) => {
                let func_def = Arc::new(FunctionDef {
                    name: name.clone(),
                    params: params.clone(),
                    body: body.clone(),
                });
                if let Err(e) = self
                    .env
                    .define(name.clone(), Value::Function(func_def), false)
                {
                    return Err(format!("Line {}: {}", line, e));
                }
                Ok(Signal::Empty)
            }
            Stmt::Return(expr) => {
                let val = if let Some(e) = expr {
                    self.eval_expr(e)?
                } else {
                    Value::Nil
                };
                Ok(Signal::Return(val))
            }
        }
    }

    fn eval_binary_op(
        &mut self,
        left: &Value,
        op: &BinaryOp,
        right: &Value,
        line: usize,
    ) -> Result<Value, String> {
        match (left, right) {
            (Value::Number(ln), Value::Number(rn)) => match op {
                BinaryOp::Add => Ok(Value::Number(ln + rn)),
                BinaryOp::Sub => Ok(Value::Number(ln - rn)),
                BinaryOp::Mul => Ok(Value::Number(ln * rn)),
                BinaryOp::Div => {
                    if *rn == 0.0 {
                        Err(format!("Line {}: Division by zero", line))
                    } else {
                        Ok(Value::Number(ln / rn))
                    }
                }
                BinaryOp::Mod => {
                    if *rn == 0.0 {
                        Err(format!("Line {}: Modulo by zero", line))
                    } else {
                        Ok(Value::Number(ln % rn))
                    }
                }
                BinaryOp::EqEq => Ok(Value::Bool(ln == rn)),
                BinaryOp::NotEq => Ok(Value::Bool(ln != rn)),
                BinaryOp::Less => Ok(Value::Bool(ln < rn)),
                BinaryOp::Greater => Ok(Value::Bool(ln > rn)),
                BinaryOp::LessEq => Ok(Value::Bool(ln <= rn)),
                BinaryOp::GreaterEq => Ok(Value::Bool(ln >= rn)),
                _ => Err(format!("Line {}: Unsupported number operation", line)),
            },
            (Value::Bool(lb), Value::Bool(rb)) => match op {
                BinaryOp::EqEq => Ok(Value::Bool(lb == rb)),
                BinaryOp::NotEq => Ok(Value::Bool(lb != rb)),
                _ => Err(format!("Line {}: Unsupported boolean operation", line)),
            },
            (Value::String(ls), right) if *op == BinaryOp::Add => {
                Ok(Value::String(format!("{}{}", ls, right)))
            }
            (left, Value::String(rs)) if *op == BinaryOp::Add => {
                Ok(Value::String(format!("{}{}", left, rs)))
            }
            (Value::String(ls), Value::String(rs)) => match op {
                BinaryOp::EqEq => Ok(Value::Bool(ls == rs)),
                BinaryOp::NotEq => Ok(Value::Bool(ls != rs)),
                _ => Err(format!("Line {}: Unsupported string operation", line)),
            },
            (Value::String(ls), Value::Number(rn)) => match op {
                BinaryOp::Mul => {
                    let count = *rn as i64;
                    if count < 0 {
                        return Err(format!(
                            "Line {}: Cannot multiply string by a negative number",
                            line
                        ));
                    }
                    if count > 1_000_000 {
                        return Err(format!(
                            "Line {}: String multiplication exceeds size limit",
                            line
                        ));
                    }
                    Ok(Value::String(ls.repeat(count as usize)))
                }
                _ => Err(format!("Line {}: Unsupported string operation", line)),
            },
            (Value::Number(ln), Value::String(rs)) => match op {
                BinaryOp::Mul => {
                    let count = *ln as i64;
                    if count < 0 {
                        return Err(format!(
                            "Line {}: Cannot multiply string by a negative number",
                            line
                        ));
                    }
                    if count > 1_000_000 {
                        return Err(format!(
                            "Line {}: String multiplication exceeds size limit",
                            line
                        ));
                    }
                    Ok(Value::String(rs.repeat(count as usize)))
                }
                _ => Err(format!("Line {}: Unsupported string operation", line)),
            },
            _ => {
                if *op == BinaryOp::EqEq {
                    Ok(Value::Bool(left == right))
                } else if *op == BinaryOp::NotEq {
                    Ok(Value::Bool(left != right))
                } else {
                    Err(format!("Line {}: Unsupported operands for {:?}", line, op))
                }
            }
        }
    }

    fn eval_expr(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::Number(n) => Ok(Value::Number(*n)),
            Expr::String(s) => Ok(Value::String(s.clone())),
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::Nil => Ok(Value::Nil),
            Expr::FormatString(parts) => {
                let mut result = String::new();
                for part in parts {
                    match part {
                        StringPart::Text(t) => result.push_str(t),
                        StringPart::Expr(e) => {
                            let val = self.eval_expr(e)?;
                            result.push_str(&val.to_string());
                        }
                    }
                }
                Ok(Value::String(result))
            }
            Expr::List(items) => {
                let mut vec = Vec::with_capacity(items.len());
                for item in items {
                    vec.push(self.eval_expr(item)?);
                }
                Ok(Value::List(vec))
            }
            Expr::Dict(items) => {
                let mut map = std::collections::HashMap::with_capacity(items.len());
                for (k_expr, v_expr) in items {
                    let k_val = self.eval_expr(k_expr)?;
                    let v_val = self.eval_expr(v_expr)?;
                    map.insert(k_val, v_val);
                }
                Ok(Value::Dict(map))
            }
            Expr::Ident(name, line) => {
                if let Some(val) = self.env.get(name) {
                    Ok(val.clone())
                } else {
                    Err(format!("Line {}: Undefined variable '{}'", line, name))
                }
            }
            Expr::StaticAccess(left, prop, line) => {
                let left_val = self.eval_expr(left)?;
                if let Value::BuiltinEnum(enum_name) = left_val {
                    if enum_name == "Key" {
                        let valid_variants = [
                            "Up",
                            "Down",
                            "Left",
                            "Right",
                            "ShiftUp",
                            "ShiftDown",
                            "ShiftLeft",
                            "ShiftRight",
                            "CtrlUp",
                            "CtrlDown",
                            "CtrlLeft",
                            "CtrlRight",
                            "CtrlShiftUp",
                            "CtrlShiftDown",
                            "CtrlShiftLeft",
                            "CtrlShiftRight",
                            "Delete",
                            "CtrlDelete",
                            "Char",
                            "Shift",
                            "Ctrl",
                            "Alt",
                            "Esc",
                            "Enter",
                            "Tab",
                            "Backspace",
                            "CtrlBackspace",
                            "None",
                            "F",
                            "Space",
                            "CapsLock",
                            "PgUp",
                            "PgDn",
                            "Home",
                            "End",
                            "PrtScr",
                            "Insert",
                            "LMeta",
                            "RMeta",
                            "LMB",
                            "RMB",
                            "MMB",
                            "SB1",
                            "SB2",
                        ];
                        if !valid_variants.contains(&prop.as_str()) {
                            return Err(format!(
                                "Line {}: Invalid variant '{}' for enum 'Key'",
                                line, prop
                            ));
                        }
                    } else if enum_name == "Color" {
                        if crate::theme::themecore::parse_color(&prop).is_err() {
                            return Err(format!("Line {}: Invalid color variant '{}'", line, prop));
                        }
                    }
                    Ok(Value::EnumVariant(enum_name, prop.clone(), None))
                } else {
                    Err(format!(
                        "Line {}: Static access '::' is only supported on enums",
                        line
                    ))
                }
            }
            Expr::Index(left, index_expr, line) => {
                let left_val = self.eval_expr(left)?;
                let index_val = self.eval_expr(index_expr)?;
                if let Value::List(vec) = left_val {
                    if let Value::Number(n) = index_val {
                        if n < 0.0 {
                            return Err(format!("Line {}: List index cannot be negative", line));
                        }
                        let idx = n as usize;
                        if idx < vec.len() {
                            Ok(vec[idx].clone())
                        } else {
                            Ok(Value::Nil)
                        }
                    } else {
                        Err(format!("Line {}: List index must be a number", line))
                    }
                } else if let Value::Dict(map) = left_val {
                    if let Some(v) = map.get(&index_val) {
                        Ok(v.clone())
                    } else {
                        Ok(Value::Nil)
                    }
                } else {
                    Err(format!("Line {}: Cannot index into non-list/dict", line))
                }
            }
            Expr::MethodCall(left, method, args, line) => {
                let mut eval_args = Vec::with_capacity(args.len());
                for (kw_opt, arg) in args {
                    if kw_opt.is_some() {
                        return Err(format!(
                            "Line {}: keyword arguments not supported in method calls",
                            line
                        ));
                    }
                    eval_args.push(self.eval_expr(arg)?);
                }
                let mut left_val = self.eval_expr(left)?;

                if let Value::BuiltinEnum(enum_name) = &left_val {
                    if enum_name == "Key" {
                        let valid_variants = [
                            "Up",
                            "Down",
                            "Left",
                            "Right",
                            "ShiftUp",
                            "ShiftDown",
                            "ShiftLeft",
                            "ShiftRight",
                            "CtrlUp",
                            "CtrlDown",
                            "CtrlLeft",
                            "CtrlRight",
                            "CtrlShiftUp",
                            "CtrlShiftDown",
                            "CtrlShiftLeft",
                            "CtrlShiftRight",
                            "Delete",
                            "CtrlDelete",
                            "Char",
                            "Shift",
                            "Ctrl",
                            "Alt",
                            "Esc",
                            "Enter",
                            "Tab",
                            "Backspace",
                            "CtrlBackspace",
                            "None",
                            "F",
                            "Space",
                            "CapsLock",
                            "PgUp",
                            "PgDn",
                            "Home",
                            "End",
                            "PrtScr",
                            "Insert",
                            "LMeta",
                            "RMeta",
                            "LMB",
                            "RMB",
                            "MMB",
                            "SB1",
                            "SB2",
                        ];
                        if !valid_variants.contains(&method.as_str()) {
                            return Err(format!(
                                "Line {}: Invalid variant '{}' for enum 'Key'",
                                line, method
                            ));
                        }
                        if eval_args.len() != 1 {
                            return Err(format!(
                                "Line {}: Enum variant constructor expects exactly 1 argument",
                                line
                            ));
                        }
                        return Ok(Value::EnumVariant(
                            enum_name.clone(),
                            method.clone(),
                            Some(Box::new(eval_args[0].clone())),
                        ));
                    } else if enum_name == "Color" {
                        if crate::theme::themecore::parse_color(&method).is_err() {
                            return Err(format!(
                                "Line {}: Invalid color variant '{}'",
                                line, method
                            ));
                        }
                        if !eval_args.is_empty() {
                            return Err(format!(
                                "Line {}: Color variant does not take arguments",
                                line
                            ));
                        }
                        return Ok(Value::EnumVariant(enum_name.clone(), method.clone(), None));
                    }
                }

                let res = self.apply_method(&mut left_val, method, eval_args, *line)?;
                self.try_assign_expr(left, left_val)?;
                Ok(res)
            }
            Expr::Not(expr, _) => {
                let val = self.eval_expr(expr)?;
                Ok(Value::Bool(!val.is_truthy()))
            }
            Expr::Binary(left, op, right, line) => {
                if *op == BinaryOp::And {
                    let l = self.eval_expr(left)?;
                    if !l.is_truthy() {
                        return Ok(l);
                    }
                    return self.eval_expr(right);
                } else if *op == BinaryOp::Or {
                    let l = self.eval_expr(left)?;
                    if l.is_truthy() {
                        return Ok(l);
                    }
                    return self.eval_expr(right);
                }

                let l = self.eval_expr(left)?;
                let r = self.eval_expr(right)?;
                self.eval_binary_op(&l, op, &r, *line)
            }
            Expr::Call(name, args, line) => {
                let mut eval_args = Vec::with_capacity(args.len());
                for (kw_opt, arg) in args {
                    eval_args.push((kw_opt.clone(), self.eval_expr(arg)?));
                }

                match name.as_str() {
                    "print" | "println" => {
                        let mut combined = String::new();
                        for (i, (_, arg)) in eval_args.iter().enumerate() {
                            if i > 0 {
                                combined.push(' ');
                            }
                            combined.push_str(&arg.to_string());
                        }

                        let segments: Vec<&str> = combined.split('\n').collect();
                        for (i, segment) in segments.iter().enumerate() {
                            if i == 0 {
                                self.current_line.lock().unwrap().push_str(segment);
                            } else {
                                let mut cur = self.current_line.lock().unwrap();
                                let prev_line = std::mem::take(&mut *cur);
                                self.output.lock().unwrap().push(prev_line);
                                *cur = segment.to_string();
                            }
                        }

                        if name == "println" {
                            let mut cur = self.current_line.lock().unwrap();
                            let prev_line = std::mem::take(&mut *cur);
                            self.output.lock().unwrap().push(prev_line);
                        }

                        self.send_output(false);
                        return Ok(Value::Nil);
                    }
                    "sleep" => {
                        if eval_args.len() != 1 {
                            return Err(format!(
                                "Line {}: 'sleep' expects exactly 1 argument",
                                line
                            ));
                        }
                        if let Value::Number(ms) = eval_args[0].1 {
                            if ms < 0.0 {
                                return Err(format!(
                                    "Line {}: 'sleep' time cannot be negative",
                                    line
                                ));
                            }

                            let target = std::time::Instant::now()
                                + std::time::Duration::from_millis(ms as u64);
                            while std::time::Instant::now() < target {
                                if self.cancel_token.load(Ordering::Relaxed) {
                                    self.should_exit = true;
                                    return Ok(Value::Nil);
                                }
                                std::thread::sleep(std::time::Duration::from_millis(10));
                            }
                            return Ok(Value::Nil);
                        } else {
                            return Err(format!("Line {}: 'sleep' expects a number", line));
                        }
                    }
                    "exit" => {
                        if eval_args.len() != 0 {
                            return Err(format!("Line {}: 'exit' expects 0 arguments", line));
                        }
                        self.should_exit = true;
                        return Ok(Value::Nil);
                    }
                    "range" => {
                        let (start, stop, step) = match eval_args.len() {
                            1 => {
                                if let Value::Number(stop) = eval_args[0].1 {
                                    (0.0, stop, 1.0)
                                } else {
                                    return Err(format!("Line {}: 'range' expects numbers", line));
                                }
                            }
                            2 => {
                                if let (Value::Number(start), Value::Number(stop)) =
                                    (&eval_args[0].1, &eval_args[1].1)
                                {
                                    (*start, *stop, 1.0)
                                } else {
                                    return Err(format!("Line {}: 'range' expects numbers", line));
                                }
                            }
                            3 => {
                                if let (
                                    Value::Number(start),
                                    Value::Number(stop),
                                    Value::Number(step),
                                ) = (&eval_args[0].1, &eval_args[1].1, &eval_args[2].1)
                                {
                                    if *step == 0.0 {
                                        return Err(format!(
                                            "Line {}: 'range' step cannot be zero",
                                            line
                                        ));
                                    }
                                    (*start, *stop, *step)
                                } else {
                                    return Err(format!("Line {}: 'range' expects numbers", line));
                                }
                            }
                            _ => {
                                return Err(format!(
                                    "Line {}: 'range' expects 1 to 3 arguments",
                                    line
                                ));
                            }
                        };

                        let mut items = Vec::new();
                        let mut curr = start;
                        if step > 0.0 {
                            while curr < stop {
                                items.push(Value::Number(curr));
                                curr += step;
                            }
                        } else {
                            while curr > stop {
                                items.push(Value::Number(curr));
                                curr += step;
                            }
                        }
                        return Ok(Value::List(items));
                    }
                    "input" => {
                        if !eval_args.is_empty() {
                            let prompt = eval_args[0].1.to_string();
                            self.current_line.lock().unwrap().push_str(&prompt);
                        }
                        self.send_output(false);

                        if self.tx.send(crate::EngineMessage::InputRequest).is_err() {
                            self.should_exit = true;
                            return Ok(Value::Nil);
                        }

                        let result = loop {
                            if self.cancel_token.load(Ordering::Relaxed) {
                                self.should_exit = true;
                                return Ok(Value::Nil);
                            }
                            if let Ok(line_in) = self
                                .input_rx
                                .recv_timeout(std::time::Duration::from_millis(16))
                            {
                                break line_in;
                            }
                        };

                        self.current_line.lock().unwrap().push_str(&result);
                        return Ok(Value::String(result));
                    }
                    "len" => {
                        if eval_args.len() != 1 {
                            return Err(format!("Line {}: 'len' expects exactly 1 argument", line));
                        }
                        match &eval_args[0].1 {
                            Value::String(s) => return Ok(Value::Number(s.chars().count() as f64)),
                            Value::List(l) => return Ok(Value::Number(l.len() as f64)),
                            Value::Dict(d) => return Ok(Value::Number(d.len() as f64)),
                            _ => {
                                return Err(format!(
                                    "Line {}: 'len' is not supported for this type",
                                    line
                                ));
                            }
                        }
                    }
                    "max" | "min" => {
                        if eval_args.is_empty() {
                            return Err(format!(
                                "Line {}: '{}' expects at least 1 argument",
                                line, name
                            ));
                        }

                        let items = if eval_args.len() == 1 {
                            match &eval_args[0].1 {
                                Value::List(l) => l.clone(),
                                Value::String(s) => {
                                    s.chars().map(|c| Value::String(c.to_string())).collect()
                                }
                                _ => {
                                    return Err(format!(
                                        "Line {}: '{}' argument is an empty sequence or not iterable",
                                        line, name
                                    ));
                                }
                            }
                        } else {
                            eval_args.into_iter().map(|(_, v)| v).collect()
                        };

                        if items.is_empty() {
                            return Err(format!(
                                "Line {}: '{}' argument is an empty sequence",
                                line, name
                            ));
                        }

                        let first = &items[0];
                        if let Value::Number(_) = first {
                            let mut best = if let Value::Number(n) = first {
                                *n
                            } else {
                                0.0
                            };
                            for item in items.iter().skip(1) {
                                if let Value::Number(n) = item {
                                    if name == "max" {
                                        if *n > best {
                                            best = *n;
                                        }
                                    } else {
                                        if *n < best {
                                            best = *n;
                                        }
                                    }
                                } else {
                                    return Err(format!(
                                        "Line {}: '{}' called with mixed types",
                                        line, name
                                    ));
                                }
                            }
                            return Ok(Value::Number(best));
                        } else if let Value::String(_) = first {
                            let mut best = if let Value::String(s) = first {
                                s.clone()
                            } else {
                                String::new()
                            };
                            for item in items.iter().skip(1) {
                                if let Value::String(s) = item {
                                    if name == "max" {
                                        if s > &best {
                                            best = s.clone();
                                        }
                                    } else {
                                        if s < &best {
                                            best = s.clone();
                                        }
                                    }
                                } else {
                                    return Err(format!(
                                        "Line {}: '{}' called with mixed types",
                                        line, name
                                    ));
                                }
                            }
                            return Ok(Value::String(best));
                        } else {
                            return Err(format!(
                                "Line {}: '{}' only supports numbers and strings",
                                line, name
                            ));
                        }
                    }
                    "exec" => {
                        if eval_args.len() != 1 {
                            return Err(format!(
                                "Line {}: 'exec' expects exactly 1 argument",
                                line
                            ));
                        }
                        if let Value::String(cmd_str) = &eval_args[0].1 {
                            let mut cmd = if cfg!(target_os = "windows") {
                                let mut c = std::process::Command::new("powershell");
                                c.args(["-NoProfile", "-Command", cmd_str]);
                                c
                            } else {
                                let mut c = std::process::Command::new("sh");
                                c.args(["-c", cmd_str]);
                                c
                            };

                            #[cfg(target_os = "windows")]
                            {
                                use std::os::windows::process::CommandExt;
                                cmd.creation_flags(0x08000000);
                            }

                            match cmd.output() {
                                Ok(out) => {
                                    let mut result =
                                        String::from_utf8_lossy(&out.stdout).into_owned();
                                    let stderr = String::from_utf8_lossy(&out.stderr);
                                    if !stderr.is_empty() {
                                        if !result.is_empty() && !result.ends_with('\n') {
                                            result.push('\n');
                                        }
                                        result.push_str(&stderr);
                                    }
                                    return Ok(Value::String(result.trim_end().to_string()));
                                }
                                Err(e) => {
                                    return Err(format!(
                                        "Line {}: Failed to execute command: {}",
                                        line, e
                                    ));
                                }
                            }
                        } else {
                            return Err(format!("Line {}: 'exec' expects a string", line));
                        }
                    }
                    "onlinux" => {
                        if !eval_args.is_empty() {
                            return Err(format!("Line {}: 'onlinux' expects 0 arguments", line));
                        }
                        return Ok(Value::Bool(cfg!(target_os = "linux")));
                    }
                    "isdown" | "isup" | "isdownfocus" | "isupfocus" => {
                        if eval_args.len() != 1 {
                            return Err(format!(
                                "Line {}: '{}' expects exactly 1 argument",
                                line, name
                            ));
                        }

                        let key_str = match variant_to_key_str(&eval_args[0].1) {
                            Ok(s) => s,
                            Err(e) => return Err(format!("Line {}: {}", line, e)),
                        };

                        let is_focus_cmd = name.ends_with("focus");
                        let has_macro_focus = self.focus_token.load(Ordering::Relaxed);

                        if is_focus_cmd && !has_macro_focus {
                            return Ok(Value::Bool(false));
                        }

                        let is_down = if is_focus_cmd {
                            match check_key_down_focus(&key_str) {
                                Ok(b) => b,
                                Err(e) => return Err(format!("Line {}: {}", line, e)),
                            }
                        } else {
                            match check_key_down(&key_str) {
                                Ok(b) => b,
                                Err(e) => return Err(format!("Line {}: {}", line, e)),
                            }
                        };

                        if name.starts_with("isdown") {
                            return Ok(Value::Bool(is_down));
                        } else {
                            return Ok(Value::Bool(!is_down));
                        }
                    }
                    "keydown" | "keyup" => {
                        if eval_args.len() != 1 {
                            return Err(format!(
                                "Line {}: '{}' expects exactly 1 argument",
                                line, name
                            ));
                        }

                        let key_str = match variant_to_key_str(&eval_args[0].1) {
                            Ok(s) => s,
                            Err(e) => return Err(format!("Line {}: {}", line, e)),
                        };

                        let is_down = name == "keydown";
                        if let Err(e) = simulate_key(&key_str, is_down) {
                            return Err(format!("Line {}: {}", line, e));
                        }
                        return Ok(Value::Nil);
                    }
                    "write" => {
                        if eval_args.len() != 1 {
                            return Err(format!(
                                "Line {}: 'write' expects exactly 1 argument",
                                line
                            ));
                        }
                        if let Value::String(text) = &eval_args[0].1 {
                            if let Err(e) = simulate_write(text) {
                                return Err(format!("Line {}: {}", line, e));
                            }
                            return Ok(Value::Nil);
                        } else {
                            return Err(format!("Line {}: 'write' expects a string", line));
                        }
                    }
                    "cursorx" => {
                        if eval_args.len() != 0 {
                            return Err(format!(
                                "Line {}: 'cursorx' expects exactly 0 arguments",
                                line
                            ));
                        }
                        let (x, _) =
                            get_cursor_pos().map_err(|e| format!("Line {}: {}", line, e))?;
                        return Ok(Value::Number(x as f64));
                    }
                    "cursory" => {
                        if eval_args.len() != 0 {
                            return Err(format!(
                                "Line {}: 'cursory' expects exactly 0 arguments",
                                line
                            ));
                        }
                        let (_, y) =
                            get_cursor_pos().map_err(|e| format!("Line {}: {}", line, e))?;
                        return Ok(Value::Number(y as f64));
                    }
                    "setcursor" => {
                        if eval_args.len() < 2 || eval_args.len() > 3 {
                            return Err(format!(
                                "Line {}: 'setcursor' expects 2 or 3 arguments",
                                line
                            ));
                        }
                        let x = if let Value::Number(n) = eval_args[0].1 {
                            n as i32
                        } else {
                            return Err(format!("Line {}: 'setcursor' x must be a number", line));
                        };
                        let y = if let Value::Number(n) = eval_args[1].1 {
                            n as i32
                        } else {
                            return Err(format!("Line {}: 'setcursor' y must be a number", line));
                        };
                        let relative = if eval_args.len() == 3 {
                            if let Value::Bool(b) = eval_args[2].1 {
                                b
                            } else {
                                return Err(format!(
                                    "Line {}: 'setcursor' relative flag must be a boolean",
                                    line
                                ));
                            }
                        } else {
                            false
                        };
                        set_cursor_pos(x, y, relative)
                            .map_err(|e| format!("Line {}: {}", line, e))?;
                        return Ok(Value::Nil);
                    }
                    "clear" => {
                        if eval_args.len() != 0 {
                            return Err(format!(
                                "Line {}: 'clear' expects exactly 0 arguments",
                                line
                            ));
                        }
                        self.output.lock().unwrap().clear();
                        self.current_line.lock().unwrap().clear();
                        self.send_output(false);
                        return Ok(Value::Nil);
                    }
                    _ => {}
                }

                let func_val = if let Some(v) = self.env.get(name) {
                    v
                } else {
                    return Err(format!("Line {}: Undefined function '{}'", line, name));
                };

                if let Value::Function(func_def) = func_val {
                    let mut final_args = std::collections::HashMap::new();
                    let mut positional_count = 0;

                    for (kw_opt, val) in eval_args {
                        if let Some(kw) = kw_opt {
                            if final_args.contains_key(&kw) {
                                return Err(format!(
                                    "Line {}: duplicate keyword argument '{}'",
                                    line, kw
                                ));
                            }
                            final_args.insert(kw, val);
                        } else {
                            if positional_count >= func_def.params.len() {
                                return Err(format!(
                                    "Line {}: too many positional arguments",
                                    line
                                ));
                            }
                            let param_name = &func_def.params[positional_count].name;
                            final_args.insert(param_name.clone(), val);
                            positional_count += 1;
                        }
                    }

                    for param in &func_def.params {
                        if !final_args.contains_key(&param.name) {
                            if let Some(default_expr) = &param.default {
                                let default_val = self.eval_expr(default_expr)?;
                                final_args.insert(param.name.clone(), default_val);
                            } else {
                                return Err(format!(
                                    "Line {}: missing required argument '{}'",
                                    line, param.name
                                ));
                            }
                        }
                    }

                    self.env.push();
                    for param in &func_def.params {
                        let val = final_args.remove(&param.name).unwrap();
                        let _ = self.env.define(param.name.clone(), val, false);
                    }

                    let res = match self.exec_block(&func_def.body)? {
                        Signal::Return(v) => v,
                        _ => Value::Nil,
                    };
                    self.env.pop();
                    return Ok(res);
                } else {
                    return Err(format!("Line {}: '{}' is not callable", line, name));
                }
            }
        }
    }
}
