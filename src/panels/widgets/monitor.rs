use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, Ordering};
use std::thread;
use std::time::Duration;

use crate::{Box, Color, Gradient, Modifier, conf::Config, theme::themecore::Theme};

pub struct MonitorState {
    pub cpu: Arc<AtomicU8>,
    pub gpu: Arc<AtomicU8>,
    pub mem_used: Arc<AtomicU32>,
    pub mem_total: Arc<AtomicU32>,
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
        let is_active = Arc::new(AtomicBool::new(false));
        let is_running = Arc::new(AtomicBool::new(true));

        let c_cpu = cpu.clone();
        let c_gpu = gpu.clone();
        let c_mem_used = mem_used.clone();
        let c_mem_total = mem_total.clone();
        let c_active = is_active.clone();
        let c_running = is_running.clone();

        let thread_handle = thread::spawn(move || {
            #[cfg(target_os = "linux")]
            let (mut prev_idle, mut prev_non_idle) = (0u64, 0u64);

            #[cfg(target_os = "windows")]
            let (mut prev_idle, mut prev_kernel, mut prev_user) = (0u64, 0u64, 0u64);

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

                let gpu_val = get_gpu_usage();

                c_cpu.store(cpu_val, Ordering::Relaxed);
                c_gpu.store(gpu_val, Ordering::Relaxed);
                c_mem_used.store(mem_u, Ordering::Relaxed);
                c_mem_total.store(mem_t, Ordering::Relaxed);

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

        if cur_cpu != self.last_cpu
            || cur_gpu != self.last_gpu
            || cur_mem != self.last_mem
            || term_w != self.last_term_w
            || term_h != self.last_term_h
        {
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

    let mut parts = Vec::new();

    if config.monitor_cpu {
        parts.push(format!("CPU: {:>2}%", state.last_cpu));
    }
    if config.monitor_gpu {
        parts.push(format!("GPU: {:>2}%", state.last_gpu));
    }
    if config.monitor_mem {
        let used = state.last_mem as f32 / 1024.0;
        let total = state.mem_total.load(Ordering::Relaxed) as f32 / 1024.0;
        if total > 0.0 {
            parts.push(format!("MEM: {:.1}/{:.1} GB", used, total));
        } else {
            parts.push(format!("MEM: {:.1} GB", used));
        }
    }
    if config.monitor_term {
        parts.push(format!("TERM: {}x{}", term_w, term_h));
    }

    let text = parts.join("  |  ");
    let text_len = text.chars().count() as u16;

    let inner_w = b.width.saturating_sub(b.padding * 2);
    let inner_h = b.height.saturating_sub(b.padding * 2);

    let x = (inner_w.saturating_sub(text_len) / 2).max(0);
    let y = (inner_h.saturating_sub(1) / 2).max(0);

    b.insert_text(
        &text,
        x as i16,
        y as i16,
        false,
        theme.main_label.clone(),
        Gradient::Solid(Color::None),
        Modifier::Bold,
    );
}
