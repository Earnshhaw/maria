use std::{collections::BTreeMap, path::PathBuf, process::ExitStatus};
use iced::{Element, Length, Task, widget::{button, column, container, grid, image::Handle, opaque, pane_grid, pick_list, row, stack, text, text_input}};
use iced_aw::{ContextMenu, Menu, menu::Item};
use crate::{config::AppConfig, data_model::{GameEntry, LocalEntry, Runner, SteamEntry}, dirs::app_data_dir, disk_sync::{delete_from_disk, save_to_disk}, error::Error, gather_games::{fetch_steam_games, load_saved_games}, id::ensure_unique_id, launch::run_game, misc::PLACEHOLDER_GRID};

#[derive(Debug, Clone)]
pub struct State {
    game_entries: Vec<(GameEntry, Handle)>,
    runners: Vec<Runner>,
    running_game: Option<GameEntry>,
    config: AppConfig,
    pane: pane_grid::State<Pane>,
    overlay: Overlay,
    game_form: Option<GameDraft>,
}

#[derive(Debug, Clone, Default)]
struct GameDraft {
    title: String, 
    id: uuid::Uuid, 
    grid_path: PathBuf,
    binary_path: String, 
    working_dir: String, 
    env_vars: Vec<(String, String)>, 
    wrappers: String,
    launch_args: String, 
    runner: Option<Runner>
}

impl From<GameDraft> for LocalEntry {
    fn from(value: GameDraft) -> Self {
        let vars = value.env_vars.into_iter().fold(BTreeMap::new(), |mut acc, (k, v)| {acc.insert(k, v); acc});
        LocalEntry::new(
            value.title, 
            value.id, 
            value.grid_path, 
            PathBuf::from(value.binary_path), 
            PathBuf::from(value.working_dir), 
            value.wrappers.split_whitespace().map(|e| e.to_string()).collect(),
            vars, 
            value.launch_args.split_whitespace().map(|e| e.to_string()).collect::<Vec<String>>(), 
            value.runner.unwrap_or_default())
    }
}

fn default_game_entry() -> GameDraft {
    let game_entry = GameDraft { 
        grid_path: app_data_dir().join("grids").join(PLACEHOLDER_GRID),
        id: ensure_unique_id(),
        ..Default::default()
    }; 
    
    game_entry
}

#[derive(Debug, Clone)]
enum Overlay {
    None,
    GameEdit
}

#[derive(Debug, Clone, Default)]
enum Pane {
    #[default]
    GameGrids,
    SidebarView
}

#[derive(Debug, Clone)]
pub enum Message {
    FetchSteamGames,
    SteamGamesFetched(Result<Vec<SteamEntry>, Error>),
    SyncGamesToDisk,
    Synced(Result<(), Error>),
    LaunchGame(String),
    GameLaunched(Result<ExitStatus, Error>),
    PaneResized(pane_grid::ResizeEvent),
    GameEditOverlay(GameEdit),
    DeleteEntry(String),
    EntryDeleted(Result<(), Error>)
}

#[derive(Debug, Clone)]
pub enum GameEdit{
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
    CancelEntry
}

#[derive(Debug, Clone)]
pub enum KeyValue {
    Key(usize),
    Value(usize)
}

#[derive(Debug, Clone)]
pub enum SetMethod {
    Rfd,
    TextInput(String)
}

const PANESPLIT: f32 = 2 as f32 / 3 as f32;
fn game_grid_view(state: &State) -> Element<'_,Message> {
    let grid = grid(
        state.game_entries
            .iter()
            .map(|(entry, handle)| {
                ContextMenu::new(
                    {
                        button(iced::widget::image(handle)                       //UNDERLAY
                            .border_radius(10)
                            .content_fit(iced::ContentFit::Contain)
                            .height(Length::Fill))
                            .on_press(Message::LaunchGame(entry.get_id())) //UNDERLAY
                    }, 
                    || {                                                         //OVERLAY
                        let button_m = match entry {
                            GameEntry::Local(_) => {column![button("Edit").on_press(Message::GameEditOverlay(GameEdit::EditLocalGame(entry.clone())))]},
                            GameEntry::Steam(_) => {column![]}
                        };
                        let cx = button_m.push(button("Delete").on_press(Message::DeleteEntry(entry.get_id())));
                        cx.into()
                    }                                                            //OVERLAY
                ).into()
            }))
    .height(grid::Sizing::AspectRatio(PANESPLIT)).fluid(200)
    .spacing(5);

    
    container(grid).width(Length::Fill)
        .padding(10)
        .into()
}
fn sidebar_view(state: &State) -> Element<'_, Message> {
    let els = text!("SIDEBAR");
    container(els)
        .into()
}
fn topbar_view(state: &State) -> Element<'_, Message> {
    let options = vec![
        Item::new(button("Add a local game")
            .on_press(Message::GameEditOverlay(GameEdit::AddLocalGame)))
            .close_on_click(true), 
        Item::new(button("Fetch steam library")
            .on_press(Message::FetchSteamGames))
            .close_on_click(true)];
    let cx = iced_aw::menu_bar!((text("Add a game..."), Menu::new(options).width(Length::Fixed(125.0))));
    container(cx)
        .width(Length::Fill)
        .into()
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
        let x = column![topbar_view(self), grid];
        match &self.overlay {
            Overlay::None => {x.into()},
            Overlay::GameEdit => {modal(x, edit_game_modal(self)).into()}
        }
    }
       
        
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FetchSteamGames => fetch_steam_games_cmd(),
            Message::SteamGamesFetched(result) => steam_games_fetched(self, result),
            Message::SyncGamesToDisk => sync_games(self),
            Message::Synced(status) => synced(self, status),
            Message::LaunchGame(strid) => launch_game(self, strid),
            Message::GameLaunched(status) => game_terminated(self, status),
            Message::PaneResized(resize) => pane_resized(self, resize),
            Message::GameEditOverlay(cmd) => game_edit_overlay_branch(self, cmd),
            Message::DeleteEntry(id) => delete_game(self, id),
            Message::EntryDeleted(result) => game_deleted(self, result),
        }
    }
    pub fn new(game_entries: Vec<(GameEntry, Handle)>, runners: Vec<Runner>, config: AppConfig) -> Self {
        let (mut panes, first) = pane_grid::State::new(Pane::SidebarView);
        let (_, split) = panes.split(pane_grid::Axis::Vertical, first, Pane::GameGrids).unwrap();
        panes.resize(split, 0.20);
        State { game_entries, runners, running_game: None, config, pane: panes, overlay: Overlay::None, game_form: None}
    }
    fn temp_entry(&mut self) -> &mut GameDraft {
       self.game_form.as_mut().unwrap()
    }
    fn get_game_by_id(&self, strid: String) -> Option<GameEntry> {
        self.game_entries.iter().find(|(entry, _)| entry.get_id() == strid).map(|(entry, _)| entry.clone())
    }
}

fn game_edit_overlay_branch(state: &mut State, cmd: GameEdit) -> Task<Message> {
    match cmd {
        GameEdit::CancelEntry => cancel_entry(state),
        GameEdit::ConfirmEntry => confirm_entry(state),
        GameEdit::SetBinary(method) => set_binary_path(state, method),
        GameEdit::SetEnvVars(payload, kv) => set_env_vars(state, payload, kv),
        GameEdit::SetGridRfd => set_grid(state),
        GameEdit::SetLaunchArguments(args) => set_launch_args(state, args),
        GameEdit::SetRunner(runner) => set_runner(state, runner),
        GameEdit::SetWorkingDir(method) => set_working_dir(state, method),
        GameEdit::AddLocalGame => add_local_game_menu(state),
        GameEdit::EditLocalGame(game) => edit_local_game(state, game),
        GameEdit::SetTitle(title) => edit_title(state, title),
        GameEdit::AddEnvField => add_env_field(state),
        GameEdit::RemoveEnvField(i) => remove_env_field(state, i),
        GameEdit::SetWrapper(wrappers) => edit_wrappers(state, wrappers)
    }
}

fn remove_env_field(state: &mut State, i:usize) -> Task<Message> {
    state.temp_entry().env_vars.remove(i);
    Task::none()
}
fn edit_title(state: &mut State, title: String) -> Task<Message> {
    state.temp_entry().title = title;
    Task::none()
}                       //GAMEDRAFT EDIT OPERATIONS
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
                env_vars: game.get_env_vars().iter().map(|(k,v)| {(k.clone(), v.to_string())}).collect(), 
                launch_args: game.get_launch_args().join(" "), 
                runner: Some(game.get_runner().to_owned()) };
            state.game_form = Some(game);
            state.overlay = Overlay::GameEdit;
            Task::none()
        },
        GameEntry::Steam(_) => {
            Task::none()
        }
    }
}
fn edit_wrappers(state: &mut State, payload: String) -> Task<Message> {
    state.temp_entry().wrappers = payload;
    Task::none()
}
fn confirm_entry(state: &mut State) -> Task<Message> {
    if let Some(new) = state.game_form.as_ref() {
        if let Some(old) = state.game_entries.iter_mut().find(|(entry, _)| {entry.get_id() == new.id.to_string()}) {
            *old = (GameEntry::Local(new.to_owned().into()), Handle::from(&new.grid_path));
        } else {
            let new_entry = (GameEntry::Local(new.to_owned().into()), Handle::from(&new.grid_path));
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
    state.temp_entry().env_vars.push((String::new(), String::new()));
    Task::none()
}
fn set_env_vars(state: &mut State, payload: String, kv: KeyValue) -> Task<Message> {
    let envs = &mut state.temp_entry().env_vars;
    match kv {
        KeyValue::Key(i) => {
            if let Some((k, _)) = envs.get_mut(i) {
                *k = payload;
            }
        },
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
fn set_working_dir(state: &mut State, method: SetMethod) -> Task<Message> {
    match method {
        SetMethod::Rfd => {
            let rfd = rfd::FileDialog::new().pick_folder();
            if let Some(folder) = rfd {
                state.temp_entry().working_dir = folder.to_string_lossy().into_owned();
            }
        }
        SetMethod::TextInput(text) => {
            state.temp_entry().working_dir = text
        }
    }
    Task::none()
}
fn set_grid(state: &mut State) -> Task<Message> {
    let rfd = rfd::FileDialog::new().add_filter("pictures", &["jpg", "jpeg", "png"]).pick_file();
    if let Some(img) = rfd {
        let entry = state.temp_entry();
        entry.grid_path = img;
    }
    Task::none()
}
fn set_binary_path(state: &mut State, method: SetMethod) -> Task<Message> {
        match method {
            SetMethod::Rfd => {
                let rfd = rfd::FileDialog::new().pick_file();
                if let Some(file) = rfd {
                    state.temp_entry().binary_path = file.to_string_lossy().into_owned();
                    return Task::done(Message::GameEditOverlay(GameEdit::SetWorkingDir(SetMethod::TextInput(file.parent().unwrap().to_string_lossy().into_owned()))));
                }
            },
            SetMethod::TextInput(inp) => {
                state.temp_entry().binary_path = inp
            }
        }
    Task::none()
}              //GAMEDRAFT EDIT OPERATIONS

fn pane_resized(state: &mut State, resize_event: pane_grid::ResizeEvent) -> Task<Message> {
    state.pane.resize(resize_event.split, resize_event.ratio);
    Task::none()
}

fn game_terminated(state: &mut State, status: Result<ExitStatus, Error>) -> Task<Message> {
    state.running_game.take();
    match status {
        Ok(s) => {
            println!("{}", s.success())
        },
        Err(e) => {eprintln!("{}", e)}
    }
    Task::none()
}   //GAME PROCESS RUN&TRACK CHAIN
fn launch_game(state: &mut State, strid: String) -> Task<Message> {
    if state.running_game.is_some() {
        return Task::none();
    }
    if let Some(game) = state.get_game_by_id(strid) {
        state.running_game = Some(game.clone());
        iced_runtime::task::blocking(move |mut sender| {
            let child = run_game(&game).unwrap().wait().map_err(|e| Error::IOError(e.to_string()));
            let _ = sender.try_send(child);
        }).map(Message::GameLaunched)
    } else {
        Task::none()
    }
}                           //GAME PROCESS RUN&TRACK CHAIN

fn synced(state: &mut State, status: Result<(), Error>) -> Task<Message> {
    match status {
        Ok(_) => {},
        Err(e) => eprintln!("{}", e)
    };
    state.game_entries = process_loaded_games();
    Task::none()
}    //DISK SYNC CHAIN
fn delete_game(state: &mut State, id: String) -> Task<Message> {
    let undesired_entry = state.game_entries.iter().position(|(e, _)| e.get_id() == id);
    if let Some(i) = undesired_entry {
        let (entry, _) = state.game_entries.remove(i);
        Task::perform(async move {
            delete_from_disk(entry).await
        }, Message::EntryDeleted)
    } else {
        Task::none()
    }
}
fn game_deleted(state: &mut State, result: Result<(), Error>) -> Task<Message> {
    match result {
        Ok(_) => {},
        Err(e) => {eprintln!("{}", e)}
    }
    Task::none()
}
fn sync_games(state: &mut State) -> Task<Message> {
    let game_library = state.game_entries.iter().map(|(entry, _)| entry.clone()).collect();
    Task::perform(async move {
        save_to_disk(game_library).await
    }, Message::Synced)
}                           //DISK SYNC CHAIN

fn fetch_steam_games_cmd() -> Task<Message> {
    Task::perform(async move {
        fetch_steam_games().await
    }, Message::SteamGamesFetched)
}                                                        //STEAM GAMES CHAIN
fn steam_games_fetched(state: &mut State, result: Result<Vec<SteamEntry>, Error>) -> Task<Message> {
        match result {
            Ok(entries) => {
                let game_entries: Vec<(GameEntry, Handle)> = entries
                    .into_iter()
                    .filter(|e| {state.get_game_by_id(e.get_appid().to_string()).is_none()})
                    .map(|entry| {
                        let handle = Handle::from_path(entry.get_grid());
                        (GameEntry::Steam(entry), handle)
                    })
                    .collect();

                state.game_entries.extend(game_entries);
                Task::done(Message::SyncGamesToDisk)
                
            },
            Err(e) => {
                eprintln!("{}", e);
                Task::none()
            },
        }
} //STEAM GAMES CHAIN

fn modal<'a>(base: impl Into<Element<'a, Message>>, content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    stack!(base.into(),
       opaque(
            content.into()
        )
    )
    .into()
}
fn edit_game_modal(state: &State) -> Element<'_, Message> {
    let entry = state.game_form.as_ref().unwrap();
    let title = text_input("Title", &entry.title).on_input(|i| Message::GameEditOverlay(GameEdit::SetTitle(i)));
    let poster = button(iced::widget::image(&entry.grid_path).height(100)).on_press(Message::GameEditOverlay(GameEdit::SetGridRfd));
    let binary_path = 
        row![text_input("Path to executable", &entry.binary_path).on_input(|i| Message::GameEditOverlay(GameEdit::SetBinary(SetMethod::TextInput(i)))),
            button("rfd").on_press(Message::GameEditOverlay(GameEdit::SetBinary(SetMethod::Rfd)))];
    let working_dir = row![
        text_input("Working directory", &entry.working_dir).on_input(|i| Message::GameEditOverlay(GameEdit::SetWorkingDir(SetMethod::TextInput(i)))), 
        button("rfd").on_press(Message::GameEditOverlay(GameEdit::SetWorkingDir(SetMethod::Rfd)))];
    let runner = pick_list(state.runners.as_ref(), entry.runner.as_ref(), |i |Message::GameEditOverlay(GameEdit::SetRunner(i)));
    let launch_args = text_input("Launch arguments, for ex. -dx11", &entry.launch_args).on_input(|i| Message::GameEditOverlay(GameEdit::SetLaunchArguments(i)));
    let wrappers = text_input("Wrappers, for ex. mangohud", &entry.wrappers).on_input(|i| Message::GameEditOverlay(GameEdit::SetWrapper(i)));
    let env_fields = entry.env_vars.iter().enumerate().fold(column![], |acc, (i, (k,v))| {
        acc.push(row![
            text_input("Key", k).on_input(move |k| Message::GameEditOverlay(GameEdit::SetEnvVars(k, KeyValue::Key(i)))), 
            text_input("Value", v).on_input(move |v| Message::GameEditOverlay(GameEdit::SetEnvVars(v, KeyValue::Value(i),))),
            button("X").on_press(Message::GameEditOverlay(GameEdit::RemoveEnvField(i)))
        ])
    });
    let env_fields = env_fields.push(button("Add an enviroment variable").on_press(Message::GameEditOverlay(GameEdit::AddEnvField)));
    
    let new_envs = container(env_fields);
    let finalization = row![
        button("Confirm").on_press(Message::GameEditOverlay(GameEdit::ConfirmEntry)),
        button("Cancel").on_press(Message::GameEditOverlay(GameEdit::CancelEntry))
    ];
    container(column![poster, title, binary_path, working_dir, runner, launch_args, wrappers, new_envs, finalization]).into()
}

pub fn process_loaded_games() -> Vec<(GameEntry, Handle)> {
    let mut game_entries: Vec<(GameEntry, Handle)> = vec![];
    let entries = load_saved_games();
    eprintln!("{:#?}", entries);
    
    for entry in entries.unwrap() {
        let handle_path = match &entry {
            GameEntry::Local(g) => {g.get_grid()},
            GameEntry::Steam(g) => {g.get_grid()}
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
