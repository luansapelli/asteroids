use bevy::prelude::*;
use bevy::window::{PresentMode, PrimaryWindow};
use bevy::audio::{PlaybackMode, Volume};
use bevy::time::TimerMode::{Once};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "asteroids".into(),
                resolution: (800., 600.).into(),
                present_mode: PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        .add_plugins((
            EnvironmentPlugin,
            PlayerPlugin,
            AsteroidPlugin,
            CollisionPlugin,
        ))
        .run();
}

#[derive(Component)]
struct Environment;
struct EnvironmentPlugin;

impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_environment);
    }
}

fn spawn_environment(
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let window = window_query.get_single().unwrap();

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                custom_size: Option::from(Vec2::new(800.0, 600.0)),
                ..default()
            },
            transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0),
            texture: asset_server.load("sprites/galaxy.png"),
            ..default()
        },
        Environment,
        ));

    commands.spawn(AudioBundle {
        source: asset_server.load("audio/background.ogg"),
        settings: PlaybackSettings {
            mode: PlaybackMode::Loop,
            volume: Volume::new_absolute(0.3),
            ..default()
        },
        ..default()
    });

}
const PLAYER_SPEED: f32 = 600.0;
const PLAYER_SIZE: f32 = 64.0;

#[derive(Component)]
struct Player;
struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_player, spawn_camera))
            .add_systems(Update, (player_movement, confine_player_movement));
    }
}

fn spawn_player(
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let window = window_query.get_single().unwrap();

    commands.spawn((
        SpriteBundle {
            transform: Transform::from_xyz(
                window.width() / 2.0,
                -window.height() / 2.0 + 350.0,
                0.0,
            ),
            texture: asset_server.load("sprites/spaceship.png"),
            ..default()
        },
        Player,
    ));
}

fn spawn_camera(window_query: Query<&Window, With<PrimaryWindow>>, mut commands: Commands) {
    let window = window_query.get_single().unwrap();

    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0),
        ..default()
    });
}

fn player_movement(
    keyboard: Res<Input<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    if let Ok(mut transform) = player_query.get_single_mut() {
        let mut direction = Vec3::ZERO;

        if keyboard.pressed(KeyCode::W) {
            direction.y += 1.0;
        }

        if keyboard.pressed(KeyCode::S) {
            direction.y -= 1.0;
        }

        if keyboard.pressed(KeyCode::A) {
            direction.x -= 1.0;
        }

        if keyboard.pressed(KeyCode::D) {
            direction.x += 1.0;
        }

        if direction.length() > 0.0 {
            direction = direction.normalize();
        }

        transform.translation += direction * PLAYER_SPEED * time.delta_seconds();
    }
}

fn confine_player_movement(
    mut player_query: Query<&mut Transform, With<Player>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    if let Ok(mut player_transform) = player_query.get_single_mut() {
        let window = window_query.get_single().unwrap();

        let half_player_size = PLAYER_SIZE / 2.0;
        let x_min = half_player_size;
        let x_max = window.width() - half_player_size;
        let y_min = half_player_size;
        let y_max = window.height() - half_player_size;

        let mut translation = player_transform.translation;

        if translation.x < x_min {
            translation.x = x_min;
        } else if translation.x > x_max {
            translation.x = x_max;
        }

        if translation.y < y_min {
            translation.y = y_min;
        } else if translation.y > y_max {
            translation.y = y_max;
        }

        player_transform.translation = translation;
    }
}


const ASTEROID_SPEED: f32 = 200.0;
const SPAWN_INTERVAL:  f32 = 0.5;
#[derive(Component)]
struct Asteroid;
struct AsteroidPlugin;
#[derive(Resource)]
struct AsteroidSpawnState {
    timer: Timer,
}

impl Default for AsteroidSpawnState {
    fn default() -> Self {
        AsteroidSpawnState {
            timer: Timer::from_seconds(SPAWN_INTERVAL, Once),
        }
    }
}

impl Plugin for AsteroidPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, asteroid_movement)
            .add_systems(Update, spawn_asteroid)
            .insert_resource(AsteroidSpawnState::default());
    }
}

fn spawn_asteroid(
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut state: Local<AsteroidSpawnState>,
    mut commands: Commands,
) {
    let window = window_query.get_single().unwrap();

    if state.timer.tick(time.delta()).just_finished() {
        let random_x = rand::random::<f32>() * window.width();
        let initial_y = window.height();
        let random_y = rand::random::<f32>() * window.width() + initial_y;

        commands.spawn((
            SpriteBundle {
                transform: Transform::from_xyz(random_x, random_y, 0.0),
                texture: asset_server.load("sprites/asteroid.png"),
                ..default()
            },
            Asteroid,
        ));

        state.timer.reset();
    }
}

fn asteroid_movement(mut asteroid_query: Query<&mut Transform, With<Asteroid>>, time: Res<Time>) {
    for mut transform in asteroid_query.iter_mut() {
        let mut direction = Vec3::ZERO;

        direction.y -= 1.0;

        transform.translation += direction * ASTEROID_SPEED * time.delta_seconds();
    }
}

#[derive(Component)]
struct Collision;
struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, asteroid_hits_player);
    }
}

fn asteroid_hits_player(
    mut player_query: Query<(Entity, &Transform), With<Player>>,
    asteroid_query: Query<&Transform, With<Asteroid>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    if let Ok((player_entity, player_transform)) = player_query.get_single_mut() {
        for asteroid_transform in asteroid_query.iter() {
            let distance = player_transform
                .translation
                .distance(asteroid_transform.translation);

            if distance < PLAYER_SIZE {
                commands.spawn(AudioBundle {
                    source: asset_server.load("audio/explosion.ogg"),
                    settings: PlaybackSettings {
                        volume: Volume::new_absolute(0.3),
                        ..default()
                    },
                    ..default()
                });
                commands.entity(player_entity).despawn();
            }
        }
    }
}
