use std::{process::ExitStatus};
use iced::{Element, Length, Task, widget::{button, column, container, image::{self, Handle}, mouse_area, pane_grid, row, scrollable, text}};
use crate::{boot::PLACEHOLDER_GRID, config::AppConfig, dirs::{app_data_dir, steam_dir}, gather_games::{Error, fetch_steam_games, load_saved_games, save_to_disk}, launch::{GameEntry, Runner, SteamEntry}, style::{gamegrid_container, sidebar_container, topbar_container}};

#[derive(Debug, Clone)]
pub struct State {
    game_entries: Vec<(GameEntry, Handle)>,
    runners: Vec<Runner>,
    running_game: Option<GameEntry>,
    config: AppConfig,
    pane: pane_grid::State<Pane>
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
}

fn single_grid_entry<'a>(entry: &GameEntry, handle: &Handle) -> Element<'a, Message> {
    container(
        mouse_area(
    column![
        image::Image::new(handle).height(200),
    ]).on_double_click(Message::LaunchGame(entry.id_as_string())))
    .into()
}

fn game_grid_view(state: &State) -> Element<'_,Message> {
    let panels = scrollable(state.game_entries.iter().fold(row![], |acc, (entry, handle)| {
        let pa = acc.push(single_grid_entry(entry, handle));
        pa
    }));
    container(panels)
        .style(gamegrid_container)
        .into()
}

fn sidebar_view(state: &State) -> Element<'_, Message> {
    let els = text!("SIDEBAR");
    container(els)
        .style(sidebar_container)
        .into()
}

fn topbar_view(state: &State) -> Element<'_, Message> {
    let el = container(text!("Top bar. todo"));
    container(el)
        .style(topbar_container).width(Length::Fill)
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
        column![topbar_view(self), grid].into()
    }
       
        
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FetchSteamGames => fetch_steam_games_cmd(),
            Message::SteamGamesFetched(result) => steam_games_fetched(self, result),
            Message::SyncGamesToDisk => sync_games(self),
            Message::Synced(status) => synced(self, status),
            Message::LaunchGame(strid) => launch_game(self, strid),
            Message::GameLaunched(status) => game_terminated(self, status),
            Message::PaneResized(resize) => pane_resized(self, resize)
        }
    }
    pub fn new(game_entries: Vec<(GameEntry, Handle)>, runners: Vec<Runner>, config: AppConfig) -> Self {
        let (mut panes, first) = pane_grid::State::new(Pane::SidebarView);
        let (_, split) = panes.split(pane_grid::Axis::Vertical, first, Pane::GameGrids).unwrap();
        panes.resize(split, 0.25);
        State { game_entries, runners, running_game: None, config, pane: panes}
    }
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