use std::time::Duration;

use bevy::prelude::*;
use rand::Rng;

#[derive(Component)]
struct Player;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()), // prevents blurry sprites
        ))
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_systems(Startup, setup)
        .insert_resource(AsteroidSpawnTimer(Timer::from_seconds(
            1.0,
            TimerMode::Once,
        )))
        .add_systems(
            Update,
            (
                sprite_movement,
                ship_movement_input,
                confine_player_to_screen,
                bullet_firing,
                spawn_asteroids,
                despawn_entities_outside_of_screen,
                asteroid_bullet_collision,
            ),
        )
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    // Spawn the spaceship
    commands.spawn((
        Player,
        SpriteBundle {
            texture: asset_server.load("spaceship.png"),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0))
                .with_scale(Vec3::splat(2.)),
            ..default()
        },
        SpriteMovement {
            direction: Vec3::splat(0.0),
            speed: 100.0,
        },
        CooldownTimer(Timer::from_seconds(0.2, TimerMode::Once)),
        BallCollider(18.0),
    ));
}

#[derive(Component)]
struct SpriteMovement {
    direction: Vec3,
    speed: f32,
}

fn sprite_movement(time: Res<Time>, mut sprite_position: Query<(&SpriteMovement, &mut Transform)>) {
    for (movement, mut transform) in &mut sprite_position {
        transform.translation +=
            movement.direction.clone().normalize_or_zero() * movement.speed * time.delta_seconds();
    }
}

fn ship_movement_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut player: Query<&mut SpriteMovement, With<Player>>,
) {
    let mut sprite_movement = player.single_mut();

    if keyboard_input.just_pressed(KeyCode::A) || keyboard_input.just_pressed(KeyCode::Left) {
        sprite_movement.direction.x = -1.0;
    } else if (keyboard_input.just_released(KeyCode::A)
        || keyboard_input.just_released(KeyCode::Left))
        && sprite_movement.direction.x < 0.0
    {
        sprite_movement.direction.x = 0.0;
    }

    if keyboard_input.just_pressed(KeyCode::D) || keyboard_input.just_pressed(KeyCode::Right) {
        sprite_movement.direction.x = 1.0;
    } else if (keyboard_input.just_released(KeyCode::D)
        || keyboard_input.just_released(KeyCode::Right))
        && sprite_movement.direction.x > 0.0
    {
        sprite_movement.direction.x = 0.0;
    }

    if keyboard_input.just_pressed(KeyCode::W) || keyboard_input.just_pressed(KeyCode::Up) {
        sprite_movement.direction.y = 1.0;
    } else if (keyboard_input.just_released(KeyCode::W)
        || keyboard_input.just_released(KeyCode::Up))
        && sprite_movement.direction.y > 0.0
    {
        sprite_movement.direction.y = 0.0;
    }

    if keyboard_input.just_pressed(KeyCode::S) || keyboard_input.just_pressed(KeyCode::Down) {
        sprite_movement.direction.y = -1.0;
    } else if (keyboard_input.just_released(KeyCode::S)
        || keyboard_input.just_released(KeyCode::Down))
        && sprite_movement.direction.y < 0.0
    {
        sprite_movement.direction.y = 0.0;
    }
}

fn confine_player_to_screen(
    mut player: Query<(&mut Transform, &mut SpriteMovement), With<Player>>,
    window: Query<&Window>,
) {
    let window = window.single();
    let half_width = window.resolution.width() / 2.0;
    let half_height = window.resolution.height() / 2.0;

    let (mut transform, mut movement) = player.single_mut();

    if transform.translation.x < -half_width && movement.direction.x < 0.0 {
        movement.direction.x = 0.0;
        transform.translation.x = -half_width;
    } else if transform.translation.x > half_width && movement.direction.x > 0.0 {
        movement.direction.x = 0.0;
        transform.translation.x = half_width;
    }

    if transform.translation.y < -half_height && movement.direction.y < 0.0 {
        movement.direction.y = 0.0;
        transform.translation.y = -half_height;
    } else if transform.translation.y > half_height && movement.direction.y > 0.0 {
        movement.direction.y = 0.0;
        transform.translation.y = half_height;
    }
}

#[derive(Component)]
struct Bullet;

#[derive(Component, Deref, DerefMut)]
struct CooldownTimer(Timer);

fn bullet_firing(
    mut cmd: Commands,
    mut player: Query<(&Transform, &mut CooldownTimer), With<Player>>,
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    asset_server: Res<AssetServer>,
) {
    let (translation, mut timer) = player.single_mut();
    timer.tick(time.delta());
    let position = translation.translation + Vec3::new(0.0, 30.0, 0.0);

    if keyboard_input.pressed(KeyCode::Space) && timer.finished() {
        cmd.spawn((
            Bullet,
            SpriteBundle {
                texture: asset_server.load("bullet.png"),
                transform: Transform::from_translation(position),
                ..default()
            },
            SpriteMovement {
                direction: Vec3::Y,
                speed: 200.0,
            },
            BallCollider(2.0),
        ));
        timer.reset();
    }
}

#[derive(Component)]
struct Asteroid;

#[derive(Resource, Deref, DerefMut)]
struct AsteroidSpawnTimer(Timer);

fn spawn_asteroids(
    mut cmd: Commands,
    window: Query<&Window>,
    time: Res<Time>,
    mut timer: ResMut<AsteroidSpawnTimer>,
    asset_server: Res<AssetServer>,
) {
    timer.tick(time.delta());

    if timer.finished() {
        let mut rng = rand::thread_rng();

        let window = window.single();
        let half_width = window.resolution.width() / 2.0;
        let half_height = window.resolution.height() / 2.0;

        // Spawn an asteroid
        cmd.spawn((
            Asteroid,
            SpriteBundle {
                texture: asset_server.load("asteroid.png"),
                transform: Transform::from_translation(Vec3::new(
                    rng.gen_range(-half_width..half_width),
                    half_height + 100.0, // Add buffer so that objects don't appear on screen from thin air
                    0.0,
                ))
                .with_scale(Vec3::splat(2.0)),
                ..default()
            },
            SpriteMovement {
                direction: Vec3::new(0.0, -1.0, 0.0),
                speed: 30.0,
            },
            BallCollider(24.0),
        ));
        timer.set_duration(Duration::from_millis(rng.gen_range(500..3000)));
        timer.reset();
    }
}

fn despawn_entities_outside_of_screen(
    mut cmd: Commands,
    window: Query<&Window>,
    query: Query<(Entity, &Transform), Or<(With<Asteroid>, With<Bullet>)>>,
) {
    let window = window.single();
    // Add buffer so that objects aren't despawned until they are completely off the screen
    let half_height = window.resolution.height() / 2.0 + 100.0;

    for (entity, transform) in &mut query.iter() {
        if transform.translation.y < -half_height || transform.translation.y > half_height {
            cmd.entity(entity).despawn();
        }
    }
}

#[derive(Component)]
struct BallCollider(f32);

fn asteroid_bullet_collision(
    mut commands: Commands,
    bullets: Query<(Entity, &Transform, &BallCollider), With<Bullet>>,
    asteroids: Query<(Entity, &Transform, &BallCollider), With<Asteroid>>,
) {
    for (bullet_entity, bullet_transform, bullet_collider) in &mut bullets.iter() {
        for (asteroid_entity, asteroid_transform, asteroid_collider) in &mut asteroids.iter() {
            if bullet_transform
                .translation
                .distance(asteroid_transform.translation)
                < bullet_collider.0 + asteroid_collider.0
            {
                commands.entity(bullet_entity).despawn();
                commands.entity(asteroid_entity).despawn();
            }
        }
    }
}
