use bevy::utils::Duration;

use bevy::{asset::ChangeWatcher, prelude::*};
use bevy_ecs_tilemap::*;

use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy::input::common_conditions::input_toggle_active;

mod helpers;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Rust Survivors-Like".into(),
                    ..default()
                }),
                ..default()
            })
            .set(ImagePlugin::default_nearest())
            .set(AssetPlugin {
                 watch_for_changes: ChangeWatcher::with_delay(Duration::from_millis(200)),
                 ..default()
            }),
        )
        .add_plugins((
            WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::F1)),
            TilemapPlugin,
            helpers::tiled::TiledMapPlugin
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (
            // character_movement,
            helpers::camera::movement,
            bevy::window::close_on_esc,
        ))
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // spawn the camera
    let camera = Camera2dBundle::default();
    commands.spawn(camera);

    let handle: Handle<helpers::tiled::TiledMap> = asset_server.load("levels.tmx");
    // println!("{:?}", Assets::<helpers::tiled::TiledMap>::get(handle.clone()).unwrap());

    // commands.spawn(helpers::ldtk::LdtkMapBundle {
    //     ldtk_map: handle,
    //     ldtk_map_config: LdtkMapConfig {
    //         selected_level: 0,
    //     },
    //     transform: Transform::from_xyz(0.0, 0.0, 0.0),
    //     ..Default::default()
    // });

    commands.spawn(helpers::tiled::TiledMapBundle {
        tiled_map: handle,
        ..Default::default()
    });

    // commands.spawn(LdtkWorldBundle {
    //     ldtk_handle: asset_server.load("levels.ldtk"),
    //     ..default()
    // });
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Wall;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Player;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Enemy;

// #[derive(Clone, Default, Bundle, LdtkEntity)]
// pub struct PlayerBundle {
//     player: Player,
//     #[sprite_sheet_bundle]
//     sprite_bundle: SpriteSheetBundle,
// }
//
// #[derive(Clone, Default, Bundle, LdtkIntCell)]
// pub struct WallBundle {
//     wall: Wall,
// }

// pub fn spawn_wall_collision(
//     mut commands: Commands,
//     wall_query: Query<(&GridCoords, &Parent), Added<Wall>>,
//     parent_query: Query<&Parent, Without<Wall>>,
//     level_query: Query<(Entity, &Handle<LdtkLevel>)>,
//     levels: Res<Assets<LdtkLevel>>,
// ) {
//     wall_query.for_each(|(&grid_coords, parent)| {
//         println!("wall: {}, {}", grid_coords.x, grid_coords.y);
//     });
// }

// fn character_movement(
//     mut characters: Query<(&mut Transform, &Player)>,
//     input: Res<Input<KeyCode>>,
//     time: Res<Time>,
// ) {
//     for (mut transform, _) in &mut characters {
//         if input.pressed(KeyCode::W) || input.pressed(KeyCode::Up) {
//             transform.translation.y += 128.0 * time.delta_seconds();
//             // transform.translation += Vec3::new(1.0, 0.0, 0.0);
//         }
//         if input.pressed(KeyCode::S) || input.pressed(KeyCode::Down) {
//             transform.translation.y -= 128.0 * time.delta_seconds();
//         }
//         if input.pressed(KeyCode::D) || input.pressed(KeyCode::Right) {
//             transform.translation.x += 128.0 * time.delta_seconds();
//         }
//         if input.pressed(KeyCode::A) || input.pressed(KeyCode::Left) {
//             transform.translation.x -= 128.0 * time.delta_seconds();
//         }
//     }
// }
