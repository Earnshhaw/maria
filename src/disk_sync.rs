use crate::{data_model::GameEntry, dirs::{app_data_dir}, error::Error, misc::GAME_CFG_ENTRY_NAME};
use std::{fs::File, path::{Path, PathBuf}};
use tokio::fs::{create_dir_all, remove_dir_all, remove_file, write};

pub async fn save_to_disk(game_entries: Vec<GameEntry>) -> Result<(), Error> {
    for game_entry in game_entries {
        let validated = game_dir_exists(game_entry.get_id())
            .await?
            .join(GAME_CFG_ENTRY_NAME);
        let formatted = toml::to_string_pretty(&game_entry)?;
        write(validated, formatted).await?;
    }
    Ok(())
}

pub async fn delete_from_disk(game_entry: GameEntry) -> Result<(), Error> {
    let undesired_entry_dir = app_data_dir().join(game_entry.get_id());
    let undesired_grid = game_entry.get_grid();
    remove_dir_all(undesired_entry_dir).await?;
    remove_file(undesired_grid).await?;
    Ok(())
}

async fn game_dir_exists(uuid: impl ToString) -> Result<PathBuf, Error> {
    let game_dir = app_data_dir().join(&uuid.to_string());
    if !game_dir.try_exists()? {
        create_dir_all(&game_dir).await?;
    }
    Ok(game_dir)
}

pub fn save_grid(from: &Path, to: &Path) -> Result<(), Error> {
    std::fs::copy(from, to)?;
    Ok(())
}
