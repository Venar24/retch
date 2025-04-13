use sysinfo::System;
use battery::{Manager, State};
use std::fs;

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
// Define the battery function outside of main
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut system = System::new_all();
    // Refresh system data
    system.refresh_all();
    
    // Get total memory in KB
    let total_memory_bt = system.total_memory();
    // Convert to MB or GB for better readability
    let total_memory_kb = total_memory_bt / 1024;
    let total_memory_mb = total_memory_kb / 1024;
    let total_memory_gb = total_memory_mb / 1024;
    
    //get cpu average utils
    if let Some(cpu) = system.cpus().first() {
        let brand = cpu.brand();
        let frequency_mhz = cpu.frequency(); // Frequency in MHz
        println!("CPU Model: {} @ {:.2} GHz", brand, frequency_mhz as f64 / 1000.0);
    }
    
    // Get battery information
    let battery_info = get_battery_info()?;
    
// Get OS Info
let os_info = if cfg!(target_os = "linux") {
    get_linux_distribution().unwrap_or("Linux (Unknown Distro)".to_string())
} else if cfg!(target_os = "windows") {
    "Windows".to_string()
} else if cfg!(target_os = "macos") {
    "macOS".to_string()
} else {
    "Unknown OS".to_string()
};


    println!("OS: {}
Ram: {} Gb
{}",
        os_info,
        total_memory_gb,
        battery_info
    );
    
    Ok(())
}