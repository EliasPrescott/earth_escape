use rand::SeedableRng;
use bevy::window::WindowId;
use bevy::winit::WinitWindows;
use winit::window::Icon;

use bevy::{
    prelude::*, 
    window::{WindowMode, WindowResizeConstraints, WindowResized},
    app::Events,
};
use bevy_embedded_assets::EmbeddedAssetPlugin;

use heron::prelude::*;

mod player;
mod types;
mod utilities;
mod asteroids;

use types::*;
use player::PlayerPlugin;
use asteroids::AsteroidPlugin;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgba(0.0, 0.0, 0.0, 1.0)))
        .insert_resource(
            WindowDescriptor {
                transparent: false,
                decorations: true,
                mode: WindowMode::Windowed,
                title: "Earth Escape".to_string(),
                width: 1200.,
                height: 800.,
                resize_constraints: WindowResizeConstraints {
                    min_height: 400.0,
                    min_width: 400.0,
                    ..Default::default()
                },
                ..Default::default()
            }
        )
        .add_plugins_with(DefaultPlugins, |group| {
            group.add_before::<bevy::asset::AssetPlugin, _>(EmbeddedAssetPlugin)
        })
        //.add_plugins(DefaultPlugins)
        .add_plugin(PhysicsPlugin::default())
        .add_plugin(PlayerPlugin)
        .add_plugin(AsteroidPlugin)
        .add_startup_system(set_window_icon)
        .add_startup_system(setup)
        .add_system(toggle_physics_pause)
        .add_system(fullscreen_toggle)
        .add_system(reset_game)
        .add_system(resize_items)
        //.add_system(text_color_system)
        .run();
}

// This could be split up into player/resize_player and asteroids/resize_asteroids, but I'm leaving it all together for convenience (laziness).
fn resize_items(
    resize_event: Res<Events<WindowResized>>,
    mut player_query: Query<(&mut Sprite, &mut CollisionShape), (With<Player>, Without<ChasingEnemy>)>,
    mut ship_query: Query<&mut TextureAtlasSprite>,
    mut chaser_query: Query<(&mut Sprite, &mut CollisionShape, &SizeScale), (With<ChasingEnemy>, Without<Player>)>,
) {
    let mut reader = resize_event.get_reader();
    for e in reader.iter(&resize_event) {
        let player_size = e.width / 20.;
        let (mut sprite, mut shape, ) = player_query.single_mut();
        sprite.custom_size = Some(Vec2::new(player_size, player_size));
        *shape =
            CollisionShape::Sphere {
                radius: player_size / 2.,
            };

        ship_query.single_mut().custom_size = Some(Vec2::new(player_size * 1.5, player_size * 1.5));
        
        for (mut sprite, mut shape, SizeScale(size_scale)) in chaser_query.iter_mut() {
            let chaser_size = e.width / 40.;
            sprite.custom_size = Some(Vec2::new(chaser_size * size_scale, chaser_size * size_scale));
            *shape =
                CollisionShape::Sphere {
                    radius: (chaser_size * size_scale) / 2.,
                };
        }
    }
}

fn fullscreen_toggle(keyboard_input: Res<Input<KeyCode>>, mut windows: ResMut<Windows>) {
    if keyboard_input.just_pressed(KeyCode::F11) {
        let window = windows.get_primary_mut().unwrap();
        
        window.set_mode(
            match window.mode() {
                WindowMode::BorderlessFullscreen => WindowMode::Windowed,
                _ => WindowMode::BorderlessFullscreen,
            }
        );
    }
}

fn setup(
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
) {
    // UI camera
    commands.spawn_bundle(UiCameraBundle::default());
    commands.spawn_bundle(OrthographicCameraBundle::new_2d()).insert(Camera2D);
    commands.insert_resource(PhysicsTime::new(1.0));
    commands.insert_resource(GamePaused(false));

    let bold_font: Handle<Font> = asset_server.load("fonts/Fredoka/Fredoka-Bold.ttf");

    // This is currently only used for the asteroid spawns, but I'm leaving it in main setup because it feels more like a universal utilility resource
    commands.insert_resource(RandomGenerator(rand::rngs::StdRng::from_entropy()));

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::ColumnReverse,
                ..Default::default()
            },
            color: Color::NONE.into(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(TextBundle {
                    // Use `Text` directly
                    text: Text {
                        // Construct a `Vec` of `TextSection`s
                        sections: vec![
                            TextSection {
                                value: "".to_string(),
                                style: TextStyle {
                                    font: bold_font.clone(),
                                    font_size: 82.0,
                                    color: Color::WHITE,
                                },
                            },
                        ],
                        ..Default::default()
                    },
                ..Default::default()
            })
            .insert(CenterMessageText);
            
                parent
                .spawn_bundle(TextBundle {
                    // Use `Text` directly
                    text: Text {
                        // Construct a `Vec` of `TextSection`s
                        sections: vec![
                            TextSection {
                                value: "".to_string(),
                                style: TextStyle {
                                    font: bold_font.clone(),
                                    font_size: 36.0,
                                    color: Color::GREEN,
                                },
                            },
                        ],
                        ..Default::default()
                    },
                ..Default::default()
            })
            .insert(SubCenterText);
        });
}

// This should behave on non-Windows builds, but it might need to be removed/refactored for those.
// Comes from https://bevy-cheatbook.github.io/cookbook/window-icon.html
// Should be updated to use the correct Bevy method whenever that is released
fn set_window_icon(
    windows: Res<WinitWindows>,
) {
    let primary = windows.get_window(WindowId::primary()).unwrap();

    if let Ok(base_image) = image::open("assets/sprites/Chicken.png") {
        let image = base_image.into_rgba8();

        let (width, height) = image.dimensions();
        let rgba = image.into_raw();

        let icon = Icon::from_rgba(rgba, width, height).unwrap();

        primary.set_window_icon(Some(icon));
    }
}


// Need to add timers to this as they are added to the game.
// Also important. Need to check GamePaused flag in other systems before applying changes.
fn toggle_physics_pause(
    input: Res<Input<KeyCode>>,
    mut physics_time: ResMut<PhysicsTime>,
    mut game_paused: ResMut<GamePaused>,
    mut enemy_spawn_timer: ResMut<SpawnTimer>,
    mut center_text: Query<&mut Text, With<CenterMessageText>>,
    player_died: Res<PlayerDied>,
) {
    if !player_died.0 && input.just_pressed(KeyCode::Space) {
        if game_paused.0 {
            physics_time.resume();
            enemy_spawn_timer.0.unpause();
            game_paused.0 = false;
            center_text.single_mut().sections[0].value = "".to_string();
            
        } else {
            physics_time.pause();
            enemy_spawn_timer.0.pause();
            game_paused.0 = true;
            center_text.single_mut().sections[0].value = "Paused".to_string();
        }
    }
}

fn reset_game(
    mut commands: Commands,
    input: Res<Input<KeyCode>>,
    mut physics_time: ResMut<PhysicsTime>,
    mut game_paused: ResMut<GamePaused>,
    mut enemy_spawn_timer: ResMut<SpawnTimer>,
    mut center_text: Query<&mut Text, (With<CenterMessageText>, Without<SubCenterText>, Without<EnemyCountText>)>,
    mut sub_center_text: Query<&mut Text, (With<SubCenterText>, Without<CenterMessageText>, Without<EnemyCountText>)>,
    mut player_died: ResMut<PlayerDied>,
    mut health_query: Query<&mut PlayerHealth>,
    chaser_query: Query<Entity, With<ChasingEnemy>>,
    mut player_query: Query<(&mut Transform, &mut Velocity), With<Player>>,
    mut chaser_count: ResMut<ChaserCount>,
    mut enemy_count_text_query: Query<&mut Text, (With<EnemyCountText>, Without<CenterMessageText>, Without<SubCenterText>)>
) {
    if player_died.0 && input.just_pressed(KeyCode::R) {
        chaser_query.iter().for_each(|e| commands.entity(e).despawn());
        chaser_count.current = 0;
        enemy_count_text_query.single_mut().sections[1].value = String::from("0");
        let (mut transform, mut velocity) = player_query.single_mut();
        *transform = Transform::from_xyz(0.0, 0.0, 0.0);
        *velocity = Velocity::from_linear(Vec3::new(0.0, 0.0, 0.0));
        physics_time.resume();
        enemy_spawn_timer.0.reset();
        enemy_spawn_timer.0 = Timer::from_seconds(0.5, true);
        enemy_spawn_timer.0.unpause();
        game_paused.0 = false;
        center_text.single_mut().sections[0].value = String::from("");
        sub_center_text.single_mut().sections[0].value = String::from("");
        health_query.single_mut().0 = 5;
        player_died.0 = false;
    }
}