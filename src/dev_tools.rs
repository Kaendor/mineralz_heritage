//! Development tools for the game. This plugin is only enabled in dev builds.

use bevy::{
    dev_tools::states::log_transitions, input::common_conditions::input_just_pressed, prelude::*,
};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

use crate::{demo::commands::path_following::display_paths, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    // Log `Screen` state transitions.
    app.add_systems(Update, log_transitions::<Screen>);

    app.add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new().run_if(debug_run_condition));
    // Toggle the debug overlay for UI.
    app.add_systems(
        Update,
        toggle_debug_ui.run_if(input_just_pressed(TOGGLE_KEY)),
    );

    app.add_systems(Update, (display_paths).run_if(debug_run_condition));
}

const TOGGLE_KEY: KeyCode = KeyCode::KeyD;

fn toggle_debug_ui(mut options: ResMut<UiDebugOptions>) {
    options.toggle();
}

fn debug_run_condition(options: Res<UiDebugOptions>) -> bool {
    options.enabled
}
