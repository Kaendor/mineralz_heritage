//! A loading screen during which game assets are loaded if necessary.
//! This reduces stuttering, especially for audio on Wasm.

use bevy::prelude::*;
use bevy_asset_loader::loading_state::{
    LoadingState, LoadingStateAppExt, config::ConfigureLoadingState,
};

use crate::{
    demo::{level::LevelAssets, player::PlayerAssets},
    menus::{Menu, credits::CreditsAssets},
    screens::Screen,
    theme::{interaction::InteractionAssets, prelude::*},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Loading), spawn_loading_screen);

    app.add_loading_state(
        LoadingState::new(Screen::Loading)
            .load_collection::<LevelAssets>()
            .load_collection::<PlayerAssets>()
            .continue_to_state(Screen::Gameplay),
    );
    app.add_loading_state(
        LoadingState::new(Screen::Splash)
            .load_collection::<InteractionAssets>()
            .continue_to_state(Screen::Title),
    );
    app.add_loading_state(LoadingState::new(Menu::Credits).load_collection::<CreditsAssets>());
}

fn spawn_loading_screen(mut commands: Commands) {
    commands.spawn((
        widget::ui_root("Loading Screen"),
        DespawnOnExit(Screen::Loading),
        children![widget::label("Loading...")],
    ));
}
