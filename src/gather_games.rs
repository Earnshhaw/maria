use crate::{
    api_keys::steamgriddb_api_key,
    data_model::{GameEntry, SteamEntry},
    dirs::{app_data_dir, steam_dir},
    error::Error,
    misc::GAME_CFG_ENTRY_NAME,
};
use acf_parser::parser::parse_acf;
use reqwest::Client;
use std::{
    fs::{create_dir_all, read_dir, read_to_string},
    path::PathBuf,
    sync::Arc,
};
use steamgriddb_api::{
    QueryType,
    query_parameters::{MimeType, Platform},
};
use tokio::{fs::File, io::AsyncWriteExt};

pub fn load_saved_games() -> Result<Vec<GameEntry>, Error> {
    let app_data_dir = app_data_dir();

    if !app_data_dir.try_exists()? {
        create_dir_all(&app_data_dir)?;
    }

    let game_folders: Vec<PathBuf> = read_dir(&app_data_dir)?
        .filter_map(Result::ok)
        .filter_map(|e| {
            if e.path().is_dir() {
                Some(e.path())
            } else {
                None
            }
        })
        .collect();

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

pub async fn fetch_steam_games() -> Result<Vec<SteamEntry>, Error> {
    if !steam_dir().try_exists()? {
        return Err(Error::IOError("Weird error".to_string())); //maybe redundant ?
    }

    let steam_client = Arc::new(steamgriddb_api::Client::new(steamgriddb_api_key()));
    let client = Client::new();
    let steam_dir = steam_dir();
    let manifest_paths: Vec<PathBuf> = read_dir(manifests_dir(&steam_dir))?
        .filter_map(Result::ok)
        .filter_map(|e| {
            e.path().extension().and_then(|ext| {
                if ext.to_string_lossy().contains("acf") {
                    Some(e.path())
                } else {
                    None
                }
            })
        })
        .collect();

    let mut game_entries = Vec::new();
    for path in manifest_paths {
        let result = parse_acf(&path.to_string_lossy());
        let contents = match result {
            Ok(s) => s,
            Err(e) => {
                eprintln!("{e}");
                continue;
            }
        };
        let root_contents = &contents.entries[0].expressions;
        let title = match is_valid_game(&root_contents["name"]) {
            Some(name) => name,
            None => continue,
        };
        let id: u64 = root_contents["appid"]
            .parse()
            .map_err(|_| Error::ParseError)?;
        let grid_path = app_data_dir().join("grids");
        if !grid_path.try_exists()? {
            create_dir_all(&grid_path)?;
        }
        let mut grid_entry_path = grid_path.join(format!("{}.png", id));
        if !grid_entry_path.try_exists()? {
            let arc_stmdb = Arc::clone(&steam_client);
            if let Err(_) = get_grid_for_id(id, client.clone(), arc_stmdb).await {
                grid_entry_path = app_data_dir().join("grids/blank.png");
            }
        }

        let game_entry = SteamEntry::new(id, title, grid_entry_path);
        game_entries.push(game_entry);
    }

    Ok(game_entries)
}

async fn get_grid_for_id(
    id: u64,
    client: Client,
    steamclient: Arc<steamgriddb_api::Client>,
) -> Result<(), Error> {
    println!("{}\n", id);
    let mut res = steamclient
        .get_images_for_platform_id(
            &Platform::Steam,
            &id.to_string(),
            &QueryType::Grid(Some(
                steamgriddb_api::query_parameters::GridQueryParameters {
                    mimes: Some(&[MimeType::Png]),
                    ..Default::default()
                },
            )),
        )
        .await
        .map_err(|e| Error::GridDBError(e.to_string()))?;
    let chosen_img = res.remove(rand::random_range(0..res.len()));
    let response = client.get(&chosen_img.url).send().await?.bytes().await?;
    let grid_path = app_data_dir().join(format!("grids/{}.png", id));
    let mut file = File::create_new(grid_path).await?;
    file.write_all(&response).await?;

    Ok(())
}

fn is_valid_game(name: &str) -> Option<String> {
    let no_no_words = [
        "Proton",
        "Runtime",
        "Linux Runtime",
        "Steamworks Common",
        "EasyAntiCheat",
    ]; //enough for now
    if no_no_words.iter().any(|forbidden| name.contains(forbidden)) {
        return None;
    }
    Some(name.to_owned())
}
