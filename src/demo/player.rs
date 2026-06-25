//! Player-specific behavior.

use std::time::Duration;

use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;
use leafwing_input_manager::{plugin::InputManagerPlugin, prelude::InputMap};

use crate::demo::{
    commands::{mining::AttackStats, path_following::MovementController},
    health::{Health, healthbar},
    input::Action,
    level::{LevelAssets, buildings::PreparedBuilding},
};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<CursorPos>();
    app.add_systems(Startup, init_cursor_tracker);
    app.add_systems(
        Update,
        (update_cursor_pos, update_cursor).run_if(resource_exists::<CursorPos>),
    );
    app.add_plugins(InputManagerPlugin::<Action>::default());
}

#[derive(Resource)]
pub struct CursorPos(pub Vec2);

#[derive(Component)]
pub struct Cursor;

impl Default for CursorPos {
    fn default() -> Self {
        // Initialize the cursor pos at some far away place. It will get updated
        // correctly when the cursor moves.
        Self(Vec2::new(-1000.0, -1000.0))
    }
}

pub fn update_cursor(cursor_pos: Res<CursorPos>, mut cursor: Single<&mut Transform, With<Cursor>>) {
    cursor.translation = cursor_pos.0.extend(500.0);
}

pub fn init_cursor_tracker(mut commands: Commands, cursor: Res<CursorPos>) {
    commands.spawn((Cursor, Transform::from_translation(cursor.0.extend(500.0))));
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
pub fn player(player_assets: &PlayerAssets, level_assets: &LevelAssets) -> impl Bundle {
    let health = Health::new(10.0);
    (
        Name::new("Player"),
        Player,
        Sprite {
            image: player_assets.player.clone(),
            ..default()
        },
        MovementController::default(),
        AttackStats::new(2.0, 26.0, Duration::from_millis(700)),
        InputMap::new([
            (Action::SpawnEnemies, KeyCode::Space),
            (Action::ChangePreparedBuilding, KeyCode::KeyC),
        ]),
        PreparedBuilding::default(),
        health,
        Faction::player(),
        children![healthbar(level_assets, health)],
    )
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Faction(pub FactionId);

impl Faction {
    pub const PLAYER_FACTION_ID: FactionId = FactionId(1);
    pub const MONSTER_FACTION_ID: FactionId = FactionId(2);

    pub fn player() -> Self {
        Faction(Self::PLAYER_FACTION_ID)
    }

    pub fn monster() -> Self {
        Faction(Self::MONSTER_FACTION_ID)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FactionId(u16);

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
