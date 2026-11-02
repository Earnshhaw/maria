use iced::{Element, Length, widget::{Space, button, column, container, grid, image, row, scrollable, text}};
use iced_aw::{ContextMenu, Menu, menu::Item};

use crate::gui::{GameEdit, GameEntry, Message, State, button_card};



const PANESPLIT: f32 = 2 as f32 / 3 as f32;
pub fn game_grid_view(state: &State) -> Element<'_, Message> {
    let grid = scrollable(
        grid(state.game_entries.iter().map(|entry| {
            ContextMenu::new(
                {
                    button(
                        image(entry.value().get_handle())
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .border_radius(25)
                            .filter_method(image::FilterMethod::Linear)
                            .height(900)
                            .height(600),
                    )
                    .style(button_card)
                    .padding(0)
                    .on_press(Message::LaunchGame(entry.key().to_owned()))
                },
                move || {
                    //OVERLAY
                    let button_m = match entry.get_entry() {
                        GameEntry::Local(_) => {
                            column![button("Edit").on_press(Message::GameEditOverlay(
                                GameEdit::EditLocalGame(entry.key().clone())
                            ))]
                        }
                        GameEntry::Steam(_) => {
                            column![]
                        }
                    };
                    let cx = button_m
                        .push(button("Run").on_press(Message::LaunchGame(entry.key().to_owned())))
                        .push(
                            button("Delete").on_press(Message::DeleteEntry(entry.key().to_owned())),
                        );
                    cx.into()
                }, //OVERLAY
            )
            .into()
        }))
        .height(grid::Sizing::AspectRatio(PANESPLIT))
        .fluid(200)
        .spacing(5),
    );

    container(grid).width(Length::Fill).padding(10).into()
}
#[allow(unused)]
pub fn sidebar_view(state: &State) -> Element<'_, Message> {
    let els = text!("SIDEBAR");
    container(els).into()
}

#[allow(unused)]
pub fn topbar_view(state: &State) -> Element<'_, Message> {
    let options = vec![
        Item::new(
            button("Add a local game").on_press(Message::GameEditOverlay(GameEdit::AddLocalGame)),
        )
        .close_on_click(true),
        Item::new(button("Fetch steam library").on_press(Message::FetchSteamGames))
            .close_on_click(true),
    ];
    let cx = iced_aw::menu_bar!((
        text("Add a game"),
        Menu::new(options).width(Length::Fixed(125.0))
    ));
    container(row![Space::new().width(Length::Fill), cx])
        .padding(10)
        .width(Length::Fill)
        .into()
}
