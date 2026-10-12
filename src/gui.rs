use std::{collections::BTreeMap, path::PathBuf, process::ExitStatus};
use iced::{Element, Length, Task, widget::{button, column, container, grid, image::Handle, mouse_area, opaque, pane_grid, pick_list, row, stack, text, text_input}};
use iced_aw::{Menu, menu::{self, Item}};
use rfd::MessageDialogResult::No;
use crate::{boot::PLACEHOLDER_GRID, config::AppConfig, dirs::{app_data_dir, steam_dir}, gather_games::{Error, fetch_steam_games, load_saved_games, save_to_disk}, gui::Message::GameEditOverlay, id::ensure_unique_id, launch::{GameEntry, LocalEntry, Runner, SteamEntry}};

#[derive(Debug, Clone)]
pub struct State {
    game_entries: Vec<(GameEntry, Handle)>,
    runners: Vec<Runner>,
    running_game: Option<GameEntry>,
    config: AppConfig,
    pane: pane_grid::State<Pane>,
    overlay: Overlay,
    game_master: Option<LocalEntry>
}

fn default_game_entry() -> LocalEntry {
    let game_entry = LocalEntry {
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
    GameEditOverlay(GameEditCommand),
}

#[derive(Debug, Clone)]
enum GameEditCommand {
    AddLocalGame,
    EditLocalGame(GameEntry),
    EditGameSetGridRfd,
    EditGameSetBinaryPath(SetMethod),
    EditGameSetWorkingDir(SetMethod),
    EditGameSetRunner(Runner),
    EditGameSetLaunchArguments(String),
    EditGameSetEnvVars(String),
    ConfirmEntry,
    CancelEntry
}

fn game_grid_view(state: &State) -> Element<'_,Message> {
    let grid = grid(
        state.game_entries
            .iter()
            .map(|(entry, handle)| mouse_area(
                button(iced::widget::image(handle).border_radius(10).content_fit(iced::ContentFit::Contain).height(Length::Fill))
                    .on_press(Message::LaunchGame(entry.id_as_string()))).on_right_press(Message::GameEditOverlay(GameEditCommand::EditLocalGame(entry.to_owned())))
            .into()))
    .height(grid::Sizing::AspectRatio(fdiv(2, 3))).fluid(200)
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
            .on_press(Message::GameEditOverlay(GameEditCommand::AddLocalGame)))
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
        let grid = pane_grid(&self.pane, |pane, state, is_maximized| {
                pane_grid::Content::new(match state {
                    Pane::GameGrids => game_grid_view(self),
                    Pane::SidebarView => sidebar_view(self),
                })
            })
            .on_resize(10, Message::PaneResized);
        let x = column![topbar_view(self), grid];
        match &self.overlay {
            Overlay::None => {x.into()},
            Overlay::GameEdit => {modal(x, new_game_modal(self)).into()}
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
            Message::GameEditOverlay(cmd) => game_edit_overlay_branch(self, cmd)
        }
    }
    pub fn new(game_entries: Vec<(GameEntry, Handle)>, runners: Vec<Runner>, config: AppConfig) -> Self {
        let (mut panes, first) = pane_grid::State::new(Pane::SidebarView);
        let (_, split) = panes.split(pane_grid::Axis::Vertical, first, Pane::GameGrids).unwrap();
        panes.resize(split, 0.20);
        State { game_entries, runners, running_game: None, config, pane: panes, overlay: Overlay::None, game_master: None}
    }
    fn temp_entry(&mut self) -> &mut LocalEntry {
       self.game_master.as_mut().unwrap()
    }
}

fn game_edit_overlay_branch(state: &mut State, cmd: GameEditCommand) -> Task<Message> {
    match cmd {
        GameEditCommand::CancelEntry => cancel_entry(state),
        GameEditCommand::ConfirmEntry => confirm_entry(state),
        GameEditCommand::EditGameSetBinaryPath(method) => set_binary_path(state, method),
        GameEditCommand::EditGameSetEnvVars(envs) => set_env_vars(state, envs),
        GameEditCommand::EditGameSetGridRfd => set_grid(state),
        GameEditCommand::EditGameSetLaunchArguments(args) => set_launch_args(state, args),
        GameEditCommand::EditGameSetRunner(runner) => set_runner(state, runner),
        GameEditCommand::EditGameSetWorkingDir(method) => set_working_dir(state, method),
        GameEditCommand::AddLocalGame => add_local_game_menu(state),
        GameEditCommand::EditLocalGame(game) => edit_local_game(state, game),
    }
}
fn edit_local_game(state: &mut State, game: GameEntry) -> Task<Message> {
    match game {
        GameEntry::Local(game) => {
            state.game_master = Some(game);
            state.overlay = Overlay::GameEdit;
            Task::none()
        },
        GameEntry::Steam(_) => {
            Task::none()
        }
    }
}
fn confirm_entry(state: &mut State) -> Task<Message> {
    if let Some(new) = state.game_master.as_ref() {
        if let Some(old) = state.game_entries.iter_mut().find(|(entry, _)| {entry.id_as_string() == new.get_id().to_string()}) {
            *old = (GameEntry::Local(new.to_owned()), Handle::from(new.get_grid()));
        } else {
            let new_entry = (GameEntry::Local(new.to_owned()), Handle::from(new.get_grid()));
            state.game_entries.push(new_entry);
        }
    }
    state.game_master = None;
    state.overlay = Overlay::None;
    Task::done(Message::SyncGamesToDisk)
}
fn cancel_entry(state: &mut State) -> Task<Message> {
    state.overlay = Overlay::None;
    state.game_master = None;
    Task::none()
}
fn set_env_vars(state: &mut State, envs: String) -> Task<Message> {
    let vars: BTreeMap<String, String> = BTreeMap::new();
    let splitted = envs.split(" ").into_iter().fold(vars, |mut acc, a| {let (k, v) = a.split_once("=").unwrap(); acc.insert(k.to_string(), v.to_string()); acc});
    state.temp_entry().env_vars = splitted;
    Task::none()
}
fn set_runner(state: &mut State, runner: Runner) -> Task<Message> {
    state.temp_entry().runner = Some(runner);
    Task::none()
}
fn add_local_game_menu(state: &mut State) -> Task<Message> {
    state.game_master = Some(default_game_entry());
    state.overlay = Overlay::GameEdit; 
    Task::none()
}
fn set_launch_args(state: &mut State, args: String) -> Task<Message> {
    let sep: Vec<String> = args.split(" ").into_iter().map(|s| s.to_string()).collect();
    state.temp_entry().launch_args = sep;
    Task::none()
}
fn set_working_dir(state: &mut State, method: SetMethod) -> Task<Message> {
    match method {
        SetMethod::Rfd => {
            let rfd = rfd::FileDialog::new().pick_folder();
            if let Some(folder) = rfd {
                state.temp_entry().working_dir = folder;
            }
        }
        SetMethod::TextInput(text) => {
            state.temp_entry().working_dir = PathBuf::from(text)
        }
    }
    Task::none()
}
fn set_grid(state: &mut State) -> Task<Message> {
    let rfd = rfd::FileDialog::new().add_filter("jpg", &["jpg", "jpeg", "jpg"]).pick_file();
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
                    state.temp_entry().binary_path = file.clone();
                    return Task::done(Message::GameEditOverlay(GameEditCommand::EditGameSetWorkingDir(SetMethod::TextInput(file.parent().unwrap().to_string_lossy().into_owned()))));
                }
            },
            SetMethod::TextInput(inp) => {
                state.temp_entry().binary_path = PathBuf::from(inp)
            }
        }
    Task::none()
}
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
}
fn launch_game(state: &mut State, strid: String) -> Task<Message> {
    if state.running_game.is_some() {
        return Task::none();
    }
    let running_game = state.game_entries.iter().find(|(entry, _)| entry.id_as_string() == strid);
    if let Some((game, _)) = running_game.cloned() {
        state.running_game = Some(game.clone());
        iced_runtime::task::blocking(move |mut sender| {
            let child = game.run_game().unwrap().wait().map_err(|e| Error::IOError(e.to_string()));
            let _ = sender.try_send(child);
        }).map(Message::GameLaunched)
    } else {
        Task::none()
    }
}
fn synced(state: &mut State, status: Result<(), Error>) -> Task<Message> {
    match status {
        Ok(_) => {},
        Err(e) => eprintln!("{}", e)
    };
    let mut game_entries: Vec<(GameEntry, Handle)> = vec![];
    for entry in load_saved_games().unwrap() {
        match entry {
            GameEntry::Local(game) => {
                if !game.get_grid().try_exists().unwrap() {
                    let handle = Handle::from_path(app_data_dir().join(format!("grids/{}", PLACEHOLDER_GRID)));
                    game_entries.push((GameEntry::Local(game), handle));
                } else {
                    let handle = Handle::from_path(game.get_grid());
                    game_entries.push((GameEntry::Local(game), handle));
                }
            },
            GameEntry::Steam(game) => {
                if !game.get_grid().try_exists().unwrap() {
                    let handle = Handle::from_path(app_data_dir().join(format!("grids/{}", PLACEHOLDER_GRID)));
                    game_entries.push((GameEntry::Steam(game), handle));
                } else {
                    let handle = Handle::from_path(game.get_grid());
                    game_entries.push((GameEntry::Steam(game), handle));
                }
            }
        }
    }
    state.game_entries = game_entries;
    Task::none()
}
fn sync_games(state: &mut State) -> Task<Message> {
    let c = state.game_entries.iter().map(|(entry, _)| entry.clone()).collect();
    Task::perform(async move {
        save_to_disk(c).await
    }, Message::Synced)
}
fn fetch_steam_games_cmd() -> Task<Message> {
    if !steam_dir().exists() {
        return Task::none()
    }
    Task::perform(async move {
        fetch_steam_games().await
    }, Message::SteamGamesFetched)
}
fn steam_games_fetched(state: &mut State, result: Result<Vec<SteamEntry>, Error>) -> Task<Message> {
        match result {
            Ok(entries) => {
                let mut new_entries = Vec::new();
                for entry in entries {
                    if state.game_entries.iter().any(|(e, _)| {
                        match &e {
                            GameEntry::Local(_) => {false},
                            GameEntry::Steam(game) => {game.get_appid() == entry.get_appid()}
                        }
                    }) {
                        continue;
                    }
                    let handle = Handle::from_path(entry.get_grid());
                    new_entries.push((GameEntry::Steam(entry), handle));
                }
                state.game_entries.extend(new_entries);
                Task::done(Message::SyncGamesToDisk)
                
            },
            Err(e) => {
                eprintln!("{}", e);
                Task::none()
            },
        }
}
fn fdiv(a: i32, b: i32) -> f32 {
    a as f32 / b as f32 //if you divide by 0 youre retarded, im not error handling this shit
}
fn modal<'a>(base: impl Into<Element<'a, Message>>, content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    stack!(base.into(),
       opaque(
            content.into()
        )
    )
    .into()
}
fn new_game_modal(state: &State) -> Element<'_, Message> {
    let entry = state.game_master.as_ref().unwrap();
    let poster = button(iced::widget::image(&entry.grid_path).height(100)).on_press(Message::GameEditOverlay(GameEditCommand::EditGameSetGridRfd));
    let binary_path = 
        row![text_input("Path to executable", &entry.get_binary_path().to_string_lossy()).on_input(|i| Message::GameEditOverlay(GameEditCommand::EditGameSetBinaryPath(SetMethod::TextInput(i)))),
            button("rfd").on_press(Message::GameEditOverlay(GameEditCommand::EditGameSetBinaryPath(SetMethod::Rfd)))];
    let working_dir = row![
        text_input("Working directory", &entry.get_working_dir().to_string_lossy()).on_input(|i| Message::GameEditOverlay(GameEditCommand::EditGameSetWorkingDir(SetMethod::TextInput(i)))), 
        button("rfd").on_press(Message::GameEditOverlay(GameEditCommand::EditGameSetWorkingDir(SetMethod::Rfd)))];
    let runner = pick_list(state.runners.as_ref(), entry.runner.as_ref(), |i |Message::GameEditOverlay(GameEditCommand::EditGameSetRunner(i)));
    let launch_args = text_input("Launch arguments", &entry.get_launch_args().join(" ")).on_input(|i| Message::GameEditOverlay(GameEditCommand::EditGameSetLaunchArguments(i)));
    let envs = entry.get_env_vars().iter().map(|(key, value)| format!("{key}={value}")).fold(String::new(), |a, b| {format!("{a} {b}")});
    let envs_variables = text_input("Enviroment variables", &envs).on_input(|i| Message::GameEditOverlay(GameEditCommand::EditGameSetEnvVars(i)));
    let finalization = row![
        button("Confirm").on_press(Message::GameEditOverlay(GameEditCommand::ConfirmEntry)),
        button("Cancel").on_press(Message::GameEditOverlay(GameEditCommand::CancelEntry))
    ];
    container(column![poster, binary_path, working_dir, runner, launch_args, envs_variables, finalization]).into()
}

#[derive(Debug, Clone)]
enum SetMethod {
    Rfd,
    TextInput(String)
}