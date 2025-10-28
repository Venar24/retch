use battery::{Manager, State};
use std::fs;
use sysinfo::System;
use toml::Value;

/// Attempt to read the human-friendly distribution name from `/etc/os-release`.
/// Falls back to `None` when the information is unavailable.
fn get_linux_distribution() -> Option<String> {
    if let Ok(content) = fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if line.starts_with("PRETTY_NAME=") {
                return Some(
                    line.trim_start_matches("PRETTY_NAME=")
                        .trim_matches('"')
                        .to_string()
                );
            }
        }
    }
    None
}

/// Compose a one-line summary of the first detected battery, including charge,
/// state, and an ETA if the driver exposes it.
fn get_battery_info() -> Result<String, Box<dyn std::error::Error>> {
    // Initialize battery manager
    let manager = Manager::new()?;
    
    // Get batteries iterator
    let batteries = manager.batteries()?;
    
    // Try to get the first battery
    for battery in batteries {
        let battery = battery?;
        
        // Get percentage (0.0 to 1.0)
        let percentage = battery.state_of_charge().value * 100.0;
        
        // Get battery state (charging, discharging, full, etc.)
        let state = match battery.state() {
            State::Charging => "Charging",
            State::Discharging => "Discharging",
            State::Full => "Full",
            State::Empty => "Empty",
            _ => "Unknown",
        };
        
        // Get time to full/empty if available
        let time_string = if battery.state() == State::Charging {
            if let Some(time) = battery.time_to_full() {
                // Convert seconds to hours and minutes
                let seconds = time.value;
                let hours = (seconds / 3600.0) as u32;
                let minutes = ((seconds % 3600.0) / 60.0) as u32;
                format!(" ({}h {}m until full)", hours, minutes)
            } else {
                String::new()
            }
        } else if battery.state() == State::Discharging {
            if let Some(time) = battery.time_to_empty() {
                // Convert seconds to hours and minutes
                let seconds = time.value;
                let hours = (seconds / 3600.0) as u32;
                let minutes = ((seconds % 3600.0) / 60.0) as u32;
                format!(" ({}h {}m remaining)", hours, minutes)
            } else {
                String::new()
            }
        } else {
            String::new()
        };
        
        return Ok(format!("Battery: {}% ({}){}", percentage as u8, state, time_string));
    }
    
    // No batteries found
    Ok("Battery: Not detected".to_string())
}

/// Convert the total physical memory reported in bytes to whole gigabytes.
fn get_total_memory_gb(system: &System) -> u64 {
    let total_memory_bt = system.total_memory();
    let total_memory_kb = total_memory_bt / 1024;
    let total_memory_mb = total_memory_kb / 1024;
    total_memory_mb / 1024
}

/// Format a concise uptime string in the form `Xd Xh Xm`.
fn format_uptime() -> String {
    let uptime_seconds = System::uptime();
    let days = uptime_seconds / 86_400;
    let hours = (uptime_seconds % 86_400) / 3_600;
    let minutes = (uptime_seconds % 3_600) / 60;
    let mut uptime_parts = Vec::new();
    if days > 0 {
        uptime_parts.push(format!("{}d", days));
    }
    if hours > 0 || !uptime_parts.is_empty() {
        uptime_parts.push(format!("{}h", hours));
    }
    uptime_parts.push(format!("{}m", minutes));
    uptime_parts.join(" ")
}

/// Report the first CPU's brand string and frequency (GHz).
fn get_cpu_info(system: &System) -> Option<String> {
    system.cpus().first().map(|cpu| {
        let brand = cpu.brand();
        let frequency_mhz = cpu.frequency();
        format!("CPU Model: {} @ {:.2} GHz", brand, frequency_mhz as f64 / 1000.0)
    })
}

/// Determine a human-friendly OS label, with Linux distributions resolved via `/etc/os-release`.
fn get_os_info() -> String {
    if cfg!(target_os = "linux") {
        get_linux_distribution().unwrap_or("Linux (Unknown Distro)".to_string())
    } else if cfg!(target_os = "windows") {
        "Windows".to_string()
    } else if cfg!(target_os = "macos") {
        "macOS".to_string()
    } else {
        "Unknown OS".to_string()
    }
}

/// Load the TOML configuration and print it verbatim.
fn print_config(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let parsed: Value = toml::from_str(&content)?;
    println!("{}", parsed);
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_config("src/.config.toml")?;

    let mut system = System::new_all();
    // Refresh system data
    system.refresh_all();

    // Hardware snapshot
    if let Some(cpu_info) = get_cpu_info(&system) {
        println!("{}", cpu_info);
    }

    // Collect system-level facts before printing them together.
    let battery_info = get_battery_info()?;
    let os_info = get_os_info();
    let uptime_string = format_uptime();
    let total_memory_gb = get_total_memory_gb(&system);

    println!(
        "OS: {}
Uptime: {}
Ram: {} Gb
{}",
        os_info, uptime_string, total_memory_gb, battery_info
    );

    Ok(())
}
