#![allow(unused)]
use std::{fs::{create_dir_all, read_dir, read_to_string, write}, path::PathBuf};
use acf_parser::parser::parse_acf;
use uuid::Uuid;
use crate::{gather_runners::steam_dir, launch::{LocalEntry, SteamEntry, app_data_dir}};

const GAME_CFG_ENTRY_NAME: &str = "entry.toml";

impl LocalEntry {
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let validated = validated_game_dir(self.get_id())?.join(GAME_CFG_ENTRY_NAME);
        let formatted = toml::to_string_pretty(&self)?;
        write(validated, formatted)?;
             
        Ok(())
    }
}

fn validated_game_dir(uuid: &Uuid) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let game_dir = app_data_dir().unwrap().join(&uuid.to_string());
    if !game_dir.try_exists()? {
        create_dir_all(&game_dir)?;
    }
    Ok(game_dir)
}

pub fn fetch_local_games() -> Result<Vec<LocalEntry>, Box<dyn std::error::Error>> {
    let app_data_dir = app_data_dir().unwrap();
    let game_folders: Vec<PathBuf> = read_dir(&app_data_dir)?
        .filter_map(Result::ok)
        .filter_map(|e| {if e.path().is_dir() {Some(e.path())} else {None}}).collect();

    let mut game_entries = Vec::new();
    for game_folder_path in game_folders {
        let entry_file = read_dir(game_folder_path)?
            .filter_map(Result::ok)
            .find(|e| e.file_name().to_string_lossy() == GAME_CFG_ENTRY_NAME);
        if let Some(entry) = entry_file {
            let unserd_contents = read_to_string(entry.path())?;
            let entry: LocalEntry = toml::from_str(&unserd_contents)?;
            game_entries.push(entry);
        }
    }
    
    Ok(game_entries)
}

fn manifests_dir(steam_dir: &PathBuf) -> PathBuf {
    steam_dir.join("steamapps")
}

pub fn fetch_steam_games() -> Result<Vec<SteamEntry>, Box<dyn std::error::Error>> {
    let steam_dir = steam_dir().unwrap();
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
         let game_entry = SteamEntry {appid: root_contents["appid"].parse()?, title};
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