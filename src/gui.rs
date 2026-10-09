use iced::{Element, Length, Task, widget::{button, column, container, image::{self, Handle}, row, scrollable, text}};

use crate::{gather_games::{CError, fetch_steam_games}, launch::{GameEntry, LocalEntry, Runner, SteamEntry}};

#[derive(Debug, Clone, Default)]
pub struct State {
    pub game_entries: Vec<(GameEntry, Handle)>,
    pub runners: Vec<Runner>,
}

#[derive(Debug, Clone)]
pub enum Message {
    FetchSteamGames,
    SteamGamesFetched(Result<Vec<SteamEntry>, CError>)
}

fn panel_el<'a>(entry: &GameEntry, handle: &Handle) -> Element<'a, Message> {
    container(
    column![
        image::Image::new(handle).height(200),
        text(entry.get_name()),
    ]).into()
}

impl State {
    pub fn view(&self) -> Element<'_, Message> {
        let panels = scrollable(self.game_entries.iter().fold(column![], |acc, (entry, handle)| {
            let pa = acc.push(panel_el(entry, handle));
            pa
        })).height(Length::Fill);
        let fetch_steam = button("Steam").on_press(Message::FetchSteamGames);
        
        row![fetch_steam, panels].into()
    }      
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FetchSteamGames => fetch_steam_games_cmd(self),
            Message::SteamGamesFetched(result) => steam_games_fetches(self, result)
        }
    }
}

fn fetch_steam_games_cmd(state: &mut State) -> Task<Message> {
    Task::perform(async move {
        fetch_steam_games().await
    }, Message::SteamGamesFetched)
}
fn steam_games_fetches(state: &mut State, result: Result<Vec<SteamEntry>, CError>) -> Task<Message> {
        match result {
            Ok(entries) => {
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
                    state.game_entries.push((GameEntry::Steam(entry), handle));
                }
            },
            Err(e) => {
                eprintln!("{}", e);
            },
        }
        Task::none() 
}