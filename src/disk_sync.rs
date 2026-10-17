use std::path::PathBuf;
use tokio::fs::{create_dir_all, write};
use crate::{data_model::GameEntry, dirs::app_data_dir, error::Error, misc::GAME_CFG_ENTRY_NAME};

pub async fn save_to_disk(gameentries: Vec<GameEntry>) -> Result<(), Error> {
    for gameentry in gameentries {
        match &gameentry {
            GameEntry::Local(game) => {
                let validated = game_dir_exists(game.get_id()).await?.join(GAME_CFG_ENTRY_NAME);
                let formatted = toml::to_string_pretty(&gameentry)?;
                write(validated, formatted).await?;},
            GameEntry::Steam(game) => {
                let validated = game_dir_exists(game.get_appid()).await?.join(GAME_CFG_ENTRY_NAME);
                let formatted = toml::to_string_pretty(&gameentry)?;
                write(validated, formatted).await?;}
        }}
    Ok(())
}

async fn game_dir_exists(uuid: impl ToString) -> Result<PathBuf, Error> {
    let game_dir = app_data_dir().join(&uuid.to_string());
    if !game_dir.try_exists()? {
        create_dir_all(&game_dir).await?;
    }
    Ok(game_dir)
}
