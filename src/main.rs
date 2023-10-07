use std::collections::{HashMap, HashSet};
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy::input::common_conditions::input_toggle_active;
use bevy_ecs_ldtk::utils::translation_to_grid_coords;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;

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
        )
        .add_plugins((
            LdtkPlugin,
            FrameTimeDiagnosticsPlugin,
            WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::F1)),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (
            character_movement,
            bevy::window::close_on_esc,
            camera_fit_inside_current_level,
        ))
        .insert_resource(LevelSelection::default())
        .register_ldtk_int_cell_for_layer::<WallBundle>("collisions", 1)
        .register_ldtk_entity_for_layer::<PlayerBundle>("entities", "player")
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // spawn the camera
    let camera = Camera2dBundle::default();
    commands.spawn(camera);

    commands.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load("levels.ldtk"),
        ..default()
    });
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Wall;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Player;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Enemy;

#[derive(Clone, Default, Bundle, LdtkEntity)]
pub struct PlayerBundle {
    pub player: Player,

    #[sprite_sheet_bundle]
    pub sprite_bundle: SpriteSheetBundle,

    // #[from_entity_instance]
    // pub collider_bundle: ColliderBundle,

    #[worldly]
    pub worldly: Worldly,

    #[grid_coords]
    grid_coords: GridCoords,

    // // Build Items Component manually by using `impl From<&EntityInstance>`
    // #[from_entity_instance]
    // items: Items,

    // The whole EntityInstance can be stored directly as an EntityInstance component
    #[from_entity_instance]
    entity_instance: EntityInstance,
}

#[derive(Clone, Default, Bundle, LdtkIntCell)]
pub struct WallBundle {
    wall: Wall,
}

pub fn update_level_selection(
    level_query: Query<(&Handle<LdtkLevel>, &Transform), Without<Player>>,
    player_query: Query<&Transform, With<Player>>,
    mut level_selection: ResMut<LevelSelection>,
    ldtk_levels: Res<Assets<LdtkLevel>>,
) {
    for (level_handle, level_transform) in &level_query {
        if let Some(ldtk_level) = ldtk_levels.get(level_handle) {
            let level_bounds = Rect {
                min: Vec2::new(level_transform.translation.x, level_transform.translation.y),
                max: Vec2::new(
                    level_transform.translation.x + ldtk_level.level.px_wid as f32,
                    level_transform.translation.y + ldtk_level.level.px_hei as f32,
                ),
            };

            for player_transform in &player_query {
                if player_transform.translation.x < level_bounds.max.x
                    && player_transform.translation.x > level_bounds.min.x
                    && player_transform.translation.y < level_bounds.max.y
                    && player_transform.translation.y > level_bounds.min.y
                    && !level_selection.is_match(&0, &ldtk_level.level)
                {
                    *level_selection = LevelSelection::Iid(ldtk_level.level.iid.clone());
                }
            }
        }
    }
}

pub fn spawn_wall_collision(
    mut commands: Commands,
    wall_query: Query<(&GridCoords, &Parent), Added<Wall>>,
    parent_query: Query<&Parent, Without<Wall>>,
    level_query: Query<(Entity, &Handle<LdtkLevel>)>,
    levels: Res<Assets<LdtkLevel>>,
) {
    // /// Represents a wide wall that is 1 tile tall
    // /// Used to spawn wall collisions
    // #[derive(Clone, Eq, PartialEq, Debug, Default, Hash)]
    // struct Plate {
    //     left: i32,
    //     right: i32,
    // }
    //
    // /// A simple rectangle type representing a wall of any size
    // struct Rect {
    //     left: i32,
    //     right: i32,
    //     top: i32,
    //     bottom: i32,
    // }

    // Consider where the walls are
    // storing them as GridCoords in a HashSet for quick, easy lookup
    //
    // The key of this map will be the entity of the level the wall belongs to.
    // This has two consequences in the resulting collision entities:
    // 1. it forces the walls to be split along level boundaries
    // 2. it lets us easily add the collision entities as children of the appropriate level entity
    let mut level_to_wall_locations: HashMap<Entity, HashSet<GridCoords>> = HashMap::new();

    wall_query.for_each(|(&grid_coords, parent)| {
        // An intgrid tile's direct parent will be a layer entity, not the level entity
        // To get the level entity, you need the tile's grandparent.
        // This is where parent_query comes in.
        if let Ok(grandparent) = parent_query.get(parent.get()) {
            level_to_wall_locations
                .entry(grandparent.get())
                .or_default()
                .insert(grid_coords);
        }
    });

    // if !wall_query.is_empty() {
    //     level_query.for_each(|(level_entity, level_handle)| {
    //         if let Some(level_walls) = level_to_wall_locations.get(&level_entity) {
    //             let level = levels
    //                 .get(level_handle)
    //                 .expect("Level should be loaded by this point");
    //
    //             let LayerInstance {
    //                 c_wid: width,
    //                 c_hei: height,
    //                 grid_size,
    //                 ..
    //             } = level
    //                 .level
    //                 .layer_instances
    //                 .clone()
    //                 .expect("Level asset should have layers")[0];
    //
    //             // combine wall tiles into flat "plates" in each individual row
    //             let mut plate_stack: Vec<Vec<Plate>> = Vec::new();
    //
    //             for y in 0..height {
    //                 let mut row_plates: Vec<Plate> = Vec::new();
    //                 let mut plate_start = None;
    //
    //                 // + 1 to the width so the algorithm "terminates" plates that touch the right edge
    //                 for x in 0..width + 1 {
    //                     match (plate_start, level_walls.contains(&GridCoords { x, y })) {
    //                         (Some(s), false) => {
    //                             row_plates.push(Plate {
    //                                 left: s,
    //                                 right: x - 1,
    //                             });
    //                             plate_start = None;
    //                         }
    //                         (None, true) => plate_start = Some(x),
    //                         _ => (),
    //                     }
    //                 }
    //
    //                 plate_stack.push(row_plates);
    //             }
    //
    //             // combine "plates" into rectangles across multiple rows
    //             let mut rect_builder: HashMap<Plate, Rect> = HashMap::new();
    //             let mut prev_row: Vec<Plate> = Vec::new();
    //             let mut wall_rects: Vec<Rect> = Vec::new();
    //
    //             // an extra empty row so the algorithm "finishes" the rects that touch the top edge
    //             plate_stack.push(Vec::new());
    //
    //             for (y, current_row) in plate_stack.into_iter().enumerate() {
    //                 for prev_plate in &prev_row {
    //                     if !current_row.contains(prev_plate) {
    //                         // remove the finished rect so that the same plate in the future starts a new rect
    //                         if let Some(rect) = rect_builder.remove(prev_plate) {
    //                             wall_rects.push(rect);
    //                         }
    //                     }
    //                 }
    //                 for plate in &current_row {
    //                     rect_builder
    //                         .entry(plate.clone())
    //                         .and_modify(|e| e.top += 1)
    //                         .or_insert(Rect {
    //                             bottom: y as i32,
    //                             top: y as i32,
    //                             left: plate.left,
    //                             right: plate.right,
    //                         });
    //                 }
    //                 prev_row = current_row;
    //             }

                // commands.entity(level_entity).with_children(|level| {
                //     // Spawn colliders for every rectangle..
                //     // Making the collider a child of the level serves two purposes:
                //     // 1. Adjusts the transforms to be relative to the level for free
                //     // 2. the colliders will be despawned automatically when levels unload
                //     for wall_rect in wall_rects {
                //         level
                //             .spawn_empty()
                //             .insert(Collider::cuboid(
                //                 (wall_rect.right as f32 - wall_rect.left as f32 + 1.)
                //                     * grid_size as f32
                //                     / 2.,
                //                 (wall_rect.top as f32 - wall_rect.bottom as f32 + 1.)
                //                     * grid_size as f32
                //                     / 2.,
                //             ))
                //             .insert(RigidBody::Fixed)
                //             .insert(Friction::new(1.0))
                //             .insert(Transform::from_xyz(
                //                 (wall_rect.left + wall_rect.right + 1) as f32 * grid_size as f32
                //                     / 2.,
                //                 (wall_rect.bottom + wall_rect.top + 1) as f32 * grid_size as f32
                //                     / 2.,
                //                 0.,
                //             ))
                //             .insert(GlobalTransform::default());
                //     }
                // });
        //     }
        // });
    // }
}

fn character_movement(
    mut characters: Query<(&mut Transform, &Player, &mut GridCoords, &EntityInstance)>,
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    for (mut transform, _, mut grid_coords, entity_instance) in &mut characters {
        let transform_vec2: Vec2 = Vec2::new(transform.translation.x, transform.translation.y);
        let grid_size: IVec2 = IVec2::new(entity_instance.width, entity_instance.height);
        let calc_grid_coords = translation_to_grid_coords(transform_vec2, grid_size);
        grid_coords.set_if_neq(calc_grid_coords);

        // println!("({},{})", transform.translation.x, transform.translation.y);
        // println!("  {:?}", grid_coords);
        // println!("  {:?}", entity_instance.grid);

        if input.any_pressed([KeyCode::W, KeyCode::Up]) {
            transform.translation.y += 128.0 * time.delta_seconds();
            // grid_coords.y += 1;
        }
        if input.any_pressed([KeyCode::S, KeyCode::Down]) {
            transform.translation.y -= 128.0 * time.delta_seconds();
            // grid_coords.y -= 1;
        }
        if input.any_pressed([KeyCode::D, KeyCode::Right]) {
            transform.translation.x += 128.0 * time.delta_seconds();
            // grid_coords.x += 1;
        }
        if input.any_pressed([KeyCode::A, KeyCode::Left]) {
            transform.translation.x -= 128.0 * time.delta_seconds();
            // grid_coords.x -= 1;
        }

        // let new_transform = grid_coords_to_translation(*grid_coords, IVec2::new(8, 8));
        //
        // println!("  ({},{})", new_transform.x, new_transform.y);
        //
        // transform.translation.x = new_transform.x * time.delta_seconds();
        // transform.translation.y = new_transform.y * time.delta_seconds();
    }
}

const ASPECT_RATIO: f32 = 16.0 / 9.0;

pub fn camera_fit_inside_current_level(
    mut camera_query: Query<(&mut OrthographicProjection, &mut Transform), Without<Player>>,
    player_query: Query<&Transform, With<Player>>,
    level_query: Query<(&Transform, &Handle<LdtkLevel>), (Without<OrthographicProjection>, Without<Player>)>,
    level_selection: Res<LevelSelection>,
    ldtk_levels: Res<Assets<LdtkLevel>>,
) {
    if let Ok(Transform { translation: player_translation, .. }) = player_query.get_single() {
        let player_translation = *player_translation;

        let (mut orthographic_projection, mut camera_transform) = camera_query.single_mut();

        for (level_transform, level_handle) in &level_query {
            if let Some(ldtk_level) = ldtk_levels.get(level_handle) {
                let level = &ldtk_level.level;
                if level_selection.is_match(&0, level) {
                    let level_ratio = level.px_wid as f32 / ldtk_level.level.px_hei as f32;
                    orthographic_projection.viewport_origin = Vec2::ZERO;
                    if level_ratio > ASPECT_RATIO {
                        // level is wider than the screen
                        let height = (level.px_hei as f32 / 9.).round() * 9.;
                        let width = height * ASPECT_RATIO;
                        orthographic_projection.scaling_mode =
                            bevy::render::camera::ScalingMode::Fixed { width, height };
                        camera_transform.translation.x =
                            (player_translation.x - level_transform.translation.x - width / 2.)
                                .clamp(0., level.px_wid as f32 - width);
                        camera_transform.translation.y = 0.;
                    } else {
                        // level is taller than the screen
                        let width = (level.px_wid as f32 / 16.).round() * 16.;
                        let height = width / ASPECT_RATIO;
                        orthographic_projection.scaling_mode =
                            bevy::render::camera::ScalingMode::Fixed { width, height };
                        camera_transform.translation.y =
                            (player_translation.y - level_transform.translation.y - height / 2.)
                                .clamp(0., level.px_hei as f32 - height);
                        camera_transform.translation.x = 0.;
                    }

                    camera_transform.translation.x += level_transform.translation.x;
                    camera_transform.translation.y += level_transform.translation.y;
                }
            }
        }
    }
}

