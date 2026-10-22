use crate::gui::gui_prelude::{
    Error, ExitStatus, GameDraft, GameEntry, Handle, Message, Overlay, PLACEHOLDER_GRID, Runner,
    State, SteamEntry, Task, app_data_dir, default_game_entry, delete_from_disk, fetch_steam_games,
    load_saved_games, pane_grid, run_game, save_to_disk,
};

#[derive(Debug, Clone)]
pub enum KeyValue {
    Key(usize),
    Value(usize),
}

#[derive(Debug, Clone)]
pub enum SetMethod {
    Rfd,
    TextInput(String),
}

#[derive(Debug, Clone)]
pub enum GameEdit {
    AddLocalGame,
    EditLocalGame(GameEntry),
    SetTitle(String),
    SetGridRfd,
    GridSet(Option<rfd::FileHandle>),
    SetBinary(SetMethod),
    BinarySet(Option<rfd::FileHandle>),
    SetWorkingDir(SetMethod),
    WorkingDirSet(Option<rfd::FileHandle>),
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

pub fn game_edit_overlay_branch(state: &mut State, cmd: GameEdit) -> Task<Message> {
    match cmd {
        GameEdit::CancelEntry => cancel_entry(state),
        GameEdit::ConfirmEntry => confirm_entry(state),
        GameEdit::SetBinary(method) => set_binary_dialogue(state, method),
        GameEdit::SetEnvVars(payload, kv) => set_env_vars(state, payload, kv),
        GameEdit::SetGridRfd => set_grid_dialogue(),
        GameEdit::SetLaunchArguments(args) => set_launch_args(state, args),
        GameEdit::SetRunner(runner) => set_runner(state, runner),
        GameEdit::SetWorkingDir(method) => set_working_dialogue(state, method),
        GameEdit::AddLocalGame => add_local_game_menu(state),
        GameEdit::EditLocalGame(game) => edit_local_game(state, game),
        GameEdit::SetTitle(title) => edit_title(state, title),
        GameEdit::AddEnvField => add_env_field(state),
        GameEdit::RemoveEnvField(i) => remove_env_field(state, i),
        GameEdit::SetWrapper(wrappers) => edit_wrappers(state, wrappers),
        GameEdit::GridSet(handle) => grid_set(state, handle),
        GameEdit::BinarySet(handle) => binary_set(state, handle),
        GameEdit::WorkingDirSet(handle) => working_dir_set(state, handle),
    }
}

pub fn remove_env_field(state: &mut State, i: usize) -> Task<Message> {
    state.temp_entry().env_vars.remove(i);
    Task::none()
}
pub fn edit_title(state: &mut State, title: String) -> Task<Message> {
    state.temp_entry().title = title;
    Task::none()
}
fn edit_local_game(state: &mut State, game: GameEntry) -> Task<Message> {
    match game {
        GameEntry::Local(game) => {
            let game = GameDraft {
                title: game.get_title().to_owned(),
                id: *game.get_id(),
                grid_path: game.get_grid().to_owned(),
                binary_path: game.get_binary_path().to_string_lossy().into_owned(),
                working_dir: game.get_working_dir().to_string_lossy().into_owned(),
                wrappers: game.get_wrappers().join(" "),
                env_vars: game
                    .get_env_vars()
                    .iter()
                    .map(|(k, v)| (k.clone(), v.to_string()))
                    .collect(),
                launch_args: game.get_launch_args().join(" "),
                runner: Some(game.get_runner().to_owned()),
            };
            state.game_form = Some(game);
            state.overlay = Overlay::GameEdit;
            Task::none()
        }
        GameEntry::Steam(_) => Task::none(),
    }
}
fn edit_wrappers(state: &mut State, payload: String) -> Task<Message> {
    state.temp_entry().wrappers = payload;
    Task::none()
}
fn confirm_entry(state: &mut State) -> Task<Message> {
    if let Some(new) = state.game_form.as_ref() {
        if let Some(old) = state
            .game_entries
            .iter_mut()
            .find(|(entry, _)| entry.get_id() == new.id.to_string())
        {
            *old = (
                GameEntry::Local(new.to_owned().into()),
                Handle::from(&new.grid_path),
            );
        } else {
            let new_entry = (
                GameEntry::Local(new.to_owned().into()),
                Handle::from(&new.grid_path),
            );
            state.game_entries.push(new_entry);
        }
    }
    state.game_form = None;
    state.overlay = Overlay::None;
    Task::done(Message::SyncGamesToDisk)
}
fn cancel_entry(state: &mut State) -> Task<Message> {
    state.overlay = Overlay::None;
    state.game_form = None;
    Task::none()
}
fn add_env_field(state: &mut State) -> Task<Message> {
    state
        .temp_entry()
        .env_vars
        .push((String::new(), String::new()));
    Task::none()
}
fn set_env_vars(state: &mut State, payload: String, kv: KeyValue) -> Task<Message> {
    let envs = &mut state.temp_entry().env_vars;
    match kv {
        KeyValue::Key(i) => {
            if let Some((k, _)) = envs.get_mut(i) {
                *k = payload;
            }
        }
        KeyValue::Value(i) => {
            if let Some((_, v)) = envs.get_mut(i) {
                *v = payload;
            }
        }
    }
    Task::none()
}
fn set_runner(state: &mut State, runner: Runner) -> Task<Message> {
    state.temp_entry().runner = Some(runner);
    Task::none()
}
fn add_local_game_menu(state: &mut State) -> Task<Message> {
    state.game_form = Some(default_game_entry());
    state.overlay = Overlay::GameEdit;
    Task::none()
}
fn set_launch_args(state: &mut State, args: String) -> Task<Message> {
    state.temp_entry().launch_args = args;
    Task::none()
}
fn working_dir_set(state: &mut State, handle: Option<rfd::FileHandle>) -> Task<Message> {
    if state.game_form.is_none() {
        return Task::none();
    }
    if let Some(picked) = handle {
        state.temp_entry().working_dir = picked.path().to_string_lossy().into_owned();
    }
    Task::none()
}
fn set_working_dialogue(state: &mut State, method: SetMethod) -> Task<Message> {
    match method {
        SetMethod::Rfd => {
            let rfd = rfd::AsyncFileDialog::new().pick_folder();
            return Task::perform(async move { rfd.await }, |i| {
                GameEdit::WorkingDirSet(i).into()
            });
        }
        SetMethod::TextInput(text) => state.temp_entry().working_dir = text,
    }
    Task::none()
}
fn grid_set(state: &mut State, handle: Option<rfd::FileHandle>) -> Task<Message> {
    if state.game_form.is_none() {
        return Task::none();
    }
    if let Some(handle) = handle {
        state.temp_entry().grid_path = handle.path().to_owned();
    }
    Task::none()
}
fn set_grid_dialogue() -> Task<Message> {
    let rfd = rfd::AsyncFileDialog::new().pick_file();
    Task::perform(async move { rfd.await }, |i| GameEdit::GridSet(i).into())
}
fn binary_set(state: &mut State, handle: Option<rfd::FileHandle>) -> Task<Message> {
    if state.game_form.is_none() {
        return Task::none();
    }

    if let Some(picked) = handle {
        let path = picked.path();
        state.temp_entry().binary_path = path.to_string_lossy().into_owned();
        return Task::done(
            GameEdit::SetWorkingDir(SetMethod::TextInput(
                path.parent().unwrap().to_string_lossy().into_owned(),
            ))
            .into(),
        );
    }
    Task::none()
}
fn set_binary_dialogue(state: &mut State, method: SetMethod) -> Task<Message> {
    match method {
        SetMethod::Rfd => {
            let rfd = rfd::AsyncFileDialog::new().pick_file();
            Task::perform(async move { rfd.await }, |i| GameEdit::BinarySet(i).into())
        }
        SetMethod::TextInput(inp) => {
            state.temp_entry().binary_path = inp;
            Task::none()
        }
    }
}

pub fn pane_resized(state: &mut State, resize_event: pane_grid::ResizeEvent) -> Task<Message> {
    state.pane.resize(resize_event.split, resize_event.ratio);
    Task::none()
}

pub fn game_terminated(state: &mut State, status: Result<ExitStatus, Error>) -> Task<Message> {
    state.running_game.take();
    match status {
        Ok(s) => {
            println!("{}", s.success())
        }
        Err(e) => {
            eprintln!("{}", e)
        }
    }
    Task::none()
}
pub fn launch_game(state: &mut State, strid: String) -> Task<Message> {
    if state.running_game.is_some() {
        return Task::none();
    }
    if let Some(game) = state.get_game_by_id(strid) {
        state.running_game = Some(game.clone());
        Task::perform(
            async move {
                let child_spawn = run_game(&game).await;
                match child_spawn {
                    Ok(mut ch) => ch.wait().await.map_err(|e| Error::IOError(e.to_string())),
                    Err(e) => Err(e),
                }
            },
            Message::GameExited,
        )
    } else {
        Task::none()
    }
}

pub fn synced(state: &mut State, status: Result<(), Error>) -> Task<Message> {
    match status {
        Ok(_) => {}
        Err(e) => eprintln!("{}", e),
    };
    state.game_entries = process_loaded_games();
    Task::none()
}
pub fn delete_game(state: &mut State, id: String) -> Task<Message> {
    let undesired_entry = state
        .game_entries
        .iter()
        .position(|(e, _)| e.get_id() == id);
    if let Some(i) = undesired_entry {
        let (entry, _) = state.game_entries.remove(i);
        Task::perform(
            async move { delete_from_disk(entry).await },
            Message::EntryDeleted,
        )
    } else {
        Task::none()
    }
}
pub fn game_deleted(result: Result<(), Error>) -> Task<Message> {
    match result {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e)
        }
    }
    Task::none()
}
pub fn sync_games(state: &mut State) -> Task<Message> {
    let game_library = state
        .game_entries
        .iter()
        .map(|(entry, _)| entry.clone())
        .collect();
    Task::perform(
        async move { save_to_disk(game_library).await },
        Message::Synced,
    )
}

pub fn fetch_steam_games_cmd() -> Task<Message> {
    Task::perform(
        async move { fetch_steam_games().await },
        Message::SteamGamesFetched,
    )
}
pub fn steam_games_fetched(
    state: &mut State,
    result: Result<Vec<SteamEntry>, Error>,
) -> Task<Message> {
    match result {
        Ok(entries) => {
            let game_entries: Vec<(GameEntry, Handle)> = entries
                .into_iter()
                .filter(|e| state.get_game_by_id(e.get_appid().to_string()).is_none())
                .map(|entry| {
                    let handle = Handle::from_path(entry.get_grid());
                    (GameEntry::Steam(entry), handle)
                })
                .collect();

            state.game_entries.extend(game_entries);
            Task::done(Message::SyncGamesToDisk)
        }
        Err(e) => {
            eprintln!("{}", e);
            Task::none()
        }
    }
}

pub fn process_loaded_games() -> Vec<(GameEntry, Handle)> {
    let mut game_entries: Vec<(GameEntry, Handle)> = vec![];
    let entries = load_saved_games();

    for entry in entries.unwrap() {
        let handle_path = match &entry {
            GameEntry::Local(g) => g.get_grid(),
            GameEntry::Steam(g) => g.get_grid(),
        };
        let handle = if !handle_path.exists() {
            Handle::from_path(app_data_dir().join(format!("grids/{}", PLACEHOLDER_GRID)))
        } else {
            Handle::from_path(handle_path)
        };
        game_entries.push((entry, handle));
    }
    game_entries
}
