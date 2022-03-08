use bevy::prelude::*;
use heron::PhysicsLayer;

// A tag to identify the player entity
#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerShip;

// Used to track the player's current health
#[derive(Component)]
pub struct PlayerHealth(pub u8);

// The u8 represents the placement of the heart
#[derive(Component)]
pub struct HeartSprite(pub u8);

#[derive(Component)]
pub struct ChasingEnemy;

#[derive(Component)]
pub struct Speed(pub f32);



// Define your physics layers
// Probably only need one or none
#[derive(PhysicsLayer)]
pub enum Layer {
    Player,
    Enemies,
}

pub struct SpawnTimer(pub Timer);

pub struct PlayerDied(pub bool);

#[derive(Component)]
pub struct SizeScale(pub f32);

// A unit pub struct to help identify the FPS UI component, since there may be many Text components
#[derive(Component)]
pub struct FpsText;

#[derive(Component)]
pub struct EnemyCountText;

#[derive(Component)]
pub struct CenterMessageText;

#[derive(Component)]
pub struct SubCenterText;

// A unit struct to help identify the color-changing Text component
#[derive(Component)]
pub struct ColorText;

#[derive(Component)]
pub struct Camera2D;

pub struct GamePaused(pub bool);
pub struct ChaserCount {
    pub current: u32,
    pub max: u32,
}

impl ChaserCount {
    pub fn new(current: u32, max: u32) -> Self {
        ChaserCount {
            current,
            max,
        }
    }

    pub fn at_max(&self) -> bool {
        self.current >= self.max
    }
}

pub struct SpawnSizeIncrements(pub u8);
pub struct IncreaseSpawnSizeTimer(pub Timer);
pub struct FullHeartSprite(pub Handle<Image>);
pub struct EmptyHeartSprite(pub Handle<Image>);
pub struct ChaserSprite(pub Handle<Image>);
pub struct RandomGenerator(pub rand::rngs::StdRng);
pub struct ChickenSprite(pub Handle<Image>);