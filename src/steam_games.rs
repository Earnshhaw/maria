use std::{fs::{read_dir}, path::PathBuf};
use crate::{gather_runners::steam_dir, launch::{GameEntry, Launch}};
use acf_parser::{parser::parse_acf};

fn manifests_dir(steam_dir: &PathBuf) -> PathBuf {
    steam_dir.join("steamapps")
}

pub fn fetch_steam_games() -> Result<Vec<GameEntry>, Box<dyn std::error::Error>> {
    let steam_dir = if cfg!(target_os = "linux") {steam_dir().unwrap()} else {PathBuf::new()};
    let manifest_paths: Vec<PathBuf> = read_dir(manifests_dir(&steam_dir))?
        .filter_map(Result::ok)
        .filter_map(|e| e.path().extension().and_then(|ext| {if ext.to_string_lossy().contains("acf") {Some(e.path())} else {None}}))
        .collect();

    let mut game_entries = Vec::new();
    for path in manifest_paths {
         let result = parse_acf(&path.to_string_lossy());
         let contents = match result {
             Ok(s) => {s},
             Err(e) => {eprintln!("{e}"); continue}
         };
         let root_contents = &contents.entries[0].expressions;
         let title: String = match filter_non_games(&root_contents["name"]) {
             Some(name) => {name},
             None => continue
         };
         let game_entry = GameEntry {
             title: title,
             launch: Launch::Steam { appid: root_contents["appid"].parse()? }
         };
         game_entries.push(game_entry);
    }
    
    Ok(game_entries)
}

fn filter_non_games(name: &str) -> Option<String> {
    let no_no_words = ["Proton", "Runtime", "Linux Runtime", "Steamworks Common", "EasyAntiCheat"];
    if no_no_words.iter().any(|forbidden| name.contains(forbidden)) {
        return None;
    }
    Some(name.to_owned())
}