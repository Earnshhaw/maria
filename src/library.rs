use std::{fs::{create_dir_all, write}, path::PathBuf};
use uuid::Uuid;
use crate::launch::{GameCfg, app_data_dir};

impl GameCfg {
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let entry_file = validated_game_dir(self.uuid())?.join("entry.toml");
        let new_cfg = toml::to_string_pretty(self)?;
        write(entry_file, new_cfg)?;
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

