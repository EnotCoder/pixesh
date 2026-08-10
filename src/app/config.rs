use std::fs;
use std::path::PathBuf;

// ── конфиг приложения (~/.config/pixesh/settings.txt) ──

fn config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/".into());
    PathBuf::from(home).join(".config").join("pixesh").join("settings.txt")
}

pub(crate) fn load_welcome_show_again() -> bool {
    match fs::read_to_string(config_path()) {
        Ok(s) => !s.lines().any(|l| l.trim() == "welcome_show_again=0"),
        Err(_) => true,
    }
}

pub(crate) fn save_welcome_show_again(show: bool) {
    if let Some(dir) = config_path().parent() {
        let _ = fs::create_dir_all(dir);
    }
    let line = format!("welcome_show_again={}\n", if show { 1 } else { 0 });
    let _ = fs::write(config_path(), line);
}