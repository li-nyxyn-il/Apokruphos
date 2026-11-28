//! A Bevy front end for the world model.
//!
//! Run it from the workspace folder with `cargo bevy`, or pass a data file:
//! `cargo bevy -- data/first_leg.ron`.
//!
//! Add `--edit` to open the NPC editor instead of the game:
//! `cargo bevy -- --edit [data file]`.

mod editor;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use std::{env, fs, path::Path, process};
use varif_core::{load_world_with_people, World as VarifWorld};

const DEFAULT_DATA: &str = "data/first_leg.ron";

/// The Varif world, held as a Bevy resource. Named apart from Bevy's own
/// [`World`] so the two do not collide.
#[derive(Resource)]
struct GameWorld(VarifWorld);

fn main() {
    let mut edit = false;
    let mut path = None;
    for argument in env::args().skip(1) {
        if argument == "--edit" {
            edit = true;
        } else if path.is_none() {
            path = Some(argument);
        }
    }
    let path = path.unwrap_or_else(|| DEFAULT_DATA.to_string());
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("Could not open {path}: {error}");
            process::exit(1);
        }
    };
    let people = match varif_core::read_people_dir(&Path::new(&path).with_file_name("people")) {
        Ok(people) => people,
        Err(error) => {
            eprintln!("Could not read the people files beside {path}: {error}");
            process::exit(1);
        }
    };
    let world = match load_world_with_people(&text, &people) {
        Ok(world) => world,
        Err(error) => {
            eprintln!("Could not load {path}: {error}");
            process::exit(1);
        }
    };

    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: if edit { "Varif editor" } else { "Varif" }.into(),
                    ..default()
                }),
                ..default()
            })
            // The workspace's own `assets/` folder, wherever the binary is run from.
            .set(AssetPlugin {
                file_path: concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets").into(),
                ..default()
            }),
    )
    .insert_resource(ClearColor(Color::srgb(0.02, 0.02, 0.06)))
    .insert_resource(GameWorld(world));
    if edit {
        app.add_plugins(EguiPlugin::default())
            .add_plugins(editor::EditorPlugin {
                data_path: path.into(),
            });
    } else {
        app.add_systems(Startup, setup);
    }
    app.run();
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
