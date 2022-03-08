use bevy::prelude::*;
use bevy::ui::Val::Px;
use heron::prelude::*;

use crate::utilities::*;
use crate::types::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(setup)
            .add_startup_system(add_player)
            .add_system(player_movement)
            .add_system(calculate_health);
    }
}

fn setup(
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(PlayerDied(false));

    let full_heart_sprite: Handle<Image> = asset_server.load("sprites/full_heart.png");
    let empty_heart_sprite: Handle<Image> = asset_server.load("sprites/empty_heart.png");

    commands.insert_resource(FullHeartSprite(full_heart_sprite.clone()));
    commands.insert_resource(EmptyHeartSprite(empty_heart_sprite.clone()));

    // Spawning the UI node to hold the player health sprites
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                padding: Rect {
                    top: Px(16.0),
                    ..Default::default()
                },
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexEnd,
                ..Default::default()
            },
            color: Color::NONE.into(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,

                        ..Default::default()
                    },
                    color: Color::NONE.into(),
                    ..Default::default()
                })
                .with_children(|nested_parent| {
                    nested_parent
                        .spawn_bundle(ImageBundle {
                            image: full_heart_sprite.clone().into(),
                            ..Default::default()
                        })
                        .insert(HeartSprite(0));
                    
                    nested_parent
                        .spawn_bundle(ImageBundle {
                            image: full_heart_sprite.clone().into(),
                            ..Default::default()
                        })
                        .insert(HeartSprite(1));

                    nested_parent
                        .spawn_bundle(ImageBundle {
                            image: full_heart_sprite.clone().into(),
                            ..Default::default()
                        })
                        .insert(HeartSprite(2));

                    nested_parent
                        .spawn_bundle(ImageBundle {
                            image: full_heart_sprite.clone().into(),
                            ..Default::default()
                        })
                        .insert(HeartSprite(3));

                    nested_parent
                        .spawn_bundle(ImageBundle {
                            image: full_heart_sprite.clone().into(),
                            ..Default::default()
                        })
                        .insert(HeartSprite(4));
                });
            
        });
}

fn add_player(
    mut commands: Commands,
    windows: Res<Windows>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let size = windows.get_primary().unwrap().width() / 20.;

    commands
        .spawn_bundle(
            SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::new(size, size)),
                    ..Default::default()
                },
                texture: asset_server.load("sprites/PlayerEarth.png"),
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                ..Default::default()
            }
        )
        .insert(Player)
        .insert(Speed(5.0))
        .insert(RigidBody::Dynamic)
        
        // Attach a collision shape
        // .insert(CollisionShape::Cuboid {
        //     half_extends: Vec3::new(size / 2., size / 2., 0.0),
        //     border_radius: None,
        // })
        .insert(CollisionShape::Sphere {
            radius: size / 2.,
        })

        
        
        // Optionally add other useful components...
        .insert(Velocity::default())
        // .insert(Acceleration::from_linear(Vec3::X * 1.0))
        .insert(PhysicMaterial { friction: 1.0, density: 20.0, ..Default::default() })
        .insert(Damping::from_linear(0.5).with_angular(1.0))
        .insert(RotationConstraints::lock())
        .insert(CollisionLayers::new(Layer::Player, Layer::Enemies))
        .insert(PlayerHealth(5));

    let ship_texture_atlas = TextureAtlas::from_grid(asset_server.load("sprites/Space_Ship_Spritesheet.png"), Vec2::new(96.0, 96.0), 2, 1);
    let texture_atlas_handle = texture_atlases.add(ship_texture_atlas);

    commands
        .spawn_bundle(
            SpriteSheetBundle {
                texture_atlas: texture_atlas_handle,
                transform: Transform::from_scale(Vec3::new(size / 45., size / 45., 1.)),
                sprite: TextureAtlasSprite {
                    index: 0,
                    ..Default::default()
                },
                ..Default::default()
            }
        )
        .insert(PlayerShip);
}

fn player_movement(
    game_paused: Res<GamePaused>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&Transform, &Speed, &mut Velocity), (With<Player>, (Without<Camera2D>, Without<PlayerShip>))>,
    mut ship_query: Query<(&mut Transform, &mut TextureAtlasSprite), (With<PlayerShip>, (Without<Camera2D>, Without<Player>))>,
    mut camera_query: Query<&mut Transform, (With<Camera2D>, (Without<Player>, Without<PlayerShip>))>,
    player_died: Res<PlayerDied>,
    // mut touches: EventReader<TouchInput>,
    // windows: Res<Windows>,
) 
{
    if !game_paused.0 {
        let (transform, Speed(speed), mut velocity) = query.single_mut();

        camera_query.single_mut().translation = transform.translation;

        let mut input_active = false;

        if !player_died.0 {
            let mut x = 0.0;
            let mut y = 0.0;

            // if let Some(touch) = touches.iter().next() {
            //     let window_width = windows.get_primary().unwrap().width();
            //     x += (touch.position.x - window_width / 2.) / (window_width / 2.);
            //     y += (touch.position.y - window_width / 2.) / (window_width / 2.);
            // } else {

                
            if keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left) {
                x -= 1.0;
                input_active = true;
            };
            if keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right) {
                x += 1.0;
                input_active = true;
            };
            if keyboard_input.pressed(KeyCode::W) || keyboard_input.pressed(KeyCode::Up) {
                y += 1.0;
                input_active = true;
            };
            if keyboard_input.pressed(KeyCode::S) || keyboard_input.pressed(KeyCode::Down) {
                y -= 1.0;
                input_active = true;
            };
            // }

            velocity.linear.x += x * speed;
            velocity.linear.y += y * speed;

            // transform.translation.x += x * speed;
            // transform.translation.y += y * speed;
        }

        let (mut ship_transform, mut ship_image) = ship_query.single_mut();

        ship_image.index =
            if input_active {
                0
            } else {
                1
            };

        // Can also just use velocity.linear as the diff because ship_transform gets updated to transform every tick, but this method seems to give a nice result
        let diff = (transform.translation + velocity.linear) - ship_transform.translation;
        let new_angle = diff.y.atan2(diff.x);

        ship_transform.rotation = Quat::from_axis_angle(Vec3::new(0., 0., 1.), new_angle - std::f32::consts::FRAC_PI_2);

        ship_transform.translation = transform.translation;
    }
}

fn calculate_health(
    mut events: EventReader<CollisionEvent>,
    mut player_died: ResMut<PlayerDied>,
    mut heart_query: Query<(&mut UiImage, &HeartSprite), Without<PlayerHealth>>,
    full_heart_sprite: Res<FullHeartSprite>,
    empty_heart_sprite: Res<EmptyHeartSprite>,
    mut health_query: Query<&mut PlayerHealth>,
    mut center_text: Query<&mut Text, With<CenterMessageText>>,
    mut sub_center_text: Query<&mut Text, (With<SubCenterText>, Without<CenterMessageText>)>,
    mut enemy_spawn_timer: ResMut<SpawnTimer>,
) 
{
    if !player_died.0 {
        let mut health = health_query.single_mut();

        events
            .iter()
            .for_each(|event| {

                if event.is_stopped() {
                    let (layers_1, layers_2) = event.collision_layers();
                    if health.0 < 5 {
                        if is_player(layers_1) && is_enemy(layers_2) {
                            health.0 += 1;
                        } else if is_player(layers_2) && is_enemy(layers_1) {
                            health.0 += 1;
                        }
                    }
                }
                if event.is_started() {
                    if health.0 > 0 {
                        let (layers_1, layers_2) = event.collision_layers();
                        if is_player(layers_1) && is_enemy(layers_2) {
                            health.0 -= 1;
                        } else if is_player(layers_2) && is_enemy(layers_1) {
                            health.0 -= 1;
                        }
                    }
                }
            });

        if health.0 <= 0 {
            player_died.0 = true;
            center_text.single_mut().sections[0].value = String::from("You Died");
            sub_center_text.single_mut().sections[0].value = String::from("Press R to restart");
            enemy_spawn_timer.0.pause();
            for (mut sprite, _) in heart_query.iter_mut() {
                sprite.0 = empty_heart_sprite.0.clone();
            }
        } else {
            for (mut sprite, HeartSprite(id)) in heart_query.iter_mut() {
                if health.0 > *id {
                    sprite.0 = full_heart_sprite.0.clone();
                } else {
                    sprite.0 = empty_heart_sprite.0.clone();
                }
            }
        }
    }
}