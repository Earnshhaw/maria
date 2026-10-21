use crate::gui::gui_prelude::*;

#[derive(Debug, Clone)]
pub struct State {
    pub game_entries: Vec<(GameEntry, Handle)>,
    pub runners: Vec<Runner>,
    pub running_game: Option<GameEntry>,
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
            value.grid_path,
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
    LaunchGame(String),
    GameExited(Result<ExitStatus, Error>),
    PaneResized(pane_grid::ResizeEvent),
    GameEditOverlay(GameEdit),
    DeleteEntry(String),
    EntryDeleted(Result<(), Error>),
}

#[derive(Debug, Clone)]
pub enum GameEdit {
    AddLocalGame,
    EditLocalGame(GameEntry),
    SetTitle(String),
    SetGridRfd,
    SetBinary(SetMethod),
    SetWorkingDir(SetMethod),
    SetRunner(Runner),
    SetLaunchArguments(String),
    SetEnvVars(String, KeyValue),
    AddEnvField,
    RemoveEnvField(usize),
    SetWrapper(String),
    ConfirmEntry,
    CancelEntry,
}

impl From<GameEdit> for Message {
    fn from(value: GameEdit) -> Self {
        Message::GameEditOverlay(value)
    }
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
            Message::LaunchGame(strid) => launch_game(self, strid),
            Message::GameExited(status) => game_terminated(self, status),
            Message::PaneResized(resize) => pane_resized(self, resize),
            Message::GameEditOverlay(cmd) => game_edit_overlay_branch(self, cmd),
            Message::DeleteEntry(id) => delete_game(self, id),
            Message::EntryDeleted(result) => game_deleted(self, result),
        }
    }
    pub fn new(
        game_entries: Vec<(GameEntry, Handle)>,
        runners: Vec<Runner>,
        config: AppConfig,
    ) -> Self {
        let (mut panes, first) = pane_grid::State::new(Pane::SidebarView);
        let (_, split) = panes
            .split(pane_grid::Axis::Vertical, first, Pane::GameGrids)
            .unwrap();
        panes.resize(split, 0.20);
        State {
            game_entries,
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
    pub fn get_game_by_id(&self, strid: String) -> Option<GameEntry> {
        self.game_entries
            .iter()
            .find(|(entry, _)| entry.get_id() == strid)
            .map(|(entry, _)| entry.clone())
    }
    pub fn game_entries(&self) -> &Vec<(GameEntry, Handle)> {
        &self.game_entries
    }
}
