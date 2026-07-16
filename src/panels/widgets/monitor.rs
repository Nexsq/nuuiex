use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::{Box, Color, Gradient, Modifier, Style, conf::Config, theme::themecore::Theme};

pub struct MonitorState {
    pub cpu: Arc<AtomicU8>,
    pub gpu: Arc<AtomicU8>,
    pub mem_used: Arc<AtomicU32>,
    pub mem_total: Arc<AtomicU32>,
    pub cpu_hist: Arc<Mutex<Vec<u8>>>,
    pub gpu_hist: Arc<Mutex<Vec<u8>>>,
    pub mem_hist: Arc<Mutex<Vec<u8>>>,
    pub tick_count: Arc<AtomicU32>,
    pub last_tick: u32,
    pub last_cpu: u8,
    pub last_gpu: u8,
    pub last_mem: u32,
    pub last_term_w: u16,
    pub last_term_h: u16,
    pub is_active: Arc<AtomicBool>,
    pub is_running: Arc<AtomicBool>,
    thread_handle: Option<thread::JoinHandle<()>>,
}

impl MonitorState {
    pub fn new() -> Self {
        let cpu = Arc::new(AtomicU8::new(0));
        let gpu = Arc::new(AtomicU8::new(0));
        let mem_used = Arc::new(AtomicU32::new(0));
        let mem_total = Arc::new(AtomicU32::new(0));
        let cpu_hist = Arc::new(Mutex::new(vec![0; 32]));
        let gpu_hist = Arc::new(Mutex::new(vec![0; 32]));
        let mem_hist = Arc::new(Mutex::new(vec![0; 32]));
        let tick_count = Arc::new(AtomicU32::new(0));
        let is_active = Arc::new(AtomicBool::new(false));
        let is_running = Arc::new(AtomicBool::new(true));

        let c_cpu = cpu.clone();
        let c_gpu = gpu.clone();
        let c_mem_used = mem_used.clone();
        let c_mem_total = mem_total.clone();
        let c_cpu_hist = cpu_hist.clone();
        let c_gpu_hist = gpu_hist.clone();
        let c_mem_hist = mem_hist.clone();
        let c_tick_count = tick_count.clone();
        let c_active = is_active.clone();
        let c_running = is_running.clone();

        let thread_handle = thread::spawn(move || {
            #[cfg(target_os = "linux")]
            let (mut prev_idle, mut prev_non_idle) = (0u64, 0u64);

            #[cfg(target_os = "windows")]
            let (mut prev_idle, mut prev_kernel, mut prev_user) = (0u64, 0u64, 0u64);

            let mut last_gpu_val = 0;
            let mut last_gpu_time = Instant::now().checked_sub(Duration::from_secs(10)).unwrap();

            while c_running.load(Ordering::Relaxed) {
                if !c_active.load(Ordering::Relaxed) {
                    thread::sleep(Duration::from_millis(200));
                    continue;
                }

                #[cfg(target_os = "linux")]
                let (cpu_val, mem_u, mem_t) = get_linux_metrics(&mut prev_idle, &mut prev_non_idle);

                #[cfg(target_os = "windows")]
                let (cpu_val, mem_u, mem_t) =
                    get_windows_metrics(&mut prev_idle, &mut prev_kernel, &mut prev_user);

                #[cfg(not(any(target_os = "linux", target_os = "windows")))]
                let (cpu_val, mem_u, mem_t) = (0, 0, 0);

                let now = Instant::now();
                if now.duration_since(last_gpu_time).as_secs() >= 1 {
                    last_gpu_val = get_gpu_usage();
                    last_gpu_time = now;
                }
                let gpu_val = last_gpu_val;

                c_cpu.store(cpu_val, Ordering::Relaxed);
                c_gpu.store(gpu_val, Ordering::Relaxed);
                c_mem_used.store(mem_u, Ordering::Relaxed);
                c_mem_total.store(mem_t, Ordering::Relaxed);

                {
                    let mut ch = c_cpu_hist.lock().unwrap();
                    ch.push(cpu_val);
                    if ch.len() > 32 {
                        ch.remove(0);
                    }
                }
                {
                    let mut gh = c_gpu_hist.lock().unwrap();
                    gh.push(gpu_val);
                    if gh.len() > 32 {
                        gh.remove(0);
                    }
                }
                {
                    let mut mh = c_mem_hist.lock().unwrap();
                    let mem_pct = if mem_t > 0 {
                        ((mem_u as u64 * 100) / mem_t as u64) as u8
                    } else {
                        0
                    };
                    mh.push(mem_pct);
                    if mh.len() > 32 {
                        mh.remove(0);
                    }
                }

                c_tick_count.fetch_add(1, Ordering::Relaxed);

                for _ in 0..5 {
                    if !c_running.load(Ordering::Relaxed) || !c_active.load(Ordering::Relaxed) {
                        break;
                    }
                    thread::sleep(Duration::from_millis(200));
                }
            }
        });

        Self {
            cpu,
            gpu,
            mem_used,
            mem_total,
            cpu_hist,
            gpu_hist,
            mem_hist,
            tick_count,
            last_tick: 0,
            last_cpu: 255,
            last_gpu: 255,
            last_mem: u32::MAX,
            last_term_w: 0,
            last_term_h: 0,
            is_active,
            is_running,
            thread_handle: Some(thread_handle),
        }
    }

    pub fn set_active(&mut self, active: bool) {
        self.is_active.store(active, Ordering::Relaxed);
    }

    pub fn tick(&mut self, term_w: u16, term_h: u16) -> bool {
        let cur_cpu = self.cpu.load(Ordering::Relaxed);
        let cur_gpu = self.gpu.load(Ordering::Relaxed);
        let cur_mem = self.mem_used.load(Ordering::Relaxed);
        let cur_tick = self.tick_count.load(Ordering::Relaxed);

        if cur_tick != self.last_tick
            || cur_cpu != self.last_cpu
            || cur_gpu != self.last_gpu
            || cur_mem != self.last_mem
            || term_w != self.last_term_w
            || term_h != self.last_term_h
        {
            self.last_tick = cur_tick;
            self.last_cpu = cur_cpu;
            self.last_gpu = cur_gpu;
            self.last_mem = cur_mem;
            self.last_term_w = term_w;
            self.last_term_h = term_h;
            true
        } else {
            false
        }
    }
}

impl Drop for MonitorState {
    fn drop(&mut self) {
        self.is_running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(target_os = "linux")]
fn get_linux_metrics(prev_idle: &mut u64, prev_non_idle: &mut u64) -> (u8, u32, u32) {
    let mut cpu_usage = 0;
    if let Ok(stat) = std::fs::read_to_string("/proc/stat") {
        if let Some(line) = stat.lines().next() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() > 8 {
                let user: u64 = parts[1].parse().unwrap_or(0);
                let nice: u64 = parts[2].parse().unwrap_or(0);
                let system: u64 = parts[3].parse().unwrap_or(0);
                let idle: u64 = parts[4].parse().unwrap_or(0);
                let iowait: u64 = parts[5].parse().unwrap_or(0);
                let irq: u64 = parts[6].parse().unwrap_or(0);
                let softirq: u64 = parts[7].parse().unwrap_or(0);
                let steal: u64 = parts[8].parse().unwrap_or(0);

                let idle_time = idle + iowait;
                let non_idle_time = user + nice + system + irq + softirq + steal;
                let total_time = idle_time + non_idle_time;

                let total_d = total_time.saturating_sub(*prev_idle + *prev_non_idle);
                let idle_d = idle_time.saturating_sub(*prev_idle);

                if total_d > 0 {
                    cpu_usage = ((total_d - idle_d) * 100 / total_d) as u8;
                }

                *prev_idle = idle_time;
                *prev_non_idle = non_idle_time;
            }
        }
    }

    let mut mem_total: u32 = 0;
    let mut mem_avail: u32 = 0;
    if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                mem_total = line
                    .split_whitespace()
                    .nth(1)
                    .unwrap_or("0")
                    .parse::<u32>()
                    .unwrap_or(0)
                    / 1024;
            } else if line.starts_with("MemAvailable:") {
                mem_avail = line
                    .split_whitespace()
                    .nth(1)
                    .unwrap_or("0")
                    .parse::<u32>()
                    .unwrap_or(0)
                    / 1024;
            }
        }
    }
    let mem_used = mem_total.saturating_sub(mem_avail);

    (cpu_usage, mem_used, mem_total)
}

#[cfg(target_os = "windows")]
fn get_windows_metrics(
    prev_idle: &mut u64,
    prev_kernel: &mut u64,
    prev_user: &mut u64,
) -> (u8, u32, u32) {
    use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
    use windows_sys::Win32::System::Threading::GetSystemTimes;

    let mut cpu_usage = 0;
    unsafe {
        let mut idle = std::mem::zeroed();
        let mut kernel = std::mem::zeroed();
        let mut user = std::mem::zeroed();
        if GetSystemTimes(&mut idle, &mut kernel, &mut user) != 0 {
            let idle_time = (idle.dwHighDateTime as u64) << 32 | (idle.dwLowDateTime as u64);
            let kernel_time = (kernel.dwHighDateTime as u64) << 32 | (kernel.dwLowDateTime as u64);
            let user_time = (user.dwHighDateTime as u64) << 32 | (user.dwLowDateTime as u64);

            let sys_diff = (kernel_time + user_time).saturating_sub(*prev_kernel + *prev_user);
            let idle_diff = idle_time.saturating_sub(*prev_idle);

            if sys_diff > 0 {
                cpu_usage = ((sys_diff.saturating_sub(idle_diff)) * 100 / sys_diff) as u8;
            }

            *prev_idle = idle_time;
            *prev_kernel = kernel_time;
            *prev_user = user_time;
        }
    }

    let mut mem_used = 0;
    let mut mem_total = 0;
    unsafe {
        let mut mem_stat: MEMORYSTATUSEX = std::mem::zeroed();
        mem_stat.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
        if GlobalMemoryStatusEx(&mut mem_stat) != 0 {
            mem_total = (mem_stat.ullTotalPhys / 1024 / 1024) as u32;
            let mem_avail = (mem_stat.ullAvailPhys / 1024 / 1024) as u32;
            mem_used = mem_total.saturating_sub(mem_avail);
        }
    }

    (cpu_usage, mem_used, mem_total)
}

fn get_gpu_usage() -> u8 {
    #[cfg(target_os = "windows")]
    use std::os::windows::process::CommandExt;

    let mut cmd = std::process::Command::new("nvidia-smi");
    cmd.args(&[
        "--query-gpu=utilization.gpu",
        "--format=csv,noheader,nounits",
    ]);

    #[cfg(target_os = "windows")]
    cmd.creation_flags(0x08000000);

    if let Ok(output) = cmd.output() {
        if let Ok(s) = String::from_utf8(output.stdout) {
            if let Ok(usage) = s.trim().parse::<u8>() {
                return usage;
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
            let mut max_usage = 0;
            let mut found = false;

            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if let Some(name) = path.file_name() {
                    let name_str = name.to_string_lossy();
                    if name_str.starts_with("card") && !name_str.contains('-') {
                        let busy_path = path.join("device/gpu_busy_percent");
                        if let Ok(s) = std::fs::read_to_string(&busy_path) {
                            if let Ok(usage) = s.trim().parse::<u8>() {
                                max_usage = max_usage.max(usage);
                                found = true;
                            }
                        }
                    }
                }
            }

            if found {
                return max_usage;
            }
        }
    }

    0
}

fn push_text(
    cells: &mut Vec<(char, Style)>,
    text: &str,
    fg: &Gradient,
    bg: &Gradient,
    md: Modifier,
) {
    let len = text.chars().count();
    for (i, c) in text.chars().enumerate() {
        cells.push((
            c,
            Style {
                fg: fg.color_at(i, len),
                bg: bg.color_at(i, len),
                md,
            },
        ));
    }
}

fn push_bar(
    cells: &mut Vec<(char, Style)>,
    usage: u8,
    width: usize,
    fg: &Gradient,
    bounds: &Gradient,
    mode: &str,
) {
    let chars = [' ', '▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'];
    let frac = (usage as u32 * (width as u32 * 8)) / 100;
    let full = (frac / 8) as usize;
    let rem = (frac % 8) as usize;

    if mode == "caps" {
        cells.push((
            '[',
            Style {
                fg: bounds.color_at(0, 1),
                bg: Color::None,
                md: Modifier::Bold,
            },
        ));
    }

    for i in 0..width {
        let c = if i < full {
            chars[8]
        } else if i == full {
            chars[rem]
        } else {
            ' '
        };

        let bg_color = if mode == "background" {
            bounds.color_at(i, width)
        } else {
            Color::None
        };

        cells.push((
            c,
            Style {
                fg: fg.color_at(i, width),
                bg: bg_color,
                md: Modifier::None,
            },
        ));
    }

    if mode == "caps" {
        cells.push((
            ']',
            Style {
                fg: bounds.color_at(0, 1),
                bg: Color::None,
                md: Modifier::Bold,
            },
        ));
    }
}

fn braille_char(v1: u8, v2: u8) -> char {
    let h1 = match v1 {
        0 => 0,
        1..=24 => 1,
        25..=49 => 2,
        50..=74 => 3,
        _ => 4,
    };
    let h2 = match v2 {
        0 => 0,
        1..=24 => 1,
        25..=49 => 2,
        50..=74 => 3,
        _ => 4,
    };
    let left = match h1 {
        1 => 0x40,
        2 => 0x44,
        3 => 0x46,
        4 => 0x47,
        _ => 0,
    };
    let right = match h2 {
        1 => 0x80,
        2 => 0xA0,
        3 => 0xB0,
        4 => 0xB8,
        _ => 0,
    };
    std::char::from_u32(0x2800 + left + right).unwrap_or(' ')
}

fn push_graph(
    cells: &mut Vec<(char, Style)>,
    hist: &[u8],
    width: usize,
    fg: &Gradient,
    bounds: &Gradient,
    mode: &str,
) {
    let req_hist = width * 2;
    let mut padded = vec![0; req_hist];

    let take_count = hist.len().min(req_hist);
    let offset = hist.len() - take_count;
    let pad_offset = req_hist - take_count;

    for i in 0..take_count {
        padded[pad_offset + i] = hist[offset + i];
    }

    if mode == "caps" {
        cells.push((
            '[',
            Style {
                fg: bounds.color_at(0, 1),
                bg: Color::None,
                md: Modifier::Bold,
            },
        ));
    }

    for i in 0..width {
        let v1 = padded[i * 2];
        let v2 = padded[i * 2 + 1];
        let c = braille_char(v1, v2);

        let bg_color = if mode == "background" {
            bounds.color_at(i, width)
        } else {
            Color::None
        };

        cells.push((
            c,
            Style {
                fg: fg.color_at(i, width),
                bg: bg_color,
                md: Modifier::None,
            },
        ));
    }

    if mode == "caps" {
        cells.push((
            ']',
            Style {
                fg: bounds.color_at(0, 1),
                bg: Color::None,
                md: Modifier::Bold,
            },
        ));
    }
}

pub fn draw(
    state: &MonitorState,
    b: &mut Box,
    config: &Config,
    theme: &Theme,
    term_w: u16,
    term_h: u16,
) {
    if b.width == 0 || b.height == 0 {
        return;
    }

    let mut groups: Vec<Vec<(char, Style)>> = Vec::new();
    let bg_none = Gradient::Solid(Color::None);

    let cpu_icon = if config.monitor_icons {
        "◈ "
    } else {
        "CPU: "
    };
    let gpu_icon = if config.monitor_icons {
        "◆ "
    } else {
        "GPU: "
    };
    let mem_icon = if config.monitor_icons {
        "▣  "
    } else {
        "MEM: "
    };
    let term_icon = if config.monitor_icons {
        "■ "
    } else {
        "TERM: "
    };
    let bar_w = config.monitor_bar_width.clamp(4, 16);

    if config.monitor_cpu != "off" {
        let mut g = Vec::new();
        push_text(
            &mut g,
            cpu_icon,
            &theme.monitor_cpu_key,
            &bg_none,
            Modifier::Bold,
        );
        if config.monitor_cpu == "pct" || config.monitor_cpu.starts_with("pct") {
            let spacing = if config.monitor_cpu.starts_with("pct") {
                " "
            } else {
                ""
            };
            push_text(
                &mut g,
                &format!("{:>2}%{}", state.last_cpu, spacing),
                &theme.monitor_cpu_val,
                &bg_none,
                Modifier::None,
            );
        }
        if config.monitor_cpu == "bar" || config.monitor_cpu == "pctbar" {
            push_bar(
                &mut g,
                state.last_cpu,
                bar_w,
                &theme.monitor_cpu_val,
                &theme.monitor_bar_bounds,
                &config.monitor_bar_mode,
            );
        }
        if config.monitor_cpu == "graph" || config.monitor_cpu == "pctgraph" {
            let hist = state.cpu_hist.lock().unwrap().clone();
            push_graph(
                &mut g,
                &hist,
                bar_w,
                &theme.monitor_cpu_val,
                &theme.monitor_bar_bounds,
                &config.monitor_bar_mode,
            );
        }
        groups.push(g);
    }

    if config.monitor_gpu != "off" {
        let mut g = Vec::new();
        push_text(
            &mut g,
            gpu_icon,
            &theme.monitor_gpu_key,
            &bg_none,
            Modifier::Bold,
        );
        if config.monitor_gpu == "pct" || config.monitor_gpu.starts_with("pct") {
            let spacing = if config.monitor_gpu.starts_with("pct") {
                " "
            } else {
                ""
            };
            push_text(
                &mut g,
                &format!("{:>2}%{}", state.last_gpu, spacing),
                &theme.monitor_gpu_val,
                &bg_none,
                Modifier::None,
            );
        }
        if config.monitor_gpu == "bar" || config.monitor_gpu == "pctbar" {
            push_bar(
                &mut g,
                state.last_gpu,
                bar_w,
                &theme.monitor_gpu_val,
                &theme.monitor_bar_bounds,
                &config.monitor_bar_mode,
            );
        }
        if config.monitor_gpu == "graph" || config.monitor_gpu == "pctgraph" {
            let hist = state.gpu_hist.lock().unwrap().clone();
            push_graph(
                &mut g,
                &hist,
                bar_w,
                &theme.monitor_gpu_val,
                &theme.monitor_bar_bounds,
                &config.monitor_bar_mode,
            );
        }
        groups.push(g);
    }

    if config.monitor_mem != "off" {
        let mut g = Vec::new();
        push_text(
            &mut g,
            mem_icon,
            &theme.monitor_mem_key,
            &bg_none,
            Modifier::Bold,
        );

        let used = state.last_mem as f32 / 1024.0;
        let total = state.mem_total.load(Ordering::Relaxed) as f32 / 1024.0;
        let mem_pct = if total > 0.0 {
            ((used / total) * 100.0) as u8
        } else {
            0
        };

        if config.monitor_mem == "used" {
            let s = if total > 0.0 {
                format!("{:.1}/{:.1} GB", used, total)
            } else {
                format!("{:.1} GB", used)
            };
            push_text(&mut g, &s, &theme.monitor_mem_val, &bg_none, Modifier::None);
        } else {
            if config.monitor_mem == "pct" || config.monitor_mem.starts_with("pct") {
                let spacing = if config.monitor_mem.starts_with("pct") {
                    " "
                } else {
                    ""
                };
                push_text(
                    &mut g,
                    &format!("{:>2}%{}", mem_pct, spacing),
                    &theme.monitor_mem_val,
                    &bg_none,
                    Modifier::None,
                );
            }
            if config.monitor_mem == "bar" || config.monitor_mem == "pctbar" {
                push_bar(
                    &mut g,
                    mem_pct,
                    bar_w,
                    &theme.monitor_mem_val,
                    &theme.monitor_bar_bounds,
                    &config.monitor_bar_mode,
                );
            }
            if config.monitor_mem == "graph" || config.monitor_mem == "pctgraph" {
                let hist = state.mem_hist.lock().unwrap().clone();
                push_graph(
                    &mut g,
                    &hist,
                    bar_w,
                    &theme.monitor_mem_val,
                    &theme.monitor_bar_bounds,
                    &config.monitor_bar_mode,
                );
            }
        }
        groups.push(g);
    }

    if config.monitor_term == "on" {
        let mut g = Vec::new();
        push_text(
            &mut g,
            term_icon,
            &theme.monitor_term_key,
            &bg_none,
            Modifier::Bold,
        );
        push_text(
            &mut g,
            &format!("{}x{}", term_w, term_h),
            &theme.monitor_term_val,
            &bg_none,
            Modifier::None,
        );
        groups.push(g);
    }

    let mut final_cells = Vec::new();
    let show_div = config.monitor_divider == "show";

    for (i, mut g) in groups.into_iter().enumerate() {
        if i > 0 {
            if show_div {
                push_text(
                    &mut final_cells,
                    "  |  ",
                    &theme.monitor_divider,
                    &bg_none,
                    Modifier::None,
                );
            } else {
                push_text(&mut final_cells, "   ", &bg_none, &bg_none, Modifier::None);
            }
        }
        final_cells.append(&mut g);
    }

    let inner_w = b.width.saturating_sub(b.padding * 2);
    let inner_h = b.height.saturating_sub(b.padding * 2);

    let total_len = final_cells.len() as u16;
    let x = (inner_w.saturating_sub(total_len) / 2).max(0) + b.padding;
    let y = (inner_h.saturating_sub(1) / 2).max(0) + b.padding;

    for (i, (c, style)) in final_cells.into_iter().enumerate() {
        let cx = x + i as u16;
        if cx < b.width.saturating_sub(b.padding) {
            b.put_cell(crate::Cell { c, s: style }, cx, y);
        }
    }
}
