//! A Bevy front end for the world model.
//!
//! Run it from the workspace folder with `cargo bevy`, or pass a data file:
//! `cargo bevy -- data/first_leg.ron`.

use bevy::prelude::*;
use std::{env, fs, process};
use varif_core::{load_world, World as VarifWorld};

const DEFAULT_DATA: &str = "data/first_leg.ron";

/// The Varif world, held as a Bevy resource. Named apart from Bevy's own
/// [`World`] so the two do not collide.
#[derive(Resource)]
struct GameWorld(VarifWorld);

fn main() {
    let path = env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_DATA.to_string());
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("Could not open {path}: {error}");
            process::exit(1);
        }
    };
    let world = match load_world(&text) {
        Ok(world) => world,
        Err(error) => {
            eprintln!("Could not load {path}: {error}");
            process::exit(1);
        }
    };

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Varif".into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.02, 0.02, 0.06)))
        .insert_resource(GameWorld(world))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, game: Res<GameWorld>) {
    commands.spawn(Camera2d);

    let majors: Vec<&str> = game.0.sky.majors().iter().map(|a| a.name()).collect();
    let minors: Vec<&str> = game.0.sky.minors().iter().map(|m| m.name()).collect();
    info!("Tonight the Akstada are {}.", majors.join(", "));
    info!("Among the lesser stars: {}.", minors.join(", "));

    commands.spawn((
        Text2d::new("Varif"),
        TextFont::from_font_size(64.0),
        TextColor(Color::srgb(0.85, 0.88, 0.95)),
        Transform::from_xyz(0.0, 40.0, 0.0),
    ));
    commands.spawn((
        Text2d::new(format!("Akstada: {}", majors.join(", "))),
        TextFont::from_font_size(22.0),
        TextColor(Color::srgb(0.65, 0.7, 0.85)),
        Transform::from_xyz(0.0, -20.0, 0.0),
    ));
}
