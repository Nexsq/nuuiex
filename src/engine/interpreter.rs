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
            let scan = MapVirtualKeyW(info.vk as u32, MAPVK_VK_TO_VSC) as u16;
            let use_scan = scan != 0;

            let mut base_flags = if use_scan { KEYEVENTF_SCANCODE } else { 0 };

            match info.vk {
                VK_UP | VK_DOWN | VK_LEFT | VK_RIGHT | VK_PRIOR | VK_NEXT | VK_END | VK_HOME
                | VK_INSERT | VK_DELETE | VK_DIVIDE | VK_RMENU | VK_RCONTROL => {
                    base_flags |= KEYEVENTF_EXTENDEDKEY;
                }
                _ => {}
            }

            if down {
                if info.req_shift {
                    let s_scan = MapVirtualKeyW(VK_SHIFT as u32, MAPVK_VK_TO_VSC) as u16;
                    inputs[count].r#type = INPUT_KEYBOARD;
                    inputs[count].Anonymous.ki.wVk = VK_SHIFT;
                    inputs[count].Anonymous.ki.wScan = s_scan;
                    inputs[count].Anonymous.ki.dwFlags =
                        if s_scan != 0 { KEYEVENTF_SCANCODE } else { 0 };
                    count += 1;
                }
                if info.req_ctrl {
                    let c_scan = MapVirtualKeyW(VK_CONTROL as u32, MAPVK_VK_TO_VSC) as u16;
                    inputs[count].r#type = INPUT_KEYBOARD;
                    inputs[count].Anonymous.ki.wVk = VK_CONTROL;
                    inputs[count].Anonymous.ki.wScan = c_scan;
                    inputs[count].Anonymous.ki.dwFlags =
                        if c_scan != 0 { KEYEVENTF_SCANCODE } else { 0 };
                    count += 1;
                }
                if info.req_alt {
                    let a_scan = MapVirtualKeyW(VK_MENU as u32, MAPVK_VK_TO_VSC) as u16;
                    inputs[count].r#type = INPUT_KEYBOARD;
                    inputs[count].Anonymous.ki.wVk = VK_MENU;
                    inputs[count].Anonymous.ki.wScan = a_scan;
                    inputs[count].Anonymous.ki.dwFlags =
                        if a_scan != 0 { KEYEVENTF_SCANCODE } else { 0 };
                    count += 1;
                }
            }

            inputs[count].r#type = INPUT_KEYBOARD;
            inputs[count].Anonymous.ki.wVk = info.vk;
            inputs[count].Anonymous.ki.wScan = scan;
            inputs[count].Anonymous.ki.dwFlags =
                base_flags | if down { 0 } else { KEYEVENTF_KEYUP };
            count += 1;

            if !down {
                if info.req_alt {
                    let a_scan = MapVirtualKeyW(VK_MENU as u32, MAPVK_VK_TO_VSC) as u16;
                    inputs[count].r#type = INPUT_KEYBOARD;
                    inputs[count].Anonymous.ki.wVk = VK_MENU;
                    inputs[count].Anonymous.ki.wScan = a_scan;
                    inputs[count].Anonymous.ki.dwFlags =
                        (if a_scan != 0 { KEYEVENTF_SCANCODE } else { 0 }) | KEYEVENTF_KEYUP;
                    count += 1;
                }
                if info.req_ctrl {
                    let c_scan = MapVirtualKeyW(VK_CONTROL as u32, MAPVK_VK_TO_VSC) as u16;
                    inputs[count].r#type = INPUT_KEYBOARD;
                    inputs[count].Anonymous.ki.wVk = VK_CONTROL;
                    inputs[count].Anonymous.ki.wScan = c_scan;
                    inputs[count].Anonymous.ki.dwFlags =
                        (if c_scan != 0 { KEYEVENTF_SCANCODE } else { 0 }) | KEYEVENTF_KEYUP;
                    count += 1;
                }
                if info.req_shift {
                    let s_scan = MapVirtualKeyW(VK_SHIFT as u32, MAPVK_VK_TO_VSC) as u16;
                    inputs[count].r#type = INPUT_KEYBOARD;
                    inputs[count].Anonymous.ki.wVk = VK_SHIFT;
                    inputs[count].Anonymous.ki.wScan = s_scan;
                    inputs[count].Anonymous.ki.dwFlags =
                        (if s_scan != 0 { KEYEVENTF_SCANCODE } else { 0 }) | KEYEVENTF_KEYUP;
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
                let scan = MapVirtualKeyW(info.vk as u32, MAPVK_VK_TO_VSC) as u16;
                let use_scan = scan != 0;
                let mut base_flags = if use_scan { KEYEVENTF_SCANCODE } else { 0 };

                match info.vk {
                    VK_UP | VK_DOWN | VK_LEFT | VK_RIGHT | VK_PRIOR | VK_NEXT | VK_END
                    | VK_HOME | VK_INSERT | VK_DELETE | VK_DIVIDE | VK_RMENU | VK_RCONTROL => {
                        base_flags |= KEYEVENTF_EXTENDEDKEY;
                    }
                    _ => {}
                }

                let mut inputs_down: [INPUT; 4] = std::mem::zeroed();
                let mut down_count = 0;

                if info.req_shift {
                    let s_scan = MapVirtualKeyW(VK_SHIFT as u32, MAPVK_VK_TO_VSC) as u16;
                    inputs_down[down_count].r#type = INPUT_KEYBOARD;
                    inputs_down[down_count].Anonymous.ki.wVk = VK_SHIFT;
                    inputs_down[down_count].Anonymous.ki.wScan = s_scan;
                    inputs_down[down_count].Anonymous.ki.dwFlags =
                        if s_scan != 0 { KEYEVENTF_SCANCODE } else { 0 };
                    down_count += 1;
                }
                if info.req_ctrl {
                    let c_scan = MapVirtualKeyW(VK_CONTROL as u32, MAPVK_VK_TO_VSC) as u16;
                    inputs_down[down_count].r#type = INPUT_KEYBOARD;
                    inputs_down[down_count].Anonymous.ki.wVk = VK_CONTROL;
                    inputs_down[down_count].Anonymous.ki.wScan = c_scan;
                    inputs_down[down_count].Anonymous.ki.dwFlags =
                        if c_scan != 0 { KEYEVENTF_SCANCODE } else { 0 };
                    down_count += 1;
                }
                if info.req_alt {
                    let a_scan = MapVirtualKeyW(VK_MENU as u32, MAPVK_VK_TO_VSC) as u16;
                    inputs_down[down_count].r#type = INPUT_KEYBOARD;
                    inputs_down[down_count].Anonymous.ki.wVk = VK_MENU;
                    inputs_down[down_count].Anonymous.ki.wScan = a_scan;
                    inputs_down[down_count].Anonymous.ki.dwFlags =
                        if a_scan != 0 { KEYEVENTF_SCANCODE } else { 0 };
                    down_count += 1;
                }

                inputs_down[down_count].r#type = INPUT_KEYBOARD;
                inputs_down[down_count].Anonymous.ki.wVk = info.vk;
                inputs_down[down_count].Anonymous.ki.wScan = scan;
                inputs_down[down_count].Anonymous.ki.dwFlags = base_flags;
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
                inputs_up[up_count].Anonymous.ki.wScan = scan;
                inputs_up[up_count].Anonymous.ki.dwFlags = base_flags | KEYEVENTF_KEYUP;
                up_count += 1;

                if info.req_alt {
                    let a_scan = MapVirtualKeyW(VK_MENU as u32, MAPVK_VK_TO_VSC) as u16;
                    inputs_up[up_count].r#type = INPUT_KEYBOARD;
                    inputs_up[up_count].Anonymous.ki.wVk = VK_MENU;
                    inputs_up[up_count].Anonymous.ki.wScan = a_scan;
                    inputs_up[up_count].Anonymous.ki.dwFlags =
                        (if a_scan != 0 { KEYEVENTF_SCANCODE } else { 0 }) | KEYEVENTF_KEYUP;
                    up_count += 1;
                }
                if info.req_ctrl {
                    let c_scan = MapVirtualKeyW(VK_CONTROL as u32, MAPVK_VK_TO_VSC) as u16;
                    inputs_up[up_count].r#type = INPUT_KEYBOARD;
                    inputs_up[up_count].Anonymous.ki.wVk = VK_CONTROL;
                    inputs_up[up_count].Anonymous.ki.wScan = c_scan;
                    inputs_up[up_count].Anonymous.ki.dwFlags =
                        (if c_scan != 0 { KEYEVENTF_SCANCODE } else { 0 }) | KEYEVENTF_KEYUP;
                    up_count += 1;
                }
                if info.req_shift {
                    let s_scan = MapVirtualKeyW(VK_SHIFT as u32, MAPVK_VK_TO_VSC) as u16;
                    inputs_up[up_count].r#type = INPUT_KEYBOARD;
                    inputs_up[up_count].Anonymous.ki.wVk = VK_SHIFT;
                    inputs_up[up_count].Anonymous.ki.wScan = s_scan;
                    inputs_up[up_count].Anonymous.ki.dwFlags =
                        (if s_scan != 0 { KEYEVENTF_SCANCODE } else { 0 }) | KEYEVENTF_KEYUP;
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
        if std::env::var("WAYLAND_DISPLAY").is_ok() || libc::geteuid() == 0 {
            return None;
        }

        let libx11 = dlopen(b"libX11.so.6\0".as_ptr() as *const i8, RTLD_LAZY);
        if libx11.is_null() {
            return None;
        }

        let xopen_sym = dlsym(libx11, b"XOpenDisplay\0".as_ptr() as *const i8);
        let xclose_sym = dlsym(libx11, b"XCloseDisplay\0".as_ptr() as *const i8);

        if xopen_sym.is_null() || xclose_sym.is_null() {
            dlclose(libx11);
            return None;
        }

        let xopendisplay: extern "C" fn(*const i8) -> *mut c_void = std::mem::transmute(xopen_sym);
        let xclosedisplay: extern "C" fn(*mut c_void) -> i32 = std::mem::transmute(xclose_sym);

        let display = xopendisplay(std::ptr::null());
        if display.is_null() {
            dlclose(libx11);
            return None;
        }

        if info.code >= 272 {
            let xquery_ptr_sym = dlsym(libx11, b"XQueryPointer\0".as_ptr() as *const i8);
            let xroot_sym = dlsym(libx11, b"XDefaultRootWindow\0".as_ptr() as *const i8);
            if !xquery_ptr_sym.is_null() && !xroot_sym.is_null() {
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
                ) -> i32 = std::mem::transmute(xquery_ptr_sym);
                let xdefaultrootwindow: extern "C" fn(*mut c_void) -> libc::c_ulong =
                    std::mem::transmute(xroot_sym);

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

                let is_down = match info.code {
                    272 => (mask & (1 << 8)) != 0,
                    273 => (mask & (1 << 10)) != 0,
                    274 => (mask & (1 << 9)) != 0,
                    _ => false,
                };

                return Some(is_down);
            }

            xclosedisplay(display);
            dlclose(libx11);
            return None;
        }

        let xquery_sym = dlsym(libx11, b"XQueryKeymap\0".as_ptr() as *const i8);
        if xquery_sym.is_null() {
            xclosedisplay(display);
            dlclose(libx11);
            return None;
        }
        let xquerykeymap: extern "C" fn(*mut c_void, *mut u8) -> i32 =
            std::mem::transmute(xquery_sym);

        let mut keys = [0u8; 32];
        xquerykeymap(display, keys.as_mut_ptr());
        xclosedisplay(display);
        dlclose(libx11);

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
            libc::ioctl(fd, 0x40045566, 8);

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
    if let Ok(fd) = get_or_create_uinput() {
        let mut evs: Vec<libc::input_event> = Vec::with_capacity(8);

        macro_rules! add_ev {
            ($code:expr, $val:expr) => {
                let mut ev: libc::input_event = unsafe { std::mem::zeroed() };
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

        let mut syn: libc::input_event = unsafe { std::mem::zeroed() };
        syn.type_ = 0;
        syn.code = 0;
        syn.value = 0;
        evs.push(syn);

        unsafe {
            libc::write(
                fd,
                evs.as_ptr() as *const libc::c_void,
                evs.len() * std::mem::size_of::<libc::input_event>(),
            );
        }
        return Ok(());
    }

    let action = if down { "keydown" } else { "keyup" };
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        Err("uinput failed. Run with sudo for keyboard simulation on Wayland.".into())
    } else {
        if std::process::Command::new("xdotool")
            .args([action, key])
            .output()
            .is_ok()
        {
            Ok(())
        } else {
            Err("Failed to simulate key. Run with sudo or install xdotool.".into())
        }
    }
}

#[cfg(target_os = "linux")]
fn simulate_write(text: &str) -> Result<(), String> {
    if let Ok(fd) = get_or_create_uinput() {
        for c in text.chars() {
            let key_str = match c {
                '\n' => "enter".to_string(),
                '\t' => "tab".to_string(),
                _ => c.to_string(),
            };

            if let Ok(info) = parse_linux_key(&key_str) {
                let mut evs: Vec<libc::input_event> = Vec::with_capacity(8);

                macro_rules! add_ev {
                    ($code:expr, $val:expr) => {
                        let mut ev: libc::input_event = unsafe { std::mem::zeroed() };
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

                let mut syn: libc::input_event = unsafe { std::mem::zeroed() };
                syn.type_ = 0;
                syn.code = 0;
                syn.value = 0;
                evs.push(syn);

                unsafe {
                    libc::write(
                        fd,
                        evs.as_ptr() as *const libc::c_void,
                        evs.len() * std::mem::size_of::<libc::input_event>(),
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
                        evs.len() * std::mem::size_of::<libc::input_event>(),
                    );
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        return Ok(());
    }

    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        if std::process::Command::new("ydotool")
            .args(["type", text])
            .output()
            .is_ok()
        {
            return Ok(());
        }
        Err("uinput failed. Run with sudo for write simulation on Wayland.".into())
    } else {
        if std::process::Command::new("xdotool")
            .args(["type", "--delay", "5", text])
            .output()
            .is_ok()
        {
            Ok(())
        } else {
            Err("Failed to simulate write. Run with sudo or install xdotool.".into())
        }
    }
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

use std::sync::atomic::AtomicI32;

static MOUSE_DX: AtomicI32 = AtomicI32::new(0);
static MOUSE_DY: AtomicI32 = AtomicI32::new(0);
static MOUSE_TRACKER_FAILED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
static INIT_MOUSE: std::sync::Once = std::sync::Once::new();

#[cfg(windows)]
fn start_mouse_tracker() {
    use std::thread;
    use windows_sys::Win32::UI::Input::*;
    use windows_sys::Win32::UI::WindowsAndMessaging::*;

    thread::spawn(|| unsafe {
        let class_name: Vec<u16> = "STATIC\0".encode_utf16().collect();
        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            std::ptr::null(),
            0,
            0,
            0,
            0,
            0,
            HWND_MESSAGE,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null(),
        );

        let rid = RAWINPUTDEVICE {
            usUsagePage: 0x01,
            usUsage: 0x02,
            dwFlags: RIDEV_INPUTSINK,
            hwndTarget: hwnd,
        };

        if RegisterRawInputDevices(&rid, 1, std::mem::size_of::<RAWINPUTDEVICE>() as u32) == 0 {
            return;
        }

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            if msg.message == WM_INPUT {
                let mut dw_size = 0;
                GetRawInputData(
                    msg.lParam as isize as HRAWINPUT,
                    RID_INPUT,
                    std::ptr::null_mut(),
                    &mut dw_size,
                    std::mem::size_of::<RAWINPUTHEADER>() as u32,
                );

                if dw_size > 0 {
                    let mut raw_data: Vec<u8> = vec![0; dw_size as usize];
                    if GetRawInputData(
                        msg.lParam as isize as HRAWINPUT,
                        RID_INPUT,
                        raw_data.as_mut_ptr() as *mut _,
                        &mut dw_size,
                        std::mem::size_of::<RAWINPUTHEADER>() as u32,
                    ) == dw_size
                    {
                        let raw = &*(raw_data.as_ptr() as *const RAWINPUT);
                        if raw.header.dwType == RIM_TYPEMOUSE {
                            if (raw.data.mouse.usFlags & MOUSE_MOVE_ABSOLUTE as u16) == 0 {
                                MOUSE_DX.fetch_add(raw.data.mouse.lLastX, Ordering::Relaxed);
                                MOUSE_DY.fetch_add(raw.data.mouse.lLastY, Ordering::Relaxed);
                            }
                        }
                    }
                }
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    });
}

#[cfg(target_os = "linux")]
fn start_mouse_tracker() {
    use std::fs::File;
    use std::io::Read;
    use std::thread;

    thread::spawn(|| {
        if let Ok(mut file) = File::open("/dev/input/mice") {
            let mut buf = [0u8; 3];
            loop {
                if file.read_exact(&mut buf).is_ok() {
                    let x_sign = (buf[0] & 0x10) != 0;
                    let y_sign = (buf[0] & 0x20) != 0;

                    let mut dx = buf[1] as i32;
                    if x_sign && dx != 0 {
                        dx -= 256;
                    } else if x_sign && dx == 0 {
                        dx = -256;
                    }

                    let mut dy = buf[2] as i32;
                    if y_sign && dy != 0 {
                        dy -= 256;
                    } else if y_sign && dy == 0 {
                        dy = -256;
                    }

                    dy = -dy;

                    MOUSE_DX.fetch_add(dx, Ordering::Relaxed);
                    MOUSE_DY.fetch_add(dy, Ordering::Relaxed);
                } else {
                    break;
                }
            }
        }

        let mut last_pos = get_mouse_pos().unwrap_or((0, 0));
        loop {
            thread::sleep(std::time::Duration::from_millis(5));
            if let Ok(pos) = get_mouse_pos() {
                let dx = pos.0 - last_pos.0;
                let dy = pos.1 - last_pos.1;
                if dx != 0 {
                    MOUSE_DX.fetch_add(dx, Ordering::Relaxed);
                }
                if dy != 0 {
                    MOUSE_DY.fetch_add(dy, Ordering::Relaxed);
                }
                last_pos = pos;
            }
        }
    });
}

#[cfg(not(any(windows, target_os = "linux")))]
fn start_mouse_tracker() {}

#[cfg(windows)]
fn get_mouse_pos() -> Result<(i32, i32), String> {
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
fn get_mouse_pos() -> Result<(i32, i32), String> {
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        if let Ok(output) = std::process::Command::new("hyprctl")
            .arg("cursorpos")
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let parts: Vec<&str> = stdout.trim().split(',').collect();
            if parts.len() == 2 {
                let x: i32 = parts[0].trim().parse().unwrap_or(0);
                let y: i32 = parts[1].trim().parse().unwrap_or(0);
                return Ok((x, y));
            }
        }
    }

    use libc::{RTLD_LAZY, c_void, dlclose, dlopen, dlsym};
    unsafe {
        if std::env::var("WAYLAND_DISPLAY").is_err() && libc::geteuid() != 0 {
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
fn get_mouse_pos() -> Result<(i32, i32), String> {
    Err("Mouse position is not supported on this OS".to_string())
}

#[cfg(windows)]
fn get_screen_size() -> Result<(i32, i32), String> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};
    unsafe {
        let w = GetSystemMetrics(SM_CXSCREEN);
        let h = GetSystemMetrics(SM_CYSCREEN);
        if w > 0 && h > 0 {
            Ok((w, h))
        } else {
            Err("Failed to get screen size".to_string())
        }
    }
}

#[cfg(target_os = "linux")]
fn get_screen_size() -> Result<(i32, i32), String> {
    if let Ok(output) = std::process::Command::new("xdotool")
        .arg("getdisplaygeometry")
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let parts: Vec<&str> = stdout.trim().split_whitespace().collect();
            if parts.len() >= 2 {
                if let (Ok(w), Ok(h)) = (parts[0].parse(), parts[1].parse()) {
                    return Ok((w, h));
                }
            }
        }
    }

    if let Ok(output) = std::process::Command::new("xrandr").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains('*') {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if let Some(res) = parts.first() {
                        let dims: Vec<&str> = res.split('x').collect();
                        if dims.len() == 2 {
                            if let (Ok(w), Ok(h)) = (dims[0].parse(), dims[1].parse()) {
                                return Ok((w, h));
                            }
                        }
                    }
                }
            }
        }
    }

    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        if let Ok(output) = std::process::Command::new("hyprctl")
            .arg("monitors")
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    let trimmed = line.trim();
                    if let Some(x_idx) = trimmed.find('x') {
                        if let Some(at_idx) = trimmed.find('@') {
                            if x_idx < at_idx {
                                let w_str = &trimmed[..x_idx];
                                let h_str = &trimmed[x_idx + 1..at_idx];
                                if let (Ok(w), Ok(h)) = (w_str.parse(), h_str.parse()) {
                                    return Ok((w, h));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Err("Failed to get screen size. Is xdotool or xrandr installed?".to_string())
}

#[cfg(not(any(windows, target_os = "linux")))]
fn get_screen_size() -> Result<(i32, i32), String> {
    Err("Screen size is not supported on this OS".to_string())
}

#[cfg(windows)]
fn get_focused_window() -> Result<String, String> {
    use windows_sys::Win32::Foundation::{CloseHandle, MAX_PATH};
    use windows_sys::Win32::System::Threading::{OpenProcess, QueryFullProcessImageNameW};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowThreadProcessId,
    };

    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() {
            return Ok(String::new());
        }
        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == 0 {
            return Ok(String::new());
        }

        let h_process = OpenProcess(0x1000, 0, pid);
        if h_process.is_null() {
            return Ok(String::new());
        }

        let mut buf = [0u16; MAX_PATH as usize];
        let mut size = MAX_PATH;
        let res = QueryFullProcessImageNameW(h_process, 0, buf.as_mut_ptr(), &mut size);
        CloseHandle(h_process);

        if res != 0 {
            let full_path = String::from_utf16_lossy(&buf[..size as usize]);
            let file_name = std::path::Path::new(&full_path)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            return Ok(file_name);
        }
        Ok(String::new())
    }
}

#[cfg(target_os = "linux")]
fn get_focused_window() -> Result<String, String> {
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        if let Ok(output) = std::process::Command::new("hyprctl")
            .arg("activewindow")
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("class:") || trimmed.starts_with("initialClass:") {
                    if let Some((_, val)) = trimmed.split_once(':') {
                        return Ok(val.trim().to_string());
                    }
                }
            }
        }
    }

    if let Ok(output) = std::process::Command::new("xdotool")
        .args(["getwindowfocus", "getwindowpid"])
        .output()
    {
        if output.status.success() {
            let pid_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if let Ok(pid) = pid_str.parse::<u32>() {
                if let Ok(comm) = std::fs::read_to_string(format!("/proc/{}/comm", pid)) {
                    return Ok(comm.trim().to_string());
                }
            }
        }
    }

    if let Ok(output) = std::process::Command::new("xdotool")
        .args(["getwindowfocus", "getwindowname"])
        .output()
    {
        if output.status.success() {
            return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
        }
    }

    Ok(String::new())
}

#[cfg(not(any(windows, target_os = "linux")))]
fn get_focused_window() -> Result<String, String> {
    Err("Focused window detection is not supported on this OS".to_string())
}

#[cfg(windows)]
fn get_screen_pixel(x: i32, y: i32) -> Result<(u8, u8, u8), String> {
    #[link(name = "user32")]
    unsafe extern "system" {
        fn GetDC(hWnd: isize) -> isize;
        fn ReleaseDC(hWnd: isize, hDC: isize) -> i32;
    }
    #[link(name = "gdi32")]
    unsafe extern "system" {
        fn GetPixel(hDC: isize, x: i32, y: i32) -> u32;
    }

    struct ScreenDC(isize);
    impl Drop for ScreenDC {
        fn drop(&mut self) {
            unsafe {
                ReleaseDC(0, self.0);
            }
        }
    }

    thread_local! {
        static DC: std::cell::RefCell<Option<ScreenDC>> = std::cell::RefCell::new(None);
    }

    DC.with(|dc_cell| {
        let mut dc_opt = dc_cell.borrow_mut();
        if dc_opt.is_none() {
            let hdc = unsafe { GetDC(0) };
            if hdc == 0 {
                return Err("Failed to get screen DC".into());
            }
            *dc_opt = Some(ScreenDC(hdc));
        }

        let hdc = dc_opt.as_ref().unwrap().0;
        let color = unsafe { GetPixel(hdc, x, y) };
        if color == 0xFFFFFFFF {
            return Err("Coordinates out of bounds".into());
        }

        let r = (color & 0xFF) as u8;
        let g = ((color >> 8) & 0xFF) as u8;
        let b = ((color >> 16) & 0xFF) as u8;
        Ok((r, g, b))
    })
}

#[cfg(target_os = "linux")]
#[repr(C)]
struct XImageFuncs {
    create_image: *const libc::c_void,
    destroy_image: extern "C" fn(*mut XImage) -> i32,
    get_pixel: extern "C" fn(*mut XImage, i32, i32) -> libc::c_ulong,
    put_pixel: *const libc::c_void,
    sub_image: *const libc::c_void,
    add_pixel: *const libc::c_void,
}

#[cfg(target_os = "linux")]
#[repr(C)]
struct XImage {
    pub width: i32,
    pub height: i32,
    pub xoffset: i32,
    pub format: i32,
    pub data: *mut u8,
    pub byte_order: i32,
    pub bitmap_unit: i32,
    pub bitmap_bit_order: i32,
    pub bitmap_pad: i32,
    pub depth: i32,
    pub bytes_per_line: i32,
    pub bits_per_pixel: i32,
    pub red_mask: libc::c_ulong,
    pub green_mask: libc::c_ulong,
    pub blue_mask: libc::c_ulong,
    pub obdata: *mut libc::c_void,
    pub f: XImageFuncs,
}

#[cfg(target_os = "linux")]
struct X11PixelContext {
    lib: *mut libc::c_void,
    display: *mut libc::c_void,
    root: libc::c_ulong,
    xgetimage: extern "C" fn(
        *mut libc::c_void,
        libc::c_ulong,
        i32,
        i32,
        u32,
        u32,
        libc::c_ulong,
        i32,
    ) -> *mut XImage,
}

#[cfg(target_os = "linux")]
impl Drop for X11PixelContext {
    fn drop(&mut self) {
        unsafe {
            let xclose_sym = libc::dlsym(self.lib, b"XCloseDisplay\0".as_ptr() as *const i8);
            if !xclose_sym.is_null() {
                let xclosedisplay: extern "C" fn(*mut libc::c_void) -> i32 =
                    std::mem::transmute(xclose_sym);
                xclosedisplay(self.display);
            }
            libc::dlclose(self.lib);
        }
    }
}

#[cfg(target_os = "linux")]
fn get_screen_pixel(x: i32, y: i32) -> Result<(u8, u8, u8), String> {
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        return Err(
            "getpixel is not supported on Wayland (requires compositor-specific tools)."
                .to_string(),
        );
    }
    unsafe {
        if libc::geteuid() == 0 {
            return Err("Cannot use getpixel as root due to XAUTHORITY restrictions.".to_string());
        }
    }

    use libc::{RTLD_LAZY, dlclose, dlopen, dlsym};

    thread_local! {
        static X11_PIXEL_CTX: std::cell::RefCell<Option<Result<X11PixelContext, String>>> = std::cell::RefCell::new(None);
    }

    X11_PIXEL_CTX.with(|ctx_cell| {
        let mut ctx_opt = ctx_cell.borrow_mut();
        if ctx_opt.is_none() {
            unsafe {
                let lib = dlopen(b"libX11.so.6\0".as_ptr() as *const i8, RTLD_LAZY);
                if lib.is_null() {
                    let err = "Failed to load libX11.so.6".to_string();
                    *ctx_opt = Some(Err(err.clone()));
                    return Err(err);
                }
                let xopen = dlsym(lib, b"XOpenDisplay\0".as_ptr() as *const i8);
                let xgetimage = dlsym(lib, b"XGetImage\0".as_ptr() as *const i8);
                let xroot = dlsym(lib, b"XDefaultRootWindow\0".as_ptr() as *const i8);

                if xopen.is_null() || xgetimage.is_null() || xroot.is_null() {
                    dlclose(lib);
                    let err = "Failed to load X11 symbols".to_string();
                    *ctx_opt = Some(Err(err.clone()));
                    return Err(err);
                }

                let xopendisplay: extern "C" fn(*const i8) -> *mut libc::c_void =
                    std::mem::transmute(xopen);
                let xdefaultrootwindow: extern "C" fn(*mut libc::c_void) -> libc::c_ulong =
                    std::mem::transmute(xroot);

                let display = xopendisplay(std::ptr::null());
                if display.is_null() {
                    dlclose(lib);
                    let err = "Failed to open X display".to_string();
                    *ctx_opt = Some(Err(err.clone()));
                    return Err(err);
                }

                let root = xdefaultrootwindow(display);
                let xgetimage_fn: extern "C" fn(
                    *mut libc::c_void,
                    libc::c_ulong,
                    i32,
                    i32,
                    u32,
                    u32,
                    libc::c_ulong,
                    i32,
                ) -> *mut XImage = std::mem::transmute(xgetimage);

                *ctx_opt = Some(Ok(X11PixelContext {
                    lib,
                    display,
                    root,
                    xgetimage: xgetimage_fn,
                }));
            }
        }

        match ctx_opt.as_ref().unwrap() {
            Ok(x11) => unsafe {
                let image = (x11.xgetimage)(x11.display, x11.root, x, y, 1, 1, !0, 2);
                if image.is_null() {
                    return Err("Coordinates out of bounds or invalid drawable".into());
                }

                let pixel = ((*image).f.get_pixel)(image, 0, 0);

                let rm = (*image).red_mask;
                let gm = (*image).green_mask;
                let bm = (*image).blue_mask;

                let r_shift = rm.trailing_zeros();
                let r_max = rm >> r_shift;
                let r = if r_max > 0 {
                    (((pixel & rm) >> r_shift) * 255 / r_max) as u8
                } else {
                    0
                };

                let g_shift = gm.trailing_zeros();
                let g_max = gm >> g_shift;
                let g = if g_max > 0 {
                    (((pixel & gm) >> g_shift) * 255 / g_max) as u8
                } else {
                    0
                };

                let b_shift = bm.trailing_zeros();
                let b_max = bm >> b_shift;
                let b = if b_max > 0 {
                    (((pixel & bm) >> b_shift) * 255 / b_max) as u8
                } else {
                    0
                };

                ((*image).f.destroy_image)(image);

                Ok((r, g, b))
            },
            Err(e) => Err(e.clone()),
        }
    })
}

#[cfg(not(any(windows, target_os = "linux")))]
fn get_screen_pixel(_x: i32, _y: i32) -> Result<(u8, u8, u8), String> {
    Err("getpixel is not supported on this OS".to_string())
}

#[cfg(windows)]
fn set_mouse_pos(x: i32, y: i32, relative: bool) -> Result<(), String> {
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
fn set_mouse_pos(x: i32, y: i32, relative: bool) -> Result<(), String> {
    if relative {
        if let Ok(fd) = get_or_create_uinput() {
            let mut evs: Vec<libc::input_event> = Vec::with_capacity(3);
            macro_rules! add_ev {
                ($type:expr, $code:expr, $val:expr) => {
                    let mut ev: libc::input_event = unsafe { std::mem::zeroed() };
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
                    evs.len() * std::mem::size_of::<libc::input_event>(),
                );
            }
            return Ok(());
        }

        if std::env::var("WAYLAND_DISPLAY").is_ok() {
            if let Ok(output) = std::process::Command::new("ydotool")
                .args(["mousemove", "-x", &x.to_string(), "-y", &y.to_string()])
                .output()
            {
                if output.status.success() {
                    return Ok(());
                }
            }
            Err("Failed to set relative mouse pos. Run with sudo or install ydotool.".to_string())
        } else {
            if let Ok(output) = std::process::Command::new("xdotool")
                .args(["mousemove_relative", "--", &x.to_string(), &y.to_string()])
                .output()
            {
                if output.status.success() {
                    return Ok(());
                }
            }
            Err("Failed to set relative mouse pos. Run with sudo or install xdotool.".to_string())
        }
    } else {
        if std::env::var("WAYLAND_DISPLAY").is_ok() {
            if let Ok(output) = std::process::Command::new("ydotool")
                .args(["mousemove", "-a", &x.to_string(), &y.to_string()])
                .output()
            {
                if output.status.success() {
                    return Ok(());
                }
            }
            return Err("Absolute cursor positioning on Wayland requires ydotool".to_string());
        }

        use libc::{RTLD_LAZY, c_void, dlclose, dlopen, dlsym};
        unsafe {
            if libc::geteuid() == 0 {
                return Err(
                    "Cannot use set_mouse_pos as root due to XAUTHORITY restrictions.".to_string(),
                );
            }
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
fn set_mouse_pos(_x: i32, _y: i32, _relative: bool) -> Result<(), String> {
    Err("Setting mouse position is not supported on this OS".to_string())
}

#[cfg(windows)]
fn simulate_scroll(num: i32) -> Result<(), String> {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        INPUT, INPUT_MOUSE, MOUSEEVENTF_WHEEL, SendInput,
    };
    unsafe {
        let mut input: INPUT = std::mem::zeroed();
        input.r#type = INPUT_MOUSE;
        input.Anonymous.mi.mouseData = (num * 120) as u32;
        input.Anonymous.mi.dwFlags = MOUSEEVENTF_WHEEL;
        SendInput(1, &input, std::mem::size_of::<INPUT>() as i32);
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn simulate_scroll(num: i32) -> Result<(), String> {
    let fd = get_or_create_uinput()?;
    let mut evs: Vec<libc::input_event> = Vec::with_capacity(2);
    macro_rules! add_ev {
        ($type:expr, $code:expr, $val:expr) => {
            let mut ev: libc::input_event = unsafe { std::mem::zeroed() };
            ev.type_ = $type;
            ev.code = $code;
            ev.value = $val;
            evs.push(ev);
        };
    }
    add_ev!(2, 8, num);
    add_ev!(0, 0, 0);

    unsafe {
        libc::write(
            fd,
            evs.as_ptr() as *const libc::c_void,
            evs.len() * std::mem::size_of::<libc::input_event>(),
        );
    }
    Ok(())
}

#[cfg(not(any(windows, target_os = "linux")))]
fn simulate_scroll(_num: i32) -> Result<(), String> {
    Err("Scrolling is not supported on this OS".to_string())
}

#[cfg(windows)]
fn system_beep(freq: u32, duration: u32) {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn Beep(dwFreq: u32, dwDuration: u32) -> i32;
    }
    unsafe {
        Beep(freq, duration);
    }
}

#[cfg(target_os = "linux")]
fn system_beep(freq: u32, duration: u32) {
    let dur_sec = duration as f32 / 1000.0;

    if std::process::Command::new("play")
        .args([
            "-n",
            "-c1",
            "synth",
            &dur_sec.to_string(),
            "sine",
            &freq.to_string(),
        ])
        .stderr(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        return;
    }

    let lavfi = format!("sine=frequency={}:duration={}", freq, dur_sec);
    if std::process::Command::new("ffplay")
        .args(["-f", "lavfi", "-i", &lavfi, "-autoexit", "-nodisp"])
        .stderr(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        return;
    }

    if std::process::Command::new("beep")
        .args(["-f", &freq.to_string(), "-l", &duration.to_string()])
        .stderr(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        return;
    }

    print!("\x07");
    let _ = std::io::Write::flush(&mut std::io::stdout());
    std::thread::sleep(std::time::Duration::from_millis(duration as u64));
}

#[cfg(not(any(windows, target_os = "linux")))]
fn system_beep(_freq: u32, duration: u32) {
    print!("\x07");
    let _ = std::io::Write::flush(&mut std::io::stdout());
    std::thread::sleep(std::time::Duration::from_millis(duration as u64));
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
        env.define(
            "Background".to_string(),
            Value::BuiltinEnum("Background".to_string()),
            true,
        )
        .unwrap();
        env.define(
            "Modifier".to_string(),
            Value::BuiltinEnum("Modifier".to_string()),
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
    pub caret: Arc<std::sync::Mutex<(usize, usize)>>,
    tx: SyncSender<crate::EngineMessage>,
    input_rx: Receiver<String>,
    pub should_exit: bool,
    pub env: Environment,
    cancel_token: Arc<AtomicBool>,
    focus_token: Arc<AtomicBool>,
    rng_state: u64,
    macro_rel_path: String,
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
        macro_rel_path: String,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap();
        let rng_state = now.as_secs() ^ (now.subsec_nanos() as u64).wrapping_shl(32);

        Self {
            output: Arc::new(std::sync::Mutex::new(Vec::new())),
            errors: Arc::new(std::sync::Mutex::new(Vec::new())),
            caret: Arc::new(std::sync::Mutex::new((0, 0))),
            tx,
            input_rx,
            should_exit: false,
            env: Environment::new(),
            cancel_token,
            focus_token,
            rng_state,
            macro_rel_path,
        }
    }

    fn write_to_output(&mut self, text: &str) {
        let mut out = self.output.lock().unwrap();
        let mut caret = self.caret.lock().unwrap();

        for c in text.chars() {
            if c == '\n' {
                caret.0 = 0;
                caret.1 += 1;
            } else {
                while out.len() <= caret.1 {
                    out.push(String::new());
                }
                let line = &mut out[caret.1];
                let char_count = line.chars().count();
                if caret.0 < char_count {
                    let byte_idx = line.char_indices().nth(caret.0).unwrap().0;
                    let next_byte_idx = line
                        .char_indices()
                        .nth(caret.0 + 1)
                        .map(|x| x.0)
                        .unwrap_or(line.len());
                    let mut buf = [0; 4];
                    line.replace_range(byte_idx..next_byte_idx, c.encode_utf8(&mut buf));
                } else {
                    let spaces = caret.0 - char_count;
                    for _ in 0..spaces {
                        line.push(' ');
                    }
                    line.push(c);
                }
                caret.0 += 1;
            }
        }
    }

    fn parse_range_args(
        &self,
        eval_args: &[(Option<String>, Value)],
        func_name: &str,
        line: usize,
    ) -> Result<(f64, f64, f64), String> {
        match eval_args.len() {
            1 => {
                if let Value::Number(stop) = eval_args[0].1 {
                    Ok((0.0, stop, 1.0))
                } else {
                    Err(format!("Line {}: '{}' expects numbers", line, func_name))
                }
            }
            2 => {
                if let (Value::Number(start), Value::Number(stop)) =
                    (&eval_args[0].1, &eval_args[1].1)
                {
                    Ok((*start, *stop, 1.0))
                } else {
                    Err(format!("Line {}: '{}' expects numbers", line, func_name))
                }
            }
            3 => {
                if let (Value::Number(start), Value::Number(stop), Value::Number(step)) =
                    (&eval_args[0].1, &eval_args[1].1, &eval_args[2].1)
                {
                    if *step == 0.0 {
                        Err(format!(
                            "Line {}: '{}' step cannot be zero",
                            line, func_name
                        ))
                    } else {
                        Ok((*start, *stop, *step))
                    }
                } else {
                    Err(format!("Line {}: '{}' expects numbers", line, func_name))
                }
            }
            _ => Err(format!(
                "Line {}: '{}' expects 1 to 3 arguments",
                line, func_name
            )),
        }
    }

    fn send_output(&mut self) {
        let mut out_lock = self.output.lock().unwrap();
        let err_lock = self.errors.lock().unwrap();
        let mut caret = self.caret.lock().unwrap();

        if out_lock.len() > 1000 {
            let excess = out_lock.len() - 1000;
            out_lock.drain(0..excess);
            caret.1 = caret.1.saturating_sub(excess);
        }

        while out_lock.len() <= caret.1 {
            out_lock.push(String::new());
        }

        let mut res = out_lock.clone();

        let mut cx = caret.0;
        let mut cy = caret.1;

        if !err_lock.is_empty() {
            if res.last().map(|s| s.is_empty()).unwrap_or(false) {
                res.pop();
            }

            res.push("Runtime Errors:".to_string());
            res.extend(err_lock.clone());

            cx = 0;
            cy = res.len().saturating_sub(1);
        }

        let send_res = res.clone();
        drop(caret);
        drop(out_lock);
        drop(err_lock);

        if self
            .tx
            .send(crate::EngineMessage::Output(send_res, cx, cy))
            .is_err()
        {
            self.should_exit = true;
            self.cancel_token.store(true, Ordering::Relaxed);
        }
    }

    pub fn exec(&mut self, stmts: &[Stmt]) {
        let _ = self.exec_block(stmts);
        self.send_output();
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
                    self.cancel_token.store(true, Ordering::Relaxed);
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
                        if n.fract() != 0.0 {
                            return Err(format!("Line {}: List index must be an integer", line));
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
                        if n.fract() != 0.0 {
                            return Err(format!("Line {}: List index must be an integer", line));
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
    ) -> Result<(Value, bool), String> {
        if let Value::List(vec) = val {
            match method {
                "len" => {
                    if args.len() != 0 {
                        return Err(format!("Line {}: 'len' expects 0 arguments", line));
                    }
                    Ok((Value::Number(vec.len() as f64), false))
                }
                "append" => {
                    if args.len() != 1 {
                        return Err(format!("Line {}: 'append' expects 1 argument", line));
                    }
                    vec.push(args[0].clone());
                    Ok((Value::List(vec.clone()), true))
                }
                "clear" => {
                    if args.len() != 0 {
                        return Err(format!("Line {}: 'clear' expects 0 arguments", line));
                    }
                    vec.clear();
                    Ok((Value::List(vec.clone()), true))
                }
                "count" => {
                    if args.len() != 1 {
                        return Err(format!("Line {}: 'count' expects 1 argument", line));
                    }
                    let target = &args[0];
                    let count = vec.iter().filter(|&x| x == target).count();
                    Ok((Value::Number(count as f64), false))
                }
                "extend" => {
                    if args.len() != 1 {
                        return Err(format!("Line {}: 'extend' expects 1 argument", line));
                    }
                    if let Value::List(other) = &args[0] {
                        vec.extend(other.clone());
                        Ok((Value::List(vec.clone()), true))
                    } else {
                        Err(format!("Line {}: 'extend' expects a list", line))
                    }
                }
                "index" => {
                    if args.len() != 1 {
                        return Err(format!("Line {}: 'index' expects 1 argument", line));
                    }
                    if let Some(pos) = vec.iter().position(|x| x == &args[0]) {
                        Ok((Value::Number(pos as f64), false))
                    } else {
                        Ok((Value::Nil, false))
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
                        if pos.fract() != 0.0 {
                            return Err(format!("Line {}: Position must be an integer", line));
                        }
                        let idx = pos as usize;
                        if idx <= vec.len() {
                            vec.insert(idx, element);
                            Ok((Value::List(vec.clone()), true))
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
                            Ok((popped, true))
                        } else {
                            Err(format!("Line {}: pop from empty list", line))
                        }
                    } else if args.len() == 1 {
                        if let Value::Number(pos) = args[0] {
                            if pos < 0.0 {
                                return Err(format!("Line {}: Position cannot be negative", line));
                            }
                            if pos.fract() != 0.0 {
                                return Err(format!("Line {}: Position must be an integer", line));
                            }
                            let idx = pos as usize;
                            if idx < vec.len() {
                                Ok((vec.remove(idx), true))
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
                    Ok((Value::List(vec.clone()), true))
                }
                _ => Err(format!("Line {}: Undefined list method '{}'", line, method)),
            }
        } else if let Value::Dict(map) = val {
            match method {
                "len" => {
                    if args.len() != 0 {
                        return Err(format!("Line {}: 'len' expects 0 arguments", line));
                    }
                    Ok((Value::Number(map.len() as f64), false))
                }
                "clear" => {
                    if args.len() != 0 {
                        return Err(format!("Line {}: 'clear' expects 0 arguments", line));
                    }
                    map.clear();
                    Ok((Value::Dict(map.clone()), true))
                }
                "get" => {
                    if args.len() != 1 && args.len() != 2 {
                        return Err(format!("Line {}: 'get' expects 1 or 2 arguments", line));
                    }
                    if let Some(v) = map.get(&args[0]) {
                        Ok((v.clone(), false))
                    } else if args.len() == 2 {
                        Ok((args[1].clone(), false))
                    } else {
                        Ok((Value::Nil, false))
                    }
                }
                "keys" => {
                    if args.len() != 0 {
                        return Err(format!("Line {}: 'keys' expects 0 arguments", line));
                    }
                    let keys: Vec<Value> = map.keys().cloned().collect();
                    Ok((Value::List(keys), false))
                }
                "values" => {
                    if args.len() != 0 {
                        return Err(format!("Line {}: 'values' expects 0 arguments", line));
                    }
                    let vals: Vec<Value> = map.values().cloned().collect();
                    Ok((Value::List(vals), false))
                }
                "pop" => {
                    if args.len() == 0 {
                        let key = map.keys().next().cloned();
                        if let Some(k) = key {
                            let v = map.remove(&k).unwrap();
                            Ok((Value::List(vec![k, v]), true))
                        } else {
                            Err(format!("Line {}: pop from empty dict", line))
                        }
                    } else if args.len() == 1 {
                        if let Some(v) = map.remove(&args[0]) {
                            Ok((v, true))
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
                        Ok((Value::Dict(map.clone()), true))
                    } else {
                        Err(format!("Line {}: 'update' expects a dict", line))
                    }
                }
                "set" => {
                    if args.len() != 2 {
                        return Err(format!("Line {}: 'set' expects 2 arguments", line));
                    }
                    map.insert(args[0].clone(), args[1].clone());
                    Ok((Value::Dict(map.clone()), true))
                }
                _ => Err(format!("Line {}: Undefined dict method '{}'", line, method)),
            }
        } else if let Value::String(s) = val {
            match method {
                "len" => {
                    if args.len() != 0 {
                        return Err(format!("Line {}: 'len' expects 0 arguments", line));
                    }
                    Ok((Value::Number(s.chars().count() as f64), false))
                }
                "capitalize" => {
                    if args.len() != 0 {
                        return Err(format!("Line {}: 'capitalize' expects 0 arguments", line));
                    }
                    let res = if let Some(f) = s.chars().next() {
                        f.to_uppercase().collect::<String>() + &s[f.len_utf8()..]
                    } else {
                        String::new()
                    };
                    Ok((Value::String(res), false))
                }
                "lower" => {
                    if args.len() != 0 {
                        return Err(format!("Line {}: 'lower' expects 0 arguments", line));
                    }
                    Ok((Value::String(s.to_lowercase()), false))
                }
                "upper" => {
                    if args.len() != 0 {
                        return Err(format!("Line {}: 'upper' expects 0 arguments", line));
                    }
                    Ok((Value::String(s.to_uppercase()), false))
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
                    Ok((Value::String(res), false))
                }
                "count" => {
                    if args.len() != 1 {
                        return Err(format!("Line {}: 'count' expects 1 argument", line));
                    }
                    if let Value::String(sub) = &args[0] {
                        Ok((Value::Number(s.matches(sub).count() as f64), false))
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
                            Ok((Value::Number(char_idx as f64), false))
                        } else {
                            Ok((Value::Nil, false))
                        }
                    } else {
                        Err(format!("Line {}: 'index' expects a string", line))
                    }
                }
                "trim" => {
                    if args.len() != 0 {
                        return Err(format!("Line {}: 'trim' expects 0 arguments", line));
                    }
                    Ok((Value::String(s.trim().to_string()), false))
                }
                "join" => {
                    if args.len() != 1 {
                        return Err(format!("Line {}: 'join' expects 1 argument", line));
                    }
                    if let Value::List(l) = &args[0] {
                        let strings: Vec<String> = l.iter().map(|v| v.to_string()).collect();
                        Ok((Value::String(strings.join(s)), false))
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
                        Ok((Value::List(parts), false))
                    } else if args.len() == 1 {
                        if let Value::String(sep) = &args[0] {
                            let parts: Vec<Value> =
                                s.split(sep).map(|p| Value::String(p.to_string())).collect();
                            Ok((Value::List(parts), false))
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
                        Ok((Value::String(s.replace(old, new)), false))
                    } else {
                        Err(format!("Line {}: 'replace' expects string arguments", line))
                    }
                }
                "startswith" => {
                    if args.len() != 1 {
                        return Err(format!("Line {}: 'startswith' expects 1 argument", line));
                    }
                    if let Value::String(sub) = &args[0] {
                        Ok((Value::Bool(s.starts_with(sub)), false))
                    } else {
                        Err(format!("Line {}: 'startswith' expects a string", line))
                    }
                }
                "endswith" => {
                    if args.len() != 1 {
                        return Err(format!("Line {}: 'endswith' expects 1 argument", line));
                    }
                    if let Value::String(sub) = &args[0] {
                        Ok((Value::Bool(s.ends_with(sub)), false))
                    } else {
                        Err(format!("Line {}: 'endswith' expects a string", line))
                    }
                }
                "asnum" => {
                    if args.len() != 0 {
                        return Err(format!("Line {}: 'asnum' expects 0 arguments", line));
                    }
                    match s.trim().parse::<f64>() {
                        Ok(num) => Ok((Value::Number(num), false)),
                        Err(_) => Ok((Value::Nil, false)),
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
                    Ok((Value::Number(n.abs()), false))
                }
                "neg" => {
                    if args.len() != 0 {
                        return Err(format!("Line {}: 'neg' expects 0 arguments", line));
                    }
                    Ok((Value::Number(-n), false))
                }
                "floor" => {
                    if args.len() != 0 {
                        return Err(format!("Line {}: 'floor' expects 0 arguments", line));
                    }
                    Ok((Value::Number(n.floor()), false))
                }
                "trunc" => {
                    if args.len() != 0 {
                        return Err(format!("Line {}: 'trunc' expects 0 arguments", line));
                    }
                    Ok((Value::Number(n.trunc()), false))
                }
                "ceil" => {
                    if args.len() != 0 {
                        return Err(format!("Line {}: 'ceil' expects 0 arguments", line));
                    }
                    Ok((Value::Number(n.ceil()), false))
                }
                "fract" => {
                    if args.len() != 0 {
                        return Err(format!("Line {}: 'fract' expects 0 arguments", line));
                    }
                    Ok((Value::Number(n.fract()), false))
                }
                "clamp" => {
                    if args.len() != 2 {
                        return Err(format!("Line {}: 'clamp' expects 2 arguments", line));
                    }
                    if let (Value::Number(min), Value::Number(max)) = (&args[0], &args[1]) {
                        Ok((Value::Number(n.clamp(*min, *max)), false))
                    } else {
                        Err(format!("Line {}: 'clamp' expects numbers", line))
                    }
                }
                "round" => {
                    if args.len() == 0 {
                        Ok((Value::Number(n.round()), false))
                    } else if args.len() == 1 {
                        if let Value::Number(places) = &args[0] {
                            let factor = 10.0_f64.powf(*places);
                            Ok((Value::Number((n * factor).round() / factor), false))
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
                        Ok((Value::Number(n.powf(*exp)), false))
                    } else {
                        Err(format!("Line {}: 'pow' expects a number", line))
                    }
                }
                "sqrt" => {
                    if args.len() != 0 {
                        return Err(format!("Line {}: 'sqrt' expects 0 arguments", line));
                    }
                    Ok((Value::Number(n.sqrt()), false))
                }
                _ => Err(format!(
                    "Line {}: Undefined number method '{}'",
                    line, method
                )),
            }
        } else if let Value::EnumVariant(enum_name, variant, _) = val {
            if (enum_name == "Color" || enum_name == "Background") && method == "tostring" {
                if args.len() != 0 {
                    return Err(format!("Line {}: 'tostring' expects 0 arguments", line));
                }
                let mut s = variant.clone();
                if s.len() == 6 && s.chars().all(|c| c.is_ascii_hexdigit()) {
                    s = s.to_lowercase();
                }
                return Ok((Value::String(s), false));
            } else {
                return Err(format!(
                    "Line {}: Undefined method '{}' for enum variant",
                    line, method
                ));
            }
        } else {
            Err(format!("Line {}: Method call on an invalid value", line))
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
                let caret_clone = Arc::clone(&self.caret);
                let next_rng_state = self.rng_state.wrapping_add(1);
                let macro_rel_path_clone = self.macro_rel_path.clone();

                std::thread::spawn(move || {
                    let (_, dummy_rx) = std::sync::mpsc::channel();
                    let mut async_interp = Interpreter {
                        output: output_clone,
                        errors: errors_clone,
                        caret: caret_clone,
                        tx: tx_clone,
                        input_rx: dummy_rx,
                        should_exit: false,
                        env: async_env,
                        cancel_token: cancel_token_clone,
                        focus_token: focus_token_clone,
                        rng_state: next_rng_state,
                        macro_rel_path: macro_rel_path_clone,
                    };

                    let _ = async_interp.exec_block(&body_clone);
                    async_interp.send_output();
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
                        if !crate::engine::core::is_valid_key_variant(&prop) {
                            return Err(format!(
                                "Line {}: Invalid variant '{}' for enum 'Key'",
                                line, prop
                            ));
                        }
                    } else if enum_name == "Color" || enum_name == "Background" {
                        if crate::theme::themecore::parse_color(&prop).is_err() {
                            return Err(format!("Line {}: Invalid color variant '{}'", line, prop));
                        }
                    } else if enum_name == "Modifier" {
                        if !crate::engine::core::is_valid_modifier_variant(&prop) {
                            return Err(format!(
                                "Line {}: Invalid variant '{}' for enum 'Modifier'",
                                line, prop
                            ));
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
                        if n.fract() != 0.0 {
                            return Err(format!("Line {}: List index must be an integer", line));
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
                        if !crate::engine::core::is_valid_key_variant(&method) {
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
                    } else if enum_name == "Color" || enum_name == "Background" {
                        if crate::theme::themecore::parse_color(&method).is_err() {
                            return Err(format!(
                                "Line {}: Invalid color variant '{}'",
                                line, method
                            ));
                        }
                        if !eval_args.is_empty() {
                            return Err(format!(
                                "Line {}: {} variant does not take arguments",
                                line, enum_name
                            ));
                        }
                        return Ok(Value::EnumVariant(enum_name.clone(), method.clone(), None));
                    } else if enum_name == "Modifier" {
                        if !crate::engine::core::is_valid_modifier_variant(&method) {
                            return Err(format!(
                                "Line {}: Invalid variant '{}' for enum 'Modifier'",
                                line, method
                            ));
                        }
                        if !eval_args.is_empty() {
                            return Err(format!(
                                "Line {}: Modifier variant does not take arguments",
                                line
                            ));
                        }
                        return Ok(Value::EnumVariant(enum_name.clone(), method.clone(), None));
                    }
                }

                let (res, is_mutated) =
                    self.apply_method(&mut left_val, method, eval_args, *line)?;
                if is_mutated {
                    self.try_assign_expr(left, left_val)?;
                }
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

                        if name == "println" {
                            combined.push('\n');
                        }

                        self.write_to_output(&combined);

                        self.send_output();
                        return Ok(Value::Nil);
                    }
                    "sleep" | "sleepaccurate" => {
                        if eval_args.len() != 1 {
                            return Err(format!(
                                "Line {}: '{}' expects exactly 1 argument",
                                line, name
                            ));
                        }
                        if let Value::Number(ms) = eval_args[0].1 {
                            if ms < 0.0 {
                                return Err(format!(
                                    "Line {}: '{}' time cannot be negative",
                                    line, name
                                ));
                            }

                            let dur = std::time::Duration::from_secs_f64(ms / 1000.0);
                            let target = std::time::Instant::now() + dur;

                            loop {
                                if self.cancel_token.load(Ordering::Relaxed) {
                                    self.should_exit = true;
                                    return Ok(Value::Nil);
                                }

                                let now = std::time::Instant::now();
                                if now >= target {
                                    break;
                                }

                                let remaining = target - now;

                                if name == "sleepaccurate" {
                                    if remaining > std::time::Duration::from_millis(2) {
                                        let safe_sleep = remaining
                                            .saturating_sub(std::time::Duration::from_millis(2));
                                        let chunk =
                                            safe_sleep.min(std::time::Duration::from_millis(10));
                                        std::thread::sleep(chunk);
                                    } else {
                                        std::hint::spin_loop();
                                    }
                                } else {
                                    let chunk = remaining.min(std::time::Duration::from_millis(10));
                                    std::thread::sleep(chunk);
                                }
                            }
                            return Ok(Value::Nil);
                        } else {
                            return Err(format!("Line {}: '{}' expects a number", line, name));
                        }
                    }
                    "exit" => {
                        if eval_args.len() != 0 {
                            return Err(format!("Line {}: 'exit' expects 0 arguments", line));
                        }
                        self.should_exit = true;
                        self.cancel_token.store(true, Ordering::Relaxed);
                        return Ok(Value::Nil);
                    }
                    "range" => {
                        let (start, stop, step) =
                            self.parse_range_args(&eval_args, "range", *line)?;

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
                    "random" => {
                        let (start, stop, step) =
                            self.parse_range_args(&eval_args, "random", *line)?;

                        let steps = if step > 0.0 {
                            ((stop - start) / step).ceil()
                        } else {
                            ((stop - start) / step).ceil()
                        };

                        if steps <= 0.0 {
                            return Err(format!("Line {}: 'random' empty range", line));
                        }

                        self.rng_state = self
                            .rng_state
                            .wrapping_mul(6364136223846793005)
                            .wrapping_add(1);
                        let r = (self.rng_state >> 11) as f64 / (1u64 << 53) as f64;
                        let choice = (r * steps).floor();

                        return Ok(Value::Number(start + choice * step));
                    }
                    "input" => {
                        if !eval_args.is_empty() {
                            let prompt = eval_args[0].1.to_string();
                            self.write_to_output(&prompt);
                        }
                        self.send_output();

                        if self.tx.send(crate::EngineMessage::InputRequest).is_err() {
                            self.should_exit = true;
                            self.cancel_token.store(true, Ordering::Relaxed);
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

                        self.write_to_output(&result);

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
                    "mousex" => {
                        if eval_args.len() != 0 {
                            return Err(format!(
                                "Line {}: 'mousex' expects exactly 0 arguments",
                                line
                            ));
                        }
                        let (x, _) =
                            get_mouse_pos().map_err(|e| format!("Line {}: {}", line, e))?;
                        return Ok(Value::Number(x as f64));
                    }
                    "mousey" => {
                        if eval_args.len() != 0 {
                            return Err(format!(
                                "Line {}: 'mousey' expects exactly 0 arguments",
                                line
                            ));
                        }
                        let (_, y) =
                            get_mouse_pos().map_err(|e| format!("Line {}: {}", line, e))?;
                        return Ok(Value::Number(y as f64));
                    }
                    "mousedelta" => {
                        INIT_MOUSE.call_once(start_mouse_tracker);
                        if MOUSE_TRACKER_FAILED.load(Ordering::Relaxed) {
                            return Err("Permission denied for mouse tracking.\nRun with sudo or add user to 'input' group.".to_string());
                        }
                        if eval_args.len() != 0 {
                            return Err(format!(
                                "Line {}: 'mousedelta' expects exactly 0 arguments",
                                line
                            ));
                        }
                        let dx = MOUSE_DX.swap(0, Ordering::Relaxed);
                        let dy = MOUSE_DY.swap(0, Ordering::Relaxed);
                        return Ok(Value::List(vec![
                            Value::Number(dx as f64),
                            Value::Number(dy as f64),
                        ]));
                    }
                    "setmouse" => {
                        if eval_args.len() < 2 || eval_args.len() > 3 {
                            return Err(format!(
                                "Line {}: 'setmouse' expects 2 or 3 arguments",
                                line
                            ));
                        }
                        let x = if let Value::Number(n) = eval_args[0].1 {
                            n as i32
                        } else {
                            return Err(format!("Line {}: 'setmouse' x must be a number", line));
                        };
                        let y = if let Value::Number(n) = eval_args[1].1 {
                            n as i32
                        } else {
                            return Err(format!("Line {}: 'setmouse' y must be a number", line));
                        };
                        let relative = if eval_args.len() == 3 {
                            if let Value::Bool(b) = eval_args[2].1 {
                                b
                            } else {
                                return Err(format!(
                                    "Line {}: 'setmouse' relative flag must be a boolean",
                                    line
                                ));
                            }
                        } else {
                            false
                        };
                        set_mouse_pos(x, y, relative)
                            .map_err(|e| format!("Line {}: {}", line, e))?;
                        return Ok(Value::Nil);
                    }
                    "clear" => {
                        if eval_args.len() > 1 {
                            return Err(format!("Line {}: 'clear' expects 0 or 1 argument", line));
                        }
                        let mut do_send = true;
                        if eval_args.len() == 1 {
                            if let Value::Bool(b) = eval_args[0].1 {
                                do_send = b;
                            } else {
                                return Err(format!(
                                    "Line {}: 'clear' argument must be a boolean",
                                    line
                                ));
                            }
                        }

                        let mut out = self.output.lock().unwrap();
                        out.clear();
                        let mut caret = self.caret.lock().unwrap();
                        caret.0 = 0;
                        caret.1 = 0;
                        drop(caret);
                        drop(out);

                        if do_send {
                            self.send_output();
                        }
                        return Ok(Value::Nil);
                    }
                    "time" => {
                        static SCRIPT_START: std::sync::OnceLock<std::time::Instant> =
                            std::sync::OnceLock::new();
                        if eval_args.len() != 0 {
                            return Err(format!(
                                "Line {}: 'time' expects exactly 0 arguments",
                                line
                            ));
                        }
                        let start = SCRIPT_START.get_or_init(std::time::Instant::now);
                        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
                        return Ok(Value::Number(elapsed));
                    }
                    "activekeys" => {
                        if eval_args.len() != 1 {
                            return Err(format!(
                                "Line {}: 'activekeys' expects exactly 1 argument",
                                line
                            ));
                        }
                        let mut active = Vec::new();
                        if let Value::List(keys) = &eval_args[0].1 {
                            for k in keys {
                                let key_str = match variant_to_key_str(k) {
                                    Ok(s) => s,
                                    Err(e) => return Err(format!("Line {}: {}", line, e)),
                                };
                                if check_key_down(&key_str).unwrap_or(false) {
                                    active.push(k.clone());
                                }
                            }
                        } else {
                            return Err(format!(
                                "Line {}: 'activekeys' expects a list of keys",
                                line
                            ));
                        }
                        return Ok(Value::List(active));
                    }
                    "setcaret" => {
                        if eval_args.len() != 2 {
                            return Err(format!(
                                "Line {}: 'setcaret' expects exactly 2 arguments",
                                line
                            ));
                        }
                        let x = if let Value::Number(n) = eval_args[0].1 {
                            n.max(0.0) as usize
                        } else {
                            return Err(format!("Line {}: 'setcaret' x must be a number", line));
                        };
                        let y = if let Value::Number(n) = eval_args[1].1 {
                            n.max(0.0) as usize
                        } else {
                            return Err(format!("Line {}: 'setcaret' y must be a number", line));
                        };

                        let mut caret = self.caret.lock().unwrap();
                        caret.0 = x;
                        caret.1 = y;
                        drop(caret);

                        self.send_output();
                        return Ok(Value::Nil);
                    }
                    "scroll" => {
                        if eval_args.len() != 1 {
                            return Err(format!(
                                "Line {}: 'scroll' expects exactly 1 argument",
                                line
                            ));
                        }
                        let num = if let Value::Number(n) = eval_args[0].1 {
                            n as i32
                        } else {
                            return Err(format!("Line {}: 'scroll' num must be a number", line));
                        };
                        simulate_scroll(num).map_err(|e| format!("Line {}: {}", line, e))?;
                        return Ok(Value::Nil);
                    }
                    "getpixel" => {
                        if eval_args.len() != 2 {
                            return Err(format!(
                                "Line {}: 'getpixel' expects exactly 2 arguments",
                                line
                            ));
                        }
                        let x = if let Value::Number(n) = eval_args[0].1 {
                            n as i32
                        } else {
                            return Err(format!("Line {}: 'getpixel' x must be a number", line));
                        };
                        let y = if let Value::Number(n) = eval_args[1].1 {
                            n as i32
                        } else {
                            return Err(format!("Line {}: 'getpixel' y must be a number", line));
                        };

                        match get_screen_pixel(x, y) {
                            Ok((r, g, b)) => {
                                let hex = format!("{:02x}{:02x}{:02x}", r, g, b);
                                return Ok(Value::EnumVariant("Color".to_string(), hex, None));
                            }
                            Err(e) => return Err(format!("Line {}: {}", line, e)),
                        }
                    }
                    "macrodata" => {
                        if eval_args.len() > 1 {
                            return Err(format!(
                                "Line {}: 'macrodata' expects 0 or 1 argument",
                                line
                            ));
                        }

                        if let Ok(config_dir) = crate::get_config_dir() {
                            let md_dir = config_dir.join("macrodata");
                            let mut md_file = md_dir.join(&self.macro_rel_path);
                            md_file.set_extension("nuuidata");

                            if eval_args.len() == 0 {
                                let mut map = std::collections::HashMap::new();
                                if let Ok(contents) = std::fs::read_to_string(&md_file) {
                                    for md_line in contents.lines() {
                                        if let Some((k, v)) = md_line.split_once('=') {
                                            let key = k.trim().to_string();
                                            let val_str = v.trim();

                                            let val = if let Ok(n) = val_str.parse::<f64>() {
                                                Value::Number(n)
                                            } else if val_str == "True" {
                                                Value::Bool(true)
                                            } else if val_str == "False" {
                                                Value::Bool(false)
                                            } else {
                                                Value::String(val_str.to_string())
                                            };

                                            map.insert(Value::String(key), val);
                                        }
                                    }
                                }
                                return Ok(Value::Dict(map));
                            } else {
                                if let Value::Dict(map) = &eval_args[0].1 {
                                    if let Some(parent) = md_file.parent() {
                                        let _ = std::fs::create_dir_all(parent);
                                    }
                                    if map.is_empty() {
                                        if md_file.exists() {
                                            let _ = std::fs::remove_file(md_file);
                                        }
                                    } else {
                                        let mut content = String::new();
                                        for (k, v) in map {
                                            let k_str = match k {
                                                Value::String(s) => s.clone(),
                                                _ => k.to_string(),
                                            };
                                            let v_str = match v {
                                                Value::String(s) => s.clone(),
                                                _ => v.to_string(),
                                            };
                                            content.push_str(&format!("{} = {}\n", k_str, v_str));
                                        }
                                        let _ = std::fs::write(md_file, content);
                                    }
                                    return Ok(Value::Nil);
                                } else {
                                    return Err(format!(
                                        "Line {}: 'macrodata' expects a dictionary to save",
                                        line
                                    ));
                                }
                            }
                        }
                        return Ok(Value::Nil);
                    }
                    "compixel" => {
                        if eval_args.len() < 3 || eval_args.len() > 4 {
                            return Err(format!(
                                "Line {}: 'compixel' expects 3 or 4 arguments",
                                line
                            ));
                        }
                        let x = if let Value::Number(n) = eval_args[0].1 {
                            n as i32
                        } else {
                            return Err(format!("Line {}: 'compixel' x must be a number", line));
                        };
                        let y = if let Value::Number(n) = eval_args[1].1 {
                            n as i32
                        } else {
                            return Err(format!("Line {}: 'compixel' y must be a number", line));
                        };

                        let target_color = if let Value::EnumVariant(enum_name, variant, _) =
                            &eval_args[2].1
                        {
                            if enum_name == "Color" {
                                if let Ok(c) = crate::theme::themecore::parse_color(variant) {
                                    if let Some(rgb) = c.to_rgb() {
                                        rgb
                                    } else {
                                        return Err(format!(
                                            "Line {}: 'compixel' color cannot be None",
                                            line
                                        ));
                                    }
                                } else {
                                    return Err(format!(
                                        "Line {}: Invalid color variant '{}'",
                                        line, variant
                                    ));
                                }
                            } else {
                                return Err(format!(
                                    "Line {}: 'compixel' expects a Color enum for the 3rd argument",
                                    line
                                ));
                            }
                        } else {
                            return Err(format!(
                                "Line {}: 'compixel' expects a Color enum for the 3rd argument",
                                line
                            ));
                        };

                        let offset = if eval_args.len() == 4 {
                            if let Value::Number(n) = eval_args[3].1 {
                                n.clamp(0.0, 255.0) as u8
                            } else {
                                return Err(format!(
                                    "Line {}: 'compixel' offset must be a number",
                                    line
                                ));
                            }
                        } else {
                            0u8
                        };

                        match get_screen_pixel(x, y) {
                            Ok((r, g, b)) => {
                                let (tr, tg, tb) = target_color;

                                let match_r = r.abs_diff(tr) <= offset;
                                let match_g = g.abs_diff(tg) <= offset;
                                let match_b = b.abs_diff(tb) <= offset;

                                return Ok(Value::Bool(match_r && match_g && match_b));
                            }
                            Err(e) => return Err(format!("Line {}: {}", line, e)),
                        }
                    }
                    "keypress" => {
                        if eval_args.len() < 1 || eval_args.len() > 2 {
                            return Err(format!(
                                "Line {}: 'keypress' expects 1 or 2 arguments",
                                line
                            ));
                        }

                        let key_str = match variant_to_key_str(&eval_args[0].1) {
                            Ok(s) => s,
                            Err(e) => return Err(format!("Line {}: {}", line, e)),
                        };

                        let ms = if eval_args.len() == 2 {
                            if let Value::Number(n) = eval_args[1].1 {
                                if n < 0.0 {
                                    return Err(format!(
                                        "Line {}: 'keypress' duration cannot be negative",
                                        line
                                    ));
                                }
                                n
                            } else {
                                return Err(format!(
                                    "Line {}: 'keypress' duration must be a number",
                                    line
                                ));
                            }
                        } else {
                            50.0
                        };

                        if let Err(e) = simulate_key(&key_str, true) {
                            return Err(format!("Line {}: {}", line, e));
                        }

                        let dur = std::time::Duration::from_secs_f64(ms / 1000.0);
                        let target = std::time::Instant::now() + dur;

                        loop {
                            if self.cancel_token.load(Ordering::Relaxed) {
                                self.should_exit = true;
                                let _ = simulate_key(&key_str, false);
                                return Ok(Value::Nil);
                            }

                            let now = std::time::Instant::now();
                            if now >= target {
                                break;
                            }

                            let remaining = target - now;
                            let chunk = remaining.min(std::time::Duration::from_millis(10));
                            std::thread::sleep(chunk);
                        }

                        if let Err(e) = simulate_key(&key_str, false) {
                            return Err(format!("Line {}: {}", line, e));
                        }

                        return Ok(Value::Nil);
                    }
                    "beep" => {
                        if eval_args.len() > 2 {
                            return Err(format!(
                                "Line {}: 'beep' expects 0, 1, or 2 arguments",
                                line
                            ));
                        }

                        let freq = if eval_args.len() >= 1 {
                            if let Value::Number(n) = eval_args[0].1 {
                                n.max(37.0).min(32767.0) as u32
                            } else {
                                return Err(format!(
                                    "Line {}: 'beep' frequency must be a number",
                                    line
                                ));
                            }
                        } else {
                            440
                        };

                        let dur = if eval_args.len() == 2 {
                            if let Value::Number(n) = eval_args[1].1 {
                                n.max(0.0) as u32
                            } else {
                                return Err(format!(
                                    "Line {}: 'beep' duration must be a number",
                                    line
                                ));
                            }
                        } else {
                            200
                        };

                        system_beep(freq, dur);
                        return Ok(Value::Nil);
                    }
                    "caretx" => {
                        if eval_args.len() != 0 {
                            return Err(format!(
                                "Line {}: 'caretx' expects exactly 0 arguments",
                                line
                            ));
                        }
                        let caret = self.caret.lock().unwrap();
                        return Ok(Value::Number(caret.0 as f64));
                    }
                    "carety" => {
                        if eval_args.len() != 0 {
                            return Err(format!(
                                "Line {}: 'carety' expects exactly 0 arguments",
                                line
                            ));
                        }
                        let caret = self.caret.lock().unwrap();
                        return Ok(Value::Number(caret.1 as f64));
                    }
                    "screenx" => {
                        if eval_args.len() != 0 {
                            return Err(format!(
                                "Line {}: 'screenx' expects exactly 0 arguments",
                                line
                            ));
                        }
                        let (x, _) =
                            get_screen_size().map_err(|e| format!("Line {}: {}", line, e))?;
                        return Ok(Value::Number(x as f64));
                    }
                    "screeny" => {
                        if eval_args.len() != 0 {
                            return Err(format!(
                                "Line {}: 'screeny' expects exactly 0 arguments",
                                line
                            ));
                        }
                        let (_, y) =
                            get_screen_size().map_err(|e| format!("Line {}: {}", line, e))?;
                        return Ok(Value::Number(y as f64));
                    }
                    "focused" => {
                        if eval_args.len() != 0 {
                            return Err(format!(
                                "Line {}: 'focused' expects exactly 0 arguments",
                                line
                            ));
                        }
                        let window_name =
                            get_focused_window().map_err(|e| format!("Line {}: {}", line, e))?;
                        return Ok(Value::String(window_name));
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
