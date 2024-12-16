use sysinfo::System;

fn main() {
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
    
    println!("OS: {}
Ram: {} Gb",
              
              
              std::env::consts::OS,
              total_memory_gb);
}
