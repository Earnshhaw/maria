use std::path::PathBuf;
use uuid::Uuid;

pub fn app_data_dir() -> PathBuf {
    dirs::data_dir().unwrap().join("maria")
}

pub fn prefix_dir(id: &Uuid) -> String {
    app_data_dir().join(id.to_string()).join("prefix").to_string_lossy().into_owned()
}

pub fn app_config_dir() -> PathBuf {
    dirs::config_dir().unwrap().join("maria")
}

#[cfg(target_os = "linux")]
pub fn steam_dir() -> PathBuf {
    dirs::data_local_dir().unwrap().join("Steam")
}

#[cfg(target_os = "windows")]
pub fn steam_dir() -> PathBuf {
    let winpath = PathBuf::from("C:/Program Files (x86)/Steam");
    if winpath.exists() {return Some(winpath);} 
    else {None}
}

#[cfg(target_os = "linux")]
pub fn heroic_dir() -> PathBuf {
    dirs::config_dir().unwrap().join("heroic/tools/proton")
}

#[cfg(target_os = "linux")]
pub fn bottles_dir() -> PathBuf {
    dirs::data_dir().unwrap().join("bottles/runners")
}

pub fn persistence_dirs_valid() -> bool {
    if let None = dirs::data_dir() {eprintln!("Data dir doesn't exist/is not accessible ?"); return false}
    if let None = dirs::config_dir() {eprintln!("Data dir doesn't exist/is not accessible ?"); return false}
    true
}