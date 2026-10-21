use crate::gui::gui_prelude::*;

pub fn modal<'a>(
    base: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    stack!(base.into(), opaque(content.into())).into()
}
pub fn edit_game_modal(state: &State) -> Element<'_, Message> {
    let entry = state.game_form.as_ref().unwrap();
    let title = text_input("Title", &entry.title).on_input(|i| GameEdit::SetTitle(i).into());
    let poster = button(iced::widget::image(&entry.grid_path).height(100))
        .on_press(GameEdit::SetGridRfd.into());
    let binary_path = row![
        text_input("Path to executable", &entry.binary_path)
            .on_input(|i| GameEdit::SetBinary(SetMethod::TextInput(i)).into()),
        button("rfd").on_press(GameEdit::SetBinary(SetMethod::Rfd).into())
    ];
    let working_dir = row![
        text_input("Working directory", &entry.working_dir)
            .on_input(|i| GameEdit::SetWorkingDir(SetMethod::TextInput(i)).into()),
        button("rfd").on_press(GameEdit::SetWorkingDir(SetMethod::Rfd).into())
    ];
    let runner = pick_list(state.runners.as_ref(), entry.runner.as_ref(), |i| {
        GameEdit::SetRunner(i).into()
    });
    let launch_args = text_input("Launch arguments, for ex. -dx11", &entry.launch_args)
        .on_input(|i| GameEdit::SetLaunchArguments(i).into());
    let wrappers = text_input("Wrappers, for ex. mangohud", &entry.wrappers)
        .on_input(|i| GameEdit::SetWrapper(i).into());
    let env_fields =
        entry
            .env_vars
            .iter()
            .enumerate()
            .fold(iced::widget::column![], |acc, (i, (k, v))| {
                acc.push(row![
                    text_input("Key", k).on_input(move |k| GameEdit::SetEnvVars(
                        k,
                        KeyValue::Key(i)
                    )
                    .into()),
                    text_input("Value", v).on_input(move |v| GameEdit::SetEnvVars(
                        v,
                        KeyValue::Value(i),
                    )
                    .into()),
                    button("X").on_press(GameEdit::RemoveEnvField(i).into())
                ])
            });
    let env_fields = env_fields
        .push(button("Add an enviroment variable").on_press(GameEdit::AddEnvField.into()));

    let new_envs = container(env_fields);
    let finalization = row![
        button("Confirm").on_press(GameEdit::ConfirmEntry.into()),
        button("Cancel").on_press(GameEdit::CancelEntry.into())
    ];
    container(iced::widget::column![
        poster,
        title,
        binary_path,
        working_dir,
        runner,
        launch_args,
        wrappers,
        new_envs,
        finalization
    ])
    .into()
}
