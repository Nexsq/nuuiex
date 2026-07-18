use super::ast::{BinaryOp, Expr, FunctionDef, Stmt, StringPart};
use super::value::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, SyncSender};

#[cfg(windows)]
fn check_key_down(key: &str) -> Result<bool, String> {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;
    let mut req_shift = false;
    let mut req_ctrl = false;
    let mut req_alt = false;

    let vk = match key.to_lowercase().as_str() {
        "backspace" | "back" => VK_BACK,
        "tab" => VK_TAB,
        "enter" | "return" => VK_RETURN,
        "shift" => VK_SHIFT,
        "ctrl" | "control" => VK_CONTROL,
        "alt" | "menu" => VK_MENU,
        "pause" => VK_PAUSE,
        "capslock" | "caps" => VK_CAPITAL,
        "esc" | "escape" => VK_ESCAPE,
        "space" => VK_SPACE,
        "pgup" | "pageup" => VK_PRIOR,
        "pgdn" | "pagedown" => VK_NEXT,
        "end" => VK_END,
        "home" => VK_HOME,
        "left" => VK_LEFT,
        "up" => VK_UP,
        "right" => VK_RIGHT,
        "down" => VK_DOWN,
        "prtscr" | "printscreen" => VK_SNAPSHOT,
        "ins" | "insert" => VK_INSERT,
        "del" | "delete" => VK_DELETE,
        "lwin" | "cmd" | "super" | "win" => VK_LWIN,
        "rwin" => VK_RWIN,
        "f1" => VK_F1,
        "f2" => VK_F2,
        "f3" => VK_F3,
        "f4" => VK_F4,
        "f5" => VK_F5,
        "f6" => VK_F6,
        "f7" => VK_F7,
        "f8" => VK_F8,
        "f9" => VK_F9,
        "f10" => VK_F10,
        "f11" => VK_F11,
        "f12" => VK_F12,
        "shiftup" => {
            req_shift = true;
            VK_UP
        }
        "shiftdown" => {
            req_shift = true;
            VK_DOWN
        }
        "shiftleft" => {
            req_shift = true;
            VK_LEFT
        }
        "shiftright" => {
            req_shift = true;
            VK_RIGHT
        }
        "ctrlup" => {
            req_ctrl = true;
            VK_UP
        }
        "ctrldown" => {
            req_ctrl = true;
            VK_DOWN
        }
        "ctrlleft" => {
            req_ctrl = true;
            VK_LEFT
        }
        "ctrlright" => {
            req_ctrl = true;
            VK_RIGHT
        }
        "ctrlshiftup" => {
            req_ctrl = true;
            req_shift = true;
            VK_UP
        }
        "ctrlshiftdown" => {
            req_ctrl = true;
            req_shift = true;
            VK_DOWN
        }
        "ctrlshiftleft" => {
            req_ctrl = true;
            req_shift = true;
            VK_LEFT
        }
        "ctrlshiftright" => {
            req_ctrl = true;
            req_shift = true;
            VK_RIGHT
        }
        "ctrldelete" => {
            req_ctrl = true;
            VK_DELETE
        }
        "ctrlbackspace" => {
            req_ctrl = true;
            VK_BACK
        }
        s if s.len() == 1 => {
            let c = key.chars().next().unwrap();
            unsafe {
                let res = VkKeyScanW(c as u16);
                if res == -1 {
                    return Ok(false);
                }
                let state = (res >> 8) & 0xFF;
                let is_alpha = c.is_ascii_alphabetic();

                if state & 1 != 0 && !is_alpha {
                    req_shift = true;
                }
                if state & 2 != 0 {
                    req_ctrl = true;
                }
                if state & 4 != 0 {
                    req_alt = true;
                }

                (res & 0xFF) as u16
            }
        }
        _ => return Ok(false),
    };

    unsafe {
        let is_down = (GetAsyncKeyState(vk as i32) as u16 & 0x8000) != 0;
        if !is_down {
            return Ok(false);
        }
        if req_shift && (GetAsyncKeyState(VK_SHIFT as i32) as u16 & 0x8000) == 0 {
            return Ok(false);
        }
        if req_ctrl && (GetAsyncKeyState(VK_CONTROL as i32) as u16 & 0x8000) == 0 {
            return Ok(false);
        }
        if req_alt && (GetAsyncKeyState(VK_MENU as i32) as u16 & 0x8000) == 0 {
            return Ok(false);
        }
        Ok(true)
    }
}

#[cfg(target_os = "linux")]
fn check_key_down(key: &str) -> Result<bool, String> {
    let mut req_shift = false;
    let mut req_ctrl = false;
    let mut search_key = key.to_string();

    if key.len() == 1 {
        let c = key.chars().next().unwrap();
        if c.is_ascii_alphabetic() {
            search_key = c.to_ascii_lowercase().to_string();
        } else {
            let shift_map = [
                ('!', '1'),
                ('@', '2'),
                ('#', '3'),
                ('$', '4'),
                ('%', '5'),
                ('^', '6'),
                ('&', '7'),
                ('*', '8'),
                ('(', '9'),
                (')', '0'),
                ('_', '-'),
                ('+', '='),
                ('{', '['),
                ('}', ']'),
                ('|', '\\'),
                (':', ';'),
                ('"', '\''),
                ('<', ','),
                ('>', '.'),
                ('?', '/'),
            ];
            for &(shifted, unshifted) in &shift_map {
                if c == shifted {
                    req_shift = true;
                    search_key = unshifted.to_string();
                    break;
                }
            }
        }
    }

    let codes = match search_key.to_lowercase().as_str() {
        "esc" | "escape" => vec![1],
        "1" => vec![2],
        "2" => vec![3],
        "3" => vec![4],
        "4" => vec![5],
        "5" => vec![6],
        "6" => vec![7],
        "7" => vec![8],
        "8" => vec![9],
        "9" => vec![10],
        "0" => vec![11],
        "minus" | "-" => vec![12],
        "equal" | "=" => vec![13],
        "backspace" | "back" => vec![14],
        "tab" => vec![15],
        "q" => vec![16],
        "w" => vec![17],
        "e" => vec![18],
        "r" => vec![19],
        "t" => vec![20],
        "y" => vec![21],
        "u" => vec![22],
        "i" => vec![23],
        "o" => vec![24],
        "p" => vec![25],
        "[" => vec![26],
        "]" => vec![27],
        "enter" | "return" => vec![28],
        "ctrl" | "control" => vec![29, 97],
        "a" => vec![30],
        "s" => vec![31],
        "d" => vec![32],
        "f" => vec![33],
        "g" => vec![34],
        "h" => vec![35],
        "j" => vec![36],
        "k" => vec![37],
        "l" => vec![38],
        ";" => vec![39],
        "'" => vec![40],
        "`" => vec![41],
        "shift" => vec![42, 54],
        "\\" => vec![43],
        "z" => vec![44],
        "x" => vec![45],
        "c" => vec![46],
        "v" => vec![47],
        "b" => vec![48],
        "n" => vec![49],
        "m" => vec![50],
        "," => vec![51],
        "." => vec![52],
        "/" => vec![53],
        "alt" | "menu" => vec![56, 100],
        "space" | " " => vec![57],
        "capslock" | "caps" => vec![58],
        "f1" => vec![59],
        "f2" => vec![60],
        "f3" => vec![61],
        "f4" => vec![62],
        "f5" => vec![63],
        "f6" => vec![64],
        "f7" => vec![65],
        "f8" => vec![66],
        "f9" => vec![67],
        "f10" => vec![68],
        "f11" => vec![87],
        "f12" => vec![88],
        "up" => vec![103],
        "left" => vec![105],
        "right" => vec![106],
        "down" => vec![108],
        "home" => vec![102],
        "end" => vec![107],
        "pgup" | "pageup" => vec![104],
        "pgdn" | "pagedown" => vec![109],
        "ins" | "insert" => vec![110],
        "del" | "delete" => vec![111],
        "prtscr" | "printscreen" => vec![99],
        "lwin" | "cmd" | "super" | "win" => vec![125, 126],
        "shiftup" => {
            req_shift = true;
            vec![103]
        }
        "shiftdown" => {
            req_shift = true;
            vec![108]
        }
        "shiftleft" => {
            req_shift = true;
            vec![105]
        }
        "shiftright" => {
            req_shift = true;
            vec![106]
        }
        "ctrlup" => {
            req_ctrl = true;
            vec![103]
        }
        "ctrldown" => {
            req_ctrl = true;
            vec![108]
        }
        "ctrlleft" => {
            req_ctrl = true;
            vec![105]
        }
        "ctrlright" => {
            req_ctrl = true;
            vec![106]
        }
        "ctrlshiftup" => {
            req_ctrl = true;
            req_shift = true;
            vec![103]
        }
        "ctrlshiftdown" => {
            req_ctrl = true;
            req_shift = true;
            vec![108]
        }
        "ctrlshiftleft" => {
            req_ctrl = true;
            req_shift = true;
            vec![105]
        }
        "ctrlshiftright" => {
            req_ctrl = true;
            req_shift = true;
            vec![106]
        }
        "ctrldelete" => {
            req_ctrl = true;
            vec![111]
        }
        "ctrlbackspace" => {
            req_ctrl = true;
            vec![14]
        }
        _ => return Ok(false),
    };

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
                        for &code in &codes {
                            if code < (96 * 8) {
                                let byte = (code / 8) as usize;
                                let bit = code % 8;
                                if (key_bits[byte] & (1 << bit)) != 0 {
                                    is_down = true;
                                }
                            }
                        }
                        if req_shift {
                            for &code in &[42, 54] {
                                let byte = (code / 8) as usize;
                                let bit = code % 8;
                                if (key_bits[byte] & (1 << bit)) != 0 {
                                    shift_down = true;
                                }
                            }
                        }
                        if req_ctrl {
                            for &code in &[29, 97] {
                                let byte = (code / 8) as usize;
                                let bit = code % 8;
                                if (key_bits[byte] & (1 << bit)) != 0 {
                                    ctrl_down = true;
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

    if req_shift && !shift_down {
        return Ok(false);
    }
    if req_ctrl && !ctrl_down {
        return Ok(false);
    }
    Ok(is_down)
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
                        return Err("Variant 'Ctrl' with argument is not supported in isdown()/isup(). Use Key::CtrlLeft or Key::CtrlRight instead.".to_string());
                    }
                    return Ok("ctrl".to_string());
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

pub struct Environment {
    pub scopes: Vec<HashMap<String, Value>>,
    pub constants: Vec<HashSet<String>>,
}

impl Environment {
    pub fn new() -> Self {
        let mut env = Self {
            scopes: vec![HashMap::new()],
            constants: vec![HashSet::new()],
        };
        env.define(
            "Key".to_string(),
            Value::BuiltinEnum("Key".to_string()),
            true,
        )
        .unwrap();
        env
    }

    pub fn push(&mut self) {
        self.scopes.push(HashMap::new());
        self.constants.push(HashSet::new());
    }

    pub fn pop(&mut self) {
        self.scopes.pop();
        self.constants.pop();
    }

    pub fn define(&mut self, name: String, val: Value, is_const: bool) -> Result<(), String> {
        let last_scope = self.scopes.last_mut().unwrap();
        if last_scope.contains_key(&name) {
            if let Some(existing_is_const) = self.constants.last().unwrap().contains(&name).into() {
                if existing_is_const {
                    return Err(format!("Cannot modify constant '{}'", name));
                }
            }
            last_scope.insert(name.clone(), val);
            return Ok(());
        }
        last_scope.insert(name.clone(), val);
        if is_const {
            self.constants.last_mut().unwrap().insert(name);
        }
        Ok(())
    }

    pub fn assign(&mut self, name: &str, val: Value) -> Result<(), String> {
        for (scope, consts) in self.scopes.iter_mut().zip(self.constants.iter()).rev() {
            if scope.contains_key(name) {
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
        for scope in self.scopes.iter().rev() {
            if let Some(val) = scope.get(name) {
                return Some(val.clone());
            }
        }
        None
    }
}

pub struct Interpreter {
    pub output: Vec<String>,
    pub errors: Vec<String>,
    current_line: String,
    tx: SyncSender<crate::EngineMessage>,
    input_rx: Receiver<String>,
    pub should_exit: bool,
    pub env: Environment,
    cancel_token: Arc<AtomicBool>,
}

#[derive(PartialEq)]
pub enum Signal {
    Empty,
    Break,
    Return(Value),
}

impl Interpreter {
    pub fn new(
        tx: SyncSender<crate::EngineMessage>,
        input_rx: Receiver<String>,
        cancel_token: Arc<AtomicBool>,
    ) -> Self {
        Self {
            output: Vec::new(),
            errors: Vec::new(),
            current_line: String::new(),
            tx,
            input_rx,
            should_exit: false,
            env: Environment::new(),
            cancel_token,
        }
    }

    fn send_output(&mut self) {
        if self.output.len() > 1000 {
            let excess = self.output.len() - 1000;
            self.output.drain(0..excess);
        }

        let mut res = self.output.clone();

        if !self.current_line.is_empty() {
            res.push(self.current_line.clone());
        }

        if !self.errors.is_empty() {
            if !res.is_empty() && !res.last().unwrap().is_empty() {
                res.push("".to_string());
            }
            res.push("--- Runtime Errors ---".to_string());
            res.extend(self.errors.clone());
        }

        if res.is_empty() {
            res.push("Execution finished with no output.".to_string());
        }

        if self.tx.send(crate::EngineMessage::Output(res)).is_err() {
            self.should_exit = true;
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
                Ok(Signal::Return(val)) => {
                    res = Ok(Signal::Return(val));
                    break;
                }
                Ok(Signal::Empty) => continue,
                Err(err) => {
                    self.errors.push(err);
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
            Stmt::Break(_) => Ok(Signal::Break),
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
                            "LWin",
                            "RWin",
                        ];
                        if !valid_variants.contains(&prop.as_str()) {
                            return Err(format!(
                                "Line {}: Invalid variant '{}' for enum 'Key'",
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
                            "LWin",
                            "RWin",
                        ];
                        if !valid_variants.contains(&method.as_str()) {
                            return Err(format!(
                                "Line {}: Invalid variant '{}' for enum 'Key'",
                                line, method
                            ));
                        }
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
                }

                let res = self.apply_method(&mut left_val, method, eval_args, *line)?;
                self.try_assign_expr(left, left_val)?;
                Ok(res)
            }
            Expr::Binary(left, op, right, line) => {
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
                                self.current_line.push_str(segment);
                            } else {
                                self.output.push(std::mem::take(&mut self.current_line));
                                self.current_line = segment.to_string();
                            }
                        }

                        if name == "println" {
                            self.output.push(std::mem::take(&mut self.current_line));
                        }

                        self.send_output();
                        return Ok(Value::Nil);
                    }
                    "sleep" => {
                        if eval_args.len() != 1 {
                            return Err(format!("Line {}: 'sleep' expects 1 argument", line));
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
                            self.current_line.push_str(&prompt);
                        }
                        self.send_output();

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

                        self.current_line.push_str(&result);
                        return Ok(Value::String(result));
                    }
                    "len" => {
                        if eval_args.len() != 1 {
                            return Err(format!("Line {}: 'len' expects 1 argument", line));
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
                            return Err(format!("Line {}: 'exec' expects 1 argument", line));
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
                    "isdown" | "isup" => {
                        if eval_args.len() != 1 {
                            return Err(format!("Line {}: '{}' expects 1 argument", line, name));
                        }

                        let key_str = match variant_to_key_str(&eval_args[0].1) {
                            Ok(s) => s,
                            Err(e) => return Err(format!("Line {}: {}", line, e)),
                        };

                        let is_down = match check_key_down(&key_str) {
                            Ok(b) => b,
                            Err(e) => return Err(format!("Line {}: {}", line, e)),
                        };

                        if name == "isdown" {
                            return Ok(Value::Bool(is_down));
                        } else {
                            return Ok(Value::Bool(!is_down));
                        }
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
