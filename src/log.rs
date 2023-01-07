pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Cmd,
}

pub fn log(level: LogLevel, message: String, debug_enabled: bool) {
    match level {
        LogLevel::Debug => {
            if debug_enabled {
                eprintln!("[DEBUG] {}", message);
            }
        }
        LogLevel::Cmd => {
            eprintln!("[CMD] {}", message);
        }
        LogLevel::Info => {
            eprintln!("[INFO] {}", message);
        }
        LogLevel::Warn => {
            eprintln!("[WARN] {}", message);
        }
    }
}
