use std::path::PathBuf;
use uuid::Uuid;

pub fn app_data_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|e| e.join("maria"))
}

pub fn prefix_dir(id: &Uuid) -> String {
    let data_dir = app_data_dir().expect("");
    data_dir.join(id.to_string()).join("prefix").to_string_lossy().into_owned()
}

pub fn app_config_dir() -> PathBuf {
    dirs::config_dir().unwrap().join("maria")
}

#[cfg(target_os = "linux")]
pub fn steam_dir() -> Option<PathBuf> {
    dirs::data_local_dir().map(|dir| dir.join("Steam"))
}

#[cfg(target_os = "windows")]
pub fn steam_dir() -> Option<PathBuf> {
    let winpath = PathBuf::from("C:/Program Files (x86)/Steam");
    if !winpath.exists() {Some(winpath)} 
    None
}

#[cfg(target_os = "linux")]
pub fn heroic_dir() -> Option<PathBuf> {
    dirs::config_local_dir().map(|dir| dir.join("heroic/tools/proton"))
}

#[cfg(target_os = "linux")]
pub fn bottles_dir() -> Option<PathBuf> {
    dirs::data_local_dir().map(|dir| dir.join("bottles/runners"))
}