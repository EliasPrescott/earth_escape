use bevy::prelude::*;
use bevy::ui::Val::Px;
use heron::prelude::*;
use rand::Rng;

use crate::types::*;

pub struct AsteroidPlugin;

impl Plugin for AsteroidPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(setup)
            .add_system(spawn_chasers)
            .add_system(move_chasing_enemies)
            .add_system(increase_spawn_size);
    }
}

fn setup(
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(SpawnTimer(Timer::from_seconds(0.5, true)));
    commands.insert_resource(IncreaseSpawnSizeTimer(Timer::from_seconds(5.0, true)));
    commands.insert_resource(ChaserCount::new(0, 1000));
    commands.insert_resource(SpawnSizeIncrements(0));
    commands.insert_resource(ChaserSprite(asset_server.load("sprites/Meteor1.png")));
    commands.insert_resource(ChickenSprite(asset_server.load("sprites/Chicken.png")));

    // Loading this in main and here is not ideal, but I don't know the best way to guarantee it is loaded in one place first
    // I should probably look into a way to do that, but is fine for now.
    let bold_font: Handle<Font> = asset_server.load("fonts/Fredoka/Fredoka-Bold.ttf");

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                padding: Rect {
                    bottom: Px(16.0),
                    right: Px(16.0),
                    left: Px(16.0),
                    ..Default::default()
                },
                justify_content: JustifyContent::SpaceBetween,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::FlexStart,
                ..Default::default()
            },
            color: Color::NONE.into(),
            ..Default::default()
        })
        .with_children(|parent| {

            parent
                .spawn_bundle(NodeBundle {
                    color: Color::NONE.into(),
                    style: Style {
                        padding: Rect::all(Px(8.0)),                        
                        ..Default::default()
                    },
                    ..Default::default()
                }).with_children(|nested_parent| {
                    nested_parent
                        .spawn_bundle(TextBundle {
                            // Use `Text` directly
                            text: Text {
                                // Construct a `Vec` of `TextSection`s
                                sections: vec![
                                    TextSection {
                                        value: "Enemy Count: ".to_string(),
                                        style: TextStyle {
                                            font: bold_font.clone(),
                                            font_size: 48.0,
                                            color: Color::WHITE,
                                        },
        
                                    },
                                    TextSection {
                                        value: "0".to_string(),
                                        style: TextStyle {
                                            font: bold_font.clone(),
                                            font_size: 48.0,
                                            color: Color::WHITE,
                                        },
                                    },
                                ],
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .insert(EnemyCountText);
                });                
        });
}

fn spawn_chasers(
    mut commands: Commands,
    mut timer: ResMut<SpawnTimer>,
    time: Res<Time>,
    mut chaser_count: ResMut<ChaserCount>,
    mut enemy_count_text_query: Query<&mut Text, With<EnemyCountText>>,
    windows: Res<Windows>,
    chaser_sprite: Res<ChaserSprite>,
    chicken_sprite: Res<ChickenSprite>,
    mut random_gen: ResMut<RandomGenerator>,
    player_query: Query<&Transform, With<Player>>,
    player_died: Res<PlayerDied>,
    game_paused: ResMut<GamePaused>,
    size_increments: Res<SpawnSizeIncrements>,
) {
    if !game_paused.0 && !player_died.0 && timer.0.tick(time.delta()).just_finished() && !chaser_count.at_max() {

        let window_width = windows.get_primary().unwrap().width();
        let window_height = windows.get_primary().unwrap().height();

        let size = window_width / 40.;

        let player_transform = player_query.single().translation;

        let size_scale =
            if random_gen.0.gen_bool(0.75) { 
                random_gen.0.gen_range(0.8..1.2)
            } else { 
                random_gen.0.gen_range(0.75..2.5 + (size_increments.0 as f32 / 50.))
            };

        let spawn_x: f32 = 
            if random_gen.0.gen_bool(0.5) { 
                random_gen.0.gen_range((player_transform.x + window_width)..(player_transform.x + window_width + 100.)) 
            } else { 
                random_gen.0.gen_range((player_transform.x - window_width - 100.)..(player_transform.x - window_width)) 
            };

        let spawn_y: f32 = 
            if random_gen.0.gen_bool(0.5) { 
                random_gen.0.gen_range((player_transform.y + window_height)..(player_transform.y + window_height + 100.)) 
            } else {
                random_gen.0.gen_range((player_transform.y - window_height - 100.)..(player_transform.y - window_height)) 
            };

        commands
            .spawn_bundle(
                SpriteBundle {
                    texture: if random_gen.0.gen_bool(0.01) { chicken_sprite.0.clone() } else { chaser_sprite.0.clone() },
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(size * size_scale, size * size_scale)),
                        ..Default::default()
                    },
                    transform: Transform::from_xyz(spawn_x, spawn_y, 0.0),
                    ..Default::default()
                }
            )
            .insert(ChasingEnemy)
            .insert(Speed(2.5))
            .insert(RigidBody::Dynamic)
            .insert(SizeScale(size_scale))
            .insert(CollisionShape::Sphere {
                radius: (size * size_scale) / 2.,
            })
            .insert(Velocity::default())
            .insert(PhysicMaterial { friction: 1.0, density: 10.0 * size_scale, ..Default::default() })
            .insert(CollisionLayers::new(Layer::Enemies, Layer::Player).with_mask(Layer::Enemies));
            
            chaser_count.current += 1;
            enemy_count_text_query.single_mut().sections[1].style.color = Color::Rgba {
                red: 1.,
                green: (255. - chaser_count.current as f32) / 255.,
                blue: (255. - chaser_count.current as f32) / 255.,
                alpha: 1.
            };
            enemy_count_text_query.single_mut().sections[1].value = format!("{:.2}", chaser_count.current);
    }
}

fn increase_spawn_size(
    mut increments: ResMut<SpawnSizeIncrements>,
    player_died: Res<PlayerDied>,
    game_paused: ResMut<GamePaused>,
    mut timer: ResMut<IncreaseSpawnSizeTimer>,
    time: Res<Time>,
) {
    if !game_paused.0 && !player_died.0 && timer.0.tick(time.delta()).just_finished() {
        if increments.0 < 100 {
            increments.0 += 1;
        }
    }
}

fn move_chasing_enemies(
    game_paused: Res<GamePaused>,
    mut query: Query<(&Transform, &Speed, &mut Velocity), With<ChasingEnemy>>,
    player_query: Query<&Transform, (With<Player>, Without<ChasingEnemy>)>,
)
{
    if !game_paused.0 {
        if let Some(player_transform) = player_query.iter().next() {
            for (transform, Speed(speed), mut velocity) in query.iter_mut() {
                if transform.translation.x > player_transform.translation.x {
                    velocity.linear.x -= speed;
                } else {
                    velocity.linear.x += speed;
                }

                if transform.translation.y > player_transform.translation.y {
                    velocity.linear.y -= speed;
                } else {
                    velocity.linear.y += speed;
                }
            }
        }
    }
}