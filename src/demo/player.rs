//! Player-specific behavior.

use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;

use crate::demo::commands::{mining::MiningPower, path_following::MovementController};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<CursorPos>();
    app.add_systems(
        Update,
        update_cursor_pos.run_if(resource_exists::<CursorPos>),
    );
}

#[derive(Resource)]
pub struct CursorPos(pub Vec2);

impl Default for CursorPos {
    fn default() -> Self {
        // Initialize the cursor pos at some far away place. It will get updated
        // correctly when the cursor moves.
        Self(Vec2::new(-1000.0, -1000.0))
    }
}

// We need to keep the cursor position updated based on any `CursorMoved` events.
pub fn update_cursor_pos(
    camera_q: Query<(&GlobalTransform, &Camera)>,
    mut cursor_moved_events: MessageReader<CursorMoved>,
    mut cursor_pos: ResMut<CursorPos>,
) {
    for cursor_moved in cursor_moved_events.read() {
        // To get the mouse's world position, we have to transform its window position by
        // any transforms on the camera. This is done by projecting the cursor position into
        // camera space (world space).
        for (cam_t, cam) in camera_q.iter() {
            if let Ok(pos) = cam.viewport_to_world_2d(cam_t, cursor_moved.position) {
                *cursor_pos = CursorPos(pos);
            }
        }
    }
}

/// The player character.
pub fn player(player_assets: &PlayerAssets) -> impl Bundle {
    (
        Name::new("Player"),
        Player,
        Sprite::from_image(player_assets.player.clone()),
        MovementController::default(),
        MiningPower::new(2.0),
    )
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Player;

#[derive(Resource, Clone, Reflect, AssetCollection)]
#[reflect(Resource)]
pub struct PlayerAssets {
    #[asset(path = "images/player_sprite.png")]
    #[asset(image(sampler(filter = nearest)))]
    player: Handle<Image>,

    #[asset(
        paths(
            "audio/sound_effects/step1.ogg",
            "audio/sound_effects/step2.ogg",
            "audio/sound_effects/step3.ogg",
            "audio/sound_effects/step4.ogg"
        ),
        collection(typed)
    )]
    pub steps: Vec<Handle<AudioSource>>,
}
