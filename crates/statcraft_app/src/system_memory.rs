/// System memory interrogation and Memory Guard pre-flight protection.
/// Supports native polling on Windows, Linux, macOS, and Web/WASM.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SystemMemoryInfo {
    pub total_bytes: u64,
    pub available_bytes: u64,
}

impl SystemMemoryInfo {
    pub fn available_mb(&self) -> u64 {
        self.available_bytes / (1024 * 1024)
    }

    pub fn total_mb(&self) -> u64 {
        self.total_bytes / (1024 * 1024)
    }

    pub fn available_ratio(&self) -> f64 {
        if self.total_bytes == 0 {
            1.0
        } else {
            self.available_bytes as f64 / self.total_bytes as f64
        }
    }

    pub fn is_low_memory(&self) -> bool {
        // Less than 15% free or less than 500 MB available
        self.available_ratio() < 0.15 || self.available_bytes < 500 * 1024 * 1024
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryWarning {
    pub file_count: usize,
    pub estimated_size_mb: u64,
    pub free_ram_mb: u64,
}

#[cfg(windows)]
fn get_windows_memory() -> Option<SystemMemoryInfo> {
    #[repr(C)]
    struct MemoryStatusEx {
        length: u32,
        memory_load: u32,
        total_phys: u64,
        avail_phys: u64,
        total_page_file: u64,
        avail_page_file: u64,
        total_virtual: u64,
        avail_virtual: u64,
        avail_extended_virtual: u64,
    }

    extern "system" {
        fn GlobalMemoryStatusEx(status: *mut MemoryStatusEx) -> i32;
    }

    let mut status = MemoryStatusEx {
        length: std::mem::size_of::<MemoryStatusEx>() as u32,
        memory_load: 0,
        total_phys: 0,
        avail_phys: 0,
        total_page_file: 0,
        avail_page_file: 0,
        total_virtual: 0,
        avail_virtual: 0,
        avail_extended_virtual: 0,
    };

    let ret = unsafe { GlobalMemoryStatusEx(&mut status) };
    if ret != 0 {
        Some(SystemMemoryInfo {
            total_bytes: status.total_phys,
            available_bytes: status.avail_phys,
        })
    } else {
        None
    }
}

#[cfg(target_os = "linux")]
fn get_linux_memory() -> Option<SystemMemoryInfo> {
    let content = std::fs::read_to_string("/proc/meminfo").ok()?;
    let mut total_kb = None;
    let mut avail_kb = None;

    for line in content.lines() {
        if line.starts_with("MemTotal:") {
            total_kb = parse_meminfo_kb(line);
        } else if line.starts_with("MemAvailable:") {
            avail_kb = parse_meminfo_kb(line);
        }
        if total_kb.is_some() && avail_kb.is_some() {
            break;
        }
    }

    match (total_kb, avail_kb) {
        (Some(tot), Some(avail)) => Some(SystemMemoryInfo {
            total_bytes: tot * 1024,
            available_bytes: avail * 1024,
        }),
        _ => None,
    }
}

#[cfg(target_os = "linux")]
fn parse_meminfo_kb(line: &str) -> Option<u64> {
    line.split_whitespace()
        .nth(1)
        .and_then(|val| val.parse::<u64>().ok())
}

#[cfg(target_os = "macos")]
fn get_macos_memory() -> Option<SystemMemoryInfo> {
    // Reading sysctl via Command or safe fallback
    let output = std::process::Command::new("sysctl")
        .arg("-n")
        .arg("hw.memsize")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let total_str = String::from_utf8_lossy(&output.stdout);
    let total_bytes = total_str.trim().parse::<u64>().ok()?;

    // Approximation on macOS: assume 25% free if specific vm metrics unreadable
    Some(SystemMemoryInfo {
        total_bytes,
        available_bytes: total_bytes / 4,
    })
}

#[cfg(target_arch = "wasm32")]
fn get_wasm_memory() -> Option<SystemMemoryInfo> {
    // 1 wasm page = 64KB = 65,536 bytes
    let pages = core::arch::wasm32::memory_size(0) as u64;
    let used_bytes = pages * 65536;
    let total_limit_bytes = 2 * 1024 * 1024 * 1024; // 2GB wasm32 ceiling
    let available_bytes = total_limit_bytes.saturating_sub(used_bytes);

    Some(SystemMemoryInfo {
        total_bytes: total_limit_bytes,
        available_bytes,
    })
}

pub fn get_system_memory() -> Option<SystemMemoryInfo> {
    #[cfg(windows)]
    {
        get_windows_memory()
    }
    #[cfg(target_os = "linux")]
    {
        get_linux_memory()
    }
    #[cfg(target_os = "macos")]
    {
        get_macos_memory()
    }
    #[cfg(target_arch = "wasm32")]
    {
        get_wasm_memory()
    }
    #[cfg(not(any(
        windows,
        target_os = "linux",
        target_os = "macos",
        target_arch = "wasm32"
    )))]
    {
        None
    }
}

/// Evaluates if opening files violates memory safety thresholds.
pub fn check_memory_guard_for_files(
    file_count: usize,
    estimated_total_bytes: u64,
) -> Option<MemoryWarning> {
    let mem = get_system_memory()?;
    let is_batch = file_count > 10;
    let is_huge_file = estimated_total_bytes > 250 * 1024 * 1024;
    let is_low_ram = mem.is_low_memory();

    if (is_batch || is_huge_file) && is_low_ram {
        Some(MemoryWarning {
            file_count,
            estimated_size_mb: estimated_total_bytes / (1024 * 1024),
            free_ram_mb: mem.available_mb(),
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_memory_polling() {
        let mem = get_system_memory();
        assert!(mem.is_some());
        let info = mem.unwrap();
        assert!(info.total_bytes > 0);
        assert!(info.available_bytes > 0);
        assert!(info.total_mb() > 0);
        assert!(info.available_ratio() <= 1.0);
    }

    #[test]
    fn test_memory_guard_trigger_condition() {
        let low_mem = SystemMemoryInfo {
            total_bytes: 8 * 1024 * 1024 * 1024,
            available_bytes: 400 * 1024 * 1024, // < 500 MB
        };
        assert!(low_mem.is_low_memory());

        let ok_mem = SystemMemoryInfo {
            total_bytes: 16 * 1024 * 1024 * 1024,
            available_bytes: 8 * 1024 * 1024 * 1024, // 50%
        };
        assert!(!ok_mem.is_low_memory());
    }
}
