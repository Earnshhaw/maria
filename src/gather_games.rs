#![allow(unused)]
use std::{fs::{create_dir_all, read_dir, read_to_string, write}, path::{Path, PathBuf}, sync::Arc};
use acf_parser::parser::parse_acf;
use reqwest::Client;
use steamgriddb_api::QueryType;
use tokio::{fs::File, io::AsyncWriteExt};
use uuid::Uuid;
use crate::{api_keys::steamgriddb_api_key, launch::{GameEntry, LocalEntry, SteamEntry}, dirs::{app_data_dir, steam_dir}};

const GAME_CFG_ENTRY_NAME: &str = "entry.toml";
const PLACEHOLDER_GRID: &str = "blank.jpg";

impl GameEntry {
    pub fn save_to_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        match &self {
            GameEntry::Local(game) => {
                let validated = game_dir_exists(game.get_id())?.join(GAME_CFG_ENTRY_NAME);
                let formatted = toml::to_string_pretty(self)?;
                write(validated, formatted)?;},
            GameEntry::Steam(game) => {
                let validated = game_dir_exists(game.get_appid())?.join(GAME_CFG_ENTRY_NAME);
                let formatted = toml::to_string_pretty(self)?;
                write(validated, formatted)?;}
        }
        Ok(())
    }
}

fn game_dir_exists(uuid: impl ToString) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let game_dir = app_data_dir().unwrap().join(&uuid.to_string());
    if !game_dir.try_exists()? {
        create_dir_all(&game_dir)?;
    }
    Ok(game_dir)
}

pub fn load_saved_games() -> Result<Vec<GameEntry>, Box<dyn std::error::Error>> {
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
            let entry: GameEntry = toml::from_str(&unserd_contents)?;
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
         let title = match is_valid_game(&root_contents["name"]) {
             Some(name) => {name},
             None => continue
         };
         let id: u64 = root_contents["appid"].parse()?;
         let grid_path = app_data_dir().unwrap().join(format!("grids/{}.jpg", id));
         
         let game_entry = SteamEntry::new(id, title, grid_path);
         game_entries.push(game_entry);
    }
    
    Ok(game_entries)
}

fn is_valid_game(name: &str) -> Option<String> {
    let no_no_words = ["Proton", "Runtime", "Linux Runtime", "Steamworks Common", "EasyAntiCheat"]; //enough for now
    if no_no_words.iter().any(|forbidden| name.contains(forbidden)) {
        return None;
    }
    Some(name.to_owned())
}
