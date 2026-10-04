use std::{collections::HashSet, fs::{DirEntry, read_dir}, hash::Hash, path::{Path, PathBuf}};

fn steam_dir() -> Option<PathBuf> {
    let home_dir = dirs::data_local_dir();
    home_dir.map(|dir| dir.join("Steam"))
}

fn heroic_dir() -> Option<PathBuf> {
    let cfg_dir = dirs::config_local_dir();
    cfg_dir.map(|dir| dir.join("heroic/tools/proton"))
}

fn bottles_dir() -> Option<PathBuf> {
    let data_dir = dirs::data_local_dir();
    data_dir.map(|dir| dir.join("bottles/runners"))
}

fn scan_dir(protons_dir: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let dir_entries: Vec<DirEntry> = read_dir(protons_dir)?.filter_map(|e| e.ok()).collect();
    let mut proton_versions = vec![];
    for dir in dir_entries {
        if !dir.path().is_dir() {
            continue;
        }
        
        let dir_contents: Vec<DirEntry> = read_dir(dir.path())?.filter_map(|e| e.ok()).collect();
        if valid_proton_dir(&dir_contents) {
            proton_versions.push(dir.path());
        }
    }
    Ok(proton_versions)
}


fn valid_proton_dir(dir_contents: &Vec<DirEntry>) -> bool {
    if dir_contents.into_iter().any(|e| e.file_name().to_string_lossy() == "proton") {
        return true;
    } else {
        return false;
    }
}

pub fn init_runners() -> Vec<RunnerEntry> {
    let mut runners: Vec<PathBuf> = vec![];
    if let Some(steam_dir) = steam_dir() {
        if let Ok(steam_proton) = scan_dir(&steam_dir.join("steamapps/common")) {
            runners.extend(steam_proton);
        }
        if let Ok(compat_proton) = scan_dir(&steam_dir.join("compatibilitytools.d")) {
            runners.extend(compat_proton);
        }
    }
    if let Some(hgl) = heroic_dir() {
        if let Ok(steam_proton) = scan_dir(&hgl) {
            runners.extend(steam_proton);
        }
    }
    if let Some(bottles) = bottles_dir() {
        if let Ok(bottles_proton) = scan_dir(&bottles) {
            runners.extend(bottles_proton)
        }
    }
    let accumulated_runners = process_paths(runners);
    dedup_runners(accumulated_runners)
}

fn process_paths(paths: Vec<PathBuf>) -> Vec<RunnerEntry> {
    paths.into_iter().map(|raw_path| RunnerEntry::new(raw_path.file_name().unwrap().to_string_lossy(), &raw_path)).collect()
}

fn dedup_runners(runners: Vec<RunnerEntry>) -> Vec<RunnerEntry> {
    let mut unique_runners: Vec<RunnerEntry> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    
    for runner in runners {
        if seen.insert(runner.name().to_owned()) {
            unique_runners.push(runner);
        }
    }
    unique_runners 
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct RunnerEntry {
    name: String,
    path: PathBuf,
}

impl RunnerEntry {
    pub fn new(name: impl Into<String>, path: impl Into<PathBuf>) -> RunnerEntry {
        RunnerEntry { name: name.into(), path: path.into() }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
}