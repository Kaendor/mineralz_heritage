use bevy::prelude::*;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

pub fn plugin(app: &mut App) {
    app.add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new());
}
