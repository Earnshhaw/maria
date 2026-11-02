use crate::{
    data_model::{Game, GameId},
    gui::{
        AppConfig, Error, GameEdit, LocalEntry, PLACEHOLDER_GRID, Runner, SteamEntry, app_data_dir,
        delete_game, edit_game_modal, fetch_steam_games_cmd, game_deleted,
        game_edit_overlay_branch, game_grid_view, game_terminated, grid_saved, launch_game, modal,
        pane_resized, sidebar_view, steam_games_fetched, sync_games, synced, topbar_view,
    },
    id::ensure_unique_id,
};
use dashmap::DashMap;
use iced::{Element, Task, widget::pane_grid};
use std::{collections::BTreeMap, path::PathBuf, process::ExitStatus};

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct State {
    pub game_entries: DashMap<GameId, Game>,
    pub runners: Vec<Runner>,
    pub running_game: Option<String>,
    pub config: AppConfig,
    pub pane: pane_grid::State<Pane>,
    pub overlay: Overlay,
    pub game_form: Option<GameDraft>,
}

#[derive(Debug, Clone, Default)]
pub struct GameDraft {
    pub title: String,
    pub id: uuid::Uuid,
    pub grid_path: PathBuf,
    pub binary_path: String,
    pub working_dir: String,
    pub env_vars: Vec<(String, String)>,
    pub wrappers: String,
    pub launch_args: String,
    pub runner: Option<Runner>,
}

impl From<GameDraft> for LocalEntry {
    fn from(value: GameDraft) -> Self {
        let vars = value
            .env_vars
            .into_iter()
            .fold(BTreeMap::new(), |mut acc, (k, v)| {
                acc.insert(k, v);
                acc
            });
        LocalEntry::new(
            value.title,
            value.id,
            app_data_dir()
                .join("grids")
                .join(value.id.to_string())
                .with_extension(value.grid_path.extension().unwrap()),
            PathBuf::from(value.binary_path),
            PathBuf::from(value.working_dir),
            value
                .wrappers
                .split_whitespace()
                .map(|e| e.to_string())
                .collect(),
            vars,
            value
                .launch_args
                .split_whitespace()
                .map(|e| e.to_string())
                .collect::<Vec<String>>(),
            value.runner.unwrap_or_default(),
        )
    }
}

pub fn default_game_entry() -> GameDraft {
    let game_entry = GameDraft {
        grid_path: app_data_dir().join("grids").join(PLACEHOLDER_GRID),
        id: ensure_unique_id(),
        ..Default::default()
    };

    game_entry
}

#[derive(Debug, Clone)]
pub enum Overlay {
    None,
    GameEdit,
}

#[derive(Debug, Clone, Default)]
pub enum Pane {
    #[default]
    GameGrids,
    SidebarView,
}

#[derive(Debug, Clone)]
pub enum Message {
    FetchSteamGames,
    SteamGamesFetched(Result<Vec<SteamEntry>, Error>),
    SyncGamesToDisk,
    Synced(Result<(), Error>),
    LaunchGame(GameId),
    GameExited(Result<ExitStatus, Error>),
    PaneResized(pane_grid::ResizeEvent),
    GameEditOverlay(GameEdit),
    DeleteEntry(GameId),
    EntryDeleted(Result<(), Error>),
    GridSaved(Result<u64, Error>),
}

impl State {
    pub fn view(&self) -> Element<'_, Message> {
        let grid = pane_grid(&self.pane, |_, state, _| {
            pane_grid::Content::new(match state {
                Pane::GameGrids => game_grid_view(self),
                Pane::SidebarView => sidebar_view(self),
            })
        })
        .on_resize(10, Message::PaneResized);
        let x = iced::widget::column![topbar_view(self), grid];
        match &self.overlay {
            Overlay::None => x.into(),
            Overlay::GameEdit => modal(x, edit_game_modal(self)).into(),
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FetchSteamGames => fetch_steam_games_cmd(),
            Message::SteamGamesFetched(result) => steam_games_fetched(self, result),
            Message::SyncGamesToDisk => sync_games(self),
            Message::Synced(status) => synced(self, status),
            Message::LaunchGame(id) => launch_game(self, id),
            Message::GameExited(status) => game_terminated(self, status),
            Message::PaneResized(resize) => pane_resized(self, resize),
            Message::GameEditOverlay(cmd) => game_edit_overlay_branch(self, cmd),
            Message::DeleteEntry(id) => delete_game(self, id),
            Message::EntryDeleted(result) => game_deleted(result),
            Message::GridSaved(result) => grid_saved(result),
        }
    }
    pub fn new(
        game_entries: DashMap<GameId, Game>,
        runners: Vec<Runner>,
        config: AppConfig,
    ) -> Self {
        let (mut panes, first) = pane_grid::State::new(Pane::SidebarView);
        let (_, split) = panes
            .split(pane_grid::Axis::Vertical, first, Pane::GameGrids)
            .unwrap();
        panes.resize(split, 0.20);
        State {
            game_entries: game_entries,
            runners,
            running_game: None,
            config,
            pane: panes,
            overlay: Overlay::None,
            game_form: None,
        }
    }
    pub fn temp_entry(&mut self) -> &mut GameDraft {
        self.game_form.as_mut().unwrap()
    }
}
