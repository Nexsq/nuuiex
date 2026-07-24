***

# NUUI ex(tended)

**NUUI ex** is a powerful, highly customizable, macro engine. It features its own scripting language (NUUI Lang), a built-in code editor with syntax highlighting, a robust macro execution engine and more.

---

## ▪ Table of Contents

1. [App Functionality & Features](#-app-functionality--features)
    - [1. Global Navigation & Exiting](#1-global-navigation--exiting)
    - [2. Library & File Manager](#2-library--file-manager)
    - [3. Integrated Text Editor](#3-integrated-text-editor)
    - [4. Deck & Dashboard Widgets](#4-deck--dashboard-widgets)
2. [The NUUI Scripting Language (NUUI Lang)](#-the-nuui-scripting-language-nuui-lang)
    - [1. Syntax & Basics](#1-syntax--basics)
    - [2. Control Flow](#2-control-flow)
    - [3. Functions & Async](#3-functions--async)
3. [Data Types & Built-in Methods](#-data-types--built-in-methods)
    - [1. Numbers](#1-numbers)
    - [2. Strings](#2-strings)
    - [3. Lists](#3-lists)
    - [4. Dictionaries](#4-dictionaries)
4. [Built-in Enums](#-built-in-enums)
    - [1. Key Enum](#1-key-enum)
    - [2. Color Enum](#2-color-enum)
    - [3. Modifier Enum](#3-modifier-enum)
5. [Built-in Functions Reference](#-built-in-functions-reference)
    - [1. I/O & System](#1-io--system)
    - [2. Mouse & Keyboard Automation](#2-mouse--keyboard-automation)
    - [3. Screen](#3-screen)
    - [4. Math & Utilities](#4-math--utilities)
6. [Configuration & Theming](#-configuration--theming)
    - [1. Theming (themes/*.conf)](#1-theming-themesconf)
    - [2. Config (config.conf)](#2-config-configconf)
7. [Examples](#-examples)
8. [Building & Installation](#-building--installation)

---

## ▪ App Functionality & Features

NUUI is divided into several panels. You can navigate between the **Library** and the **Editor** using `Tab`.

### 1. Global Navigation & Exiting
* **`Esc`**: Acts as the "back" key. 
  * If you are in the Editor in **Insert Mode** or **Visual Mode**, `Esc` will return you to **Command Mode**.
  * If you are already in **Command Mode**, or if you are browsing the **Library**, pressing `Esc` will open the **Settings Modal**.
* **`q`**: Properly **exits the application** (can be set to require a double-press in settings).

### 2. Library & File Manager
The Library panel manages your `.nuui` scripts. It supports infinite nesting and custom sorting.
* **Navigation:** `Up`/`Down`/`Left`/`Right` arrow keys.
* **Actions:**
  * **Create File/Folder:** Press `c` (file) or `z` (folder).
  * **Rename:** Press `r`.
  * **Delete:** Press `d` (or `Shift+D` to bypass the prompt for empty files).
  * **Edit Script:** Press `e`.
  * **Run Script:** Press `Enter`.
  * **Move Up/Down:** Press `k` / `j` (Requires "custom" sorting enabled in settings).

### 3. Integrated Text Editor
A modal code editor, complete with an AST-based real-time syntax checker.

**Modes:**
* **Command Mode `[CMD]`:** Default mode for navigation and manipulation.
* **Insert Mode `[INS]`:** For typing code (Press `i` to enter).
* **Visual Mode `[VIS]`:** For selecting text blocks (Press `v` to enter).
* **Search Mode `[FND]`:** For finding text (Press `f` to enter).
* **Line Search Mode `[LNE]`:** For jumping to specific lines by number (Press `Ctrl+g` to enter).

**Default Editor Keybinds (Command Mode):**
* `i`: Enter Insert Mode.
* `v`: Enter Visual Mode.
* `f`: Search.
* `u` / `r`: Undo / Redo.
* `y` / `p`: Copy / Paste (Integrates with the system clipboard via `arboard`).
* `d`: Delete char/selection.
* `t`: Fold/Unfold function blocks.
* `s`: Save file.
* `a`: Select All.
* `g` / `Shift+g` / `Ctrl+g`: Jump to start / Jump to end / Open Line Search Mode.
* `w` / `b`: Jump word forward/backward.

### 4. Deck & Dashboard Widgets
The "Deck" is a customizable widget area.
* **Keyvis:** A physics-based, gravity-simulated key visualizer.
* **System Monitor:** Real-time hardware graphs.
* **Macrostats:** Displays metadata, code info, errors, and running times of macros.
* **Clock:** A highly customizable digital clock.

---

## ▪ The NUUI Scripting Language (NUUI Lang)

NUUI uses its custom scripting language, called NUUI Lang. It is dynamically typed, supports asynchronous execution, and features a clean syntax inspired by Rust and Python.

### 1. Syntax & Basics

**Variables:**
```python
let x = 10          # Mutable variable
const PI = 3.14159  # Immutable constant
x += 5              # Compound assignments supported (+, -, *, /, %)
```

**String Interpolation:**
```python
let name = "Nuui"
println("Hello, {name}! Math: {10 * 2}") # Outputs: Hello, Nuui! Math: 20
```

### 2. Control Flow

**If / Elif / Else:**
```python
if x > 10:
    println("Greater")
elif x == 10:
    println("Equal")
else:
    println("Lesser")
```

**Loops:**
```python
# Infinite loop
loop:
    if isdown(Key:Esc):
        break

# While loop
while x > 0:
    x -= 1

# For loop (Iterates over Lists, Strings, and Dicts)
for item in range(0, 5):
    if item == 2:
        continue
    println(item)
```

### 3. Functions & Async

**Defining Functions:**
```python
fn foo(name, punctuation = "!"):
    return "Hello " + name + punctuation
println(greet("User"))
```

**Asynchronous Blocks:**
Spawns a new background thread that shares the current environment state.
```python
async:
    sleep(5000)
    println("This prints after 5 seconds")
println("This prints immediately")
```

---

## ▪ Data Types & Built-in Methods

NUUI features an object-oriented approach to primitives. You can call methods directly on variables or literal values using the `::` syntax.

### 1. Numbers
Numbers are represented internally as 64-bit floats (`f64`).

| Method | Description |
| :--- | :--- |
| `abs()` | Returns the absolute value. |
| `neg()` | Returns the negative value. |
| `floor()` | Rounds down to the nearest integer. |
| `ceil()` | Rounds up to the nearest integer. |
| `trunc()` | Returns the integer part, removing decimals. |
| `fract()` | Returns the fractional (decimal) part. |
| `clamp(min, max)` | Constrains the number between `min` and `max`. |
| `round(places?)` | Rounds the number (optionally to `places` decimal points). |
| `pow(exp)` | Raises the number to the power of `exp`. |
| `sqrt()` | Returns the square root. |

### 2. Strings

| Method | Description |
| :--- | :--- |
| `len()` | Returns the number of characters. |
| `capitalize()` | Capitalizes the first letter. |
| `lower()` / `upper()` | Converts the string to lowercase or uppercase. |
| `swapcase()` | Swaps the casing of all characters. |
| `count(sub)` | Returns how many times `sub` appears in the string. |
| `index(sub)` | Returns the starting index of `sub`, or `None`. |
| `trim()` | Removes leading and trailing whitespace. |
| `split(sep?)` | Splits the string into a List (by whitespace if `sep` is omitted). |
| `join(list)` | Joins a list of items using the string as a separator. |
| `replace(old, new)`| Replaces all instances of `old` with `new`. |
| `startswith(s)` | Returns `True` if the string starts with `s`. |
| `endswith(s)` | Returns `True` if the string ends with `s`. |
| `asnum()` | Parses the string into a Number, or `None` if invalid. |

### 3. Lists
Lists are dynamically sized arrays: `let arr = [1, "two", False]`

| Method | Description |
| :--- | :--- |
| `len()` | Returns the number of elements. |
| `append(val)` | Adds `val` to the end of the list. Returns the modified list. |
| `extend(list)` | Appends all items from another `list`. |
| `insert(val, pos)` | Inserts `val` at index `pos`. |
| `pop(pos?)` | Removes and returns the item at `pos` (or the last item). |
| `remove(val)` | Removes the first occurrence of `val`. |
| `index(val)` | Returns the index of `val`, or `None`. |
| `count(val)` | Returns the number of `val` occurrences. |
| `clear()` | Empties the list. |

### 4. Dictionaries
Dictionaries are HashMaps with dynamic keys and values: `let dict = {"key": "value"}`

| Method | Description |
| :--- | :--- |
| `len()` | Returns the number of key-value pairs. |
| `get(key, def?)` | Returns the value for `key`, or `def` (or `None`) if not found. |
| `set(key, val)` | Sets `key` to `val`. Returns the value. |
| `update(dict)` | Merges another dictionary into this one. |
| `keys()` | Returns a List of all keys. |
| `values()` | Returns a List of all values. |
| `pop(key?)` | Removes and returns the value for `key`, or a random `[key, value]` pair. |
| `clear()` | Empties the dictionary. |

---

## ▪ Built-in Enums

Enums are accessed using the double-colon static syntax.

### 1. Key Enum
Represents keyboard and mouse inputs for automation functions (`isdown`, `keydown`, etc.).

**Available Variants:**
* **Base Keys:** `Space`, `Enter`, `Tab`, `Backspace`, `Esc`, `Delete`, `Insert`, `CapsLock`, `None`
* **Characters:** `Char("a")` *(Takes a 1-character string argument)*
* **Modifiers:** `Ctrl`, `Alt`, `LMeta` (Win/Cmd), `RMeta`, `Shift` (or `Shift("a")` to shift a specific key)
* **Function Keys:** `F(1)` through `F(24)` *(Takes a number argument)*
* **Navigation:** `Up`, `Down`, `Left`, `Right`, `Home`, `End`, `PgUp`, `PgDn`
* **Mouse Buttons:** `LMB`, `RMB`, `MMB`, `SB1` (Side Button 1), `SB2` (Side Button 2)
* **Shortcuts/Combos:** `ShiftUp`, `ShiftDown`, `ShiftLeft`, `ShiftRight`, `CtrlUp`, `CtrlDown`, `CtrlLeft`, `CtrlRight`, `CtrlShiftUp`, `CtrlShiftDown`, `CtrlShiftLeft`, `CtrlShiftRight`, `CtrlDelete`, `CtrlBackspace`
* **System:** `PrtScr`

### 2. Color & Background Enums
Used for terminal styling and pixel color comparisons. The `Background` enum shares the exact same variants and is used to apply background colors.

* **Standard Colors:** `None`, `Black`, `Red`, `Green`, `Yellow`, `Blue`, `Magenta`, `Cyan`, `White`, `DarkGray`
* **Bright Colors:** `BrightRed`, `BrightGreen`, `BrightYellow`, `BrightBlue`, `BrightMagenta`, `BrightCyan`, `BrightWhite`
* **Custom Hex Colors:** You can use exact hex codes, e.g., `Color:ff0055` or `Background:00ff00`.

*(You can call `Color:Red::tostring()` to get its string representation).*

### 3. Modifier Enum
Used to apply ANSI text modifiers in the terminal.

* **Variants:** `None`, `Bold`, `Dim`, `Italic`, `Underline`, `Reverse`, `Strikethrough`

---

## ▪ Built-in Functions Reference

NUUI provides powerful built-in functions injected directly into the runtime environment. 

### 1. I/O & System

| Function | Arguments | Description |
| :--- | :--- | :--- |
| `print(...)` | `*args` | Prints arguments to the script output panel separated by spaces. |
| `println(...)`| `*args` | Same as `print`, but appends a newline at the end. |
| `clear(bool?)` | `send=True` | Clears the output panel buffer. |
| `time()` | None | Returns the current system time in milliseconds (since UNIX epoch). |
| `input(prompt?)`| `String` | Pauses execution and prompts the user for text input in the output panel. Returns a `String`. |
| `sleep(ms)` | `Number` | Pauses the current thread for `ms` milliseconds. Number can be float. |
| `sleepaccurate(ms)`| `Number` | Pauses the current thread for `ms` milliseconds. Uses high-precision hardware timers and active spin-waiting to guarantee microsecond-level accuracy at the cost of high CPU usage. |
| `exit()` | None | Immediately terminates the script execution. |
| `exec(cmd)` | `String` | Executes a shell command (using `sh` on Linux, `powershell` on Windows) and returns the output (stdout + stderr) as a String. |
| `onlinux()` | None | Returns `True` if the OS is Linux, `False` otherwise. |
| `onwindows()` | None | Returns `True` if the OS is Windows, `False` otherwise. |
| `macrodata(dict?)`| `Dict` | If passed a dict, saves it to a persistent `.nuuidata` file linked to the script. If called with no arguments, reads and returns the dict from disk. |
| `beep(freq?, dur?)`| `Number, Number` | Plays a system beep. Optional arguments: frequency (Hz) and duration (ms). |
| `focused()` | None | Returns the executable name of the currently focused/active window (e.g. `"Discord.exe"` or `"firefox"`). Returns an empty string if it cannot be determined. |

### 2. Mouse & Keyboard Automation

*(On Linux, some of these require running NUUI with `sudo` or being in the `input` group. On Windows, they use the Win32 API).*

| Function | Arguments | Description |
| :--- | :--- | :--- |
| `isdown(key)` | `Key:Variant` | Returns `True` if the physical key/mouse button is currently held down. |
| `isup(key)` | `Key:Variant` | Returns `True` if the physical key/mouse button is not held down. |
| `isdownfocus(key)`| `Key:Variant` | Same as `isdown`, but only returns `True` if the macro is in focus. |
| `isupfocus(key)`| `Key:Variant` | I think it's self explanatory. |
| `keydown(key)`| `Key:Variant` | Simulates a hardware key press (pushes the key down). |
| `keyup(key)` | `Key:Variant` | Simulates a hardware key release. |
| `keypress(key, ms?)`| `Key:Variant, Number` | Simulates pressing a key down, waiting `ms` milliseconds (default 50), and then releasing it. |
| `activekeys(list)` | `List` | Takes a List of Keys and returns a sub-list of only the ones currently pressed. |
| `write(text)` | `String` | Simulates typing out a string of text sequentially at the OS level. |
| `scroll(amount)`| `Number` | Simulates the mouse scroll wheel. Positive = Up, Negative = Down. |

### 3. Screen

| Function | Arguments | Description |
| :--- | :--- | :--- |
| `mousex()` | None | Returns the absolute X coordinate of the OS mouse cursor. |
| `mousey()` | None | Returns the absolute Y coordinate of the OS mouse cursor. |
| `mousedelta()` | None | Returns a List `[dx, dy]` of raw relative mouse movement since the last call. |
| `setmouse(x, y, relative?)` | `Number, Number, Bool` | Moves the mouse cursor. If `relative` is `True`, moves it by an offset rather than to absolute screen coordinates. |
| `screenx()` | None | Returns the absolute screen resolution width (X) of the primary monitor. |
| `screeny()` | None | Returns the absolute screen resolution height (Y) of the primary monitor. |
| `getpixel(x, y)` | `Number, Number` | Returns the RGB color of the screen pixel at `(x, y)` as a `Color:Variant`. |
| `compixel(x, y, color, tol?)` | `Num, Num, Color, Num` | Compares the pixel at `(x, y)` against the provided `Color`. Optional `tol` (0-255) defines the acceptable RGB tolerance. Returns `Bool`. |
| `setcaret(x, y)` | `Number, Number` | Moves the internal terminal caret to a specific row and column in the output box. |
| `caretx()` | None | Returns the current X (column) position of the terminal output caret. |
| `carety()` | None | Returns the current Y (row) position of the terminal output caret. |

### 4. Math & Utilities

| Function | Arguments | Description |
| :--- | :--- | :--- |
| `range(start?, stop, step?)`| `Numbers` | Returns a List of numbers. E.g., `range(5)` -> `[0, 1, 2, 3, 4]`. `range(1, 10, 2)` -> `[1, 3, 5, 7, 9]`. |
| `random(start?, stop, step?)`| `Numbers` | Returns a random number between `start` and `stop`. If `step` is provided, the number will be snapped to that grid. |
| `len(obj)` | `List/String/Dict`| Returns the length of the given object. |
| `max(args)` | `*args` / `List` / `String`| Returns the maximum value from the arguments or collection. |
| `min(args)` | `*args` / `List` / `String`| Returns the minimum value from the arguments or collection. |

---

## ▪ Configuration & Theming

NUUI utilizes a highly robust theming engine. Themes and configs are stored in your OS's native configuration directory.

### 1. Theming (`themes/*.conf`)
Themes map colors to specific UI components. A theme can also specify Gradients!

**Default Theme:**
```ini
title = ' {Green}▄▄▄   {BrightGreen}▄ {BrightCyan}▄   {Cyan}▄ {BrightBlue}▄   {Blue}▄▄▄
 {Green}█ █   {BrightGreen}█ {BrightCyan}█   {Cyan}█ {BrightBlue}█    {Blue}█
 {Green}█ █   {BrightGreen}█▄{BrightCyan}█   {Cyan}█▄{BrightBlue}█   {Blue}▄█▄ EX'

main_label = Yellow
warning_color = Yellow

tabs_box = BrightBlue
list_box = Blue
main_box = Blue

settings_category_box = BrightBlue
settings_options_box = Blue
selected_box = White

list_folder = Green
list_file = White

tab_lazy = White
tab_selected = Yellow

settings_entry = White
settings_selected = Green
settings_special = Red

editor_ins = Magenta
editor_cmd = Blue
editor_vis = Cyan
editor_fnd = BrightCyan
editor_lne = BrightCyan

editor_fnd_bg = White
editor_keywords = Magenta
editor_functions = BrightBlue
editor_strings = BrightGreen
editor_numbers = BrightYellow
editor_bool = Cyan
editor_comments = DarkGray
editor_variables = White
editor_operators = BrightCyan
editor_brackets = White
editor_errors = Red

keyview_color = BrightBlue

monitor_cpu_key = Blue
monitor_cpu_val = Magenta
monitor_gpu_key = Blue
monitor_gpu_val = Magenta
monitor_mem_key = Blue
monitor_mem_val = Magenta
monitor_term_key = Blue
monitor_term_val = Magenta
monitor_divider = DarkGray
monitor_bar_bounds = DarkGray

clock_time_color = Green
clock_date_color = Cyan

macrostats_key = Blue
macrostats_val = Magenta
macrostats_err = Green Yellow Red
```

### 2. Config (`config.conf`)
Settings are configured via the built-in **Settings Modal**, but can also be manually edited in `conf/config.conf`.

---

## ▪ Examples

**Basic Auto-Clicker**
```python
# Press 'esc' to stop the macro at any time!
let clicks = 0

loop:
    if isdown(Key:Esc):
        println("Stopped. Total clicks: {clicks}")
        break

    keydown(Key:LMB)
    sleep(10)
    keyup(Key:LMB)
    sleep(40)

    clicks += 1
```

**Persistent Data Storage**
```python
# Load saved data
let data = macrodata()
let runs = data::get("runs", 0)

runs += 1
data::set("runs", runs)

# Save it back to disk
macrodata(data)

println("You have run this macro {runs} times!")
```

**Pixel Bot**
```python
let target_color = Color:ff0055

loop:
    if isdown(Key:Esc):
        break

    let x = mousex()
    let y = mousey()

    # Check screen at mouse position Allow a color variance/tolerance of 10.
    if compixel((x - 2)::abs(), (y - 2)::abs(), target_color, 10):
        println("Target spotted!")

    sleep(50)
```

---

## ▪ Building & Installation

To compile and run NUUI from source, you must have Rust and Cargo installed.

### 1. Dependencies
* **Linux:** Requires `libX11` for pixel reading and mouse tracking (`libx11-dev`). Keyboard simulation relies on `/dev/uinput` (Ensure you have root/sudo privileges or the correct `udev` rules set for the `input` group to use some functions).
* **Windows:** Uses standard `user32`/`gdi32` libraries, requiring no external setup.

### 2. Compiling
```bash
git clone https://github.com/Nexsq/nuuiex.git
cd nuuiex
cargo build --release
```

To run it:
```bash
cargo run --release
```

To build portable:
```bash
cargo build --release --features portable
```

***

huge thanks to [@Grengorio](https://github.com/Grengorio) for bug hunting
